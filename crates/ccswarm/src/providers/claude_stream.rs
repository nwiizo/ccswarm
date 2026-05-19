//! Parser for Claude Code's `--output-format stream-json --verbose` output.
//!
//! The CLI emits one JSON object per stdout line. The shapes ccswarm cares
//! about are documented in Claude Code's docs and verified empirically; we
//! decode just enough to recover the final answer text and surface tool /
//! usage events to the EventRecorder. Unknown line types are recorded as
//! `StreamEvent::Other` so the parser is forward-compatible.

use serde::Deserialize;

/// Aggregated view of a single Claude Code invocation in stream-json mode.
#[derive(Debug, Default)]
pub(crate) struct StreamSummary {
    /// Final text the model emitted, concatenated from `result.result` (when
    /// present) or assistant text content blocks (fallback).
    pub result_text: String,

    /// Tool invocations the model issued, in order. Useful for richer event
    /// telemetry than the previous text-only output.
    pub tool_uses: Vec<ToolUse>,

    /// Per-turn usage records, if the CLI reported them.
    pub usage: Vec<Usage>,

    /// The session UUID Claude assigned, if present. Useful for `--resume`.
    pub session_id: Option<String>,

    /// Total cost in USD if the result envelope reported it.
    pub total_cost_usd: Option<f64>,

    /// Subtype of the terminal `result` event (e.g. `success`, `error`,
    /// `error_max_turns`).
    pub result_subtype: Option<String>,
}

/// Parsed tool invocation.
#[derive(Debug, Clone)]
#[allow(dead_code)] // `input` is captured for v0.8.0 EventRecorder wiring; the bridge currently only logs `name`.
pub(crate) struct ToolUse {
    pub name: String,
    /// Raw input JSON; opaque to us but useful when logged.
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)] // Fields are populated for v0.8.0 EventRecorder wiring; until that lands the bridge only emits a debug log.
pub(crate) struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
}

/// Parse a full Claude Code stream-json stdout buffer (NDJSON). Lines that
/// fail to parse as JSON are skipped with a debug log rather than aborting
/// the whole run — surfacing one malformed line shouldn't lose the answer.
pub(crate) fn parse_stream(stdout: &str) -> StreamSummary {
    let mut summary = StreamSummary::default();

    for raw in stdout.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }

        let value: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                tracing::debug!(
                    "stream-json: skipping malformed line ({} bytes): {}",
                    line.len(),
                    e
                );
                continue;
            }
        };

        match value.get("type").and_then(|v| v.as_str()) {
            Some("system") => {
                if let Some(sid) = value.get("session_id").and_then(|v| v.as_str()) {
                    summary.session_id = Some(sid.to_string());
                }
            }
            Some("assistant") => {
                extract_assistant(&value, &mut summary);
            }
            Some("result") => {
                extract_result(&value, &mut summary);
            }
            _ => {} // user echo, tool_result blocks, unknown — ignored
        }
    }

    summary
}

fn extract_assistant(value: &serde_json::Value, summary: &mut StreamSummary) {
    // Shape: { type: "assistant", message: { content: [ {type, text|name|input, ...} ], usage: {...} } }
    let Some(message) = value.get("message") else {
        return;
    };

    if let Some(content) = message.get("content").and_then(|v| v.as_array()) {
        for block in content {
            match block.get("type").and_then(|v| v.as_str()) {
                Some("text") => {
                    if let Some(t) = block.get("text").and_then(|v| v.as_str()) {
                        // Concatenating per-turn text gives us the same payload
                        // a non-streaming run would expose as `result`. Newlines
                        // separate distinct content blocks to keep markdown
                        // structure readable.
                        if !summary.result_text.is_empty() {
                            summary.result_text.push('\n');
                        }
                        summary.result_text.push_str(t);
                    }
                }
                Some("tool_use") => {
                    let name = block
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let input = block
                        .get("input")
                        .cloned()
                        .unwrap_or(serde_json::Value::Null);
                    summary.tool_uses.push(ToolUse { name, input });
                }
                _ => {}
            }
        }
    }

    if let Some(usage) = message.get("usage") {
        summary.usage.push(parse_usage(usage));
    }
}

