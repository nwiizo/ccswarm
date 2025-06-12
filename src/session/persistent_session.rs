/// Session management for persistent Claude Code agents
///
/// This module provides session lifecycle management, pooling, and cleanup
/// for the Session-Persistent Agent Architecture. Key features:
/// - Automatic session creation and reuse
/// - Lifecycle management with timeouts
/// - Resource cleanup and garbage collection
/// - Session health monitoring
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use uuid::Uuid;

use crate::agent::persistent::{PersistentClaudeAgent, SessionStats};
use crate::agent::{Task, TaskResult};
use crate::config::ClaudeConfig;
use crate::identity::{AgentIdentity, AgentRole};

/// Type alias for session storage
type SessionStorage = Arc<RwLock<HashMap<String, Arc<Mutex<PersistentClaudeAgent>>>>>;

/// Session information for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentSessionInfo {
    pub session_id: String,
    pub agent_id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub status: PersistentSessionStatus,
    pub tasks_completed: usize,
    pub workspace_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PersistentSessionStatus {
    Creating,
    Active,
    Idle,
    Shutting,
    Terminated,
}

/// Configuration for session management
#[derive(Debug, Clone)]
pub struct PersistentSessionManagerConfig {
    /// Maximum number of concurrent sessions
    pub max_concurrent_sessions: usize,

    /// Session idle timeout before cleanup
    pub idle_timeout: Duration,

    /// Cleanup interval
    pub cleanup_interval: Duration,

    /// Maximum session lifetime
    pub max_session_lifetime: Duration,

    /// Enable session reuse
    pub enable_session_reuse: bool,
}

impl Default for PersistentSessionManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_sessions: 10,
            idle_timeout: Duration::from_secs(300), // 5 minutes
            cleanup_interval: Duration::from_secs(60), // 1 minute
            max_session_lifetime: Duration::from_secs(3600), // 1 hour
            enable_session_reuse: true,
        }
    }
}

/// Manages persistent Claude Code agent sessions
#[derive(Debug)]
pub struct PersistentSessionManager {
    /// Active sessions by agent ID
    sessions: Arc<RwLock<HashMap<String, Arc<Mutex<PersistentClaudeAgent>>>>>,

    /// Session metadata
    session_info: Arc<RwLock<HashMap<String, PersistentSessionInfo>>>,

    /// Configuration
    config: PersistentSessionManagerConfig,

    /// Workspace root path
    workspace_root: PathBuf,

    /// Cleanup task handle
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

impl PersistentSessionManager {
    /// Create a new session manager
    pub fn new(workspace_root: PathBuf, config: PersistentSessionManagerConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_info: Arc::new(RwLock::new(HashMap::new())),
            config,
            workspace_root,
            cleanup_handle: None,
        }
    }

    /// Start the session manager (begins cleanup task)
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting persistent session manager");

        let sessions = Arc::clone(&self.sessions);
        let session_info = Arc::clone(&self.session_info);
        let cleanup_interval = self.config.cleanup_interval;
        let idle_timeout = self.config.idle_timeout;
        let max_lifetime = self.config.max_session_lifetime;

        let cleanup_handle = tokio::spawn(async move {
            let mut interval = interval(cleanup_interval);

            loop {
                interval.tick().await;

                if let Err(e) =
                    Self::cleanup_sessions(&sessions, &session_info, idle_timeout, max_lifetime)
                        .await
                {
                    tracing::error!("Session cleanup error: {}", e);
                }
            }
        });

