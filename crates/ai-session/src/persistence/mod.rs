//! Session state persistence and recovery

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::context::SessionContext;
use crate::core::{SessionConfig, SessionId, SessionStatus};

/// Manages persistent storage of session state
pub struct PersistenceManager {
    /// Base directory for session storage
    storage_path: PathBuf,
    /// Compression enabled
    enable_compression: bool,
    /// Encryption key (optional)
    encryption_key: Option<Vec<u8>>,
}

impl PersistenceManager {
    /// Create a new persistence manager
    pub fn new(storage_path: PathBuf) -> Self {
        Self {
            storage_path,
            enable_compression: true,
            encryption_key: None,
        }
    }

    /// Enable encryption with key
    pub fn with_encryption(mut self, key: Vec<u8>) -> Self {
        self.encryption_key = Some(key);
        self
    }

    /// Save session state
    pub async fn save_session(&self, session_id: &SessionId, state: &SessionState) -> Result<()> {
        let session_dir = self.session_directory(session_id);
        fs::create_dir_all(&session_dir).await?;

        // Serialize state
        let data = serde_json::to_vec_pretty(state)?;

        // Optionally compress
        let data = if self.enable_compression {
            self.compress_data(&data)?
        } else {
            data
        };

        // Optionally encrypt
        let data = if let Some(key) = &self.encryption_key {
            self.encrypt_data(&data, key)?
        } else {
            data
        };

        // Write to file
        let state_file = session_dir.join("state.json");
        let mut file = fs::File::create(&state_file).await?;
        file.write_all(&data).await?;
        file.sync_all().await?;

        Ok(())
    }

    /// Load session state
    pub async fn load_session(&self, session_id: &SessionId) -> Result<SessionState> {
        let state_file = self.session_directory(session_id).join("state.json");

        // Read file
        let mut file = fs::File::open(&state_file).await?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).await?;

        // Optionally decrypt
        let data = if let Some(key) = &self.encryption_key {
            self.decrypt_data(&data, key)?
        } else {
            data
        };

        // Optionally decompress
        let data = if self.enable_compression {
            self.decompress_data(&data)?
        } else {
            data
        };

        // Deserialize
        let state: SessionState = serde_json::from_slice(&data)?;
        Ok(state)
    }

    /// List all saved sessions
    pub async fn list_sessions(&self) -> Result<Vec<SessionId>> {
        let mut sessions = Vec::new();

        let mut entries = fs::read_dir(&self.storage_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Ok(name) = entry.file_name().into_string() {
                    if let Ok(id) = SessionId::parse_str(&name) {
                        sessions.push(id);
                    }
                }
            }
        }

        Ok(sessions)
    }

    /// Delete session data
    pub async fn delete_session(&self, session_id: &SessionId) -> Result<()> {
        let session_dir = self.session_directory(session_id);
        if session_dir.exists() {
            fs::remove_dir_all(&session_dir).await?;
        }
        Ok(())
    }

    /// Get session directory
    fn session_directory(&self, session_id: &SessionId) -> PathBuf {
        self.storage_path.join(session_id.to_string())
    }

    /// Compress data
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use zstd::stream::encode_all;

        encode_all(data, 3).map_err(|e| anyhow::anyhow!("Failed to compress data: {}", e))
    }

    /// Decompress data
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use zstd::stream::decode_all;

        decode_all(data).map_err(|e| anyhow::anyhow!("Failed to decompress data: {}", e))
    }

    /// Encrypt data (simplified - use proper crypto in production)
    fn encrypt_data(&self, data: &[u8], _key: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement proper encryption
        Ok(data.to_vec())
    }

    /// Decrypt data (simplified - use proper crypto in production)
    fn decrypt_data(&self, data: &[u8], _key: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement proper decryption
        Ok(data.to_vec())
    }
}

/// Persistent session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Session ID
    pub session_id: SessionId,
    /// Session configuration
    pub config: SessionConfig,
    /// Current status
    pub status: SessionStatus,
    /// Session context
    pub context: SessionContext,
    /// Command history
    pub command_history: Vec<CommandRecord>,
    /// Session metadata
    pub metadata: SessionMetadata,
}

/// Command execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecord {
    /// Command text
    pub command: String,
    /// Execution timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Exit code
    pub exit_code: Option<i32>,
    /// Output preview
    pub output_preview: String,
    /// Execution duration
    pub duration_ms: u64,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last accessed time
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Total commands executed
    pub command_count: usize,
    /// Total tokens used
    pub total_tokens: usize,
    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

impl Default for SessionMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            last_accessed: now,
            command_count: 0,
            total_tokens: 0,
            custom: HashMap::new(),
        }
    }
}

/// Session snapshot for quick restore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// Snapshot ID
    pub id: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Session state at snapshot time
    pub state: SessionState,
    /// Snapshot description
    pub description: Option<String>,
}

/// Manages session snapshots
pub struct SnapshotManager {
    /// Base directory for snapshots
    snapshot_path: PathBuf,
}

impl SnapshotManager {
    /// Create new snapshot manager
    pub fn new(snapshot_path: PathBuf) -> Self {
        Self { snapshot_path }
    }

    /// Create a snapshot
    pub async fn create_snapshot(
        &self,
        session_id: &SessionId,
        state: &SessionState,
        description: Option<String>,
    ) -> Result<String> {
        let snapshot = SessionSnapshot {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now(),
            state: state.clone(),
            description,
        };

        let snapshot_dir = self.snapshot_path.join(session_id.to_string());
        fs::create_dir_all(&snapshot_dir).await?;

        let snapshot_file = snapshot_dir.join(format!("{}.json", snapshot.id));
        let data = serde_json::to_vec_pretty(&snapshot)?;
        fs::write(&snapshot_file, data).await?;

        Ok(snapshot.id)
    }

    /// List snapshots for a session
    pub async fn list_snapshots(&self, session_id: &SessionId) -> Result<Vec<SessionSnapshot>> {
        let snapshot_dir = self.snapshot_path.join(session_id.to_string());
        if !snapshot_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();
        let mut entries = fs::read_dir(&snapshot_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry
                .path()
                .extension()
                .map(|e| e == "json")
                .unwrap_or(false)
            {
                let data = fs::read(entry.path()).await?;
                if let Ok(snapshot) = serde_json::from_slice::<SessionSnapshot>(&data) {
                    snapshots.push(snapshot);
                }
            }
        }

        // Sort by creation time
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(snapshots)
    }

    /// Restore from snapshot
    pub async fn restore_snapshot(
        &self,
        session_id: &SessionId,
        snapshot_id: &str,
    ) -> Result<SessionState> {
        let snapshot_file = self
            .snapshot_path
            .join(session_id.to_string())
            .join(format!("{}.json", snapshot_id));

        let data = fs::read(&snapshot_file).await?;
        let snapshot: SessionSnapshot = serde_json::from_slice(&data)?;

        Ok(snapshot.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_persistence_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PersistenceManager::new(temp_dir.path().to_path_buf());

        let session_id = SessionId::new_v4();
        let state = SessionState {
            session_id: session_id.clone(),
            config: SessionConfig::default(),
            status: SessionStatus::Running,
            context: SessionContext::new(session_id.clone()),
            command_history: vec![],
            metadata: SessionMetadata::default(),
        };

        // Save and load
        manager.save_session(&session_id, &state).await.unwrap();
        let loaded = manager.load_session(&session_id).await.unwrap();

        assert_eq!(loaded.session_id, state.session_id);
    }
}
