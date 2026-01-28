//! Session Fork System
//!
//! Enables session branching for parallel exploration and "what-if" scenarios.

use super::checkpoint::{CheckpointStore, SessionCheckpoint};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::sync::RwLock;

/// Information about a forked session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkInfo {
    /// Fork ID (also serves as new session ID)
    pub fork_id: String,
    /// Original session ID
    pub parent_session_id: String,
    /// Checkpoint ID the fork was created from
    pub checkpoint_id: String,
    /// Optional branch name
    pub branch_name: Option<String>,
    /// When the fork was created
    pub created_at: DateTime<Utc>,
    /// Current status of the fork
    pub status: ForkStatus,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Status of a fork
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForkStatus {
    /// Fork is being created
    Creating,
    /// Fork is active
    Active,
    /// Fork has been merged back
    Merged,
    /// Fork was abandoned
    Abandoned,
    /// Fork is archived
    Archived,
}

impl std::fmt::Display for ForkStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForkStatus::Creating => write!(f, "creating"),
            ForkStatus::Active => write!(f, "active"),
            ForkStatus::Merged => write!(f, "merged"),
            ForkStatus::Abandoned => write!(f, "abandoned"),
            ForkStatus::Archived => write!(f, "archived"),
        }
    }
}

impl ForkInfo {
    /// Create new fork info
    pub fn new(
        parent_session_id: impl Into<String>,
        checkpoint_id: impl Into<String>,
        branch_name: Option<String>,
    ) -> Self {
        Self {
            fork_id: uuid::Uuid::new_v4().to_string(),
            parent_session_id: parent_session_id.into(),
            checkpoint_id: checkpoint_id.into(),
            branch_name,
            created_at: Utc::now(),
            status: ForkStatus::Creating,
            metadata: HashMap::new(),
        }
    }

    /// Check if fork is active
    pub fn is_active(&self) -> bool {
        self.status == ForkStatus::Active
    }

    /// Check if fork can be used
    pub fn is_usable(&self) -> bool {
        matches!(self.status, ForkStatus::Active | ForkStatus::Creating)
    }
}

/// Fork registry tracks relationships between sessions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForkRegistry {
    /// Map of session ID to fork info
    pub forks: HashMap<String, ForkInfo>,
    /// Map of parent session ID to list of fork IDs
    pub children: HashMap<String, Vec<String>>,
}

