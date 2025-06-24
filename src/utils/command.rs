//! Command execution utilities

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

/// Utility for executing shell commands with consistent error handling
pub struct CommandExecutor;

impl CommandExecutor {
    /// Run a command with arguments
    pub async fn run(cmd: &str, args: &[&str], dir: Option<&Path>) -> Result<String> {
        let mut command = Command::new(cmd);
        command.args(args);
        
        if let Some(dir) = dir {
            command.current_dir(dir);
        }
        
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        
        let output = command
            .output()
            .await
            .context(format!("Failed to execute command: {} {}", cmd, args.join(" ")))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "Command failed: {} {}\nStderr: {}",
                cmd,
                args.join(" "),
                stderr
            );
        }
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    /// Run a command with timeout
    pub async fn run_with_timeout(
        cmd: &str,
        args: &[&str],
        dir: Option<&Path>,
        timeout: Duration,
    ) -> Result<String> {
        tokio::time::timeout(timeout, Self::run(cmd, args, dir))
            .await
            .context("Command timed out")?
    }
    
    /// Run a command and capture both stdout and stderr
    pub async fn run_with_output(
        cmd: &str,
        args: &[&str],
        dir: Option<&Path>,
    ) -> Result<(String, String)> {
        let mut command = Command::new(cmd);
        command.args(args);
        
        if let Some(dir) = dir {
            command.current_dir(dir);
        }
        
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        
        let output = command
            .output()
            .await
            .context(format!("Failed to execute command: {} {}", cmd, args.join(" ")))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        if !output.status.success() {
            anyhow::bail!(
                "Command failed: {} {}\nStdout: {}\nStderr: {}",
                cmd,
                args.join(" "),
                stdout,
                stderr
            );
        }
        
        Ok((stdout, stderr))
    }
    
    /// Check if a command exists in PATH
    pub async fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .await
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}