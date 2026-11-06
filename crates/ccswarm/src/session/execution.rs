use anyhow::Result;
use std::path::Path;
use tokio::process::Command;

pub use crate::session::herdr::CommandExecution;
use crate::session::herdr::{HerdrCall, HerdrContext};

pub const DEFAULT_MAX_PROMPT_BYTES: usize = 200_000;

pub fn prepare_provider_prompt(
    prompt: &str,
    working_dir: &Path,
    max_prompt_bytes: usize,
) -> Result<String> {
    if prompt.len() > max_prompt_bytes {
        anyhow::bail!(
            "prompt is {} bytes (cap is {}) - shorten the task description or split it",
            prompt.len(),
            max_prompt_bytes
        );
    }

    Ok(format!(
        "# Working directory\n{}\nUse this directory as the current working directory. Prefer relative paths anchored here.\n\n# Task\n{}",
        working_dir.display(),
        prompt
    ))
}

/// Run a non-interactive provider command in a background tab of the current
/// Herdr session.
///
/// Local provider execution requires a Herdr-managed pane; see
/// [`HerdrContext::from_env`].
pub async fn run_provider_command(
    command: Command,
    working_dir: &Path,
    call: &HerdrCall,
) -> Result<CommandExecution> {
    HerdrContext::from_env()?
        .run_headless(&command, working_dir, call)
        .await
}
