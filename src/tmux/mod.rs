use serde::{Deserialize, Serialize};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Error types for tmux operations
#[derive(Debug, thiserror::Error)]
pub enum TmuxError {
    #[error("Tmux not found or not installed")]
    TmuxNotFound,
    #[error("Session '{0}' not found")]
    SessionNotFound(String),
    #[error("Window '{0}' not found in session '{1}'")]
    WindowNotFound(String, String),
    #[error("Command failed: {0}")]
    CommandFailed(String),
    #[error("Tmux command execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Invalid session name: {0}")]
    InvalidSessionName(String),
    #[error("Tmux server not running")]
    ServerNotRunning,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Information about a tmux session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxSession {
    pub name: String,
    pub id: String,
    pub windows: Vec<TmuxWindow>,
    pub attached: bool,
    pub created: String,
    pub last_attached: Option<String>,
}

/// Information about a tmux window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxWindow {
    pub id: String,
    pub name: String,
    pub active: bool,
    pub layout: String,
    pub panes: Vec<TmuxPane>,
}

/// Information about a tmux pane
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmuxPane {
    pub id: String,
    pub active: bool,
    pub current_path: String,
    pub current_command: String,
}

/// Configuration for tmux client
#[derive(Debug, Clone)]
pub struct TmuxClientConfig {
    /// Timeout for tmux commands
    pub command_timeout: Duration,
    /// Whether to check if tmux server is running before commands
    pub check_server: bool,
    /// Whether to automatically start tmux server if not running
    pub auto_start_server: bool,
    /// Maximum number of retry attempts for failed commands
    pub max_retries: u32,
    /// Default shell to use in new sessions
    pub default_shell: Option<String>,
    /// Global environment variables to set in all sessions
    pub global_env: HashMap<String, String>,
}

impl Default for TmuxClientConfig {
    fn default() -> Self {
        Self {
            command_timeout: Duration::from_secs(30),
            check_server: true,
            auto_start_server: true,
            max_retries: 3,
            default_shell: None,
            global_env: HashMap::new(),
        }
    }
}

/// Enhanced client for interacting with tmux
pub struct TmuxClient {
    /// Configuration for the tmux client
    config: TmuxClientConfig,
    /// Cache of session information to reduce tmux calls
    #[allow(dead_code)] // Will be used in future caching implementation
    session_cache: std::sync::Mutex<HashMap<String, (TmuxSession, Instant)>>,
    /// Cache timeout duration
    #[allow(dead_code)] // Will be used in future caching implementation
    cache_timeout: Duration,
}

impl TmuxClient {
    /// Creates a new tmux client with default configuration
    ///
    /// # Returns
    /// A new TmuxClient instance
    ///
    /// # Errors
    /// Returns TmuxError::TmuxNotFound if tmux is not installed
    pub fn new() -> Result<Self, TmuxError> {
        Self::with_config(TmuxClientConfig::default())
    }

    /// Creates a new tmux client with custom configuration
    ///
    /// # Arguments
    /// * `config` - Configuration for the tmux client
    ///
    /// # Returns
    /// A new TmuxClient instance
    ///
    /// # Errors
    /// Returns TmuxError::TmuxNotFound if tmux is not installed
    pub fn with_config(config: TmuxClientConfig) -> Result<Self, TmuxError> {
        // Check if tmux is available
        let output = Command::new("tmux")
            .arg("-V")
            .output()
            .map_err(|_| TmuxError::TmuxNotFound)?;

        if !output.status.success() {
            return Err(TmuxError::TmuxNotFound);
        }

        Ok(Self {
            config,
            session_cache: std::sync::Mutex::new(HashMap::new()),
            cache_timeout: Duration::from_secs(5), // Cache for 5 seconds
        })
    }

    /// Creates a new tmux client with custom timeout (legacy method)
    pub fn with_timeout(timeout: Duration) -> Result<Self, TmuxError> {
        let mut config = TmuxClientConfig::default();
        config.command_timeout = timeout;
        Self::with_config(config)
    }

