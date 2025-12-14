/// Bridge between ccswarm's memory system and context management
///
/// This module provides context management capabilities for ccswarm
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::session::memory::{SessionMemory, WorkingMemoryType};

/// Enhanced memory summary with context statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnhancedMemorySummary {
    /// Base memory summary from ccswarm
    pub base_summary: crate::session::memory::MemorySummary,
    /// Context statistics
    pub context_stats: ContextStats,
}

/// Context statistics for tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContextStats {
    /// Total tokens currently in context
    pub total_tokens: usize,
    /// Total number of messages
    pub message_count: usize,
    /// Compression ratio achieved
    pub compression_ratio: f64,
    /// Token savings percentage
    pub token_savings_percent: f64,
}

/// Context bridge that integrates context with ccswarm's memory
pub struct ContextBridge {
    /// Reference to ccswarm's session memory for compatibility
    session_memory: Arc<RwLock<SessionMemory>>,

    /// Agent ID for context tracking
    #[allow(dead_code)]
    agent_id: String,

    /// Session ID for context tracking
    #[allow(dead_code)]
    session_id: String,
}

impl ContextBridge {
    /// Create a new context bridge
    pub fn new(
        session_memory: Arc<RwLock<SessionMemory>>,
        agent_id: String,
        session_id: String,
    ) -> Self {
        Self {
            session_memory,
            agent_id,
            session_id,
        }
    }

    /// Add content to working memory
    pub async fn add_to_working_memory(
        &self,
        content: String,
        memory_type: WorkingMemoryType,
        priority: f32,
    ) -> Result<()> {
        // Add to ccswarm's memory system
        let mut memory = self.session_memory.write().await;
        memory.add_to_working_memory(content, memory_type, priority);
        Ok(())
    }

    /// Get memory summary with context stats
    pub async fn get_enhanced_memory_summary(&self) -> Result<EnhancedMemorySummary> {
        let memory = self.session_memory.read().await;
        let base_summary = memory.get_summary();

        Ok(EnhancedMemorySummary {
            base_summary,
            context_stats: ContextStats {
                total_tokens: 0,
                message_count: 0,
                compression_ratio: 1.0,
                token_savings_percent: 0.0,
            },
        })
    }

    /// Estimate tokens in a string (simple approximation)
    #[allow(dead_code)]
    fn estimate_tokens(content: &str) -> usize {
        // Simple approximation: ~4 characters per token
        content.len() / 4
    }
}
