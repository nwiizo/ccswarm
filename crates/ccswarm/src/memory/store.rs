//! Memory storage backends

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use super::embedding::Embedding;

/// Unique identifier for a memory
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(pub String);

impl MemoryId {
    /// Create a new random ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from string
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl Default for MemoryId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MemoryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of memory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    /// Conversation history
    Conversation,
    /// Task context
    TaskContext,
    /// Code patterns
    CodePattern,
    /// Error patterns
    ErrorPattern,
    /// Documentation
    Documentation,
    /// Learned behavior
    Behavior,
    /// User preference
    Preference,
    /// Entity information
    Entity,
}

impl MemoryType {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            MemoryType::Conversation => "Conversation",
            MemoryType::TaskContext => "Task Context",
            MemoryType::CodePattern => "Code Pattern",
            MemoryType::ErrorPattern => "Error Pattern",
            MemoryType::Documentation => "Documentation",
            MemoryType::Behavior => "Behavior",
            MemoryType::Preference => "Preference",
            MemoryType::Entity => "Entity",
        }
    }
}

/// A single memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Unique ID
    pub id: MemoryId,
    /// Content of the memory
    pub content: String,
    /// Type of memory
    pub memory_type: MemoryType,
    /// Vector embedding
    #[serde(skip)]
    pub embedding: Option<Embedding>,
    /// When created
    pub created_at: DateTime<Utc>,
    /// When last accessed
    pub last_accessed: DateTime<Utc>,
    /// Access count
    pub access_count: u64,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Associated agent
    pub agent_id: Option<String>,
    /// Associated task
    pub task_id: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

impl Memory {
    /// Create a new memory
    pub fn new(content: impl Into<String>, memory_type: MemoryType) -> Self {
        let now = Utc::now();
        Self {
            id: MemoryId::new(),
            content: content.into(),
            memory_type,
            embedding: None,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            importance: 0.5,
            metadata: HashMap::new(),
            agent_id: None,
            task_id: None,
            tags: Vec::new(),
        }
    }

    /// Set embedding
    pub fn with_embedding(mut self, embedding: Embedding) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Set importance
    pub fn with_importance(mut self, importance: f32) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Set agent ID
    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    /// Set task ID
    pub fn with_task(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Record access
    pub fn record_access(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }

    /// Calculate decay factor based on time
    pub fn decay_factor(&self) -> f32 {
        let hours_since_access = (Utc::now() - self.last_accessed).num_hours() as f32;
        // Exponential decay with half-life of 24 hours
        (0.5_f32).powf(hours_since_access / 24.0)
    }

    /// Get effective importance (considering decay and access)
    pub fn effective_importance(&self) -> f32 {
        let access_bonus = (self.access_count as f32 / 100.0).min(0.2);
        (self.importance + access_bonus) * self.decay_factor()
    }
}

/// Trait for memory storage backends
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// Store a memory
    async fn store(&mut self, memory: Memory) -> Result<(), String>;

    /// Get a memory by ID
    async fn get(&self, id: &MemoryId) -> Option<Memory>;

    /// Delete a memory
    async fn delete(&mut self, id: &MemoryId) -> Result<(), String>;

    /// Search by embedding similarity
    async fn search(
        &self,
        embedding: &Embedding,
        limit: usize,
    ) -> Result<Vec<(Memory, f32)>, String>;

    /// Search by tags
    async fn search_by_tags(&self, tags: &[String], limit: usize) -> Result<Vec<Memory>, String>;

    /// Search by type
    async fn search_by_type(
        &self,
        memory_type: MemoryType,
        limit: usize,
    ) -> Result<Vec<Memory>, String>;

    /// Update importance
    async fn update_importance(&mut self, id: &MemoryId, importance: f32) -> Result<(), String>;

    /// Get count
    async fn count(&self) -> usize;

    /// Clear all memories
    async fn clear(&mut self);
}

/// In-memory storage backend
pub struct InMemoryStore {
    memories: RwLock<HashMap<MemoryId, Memory>>,
    max_entries: usize,
}

impl InMemoryStore {
    /// Create a new in-memory store
    pub fn new(max_entries: usize) -> Self {
        Self {
            memories: RwLock::new(HashMap::new()),
            max_entries,
        }
    }

    /// Enforce maximum entries by removing least important
    fn enforce_limit(&self, memories: &mut HashMap<MemoryId, Memory>) {
        while memories.len() >= self.max_entries {
            if let Some(id) = memories
                .iter()
                .min_by(|(_, a), (_, b)| {
                    a.effective_importance()
                        .partial_cmp(&b.effective_importance())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(id, _)| id.clone())
            {
                memories.remove(&id);
            }
        }
    }
}

#[async_trait]
impl MemoryStore for InMemoryStore {
    async fn store(&mut self, memory: Memory) -> Result<(), String> {
        let mut memories = self
            .memories
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        self.enforce_limit(&mut memories);
        memories.insert(memory.id.clone(), memory);
        Ok(())
    }

    async fn get(&self, id: &MemoryId) -> Option<Memory> {
        let memories = self.memories.read().ok()?;
        memories.get(id).cloned()
    }

