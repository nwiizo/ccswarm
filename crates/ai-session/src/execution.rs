//! Provider-neutral execution helpers for AI sessions.
//!
//! Workflow crates can still decide which provider CLI to run, but the common
//! session concerns live here: prompt sizing, working-directory context, cwd
//! enforcement, and structured subprocess results.

use anyhow::{Context, Result};
use std::path::Path;
use std::time::Instant;
use tokio::process::Command;

/// Default prompt byte cap before spawning a provider subprocess.
pub const DEFAULT_MAX_PROMPT_BYTES: usize = 200_000;

/// Result of a single provider subprocess execution.
#[derive(Debug, Clone)]
pub struct CommandExecution {
    /// Captured stdout as UTF-8 lossy text.
    pub stdout: String,
    /// Captured stderr as UTF-8 lossy text.
    pub stderr: String,
    /// Process exit status.
    pub status: std::process::ExitStatus,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
}

impl CommandExecution {
    /// Whether the provider subprocess exited successfully.
    pub fn success(&self) -> bool {
        self.status.success()
    }
}

/// Validate and annotate a provider prompt with the intended working directory.
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

/// Execute a provider command with a centrally enforced working directory.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn prepare_provider_prompt_adds_working_directory_context() {
        let prompt = prepare_provider_prompt("Build it", Path::new("/tmp/project"), 200_000)
            .expect("prompt should fit");

        assert!(prompt.contains("# Working directory\n/tmp/project"));
        assert!(prompt.contains("# Task\nBuild it"));
    }

    #[test]
    fn prepare_provider_prompt_rejects_large_prompt() {
        let err = prepare_provider_prompt("abcdef", Path::new("."), 5)
            .expect_err("prompt should exceed cap");

        assert!(err.to_string().contains("prompt is 6 bytes"));
    }

    #[tokio::test]
    async fn run_provider_command_enforces_working_directory() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut command = Command::new("pwd");
        command.current_dir(PathBuf::from("/"));

        let result = run_provider_command(command, dir.path(), "pwd")
            .await
            .expect("command should run");

        let actual = std::fs::canonicalize(result.stdout.trim()).expect("actual cwd exists");
        let expected = std::fs::canonicalize(dir.path()).expect("expected cwd exists");

        assert!(result.success());
        assert_eq!(actual, expected);
    }
}