        self.cleanup_handle = Some(cleanup_handle);
        Ok(())
    }

    /// Get or create a session for an agent
    pub async fn get_or_create_session(
        &self,
        role: AgentRole,
        claude_config: ClaudeConfig,
    ) -> Result<Arc<Mutex<PersistentClaudeAgent>>> {
        let agent_id = format!("{}-agent-{}", role.name().to_lowercase(), Uuid::new_v4());

        // Check if we can reuse an existing session
        if self.config.enable_session_reuse {
            if let Some(existing) = self.find_reusable_session(&role).await {
                tracing::info!("Reusing existing session for role: {}", role.name());
                return Ok(existing);
            }
        }

        // Check session limits
        let current_count = self.sessions.read().await.len();
        if current_count >= self.config.max_concurrent_sessions {
            return Err(anyhow::anyhow!(
                "Maximum concurrent sessions reached: {}",
                self.config.max_concurrent_sessions
            ));
        }

        // Create new session
        self.create_new_session(agent_id, role, claude_config).await
    }

    /// Find a reusable session for the given role
    async fn find_reusable_session(
        &self,
        role: &AgentRole,
    ) -> Option<Arc<Mutex<PersistentClaudeAgent>>> {
        let sessions = self.sessions.read().await;
        let session_info = self.session_info.read().await;

        for (agent_id, session) in sessions.iter() {
            if let Some(info) = session_info.get(agent_id) {
                if info.status == PersistentSessionStatus::Idle {
                    let agent = session.lock().await;
                    if agent.identity.specialization.name() == role.name() {
                        // Found a reusable session
                        return Some(Arc::clone(session));
                    }
                }
            }
        }

        None
    }

    /// Create a new session
    async fn create_new_session(
        &self,
        agent_id: String,
        role: AgentRole,
        claude_config: ClaudeConfig,
    ) -> Result<Arc<Mutex<PersistentClaudeAgent>>> {
        tracing::info!("Creating new persistent session for agent: {}", agent_id);

        // Create workspace path
        let workspace_path = self.workspace_root.join(format!("agents/{}", &agent_id));
        tokio::fs::create_dir_all(&workspace_path)
            .await
            .context("Failed to create agent workspace")?;

        // Create agent identity
        let identity = AgentIdentity {
            agent_id: agent_id.clone(),
            specialization: role,
            workspace_path,
            env_vars: Self::create_env_vars(&agent_id),
            session_id: Uuid::new_v4().to_string(),
            parent_process_id: std::process::id().to_string(),
            initialized_at: Utc::now(),
        };

        // Create persistent agent
        let agent = PersistentClaudeAgent::new(identity, claude_config).await?;
        let agent = Arc::new(Mutex::new(agent));

        // Create session info
        let session_info = PersistentSessionInfo {
            session_id: Uuid::new_v4().to_string(),
            agent_id: agent_id.clone(),
            created_at: Utc::now(),
            last_activity: Utc::now(),
            status: PersistentSessionStatus::Creating,
            tasks_completed: 0,
            workspace_path: agent.lock().await.identity.workspace_path.clone(),
        };

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(agent_id.clone(), Arc::clone(&agent));
        }

        {
            let mut info_map = self.session_info.write().await;
            info_map.insert(agent_id, session_info);
        }

        tracing::info!("Persistent session created successfully");
        Ok(agent)
    }

    /// Execute a task on an agent session
    pub async fn execute_task(
        &self,
        role: AgentRole,
        task: Task,
        claude_config: ClaudeConfig,
    ) -> Result<TaskResult> {
        let session = self.get_or_create_session(role, claude_config).await?;

        // Update session status to active
        let agent_id = {
            let agent = session.lock().await;
            agent.identity.agent_id.clone()
        };

        self.update_session_status(&agent_id, PersistentSessionStatus::Active)
            .await;

        // Execute task
        let result = {
            let mut agent = session.lock().await;
            agent.execute_task(task).await?
        };

        // Update session info
        self.update_session_activity(&agent_id).await;

        // Mark as idle after task completion
        self.update_session_status(&agent_id, PersistentSessionStatus::Idle)
            .await;

        Ok(result)
    }

    /// Execute multiple tasks in batch - THIS IS THE KEY EFFICIENCY GAIN
    pub async fn execute_task_batch(
        &self,
        role: AgentRole,
        tasks: Vec<Task>,
        claude_config: ClaudeConfig,
    ) -> Result<Vec<TaskResult>> {
        if tasks.is_empty() {
            return Ok(Vec::new());
        }

        tracing::info!(
            "Executing batch of {} tasks for role: {}",
            tasks.len(),
            role.name()
        );

        let session = self.get_or_create_session(role, claude_config).await?;

        // Update session status to active
        let agent_id = {
            let agent = session.lock().await;
            agent.identity.agent_id.clone()
        };

        self.update_session_status(&agent_id, PersistentSessionStatus::Active)
            .await;

        // Execute batch - this amortizes the identity establishment cost
        let results = {
            let mut agent = session.lock().await;
            agent.execute_task_batch(tasks).await?
        };

        // Update session info
        self.update_session_activity(&agent_id).await;
        self.update_session_status(&agent_id, PersistentSessionStatus::Idle)
            .await;

        tracing::info!("Batch execution completed successfully");
        Ok(results)
    }

    /// Update session activity timestamp
    async fn update_session_activity(&self, agent_id: &str) {
        let mut session_info = self.session_info.write().await;
        if let Some(info) = session_info.get_mut(agent_id) {
            info.last_activity = Utc::now();
            info.tasks_completed += 1;
        }
    }

    /// Update session status
    async fn update_session_status(&self, agent_id: &str, status: PersistentSessionStatus) {
        let mut session_info = self.session_info.write().await;
        if let Some(info) = session_info.get_mut(agent_id) {
            info.status = status;
        }
    }

    /// Get session statistics
    pub async fn get_session_stats(&self, agent_id: &str) -> Option<SessionStats> {
        let sessions = self.sessions.read().await;
        if let Some(session) = sessions.get(agent_id) {
            let agent = session.lock().await;
            Some(agent.get_session_stats().await)
        } else {
            None
        }
    }

    /// List all active sessions
    pub async fn list_sessions(&self) -> Vec<PersistentSessionInfo> {
        let session_info = self.session_info.read().await;
        session_info.values().cloned().collect()
    }

    /// Get sessions by role
    pub async fn get_sessions_by_role(&self, role: &AgentRole) -> Vec<PersistentSessionInfo> {
        let sessions = self.sessions.read().await;
        let session_info = self.session_info.read().await;

        let mut result = Vec::new();
        for (agent_id, session) in sessions.iter() {
            if let Some(info) = session_info.get(agent_id) {
                let agent = session.lock().await;
                if agent.identity.specialization.name() == role.name() {
                    result.push(info.clone());
                }
            }
        }
        result
    }

    /// Cleanup sessions based on timeout and lifetime rules
    async fn cleanup_sessions(
        sessions: &SessionStorage,
        session_info: &Arc<RwLock<HashMap<String, PersistentSessionInfo>>>,
        idle_timeout: Duration,
        max_lifetime: Duration,
    ) -> Result<()> {
        let now = Utc::now();
        let mut to_remove = Vec::new();

        {
            let info_map = session_info.read().await;
            for (agent_id, info) in info_map.iter() {
                let idle_duration = now.signed_duration_since(info.last_activity);
                let lifetime_duration = now.signed_duration_since(info.created_at);

                let should_cleanup = match info.status {
                    PersistentSessionStatus::Idle => {
                        idle_duration.to_std().unwrap_or(Duration::ZERO) > idle_timeout
                    }
                    PersistentSessionStatus::Terminated => true,
                    _ => lifetime_duration.to_std().unwrap_or(Duration::ZERO) > max_lifetime,
                };

                if should_cleanup {
                    to_remove.push(agent_id.clone());
                }
            }
        }

        // Remove expired sessions
        for agent_id in to_remove {
            if let Err(e) = Self::remove_session(sessions, session_info, &agent_id).await {
                tracing::error!("Failed to remove session {}: {}", agent_id, e);
            } else {
                tracing::info!("Cleaned up expired session: {}", agent_id);
            }
        }

        Ok(())
    }

    /// Remove a specific session
    async fn remove_session(
        sessions: &SessionStorage,
        session_info: &Arc<RwLock<HashMap<String, PersistentSessionInfo>>>,
        agent_id: &str,
    ) -> Result<()> {
        // Shutdown the agent
        {
            let sessions_guard = sessions.read().await;
            if let Some(session) = sessions_guard.get(agent_id) {
                let mut agent = session.lock().await;
                agent.shutdown().await?;
            }
        }

        // Remove from maps
        {
            let mut sessions_guard = sessions.write().await;
            sessions_guard.remove(agent_id);
        }

        {
            let mut info_guard = session_info.write().await;
            info_guard.remove(agent_id);
        }

        Ok(())
    }

    /// Shutdown all sessions
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down persistent session manager");

        // Cancel cleanup task
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
        }

        // Shutdown all sessions
        let agent_ids: Vec<String> = {
            let session_info = self.session_info.read().await;
            session_info.keys().cloned().collect()
        };

        for agent_id in agent_ids {
            if let Err(e) =
                Self::remove_session(&self.sessions, &self.session_info, &agent_id).await
            {
                tracing::error!("Failed to shutdown session {}: {}", agent_id, e);
            }
        }

        tracing::info!("Persistent session manager shutdown complete");
        Ok(())
    }

    /// Create environment variables for agent
    fn create_env_vars(agent_id: &str) -> std::collections::HashMap<String, String> {
        let mut env_vars = std::collections::HashMap::new();
        env_vars.insert("CCSWARM_AGENT_ID".to_string(), agent_id.to_string());
        env_vars.insert("CCSWARM_SESSION_ID".to_string(), Uuid::new_v4().to_string());
        env_vars.insert(
            "CCSWARM_ROLE".to_string(),
            agent_id.split('-').next().unwrap_or("unknown").to_string(),
        );
        env_vars
    }

    /// Get efficiency statistics
    pub async fn get_efficiency_stats(&self) -> EfficiencyStats {
        let sessions = self.sessions.read().await;
        let session_info = self.session_info.read().await;

        let total_sessions = sessions.len();
        let active_sessions = session_info
            .values()
            .filter(|info| info.status == PersistentSessionStatus::Active)
            .count();
        let idle_sessions = session_info
            .values()
            .filter(|info| info.status == PersistentSessionStatus::Idle)
            .count();
        let total_tasks = session_info.values().map(|info| info.tasks_completed).sum();

        // Estimate token savings based on task reuse
        let estimated_token_savings = if total_tasks > total_sessions {
            // Each reused session saves ~3400 tokens
            (total_tasks - total_sessions) * 3400
        } else {
            0
        };

        EfficiencyStats {
            total_sessions,
            active_sessions,
            idle_sessions,
            total_tasks_completed: total_tasks,
            estimated_token_savings,
            session_reuse_rate: if total_tasks > 0 {
                (total_tasks - total_sessions) as f64 / total_tasks as f64
            } else {
                0.0
            },
        }
    }
}