    /// Gets the tmux version
    pub fn get_version(&self) -> Result<String, TmuxError> {
        let output = self.run_command_with_output(&["-V"])?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Checks if the tmux server is running
    pub fn is_server_running(&self) -> bool {
        Command::new("tmux")
            .args(&["list-sessions"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Starts the tmux server if not running
    pub fn ensure_server_running(&self) -> Result<(), TmuxError> {
        if !self.is_server_running() {
            self.run_command(&["start-server"])?;
        }
        Ok(())
    }

    /// Creates a new tmux session
    ///
    /// # Arguments
    /// * `session_name` - Name of the session to create
    /// * `working_directory` - Initial working directory for the session
    ///
    /// # Returns
    /// Ok(()) on success
    ///
    /// # Errors
    /// Returns TmuxError if session creation fails
    pub fn create_session(
        &self,
        session_name: &str,
        working_directory: &str,
    ) -> Result<(), TmuxError> {
        self.validate_session_name(session_name)?;

        if self.config.check_server {
            self.ensure_server_running()?;
        }

        // Check if session already exists
        if self.session_exists(session_name)? {
            return Err(TmuxError::CommandFailed(format!(
                "Session '{}' already exists",
                session_name
            )));
        }

        let args = vec![
            "new-session",
            "-d",
            "-s",
            session_name,
            "-c",
            working_directory,
        ];

        self.run_command(&args)?;
        Ok(())
    }

    /// Creates a new session with a specific command
    pub fn create_session_with_command(
        &self,
        session_name: &str,
        working_directory: &str,
        command: &str,
    ) -> Result<(), TmuxError> {
        self.validate_session_name(session_name)?;

        if self.config.check_server {
            self.ensure_server_running()?;
        }

        let args = vec![
            "new-session",
            "-d",
            "-s",
            session_name,
            "-c",
            working_directory,
            command,
        ];

        self.run_command(&args)?;
        Ok(())
    }

    /// Kills a tmux session
    ///
    /// # Arguments
    /// * `session_name` - Name of the session to kill
    ///
    /// # Returns
    /// Ok(()) on success
    pub fn kill_session(&self, session_name: &str) -> Result<(), TmuxError> {
        if !self.session_exists(session_name)? {
            return Err(TmuxError::SessionNotFound(session_name.to_string()));
        }

        let args = vec!["kill-session", "-t", session_name];
        self.run_command(&args)?;
        Ok(())
    }

    /// Attaches to a tmux session
    ///
    /// # Arguments
    /// * `session_name` - Name of the session to attach to
    ///
    /// # Returns
    /// Ok(()) on success
    pub fn attach_session(&self, session_name: &str) -> Result<(), TmuxError> {
        if !self.session_exists(session_name)? {
            return Err(TmuxError::SessionNotFound(session_name.to_string()));
        }

        let args = vec!["attach-session", "-t", session_name];
        self.run_command(&args)?;
        Ok(())
    }

    /// Detaches from a tmux session
    ///
    /// # Arguments
    /// * `session_name` - Name of the session to detach from
    ///
    /// # Returns
    /// Ok(()) on success
    pub fn detach_session(&self, session_name: &str) -> Result<(), TmuxError> {
        let args = vec!["detach-client", "-s", session_name];
        self.run_command(&args)?;
        Ok(())
    }

    /// Sends keys to a tmux session
    ///
    /// # Arguments
    /// * `session_name` - Name of the target session
    /// * `keys` - Keys to send (e.g., "ls Enter", "C-c")
    ///
    /// # Returns
    /// Ok(()) on success
    pub fn send_keys(&self, session_name: &str, keys: &str) -> Result<(), TmuxError> {
        if !self.session_exists(session_name)? {
            return Err(TmuxError::SessionNotFound(session_name.to_string()));
        }

        let args = vec!["send-keys", "-t", session_name, keys];
        self.run_command(&args)?;
        Ok(())
    }

    /// Sends a command to a tmux session (automatically adds Enter)
    ///
    /// # Arguments
    /// * `session_name` - Name of the target session
    /// * `command` - Command to send
    ///
    /// # Returns
    /// Ok(()) on success
    pub fn send_command(&self, session_name: &str, command: &str) -> Result<(), TmuxError> {
        if !self.session_exists(session_name)? {
            return Err(TmuxError::SessionNotFound(session_name.to_string()));
        }

        let args = vec!["send-keys", "-t", session_name, command, "Enter"];
        self.run_command(&args)?;
        Ok(())
    }

    /// Captures the output from a tmux pane
    ///
    /// # Arguments
    /// * `session_name` - Name of the target session
    /// * `pane_id` - Optional pane ID (defaults to current pane)
    ///
    /// # Returns
    /// String containing the pane output
    pub fn capture_pane(
        &self,
        session_name: &str,
        pane_id: Option<&str>,
    ) -> Result<String, TmuxError> {
        if !self.session_exists(session_name)? {
            return Err(TmuxError::SessionNotFound(session_name.to_string()));
        }

        let target = match pane_id {
            Some(pane) => format!("{}:{}", session_name, pane),
            None => session_name.to_string(),
        };

        let args = vec!["capture-pane", "-t", &target, "-p"];
        let output = self.run_command_with_output(&args)?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Lists all tmux sessions
    ///
    /// # Returns
    /// Vector of TmuxSession information
    pub fn list_sessions(&self) -> Result<Vec<TmuxSession>, TmuxError> {
        let args = vec![
            "list-sessions",
            "-F",
            "#{session_name}|#{session_id}|#{session_attached}|#{session_created}|#{session_last_attached}",
        ];

        let output = self.run_command_with_output(&args)?;
        let output_str = String::from_utf8_lossy(&output.stdout);

        let mut sessions = Vec::new();
        for line in output_str.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                let session = TmuxSession {
                    name: parts[0].to_string(),
                    id: parts[1].to_string(),
                    attached: parts[2] == "1",
                    created: parts[3].to_string(),
                    last_attached: if parts.len() > 4 && !parts[4].is_empty() {
                        Some(parts[4].to_string())
                    } else {
                        None
                    },
                    windows: Vec::new(), // Will be populated by list_windows if needed
                };
                sessions.push(session);
            }
        }

        Ok(sessions)
    }

    /// Lists windows in a session
    ///
    /// # Arguments
    /// * `session_name` - Name of the session
    ///
    /// # Returns
    /// Vector of TmuxWindow information
    pub fn list_windows(&self, session_name: &str) -> Result<Vec<TmuxWindow>, TmuxError> {
        if !self.session_exists(session_name)? {
            return Err(TmuxError::SessionNotFound(session_name.to_string()));
        }

        let args = vec![
            "list-windows",
            "-t",
            session_name,
            "-F",
            "#{window_id}|#{window_name}|#{window_active}|#{window_layout}",
        ];

        let output = self.run_command_with_output(&args)?;
        let output_str = String::from_utf8_lossy(&output.stdout);

        let mut windows = Vec::new();
        for line in output_str.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 4 {
                let window = TmuxWindow {
                    id: parts[0].to_string(),
                    name: parts[1].to_string(),
                    active: parts[2] == "1",
                    layout: parts[3].to_string(),
                    panes: Vec::new(), // Will be populated by list_panes if needed
                };
                windows.push(window);
            }
        }

        Ok(windows)
    }

    /// Creates a new window in a session
    ///
    /// # Arguments
    /// * `session_name` - Name of the target session
    /// * `window_name` - Name for the new window
    /// * `working_directory` - Working directory for the window
    ///
    /// # Returns
    /// Ok(()) on success
    pub fn new_window(
        &self,
        session_name: &str,
        window_name: &str,
        working_directory: Option<&str>,
    ) -> Result<(), TmuxError> {
        if !self.session_exists(session_name)? {
            return Err(TmuxError::SessionNotFound(session_name.to_string()));
        }

        let mut args = vec!["new-window", "-t", session_name, "-n", window_name];

        if let Some(dir) = working_directory {
            args.extend_from_slice(&["-c", dir]);
        }

        self.run_command(&args)?;
        Ok(())
    }

    /// Sets an environment variable in a session
    ///
    /// # Arguments
    /// * `session_name` - Name of the target session
    /// * `name` - Environment variable name
    /// * `value` - Environment variable value
    ///
    /// # Returns
    /// Ok(()) on success
    pub fn set_environment(
        &self,
        session_name: &str,
        name: &str,
        value: &str,
    ) -> Result<(), TmuxError> {
        if !self.session_exists(session_name)? {
            return Err(TmuxError::SessionNotFound(session_name.to_string()));
        }

        let args = vec!["set-environment", "-t", session_name, name, value];
        self.run_command(&args)?;
        Ok(())
    }

    /// Sets a tmux option for a session
    ///
    /// # Arguments
    /// * `session_name` - Name of the target session
    /// * `option` - Option name
    /// * `value` - Option value
    ///
    /// # Returns
    /// Ok(()) on success
    pub fn set_option(
        &self,
        session_name: &str,
        option: &str,
        value: &str,
    ) -> Result<(), TmuxError> {
        if !self.session_exists(session_name)? {
            return Err(TmuxError::SessionNotFound(session_name.to_string()));
        }

        let args = vec!["set-option", "-t", session_name, option, value];
        self.run_command(&args)?;
        Ok(())
    }

    /// Gets information about a specific session
    ///
    /// # Arguments
    /// * `session_name` - Name of the session
    ///
    /// # Returns
    /// TmuxSession with detailed information
    pub fn get_session_info(&self, session_name: &str) -> Result<TmuxSession, TmuxError> {
        if !self.session_exists(session_name)? {
            return Err(TmuxError::SessionNotFound(session_name.to_string()));
        }

        let sessions = self.list_sessions()?;
        let session = sessions
            .into_iter()
            .find(|s| s.name == session_name)
            .ok_or_else(|| TmuxError::SessionNotFound(session_name.to_string()))?;

        // Get windows for this session
        let windows = self.list_windows(session_name)?;

        Ok(TmuxSession { windows, ..session })
    }

    /// Checks if a session exists
    fn session_exists(&self, session_name: &str) -> Result<bool, TmuxError> {
        let args = vec!["has-session", "-t", session_name];
        let result = self.run_command(&args);

        match result {
            Ok(_) => Ok(true),
            Err(TmuxError::CommandFailed(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Validates a session name
    fn validate_session_name(&self, name: &str) -> Result<(), TmuxError> {
        if name.is_empty() {
            return Err(TmuxError::InvalidSessionName(
                "Session name cannot be empty".to_string(),
            ));
        }

        // Tmux session names cannot contain certain characters
        if name.contains(':') || name.contains('.') {
            return Err(TmuxError::InvalidSessionName(
                "Session name cannot contain ':' or '.' characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Runs a tmux command without capturing output
    fn run_command(&self, args: &[&str]) -> Result<(), TmuxError> {
        let mut cmd = Command::new("tmux");
        cmd.args(args);

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Runs a tmux command and captures output
    fn run_command_with_output(&self, args: &[&str]) -> Result<Output, TmuxError> {
        let mut cmd = Command::new("tmux");
        cmd.args(args);

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(TmuxError::CommandFailed(stderr.to_string()));
        }

        Ok(output)
    }
}

impl Default for TmuxClient {
    fn default() -> Self {
        Self::new().expect("Failed to create TmuxClient")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_name_validation() {
        let client = TmuxClient::new().unwrap();

        // Valid names
        assert!(client.validate_session_name("test-session").is_ok());
        assert!(client.validate_session_name("agent_123").is_ok());
        assert!(client.validate_session_name("MySession").is_ok());

        // Invalid names
        assert!(client.validate_session_name("").is_err());
        assert!(client.validate_session_name("test:session").is_err());
        assert!(client.validate_session_name("test.session").is_err());
    }

    #[test]
    fn test_tmux_session_creation() {
        let session = TmuxSession {
            name: "test-session".to_string(),
            id: "$1".to_string(),
            windows: Vec::new(),
            attached: false,
            created: "1234567890".to_string(),
            last_attached: None,
        };

        assert_eq!(session.name, "test-session");
        assert!(!session.attached);
        assert!(session.windows.is_empty());
    }

    #[test]
    fn test_tmux_error_display() {
        let error = TmuxError::SessionNotFound("test".to_string());
        assert_eq!(error.to_string(), "Session 'test' not found");

        let error = TmuxError::TmuxNotFound;
        assert_eq!(error.to_string(), "Tmux not found or not installed");
    }
}
