//! Agent provider abstraction — build subprocess commands for Claude / Codex / Copilot CLIs.
//!
//! The `AISessionBridge` owns context/persistence/parsing logic; providers only know how to
//! construct an executable command from a prompt and [`ProviderOptions`]. This keeps the
//! bridge neutral to which underlying CLI is spoken.

use std::path::Path;

/// Options passed to a provider for a single execution.
#[derive(Debug, Clone, Default)]
pub(crate) struct ProviderOptions {
    /// Tools the provider is allowed to use (provider-specific name mapping).
    pub allowed_tools: Vec<String>,
    /// Model override.
    pub model: Option<String>,
    /// System prompt (persona-derived).
    pub system_prompt: Option<String>,
    /// Agent definition name for `--agent` flag (Claude only).
    pub agent_name: Option<String>,
    /// Session identifier for resume.
    pub session_id: Option<String>,
    /// If true, attempt to continue the prior session (no explicit session id).
    pub continue_session: bool,
    /// Maximum budget in USD (Claude only).
    pub max_budget: Option<f64>,
    /// Worktree name (Claude only).
    pub worktree_name: Option<String>,
    /// Request Claude's streaming JSON output format (`--output-format
    /// stream-json --verbose`). The bridge parses each NDJSON line to extract
    /// the final result text and forward `tool_use` / `usage` events to the
    /// ccswarm EventRecorder. No effect on Codex / Copilot.
    ///
    /// Defaults to false in v0.7.0 so existing callers keep getting the simple
    /// text format. Toggled on by `AISessionBridge` when the environment sets
    /// `CCSWARM_CLAUDE_STREAM_JSON=1`.
    pub claude_stream_json: bool,
    /// Request Codex's JSONL event output (`codex exec --json`). The bridge
    /// parses each line to extract the final agent message, real token usage,
    /// and the thread ID needed for `codex exec resume`. No effect on Claude /
    /// Copilot. Toggled on by `AISessionBridge` when the environment sets
    /// `CCSWARM_CODEX_JSON=1` — and forced on during codex multi-turn runs,
    /// which need the thread ID to continue.
    pub codex_json: bool,
}

/// Provider support for continuing a stage in the same conversation thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SameThreadContinuation {
    Unsupported,
    /// Caller supplies the session ID up front (Claude `--session-id`).
    ExplicitSessionId,
    /// Provider assigns an ID on the first turn; the caller learns it from the
    /// provider's output and passes it back to resume (Codex `exec resume <id>`,
    /// learned from the `--json` thread.started event).
    ProviderAssignedId,
}

impl SameThreadContinuation {
    /// Whether multi-turn continuation is possible at all (by either model).
    pub(crate) fn supports_multi_turn(self) -> bool {
        !matches!(self, Self::Unsupported)
    }
}

/// Identifier for a provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderKind {
    Claude,
    Codex,
    Copilot,
}

impl ProviderKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "claude" | "claude-code" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            "copilot" | "gh-copilot" | "github-copilot" => Some(Self::Copilot),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Copilot => "copilot",
        }
    }
}

/// Contract every provider must implement to be callable from `AISessionBridge`.
pub(crate) trait AgentProvider {
    fn kind(&self) -> ProviderKind;

    fn same_thread_continuation(&self) -> SameThreadContinuation {
        SameThreadContinuation::Unsupported
    }

    /// Construct a `tokio::process::Command` ready to spawn.
    fn build_command(
        &self,
        prompt: &str,
        working_dir: &Path,
        options: &ProviderOptions,
    ) -> tokio::process::Command;
}

pub mod claude;
pub(crate) mod claude_stream;
pub mod codex;
pub(crate) mod codex_stream;
pub mod copilot;

#[cfg(test)]
mod tests;

/// Resolve a provider by kind.
pub(crate) fn resolve(kind: ProviderKind) -> Box<dyn AgentProvider + Send + Sync> {
    match kind {
        ProviderKind::Claude => Box::new(claude::ClaudeProvider),
        ProviderKind::Codex => Box::new(codex::CodexProvider),
        ProviderKind::Copilot => Box::new(copilot::CopilotProvider),
    }
}

/// Capitalize the first character; used for normalizing tool names across providers.
pub(crate) fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
