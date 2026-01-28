//! Process management for AI sessions

use anyhow::Result;
use std::process::Stdio;
use tokio::process::{Child, Command};

/// Handle to a managed process
pub struct ProcessHandle {
    /// The underlying child process
    child: Child,
    /// Process ID
    pid: u32,
}

impl ProcessHandle {
    /// Create a new process
    pub async fn spawn(
        command: &str,
        args: &[String],
        working_dir: &std::path::Path,
        env: &[(String, String)],
    ) -> Result<Self> {
        let mut cmd = Command::new(command);

        cmd.args(args)
            .current_dir(working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (key, value) in env {
            cmd.env(key, value);
        }

        let child = cmd.spawn()?;
        let pid = child
            .id()
            .ok_or_else(|| anyhow::anyhow!("Failed to get process ID"))?;

        Ok(Self { child, pid })
    }

    /// Get the process ID
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Check if the process is still running
    pub async fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Kill the process
    pub async fn kill(&mut self) -> Result<()> {
        self.child.kill().await?;
        Ok(())
    }

    /// Wait for the process to exit
    pub async fn wait(&mut self) -> Result<std::process::ExitStatus> {
        Ok(self.child.wait().await?)
    }
}
