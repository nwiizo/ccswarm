//! Bridge module that provides tmux-compatible interface using native session management
//!
//! This module provides a fully async TMux-compatible API that ccswarm can use
//! while internally using native session management for better performance and control.

use crate::native::{NativeSession, NativeSessionManager};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

/// Configuration for TMux bridge operations
#[derive(Debug, Clone)]
pub struct TmuxConfig {
    /// Command timeout in seconds
    pub command_timeout: Duration,
    /// Number of retries for failed commands
    pub retry_count: u32,
    /// Delay between retries
    pub retry_delay: Duration,
    /// Default history limit for sessions
    pub history_limit: usize,
    /// Whether to auto-start TMux server
    pub auto_start_server: bool,
    /// Session name prefix (e.g., "ccswarm-")
    pub session_prefix: String,
}

impl Default for TmuxConfig {
    fn default() -> Self {
        Self {
            command_timeout: Duration::from_secs(30),
            retry_count: 3,
            retry_delay: Duration::from_millis(500),
            history_limit: 10000,
            auto_start_server: true,
            session_prefix: String::new(),
        }
    }
}

/// TmuxClient replacement that uses native session management
/// Provides fully async interface without any blocking operations
pub struct TmuxClient {
    /// Native session manager
    session_manager: Arc<NativeSessionManager>,
    /// Session cache for quick lookups
    session_cache: Arc<RwLock<HashMap<String, Arc<tokio::sync::Mutex<NativeSession>>>>>,
    /// Window information per session
    windows: Arc<RwLock<HashMap<String, Vec<TmuxWindow>>>>,
    /// Pane information per window
    #[allow(dead_code)]
    panes: Arc<RwLock<HashMap<String, Vec<TmuxPane>>>>,
    /// Configuration
    config: TmuxConfig,
    /// TMux server status
    server_running: Arc<RwLock<bool>>,
}

impl TmuxClient {
    /// Creates a new tmux-compatible client
    pub async fn new() -> Result<Self> {
        Self::with_config(TmuxConfig::default()).await
    }

    /// Creates a new client with custom configuration
    pub async fn with_config(config: TmuxConfig) -> Result<Self> {
        let client = Self {
            session_manager: Arc::new(NativeSessionManager::new()),
            session_cache: Arc::new(RwLock::new(HashMap::new())),
            windows: Arc::new(RwLock::new(HashMap::new())),
            panes: Arc::new(RwLock::new(HashMap::new())),
            config,
            server_running: Arc::new(RwLock::new(true)), // Assume running for native
        };

        if client.config.auto_start_server {
            client.ensure_server_running().await?;
        }

        Ok(client)
    }

    /// Checks if the TMux server is running
    pub async fn is_server_running(&self) -> bool {
        *self.server_running.read().await
    }

    /// Ensures the TMux server is running, starting it if necessary
    pub async fn ensure_server_running(&self) -> Result<()> {
        let mut running = self.server_running.write().await;
        if !*running {
            // For native sessions, we don't need a server
            // This is just for compatibility
            *running = true;
        }
        Ok(())
    }

    /// Validates a session name according to TMux rules
    pub fn validate_session_name(name: &str) -> Result<()> {
        if name.contains(':') || name.contains('.') {
            return Err(anyhow::anyhow!(
                "Session name cannot contain ':' or '.' characters"
            ));
        }
        if name.is_empty() {
            return Err(anyhow::anyhow!("Session name cannot be empty"));
        }
        Ok(())
    }

    /// Creates a new session (fully async)
    pub async fn create_session(&self, session_name: &str, working_directory: &str) -> Result<()> {
        Self::validate_session_name(session_name)?;

        let full_name = format!("{}{}", self.config.session_prefix, session_name);

        // Execute with timeout and retry
        self.execute_with_retry(|| async {
            let session = self.session_manager.create_session(&full_name).await?;

            // Change to working directory
            let session_lock = session.lock().await;
            session_lock
                .send_input(&format!("cd {}\n", working_directory))
                .await?;
            drop(session_lock);

            // Cache the session
            let mut cache = self.session_cache.write().await;
            cache.insert(full_name.clone(), session);

            // Initialize default window
            let mut windows = self.windows.write().await;
            windows.insert(
                full_name.clone(),
                vec![TmuxWindow {
                    id: "@1".to_string(),
                    name: "main".to_string(),
                    active: true,
                    layout: "".to_string(),
                    panes: vec![TmuxPane {
                        id: "%1".to_string(),
                        active: true,
                        current_path: working_directory.to_string(),
                        current_command: "bash".to_string(),
                    }],
                }],
            );

            Ok(())
        })
        .await
    }

