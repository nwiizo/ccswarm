use anyhow::{Context, Result};
use std::path::Path;
use std::time::Instant;
use tokio::process::Command;

pub const DEFAULT_MAX_PROMPT_BYTES: usize = 200_000;

#[derive(Debug, Clone)]
pub struct CommandExecution {
    pub stdout: String,
    pub stderr: String,
    pub status: std::process::ExitStatus,
    pub duration_ms: u64,
}

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

pub async fn run_provider_command(
    mut command: Command,
    working_dir: &Path,
    provider_name: &str,
) -> Result<CommandExecution> {
    command.current_dir(working_dir);
    let start = Instant::now();
    let output = command
        .output()
        .await
        .with_context(|| format!("Failed to execute provider CLI: {provider_name}"))?;

    Ok(CommandExecution {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        status: output.status,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}
