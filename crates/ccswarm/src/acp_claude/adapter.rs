//! CLI-based Claude Code adapter implementation
//!
//! This adapter executes `claude` CLI directly instead of WebSocket connection.

use super::{ACPError, ACPResult, ClaudeACPConfig};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, info};

/// CLI-based Claude Code adapter
pub struct SimplifiedClaudeAdapter {
    /// Current session ID
    session_id: Option<String>,

    /// Configuration
    config: ClaudeACPConfig,

    /// Connection status (always true for CLI mode)
    connected: bool,
}

impl SimplifiedClaudeAdapter {
    /// Create a new adapter with default configuration
    pub fn new(config: ClaudeACPConfig) -> Self {
        Self {
            session_id: None,
            config,
            connected: false,
        }
    }

    /// Connect (for CLI mode, this just verifies claude command exists)
    pub async fn connect_with_retry(&mut self) -> ACPResult<()> {
        self.connect().await
    }

    /// Connect (verify claude CLI is available)
    pub async fn connect(&mut self) -> ACPResult<()> {
        debug!("Verifying Claude CLI is available...");

        // Check if claude command exists
        let output = Command::new("claude")
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| ACPError::WebSocketError(format!("Claude CLI not found: {}", e)))?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("✅ Claude CLI found: {}", version.trim());

            // Generate session ID
            let session_id = format!("cli-session-{}", uuid::Uuid::new_v4());
            self.session_id = Some(session_id);
            self.connected = true;

            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ACPError::WebSocketError(format!(
                "Claude CLI error: {}",
                stderr
            )))
        }
    }

    /// Send a task to Claude Code via CLI
    pub async fn send_task(&self, task: &str) -> ACPResult<String> {
        if !self.connected {
            return Err(ACPError::NotConnected);
        }

        info!("📤 Executing task via Claude CLI: {}", task);

        let mut cmd = Command::new("claude");
        cmd.arg("-p")
            .arg(task)
            .arg("--output-format")
            .arg("text")
            .arg("--dangerously-skip-permissions")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = tokio::time::timeout(
            std::time::Duration::from_secs(self.config.timeout),
            cmd.output(),
        )
        .await
        .map_err(|_| ACPError::Timeout(self.config.timeout))?
        .map_err(|e| ACPError::WebSocketError(format!("Failed to execute: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            if stdout.trim().is_empty() {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                if !stderr.trim().is_empty() {
                    return Ok(stderr);
                }
            }
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(ACPError::WebSocketError(format!(
                "Claude CLI failed: {}",
                stderr
            )))
        }
    }

    /// Disconnect (no-op for CLI mode)
    pub async fn disconnect(&mut self) {
        self.session_id = None;
        self.connected = false;
        info!("👋 Claude CLI session ended");
    }

    /// Test connection by running a simple command
    pub async fn test_connection(&mut self) -> ACPResult<String> {
        self.connect().await?;

        // Run a simple test command
        let result = self
            .send_task("Say 'Hello from ccswarm!' and nothing else.")
            .await?;

        Ok(format!("Connection test successful: {}", result.trim()))
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get configuration
    pub fn config(&self) -> &ClaudeACPConfig {
        &self.config
    }

    /// Get session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}
