//! Integration layers for compatibility with existing systems

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::process::Command;

use crate::core::{SessionConfig, SessionId};

/// Tmux compatibility layer for migration support
pub struct TmuxCompatLayer {
    /// Tmux command path
    tmux_path: String,
    /// Session name prefix
    session_prefix: String,
}

impl TmuxCompatLayer {
    /// Create new tmux compatibility layer
    pub fn new() -> Self {
        Self {
            tmux_path: "tmux".to_string(),
            session_prefix: "ai-session".to_string(),
        }
    }

    /// Convert AI session to tmux session
    pub async fn create_tmux_session(
        &self,
        session_id: &SessionId,
        config: &SessionConfig,
    ) -> Result<String> {
        let tmux_name = format!(
            "{}-{}",
            self.session_prefix,
            session_id
                .to_string()
                .split('-')
                .next()
                .unwrap_or("unknown")
        );

        let mut cmd = Command::new(&self.tmux_path);
        cmd.args(&["new-session", "-d", "-s", &tmux_name]);

        if let Some(shell) = &config.shell_command {
            cmd.arg("-c")
                .arg(&config.working_directory.display().to_string());
            cmd.arg(shell);
        }

        cmd.output()
            .await
            .context("Failed to create tmux session")?;

        Ok(tmux_name)
    }

    /// List existing tmux sessions
    pub async fn list_tmux_sessions(&self) -> Result<Vec<TmuxSession>> {
        let output = Command::new(&self.tmux_path)
            .args(&[
                "list-sessions",
                "-F",
                "#{session_name}:#{session_created}:#{session_attached}",
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let mut sessions = Vec::new();
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                sessions.push(TmuxSession {
                    name: parts[0].to_string(),
                    created: parts[1].parse().unwrap_or(0),
                    attached: parts[2] == "1",
                });
            }
        }

        Ok(sessions)
    }

    /// Send command to tmux session
    pub async fn send_command(&self, session_name: &str, command: &str) -> Result<()> {
        Command::new(&self.tmux_path)
            .args(&["send-keys", "-t", session_name, command, "Enter"])
            .output()
            .await
            .context("Failed to send command to tmux")?;

        Ok(())
    }

    /// Capture tmux pane output
    pub async fn capture_output(&self, session_name: &str, lines: Option<usize>) -> Result<String> {
        let mut args = vec!["capture-pane", "-t", session_name, "-p"];

        let line_arg;
        if let Some(n) = lines {
            args.push("-S");
            line_arg = format!("-{}", n);
            args.push(&line_arg);
        }

        let output = Command::new(&self.tmux_path).args(&args).output().await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Kill tmux session
    pub async fn kill_session(&self, session_name: &str) -> Result<()> {
        Command::new(&self.tmux_path)
            .args(&["kill-session", "-t", session_name])
            .output()
            .await?;

        Ok(())
    }
}

/// Tmux session information
#[derive(Debug, Clone)]
pub struct TmuxSession {
    /// Session name
    pub name: String,
    /// Creation timestamp
    pub created: i64,
    /// Whether session is attached
    pub attached: bool,
}

/// Screen compatibility layer (for legacy systems)
pub struct ScreenCompatLayer {
    /// Screen command path
    screen_path: String,
}

impl ScreenCompatLayer {
    /// Create new screen compatibility layer
    pub fn new() -> Self {
        Self {
            screen_path: "screen".to_string(),
        }
    }

    /// Create screen session
    pub async fn create_screen_session(&self, session_id: &SessionId) -> Result<String> {
        let screen_name = format!(
            "ai-{}",
            session_id
                .to_string()
                .split('-')
                .next()
                .unwrap_or("unknown")
        );

        Command::new(&self.screen_path)
            .args(&["-dmS", &screen_name])
            .output()
            .await?;

        Ok(screen_name)
    }
}

/// Migration helper for transitioning from tmux to native sessions
pub struct MigrationHelper {
    /// Tmux compatibility layer
    tmux: TmuxCompatLayer,
}

impl MigrationHelper {
    /// Create new migration helper
    pub fn new() -> Self {
        Self {
            tmux: TmuxCompatLayer::new(),
        }
    }

    /// Migrate tmux session to AI session
    pub async fn migrate_tmux_session(&self, tmux_name: &str) -> Result<MigrationResult> {
        // Capture current state
        let output = self.tmux.capture_output(tmux_name, Some(1000)).await?;
        let env_vars = self.capture_tmux_environment(tmux_name).await?;
        let working_dir = self.get_tmux_working_directory(tmux_name).await?;

        Ok(MigrationResult {
            session_name: tmux_name.to_string(),
            captured_output: output,
            environment: env_vars,
            working_directory: working_dir,
        })
    }

    /// Capture tmux environment variables
    async fn capture_tmux_environment(
        &self,
        session_name: &str,
    ) -> Result<HashMap<String, String>> {
        let output = Command::new("tmux")
            .args(&["show-environment", "-t", session_name])
            .output()
            .await?;

        let mut env_vars = HashMap::new();
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if let Some((key, value)) = line.split_once('=') {
                env_vars.insert(key.to_string(), value.to_string());
            }
        }

        Ok(env_vars)
    }

    /// Get tmux working directory
    async fn get_tmux_working_directory(&self, session_name: &str) -> Result<String> {
        let output = Command::new("tmux")
            .args(&[
                "display-message",
                "-t",
                session_name,
                "-p",
                "#{pane_current_path}",
            ])
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

/// Result of migrating a tmux session
#[derive(Debug)]
pub struct MigrationResult {
    /// Original session name
    pub session_name: String,
    /// Captured output
    pub captured_output: String,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Working directory
    pub working_directory: String,
}

/// Integration with external tools
#[async_trait]
pub trait ExternalIntegration: Send + Sync {
    /// Name of the integration
    fn name(&self) -> &str;

    /// Initialize the integration
    async fn initialize(&mut self) -> Result<()>;

    /// Handle session creation
    async fn on_session_created(&self, session_id: &SessionId) -> Result<()>;

    /// Handle session termination
    async fn on_session_terminated(&self, session_id: &SessionId) -> Result<()>;

    /// Export session data
    async fn export_session_data(&self, session_id: &SessionId) -> Result<serde_json::Value>;
}

/// VS Code integration
pub struct VSCodeIntegration {
    /// Extension communication port
    port: u16,
}

impl VSCodeIntegration {
    /// Create new VS Code integration
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

#[async_trait]
impl ExternalIntegration for VSCodeIntegration {
    fn name(&self) -> &str {
        "vscode"
    }

    async fn initialize(&mut self) -> Result<()> {
        // Initialize VS Code extension communication
        Ok(())
    }

    async fn on_session_created(&self, session_id: &SessionId) -> Result<()> {
        // Notify VS Code extension
        tracing::info!("VS Code integration: session {} created", session_id);
        Ok(())
    }

    async fn on_session_terminated(&self, session_id: &SessionId) -> Result<()> {
        // Notify VS Code extension
        tracing::info!("VS Code integration: session {} terminated", session_id);
        Ok(())
    }

    async fn export_session_data(&self, session_id: &SessionId) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "session_id": session_id.to_string(),
            "integration": "vscode",
            "port": self.port,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmux_compat_layer() {
        let tmux = TmuxCompatLayer::new();
        assert_eq!(tmux.session_prefix, "ai-session");
    }

    #[tokio::test]
    async fn test_vscode_integration() {
        let mut vscode = VSCodeIntegration::new(3000);
        assert_eq!(vscode.name(), "vscode");
        vscode.initialize().await.unwrap();
    }
}
