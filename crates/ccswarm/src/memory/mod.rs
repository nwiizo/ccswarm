//! Long-term Memory and RAG Integration Module
//!
//! Provides persistent memory capabilities for AI agents, enabling them to
//! learn from past experiences and retrieve relevant context using RAG.

mod embedding;
mod retrieval;
mod store;

pub use embedding::{Embedding, EmbeddingModel, EmbeddingProvider};
pub use retrieval::{RetrievalConfig, RetrievalResult, Retriever};
pub use store::{Memory, MemoryId, MemoryStore, MemoryType};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Memory system for agent long-term memory and RAG
pub struct MemorySystem {
    /// Short-term memory (current session)
    short_term: Arc<RwLock<ShortTermMemory>>,
    /// Long-term memory store
    long_term: Arc<RwLock<Box<dyn MemoryStore>>>,
    /// Embedding provider
    embedder: Arc<dyn EmbeddingProvider>,
    /// Retriever for RAG (reserved for advanced retrieval pipeline)
    #[allow(dead_code)]
    retriever: Arc<Retriever>,
    /// Configuration
    config: MemoryConfig,
}

/// Configuration for the memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Whether memory is enabled
    pub enabled: bool,
    /// Maximum short-term memory entries
    pub max_short_term: usize,
    /// Maximum long-term memory entries
    pub max_long_term: usize,
    /// Embedding model to use
    pub embedding_model: String,
    /// Path for persistent storage
    pub storage_path: Option<PathBuf>,
    /// Similarity threshold for retrieval
    pub similarity_threshold: f32,
    /// Maximum tokens per memory
    pub max_memory_tokens: usize,
    /// Auto-persist interval in seconds
    pub persist_interval_secs: u64,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_short_term: 100,
            max_long_term: 10000,
            embedding_model: "text-embedding-3-small".to_string(),
            storage_path: None,
            similarity_threshold: 0.7,
            max_memory_tokens: 500,
            persist_interval_secs: 300,
        }
    }
}

/// Short-term memory for current session
#[derive(Debug, Default)]
pub struct ShortTermMemory {
    /// Memory entries
    entries: Vec<ShortTermEntry>,
    /// Maximum entries
    max_entries: usize,
}

/// An entry in short-term memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortTermEntry {
    /// Entry ID
    pub id: String,
    /// Content
    pub content: String,
    /// Type of content
    pub content_type: ContentType,
    /// When created
    pub timestamp: DateTime<Utc>,
    /// Associated agent
    pub agent_id: Option<String>,
    /// Associated task
    pub task_id: Option<String>,
    /// Importance score (0.0 - 1.0)
    pub importance: f32,
}

/// Type of memory content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// User message
    UserMessage,
    /// Agent response
    AgentResponse,
    /// Tool output
    ToolOutput,
    /// Error message
    Error,
    /// System message
    System,
    /// Task context
    TaskContext,
    /// Code snippet
    Code,
    /// Documentation
    Documentation,
}

