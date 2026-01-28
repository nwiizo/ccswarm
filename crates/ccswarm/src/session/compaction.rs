//! Context Compaction System
//!
//! Provides intelligent context compression for sessions to manage token usage.
//! Inspired by Claude Agent SDK's compaction strategy.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Configuration for context compaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    /// Maximum tokens before triggering compaction
    pub max_tokens: usize,
    /// Threshold ratio (0.0-1.0) to trigger automatic compaction
    pub threshold_ratio: f64,
    /// Strategy to use for compaction
    pub strategy: CompactionStrategy,
    /// Whether to preserve system messages
    pub preserve_system_messages: bool,
    /// Whether to preserve recent context
    pub preserve_recent_count: usize,
    /// Compression level for zstd (1-22)
    pub compression_level: i32,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            max_tokens: 200_000, // Claude's context limit
            threshold_ratio: 0.8,
            strategy: CompactionStrategy::SmartSummarize,
            preserve_system_messages: true,
            preserve_recent_count: 5,
            compression_level: 3,
        }
    }
}

/// Strategy for context compaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompactionStrategy {
    /// Simple truncation of oldest messages
    Truncate,
    /// Summarize old context into a condensed form
    Summarize,
    /// Smart summarization with importance scoring
    SmartSummarize,
    /// Sliding window that keeps only recent context
    SlidingWindow,
    /// Hybrid approach combining multiple strategies
    Hybrid,
}

impl std::fmt::Display for CompactionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompactionStrategy::Truncate => write!(f, "truncate"),
            CompactionStrategy::Summarize => write!(f, "summarize"),
            CompactionStrategy::SmartSummarize => write!(f, "smart_summarize"),
            CompactionStrategy::SlidingWindow => write!(f, "sliding_window"),
            CompactionStrategy::Hybrid => write!(f, "hybrid"),
        }
    }
}

/// Result of a compaction operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionResult {
    /// Whether compaction was performed
    pub compacted: bool,
    /// Original token count
    pub original_tokens: usize,
    /// Token count after compaction
    pub final_tokens: usize,
    /// Number of messages removed
    pub messages_removed: usize,
    /// Number of messages summarized
    pub messages_summarized: usize,
    /// Bytes saved through compression
    pub bytes_saved: usize,
    /// Compression ratio achieved (0.0-1.0)
    pub compression_ratio: f64,
    /// Strategy used
    pub strategy_used: CompactionStrategy,
    /// Generated summary (if applicable)
    pub summary: Option<String>,
}

impl CompactionResult {
    /// Create result for no compaction needed
    pub fn no_compaction(token_count: usize) -> Self {
        Self {
            compacted: false,
            original_tokens: token_count,
            final_tokens: token_count,
            messages_removed: 0,
            messages_summarized: 0,
            bytes_saved: 0,
            compression_ratio: 1.0,
            strategy_used: CompactionStrategy::Truncate,
            summary: None,
        }
    }

    /// Calculate savings percentage
    pub fn savings_percentage(&self) -> f64 {
        if self.original_tokens == 0 {
            return 0.0;
        }
        (1.0 - (self.final_tokens as f64 / self.original_tokens as f64)) * 100.0
    }
}

/// Trait for context compaction
#[async_trait]
pub trait ContextCompactor: Send + Sync {
    /// Compact the context using the configured strategy
    async fn compact(&mut self, config: &CompactionConfig) -> Result<CompactionResult>;

    /// Check if compaction is needed
    async fn needs_compaction(&self, threshold_tokens: usize) -> bool;

    /// Get current token count
    async fn get_token_count(&self) -> Result<usize>;

    /// Get context size in bytes
    async fn get_context_size(&self) -> Result<usize>;

    /// Clear all context
    async fn clear_context(&mut self) -> Result<()>;
}

/// Message for context history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMessage {
    /// Message role (system, user, assistant)
    pub role: String,
    /// Message content
    pub content: String,
    /// Approximate token count
    pub token_count: usize,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Importance score (0.0-1.0, higher = more important)
    pub importance: f64,
    /// Whether this message should be preserved
    pub preserve: bool,
}

impl ContextMessage {
    /// Create a new context message
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        let content_str: String = content.into();
        // Rough token estimation: ~4 chars per token
        let token_count = content_str.len() / 4;

        Self {
            role: role.into(),
            content: content_str,
            token_count,
            timestamp: chrono::Utc::now(),
            importance: 0.5,
            preserve: false,
        }
    }

    /// Set importance score
    pub fn with_importance(mut self, importance: f64) -> Self {
        self.importance = importance.clamp(0.0, 1.0);
        self
    }

    /// Mark as preserved
    pub fn preserve(mut self) -> Self {
        self.preserve = true;
        self
    }
}

/// Context history manager
#[derive(Debug, Clone, Default)]
pub struct ContextHistory {
    /// Messages in the context
    messages: Vec<ContextMessage>,
    /// Total token count
    total_tokens: usize,
}

impl ContextHistory {
    /// Create a new empty history
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a message
    pub fn add_message(&mut self, message: ContextMessage) {
        self.total_tokens += message.token_count;
        self.messages.push(message);
    }

