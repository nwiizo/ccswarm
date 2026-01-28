//! Session persistence store

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::core::{SessionId, SessionStatus, SessionConfig};

/// Session metadata for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: SessionId,
    pub name: Option<String>,
    pub status: SessionStatus,
    pub config: SessionConfig,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub pid: Option<u32>,
}

/// Session store for persistence
pub struct SessionStore {
    /// Base directory for session data
    base_dir: PathBuf,
    /// In-memory cache
    cache: Arc<RwLock<HashMap<SessionId, SessionMetadata>>>,
}

impl SessionStore {
    /// Create a new session store
    pub fn new() -> Result<Self> {
        let base_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
            .join("ai-session")
            .join("sessions");
        
        // Ensure directory exists
        fs::create_dir_all(&base_dir)?;
        
        let mut store = Self {
            base_dir,
            cache: Arc::new(RwLock::new(HashMap::new())),
        };
        
        // Load existing sessions
        store.load_all()?;
        
        Ok(store)
    }
    
    /// Save session metadata
    pub async fn save(&self, metadata: SessionMetadata) -> Result<()> {
        let session_file = self.session_file(&metadata.id);
        
        // Save to disk
        let json = serde_json::to_string_pretty(&metadata)?;
        fs::write(session_file, json)?;
        
        // Update cache
        self.cache.write().await.insert(metadata.id.clone(), metadata);
        
        Ok(())
    }
    
    /// Load session metadata
    pub async fn load(&self, id: &SessionId) -> Result<Option<SessionMetadata>> {
        // Check cache first
        if let Some(metadata) = self.cache.read().await.get(id) {
            return Ok(Some(metadata.clone()));
        }
        
        // Load from disk
        let session_file = self.session_file(id);
        if !session_file.exists() {
            return Ok(None);
        }
        
        let json = fs::read_to_string(session_file)?;
        let metadata: SessionMetadata = serde_json::from_str(&json)?;
        
        // Update cache
        self.cache.write().await.insert(id.clone(), metadata.clone());
        
        Ok(Some(metadata))
    }
    
    /// List all sessions
    pub async fn list(&self) -> Result<Vec<SessionMetadata>> {
        let cache = self.cache.read().await;
        let mut sessions: Vec<_> = cache.values().cloned().collect();
        sessions.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
        Ok(sessions)
    }
    
    /// Delete session metadata
    pub async fn delete(&self, id: &SessionId) -> Result<()> {
        let session_file = self.session_file(id);
        if session_file.exists() {
            fs::remove_file(session_file)?;
        }
        
        self.cache.write().await.remove(id);
        
        Ok(())
    }
    
    /// Update session status
    pub async fn update_status(&self, id: &SessionId, status: SessionStatus) -> Result<()> {
        if let Some(mut metadata) = self.load(id).await? {
            metadata.status = status;
            metadata.last_activity = chrono::Utc::now();
            self.save(metadata).await?;
        }
        Ok(())
    }
    
    /// Clean up terminated sessions
    pub async fn cleanup_terminated(&self) -> Result<usize> {
        let sessions = self.list().await?;
        let mut cleaned = 0;
        
        for session in sessions {
            if session.status == SessionStatus::Terminated {
                self.delete(&session.id).await?;
                cleaned += 1;
            }
        }
        
        Ok(cleaned)
    }
    
    /// Get session file path
    fn session_file(&self, id: &SessionId) -> PathBuf {
        self.base_dir.join(format!("{}.json", id))
    }
    
    /// Load all sessions from disk
    fn load_all(&mut self) -> Result<()> {
        if !self.base_dir.exists() {
            return Ok(());
        }
        
        for entry in fs::read_dir(&self.base_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(json) = fs::read_to_string(&path) {
                    if let Ok(metadata) = serde_json::from_str::<SessionMetadata>(&json) {
                        // Use block_on here since we're in a sync context
                        let cache = self.cache.clone();
                        let id = metadata.id.clone();
                        let meta = metadata.clone();
                        
                        std::thread::spawn(move || {
                            let rt = tokio::runtime::Handle::current();
                            rt.block_on(async move {
                                cache.write().await.insert(id, meta);
                            });
                        }).join().unwrap();
                    }
                }
            }
        }
        
        Ok(())
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new().expect("Failed to create session store")
    }
}

/// Global session store instance
static STORE_INIT: std::sync::Once = std::sync::Once::new();
static mut STORE: Option<Arc<SessionStore>> = None;

/// Get the global session store
pub fn get_store() -> Arc<SessionStore> {
    unsafe {
        STORE_INIT.call_once(|| {
            STORE = Some(Arc::new(SessionStore::new().expect("Failed to initialize session store")));
        });
        STORE.as_ref().unwrap().clone()
    }
}