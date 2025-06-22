pub mod claude_session;
pub mod coordinator;
pub mod memory;
pub mod persistent_session;
pub mod session_pool;
pub mod worktree_session;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::auto_accept::AutoAcceptConfig;
use crate::identity::AgentRole;
use crate::tmux::TmuxClient;

/// Represents the current status of an agent session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is actively running and processing tasks
    Active,
    /// Session is temporarily paused but can be resumed
    Paused,
    /// Session is detached from current terminal but still running
    Detached,
    /// Session is running in background mode with auto-acceptance
    Background,
    /// Session has been terminated
    Terminated,
    /// Session encountered an error
    Error(String),
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "Active"),
            SessionStatus::Paused => write!(f, "Paused"),
            SessionStatus::Detached => write!(f, "Detached"),
            SessionStatus::Background => write!(f, "Background"),
            SessionStatus::Terminated => write!(f, "Terminated"),
            SessionStatus::Error(e) => write!(f, "Error: {}", e),
        }
    }
}

/// Represents an active agent session with tmux integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    /// Unique identifier for this session
    pub id: String,
    /// ID of the agent running in this session
    pub agent_id: String,
    /// Role of the agent in this session
    pub agent_role: AgentRole,
    /// Name of the tmux session
    pub tmux_session: String,
    /// Current status of the session
    pub status: SessionStatus,
    /// Whether the session is in background mode
    pub background_mode: bool,
    /// Whether tasks are automatically accepted in this session
    pub auto_accept: bool,

    /// Configuration for auto-accept mode (if enabled)
    pub auto_accept_config: Option<AutoAcceptConfig>,
    /// Timestamp when the session was created
    pub created_at: DateTime<Utc>,
    /// Timestamp of the last activity in this session
    pub last_activity: DateTime<Utc>,
    /// Optional description or notes about the session
    pub description: Option<String>,
    /// Current working directory for the session
    pub working_directory: String,
    /// Number of tasks processed in this session
    pub tasks_processed: usize,
    /// Number of tasks currently in queue
    pub tasks_queued: usize,
}

impl AgentSession {
    /// Creates a new agent session
    pub fn new(
        agent_id: String,
        agent_role: AgentRole,
        working_directory: String,
        description: Option<String>,
    ) -> Self {
        let session_id = Uuid::new_v4().to_string();
        let tmux_session = format!(
            "ccswarm-{}-{}",
            agent_role.name().to_lowercase(),
            &session_id[..8]
        );

        Self {
            id: session_id,
            agent_id,
            agent_role,
            tmux_session,
            status: SessionStatus::Active,
            background_mode: false,
            auto_accept: false,
            auto_accept_config: None,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            description,
            working_directory,
            tasks_processed: 0,
            tasks_queued: 0,
        }
    }

    /// Updates the last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Utc::now();
    }

    /// Checks if the session is in a runnable state
    pub fn is_runnable(&self) -> bool {
        matches!(
            self.status,
            SessionStatus::Active | SessionStatus::Background | SessionStatus::Detached
        )
    }

    /// Increments the processed task counter
    pub fn increment_tasks_processed(&mut self) {
        self.tasks_processed += 1;
        self.touch();
    }

    /// Enables auto-accept mode with the given configuration
    pub fn enable_auto_accept(&mut self, config: AutoAcceptConfig) {
        self.auto_accept = true;
        self.auto_accept_config = Some(config);
        self.touch();
    }

    /// Disables auto-accept mode
    pub fn disable_auto_accept(&mut self) {
        self.auto_accept = false;
        self.auto_accept_config = None;
        self.touch();
    }

    /// Updates auto-accept configuration
    pub fn update_auto_accept_config(&mut self, config: AutoAcceptConfig) {
        if self.auto_accept {
            self.auto_accept_config = Some(config);
            self.touch();
        }
    }

    /// Gets the current auto-accept configuration
    pub fn get_auto_accept_config(&self) -> Option<&AutoAcceptConfig> {
        self.auto_accept_config.as_ref()
    }

    /// Checks if auto-accept is enabled and properly configured
    pub fn is_auto_accept_ready(&self) -> bool {
        self.auto_accept && self.auto_accept_config.is_some()
    }
}