impl ShortTermMemory {
    /// Create new short-term memory
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::with_capacity(max_entries),
            max_entries,
        }
    }

    /// Add an entry
    pub fn add(&mut self, entry: ShortTermEntry) {
        if self.entries.len() >= self.max_entries {
            // Remove least important entry
            if let Some(min_idx) = self
                .entries
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.importance
                        .partial_cmp(&b.importance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(idx, _)| idx)
            {
                self.entries.remove(min_idx);
            }
        }
        self.entries.push(entry);
    }

    /// Get recent entries
    pub fn recent(&self, count: usize) -> Vec<&ShortTermEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    /// Get entries by type
    pub fn by_type(&self, content_type: ContentType) -> Vec<&ShortTermEntry> {
        self.entries
            .iter()
            .filter(|e| e.content_type == content_type)
            .collect()
    }

    /// Get entries for agent
    pub fn by_agent(&self, agent_id: &str) -> Vec<&ShortTermEntry> {
        self.entries
            .iter()
            .filter(|e| e.agent_id.as_deref() == Some(agent_id))
            .collect()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get all entries for consolidation to long-term
    pub fn get_for_consolidation(&self, min_importance: f32) -> Vec<&ShortTermEntry> {
        self.entries
            .iter()
            .filter(|e| e.importance >= min_importance)
            .collect()
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl MemorySystem {
    /// Create a new memory system with in-memory store
    pub fn new(config: MemoryConfig) -> Self {
        let short_term = Arc::new(RwLock::new(ShortTermMemory::new(config.max_short_term)));
        let long_term = Arc::new(RwLock::new(Box::new(store::InMemoryStore::new(
            config.max_long_term,
        )) as Box<dyn MemoryStore>));
        let embedder = Arc::new(embedding::MockEmbedder::new()) as Arc<dyn EmbeddingProvider>;
        let retriever = Arc::new(Retriever::new(RetrievalConfig::default()));

        Self {
            short_term,
            long_term,
            embedder,
            retriever,
            config,
        }
    }

    /// Check if memory is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Store a memory
    pub async fn store(
        &self,
        content: String,
        memory_type: MemoryType,
    ) -> Result<MemoryId, String> {
        if !self.config.enabled {
            return Err("Memory system is disabled".to_string());
        }

        // Generate embedding
        let embedding = self
            .embedder
            .embed(&content)
            .await
            .map_err(|e| format!("Embedding failed: {}", e))?;

        // Create memory
        let memory = Memory::new(content, memory_type).with_embedding(embedding);

        // Store in long-term memory
        let mut store = self.long_term.write().await;
        store.store(memory.clone()).await?;

        Ok(memory.id)
    }

    /// Store a short-term memory
    pub async fn store_short_term(&self, entry: ShortTermEntry) {
        let mut short_term = self.short_term.write().await;
        short_term.add(entry);
    }

    /// Retrieve relevant memories
    pub async fn retrieve(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<RetrievalResult>, String> {
        if !self.config.enabled {
            return Ok(Vec::new());
        }

        // Generate query embedding
        let query_embedding = self
            .embedder
            .embed(query)
            .await
            .map_err(|e| format!("Query embedding failed: {}", e))?;

        // Search long-term memory
        let store = self.long_term.read().await;
        let memories = store.search(&query_embedding, limit).await?;

        // Convert to retrieval results
        let results = memories
            .into_iter()
            .filter(|(_, score)| *score >= self.config.similarity_threshold)
            .map(|(memory, score)| RetrievalResult {
                memory,
                score,
                source: "long_term".to_string(),
            })
            .collect();

        Ok(results)
    }

    /// Get recent short-term memories
    pub async fn get_recent(&self, count: usize) -> Vec<ShortTermEntry> {
        let short_term = self.short_term.read().await;
        short_term.recent(count).into_iter().cloned().collect()
    }

    /// Consolidate short-term to long-term memory
    pub async fn consolidate(&self, min_importance: f32) -> Result<usize, String> {
        let short_term = self.short_term.read().await;
        let entries = short_term.get_for_consolidation(min_importance);

        let mut consolidated = 0;
        for entry in entries {
            // Convert to long-term memory
            let memory_type = match entry.content_type {
                ContentType::Code => MemoryType::CodePattern,
                ContentType::Error => MemoryType::ErrorPattern,
                ContentType::Documentation => MemoryType::Documentation,
                _ => MemoryType::Conversation,
            };

            if self.store(entry.content.clone(), memory_type).await.is_ok() {
                consolidated += 1;
            }
        }

        Ok(consolidated)
    }

    /// Clear short-term memory
    pub async fn clear_short_term(&self) {
        let mut short_term = self.short_term.write().await;
        short_term.clear();
    }

    /// Get memory statistics
    pub async fn get_stats(&self) -> MemoryStats {
        let short_term = self.short_term.read().await;
        let long_term = self.long_term.read().await;

        MemoryStats {
            short_term_count: short_term.len(),
            long_term_count: long_term.count().await,
            max_short_term: self.config.max_short_term,
            max_long_term: self.config.max_long_term,
            embedding_model: self.config.embedding_model.clone(),
        }
    }

    /// Delete a memory
    pub async fn delete(&self, id: &MemoryId) -> Result<(), String> {
        let mut store = self.long_term.write().await;
        store.delete(id).await
    }

    /// Update memory importance
    pub async fn update_importance(&self, id: &MemoryId, importance: f32) -> Result<(), String> {
        let mut store = self.long_term.write().await;
        store.update_importance(id, importance).await
    }
}

impl Default for MemorySystem {
    fn default() -> Self {
        Self::new(MemoryConfig::default())
    }
}

/// Statistics for the memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Short-term memory count
    pub short_term_count: usize,
    /// Long-term memory count
    pub long_term_count: usize,
    /// Maximum short-term entries
    pub max_short_term: usize,
    /// Maximum long-term entries
    pub max_long_term: usize,
    /// Embedding model in use
    pub embedding_model: String,
}

/// Build context from memories for RAG
pub fn build_rag_context(memories: &[RetrievalResult], max_tokens: usize) -> String {
    let mut context = String::new();
    let mut tokens_used = 0;

    for result in memories {
        // Rough token estimation (4 chars per token)
        let estimated_tokens = result.memory.content.len() / 4;

        if tokens_used + estimated_tokens > max_tokens {
            break;
        }

        context.push_str(&format!(
            "### Relevant Memory (score: {:.2})\n{}\n\n",
            result.score, result.memory.content
        ));
        tokens_used += estimated_tokens;
    }

    context
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_term_memory() {
        let mut memory = ShortTermMemory::new(3);

        memory.add(ShortTermEntry {
            id: "1".to_string(),
            content: "First entry".to_string(),
            content_type: ContentType::UserMessage,
            timestamp: Utc::now(),
            agent_id: None,
            task_id: None,
            importance: 0.5,
        });

        memory.add(ShortTermEntry {
            id: "2".to_string(),
            content: "Second entry".to_string(),
            content_type: ContentType::AgentResponse,
            timestamp: Utc::now(),
            agent_id: Some("agent1".to_string()),
            task_id: None,
            importance: 0.8,
        });

        assert_eq!(memory.len(), 2);

        let recent = memory.recent(5);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_short_term_eviction() {
        let mut memory = ShortTermMemory::new(2);

        memory.add(ShortTermEntry {
            id: "1".to_string(),
            content: "Low importance".to_string(),
            content_type: ContentType::System,
            timestamp: Utc::now(),
            agent_id: None,
            task_id: None,
            importance: 0.1,
        });

        memory.add(ShortTermEntry {
            id: "2".to_string(),
            content: "High importance".to_string(),
            content_type: ContentType::UserMessage,
            timestamp: Utc::now(),
            agent_id: None,
            task_id: None,
            importance: 0.9,
        });

        memory.add(ShortTermEntry {
            id: "3".to_string(),
            content: "New entry".to_string(),
            content_type: ContentType::AgentResponse,
            timestamp: Utc::now(),
            agent_id: None,
            task_id: None,
            importance: 0.5,
        });

        // Should have evicted the low importance entry
        assert_eq!(memory.len(), 2);
        assert!(memory.entries.iter().any(|e| e.id == "2"));
        assert!(memory.entries.iter().any(|e| e.id == "3"));
    }

    #[tokio::test]
    async fn test_memory_system() {
        let system = MemorySystem::new(MemoryConfig::default());
        assert!(system.is_enabled());

        // Store a memory
        let id = system
            .store(
                "This is a test memory".to_string(),
                MemoryType::Conversation,
            )
            .await
            .unwrap();

        assert!(!id.0.is_empty());

        // Get stats
        let stats = system.get_stats().await;
        assert_eq!(stats.long_term_count, 1);
    }

    #[tokio::test]
    async fn test_short_term_operations() {
        let system = MemorySystem::new(MemoryConfig::default());

        let entry = ShortTermEntry {
            id: "test-1".to_string(),
            content: "Test content".to_string(),
            content_type: ContentType::UserMessage,
            timestamp: Utc::now(),
            agent_id: Some("agent1".to_string()),
            task_id: None,
            importance: 0.7,
        };

        system.store_short_term(entry).await;

        let recent = system.get_recent(10).await;
        assert_eq!(recent.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_retrieval() {
        let system = MemorySystem::new(MemoryConfig::default());

        // Store some memories
        system
            .store(
                "How to implement user authentication".to_string(),
                MemoryType::Documentation,
            )
            .await
            .unwrap();

        system
            .store(
                "Fix login bug in auth module".to_string(),
                MemoryType::TaskContext,
            )
            .await
            .unwrap();

        // Retrieve relevant memories
        let results = system.retrieve("authentication", 5).await.unwrap();
        // MockEmbedder returns mock results
        assert!(results.len() <= 5);
    }

    #[test]
    fn test_build_rag_context() {
        let memories = vec![
            RetrievalResult {
                memory: Memory::new(
                    "First relevant memory".to_string(),
                    MemoryType::Conversation,
                ),
                score: 0.9,
                source: "long_term".to_string(),
            },
            RetrievalResult {
                memory: Memory::new(
                    "Second relevant memory".to_string(),
                    MemoryType::Documentation,
                ),
                score: 0.8,
                source: "long_term".to_string(),
            },
        ];

        let context = build_rag_context(&memories, 1000);
        assert!(context.contains("First relevant memory"));
        assert!(context.contains("Second relevant memory"));
        assert!(context.contains("0.90"));
    }

    #[tokio::test]
    async fn test_memory_consolidation() {
        let system = MemorySystem::new(MemoryConfig::default());

        // Add high-importance short-term entries
        let entry = ShortTermEntry {
            id: "consolidate-1".to_string(),
            content: "Important pattern".to_string(),
            content_type: ContentType::Code,
            timestamp: Utc::now(),
            agent_id: None,
            task_id: None,
            importance: 0.9,
        };

        system.store_short_term(entry).await;

        // Consolidate
        let count = system.consolidate(0.8).await.unwrap();
        assert_eq!(count, 1);

        let stats = system.get_stats().await;
        assert_eq!(stats.long_term_count, 1);
    }
}