/// Efficiency statistics for monitoring token savings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub idle_sessions: usize,
    pub total_tasks_completed: usize,
    pub estimated_token_savings: usize,
    pub session_reuse_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::default_frontend_role;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_persistent_session_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = PersistentSessionManagerConfig::default();
        let manager = PersistentSessionManager::new(temp_dir.path().to_path_buf(), config);

        assert_eq!(manager.sessions.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_persistent_session_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = PersistentSessionManagerConfig::default();
        let manager = PersistentSessionManager::new(temp_dir.path().to_path_buf(), config);

        let claude_config = ClaudeConfig::default();
        let session = manager
            .get_or_create_session(default_frontend_role(), claude_config)
            .await
            .unwrap();

        assert_eq!(manager.sessions.read().await.len(), 1);

        let agent = session.lock().await;
        assert!(agent.identity.agent_id.contains("frontend"));
    }

    #[tokio::test]
    async fn test_session_reuse() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = PersistentSessionManagerConfig::default();
        config.enable_session_reuse = true;

        let manager = PersistentSessionManager::new(temp_dir.path().to_path_buf(), config);

        let claude_config = ClaudeConfig::default();

        // Create first session
        let session1 = manager
            .get_or_create_session(default_frontend_role(), claude_config.clone())
            .await
            .unwrap();

        // Mark as idle
        let agent_id = {
            let agent = session1.lock().await;
            agent.identity.agent_id.clone()
        };
        manager
            .update_session_status(&agent_id, PersistentSessionStatus::Idle)
            .await;

        // Request another session of same type
        let session2 = manager
            .get_or_create_session(default_frontend_role(), claude_config)
            .await
            .unwrap();

        // Should still have only 1 session (reused)
        assert_eq!(manager.sessions.read().await.len(), 1);

        // Sessions should be the same
        assert_eq!(
            session1.lock().await.identity.agent_id,
            session2.lock().await.identity.agent_id
        );
    }

    #[tokio::test]
    async fn test_efficiency_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = PersistentSessionManagerConfig::default();
        let manager = PersistentSessionManager::new(temp_dir.path().to_path_buf(), config);

        // Initially no sessions
        let stats = manager.get_efficiency_stats().await;
        assert_eq!(stats.total_sessions, 0);
        assert_eq!(stats.total_tasks_completed, 0);
        assert_eq!(stats.estimated_token_savings, 0);
    }
}
