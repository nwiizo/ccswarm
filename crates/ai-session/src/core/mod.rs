//! Core session management functionality
//!
//! This module provides the foundational components for AI-optimized terminal session management.
//! The core functionality includes session creation, lifecycle management, and AI context integration.
//!
//! # Key Features
//!
//! - **AISession**: Advanced terminal session with AI capabilities
//! - **SessionManager**: Pool-based session management with automatic cleanup
//! - **SessionConfig**: Comprehensive configuration for AI features and performance
//! - **Context Integration**: Seamless integration with AI conversation context
//!
//! # Quick Start
//!
//! ```no_run
//! use ai_session::{SessionManager, SessionConfig, ContextConfig};
//! use tokio;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let manager = SessionManager::new();
//!     
//!     // Create a basic session
//!     let session = manager.create_session().await?;
//!     session.start().await?;
//!     
//!     // Send a command
//!     session.send_input("echo 'Hello AI Session!'\n").await?;
//!     
//!     // Read the output
//!     tokio::time::sleep(std::time::Duration::from_millis(300)).await;
//!     let output = session.read_output().await?;
//!     println!("Output: {}", String::from_utf8_lossy(&output));
//!     
//!     // Clean up
//!     session.stop().await?;
//!     Ok(())
//! }
//! ```
//!
//! # Advanced Configuration
//!
//! ```no_run
//! use ai_session::{SessionManager, SessionConfig, ContextConfig};
//! use std::collections::HashMap;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let manager = SessionManager::new();
//!     
//!     // Configure session with AI features
//!     let mut config = SessionConfig::default();
//!     config.enable_ai_features = true;
//!     config.agent_role = Some("rust-developer".to_string());
//!     config.context_config = ContextConfig {
//!         max_tokens: 8192,
//!         compression_threshold: 0.8,
//!     };
//!     
//!     // Set environment variables
//!     config.environment.insert("RUST_LOG".to_string(), "debug".to_string());
//!     config.working_directory = "/path/to/project".into();
//!     
//!     let session = manager.create_session_with_config(config).await?;
//!     session.start().await?;
//!     
//!     // Session is now ready for AI-enhanced development
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod headless;
pub mod lifecycle;
pub mod process;
pub mod pty;
pub mod terminal;

use crate::context::SessionContext;
use crate::persistence::CommandRecord;

/// Session error type
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found: {0}")]
    NotFound(SessionId),

    #[error("Session already exists: {0}")]
    AlreadyExists(SessionId),

    #[error("PTY error: {0}")]
    PtyError(String),

    #[error("Process error: {0}")]
    ProcessError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Session result type
pub type SessionResult<T> = std::result::Result<T, SessionError>;

/// Unique session identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionId {
    /// Create a new unique session ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a new v4 UUID session ID
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse from string
    pub fn parse_str(s: &str) -> Result<Self> {
        Ok(Self(Uuid::parse_str(s)?))
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    /// Get inner UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SessionStatus {
    /// Session is being initialized
    #[default]
    Initializing,
    /// Session is running and ready
    Running,
    /// Session is paused
    Paused,
    /// Session is being terminated
    Terminating,
    /// Session has been terminated
    Terminated,
    /// Session encountered an error
    Error,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionConfig {
    /// Session name (optional)
    pub name: Option<String>,
    /// Working directory
    pub working_directory: PathBuf,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Shell command to execute
    pub shell: Option<String>,
    /// Shell command to execute (alternative field for compatibility)
    pub shell_command: Option<String>,
    /// PTY size (rows, cols)
    pub pty_size: (u16, u16),
    /// Output buffer size in bytes
    pub output_buffer_size: usize,
    /// Session timeout (None for no timeout)
    pub timeout: Option<Duration>,
    /// Enable output compression
    pub compress_output: bool,
    /// Enable semantic output parsing
    pub parse_output: bool,
    /// Enable AI features
    pub enable_ai_features: bool,
    /// Context configuration
    pub context_config: ContextConfig,
    /// Agent role (optional)
    pub agent_role: Option<String>,
    /// Force headless (non-PTY) execution (useful for restricted sandboxes)
    pub force_headless: bool,
    /// Allow automatic fallback to headless mode when PTY creation fails
    pub allow_headless_fallback: bool,
}

/// Context configuration for AI features
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ContextConfig {
    /// Maximum tokens for context
    pub max_tokens: usize,
    /// Compression threshold (0.0 to 1.0)
    pub compression_threshold: f64,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            name: None,
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            environment: HashMap::new(),
            shell: None,
            shell_command: None,
            pty_size: (24, 80),
            output_buffer_size: 1024 * 1024, // 1MB
            timeout: None,
            compress_output: true,
            parse_output: true,
            enable_ai_features: false,
            context_config: ContextConfig::default(),
            agent_role: None,
            force_headless: false,
            allow_headless_fallback: true,
        }
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            max_tokens: 4096,
            compression_threshold: 0.8,
        }
    }
}

