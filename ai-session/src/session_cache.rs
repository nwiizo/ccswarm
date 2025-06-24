//! Simple session cache for CLI persistence

use crate::core::{SessionId, SessionConfig};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: SessionId,
    pub name: Option<String>,
    pub config: SessionConfig,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub pid: Option<u32>,
}

pub struct SessionCache {
    cache_file: PathBuf,
    sessions: HashMap<SessionId, SessionInfo>,
}

impl SessionCache {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?
            .join("ai-session");
        
        fs::create_dir_all(&cache_dir)?;
        
        let cache_file = cache_dir.join("sessions.json");
        let mut cache = Self {
            cache_file,
            sessions: HashMap::new(),
        };
        
        cache.load()?;
        Ok(cache)
    }
    
    pub fn add_session(&mut self, info: SessionInfo) -> Result<()> {
        self.sessions.insert(info.id.clone(), info);
        self.save()
    }
    
    pub fn get_session(&self, id: &SessionId) -> Option<&SessionInfo> {
        self.sessions.get(id)
    }
    
    pub fn list_sessions(&self) -> Vec<&SessionInfo> {
        let mut sessions: Vec<_> = self.sessions.values().collect();
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        sessions
    }
    
    pub fn remove_session(&mut self, id: &SessionId) -> Result<()> {
        self.sessions.remove(id);
        self.save()
    }
    
    fn load(&mut self) -> Result<()> {
        if self.cache_file.exists() {
            let content = fs::read_to_string(&self.cache_file)?;
            if !content.is_empty() {
                self.sessions = serde_json::from_str(&content)?;
            }
        }
        Ok(())
    }
    
    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.sessions)?;
        fs::write(&self.cache_file, json)?;
        Ok(())
    }
}