    /// Checks if a session exists
    pub async fn has_session(&self, session_name: &str) -> Result<bool> {
        let full_name = format!("{}{}", self.config.session_prefix, session_name);
        Ok(self.session_manager.has_session(&full_name).await)
    }

    /// Kills a session (fully async)
    pub async fn kill_session(&self, session_name: &str) -> Result<()> {
        let full_name = format!("{}{}", self.config.session_prefix, session_name);

        self.execute_with_retry(|| async {
            self.session_manager.delete_session(&full_name).await?;

            // Remove from cache
            let mut cache = self.session_cache.write().await;
            cache.remove(&full_name);

            // Remove windows and panes
            let mut windows = self.windows.write().await;
            windows.remove(&full_name);

            Ok(())
        })
        .await
    }

    /// Sends keys to a session (fully async)
    pub async fn send_keys(&self, session_name: &str, keys: &str) -> Result<()> {
        let full_name = format!("{}{}", self.config.session_prefix, session_name);

        self.execute_with_retry(|| async {
            if let Some(session) = self.session_manager.get_session(&full_name).await {
                let session_lock = session.lock().await;

                // Handle special keys
                let input = match keys {
                    "C-c" => "\x03",
                    "C-z" => "\x1a",
                    "C-d" => "\x04",
                    "C-a" => "\x01",
                    "C-e" => "\x05",
                    "C-k" => "\x0b",
                    "C-l" => "\x0c",
                    "C-u" => "\x15",
                    "C-w" => "\x17",
                    "Enter" => "\n",
                    "Tab" => "\t",
                    "Escape" => "\x1b",
                    "Space" => " ",
                    _ => keys,
                };

                session_lock.send_input(input).await?;
                Ok(())
            } else {
                Err(TmuxError::SessionNotFound(session_name.to_string()).into())
            }
        })
        .await
    }

    /// Sends a command to a session (fully async)
    pub async fn send_command(&self, session_name: &str, command: &str) -> Result<()> {
        let full_name = format!("{}{}", self.config.session_prefix, session_name);

        self.execute_with_retry(|| async {
            if let Some(session) = self.session_manager.get_session(&full_name).await {
                let session_lock = session.lock().await;
                session_lock.send_input(&format!("{}\n", command)).await?;
                Ok(())
            } else {
                Err(TmuxError::SessionNotFound(session_name.to_string()).into())
            }
        })
        .await
    }

    /// Captures pane output (fully async)
    pub async fn capture_pane(&self, session_name: &str, pane_id: Option<&str>) -> Result<String> {
        self.capture_pane_with_options(session_name, pane_id, None)
            .await
    }

    /// Captures pane output with line limit (fully async)
    pub async fn capture_pane_with_options(
        &self,
        session_name: &str,
        _pane_id: Option<&str>,
        line_limit: Option<usize>,
    ) -> Result<String> {
        let full_name = format!("{}{}", self.config.session_prefix, session_name);

        self.execute_with_retry(|| async {
            if let Some(session) = self.session_manager.get_session(&full_name).await {
                let session_lock = session.lock().await;
                let lines = line_limit.unwrap_or(self.config.history_limit);
                let output = session_lock.get_output(lines).await?;
                Ok(output.join("\n"))
            } else {
                Err(TmuxError::SessionNotFound(session_name.to_string()).into())
            }
        })
        .await
    }