/// AI-optimized session
pub struct AISession {
    /// Unique session ID
    pub id: SessionId,
    /// Session configuration
    pub config: SessionConfig,
    /// Current status
    pub status: RwLock<SessionStatus>,
    /// Session context (AI state, history, etc.)
    pub context: Arc<RwLock<SessionContext>>,
    /// Process handle
    process: Arc<RwLock<Option<process::ProcessHandle>>>,
    /// Terminal handle (PTY or headless)
    terminal: Arc<RwLock<Option<terminal::TerminalHandle>>>,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity: Arc<RwLock<DateTime<Utc>>>,
    /// Session metadata
    pub metadata: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Command history tracking
    pub command_history: Arc<RwLock<Vec<CommandRecord>>>,
    /// Command count
    pub command_count: Arc<RwLock<usize>>,
    /// Total tokens used
    pub total_tokens: Arc<RwLock<usize>>,
}

impl AISession {
    /// Create a new AI session
    pub async fn new(config: SessionConfig) -> Result<Self> {
        let id = SessionId::new();
        let now = Utc::now();

        Ok(Self {
            id: id.clone(),
            config,
            status: RwLock::new(SessionStatus::Initializing),
            context: Arc::new(RwLock::new(SessionContext::new(id))),
            process: Arc::new(RwLock::new(None)),
            terminal: Arc::new(RwLock::new(None)),
            created_at: now,
            last_activity: Arc::new(RwLock::new(now)),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            command_history: Arc::new(RwLock::new(Vec::new())),
            command_count: Arc::new(RwLock::new(0)),
            total_tokens: Arc::new(RwLock::new(0)),
        })
    }

    /// Create an AI session with a specific ID (for restoration)
    pub async fn new_with_id(
        id: SessionId,
        config: SessionConfig,
        created_at: DateTime<Utc>,
    ) -> Result<Self> {
        let now = Utc::now();

        Ok(Self {
            id: id.clone(),
            config,
            status: RwLock::new(SessionStatus::Initializing),
            context: Arc::new(RwLock::new(SessionContext::new(id))),
            process: Arc::new(RwLock::new(None)),
            terminal: Arc::new(RwLock::new(None)),
            created_at,
            last_activity: Arc::new(RwLock::new(now)),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            command_history: Arc::new(RwLock::new(Vec::new())),
            command_count: Arc::new(RwLock::new(0)),
            total_tokens: Arc::new(RwLock::new(0)),
        })
    }

    /// Start the session
    pub async fn start(&self) -> Result<()> {
        lifecycle::start_session(self).await
    }

    /// Stop the session
    pub async fn stop(&self) -> Result<()> {
        lifecycle::stop_session(self).await
    }

    /// Send input to the session
    pub async fn send_input(&self, input: &str) -> Result<()> {
        let terminal_guard = self.terminal.read().await;
        if let Some(terminal) = terminal_guard.as_ref() {
            terminal.write(input.as_bytes()).await?;
            *self.last_activity.write().await = Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Session not started"))
        }
    }

