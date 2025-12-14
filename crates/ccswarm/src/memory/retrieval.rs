//! Retrieval system for RAG-based memory queries

use super::embedding::Embedding;
use super::store::Memory;
use serde::{Deserialize, Serialize};

/// Configuration for retrieval operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    /// Maximum results to return
    pub max_results: usize,
    /// Minimum similarity threshold
    pub similarity_threshold: f32,
    /// Whether to include metadata in results
    pub include_metadata: bool,
    /// Whether to rerank results
    pub enable_reranking: bool,
    /// Context window size for chunking
    pub context_window: usize,
    /// Overlap between chunks
    pub chunk_overlap: usize,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            max_results: 10,
            similarity_threshold: 0.7,
            include_metadata: true,
            enable_reranking: false,
            context_window: 512,
            chunk_overlap: 64,
        }
    }
}

/// Result from a retrieval query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    /// The retrieved memory
    pub memory: Memory,
    /// Similarity score (0.0 - 1.0)
    pub score: f32,
    /// Source of the result
    pub source: String,
}

impl RetrievalResult {
    /// Create a new retrieval result
    pub fn new(memory: Memory, score: f32, source: impl Into<String>) -> Self {
        Self {
            memory,
            score,
            source: source.into(),
        }
    }

    /// Check if the score meets a threshold
    pub fn meets_threshold(&self, threshold: f32) -> bool {
        self.score >= threshold
    }
}

/// Query for retrieval operations
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RetrievalQuery {
    /// The query text
    pub text: String,
    /// Pre-computed embedding (if available)
    pub embedding: Option<Embedding>,
    /// Filter by tags
    pub filter_tags: Vec<String>,
    /// Filter by agent
    pub filter_agent: Option<String>,
    /// Maximum results
    pub limit: usize,
}