impl ForkRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new fork
    pub fn register(&mut self, fork: ForkInfo) {
        let fork_id = fork.fork_id.clone();
        let parent_id = fork.parent_session_id.clone();

        self.forks.insert(fork_id.clone(), fork);
        self.children.entry(parent_id).or_default().push(fork_id);
    }

    /// Get fork info by ID
    pub fn get(&self, fork_id: &str) -> Option<&ForkInfo> {
        self.forks.get(fork_id)
    }

    /// Get mutable fork info by ID
    pub fn get_mut(&mut self, fork_id: &str) -> Option<&mut ForkInfo> {
        self.forks.get_mut(fork_id)
    }

    /// Get all forks of a parent session
    pub fn get_children(&self, parent_session_id: &str) -> Vec<&ForkInfo> {
        self.children
            .get(parent_session_id)
            .map(|ids| ids.iter().filter_map(|id| self.forks.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get active forks of a parent session
    pub fn get_active_children(&self, parent_session_id: &str) -> Vec<&ForkInfo> {
        self.get_children(parent_session_id)
            .into_iter()
            .filter(|f| f.is_active())
            .collect()
    }

    /// Update fork status
    pub fn update_status(&mut self, fork_id: &str, status: ForkStatus) -> Option<()> {
        self.forks.get_mut(fork_id).map(|f| {
            f.status = status;
        })
    }

    /// Remove a fork
    pub fn remove(&mut self, fork_id: &str) -> Option<ForkInfo> {
        if let Some(fork) = self.forks.remove(fork_id) {
            if let Some(children) = self.children.get_mut(&fork.parent_session_id) {
                children.retain(|id| id != fork_id);
            }
            Some(fork)
        } else {
            None
        }
    }

    /// List all forks
    pub fn list_all(&self) -> Vec<&ForkInfo> {
        self.forks.values().collect()
    }
}

/// Fork manager handles fork operations with persistence
pub struct ForkManager {
    /// Base path for fork data
    base_path: PathBuf,
    /// Fork registry
    registry: RwLock<ForkRegistry>,
    /// Reference to checkpoint store
    checkpoint_store: Option<CheckpointStore>,
}

impl ForkManager {
    /// Create a new fork manager
    pub async fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base).await?;

        let manager = Self {
            base_path: base.clone(),
            registry: RwLock::new(ForkRegistry::new()),
            checkpoint_store: None,
        };

        manager.load_registry().await?;
        Ok(manager)
    }

    /// Create with a checkpoint store reference
    pub async fn with_checkpoint_store(
        base_path: impl AsRef<Path>,
        checkpoint_store: CheckpointStore,
    ) -> Result<Self> {
        let mut manager = Self::new(&base_path).await?;
        manager.checkpoint_store = Some(checkpoint_store);
        Ok(manager)
    }

    /// Create a fork from a checkpoint
    pub async fn create_fork(
        &self,
        parent_session_id: &str,
        checkpoint_id: &str,
        branch_name: Option<String>,
    ) -> Result<ForkInfo> {
        // Verify checkpoint exists if we have a store
        if let Some(ref store) = self.checkpoint_store {
            store.load(checkpoint_id).await?;
        }

        let mut fork = ForkInfo::new(parent_session_id, checkpoint_id, branch_name);
        fork.status = ForkStatus::Active;

        // Register the fork
        {
            let mut registry = self.registry.write().await;
            registry.register(fork.clone());
        }

        // Save registry
        self.save_registry().await?;

        tracing::info!(
            fork_id = %fork.fork_id,
            parent = %parent_session_id,
            checkpoint = %checkpoint_id,
            "Fork created"
        );

        Ok(fork)
    }

    /// Get fork info
    pub async fn get_fork(&self, fork_id: &str) -> Option<ForkInfo> {
        let registry = self.registry.read().await;
        registry.get(fork_id).cloned()
    }

    /// List all forks for a session
    pub async fn list_forks(&self, session_id: &str) -> Vec<ForkInfo> {
        let registry = self.registry.read().await;
        registry
            .get_children(session_id)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Update fork status
    pub async fn update_fork_status(&self, fork_id: &str, status: ForkStatus) -> Result<()> {
        {
            let mut registry = self.registry.write().await;
            registry.update_status(fork_id, status);
        }
        self.save_registry().await?;
        Ok(())
    }

    /// Abandon a fork (mark as abandoned)
    pub async fn abandon_fork(&self, fork_id: &str) -> Result<()> {
        self.update_fork_status(fork_id, ForkStatus::Abandoned)
            .await
    }

    /// Archive a fork
    pub async fn archive_fork(&self, fork_id: &str) -> Result<()> {
        self.update_fork_status(fork_id, ForkStatus::Archived).await
    }

    /// Delete a fork completely
    pub async fn delete_fork(&self, fork_id: &str) -> Result<()> {
        {
            let mut registry = self.registry.write().await;
            registry.remove(fork_id);
        }
        self.save_registry().await?;
        tracing::info!(fork_id = %fork_id, "Fork deleted");
        Ok(())
    }

    /// Get the checkpoint a fork was created from
    pub async fn get_fork_checkpoint(&self, fork_id: &str) -> Result<Option<SessionCheckpoint>> {
        let registry = self.registry.read().await;
        if let Some(fork) = registry.get(fork_id) {
            if let Some(ref store) = self.checkpoint_store {
                return Ok(Some(store.load(&fork.checkpoint_id).await?));
            }
        }
        Ok(None)
    }

    /// Save registry to disk
    async fn save_registry(&self) -> Result<()> {
        let registry = self.registry.read().await;
        let data = serde_json::to_vec_pretty(&*registry)?;
        fs::write(self.base_path.join("forks.json"), data).await?;
        Ok(())
    }

    /// Load registry from disk
    async fn load_registry(&self) -> Result<()> {
        let path = self.base_path.join("forks.json");
        if path.exists() {
            let data = fs::read(&path).await?;
            let loaded: ForkRegistry = serde_json::from_slice(&data)?;
            let mut registry = self.registry.write().await;
            *registry = loaded;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_fork_info_creation() {
        let fork = ForkInfo::new(
            "session-1",
            "checkpoint-1",
            Some("feature-branch".to_string()),
        );

        assert_eq!(fork.parent_session_id, "session-1");
        assert_eq!(fork.checkpoint_id, "checkpoint-1");
        assert_eq!(fork.branch_name, Some("feature-branch".to_string()));
        assert_eq!(fork.status, ForkStatus::Creating);
    }

    #[test]
    fn test_fork_registry() {
        let mut registry = ForkRegistry::new();

        let mut fork1 = ForkInfo::new("parent-1", "cp-1", None);
        fork1.status = ForkStatus::Active;
        let fork1_id = fork1.fork_id.clone();

        let mut fork2 = ForkInfo::new("parent-1", "cp-2", None);
        fork2.status = ForkStatus::Active;

        registry.register(fork1);
        registry.register(fork2);

        let children = registry.get_children("parent-1");
        assert_eq!(children.len(), 2);

        registry.update_status(&fork1_id, ForkStatus::Merged);
        let active = registry.get_active_children("parent-1");
        assert_eq!(active.len(), 1);
    }

    #[tokio::test]
    async fn test_fork_manager() {
        let temp = tempdir().expect("Failed to create temp dir");
        let manager = ForkManager::new(temp.path()).await.unwrap();

        // Note: Without a checkpoint store, we can't verify the checkpoint
        // but we can still test the fork creation logic
        let fork = manager
            .create_fork("session-1", "checkpoint-1", Some("test-branch".to_string()))
            .await
            .unwrap();

        assert!(fork.is_active());

        let forks = manager.list_forks("session-1").await;
        assert_eq!(forks.len(), 1);

        manager.abandon_fork(&fork.fork_id).await.unwrap();
        let updated = manager.get_fork(&fork.fork_id).await.unwrap();
        assert_eq!(updated.status, ForkStatus::Abandoned);
    }
}