    /// Lists all sessions (fully async)
    pub async fn list_sessions(&self) -> Result<Vec<TmuxSession>> {
        let session_names = self.session_manager.list_sessions().await;
        let mut sessions = Vec::new();

        for (idx, name) in session_names.iter().enumerate() {
            // Strip prefix if present
            let display_name = if name.starts_with(&self.config.session_prefix) {
                &name[self.config.session_prefix.len()..]
            } else {
                name
            };

            let windows = self.list_windows(display_name).await?;

            sessions.push(TmuxSession {
                name: display_name.to_string(),
                id: format!("${}", idx + 1),
                windows,
                attached: false,
                created: chrono::Utc::now().to_rfc3339(),
                last_attached: None,
            });
        }

        Ok(sessions)
    }

    /// Checks if a session exists (async wrapper)
    pub async fn session_exists(&self, session_name: &str) -> Result<bool> {
        self.has_session(session_name).await
    }

    /// Sets environment variable in a session
    pub async fn set_environment(&self, session_name: &str, name: &str, value: &str) -> Result<()> {
        let full_name = format!("{}{}", self.config.session_prefix, session_name);

        self.execute_with_retry(|| async {
            if let Some(session) = self.session_manager.get_session(&full_name).await {
                let session_lock = session.lock().await;
                // Export the environment variable in the shell
                session_lock
                    .send_input(&format!("export {}='{}'\n", name, value))
                    .await?;
                Ok(())
            } else {
                Err(TmuxError::SessionNotFound(session_name.to_string()).into())
            }
        })
        .await
    }

    /// Sets a TMux option (implemented as no-op for compatibility)
    pub async fn set_option(&self, _session_name: &str, option: &str, value: &str) -> Result<()> {
        // Store options for reference but don't apply them to native sessions
        log::debug!("TMux option set (no-op): {} = {}", option, value);
        Ok(())
    }

    /// Creates a new window in a session
    pub async fn new_window(
        &self,
        session_name: &str,
        window_name: &str,
        working_directory: Option<&str>,
    ) -> Result<String> {
        let full_name = format!("{}{}", self.config.session_prefix, session_name);

        let mut windows = self.windows.write().await;
        let session_windows = windows.entry(full_name.clone()).or_insert_with(Vec::new);

        let window_id = format!("@{}", session_windows.len() + 1);

        // Deactivate other windows
        for window in session_windows.iter_mut() {
            window.active = false;
        }

        session_windows.push(TmuxWindow {
            id: window_id.clone(),
            name: window_name.to_string(),
            active: true,
            layout: "".to_string(),
            panes: vec![TmuxPane {
                id: format!("%{}", session_windows.len() + 1),
                active: true,
                current_path: working_directory.unwrap_or("/").to_string(),
                current_command: "bash".to_string(),
            }],
        });

        // If working directory specified, change to it
        if let Some(dir) = working_directory {
            self.send_command(session_name, &format!("cd {}", dir))
                .await?;
        }

        Ok(window_id)
    }

    /// Lists windows in a session
    pub async fn list_windows(&self, session_name: &str) -> Result<Vec<TmuxWindow>> {
        let full_name = format!("{}{}", self.config.session_prefix, session_name);
        let windows = self.windows.read().await;

        Ok(windows.get(&full_name).cloned().unwrap_or_else(|| {
            vec![TmuxWindow {
                id: "@1".to_string(),
                name: "main".to_string(),
                active: true,
                layout: "".to_string(),
                panes: vec![],
            }]
        }))
    }

    /// Kills a window
    pub async fn kill_window(&self, session_name: &str, window_id: &str) -> Result<()> {
        let full_name = format!("{}{}", self.config.session_prefix, session_name);
        let mut windows = self.windows.write().await;

        if let Some(session_windows) = windows.get_mut(&full_name) {
            session_windows.retain(|w| w.id != window_id);

            // Activate first window if active window was killed
            if !session_windows.iter().any(|w| w.active) && !session_windows.is_empty() {
                session_windows[0].active = true;
            }
        }

        Ok(())
    }

