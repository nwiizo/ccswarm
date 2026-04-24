//! Codex CLI provider (`codex` binary from openai/codex).
//!
//! Codex CLI is non-interactive via `codex exec "<prompt>"`. System prompt is prepended to
//! the user prompt because Codex has no dedicated `--system-prompt` flag.

use std::path::Path;

use super::{AgentProvider, ProviderKind, ProviderOptions};

pub(crate) struct CodexProvider;

impl AgentProvider for CodexProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Codex
    }

    fn build_command(
        &self,
        prompt: &str,
        working_dir: &Path,
        options: &ProviderOptions,
    ) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new("codex");
        cmd.arg("exec");

        // Non-interactive pipeline execution:
        //   1. `--sandbox workspace-write` lets Codex write inside the cwd the way
        //      `claude --dangerously-skip-permissions` does. The blunter
        //      `--dangerously-bypass-approvals-and-sandbox` still observes some
        //      read-only policies in practice (observed 2026-04), so prefer the
        //      explicit policy flag.
        //   2. `--skip-git-repo-check` lets the pipeline run in scratch dirs.
        cmd.args(["--sandbox", "workspace-write", "--skip-git-repo-check"]);

        if let Some(model) = &options.model {
            cmd.args(["--model", model]);
        }

        // NOTE: `codex exec` has no `--session` flag; resumption goes through a separate
        // `codex exec resume` subcommand that takes a different argv shape. We therefore
        // drop `session_id` silently for now — this is better than sending a flag Codex
        // will reject. Proper resume support is tracked as follow-up work.
        let _ = &options.session_id;

        // Codex has no allow-list, worktree, or budget flag. Unsupported options are
        // silently ignored so the same flow YAML stays portable across providers.

        // Codex expects the prompt as the final positional argument (after flags).
        let merged_prompt = match &options.system_prompt {
            Some(sys) if !sys.is_empty() => format!("{sys}\n\n---\n\n{prompt}"),
            _ => prompt.to_string(),
        };
        cmd.arg(&merged_prompt);

        cmd.current_dir(working_dir);
        cmd
    }
}
