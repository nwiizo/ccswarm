use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::context::SessionContext;
use super::types::ExecutionSessionId;

pub struct PersistenceManager {
    storage_path: PathBuf,
}

impl PersistenceManager {
    pub fn new(storage_path: PathBuf) -> Self {
        Self { storage_path }
    }

    pub async fn save_session(
        &self,
        session_id: &ExecutionSessionId,
        state: &SessionState,
    ) -> Result<()> {
        let session_dir = self.storage_path.join(session_id.to_string());
        fs::create_dir_all(&session_dir).await?;
        let state_file = session_dir.join("state.json");
        let data = serde_json::to_vec_pretty(state)?;
        let mut file = fs::File::create(&state_file).await?;
        file.write_all(&data).await?;
        file.sync_all().await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: ExecutionSessionId,
    pub status: ExecutionSessionStatus,
    pub context: SessionContext,
    #[serde(default)]
    pub command_history: Vec<CommandRecord>,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExecutionSessionStatus {
    #[default]
    Running,
    Completed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecord {
    pub command: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub exit_code: Option<i32>,
    pub output_preview: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    pub total_tokens: usize,
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            last_accessed: now,
            total_tokens: 0,
            custom: HashMap::new(),
        }
    }
}