    /// Lists panes in a window
    pub async fn list_panes(
        &self,
        session_name: &str,
        window_id: Option<&str>,
    ) -> Result<Vec<TmuxPane>> {
        let full_name = format!("{}{}", self.config.session_prefix, session_name);
        let windows = self.windows.read().await;

        if let Some(session_windows) = windows.get(&full_name) {
            if let Some(window_id) = window_id {
                // Get panes for specific window
                if let Some(window) = session_windows.iter().find(|w| w.id == window_id) {
                    Ok(window.panes.clone())
                } else {
                    Ok(vec![])
                }
            } else {
                // Get panes for active window
                if let Some(window) = session_windows.iter().find(|w| w.active) {
                    Ok(window.panes.clone())
                } else {
                    Ok(vec![])
                }
            }
        } else {
            Ok(vec![])
        }
    }

    /// Splits a window into panes
    pub async fn split_window(
        &self,
        session_name: &str,
        window_id: Option<&str>,
        vertical: bool,
        percentage: Option<u8>,
    ) -> Result<String> {
        let full_name = format!("{}{}", self.config.session_prefix, session_name);
        let mut windows = self.windows.write().await;

        if let Some(session_windows) = windows.get_mut(&full_name) {
            let window = if let Some(window_id) = window_id {
                session_windows.iter_mut().find(|w| w.id == window_id)
            } else {
                session_windows.iter_mut().find(|w| w.active)
            };

            if let Some(window) = window {
                let pane_id = format!("%{}", window.panes.len() + 1);

                // Deactivate other panes
                for pane in window.panes.iter_mut() {
                    pane.active = false;
                }

                window.panes.push(TmuxPane {
                    id: pane_id.clone(),
                    active: true,
                    current_path: "/".to_string(),
                    current_command: "bash".to_string(),
                });

                // Log split information
                log::debug!(
                    "Split window {} {} with {}% size",
                    if vertical {
                        "vertically"
                    } else {
                        "horizontally"
                    },
                    window.id,
                    percentage.unwrap_or(50)
                );

                Ok(pane_id)
            } else {
                Err(anyhow::anyhow!("Window not found"))
            }
        } else {
            Err(anyhow::anyhow!("Session not found"))
        }
    }

    /// Selects a pane
    pub async fn select_pane(&self, session_name: &str, pane_id: &str) -> Result<()> {
        let full_name = format!("{}{}", self.config.session_prefix, session_name);
        let mut windows = self.windows.write().await;

        if let Some(session_windows) = windows.get_mut(&full_name) {
            for window in session_windows.iter_mut() {
                for pane in window.panes.iter_mut() {
                    pane.active = pane.id == pane_id;
                }
            }
            Ok(())
        } else {
            Err(TmuxError::SessionNotFound(session_name.to_string()).into())
        }
    }

    /// Attaches to session (no-op for compatibility)
    pub async fn attach_session(&self, session_name: &str) -> Result<()> {
        if !self.has_session(session_name).await? {
            return Err(TmuxError::SessionNotFound(session_name.to_string()).into());
        }

        // Update session as attached
        log::debug!("Session '{}' marked as attached", session_name);
        Ok(())
    }

    /// Detaches from session (no-op for compatibility)
    pub async fn detach_session(&self, session_name: &str) -> Result<()> {
        if !self.has_session(session_name).await? {
            return Err(TmuxError::SessionNotFound(session_name.to_string()).into());
        }

        // Update session as detached
        log::debug!("Session '{}' marked as detached", session_name);
        Ok(())
    }

    /// Gets session info
    pub async fn get_session_info(&self, session_name: &str) -> Result<TmuxSession> {
        if self.has_session(session_name).await? {
            let windows = self.list_windows(session_name).await?;

            Ok(TmuxSession {
                name: session_name.to_string(),
                id: "$1".to_string(),
                windows,
                attached: false,
                created: chrono::Utc::now().to_rfc3339(),
                last_attached: None,
            })
        } else {
            Err(TmuxError::SessionNotFound(session_name.to_string()).into())
        }
    }

    /// Executes an operation with timeout and retry logic
    async fn execute_with_retry<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;

