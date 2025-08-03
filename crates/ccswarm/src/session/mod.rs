pub mod ai_session_adapter;
pub mod base_session;
pub mod claude_session;
pub mod context_bridge;
pub mod coordinator;
pub mod error;
pub mod memory;
pub mod persistent_session;
pub mod session_pool;
pub mod traits;
pub mod worktree_session;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use self::error::{LockResultExt, SessionError, SessionResult};

use crate::auto_accept::AutoAcceptConfig;
use crate::identity::AgentRole;
use crate::resource::{ResourceMonitor, SessionResourceIntegration};
use ai_session::native::NativeSessionManager;
use memory::{
    EpisodeOutcome, EpisodeType, MemorySummary, RetrievalResult, SessionMemory, WorkingMemoryType,
};

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

    /// Integrated memory system for this session
    pub memory: SessionMemory,
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

        let memory = SessionMemory::new(session_id.clone(), agent_id.clone());

        Self {
            id: session_id,
            agent_id: agent_id.clone(),
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
            memory,
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

    /// Add item to working memory
    pub fn add_memory(&mut self, content: String, item_type: WorkingMemoryType, priority: f32) {
        self.memory
            .add_to_working_memory(content, item_type, priority);
        self.touch();
    }

    /// Set current task context
    pub fn set_task_context(&mut self, task_id: String, description: String) {
        self.memory.set_task_context(task_id, description);
        self.touch();
    }

    /// Add episode to memory
    pub fn add_episode(
        &mut self,
        event_type: EpisodeType,
        description: String,
        context: HashMap<String, String>,
        outcome: EpisodeOutcome,
    ) {
        self.memory
            .add_episode(event_type, description, context, outcome);
        self.touch();
    }

    /// Consolidate memories
    pub fn consolidate_memories(&mut self) {
        self.memory.consolidate_memories();
        self.touch();
    }

    /// Retrieve relevant memories
    pub fn retrieve_memories(&self, query: &str) -> RetrievalResult {
        self.memory.retrieve_relevant_memories(query)
    }

    /// Get memory summary
    pub fn get_memory_summary(&self) -> MemorySummary {
        self.memory.generate_memory_summary()
    }
}

/// Manages multiple agent sessions with native session management
pub struct SessionManager {
    /// Map of session ID to agent session
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
    /// Native session manager for session operations
    session_manager: NativeSessionManager,
    /// Resource monitor for tracking agent resources
    resource_monitor: Option<Arc<ResourceMonitor>>,
    /// Resource integration handler
    resource_integration: Option<Arc<SessionResourceIntegration>>,
}

