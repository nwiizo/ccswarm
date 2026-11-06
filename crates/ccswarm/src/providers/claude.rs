//! Claude Code CLI provider (`claude` binary).

use std::path::Path;

use super::{AgentProvider, ProviderKind, ProviderOptions, capitalize_first};

pub(crate) struct ClaudeProvider;

/// Value for Claude's `--allowed-tools`, or `None` when no tools are listed.
pub(crate) fn allowed_tools_arg(tools: &[String]) -> Option<String> {
    (!tools.is_empty()).then(|| {
        tools
            .iter()
            .map(|t| capitalize_first(t))
            .collect::<Vec<_>>()
            .join(",")
    })
}

impl AgentProvider for ClaudeProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Claude
    }

    fn build_command(
        &self,
        prompt: &str,
        working_dir: &Path,
        options: &ProviderOptions,
    ) -> tokio::process::Command {
        let mut cmd = tokio::process::Command::new("claude");
        if options.claude_stream_json {
            // stream-json requires --verbose to actually emit per-event lines;
            // without it, Claude Code only prints the final result envelope.
            cmd.args(["-p", prompt, "--output-format", "stream-json", "--verbose"]);
        } else {
            cmd.args(["-p", prompt, "--output-format", "text"]);
        }
        cmd.arg("--dangerously-skip-permissions");

        if let Some(name) = &options.agent_name {
            cmd.args(["--agent", name]);
        }

        if let Some(tools) = allowed_tools_arg(&options.allowed_tools) {
            cmd.args(["--allowed-tools", &tools]);
        }

        if let Some(model) = &options.model {
            cmd.args(["--model", model]);
        }

        if let Some(sys) = &options.system_prompt {
            // Use --append-system-prompt instead of --system-prompt: facet personas
            // are role *additions*, not replacements. --system-prompt would discard
            // Claude Code's default system prompt (CLAUDE.md auto-load, tool guidance,
            // etc.) and degrade behavior; --append-system-prompt preserves those.
            cmd.args(["--append-system-prompt", sys]);
        }

        if let Some(budget) = options.max_budget {
            cmd.args(["--max-budget-usd", &budget.to_string()]);
        }

        if let Some(wt) = &options.worktree_name {
            cmd.args(["--worktree", wt]);
        }

        cmd.current_dir(working_dir);
        cmd
    }
}