        for attempt in 0..=self.config.retry_count {
            if attempt > 0 {
                tokio::time::sleep(self.config.retry_delay).await;
                log::debug!("Retrying operation (attempt {})", attempt);
            }

            match timeout(self.config.command_timeout, operation()).await {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(e)) => {
                    last_error = Some(e);
                    if attempt < self.config.retry_count {
                        log::warn!(
                            "Operation failed, will retry: {}",
                            last_error.as_ref().unwrap()
                        );
                    }
                }
                Err(_) => {
                    last_error = Some(anyhow::anyhow!("Operation timed out"));
                    if attempt < self.config.retry_count {
                        log::warn!("Operation timed out, will retry");
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Operation failed after retries")))
    }
}

// Define tmux types for compatibility
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TmuxSession {
    pub name: String,
    pub id: String,
    pub windows: Vec<TmuxWindow>,
    pub attached: bool,
    pub created: String,
    pub last_attached: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TmuxWindow {
    pub id: String,
    pub name: String,
    pub active: bool,
    pub layout: String,
    pub panes: Vec<TmuxPane>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TmuxPane {
    pub id: String,
    pub active: bool,
    pub current_path: String,
    pub current_command: String,
}

/// Structured TMux error types
#[derive(Debug, thiserror::Error)]
pub enum TmuxError {
    #[error("Tmux server is not running")]
    ServerNotRunning,

    #[error("Session '{0}' not found")]
    SessionNotFound(String),

    #[error("Window '{0}' not found")]
    WindowNotFound(String),

    #[error("Pane '{0}' not found")]
    PaneNotFound(String),

    #[error("Invalid session name: {0}")]
    InvalidSessionName(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("Command timed out after {0:?}")]
    CommandTimeout(Duration),

    #[error("Server error: {0}")]
    ServerError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_lifecycle() -> Result<()> {
        let client = TmuxClient::new().await?;

        // Create session
        client.create_session("test", "/tmp").await?;

        // Check it exists
        assert!(client.has_session("test").await?);

        // Send command
        client
            .send_command("test", "echo 'Hello TMux Bridge'")
            .await?;

        // Send special keys
        client.send_keys("test", "C-c").await?;

        // Capture output
        let output = client.capture_pane("test", None).await?;
        assert!(!output.is_empty());

        // Kill session
        client.kill_session("test").await?;
        assert!(!client.has_session("test").await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_window_management() -> Result<()> {
        let client = TmuxClient::new().await?;

        // Create session
        client.create_session("window-test", "/tmp").await?;

        // Create new window
        let window_id = client
            .new_window("window-test", "dev", Some("/home"))
            .await?;
        assert!(!window_id.is_empty());

        // List windows
        let windows = client.list_windows("window-test").await?;
        assert_eq!(windows.len(), 2);

        // Kill window
        client.kill_window("window-test", &window_id).await?;

        let windows = client.list_windows("window-test").await?;
        assert_eq!(windows.len(), 1);

        // Cleanup
        client.kill_session("window-test").await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_pane_management() -> Result<()> {
        let client = TmuxClient::new().await?;

        // Create session
        client.create_session("pane-test", "/tmp").await?;

        // Split window
        let pane_id = client
            .split_window("pane-test", None, true, Some(50))
            .await?;
        assert!(!pane_id.is_empty());

        // List panes
        let panes = client.list_panes("pane-test", None).await?;
        assert_eq!(panes.len(), 2);

        // Select pane
        client.select_pane("pane-test", &pane_id).await?;

        // Cleanup
        client.kill_session("pane-test").await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_session_name() {
        let client = TmuxClient::new().await.unwrap();

        // Test invalid names
        assert!(client.create_session("test:invalid", "/tmp").await.is_err());
        assert!(client.create_session("test.invalid", "/tmp").await.is_err());
        assert!(client.create_session("", "/tmp").await.is_err());
    }

    #[tokio::test]
    async fn test_session_prefix() -> Result<()> {
        let config = TmuxConfig {
            session_prefix: "ccswarm-".to_string(),
            ..Default::default()
        };

        let client = TmuxClient::with_config(config).await?;

        // Create session
        client.create_session("frontend", "/tmp").await?;

        // Session should exist with prefix
        assert!(client.has_session("frontend").await?);

        // List should show without prefix
        let sessions = client.list_sessions().await?;
        assert!(sessions.iter().any(|s| s.name == "frontend"));

        // Cleanup
        client.kill_session("frontend").await?;

        Ok(())
    }
}