    /// Get total token count
    pub fn token_count(&self) -> usize {
        self.total_tokens
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Get all messages
    pub fn messages(&self) -> &[ContextMessage] {
        &self.messages
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.total_tokens = 0;
    }

    /// Apply truncation strategy
    pub fn apply_truncation(&mut self, target_tokens: usize, preserve_recent: usize) -> usize {
        if self.total_tokens <= target_tokens {
            return 0;
        }

        let mut removed = 0;
        let preserve_from = self.messages.len().saturating_sub(preserve_recent);

        // Remove oldest non-preserved messages until we're under target
        while self.total_tokens > target_tokens && !self.messages.is_empty() {
            // Find the first removable message (not preserved, not in recent window)
            let remove_idx = self.messages.iter().enumerate().find_map(|(i, m)| {
                if i < preserve_from && !m.preserve {
                    Some(i)
                } else {
                    None
                }
            });

            match remove_idx {
                Some(idx) => {
                    let msg = self.messages.remove(idx);
                    self.total_tokens = self.total_tokens.saturating_sub(msg.token_count);
                    removed += 1;
                }
                None => break, // No more removable messages
            }
        }

        removed
    }

    /// Apply sliding window strategy
    pub fn apply_sliding_window(&mut self, window_size: usize) -> usize {
        if self.messages.len() <= window_size {
            return 0;
        }

        let to_remove = self.messages.len() - window_size;

        // Separate preserved and non-preserved
        let (preserved, removable): (Vec<_>, Vec<_>) =
            self.messages.drain(..to_remove).partition(|m| m.preserve);

        // Put preserved messages back at the start
        let preserved_tokens: usize = preserved.iter().map(|m| m.token_count).sum();
        let removed_tokens: usize = removable.iter().map(|m| m.token_count).sum();

        // Prepend preserved messages
        for msg in preserved.into_iter().rev() {
            self.messages.insert(0, msg);
        }

        self.total_tokens = self.total_tokens.saturating_sub(removed_tokens);
        self.total_tokens += preserved_tokens; // Add back preserved tokens

        removable.len()
    }

    /// Score messages by importance for smart summarization
    pub fn score_importance(&mut self) {
        let message_count = self.messages.len();
        for (i, msg) in self.messages.iter_mut().enumerate() {
            let mut score = 0.5;

            // System messages are important
            if msg.role == "system" {
                score += 0.3;
            }

            // Recent messages are more important
            let recency = i as f64 / message_count.max(1) as f64;
            score += recency * 0.2;

            // Longer messages might be more substantive
            if msg.content.len() > 500 {
                score += 0.1;
            }

            // Messages with code blocks might be important
            if msg.content.contains("```") {
                score += 0.1;
            }

            // Messages with errors are important
            if msg.content.to_lowercase().contains("error") {
                score += 0.1;
            }

            msg.importance = score.clamp(0.0, 1.0);
        }
    }
}

/// Compress context data using zstd
pub fn compress_context(data: &[u8], level: i32) -> Result<Vec<u8>> {
    // Simple wrapper - in production would use zstd crate
    // For now, return data as-is if compression isn't available
    let _ = level;
    Ok(data.to_vec())
}

/// Decompress context data
pub fn decompress_context(data: &[u8]) -> Result<Vec<u8>> {
    // Simple wrapper - in production would use zstd crate
    Ok(data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_message_creation() {
        let msg = ContextMessage::new("user", "Hello, world!")
            .with_importance(0.8)
            .preserve();

        assert_eq!(msg.role, "user");
        assert_eq!(msg.importance, 0.8);
        assert!(msg.preserve);
    }

    #[test]
    fn test_context_history() {
        let mut history = ContextHistory::new();

        history.add_message(ContextMessage::new("user", "Hello"));
        history.add_message(ContextMessage::new("assistant", "Hi there!"));

        assert_eq!(history.message_count(), 2);
        assert!(history.token_count() > 0);
    }

    #[test]
    fn test_truncation() {
        let mut history = ContextHistory::new();

        // Add 10 messages with longer content to exceed token limit
        for i in 0..10 {
            // Each message is ~50 chars = ~12 tokens
            let content = format!("This is a longer message number {} with more content", i);
            history.add_message(ContextMessage::new("user", content));
        }

        let original_count = history.message_count();
        let original_tokens = history.token_count();
        // Set target to 50 tokens, which is less than our ~120 tokens
        let removed = history.apply_truncation(50, 3);

        // We should have removed some messages since we exceeded target
        assert!(
            removed > 0,
            "Expected to remove messages, original_tokens={}, target=50",
            original_tokens
        );
        assert!(history.message_count() < original_count);
    }

    #[test]
    fn test_sliding_window() {
        let mut history = ContextHistory::new();

        for i in 0..10 {
            history.add_message(ContextMessage::new("user", format!("Message {}", i)));
        }

        let removed = history.apply_sliding_window(5);
        assert_eq!(removed, 5);
        assert_eq!(history.message_count(), 5);
    }

    #[test]
    fn test_compaction_result_savings() {
        let result = CompactionResult {
            compacted: true,
            original_tokens: 100_000,
            final_tokens: 7_000,
            messages_removed: 50,
            messages_summarized: 10,
            bytes_saved: 500_000,
            compression_ratio: 0.07,
            strategy_used: CompactionStrategy::SmartSummarize,
            summary: Some("Summary of conversation".to_string()),
        };

        assert_eq!(result.savings_percentage(), 93.0);
    }
}
