//! Orchestrator-worker decomposition (takt's `team_leader`).
//!
//! A stage marked `team_leader:` runs in two phases: a *leader* call asks the
//! model to split the stage's task into up to `max_parts` independent parts
//! (returned as JSON), then the parts execute concurrently as synthesized
//! worker stages and their outputs are aggregated back into the parallel
//! output shape — so existing `all()`/`any()` rule aggregation works on the
//! results unchanged.
//!
//! This module holds the pure logic (prompt construction, JSON extraction,
//! aggregation); the engine wiring lives in `flow.rs::execute_team_leader`.

use serde::{Deserialize, Serialize};

use super::flow::{MovementPermission, Stage};

/// Stage-level orchestrator-worker configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamLeaderSpec {
    /// Maximum number of parts the leader may produce (default 3).
    #[serde(default = "default_max_parts")]
    pub max_parts: u32,
    /// Persona for worker stages (defaults to the parent stage's persona).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part_persona: Option<String>,
    /// Permission level for worker stages (default: the parent's).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part_permission: Option<MovementPermission>,
    /// Explicit tool list for worker stages (default: the parent's).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part_tools: Option<Vec<String>>,
    /// Per-part timeout in seconds (default: the parent's `timeout`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part_timeout_secs: Option<u32>,
}

fn default_max_parts() -> u32 {
    3
}

/// One part of a decomposed task, as returned by the leader.
#[derive(Debug, Clone, Deserialize)]
pub struct TaskPart {
    pub id: String,
    #[serde(default)]
    pub title: String,
    pub instruction: String,
}

/// Build the decomposition prompt for the leader call.
pub fn decomposition_prompt(task_instruction: &str, max_parts: u32) -> String {
    format!(
        "You are a team leader decomposing a task for parallel execution by \
         independent workers.\n\n# Task\n{task}\n\n# Instructions\n\
         Split the task into at most {max} independent parts that can run \
         concurrently without coordinating with each other. Fewer parts are \
         fine when the task doesn't divide cleanly; use a single part if it \
         is indivisible.\n\n\
         Respond with ONLY a JSON array, no prose:\n\
         [{{\"id\": \"part-1\", \"title\": \"short title\", \"instruction\": \
         \"complete, self-contained instructions for this worker\"}}]\n\n\
         Each part's instruction must stand alone — workers do not see the \
         other parts or this conversation.",
        task = task_instruction,
        max = max_parts.max(1)
    )
}

/// Extract the JSON array of parts from a leader reply. Tolerates markdown
/// code fences and surrounding prose by scanning for the outermost `[...]`.
/// Caps the result at `max_parts` and drops parts with empty instructions.
pub fn parse_parts(reply: &str, max_parts: u32) -> Result<Vec<TaskPart>, String> {
    let json_slice = extract_json_array(reply)
        .ok_or_else(|| "no JSON array found in leader reply".to_string())?;

    let parts: Vec<TaskPart> = serde_json::from_str(json_slice)
        .map_err(|e| format!("leader reply is not a valid part array: {e}"))?;

    let mut parts: Vec<TaskPart> = parts
        .into_iter()
        .filter(|p| !p.instruction.trim().is_empty())
        .collect();
    if parts.is_empty() {
        return Err("leader returned no usable parts".to_string());
    }
    parts.truncate(max_parts.max(1) as usize);

    // IDs become output-map keys and report filenames; normalize duplicates
    // and empties defensively rather than failing the whole decomposition.
    for (i, part) in parts.iter_mut().enumerate() {
        if part.id.trim().is_empty() {
            part.id = format!("part-{}", i + 1);
        }
    }
    let mut seen = std::collections::HashSet::new();
    for (i, part) in parts.iter_mut().enumerate() {
        if !seen.insert(part.id.clone()) {
            part.id = format!("{}-{}", part.id, i + 1);
            seen.insert(part.id.clone());
        }
    }

    Ok(parts)
}

/// Find the outermost JSON array in a reply (handles ```json fences and prose).
fn extract_json_array(reply: &str) -> Option<&str> {
    let start = reply.find('[')?;
    let end = reply.rfind(']')?;
    (end > start).then(|| &reply[start..=end])
}

