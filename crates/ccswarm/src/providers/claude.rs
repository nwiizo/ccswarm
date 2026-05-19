//! Claude Code CLI provider (`claude` binary).

use std::path::Path;

use super::{
    AgentProvider, ProviderKind, ProviderOptions, SameThreadContinuation, capitalize_first,
};

pub(crate) struct ClaudeProvider;

impl AgentProvider for ClaudeProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Claude
    }

    fn same_thread_continuation(&self) -> SameThreadContinuation {
        SameThreadContinuation::ExplicitSessionId
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

        if let Some(sid) = &options.session_id {
            cmd.args(["--session-id", sid]);
        } else if options.continue_session {
            cmd.arg("--continue");
        }

        if !options.allowed_tools.is_empty() {
            let tools_str = options
                .allowed_tools
                .iter()
                .map(|t| capitalize_first(t))
                .collect::<Vec<_>>()
                .join(",");
            cmd.args(["--allowed-tools", &tools_str]);
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
