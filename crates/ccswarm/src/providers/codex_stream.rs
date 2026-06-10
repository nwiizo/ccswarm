//! Parser for Codex CLI's `codex exec --json` JSONL output.
//!
//! Codex emits one JSON event per stdout line. Shapes verified against codex
//! CLI 0.139.0:
//!
//! ```text
//! {"type":"thread.started","thread_id":"<uuid>"}
//! {"type":"turn.started"}
//! {"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"..."}}
//! {"type":"turn.completed","usage":{"input_tokens":N,"cached_input_tokens":N,"output_tokens":N,...}}
//! {"type":"turn.failed","error":{"message":"..."}}
//! ```
//!
//! Unknown event types and malformed lines are skipped so the parser stays
//! forward-compatible, mirroring `claude_stream`.

/// Aggregated view of a single `codex exec --json` invocation.
#[derive(Debug, Default)]
pub(crate) struct CodexStreamSummary {
    /// Agent message text, concatenated across `item.completed` events.
    pub result_text: String,

    /// Thread ID assigned by Codex — the handle for `codex exec resume <id>`.
    pub thread_id: Option<String>,

    /// Non-message item types completed during the turn (e.g.
    /// `command_execution`), the closest analogue to Claude's tool names.
    pub tool_names: Vec<String>,

    /// Real token totals `(input, output)` from the last `turn.completed`
    /// usage record. Codex's `input_tokens` already includes cached tokens
    /// (`cached_input_tokens` is a breakdown subset, not an addend).
    pub tokens: Option<(u64, u64)>,

    /// Error message if the turn failed.
    pub failed: Option<String>,
}

/// Parse a full `codex exec --json` stdout buffer (JSONL). Lines that fail to
/// parse are skipped with a debug log rather than aborting the run.
pub(crate) fn parse_stream(stdout: &str) -> CodexStreamSummary {
    let mut summary = CodexStreamSummary::default();

    for raw in stdout.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }

        let value: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                tracing::debug!(
                    "codex-json: skipping malformed line ({} bytes): {}",
                    line.len(),
                    e
                );
                continue;
            }
        };

        match value.get("type").and_then(|v| v.as_str()) {
            Some("thread.started") => {
                if let Some(id) = value.get("thread_id").and_then(|v| v.as_str()) {
                    summary.thread_id = Some(id.to_string());
                }
            }
            Some("item.completed") => {
                let Some(item) = value.get("item") else {
                    continue;
                };
                match item.get("type").and_then(|v| v.as_str()) {
                    Some("agent_message") => {
                        if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                            if !summary.result_text.is_empty() {
                                summary.result_text.push('\n');
                            }
                            summary.result_text.push_str(text);
                        }
                    }
                    // Reasoning is internal narration, not a tool action.
                    Some("reasoning") | None => {}
                    Some(other) => summary.tool_names.push(other.to_string()),
                }
            }
            Some("turn.completed") => {
                if let Some(usage) = value.get("usage") {
                    let input = usage
                        .get("input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let output = usage
                        .get("output_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    summary.tokens = Some((input, output));
                }
            }
            Some("turn.failed") | Some("error") => {
                let message = value
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .or_else(|| value.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("codex turn failed")
                    .to_string();
                summary.failed = Some(message);
            }
            _ => {} // turn.started, item.started, unknown — ignored
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_text_thread_id_and_usage() {
        let stdout = r#"{"type":"thread.started","thread_id":"abc-123"}
{"type":"turn.started"}
{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"ok"}}
{"type":"turn.completed","usage":{"input_tokens":13648,"cached_input_tokens":4992,"output_tokens":17}}
"#;
        let s = parse_stream(stdout);
        assert_eq!(s.result_text, "ok");
        assert_eq!(s.thread_id.as_deref(), Some("abc-123"));
        assert_eq!(s.tokens, Some((13648, 17)));
        assert!(s.failed.is_none());
    }

    #[test]
    fn collects_non_message_items_as_tool_names() {
        let stdout = r#"{"type":"item.completed","item":{"id":"i1","type":"command_execution","command":"ls"}}
{"type":"item.completed","item":{"id":"i2","type":"reasoning"}}
{"type":"item.completed","item":{"id":"i3","type":"agent_message","text":"done"}}
"#;
        let s = parse_stream(stdout);
        assert_eq!(s.tool_names, vec!["command_execution"]);
        assert_eq!(s.result_text, "done");
    }

    #[test]
    fn captures_turn_failure() {
        let stdout = r#"{"type":"thread.started","thread_id":"t1"}
{"type":"turn.failed","error":{"message":"model not supported"}}
"#;
        let s = parse_stream(stdout);
        assert_eq!(s.failed.as_deref(), Some("model not supported"));
        assert_eq!(s.thread_id.as_deref(), Some("t1"));
    }

    #[test]
    fn skips_malformed_lines_without_aborting() {
        let stdout = "{\"type\":\"thread.started\",\"thread_id\":\"t2\"}\nnot json at all\n{\"type\":\"item.completed\",\"item\":{\"type\":\"agent_message\",\"text\":\"fine\"}}\n";
        let s = parse_stream(stdout);
        assert_eq!(s.thread_id.as_deref(), Some("t2"));
        assert_eq!(s.result_text, "fine");
    }

    #[test]
    fn handles_empty_stdout() {
        let s = parse_stream("");
        assert_eq!(s.result_text, "");
        assert!(s.thread_id.is_none());
        assert!(s.tokens.is_none());
    }
}
