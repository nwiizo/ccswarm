//! Session Checkpoint System
//!
//! Provides checkpoint creation, storage, and restoration for sessions.
//! Enables "what-if" exploration and error recovery.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::RwLock;

/// A checkpoint of session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCheckpoint {
    /// Unique checkpoint ID
    pub id: String,
    /// Session ID this checkpoint belongs to
    pub session_id: String,
    /// Optional human-readable label
    pub label: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Serialized session state
    pub state: serde_json::Value,
    /// Context/conversation history (compressed)
    pub context: Option<Vec<u8>>,
    /// Metadata about the checkpoint
    pub metadata: CheckpointMetadata,
}

/// Metadata about a checkpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMetadata {
    /// Number of tasks executed at checkpoint time
    pub tasks_completed: usize,
    /// Approximate token count in context
    pub token_count: Option<usize>,
    /// Size of state in bytes
    pub state_size_bytes: usize,
    /// Whether context is compressed
    pub context_compressed: bool,
    /// Parent checkpoint ID (if created from restore)
    pub parent_checkpoint_id: Option<String>,
    /// Additional metadata
    pub extra: HashMap<String, serde_json::Value>,
}

impl CheckpointMetadata {
    /// Create new metadata with basic info
    pub fn new(tasks_completed: usize, state_size: usize) -> Self {
        Self {
            tasks_completed,
            token_count: None,
            state_size_bytes: state_size,
            context_compressed: false,
            parent_checkpoint_id: None,
            extra: HashMap::new(),
        }
    }

    /// Set token count
    pub fn with_token_count(mut self, count: usize) -> Self {
        self.token_count = Some(count);
        self
    }

    /// Set parent checkpoint
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_checkpoint_id = Some(parent_id.into());
        self
    }
}

