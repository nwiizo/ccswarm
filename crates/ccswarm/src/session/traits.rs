//! Common traits for session management
//!
//! This module defines the core traits that all session types must implement,
//! enabling code reuse and consistent behavior across different session implementations.

#![allow(async_fn_in_trait)]

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::agent::{Task, TaskResult};

/// Status of a session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is being initialized
    Initializing,
    /// Session is active and ready for tasks
    Active,
    /// Session is idle (no recent activity)
    Idle,
    /// Session is processing a task
    Busy,
    /// Session has encountered an error
    Error,
    /// Session is being terminated
    Terminating,
    /// Session has been terminated
    Terminated,
}

/// Basic information about a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub session_type: String,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub tasks_completed: usize,
}

/// Efficiency statistics for a session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EfficiencyStats {
    pub total_tasks: usize,
    pub successful_tasks: usize,
    pub failed_tasks: usize,
    pub total_duration: Duration,
    pub average_task_duration: Duration,
    pub token_savings_percentage: f64,
}

/// Configuration for creating a new session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub agent_id: String,
    pub working_directory: Option<String>,
    pub env_vars: HashMap<String, String>,
    pub auto_cleanup: bool,
    pub idle_timeout: Option<Duration>,
}

/// Trait for managing session lifecycle
pub trait SessionLifecycle: Send + Sync {
    /// Initialize the session
    async fn initialize(&mut self) -> Result<()>;

    /// Shutdown the session gracefully
    async fn shutdown(&mut self) -> Result<()>;

    /// Get the current session status
    fn get_status(&self) -> SessionStatus;

    /// Update the session status
    fn update_status(&mut self, status: SessionStatus);

    /// Check if the session is ready for tasks
    fn is_ready(&self) -> bool {
        matches!(
            self.get_status(),
            SessionStatus::Active | SessionStatus::Idle
        )
    }

    /// Check if the session is terminated
    fn is_terminated(&self) -> bool {
        matches!(self.get_status(), SessionStatus::Terminated)
    }
}

/// Trait for executing tasks in a session
#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
    /// Execute a single task
    async fn execute_task(&mut self, task: Task) -> Result<TaskResult>;

    /// Execute multiple tasks in batch for efficiency
    async fn execute_task_batch(&mut self, tasks: Vec<Task>) -> Result<Vec<TaskResult>> {
        // Default implementation: execute tasks sequentially
        let mut results = Vec::new();
        for task in tasks {
            results.push(self.execute_task(task).await?);
        }
        Ok(results)
    }

    /// Check if the session can handle a specific task type
    fn can_handle_task(&self, task: &Task) -> bool {
        // Default: can handle any task
        let _ = task;
        true
    }
}

/// Trait for session metadata management
pub trait SessionMetadata: Send + Sync {
    /// Get the unique session ID
    fn get_id(&self) -> &str;

    /// Get the session creation timestamp
    fn get_created_at(&self) -> DateTime<Utc>;

    /// Get the last activity timestamp
    fn get_last_activity(&self) -> DateTime<Utc>;

    /// Update the last activity timestamp to now
    fn touch(&mut self);

    /// Get the session type name
    fn get_session_type(&self) -> &str;
}

/// Trait for managing session environment
pub trait SessionEnvironment: Send + Sync {
    /// Get the working directory for this session
    fn get_working_directory(&self) -> &Path;

    /// Get environment variables for this session
    fn get_env_vars(&self) -> &HashMap<String, String>;

    /// Create standard environment variables for a session
    fn create_env_vars(agent_id: &str, session_id: &str) -> HashMap<String, String> {
        let mut env_vars = crate::utils::common::collections::new_hashmap();
        env_vars.insert("CCSWARM_AGENT_ID".to_string(), agent_id.to_string());
        env_vars.insert("CCSWARM_SESSION_ID".to_string(), session_id.to_string());
        env_vars.insert("CCSWARM_SESSION_TYPE".to_string(), "generic".to_string());
        env_vars
    }
}

/// Trait for tracking session statistics
pub trait SessionStatistics: Send + Sync {
    /// Get the number of tasks completed
    fn get_tasks_completed(&self) -> usize;

    /// Get efficiency statistics
    fn get_efficiency_stats(&self) -> EfficiencyStats;

    /// Update statistics after task completion
    fn update_task_stats(&mut self, result: &TaskResult, duration: Duration);

    /// Reset statistics
    fn reset_stats(&mut self);
}

/// Combined trait for a complete session
pub trait Session:
    SessionLifecycle + TaskExecutor + SessionMetadata + SessionEnvironment + SessionStatistics
{
}

/// Trait for managing collections of sessions
#[async_trait::async_trait]
pub trait SessionManager: Send + Sync {
    /// The concrete session type this manager handles
    type SessionType: Session + 'static;

    /// Create a new session
    async fn create_session(&self, config: SessionConfig) -> Result<Arc<Mutex<Self::SessionType>>>;

    /// Get an existing session by ID
    async fn get_session(&self, id: &str) -> Option<Arc<Mutex<Self::SessionType>>>;

    /// List all sessions with their basic info
    async fn list_sessions(&self) -> Vec<SessionInfo>;

    /// Clean up terminated sessions
    async fn cleanup_terminated(&self) -> Result<usize>;

    /// Get or create a session for an agent
    async fn get_or_create_session(&self, agent_id: &str) -> Result<Arc<Mutex<Self::SessionType>>> {
        // Default implementation
        let sessions = self.list_sessions().await;

        // Look for an existing active session for this agent
        for session_info in sessions {
            if session_info.id.contains(agent_id)
                && matches!(
                    session_info.status,
                    SessionStatus::Active | SessionStatus::Idle
                )
            {
                if let Some(session) = self.get_session(&session_info.id).await {
                    return Ok(session);
                }
            }
        }

        // Create a new session
        let config = SessionConfig {
            agent_id: agent_id.to_string(),
            working_directory: None,
            env_vars: HashMap::new(),
            auto_cleanup: true,
            idle_timeout: Some(Duration::from_secs(300)),
        };

        self.create_session(config).await
    }
}

/// Helper trait for sessions that support pooling
pub trait PoolableSession: Session {
    /// Check if this session can be reused
    fn is_reusable(&self) -> bool;

    /// Prepare session for reuse (clear state, etc.)
    async fn prepare_for_reuse(&mut self) -> Result<()>;
}

/// Extension trait for sessions with persistence capabilities
#[async_trait::async_trait]
pub trait PersistentSession: Session {
    /// Save session state to disk
    async fn save_state(&self) -> Result<()>;

    /// Load session state from disk
    async fn load_state(&mut self) -> Result<()>;

    /// Get the persistence path
    fn get_persistence_path(&self) -> &Path;
}
