//! Agent provider abstraction for Claude and Codex subprocesses.
//!
//! Copilot configuration names remain registered for compatibility, but the
//! provider fails fast because its interactive CLI cannot perform code edits.
//!
//! The `A2ABridge` owns routing, retry, and parsing; providers only know how to
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
    /// Maximum budget in USD (Claude only).
    pub max_budget: Option<f64>,
    /// Worktree name (Claude only).
    pub worktree_name: Option<String>,
    /// Request Claude stream-json output (`CCSWARM_CLAUDE_STREAM_JSON=1`) for
    /// tool names, real token usage, and cost. Claude only.
    pub claude_stream_json: bool,
    /// Request Codex JSONL output (`CCSWARM_CODEX_JSON=1`) for real token
    /// usage. Codex only.
    pub codex_json: bool,
    /// Workspace access the stage requested. Codex maps it to `--sandbox`;
    /// Claude does not enforce it.
    pub workspace_access: WorkspaceAccess,
}

/// Workspace access a stage requests from its provider.
///
/// Flow stages set it from their `permission` level; the stage's tool list
/// does not change it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum WorkspaceAccess {
    /// The provider may modify the working directory.
    #[default]
    Write,
    /// The provider must not modify the working directory.
    ReadOnly,
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

    /// Whether the provider CLI enforces [`WorkspaceAccess::ReadOnly`] with
    /// its own sandbox. Claude runs with `--dangerously-skip-permissions`, so
    /// its allowed-tools list is not a sandbox.
    pub fn enforces_read_only(&self) -> bool {
        matches!(self, Self::Codex)
    }
}

/// Contract every provider must implement to be callable from `A2ABridge`.
pub(crate) trait AgentProvider {
    fn kind(&self) -> ProviderKind;

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