impl SessionCheckpoint {
    /// Create a new checkpoint
    pub fn new(session_id: impl Into<String>, state: serde_json::Value) -> Self {
        let state_bytes = serde_json::to_vec(&state).unwrap_or_default();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.into(),
            label: None,
            created_at: Utc::now(),
            state,
            context: None,
            metadata: CheckpointMetadata::new(0, state_bytes.len()),
        }
    }

    /// Add a label
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Add context data (will be compressed)
    pub fn with_context(mut self, context: Vec<u8>, compressed: bool) -> Self {
        self.metadata.context_compressed = compressed;
        self.context = Some(context);
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: CheckpointMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Index entry for checkpoint storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointIndexEntry {
    /// Checkpoint ID
    pub id: String,
    /// Session ID
    pub session_id: String,
    /// Optional label
    pub label: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// File path relative to checkpoint store
    pub path: String,
    /// State size in bytes
    pub size_bytes: usize,
}

/// Checkpoint store manages persistent storage of checkpoints
pub struct CheckpointStore {
    /// Base directory for checkpoint storage
    base_path: PathBuf,
    /// In-memory index of checkpoints
    index: RwLock<HashMap<String, CheckpointIndexEntry>>,
}

impl CheckpointStore {
    /// Create a new checkpoint store at the given path
    pub async fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base).await?;
        fs::create_dir_all(base.join("sessions")).await?;

        let store = Self {
            base_path: base.clone(),
            index: RwLock::new(HashMap::new()),
        };

        // Load existing index if present
        store.load_index().await?;

        Ok(store)
    }

    /// Get the default checkpoint store path
    pub fn default_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".ccswarm")
            .join("checkpoints")
    }

    /// Save a checkpoint
    pub async fn save(&self, checkpoint: &SessionCheckpoint) -> Result<()> {
        // Create session directory
        let session_dir = self.base_path.join("sessions").join(&checkpoint.session_id);
        fs::create_dir_all(&session_dir).await?;

        // Save checkpoint file
        let checkpoint_path = session_dir.join(format!("{}.json", checkpoint.id));
        let data = serde_json::to_vec_pretty(checkpoint)?;
        fs::write(&checkpoint_path, &data).await?;

        // Update index
        let entry = CheckpointIndexEntry {
            id: checkpoint.id.clone(),
            session_id: checkpoint.session_id.clone(),
            label: checkpoint.label.clone(),
            created_at: checkpoint.created_at,
            path: format!("sessions/{}/{}.json", checkpoint.session_id, checkpoint.id),
            size_bytes: data.len(),
        };

        self.index
            .write()
            .await
            .insert(checkpoint.id.clone(), entry);
        self.save_index().await?;

        tracing::info!(
            checkpoint_id = %checkpoint.id,
            session_id = %checkpoint.session_id,
            "Checkpoint saved"
        );

        Ok(())
    }

    /// Load a checkpoint by ID
    pub async fn load(&self, checkpoint_id: &str) -> Result<SessionCheckpoint> {
        let index = self.index.read().await;
        let entry = index
            .get(checkpoint_id)
            .ok_or_else(|| anyhow::anyhow!("Checkpoint not found: {}", checkpoint_id))?;

        let path = self.base_path.join(&entry.path);
        let data = fs::read(&path).await?;
        let checkpoint: SessionCheckpoint = serde_json::from_slice(&data)?;

        Ok(checkpoint)
    }

    /// List checkpoints for a session
    pub async fn list_for_session(&self, session_id: &str) -> Vec<CheckpointIndexEntry> {
        let index = self.index.read().await;
        index
            .values()
            .filter(|e| e.session_id == session_id)
            .cloned()
            .collect()
    }

    /// List all checkpoints
    pub async fn list_all(&self) -> Vec<CheckpointIndexEntry> {
        let index = self.index.read().await;
        index.values().cloned().collect()
    }

    /// Delete a checkpoint
    pub async fn delete(&self, checkpoint_id: &str) -> Result<()> {
        let entry = {
            let mut index = self.index.write().await;
            index.remove(checkpoint_id)
        };

        if let Some(entry) = entry {
            let path = self.base_path.join(&entry.path);
            if path.exists() {
                fs::remove_file(&path).await?;
            }
            self.save_index().await?;
            tracing::info!(checkpoint_id = %checkpoint_id, "Checkpoint deleted");
        }

        Ok(())
    }

    /// Delete all checkpoints for a session
    pub async fn delete_session_checkpoints(&self, session_id: &str) -> Result<usize> {
        let to_delete: Vec<String> = {
            let index = self.index.read().await;
            index
                .values()
                .filter(|e| e.session_id == session_id)
                .map(|e| e.id.clone())
                .collect()
        };

        let count = to_delete.len();
        for id in to_delete {
            self.delete(&id).await?;
        }

        // Remove session directory if empty
        let session_dir = self.base_path.join("sessions").join(session_id);
        if session_dir.exists()
            && let Ok(mut entries) = tokio::fs::read_dir(&session_dir).await
        {
            // Check if directory is empty by trying to read the first entry
            let is_empty = entries.next_entry().await.map_or(true, |e| e.is_none());
            if is_empty {
                let _ = fs::remove_dir(&session_dir).await;
            }
        }

        Ok(count)
    }

    /// Get the latest checkpoint for a session
    pub async fn get_latest(&self, session_id: &str) -> Option<CheckpointIndexEntry> {
        let checkpoints = self.list_for_session(session_id).await;
        checkpoints.into_iter().max_by_key(|c| c.created_at)
    }

    /// Save index to disk
    async fn save_index(&self) -> Result<()> {
        let index = self.index.read().await;
        let data = serde_json::to_vec_pretty(&*index)?;
        fs::write(self.base_path.join("index.json"), data).await?;
        Ok(())
    }

    /// Load index from disk
    async fn load_index(&self) -> Result<()> {
        let index_path = self.base_path.join("index.json");
        if index_path.exists() {
            let data = fs::read(&index_path).await?;
            let loaded: HashMap<String, CheckpointIndexEntry> = serde_json::from_slice(&data)?;
            let mut index = self.index.write().await;
            *index = loaded;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_checkpoint_creation() {
        let checkpoint = SessionCheckpoint::new("session-1", serde_json::json!({"key": "value"}))
            .with_label("test checkpoint");

        assert_eq!(checkpoint.session_id, "session-1");
        assert!(checkpoint.label.is_some());
    }

    #[tokio::test]
    async fn test_checkpoint_store_save_load() {
        let temp = tempdir().expect("Failed to create temp dir");
        let store = CheckpointStore::new(temp.path()).await.unwrap();

        let checkpoint =
            SessionCheckpoint::new("session-1", serde_json::json!({"state": "active"}));
        let id = checkpoint.id.clone();

        store.save(&checkpoint).await.unwrap();

        let loaded = store.load(&id).await.unwrap();
        assert_eq!(loaded.session_id, "session-1");
        assert_eq!(loaded.state["state"], "active");
    }

    #[tokio::test]
    async fn test_checkpoint_store_list() {
        let temp = tempdir().expect("Failed to create temp dir");
        let store = CheckpointStore::new(temp.path()).await.unwrap();

        let c1 = SessionCheckpoint::new("session-1", serde_json::json!({}));
        let c2 = SessionCheckpoint::new("session-1", serde_json::json!({}));
        let c3 = SessionCheckpoint::new("session-2", serde_json::json!({}));

        store.save(&c1).await.unwrap();
        store.save(&c2).await.unwrap();
        store.save(&c3).await.unwrap();

        let session1_checkpoints = store.list_for_session("session-1").await;
        assert_eq!(session1_checkpoints.len(), 2);

        let all_checkpoints = store.list_all().await;
        assert_eq!(all_checkpoints.len(), 3);
    }
}
