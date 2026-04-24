//! GitHub Copilot CLI provider — **currently unsupported for code generation**.
//!
//! codex #4 finding: `gh copilot suggest` is an interactive command that proposes a
//! single shell command (or git command, etc.) and then prompts the user whether to run
//! it. It does not non-interactively produce file edits comparable to `claude -p` or
//! `codex exec`. Wiring it up as a coding provider would either hang the pipeline waiting
//! for interactive input or produce shell-command strings where we expect file
//! modifications.
//!
//! We therefore keep the provider registered in [`ProviderKind::Copilot`] (so flow YAMLs
//! and CLI args that mention `copilot` still parse cleanly) but `build_command` returns a
//! command that exits with an informative error. If/when GitHub ships a non-interactive
//! code-editing CLI, this module can be rewritten without touching callers.

use std::path::Path;

use super::{AgentProvider, ProviderKind, ProviderOptions};

pub(crate) struct CopilotProvider;

impl AgentProvider for CopilotProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Copilot
    }

    fn build_command(
        &self,
        _prompt: &str,
        working_dir: &Path,
        _options: &ProviderOptions,
    ) -> tokio::process::Command {
        // Emit a friendly error and exit non-zero. Bridge retry will propagate this as a
        // structured failure instead of hanging on interactive input.
        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-c").arg(
            "printf 'ccswarm: the `copilot` provider is not supported for code generation.\\n\
             `gh copilot suggest` is interactive and returns shell-command strings, not file edits.\\n\
             Use `claude` (default) or `codex` instead.\\n' >&2; exit 2",
        );
        cmd.current_dir(working_dir);
        cmd
    }
}