    /// Read output from the session
    pub async fn read_output(&self) -> Result<Vec<u8>> {
        let terminal = self.terminal.read().await;
        if let Some(terminal) = terminal.as_ref() {
            let output = terminal.read().await?;
            *self.last_activity.write().await = Utc::now();
            Ok(output)
        } else {
            Err(anyhow::anyhow!("Session not started"))
        }
    }

    /// Get current session status
    pub async fn status(&self) -> SessionStatus {
        *self.status.read().await
    }

    /// Update session metadata
    pub async fn set_metadata(&self, key: String, value: serde_json::Value) -> Result<()> {
        self.metadata.write().await.insert(key, value);
        Ok(())
    }

    /// Get session metadata
    pub async fn get_metadata(&self, key: &str) -> Option<serde_json::Value> {
        self.metadata.read().await.get(key).cloned()
    }

    /// Execute a command and record it in history
    pub async fn execute_command(&self, command: &str) -> Result<String> {
        let start_time = Utc::now();

        // Send the command
        self.send_input(&format!("{}\n", command)).await?;

        // Wait for output
        tokio::time::sleep(Duration::from_millis(500)).await;
        let output_bytes = self.read_output().await?;
        let output = String::from_utf8_lossy(&output_bytes).to_string();

        // Record the command in history
        let end_time = Utc::now();
        let duration_ms = (end_time - start_time).num_milliseconds() as u64;

        let record = CommandRecord {
            command: command.to_string(),
            timestamp: start_time,
            exit_code: None, // TODO: Extract exit code from output
            output_preview: if output.len() > 200 {
                format!("{}...", &output[..200])
            } else {
                output.clone()
            },
            duration_ms,
        };

        // Update history and counters
        self.command_history.write().await.push(record);
        *self.command_count.write().await += 1;

        Ok(output)
    }

    /// Add tokens to the session total
    pub async fn add_tokens(&self, token_count: usize) {
        *self.total_tokens.write().await += token_count;
    }

    /// Get command history
    pub async fn get_command_history(&self) -> Vec<CommandRecord> {
        self.command_history.read().await.clone()
    }

    /// Get command count
    pub async fn get_command_count(&self) -> usize {
        *self.command_count.read().await
    }

    /// Get total tokens used
    pub async fn get_total_tokens(&self) -> usize {
        *self.total_tokens.read().await
    }

    /// Clear command history (keep recent N commands)
    pub async fn trim_command_history(&self, keep_recent: usize) {
        let mut history = self.command_history.write().await;
        if history.len() > keep_recent {
            let start_index = history.len() - keep_recent;
            history.drain(0..start_index);
        }
    }
}