/// Synthesize an in-memory worker `Stage` for one part. Workers inherit the
/// parent's provider/model/agent/working_dir; persona/permission/tools come
/// from the spec when set. Workers carry no rules (their routing happens at
/// the parent), no gates, and no nested team_leader/call/parallel.
pub fn worker_stage(parent: &Stage, spec: &TeamLeaderSpec, part: &TaskPart) -> Stage {
    Stage {
        id: part.id.clone(),
        persona: spec.part_persona.clone().or_else(|| parent.persona.clone()),
        policy: parent.policy.clone(),
        knowledge: parent.knowledge.clone(),
        provider: parent.provider.clone(),
        model: parent.model.clone(),
        instruction: if part.title.is_empty() {
            part.instruction.clone()
        } else {
            format!("# {}\n\n{}", part.title, part.instruction)
        },
        tools: spec
            .part_tools
            .clone()
            .unwrap_or_else(|| parent.tools.clone()),
        permission: spec
            .part_permission
            .clone()
            .unwrap_or_else(|| parent.permission.clone()),
        rules: Vec::new(),
        parallel: false,
        sub_movements: Vec::new(),
        output_contract: None,
        timeout: spec.part_timeout_secs.or(parent.timeout),
        max_retries: parent.max_retries,
        agent: parent.agent.clone(),
        working_dir: parent.working_dir.clone(),
        retry_delay_ms: parent.retry_delay_ms,
        pass_previous_response: parent.pass_previous_response,
        call: None,
        promotion: Vec::new(),
        gates: Vec::new(),
        team_leader: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_parts_handles_plain_json() {
        let reply = r#"[{"id":"a","title":"API","instruction":"build the API"},{"id":"b","title":"UI","instruction":"build the UI"}]"#;
        let parts = parse_parts(reply, 3).expect("parse");
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].id, "a");
        assert_eq!(parts[1].instruction, "build the UI");
    }

    #[test]
    fn parse_parts_handles_markdown_fence_and_prose() {
        let reply = "Here is the decomposition:\n```json\n[\n  {\"id\": \"backend\", \"title\": \"\", \"instruction\": \"implement endpoints\"}\n]\n```\nGood luck!";
        let parts = parse_parts(reply, 3).expect("parse");
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].id, "backend");
    }

    #[test]
    fn parse_parts_caps_at_max_and_normalizes_ids() {
        let reply = r#"[
            {"id":"x","instruction":"one"},
            {"id":"x","instruction":"two"},
            {"id":"","instruction":"three"},
            {"id":"y","instruction":"four"}
        ]"#;
        let parts = parse_parts(reply, 3).expect("parse");
        assert_eq!(parts.len(), 3, "capped at max_parts");
        let ids: Vec<&str> = parts.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids[0], "x");
        assert_ne!(ids[1], "x", "duplicate id must be renamed");
        assert_eq!(ids[2], "part-3", "empty id must be filled");
    }

    #[test]
    fn parse_parts_rejects_garbage() {
        assert!(parse_parts("no json here", 3).is_err());
        assert!(parse_parts("[]", 3).is_err());
        assert!(parse_parts(r#"[{"id":"a","instruction":"  "}]"#, 3).is_err());
        assert!(parse_parts("[{not json]", 3).is_err());
    }

    #[test]
    fn worker_stage_inherits_and_overrides() {
        let parent = Stage {
            id: "build".to_string(),
            persona: Some("coder".to_string()),
            policy: Some("coding".to_string()),
            knowledge: None,
            provider: Some("codex".to_string()),
            model: Some("gpt-5".to_string()),
            instruction: "parent instruction".to_string(),
            tools: vec!["Read".to_string()],
            permission: MovementPermission::Edit,
            rules: Vec::new(),
            parallel: false,
            sub_movements: Vec::new(),
            output_contract: None,
            timeout: Some(600),
            max_retries: 1,
            agent: None,
            working_dir: None,
            retry_delay_ms: 1000,
            pass_previous_response: true,
            call: None,
            promotion: Vec::new(),
            gates: Vec::new(),
            team_leader: None,
        };
        let spec = TeamLeaderSpec {
            max_parts: 3,
            part_persona: Some("backend-specialist".to_string()),
            part_permission: None,
            part_tools: None,
            part_timeout_secs: Some(120),
        };
        let part = TaskPart {
            id: "api".to_string(),
            title: "API layer".to_string(),
            instruction: "implement the API".to_string(),
        };

        let worker = worker_stage(&parent, &spec, &part);
        assert_eq!(worker.id, "api");
        assert_eq!(worker.persona.as_deref(), Some("backend-specialist"));
        assert_eq!(worker.provider.as_deref(), Some("codex"));
        assert_eq!(worker.timeout, Some(120));
        assert!(worker.instruction.contains("API layer"));
        assert!(worker.instruction.contains("implement the API"));
        assert!(worker.rules.is_empty());
        assert!(worker.gates.is_empty());
    }
}