    async fn delete(&mut self, id: &MemoryId) -> Result<(), String> {
        let mut memories = self
            .memories
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;
        memories.remove(id);
        Ok(())
    }

    async fn search(
        &self,
        query_embedding: &Embedding,
        limit: usize,
    ) -> Result<Vec<(Memory, f32)>, String> {
        let memories = self
            .memories
            .read()
            .map_err(|e| format!("Lock error: {}", e))?;

        let mut results: Vec<(Memory, f32)> = memories
            .values()
            .filter_map(|memory| {
                memory.embedding.as_ref().map(|emb| {
                    let score = cosine_similarity(&query_embedding.vector, &emb.vector);
                    (memory.clone(), score)
                })
            })
            .collect();

        // Sort by score descending
        results.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results.into_iter().take(limit).collect())
    }

    async fn search_by_tags(&self, tags: &[String], limit: usize) -> Result<Vec<Memory>, String> {
        let memories = self
            .memories
            .read()
            .map_err(|e| format!("Lock error: {}", e))?;

        let mut results: Vec<Memory> = memories
            .values()
            .filter(|m| tags.iter().any(|t| m.tags.contains(t)))
            .cloned()
            .collect();

        results.sort_by(|a, b| {
            b.effective_importance()
                .partial_cmp(&a.effective_importance())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results.into_iter().take(limit).collect())
    }

    async fn search_by_type(
        &self,
        memory_type: MemoryType,
        limit: usize,
    ) -> Result<Vec<Memory>, String> {
        let memories = self
            .memories
            .read()
            .map_err(|e| format!("Lock error: {}", e))?;

        let mut results: Vec<Memory> = memories
            .values()
            .filter(|m| m.memory_type == memory_type)
            .cloned()
            .collect();

        results.sort_by(|a, b| {
            b.effective_importance()
                .partial_cmp(&a.effective_importance())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results.into_iter().take(limit).collect())
    }

    async fn update_importance(&mut self, id: &MemoryId, importance: f32) -> Result<(), String> {
        let mut memories = self
            .memories
            .write()
            .map_err(|e| format!("Lock error: {}", e))?;

        if let Some(memory) = memories.get_mut(id) {
            memory.importance = importance.clamp(0.0, 1.0);
            Ok(())
        } else {
            Err(format!("Memory not found: {}", id))
        }
    }

    async fn count(&self) -> usize {
        self.memories.read().map(|m| m.len()).unwrap_or(0)
    }

    async fn clear(&mut self) {
        if let Ok(mut memories) = self.memories.write() {
            memories.clear();
        }
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new("Test content", MemoryType::Conversation);
        assert!(!memory.id.0.is_empty());
        assert_eq!(memory.content, "Test content");
        assert_eq!(memory.memory_type, MemoryType::Conversation);
    }

    #[test]
    fn test_memory_builder() {
        let memory = Memory::new("Test", MemoryType::CodePattern)
            .with_importance(0.8)
            .with_agent("agent1")
            .with_task("task1")
            .with_tag("rust")
            .with_metadata("key", "value");

        assert_eq!(memory.importance, 0.8);
        assert_eq!(memory.agent_id, Some("agent1".to_string()));
        assert!(memory.tags.contains(&"rust".to_string()));
    }

    #[test]
    fn test_effective_importance() {
        let memory = Memory::new("Test", MemoryType::Conversation).with_importance(0.5);

        let effective = memory.effective_importance();
        // Should be close to 0.5 for new memory (no decay)
        assert!((0.4..=0.6).contains(&effective));
    }

    #[tokio::test]
    async fn test_in_memory_store() {
        let mut store = InMemoryStore::new(100);

        let memory = Memory::new("Test content", MemoryType::Conversation);
        let id = memory.id.clone();

        store.store(memory).await.unwrap();

        let retrieved = store.get(&id).await.unwrap();
        assert_eq!(retrieved.content, "Test content");
    }

    #[tokio::test]
    async fn test_store_limit_enforcement() {
        let mut store = InMemoryStore::new(2);

        let m1 = Memory::new("First", MemoryType::Conversation).with_importance(0.9);
        let m2 = Memory::new("Second", MemoryType::Conversation).with_importance(0.1);
        let m3 = Memory::new("Third", MemoryType::Conversation).with_importance(0.5);

        let id2 = m2.id.clone();

        store.store(m1).await.unwrap();
        store.store(m2).await.unwrap();
        store.store(m3).await.unwrap();

        assert_eq!(store.count().await, 2);
        // Low importance memory should be evicted
        assert!(store.get(&id2).await.is_none());
    }

    #[tokio::test]
    async fn test_search_by_type() {
        let mut store = InMemoryStore::new(100);

        store
            .store(Memory::new("Code 1", MemoryType::CodePattern))
            .await
            .unwrap();
        store
            .store(Memory::new("Doc 1", MemoryType::Documentation))
            .await
            .unwrap();
        store
            .store(Memory::new("Code 2", MemoryType::CodePattern))
            .await
            .unwrap();

        let results = store
            .search_by_type(MemoryType::CodePattern, 10)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);

        let d = vec![0.707, 0.707, 0.0];
        let sim = cosine_similarity(&a, &d);
        assert!(sim > 0.7 && sim < 0.8);
    }
}
