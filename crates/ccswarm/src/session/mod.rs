/// Minimal session implementation
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod ai_session_adapter;
pub mod claude_session;
pub mod memory;
pub mod persistent_session;
// pub mod pool_session; // Module not found
pub mod session_pool;
// pub mod simple_session; // Module not found
pub mod traits;
pub mod worktree_session;

pub use self::ai_session_adapter::SessionManagerAdapter as AISessionAdapter;
pub use memory::SessionMemory;

// Re-export SessionStatus from traits module
pub use self::traits::SessionStatus;

// Agent session wrapper struct
pub struct AgentSession {
    pub agent_id: String,
    pub role: crate::agent::AgentRole,
    pub session_id: String,
    pub working_directory: String,
    pub description: String,
    pub status: SessionStatus,
}

impl AgentSession {
    pub fn new(
        agent_id: String,
        role: crate::agent::AgentRole,
        working_directory: String,
        description: String,
    ) -> Self {
        Self {
            agent_id,
            role,
            session_id: uuid::Uuid::new_v4().to_string(),
            working_directory,
            description,
            status: SessionStatus::Active,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub token_count: usize,
    pub messages_processed: usize,
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
}

#[derive(Clone)]
struct SessionInfo {
    _id: String,
    _agent_name: String,
    _created_at: std::time::Instant,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_session(&self, agent_name: String) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let info = SessionInfo {
            _id: id.clone(),
            _agent_name: agent_name,
            _created_at: std::time::Instant::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(id.clone(), info);
        Ok(id)
    }

    pub async fn get_stats(&self) -> Result<SessionStats> {
        let sessions = self.sessions.read().await;
        Ok(SessionStats {
            total_sessions: sessions.len(),
            active_sessions: sessions.len(),
            token_count: 0,
            messages_processed: 0,
        })
    }

    pub async fn list_sessions(&self) -> Result<Vec<String>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.keys().cloned().collect())
    }
}