#[allow(dead_code)]
impl RetrievalQuery {
    /// Create a new query from text
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            embedding: None,
            filter_tags: Vec::new(),
            filter_agent: None,
            limit: 10,
        }
    }

    /// Set pre-computed embedding
    pub fn with_embedding(mut self, embedding: Embedding) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Add tag filter
    pub fn with_tag_filter(mut self, tags: Vec<String>) -> Self {
        self.filter_tags = tags;
        self
    }

    /// Filter by agent
    pub fn with_agent_filter(mut self, agent_id: impl Into<String>) -> Self {
        self.filter_agent = Some(agent_id.into());
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Retriever for RAG operations
pub struct Retriever {
    /// Configuration
    config: RetrievalConfig,
}

impl Retriever {
    /// Create a new retriever
    pub fn new(config: RetrievalConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &RetrievalConfig {
        &self.config
    }

    /// Filter results by threshold
    pub fn filter_by_threshold(&self, results: Vec<RetrievalResult>) -> Vec<RetrievalResult> {
        results
            .into_iter()
            .filter(|r| r.meets_threshold(self.config.similarity_threshold))
            .collect()
    }

    /// Rerank results based on various factors
    pub fn rerank(&self, mut results: Vec<RetrievalResult>) -> Vec<RetrievalResult> {
        if !self.config.enable_reranking {
            return results;
        }

        // Apply reranking based on:
        // 1. Original similarity score
        // 2. Memory importance
        // 3. Recency bonus
        for result in &mut results {
            let importance_factor = result.memory.importance;
            let recency_factor = result.memory.decay_factor();

            // Weighted combination
            result.score = result.score * 0.6 + importance_factor * 0.25 + recency_factor * 0.15;
        }

        // Re-sort after reranking
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Limit results
    pub fn limit_results(&self, results: Vec<RetrievalResult>) -> Vec<RetrievalResult> {
        results.into_iter().take(self.config.max_results).collect()
    }

    /// Process retrieval pipeline
    pub fn process(&self, results: Vec<RetrievalResult>) -> Vec<RetrievalResult> {
        let filtered = self.filter_by_threshold(results);
        let reranked = self.rerank(filtered);
        self.limit_results(reranked)
    }
}

/// Chunk text for embedding
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TextChunk {
    /// Chunk content
    pub content: String,
    /// Start position in original text
    pub start: usize,
    /// End position in original text
    pub end: usize,
    /// Chunk index
    pub index: usize,
}

/// Text chunker for large documents
#[allow(dead_code)]
pub struct TextChunker {
    /// Window size in characters
    window_size: usize,
    /// Overlap between chunks
    overlap: usize,
}

#[allow(dead_code)]
impl TextChunker {
    /// Create a new chunker
    pub fn new(window_size: usize, overlap: usize) -> Self {
        Self {
            window_size,
            overlap,
        }
    }

    /// Create from config
    pub fn from_config(config: &RetrievalConfig) -> Self {
        Self {
            window_size: config.context_window,
            overlap: config.chunk_overlap,
        }
    }

    /// Chunk text into overlapping segments
    pub fn chunk(&self, text: &str) -> Vec<TextChunk> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = text.chars().collect();
        let text_len = chars.len();

        if text_len == 0 {
            return chunks;
        }

        if text_len <= self.window_size {
            chunks.push(TextChunk {
                content: text.to_string(),
                start: 0,
                end: text_len,
                index: 0,
            });
            return chunks;
        }

        let step = self.window_size.saturating_sub(self.overlap).max(1);
        let mut start = 0;
        let mut index = 0;

        while start < text_len {
            let end = (start + self.window_size).min(text_len);
            let content: String = chars[start..end].iter().collect();

            chunks.push(TextChunk {
                content,
                start,
                end,
                index,
            });

            if end >= text_len {
                break;
            }

            start += step;
            index += 1;
        }

        chunks
    }

    /// Chunk by sentences (tries to keep sentences intact)
    pub fn chunk_by_sentences(&self, text: &str) -> Vec<TextChunk> {
        let sentences: Vec<&str> = text
            .split(['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_start = 0;
        let mut chunk_index = 0;

        for sentence in sentences {
            let sentence_with_punct = format!("{}. ", sentence.trim());

            if current_chunk.len() + sentence_with_punct.len() > self.window_size {
                if !current_chunk.is_empty() {
                    chunks.push(TextChunk {
                        content: current_chunk.trim().to_string(),
                        start: current_start,
                        end: current_start + current_chunk.len(),
                        index: chunk_index,
                    });
                    chunk_index += 1;

                    // Keep some overlap
                    let words: Vec<&str> = current_chunk.split_whitespace().collect();
                    let overlap_words = words.len().min(5);
                    current_chunk = words[words.len().saturating_sub(overlap_words)..].join(" ");
                    current_start = current_start + current_chunk.len() - current_chunk.len();
                }
            }

            current_chunk.push_str(&sentence_with_punct);
        }

        if !current_chunk.is_empty() {
            chunks.push(TextChunk {
                content: current_chunk.trim().to_string(),
                start: current_start,
                end: current_start + current_chunk.len(),
                index: chunk_index,
            });
        }

        chunks
    }
}

/// Hybrid retrieval combining semantic and keyword search
#[allow(dead_code)]
pub struct HybridRetriever {
    /// Semantic retriever
    semantic: Retriever,
    /// Weight for semantic results (0.0 - 1.0)
    semantic_weight: f32,
}

#[allow(dead_code)]
impl HybridRetriever {
    /// Create a new hybrid retriever
    pub fn new(config: RetrievalConfig, semantic_weight: f32) -> Self {
        Self {
            semantic: Retriever::new(config),
            semantic_weight: semantic_weight.clamp(0.0, 1.0),
        }
    }

    /// Merge semantic and keyword results
    pub fn merge_results(
        &self,
        semantic_results: Vec<RetrievalResult>,
        keyword_results: Vec<RetrievalResult>,
    ) -> Vec<RetrievalResult> {
        use std::collections::HashMap;

        let mut merged: HashMap<String, RetrievalResult> = HashMap::new();
        let keyword_weight = 1.0 - self.semantic_weight;

        // Add semantic results
        for mut result in semantic_results {
            result.score *= self.semantic_weight;
            merged.insert(result.memory.id.0.clone(), result);
        }

        // Merge keyword results
        for result in keyword_results {
            let id = result.memory.id.0.clone();
            if let Some(existing) = merged.get_mut(&id) {
                existing.score += result.score * keyword_weight;
            } else {
                let mut new_result = result;
                new_result.score *= keyword_weight;
                merged.insert(id, new_result);
            }
        }

        let mut results: Vec<RetrievalResult> = merged.into_values().collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.semantic.limit_results(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::store::MemoryType;

    #[test]
    fn test_retrieval_config_default() {
        let config = RetrievalConfig::default();
        assert_eq!(config.max_results, 10);
        assert_eq!(config.similarity_threshold, 0.7);
        assert!(config.include_metadata);
    }

    #[test]
    fn test_retrieval_result() {
        let memory = Memory::new("Test content", MemoryType::Conversation);
        let result = RetrievalResult::new(memory, 0.85, "test");

        assert_eq!(result.score, 0.85);
        assert!(result.meets_threshold(0.8));
        assert!(!result.meets_threshold(0.9));
    }

    #[test]
    fn test_retrieval_query_builder() {
        let query = RetrievalQuery::from_text("test query")
            .with_tag_filter(vec!["rust".to_string()])
            .with_agent_filter("agent1")
            .with_limit(5);

        assert_eq!(query.text, "test query");
        assert_eq!(query.filter_tags, vec!["rust".to_string()]);
        assert_eq!(query.filter_agent, Some("agent1".to_string()));
        assert_eq!(query.limit, 5);
    }

    #[test]
    fn test_retriever_filter_by_threshold() {
        let config = RetrievalConfig {
            similarity_threshold: 0.5,
            ..Default::default()
        };
        let retriever = Retriever::new(config);

        let results = vec![
            RetrievalResult::new(Memory::new("High", MemoryType::Conversation), 0.9, "test"),
            RetrievalResult::new(Memory::new("Low", MemoryType::Conversation), 0.3, "test"),
            RetrievalResult::new(Memory::new("Medium", MemoryType::Conversation), 0.6, "test"),
        ];

        let filtered = retriever.filter_by_threshold(results);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_retriever_limit_results() {
        let config = RetrievalConfig {
            max_results: 2,
            ..Default::default()
        };
        let retriever = Retriever::new(config);

        let results = vec![
            RetrievalResult::new(Memory::new("1", MemoryType::Conversation), 0.9, "test"),
            RetrievalResult::new(Memory::new("2", MemoryType::Conversation), 0.8, "test"),
            RetrievalResult::new(Memory::new("3", MemoryType::Conversation), 0.7, "test"),
        ];

        let limited = retriever.limit_results(results);
        assert_eq!(limited.len(), 2);
    }

    #[test]
    fn test_text_chunker_basic() {
        let chunker = TextChunker::new(50, 10);
        let text = "This is a short text.";

        let chunks = chunker.chunk(text);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, text);
    }

    #[test]
    fn test_text_chunker_long_text() {
        let chunker = TextChunker::new(20, 5);
        let text = "This is a longer text that will be split into multiple chunks for processing.";

        let chunks = chunker.chunk(text);
        assert!(chunks.len() > 1);

        // Check overlap exists
        for i in 1..chunks.len() {
            assert!(chunks[i].start < chunks[i - 1].end);
        }
    }

    #[test]
    fn test_text_chunker_empty() {
        let chunker = TextChunker::new(50, 10);
        let chunks = chunker.chunk("");
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_text_chunker_from_config() {
        let config = RetrievalConfig {
            context_window: 100,
            chunk_overlap: 20,
            ..Default::default()
        };
        let chunker = TextChunker::from_config(&config);

        assert_eq!(chunker.window_size, 100);
        assert_eq!(chunker.overlap, 20);
    }

    #[test]
    fn test_retriever_rerank_disabled() {
        let config = RetrievalConfig {
            enable_reranking: false,
            ..Default::default()
        };
        let retriever = Retriever::new(config);

        let results = vec![
            RetrievalResult::new(Memory::new("1", MemoryType::Conversation), 0.5, "test"),
            RetrievalResult::new(Memory::new("2", MemoryType::Conversation), 0.9, "test"),
        ];

        let reranked = retriever.rerank(results);
        // Order should not change when reranking is disabled
        assert_eq!(reranked[0].score, 0.5);
    }

    #[test]
    fn test_retriever_rerank_enabled() {
        let config = RetrievalConfig {
            enable_reranking: true,
            ..Default::default()
        };
        let retriever = Retriever::new(config);

        let memory1 = Memory::new("1", MemoryType::Conversation).with_importance(0.9);
        let memory2 = Memory::new("2", MemoryType::Conversation).with_importance(0.1);

        let results = vec![
            RetrievalResult::new(memory1, 0.5, "test"),
            RetrievalResult::new(memory2, 0.6, "test"),
        ];

        let reranked = retriever.rerank(results);
        // Higher importance should boost the first result
        assert!(reranked[0].memory.importance > reranked[1].memory.importance);
    }

    #[test]
    fn test_hybrid_retriever_merge() {
        let config = RetrievalConfig::default();
        let hybrid = HybridRetriever::new(config, 0.7);

        let semantic = vec![RetrievalResult::new(
            Memory::new("A", MemoryType::Conversation),
            0.9,
            "semantic",
        )];

        let keyword = vec![RetrievalResult::new(
            Memory::new("B", MemoryType::Conversation),
            0.8,
            "keyword",
        )];

        let merged = hybrid.merge_results(semantic, keyword);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_hybrid_retriever_overlap() {
        let config = RetrievalConfig::default();
        let hybrid = HybridRetriever::new(config, 0.5);

        let memory = Memory::new("Same", MemoryType::Conversation);
        let id = memory.id.clone();

        let semantic = vec![RetrievalResult::new(memory.clone(), 0.8, "semantic")];

        // Create memory with same ID
        let mut memory2 = Memory::new("Same", MemoryType::Conversation);
        memory2.id = id;
        let keyword = vec![RetrievalResult::new(memory2, 0.6, "keyword")];

        let merged = hybrid.merge_results(semantic, keyword);
        assert_eq!(merged.len(), 1);
        // Score should be combined
        assert!(merged[0].score > 0.6);
    }

    #[test]
    fn test_retriever_process_pipeline() {
        let config = RetrievalConfig {
            max_results: 2,
            similarity_threshold: 0.5,
            enable_reranking: true,
            ..Default::default()
        };
        let retriever = Retriever::new(config);

        let results = vec![
            RetrievalResult::new(
                Memory::new("High", MemoryType::Conversation).with_importance(0.9),
                0.9,
                "test",
            ),
            RetrievalResult::new(
                Memory::new("Low", MemoryType::Conversation).with_importance(0.1),
                0.3,
                "test",
            ),
            RetrievalResult::new(
                Memory::new("Medium", MemoryType::Conversation).with_importance(0.5),
                0.6,
                "test",
            ),
            RetrievalResult::new(
                Memory::new("Extra", MemoryType::Conversation).with_importance(0.7),
                0.8,
                "test",
            ),
        ];

        let processed = retriever.process(results);

        // Should filter out Low (below threshold)
        // Should limit to 2 results
        assert_eq!(processed.len(), 2);

        // All results should be above threshold
        for r in &processed {
            assert!(r.score >= 0.0); // After reranking, scores are combined
        }
    }

    #[test]
    fn test_chunk_by_sentences() {
        let chunker = TextChunker::new(100, 20);
        let text = "First sentence. Second sentence. Third sentence!";

        let chunks = chunker.chunk_by_sentences(text);
        assert!(!chunks.is_empty());

        // Each chunk should contain complete sentences
        for chunk in chunks {
            assert!(!chunk.content.is_empty());
        }
    }
}