/// Manages multiple agent sessions with tmux integration
pub struct SessionManager {
    /// Map of session ID to agent session
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
    /// Tmux client for session operations
    tmux_client: TmuxClient,
}

impl SessionManager {
    /// Creates a new session manager
    pub fn new() -> Result<Self> {
        Ok(Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            tmux_client: TmuxClient::new()?,
        })
    }

    /// Creates a new agent session
    ///
    /// # Arguments
    /// * `agent_id` - ID of the agent to run in this session
    /// * `agent_role` - Role of the agent
    /// * `working_directory` - Working directory for the session
    /// * `description` - Optional description for the session
    /// * `auto_start` - Whether to automatically start the session
    ///
    /// # Returns
    /// The created AgentSession on success
    pub fn create_session(
        &self,
        agent_id: String,
        agent_role: AgentRole,
        working_directory: String,
        description: Option<String>,
        auto_start: bool,
    ) -> Result<AgentSession> {
        let session =
            AgentSession::new(agent_id, agent_role, working_directory.clone(), description);

        // Create the tmux session
        self.tmux_client
            .create_session(&session.tmux_session, &working_directory)?;

        // Set up the session environment
        self.setup_session_environment(&session)?;

        if auto_start {
            // Start the agent in the session
            self.start_agent_in_session(&session)?;
        }

        // Store the session
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session.id.clone(), session.clone());

        Ok(session)
    }

    /// Pauses an active session
    ///
    /// # Arguments
    /// * `session_id` - ID of the session to pause
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found or cannot be paused
    pub fn pause_session(&self, session_id: &str) -> Result<(), anyhow::Error> {
        let mut sessions = self.sessions.lock().unwrap();

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session {} not found", session_id))?;

        match session.status {
            SessionStatus::Active | SessionStatus::Background => {
                // Send pause signal to the tmux session
                self.tmux_client.send_keys(&session.tmux_session, "C-z")?;
                session.status = SessionStatus::Paused;
                session.touch();
                Ok(())
            }
            _ => Err(anyhow::anyhow!(
                "Cannot pause session in {} state",
                session.status
            )),
        }
    }

    /// Resumes a paused session
    ///
    /// # Arguments
    /// * `session_id` - ID of the session to resume
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found or cannot be resumed
    pub fn resume_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session {} not found", session_id))?;

        match session.status {
            SessionStatus::Paused => {
                // Send resume signal to the tmux session
                self.tmux_client.send_command(&session.tmux_session, "fg")?;
                session.status = if session.background_mode {
                    SessionStatus::Background
                } else {
                    SessionStatus::Active
                };
                session.touch();
                Ok(())
            }
            _ => Err(anyhow!("Cannot resume session in {} state", session.status)),
        }
    }

    /// Detaches a session from the current terminal
    ///
    /// # Arguments
    /// * `session_id` - ID of the session to detach
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found or cannot be detached
    pub fn detach_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session {} not found", session_id))?;

        match session.status {
            SessionStatus::Active | SessionStatus::Background => {
                // Detach the tmux session
                self.tmux_client.detach_session(&session.tmux_session)?;
                session.status = SessionStatus::Detached;
                session.touch();
                Ok(())
            }
            _ => Err(anyhow!("Cannot detach session in {} state", session.status)),
        }
    }

    /// Attaches to a detached session
    ///
    /// # Arguments
    /// * `session_id` - ID of the session to attach
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found or cannot be attached
    pub fn attach_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session {} not found", session_id))?;

        match session.status {
            SessionStatus::Detached => {
                // Attach to the tmux session
                self.tmux_client.attach_session(&session.tmux_session)?;
                session.status = if session.background_mode {
                    SessionStatus::Background
                } else {
                    SessionStatus::Active
                };
                session.touch();
                Ok(())
            }
            _ => Err(anyhow!(
                "Cannot attach to session in {} state",
                session.status
            )),
        }
    }

    /// Terminates a session
    ///
    /// # Arguments
    /// * `session_id` - ID of the session to terminate
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found
    pub fn terminate_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session {} not found", session_id))?;

        // Kill the tmux session
        self.tmux_client.kill_session(&session.tmux_session)?;
        session.status = SessionStatus::Terminated;
        session.touch();

        Ok(())
    }

    /// Sets a session to background mode
    ///
    /// # Arguments
    /// * `session_id` - ID of the session
    /// * `auto_accept` - Whether to automatically accept tasks
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found
    pub fn set_background_mode(&self, session_id: &str, auto_accept: bool) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session {} not found", session_id))?;

        session.background_mode = true;
        if auto_accept && session.auto_accept_config.is_none() {
            // Enable auto-accept with default configuration if not already configured
            session.enable_auto_accept(AutoAcceptConfig::default());
        } else {
            session.auto_accept = auto_accept;
        }

        if session.status == SessionStatus::Active {
            session.status = SessionStatus::Background;
        }
        session.touch();

        Ok(())
    }

    /// Enables auto-accept mode for a session
    ///
    /// # Arguments
    /// * `session_id` - ID of the session
    /// * `config` - Auto-accept configuration
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found
    pub fn enable_auto_accept(&self, session_id: &str, config: AutoAcceptConfig) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session {} not found", session_id))?;

        session.enable_auto_accept(config);

        Ok(())
    }

    /// Disables auto-accept mode for a session
    ///
    /// # Arguments
    /// * `session_id` - ID of the session
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found
    pub fn disable_auto_accept(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session {} not found", session_id))?;

        session.disable_auto_accept();

        Ok(())
    }

    /// Updates auto-accept configuration for a session
    ///
    /// # Arguments
    /// * `session_id` - ID of the session
    /// * `config` - New auto-accept configuration
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found
    pub fn update_auto_accept_config(
        &self,
        session_id: &str,
        config: AutoAcceptConfig,
    ) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session {} not found", session_id))?;

        session.update_auto_accept_config(config);

        Ok(())
    }

    /// Emergency stops auto-accept for all sessions
    ///
    /// # Returns
    /// Number of sessions that had auto-accept disabled
    pub fn emergency_stop_all_auto_accept(&self) -> usize {
        let mut sessions = self.sessions.lock().unwrap();
        let mut count = 0;

        for session in sessions.values_mut() {
            if session.auto_accept {
                session.disable_auto_accept();
                count += 1;
            }
        }

        count
    }

    /// Gets sessions with auto-accept enabled
    pub fn get_auto_accept_sessions(&self) -> Vec<AgentSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .values()
            .filter(|s| s.is_auto_accept_ready())
            .cloned()
            .collect()
    }

    /// Gets a session by ID
    pub fn get_session(&self, session_id: &str) -> Option<AgentSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).cloned()
    }

    /// Lists all sessions
    pub fn list_sessions(&self) -> Vec<AgentSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions.values().cloned().collect()
    }

    /// Lists active sessions only
    pub fn list_active_sessions(&self) -> Vec<AgentSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .values()
            .filter(|s| s.is_runnable())
            .cloned()
            .collect()
    }

    /// Gets sessions by agent role
    pub fn get_sessions_by_role(&self, role: AgentRole) -> Vec<AgentSession> {
        let sessions = self.sessions.lock().unwrap();
        sessions
            .values()
            .filter(|s| s.agent_role == role)
            .cloned()
            .collect()
    }

    /// Cleans up terminated sessions
    pub fn cleanup_terminated_sessions(&self) -> Result<usize> {
        let mut sessions = self.sessions.lock().unwrap();
        let terminated: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| s.status == SessionStatus::Terminated)
            .map(|(id, _)| id.clone())
            .collect();

        let count = terminated.len();
        for id in terminated {
            sessions.remove(&id);
        }

        Ok(count)
    }

    /// Sets up the environment for a new session
    fn setup_session_environment(&self, session: &AgentSession) -> Result<()> {
        // Set environment variables
        self.tmux_client.set_environment(
            &session.tmux_session,
            "CCSWARM_SESSION_ID",
            &session.id,
        )?;
        self.tmux_client.set_environment(
            &session.tmux_session,
            "CCSWARM_AGENT_ID",
            &session.agent_id,
        )?;
        self.tmux_client.set_environment(
            &session.tmux_session,
            "CCSWARM_AGENT_ROLE",
            session.agent_role.name(),
        )?;

        // Create status line
        let status_line = format!(
            "[{}] Agent: {} | Session: {} | Status: {{}}",
            session.agent_role.name(),
            &session.agent_id[..8],
            &session.id[..8]
        );
        self.tmux_client
            .set_option(&session.tmux_session, "status-right", &status_line)?;

        Ok(())
    }

    /// Starts the agent in the session
    fn start_agent_in_session(&self, session: &AgentSession) -> Result<()> {
        // Build the command to start the agent
        let command = format!(
            "ccswarm agent start --id {} --role {} --session {} --working-dir {}",
            session.agent_id,
            session.agent_role.name(),
            session.id,
            session.working_directory
        );

        // Send the command to the tmux session
        self.tmux_client
            .send_command(&session.tmux_session, &command)?;

        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create SessionManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_session_creation() {
        use crate::identity::default_frontend_role;

        let session = AgentSession::new(
            "agent-123".to_string(),
            default_frontend_role(),
            "/tmp/test".to_string(),
            Some("Test session".to_string()),
        );

        assert_eq!(session.agent_id, "agent-123");
        assert_eq!(session.agent_role.name(), "Frontend");
        assert_eq!(session.status, SessionStatus::Active);
        assert!(!session.background_mode);
        assert!(!session.auto_accept);
        assert!(session.auto_accept_config.is_none());
        assert_eq!(session.tasks_processed, 0);
    }

    #[test]
    fn test_session_is_runnable() {
        use crate::identity::default_backend_role;

        let mut session = AgentSession::new(
            "agent-123".to_string(),
            default_backend_role(),
            "/tmp/test".to_string(),
            None,
        );

        assert!(session.is_runnable());

        session.status = SessionStatus::Paused;
        assert!(!session.is_runnable());

        session.status = SessionStatus::Background;
        assert!(session.is_runnable());

        session.status = SessionStatus::Terminated;
        assert!(!session.is_runnable());
    }

    #[test]
    fn test_session_status_display() {
        assert_eq!(SessionStatus::Active.to_string(), "Active");
        assert_eq!(SessionStatus::Paused.to_string(), "Paused");
        assert_eq!(SessionStatus::Background.to_string(), "Background");
        assert_eq!(
            SessionStatus::Error("test error".to_string()).to_string(),
            "Error: test error"
        );
    }

    #[test]
    fn test_auto_accept_configuration() {
        use crate::identity::default_frontend_role;

        let mut session = AgentSession::new(
            "agent-123".to_string(),
            default_frontend_role(),
            "/tmp/test".to_string(),
            None,
        );

        // Initially auto-accept should be disabled
        assert!(!session.auto_accept);
        assert!(!session.is_auto_accept_ready());
        assert!(session.get_auto_accept_config().is_none());

        // Enable auto-accept
        let config = AutoAcceptConfig {
            enabled: true,
            max_file_changes: 3,
            ..AutoAcceptConfig::default()
        };
        session.enable_auto_accept(config.clone());

        assert!(session.auto_accept);
        assert!(session.is_auto_accept_ready());
        assert!(session.get_auto_accept_config().is_some());
        assert_eq!(
            session.get_auto_accept_config().unwrap().max_file_changes,
            3
        );

        // Update configuration
        let new_config = AutoAcceptConfig {
            enabled: true,
            max_file_changes: 5,
            ..AutoAcceptConfig::default()
        };
        session.update_auto_accept_config(new_config);
        assert_eq!(
            session.get_auto_accept_config().unwrap().max_file_changes,
            5
        );

        // Disable auto-accept
        session.disable_auto_accept();
        assert!(!session.auto_accept);
        assert!(!session.is_auto_accept_ready());
        assert!(session.get_auto_accept_config().is_none());
    }
}