fn extract_result(value: &serde_json::Value, summary: &mut StreamSummary) {
    // Shape: { type: "result", subtype: "success", result: "...", total_cost_usd: ..., usage: {...} }
    if let Some(subtype) = value.get("subtype").and_then(|v| v.as_str()) {
        summary.result_subtype = Some(subtype.to_string());
    }
    if let Some(text) = value.get("result").and_then(|v| v.as_str()) {
        // `result` is authoritative when present — prefer it over the
        // per-assistant accumulation, since Claude Code may include final
        // formatting that the per-block text doesn't.
        summary.result_text = text.to_string();
    }
    if let Some(cost) = value.get("total_cost_usd").and_then(|v| v.as_f64()) {
        summary.total_cost_usd = Some(cost);
    }
    if let Some(usage) = value.get("usage") {
        summary.usage.push(parse_usage(usage));
    }
}

#[derive(Deserialize, Default)]
struct UsageRaw {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}

fn parse_usage(value: &serde_json::Value) -> Usage {
    let raw: UsageRaw = serde_json::from_value(value.clone()).unwrap_or_default();
    Usage {
        input_tokens: raw.input_tokens,
        output_tokens: raw.output_tokens,
        cache_creation_input_tokens: raw.cache_creation_input_tokens,
        cache_read_input_tokens: raw.cache_read_input_tokens,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_result_text_and_session_id() {
        let stdout = r#"{"type":"system","subtype":"init","session_id":"abc-123"}
{"type":"assistant","message":{"content":[{"type":"text","text":"Hello"}],"usage":{"input_tokens":10,"output_tokens":2}}}
{"type":"result","subtype":"success","result":"Final answer.","total_cost_usd":0.001,"usage":{"input_tokens":15,"output_tokens":4}}
"#;
        let s = parse_stream(stdout);
        assert_eq!(s.result_text, "Final answer.");
        assert_eq!(s.session_id.as_deref(), Some("abc-123"));
        assert_eq!(s.result_subtype.as_deref(), Some("success"));
        assert!((s.total_cost_usd.unwrap_or(0.0) - 0.001).abs() < 1e-9);
        assert_eq!(s.usage.len(), 2);
    }

    #[test]
    fn falls_back_to_assistant_text_when_no_result_block() {
        let stdout = r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Part one"},{"type":"text","text":"Part two"}]}}"#;
        let s = parse_stream(stdout);
        assert_eq!(s.result_text, "Part one\nPart two");
    }

    #[test]
    fn captures_tool_uses() {
        let stdout = r#"{"type":"assistant","message":{"content":[{"type":"tool_use","id":"t1","name":"Read","input":{"path":"/a.rs"}},{"type":"tool_use","id":"t2","name":"Bash","input":{"command":"ls"}}]}}"#;
        let s = parse_stream(stdout);
        assert_eq!(s.tool_uses.len(), 2);
        assert_eq!(s.tool_uses[0].name, "Read");
        assert_eq!(s.tool_uses[1].name, "Bash");
        assert_eq!(
            s.tool_uses[1].input.get("command").and_then(|v| v.as_str()),
            Some("ls")
        );
    }

    #[test]
    fn skips_malformed_lines_without_aborting() {
        let stdout = "{\"type\":\"system\",\"session_id\":\"ok\"}\nthis is not json\n{\"type\":\"result\",\"result\":\"done\"}\n";
        let s = parse_stream(stdout);
        assert_eq!(s.result_text, "done");
        assert_eq!(s.session_id.as_deref(), Some("ok"));
    }

    #[test]
    fn handles_empty_stdout() {
        let s = parse_stream("");
        assert_eq!(s.result_text, "");
        assert!(s.tool_uses.is_empty());
        assert!(s.usage.is_empty());
    }
}