impl SessionManager {
    /// Creates a new session manager
    pub async fn new() -> SessionResult<Self> {
        Ok(Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            session_manager: NativeSessionManager::new(),
            resource_monitor: None,
            resource_integration: None,
        })
    }

    /// Creates a new session manager with resource monitoring
    pub async fn with_resource_monitoring(
        resource_limits: crate::resource::ResourceLimits,
    ) -> SessionResult<Self> {
        let resource_monitor = Arc::new(ResourceMonitor::new(resource_limits));
        let resource_integration =
            Arc::new(SessionResourceIntegration::new(resource_monitor.clone()));

        // Start the monitoring loop
        let monitor_clone = resource_monitor.clone();
        tokio::spawn(async move {
            monitor_clone.start_monitoring_loop().await;
        });

        Ok(Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            session_manager: NativeSessionManager::new(),
            resource_monitor: Some(resource_monitor),
            resource_integration: Some(resource_integration),
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
    pub async fn create_session(
        &self,
        agent_id: String,
        agent_role: AgentRole,
        working_directory: String,
        description: Option<String>,
        auto_start: bool,
    ) -> SessionResult<AgentSession> {
        let session =
            AgentSession::new(agent_id, agent_role, working_directory.clone(), description);

        // Create the native session
        let _native_session = self
            .session_manager
            .create_session(&session.tmux_session)
            .await
            .map_err(|e| SessionError::CreationFailed {
                reason: e.to_string(),
            })?;

        // Set up the session environment
        self.setup_session_environment(&session).await?;

        if auto_start {
            // Start the agent in the session
            self.start_agent_in_session(&session).await?;
        }

        // Store the session
        {
            let mut sessions = self.sessions.lock().map_lock_error()?;
            sessions.insert(session.id.clone(), session.clone());
        }

        // Start resource monitoring if enabled
        if let Some(ref integration) = self.resource_integration {
            if let Err(e) = integration
                .on_session_created(&session.id, &session.agent_id, None)
                .await
            {
                tracing::warn!("Failed to start resource monitoring: {}", e);
            }
        }

        Ok(session)
    }

    /// Pauses an active session
    ///
    /// # Arguments
    /// * `session_id` - ID of the session to pause
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found or cannot be paused
    pub async fn pause_session(&self, session_id: &str) -> SessionResult<()> {
        let mut sessions = self.sessions.lock().map_lock_error()?;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SessionError::NotFound {
                id: session_id.to_string(),
            })?;

        match session.status {
            SessionStatus::Active | SessionStatus::Background => {
                // For native sessions, we just update the status
                // The actual pause/resume will be handled by the session itself
                session.status = SessionStatus::Paused;
                session.touch();
                Ok(())
            }
            _ => Err(SessionError::InvalidState {
                state: format!("{:?}", session.status),
                operation: "pause".to_string(),
            }),
        }
    }

    /// Resumes a paused session
    ///
    /// # Arguments
    /// * `session_id` - ID of the session to resume
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found or cannot be resumed
    pub async fn resume_session(&self, session_id: &str) -> SessionResult<()> {
        let mut sessions = self.sessions.lock().map_lock_error()?;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SessionError::NotFound {
                id: session_id.to_string(),
            })?;

        match session.status {
            SessionStatus::Paused => {
                // For native sessions, we just update the status
                session.status = if session.background_mode {
                    SessionStatus::Background
                } else {
                    SessionStatus::Active
                };
                session.touch();
                Ok(())
            }
            _ => Err(SessionError::InvalidState {
                state: format!("{:?}", session.status),
                operation: "resume".to_string(),
            }),
        }
    }

    /// Detaches a session from the current terminal
    ///
    /// # Arguments
    /// * `session_id` - ID of the session to detach
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found or cannot be detached
    pub async fn detach_session(&self, session_id: &str) -> SessionResult<()> {
        let mut sessions = self.sessions.lock().map_lock_error()?;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SessionError::NotFound {
                id: session_id.to_string(),
            })?;

        match session.status {
            SessionStatus::Active | SessionStatus::Background => {
                // For native sessions, detaching just changes status
                session.status = SessionStatus::Detached;
                session.touch();
                Ok(())
            }
            _ => Err(SessionError::InvalidState {
                state: format!("{:?}", session.status),
                operation: "detach".to_string(),
            }),
        }
    }

    /// Attaches to a detached session
    ///
    /// # Arguments
    /// * `session_id` - ID of the session to attach
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found or cannot be attached
    pub async fn attach_session(&self, session_id: &str) -> SessionResult<()> {
        let mut sessions = self.sessions.lock().map_lock_error()?;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SessionError::NotFound {
                id: session_id.to_string(),
            })?;

        match session.status {
            SessionStatus::Detached => {
                // For native sessions, attaching just changes status
                session.status = if session.background_mode {
                    SessionStatus::Background
                } else {
                    SessionStatus::Active
                };
                session.touch();
                Ok(())
            }
            _ => Err(SessionError::InvalidState {
                state: format!("{:?}", session.status),
                operation: "attach".to_string(),
            }),
        }
    }

    /// Terminates a session
    ///
    /// # Arguments
    /// * `session_id` - ID of the session to terminate
    ///
    /// # Returns
    /// Ok(()) on success, error if session not found
    pub async fn terminate_session(&self, session_id: &str) -> SessionResult<()> {
        let (tmux_session, agent_id) = {
            let sessions = self.sessions.lock().map_lock_error()?;
            let session = sessions
                .get(session_id)
                .ok_or_else(|| SessionError::NotFound {
                    id: session_id.to_string(),
                })?;
            (session.tmux_session.clone(), session.agent_id.clone())
        };

        // Stop resource monitoring if enabled
        if let Some(ref integration) = self.resource_integration {
            if let Err(e) = integration
                .on_session_terminated(session_id, &agent_id)
                .await
            {
                tracing::warn!("Failed to stop resource monitoring: {}", e);
            }
        }

        // Terminate the native session
        if self.session_manager.has_session(&tmux_session).await {
            self.session_manager
                .delete_session(&tmux_session)
                .await
                .map_err(|e| SessionError::TerminationFailed {
                    reason: e.to_string(),
                })?;
        }

        // Update session status
        {
            let mut sessions = self.sessions.lock().map_lock_error()?;
            if let Some(session) = sessions.get_mut(session_id) {
                session.status = SessionStatus::Terminated;
                session.touch();
            }
        }

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
    pub async fn set_background_mode(
        &self,
        session_id: &str,
        auto_accept: bool,
    ) -> SessionResult<()> {
        let mut sessions = self.sessions.lock().map_lock_error()?;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SessionError::NotFound {
                id: session_id.to_string(),
            })?;

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
    pub async fn enable_auto_accept(
        &self,
        session_id: &str,
        config: AutoAcceptConfig,
    ) -> SessionResult<()> {
        let mut sessions = self.sessions.lock().map_lock_error()?;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SessionError::NotFound {
                id: session_id.to_string(),
            })?;

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
    pub async fn disable_auto_accept(&self, session_id: &str) -> SessionResult<()> {
        let mut sessions = self.sessions.lock().map_lock_error()?;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SessionError::NotFound {
                id: session_id.to_string(),
            })?;

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
    pub async fn update_auto_accept_config(
        &self,
        session_id: &str,
        config: AutoAcceptConfig,
    ) -> SessionResult<()> {
        let mut sessions = self.sessions.lock().map_lock_error()?;

        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| SessionError::NotFound {
                id: session_id.to_string(),
            })?;

        session.update_auto_accept_config(config);

        Ok(())
    }

    /// Emergency stops auto-accept for all sessions
    ///
    /// # Returns
    /// Number of sessions that had auto-accept disabled
    pub async fn emergency_stop_all_auto_accept(&self) -> usize {
        if let Ok(mut sessions) = self.sessions.lock() {
            let mut count = 0;

            for session in sessions.values_mut() {
                if session.auto_accept {
                    session.disable_auto_accept();
                    count += 1;
                }
            }

            count
        } else {
            // If we can't acquire the lock, return 0
            0
        }
    }

    /// Gets sessions with auto-accept enabled
    pub fn get_auto_accept_sessions(&self) -> Vec<AgentSession> {
        if let Ok(sessions) = self.sessions.lock() {
            sessions
                .values()
                .filter(|s| s.is_auto_accept_ready())
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Gets a session by ID
    pub fn get_session(&self, session_id: &str) -> Option<AgentSession> {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.get(session_id).cloned()
        } else {
            None
        }
    }

    /// Lists all sessions
    pub fn list_sessions(&self) -> Vec<AgentSession> {
        if let Ok(sessions) = self.sessions.lock() {
            sessions.values().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Lists active sessions only
    pub fn list_active_sessions(&self) -> Vec<AgentSession> {
        if let Ok(sessions) = self.sessions.lock() {
            sessions
                .values()
                .filter(|s| s.is_runnable())
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Gets sessions by agent role
    pub fn get_sessions_by_role(&self, role: AgentRole) -> Vec<AgentSession> {
        if let Ok(sessions) = self.sessions.lock() {
            sessions
                .values()
                .filter(|s| s.agent_role == role)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Cleans up terminated sessions
    pub async fn cleanup_terminated_sessions(&self) -> SessionResult<usize> {
        let mut sessions = self.sessions.lock().map_lock_error()?;
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

    /// Check for idle agents and suspend them if needed
    pub async fn check_and_suspend_idle_agents(&self) -> SessionResult<Vec<String>> {
        let mut suspended_agents = Vec::new();

        if let Some(ref integration) = self.resource_integration {
            let agents_to_check: Vec<(String, String)> = {
                let sessions = self.sessions.lock().map_lock_error()?;
                sessions
                    .values()
                    .filter(|s| s.is_runnable())
                    .map(|s| (s.id.clone(), s.agent_id.clone()))
                    .collect()
            };

            for (session_id, agent_id) in agents_to_check {
                if integration.check_agent_suspension(&agent_id).await {
                    if let Err(e) = self.pause_session(&session_id).await {
                        tracing::warn!("Failed to suspend idle agent {}: {}", agent_id, e);
                    } else {
                        suspended_agents.push(agent_id);
                    }
                }
            }
        }

        Ok(suspended_agents)
    }

    /// Get resource usage for a session
    pub fn get_session_resource_usage(
        &self,
        session_id: &str,
    ) -> Option<crate::resource::ResourceUsage> {
        if let Ok(sessions) = self.sessions.lock() {
            let session = sessions.get(session_id)?;

            self.resource_monitor
                .as_ref()
                .and_then(|monitor| monitor.get_agent_usage(&session.agent_id))
        } else {
            None
        }
    }

    /// Get resource efficiency statistics
    pub fn get_resource_efficiency_stats(
        &self,
    ) -> Option<crate::resource::ResourceEfficiencyStats> {
        self.resource_monitor
            .as_ref()
            .map(|monitor| monitor.get_efficiency_stats())
    }

    /// Sets up the environment for a new session
    async fn setup_session_environment(&self, session: &AgentSession) -> SessionResult<()> {
        // For native sessions, environment setup would be handled during session creation
        // We can set environment variables if the native session supports it
        if let Some(native_session) = self
            .session_manager
            .get_session(&session.tmux_session)
            .await
        {
            let env_commands = vec![
                format!("export CCSWARM_SESSION_ID={}", session.id),
                format!("export CCSWARM_AGENT_ID={}", session.agent_id),
                format!("export CCSWARM_AGENT_ROLE={}", session.agent_role.name()),
            ];

            let session_lock = native_session.lock().await;
            for cmd in env_commands {
                session_lock.send_input(&format!("{}\n", cmd)).await?;
            }
        }

        Ok(())
    }

    /// Starts the agent in the session
    async fn start_agent_in_session(&self, session: &AgentSession) -> SessionResult<()> {
        // Build the command to start the agent
        let command = format!(
            "ccswarm agent start --id {} --role {} --session {} --working-dir {}",
            session.agent_id,
            session.agent_role.name(),
            session.id,
            session.working_directory
        );

        // Send the command to the native session
        if let Some(native_session) = self
            .session_manager
            .get_session(&session.tmux_session)
            .await
        {
            let session_lock = native_session.lock().await;
            session_lock.send_input(&format!("{}\n", command)).await?;
        }

        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        // Note: This is a blocking call in an async context
        // Consider using lazy_static or once_cell for production
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create with default resource monitoring enabled
                Self::with_resource_monitoring(crate::resource::ResourceLimits::default())
                    .await
                    .expect("Failed to create SessionManager")
            })
        })
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
