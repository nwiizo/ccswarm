//! Sangha consensus stage support.
//!
//! A `sangha:` stage asks multiple independent members to evaluate the same
//! task and returns a single consensus result. Unlike `team_leader`, no member
//! owns decomposition authority; the stage succeeds only when enough members
//! approve independently.

use serde::{Deserialize, Serialize};

use super::flow::{MovementPermission, Stage};

/// Stage-level consensus configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanghaSpec {
    /// Minimum number of approving members required for consensus.
    #[serde(default = "default_quorum")]
    pub quorum: u32,
    /// Members participating in the consensus round. Defaults to planner,
    /// reviewer, and QA perspectives.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<SanghaMember>,
    /// Permission level for member stages. Defaults to readonly because Sangha
    /// is a governance/decision phase, not concurrent editing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_permission: Option<MovementPermission>,
    /// Explicit tool list for member stages. Defaults to the parent tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_tools: Option<Vec<String>>,
    /// Per-member timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member_timeout_secs: Option<u32>,
}

fn default_quorum() -> u32 {
    2
}

/// One member of a Sangha consensus round.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanghaMember {
    /// Stable output key for this member.
    pub id: String,
    /// Persona facet used by this member.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub persona: Option<String>,
    /// Claude Code agent name, when using a provider that supports agent
    /// routing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
}

/// Consensus decision extracted from a member reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SanghaDecision {
    Approve,
    Revise,
    Abstain,
}

impl SanghaDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Revise => "revise",
            Self::Abstain => "abstain",
        }
    }
}

/// Effective members, using default perspectives when YAML omits them.
pub fn members_or_default(spec: &SanghaSpec) -> Vec<SanghaMember> {
    if !spec.members.is_empty() {
        return normalize_members(spec.members.clone());
    }

    normalize_members(vec![
        SanghaMember {
            id: "planner".to_string(),
            persona: Some("planner".to_string()),
            agent: None,
        },
        SanghaMember {
            id: "reviewer".to_string(),
            persona: Some("reviewer".to_string()),
            agent: None,
        },
        SanghaMember {
            id: "qa".to_string(),
            persona: Some("qa".to_string()),
            agent: None,
        },
    ])
}

fn normalize_members(mut members: Vec<SanghaMember>) -> Vec<SanghaMember> {
    for (i, member) in members.iter_mut().enumerate() {
        if member.id.trim().is_empty() {
            member.id = format!("member-{}", i + 1);
        }
    }

    let mut seen = std::collections::HashSet::new();
    for (i, member) in members.iter_mut().enumerate() {
        if !seen.insert(member.id.clone()) {
            member.id = format!("{}-{}", member.id, i + 1);
            seen.insert(member.id.clone());
        }
    }

    members
}

/// Build the prompt for one Sangha member.
pub fn consensus_prompt(task_instruction: &str, member_id: &str, quorum: u32) -> String {
    format!(
        "You are Sangha member '{member_id}'. Evaluate the decision independently; \
         no single member is the leader.\n\n# Decision Subject\n{task_instruction}\n\n\
         # Consensus Contract\n\
         - State the concrete recommendation you support.\n\
         - Call out risks, missing information, or required changes.\n\
         - End your response with exactly one final line:\n\
           SANGHA_DECISION=APPROVE\n\
           or\n\
           SANGHA_DECISION=REVISE\n\n\
         Consensus requires at least {quorum} independent APPROVE decisions."
    )
}

/// Synthesize a readonly member stage from a parent `sangha:` stage.
pub fn member_stage(parent: &Stage, spec: &SanghaSpec, member: &SanghaMember) -> Stage {
    Stage {
        id: member.id.clone(),
        persona: member.persona.clone().or_else(|| parent.persona.clone()),
        policy: parent.policy.clone(),
        knowledge: parent.knowledge.clone(),
        provider: parent.provider.clone(),
        model: parent.model.clone(),
        instruction: consensus_prompt(&parent.instruction, &member.id, spec.quorum.max(1)),
        tools: spec
            .member_tools
            .clone()
            .unwrap_or_else(|| parent.tools.clone()),
        permission: spec
            .member_permission
            .clone()
            .unwrap_or(MovementPermission::Readonly),
        rules: Vec::new(),
        parallel: false,
        sub_movements: Vec::new(),
        output_contract: None,
        timeout: spec.member_timeout_secs.or(parent.timeout),
        max_retries: parent.max_retries,
        agent: member.agent.clone().or_else(|| parent.agent.clone()),
        working_dir: parent.working_dir.clone(),
        retry_delay_ms: parent.retry_delay_ms,
        pass_previous_response: parent.pass_previous_response,
        call: None,
        promotion: Vec::new(),
        gates: Vec::new(),
        team_leader: None,
        sangha: None,
    }
}

/// Extract the explicit Sangha decision from a member reply.
pub fn extract_decision(text: &str) -> SanghaDecision {
    let final_line = text
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase();
    match final_line.as_str() {
        "SANGHA_DECISION=APPROVE" => SanghaDecision::Approve,
        "SANGHA_DECISION=REVISE" => SanghaDecision::Revise,
        _ => SanghaDecision::Abstain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_three_distinct_members() {
        let spec = SanghaSpec {
            quorum: 2,
            members: Vec::new(),
            member_permission: None,
            member_tools: None,
            member_timeout_secs: None,
        };

        let members = members_or_default(&spec);

        assert_eq!(members.len(), 3);
        assert_eq!(members[0].id, "planner");
        assert_eq!(members[1].persona.as_deref(), Some("reviewer"));
    }

    #[test]
    fn normalizes_duplicate_and_empty_member_ids() {
        let spec = SanghaSpec {
            quorum: 1,
            members: vec![
                SanghaMember {
                    id: "a".to_string(),
                    persona: None,
                    agent: None,
                },
                SanghaMember {
                    id: "a".to_string(),
                    persona: None,
                    agent: None,
                },
                SanghaMember {
                    id: String::new(),
                    persona: None,
                    agent: None,
                },
            ],
            member_permission: None,
            member_tools: None,
            member_timeout_secs: None,
        };

        let ids = members_or_default(&spec)
            .into_iter()
            .map(|member| member.id)
            .collect::<Vec<_>>();

        assert_eq!(ids, vec!["a", "a-2", "member-3"]);
    }

    #[test]
    fn extracts_decision_from_final_contract_line() {
        assert_eq!(
            extract_decision("Looks good\nSANGHA_DECISION=APPROVE"),
            SanghaDecision::Approve
        );
        assert_eq!(
            extract_decision("Needs tests\nsangha_decision=revise"),
            SanghaDecision::Revise
        );
        assert_eq!(
            extract_decision("SANGHA_DECISION=APPROVE\nor\nSANGHA_DECISION=REVISE\nMore text"),
            SanghaDecision::Abstain
        );
        assert_eq!(
            extract_decision("No explicit vote"),
            SanghaDecision::Abstain
        );
    }
}