/// AI-optimized session manager for creating and managing multiple terminal sessions.
///
/// The `SessionManager` provides a centralized way to create, track, and manage AI-enhanced
/// terminal sessions. It includes automatic cleanup, session restoration, and efficient
/// resource management.
///
/// # Features
///
/// - **Session Pooling**: Efficient management of multiple concurrent sessions
/// - **Automatic Cleanup**: Garbage collection of terminated sessions
/// - **Session Restoration**: Restore sessions from persistent storage
/// - **Resource Management**: Automatic cleanup and memory management
///
/// # Examples
///
/// ## Basic Session Management
///
/// ```no_run
/// use ai_session::{SessionManager, SessionConfig};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let manager = SessionManager::new();
///     
///     // Create multiple sessions
///     let session1 = manager.create_session().await?;
///     let session2 = manager.create_session().await?;
///     
///     session1.start().await?;
///     session2.start().await?;
///     
///     // List all active sessions
///     let session_ids = manager.list_sessions();
///     println!("Active sessions: {}", session_ids.len());
///     
///     // Clean up
///     manager.remove_session(&session1.id).await?;
///     manager.remove_session(&session2.id).await?;
///     
///     Ok(())
/// }
/// ```
///
/// ## Custom Configuration
///
/// ```no_run
/// use ai_session::{SessionManager, SessionConfig, ContextConfig};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let manager = SessionManager::new();
///     
///     // Configure for AI development agent
///     let mut config = SessionConfig::default();
///     config.enable_ai_features = true;
///     config.agent_role = Some("backend-developer".to_string());
///     config.working_directory = "/project/backend".into();
///     config.context_config = ContextConfig {
///         max_tokens: 8192,
///         compression_threshold: 0.8,
///     };
///     
///     let session = manager.create_session_with_config(config).await?;
///     session.start().await?;
///     
///     // Session is optimized for AI backend development
///     Ok(())
/// }
/// ```
///
/// ## Session Persistence
///
/// ```no_run
/// use ai_session::{SessionManager, SessionConfig, SessionId};
/// use chrono::Utc;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let manager = SessionManager::new();
///     
///     // Create session
///     let config = SessionConfig::default();
///     let session = manager.create_session_with_config(config.clone()).await?;
///     let session_id = session.id.clone();
///     let created_at = session.created_at;
///     
///     // Later, restore the session
///     let restored = manager.restore_session(session_id, config, created_at).await?;
///     restored.start().await?;
///     
///     Ok(())
/// }
/// ```
pub struct SessionManager {
    /// Active sessions
    sessions: Arc<DashMap<SessionId, Arc<AISession>>>,
    /// Default session configuration
    default_config: SessionConfig,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            default_config: SessionConfig::default(),
        }
    }

    /// Create a new session with default config
    pub async fn create_session(&self) -> Result<Arc<AISession>> {
        self.create_session_with_config(self.default_config.clone())
            .await
    }

    /// Create a new session with custom config
    pub async fn create_session_with_config(
        &self,
        config: SessionConfig,
    ) -> Result<Arc<AISession>> {
        let session = Arc::new(AISession::new(config).await?);
        self.sessions.insert(session.id.clone(), session.clone());
        Ok(session)
    }

    /// Restore a session with a specific ID (for persistence)
    pub async fn restore_session(
        &self,
        id: SessionId,
        config: SessionConfig,
        created_at: DateTime<Utc>,
    ) -> Result<Arc<AISession>> {
        // Check if session already exists
        if self.sessions.contains_key(&id) {
            return Err(SessionError::AlreadyExists(id).into());
        }

        let session = Arc::new(AISession::new_with_id(id.clone(), config, created_at).await?);
        self.sessions.insert(id, session.clone());
        Ok(session)
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &SessionId) -> Option<Arc<AISession>> {
        self.sessions.get(id).map(|entry| entry.clone())
    }

    /// List all active sessions
    pub fn list_sessions(&self) -> Vec<SessionId> {
        self.sessions
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// List all active session references
    pub fn list_session_refs(&self) -> Vec<Arc<AISession>> {
        self.sessions
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Remove a session
    pub async fn remove_session(&self, id: &SessionId) -> Result<()> {
        if let Some((_, session)) = self.sessions.remove(id) {
            session.stop().await?;
        }
        Ok(())
    }

    /// Clean up terminated sessions
    pub async fn cleanup_terminated(&self) -> Result<usize> {
        let mut removed = 0;
        let terminated_ids: Vec<SessionId> = self
            .sessions
            .iter()
            .filter(|entry| {
                let session = entry.value();
                if let Ok(status) = session.status.try_read() {
                    *status == SessionStatus::Terminated
                } else {
                    false
                }
            })
            .map(|entry| entry.key().clone())
            .collect();

        for id in terminated_ids {
            self.sessions.remove(&id);
            removed += 1;
        }

        Ok(removed)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Session error types

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_id() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_session_manager() {
        let manager = SessionManager::new();
        let session = manager.create_session().await.unwrap();

        assert!(manager.get_session(&session.id).is_some());
        assert_eq!(manager.list_sessions().len(), 1);

        manager.remove_session(&session.id).await.unwrap();
        assert!(manager.get_session(&session.id).is_none());
    }
}
