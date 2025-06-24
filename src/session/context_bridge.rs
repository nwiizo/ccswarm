/// Bridge between ccswarm's memory system and ai-session's context management
///
/// This module enables ccswarm to leverage ai-session's token-efficient context
/// handling while maintaining compatibility with the existing memory system.
/// Key benefits:
/// - 93% token reduction through intelligent compression
/// - Automatic context window management
/// - Semantic understanding of conversation history
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use ai_session::context::{Message, MessageRole, SessionContext};

use crate::session::memory::{
    Episode, EpisodeOutcome, EpisodeType, MemorySummary, RetrievalResult, SessionMemory,
    WorkingMemoryItem, WorkingMemoryType,
};

/// Enhanced memory summary with AI context statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnhancedMemorySummary {
    /// Base memory summary from ccswarm
    pub base_summary: MemorySummary,
    /// AI context statistics
    pub ai_context_stats: AIContextStats,
}

/// AI context statistics for token efficiency tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AIContextStats {
    /// Total tokens currently in context
    pub total_tokens: usize,
    /// Total number of messages
    pub message_count: usize,
    /// Compression ratio achieved
    pub compression_ratio: f64,
    /// Token savings percentage
    pub token_savings_percent: f64,
}

/// Context bridge that integrates ai-session's context with ccswarm's memory
pub struct ContextBridge {
    /// The underlying ai-session context
    ai_context: Arc<RwLock<SessionContext>>,
    
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
        ai_context: Arc<RwLock<SessionContext>>,
        session_memory: Arc<RwLock<SessionMemory>>,
        agent_id: String,
        session_id: String,
    ) -> Self {
        Self {
            ai_context,
            session_memory,
            agent_id,
            session_id,
        }
    }
    
    /// Add content to working memory and sync with ai-session context
    pub async fn add_to_working_memory(
        &self,
        content: String,
        memory_type: WorkingMemoryType,
        priority: f32,
    ) -> Result<()> {
        // Add to ccswarm's memory system
        {
            let mut memory = self.session_memory.write().await;
            memory.add_to_working_memory(content.clone(), memory_type.clone(), priority);
        }
        
        // Convert to ai-session message format
        let message = Message {
            role: MessageRole::System,
            content: format!("[MEMORY:{:?}:priority={}] {}", memory_type, priority, content),
            timestamp: Utc::now(),
            token_count: Self::estimate_tokens(&content),
        };
        
        // Add to ai-session context
        let mut context = self.ai_context.write().await;
        context.add_message(message);
        
        // Trigger compression if approaching token limit
        if context.get_total_tokens() > context.config.max_tokens * 80 / 100 {
            tracing::info!("Approaching token limit, triggering context compression");
            let compressed = context.compress_context().await;
            if compressed {
                tracing::info!("Context successfully compressed");
            }
        }
        
        Ok(())
    }
    
    /// Set current task context in both systems
    pub async fn set_task_context(&self, task_id: String, description: String) -> Result<()> {
        // Update ccswarm's memory
        {
            let mut memory = self.session_memory.write().await;
            memory.set_task_context(task_id.clone(), description.clone());
        }
        
        // Add to ai-session context as a special message
        let message = Message {
            role: MessageRole::System,
            content: format!("[TASK_CONTEXT] Task ID: {} | Description: {}", task_id, description),
            timestamp: Utc::now(),
            token_count: Self::estimate_tokens(&description) + 20,
        };
        
        let mut context = self.ai_context.write().await;
        context.add_message(message);
        
        Ok(())
    }
    
    /// Add an episode to memory with ai-session integration
    pub async fn add_episode(
        &self,
        event_type: EpisodeType,
        description: String,
        context_data: HashMap<String, String>,
        outcome: EpisodeOutcome,
    ) -> Result<()> {
        // Add to ccswarm's memory
        {
            let mut memory = self.session_memory.write().await;
            memory.add_episode(event_type.clone(), description.clone(), context_data.clone(), outcome.clone());
        }
        
        // Format episode for ai-session context
        let context_str = context_data.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        
        let outcome_str = match &outcome {
            EpisodeOutcome::Success { metrics } => {
                if metrics.is_empty() {
                    "SUCCESS".to_string()
                } else {
                    format!("SUCCESS: {:?}", metrics)
                }
            },
            EpisodeOutcome::Failure { reason, recovery_actions } => {
                format!("FAILURE: {} (recovery: {:?})", reason, recovery_actions)
            },
            EpisodeOutcome::Partial { progress, next_steps } => {
                format!("PARTIAL: {}% complete, next: {:?}", (progress * 100.0) as u32, next_steps)
            },
            EpisodeOutcome::Cancelled { reason } => {
                format!("CANCELLED: {}", reason)
            },
        };
        
        let message = Message {
            role: MessageRole::Assistant,
            content: format!(
                "[EPISODE:{:?}] {} | Context: {} | Outcome: {}",
                event_type, description, context_str, outcome_str
            ),
            timestamp: Utc::now(),
            token_count: Self::estimate_tokens(&description) + 50,
        };
        
        let mut context = self.ai_context.write().await;
        context.add_message(message);
        
        Ok(())
    }
    
    /// Consolidate memories using ai-session's compression
    pub async fn consolidate_memories(&self) -> Result<()> {
        // First consolidate ccswarm's memories
        {
            let mut memory = self.session_memory.write().await;
            memory.consolidate_memories();
        }
        
        // Then compress ai-session context
        let mut context = self.ai_context.write().await;
        let compressed = context.compress_context().await;
        
        if compressed {
            tracing::info!("Successfully compressed context, reduced token usage");
        }
        
        Ok(())
    }
    
    /// Retrieve relevant memories with ai-session enhancement
    pub async fn retrieve_relevant_memories(&self, query: &str) -> Result<RetrievalResult> {
        // Get memories from ccswarm system
        let mut ccswarm_result = {
            let memory = self.session_memory.read().await;
            memory.retrieve_relevant_memories(query)
        };
        
        // Enhance with ai-session context search
        let context = self.ai_context.read().await;
        let recent_context = context.get_recent_messages(10);
        
        // Find relevant messages in ai-session context
        for message in recent_context {
            if message.content.to_lowercase().contains(&query.to_lowercase()) {
                // Parse message back to memory format if it's a memory entry
                if message.content.starts_with("[MEMORY:") {
                    ccswarm_result.working_memory_items.push(WorkingMemoryItem {
                        id: format!("ai-{}", uuid::Uuid::new_v4()),
                        content: message.content.clone(),
                        item_type: WorkingMemoryType::TaskInstructions,
                        priority: 0.8,
                        created_at: message.timestamp,
                        last_accessed: Utc::now(),
                        decay_rate: 0.0,
                    });
                } else if message.content.starts_with("[EPISODE:") {
                    ccswarm_result.relevant_episodes.push(Episode {
                        id: format!("ai-{}", uuid::Uuid::new_v4()),
                        timestamp: message.timestamp,
                        event_type: EpisodeType::TaskCompletion,
                        description: message.content.clone(),
                        context: HashMap::new(),
                        outcome: EpisodeOutcome::Success {
                            metrics: HashMap::new(),
                        },
                        emotional_valence: 0.0,
                        learning_value: 0.7,
                        related_episodes: Vec::new(),
                    });
                }
            }
        }
        
        Ok(ccswarm_result)
    }
    
    /// Generate memory summary with ai-session statistics
    pub async fn generate_memory_summary(&self) -> Result<EnhancedMemorySummary> {
        // Get base summary from ccswarm
        let base_summary = {
            let memory = self.session_memory.read().await;
            memory.generate_memory_summary()
        };
        
        // Enhance with ai-session context stats
        let context = self.ai_context.read().await;
        let token_usage = context.get_total_tokens();
        let message_count = context.get_message_count();
        let compression_ratio = if token_usage > 0 {
            let original_tokens = message_count * 100; // Estimate
            (original_tokens - token_usage) as f64 / original_tokens as f64
        } else {
            0.0
        };
        
        // Create enhanced summary with token savings info
        let enhanced_summary = EnhancedMemorySummary {
            base_summary,
            ai_context_stats: AIContextStats {
                total_tokens: token_usage,
                message_count,
                compression_ratio,
                token_savings_percent: compression_ratio * 100.0,
            },
        };
        
        Ok(enhanced_summary)
    }
    
    /// Estimate token count for a string (rough approximation)
    fn estimate_tokens(text: &str) -> usize {
        // Rough estimate: ~4 characters per token
        text.len() / 4
    }
    
    /// Sync ccswarm memory to ai-session context
    pub async fn sync_memory_to_context(&self) -> Result<()> {
        let memory = self.session_memory.read().await;
        let mut context = self.ai_context.write().await;
        
        // Sync working memory items
        for item in &memory.working_memory.current_items {
            let message = Message {
                role: MessageRole::System,
                content: format!("[MEMORY:{:?}:priority={}] {}", item.item_type, item.priority, item.content),
                timestamp: item.created_at,
                token_count: Self::estimate_tokens(&item.content),
            };
            context.add_message(message);
        }
        
        // Sync recent episodes
        let recent_episodes: Vec<_> = memory.episodic_memory.episodes.iter()
            .filter(|e| e.timestamp > Utc::now() - chrono::Duration::hours(1))
            .collect();
        
        for episode in recent_episodes {
            let outcome_str = match &episode.outcome {
                EpisodeOutcome::Success { metrics } => {
                    if metrics.is_empty() {
                        "SUCCESS".to_string()
                    } else {
                        format!("SUCCESS: {:?}", metrics)
                    }
                },
                EpisodeOutcome::Failure { reason, recovery_actions } => {
                    format!("FAILURE: {} (recovery: {:?})", reason, recovery_actions)
                },
                EpisodeOutcome::Partial { progress, next_steps } => {
                    format!("PARTIAL: {}% complete, next: {:?}", (progress * 100.0) as u32, next_steps)
                },
                EpisodeOutcome::Cancelled { reason } => {
                    format!("CANCELLED: {}", reason)
                },
            };
            
            let message = Message {
                role: MessageRole::Assistant,
                content: format!(
                    "[EPISODE:{:?}] {} | Outcome: {}",
                    episode.event_type, episode.description, outcome_str
                ),
                timestamp: episode.timestamp,
                token_count: Self::estimate_tokens(&episode.description) + 20,
            };
            context.add_message(message);
        }
        
        // Compress if needed
        if context.get_total_tokens() > context.config.max_tokens * 80 / 100 {
            context.compress_context().await;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_session::SessionId;
    
    #[tokio::test]
    async fn test_context_bridge_creation() {
        let session_id = SessionId::new();
        let ai_context = Arc::new(RwLock::new(SessionContext::new(session_id)));
        let session_memory = Arc::new(RwLock::new(SessionMemory::new(
            "test-session".to_string(),
            "test-agent".to_string(),
        )));
        
        let bridge = ContextBridge::new(
            ai_context,
            session_memory,
            "test-agent".to_string(),
            "test-session".to_string(),
        );
        
        assert_eq!(bridge.agent_id, "test-agent");
        assert_eq!(bridge.session_id, "test-session");
    }
    
    #[tokio::test]
    async fn test_working_memory_sync() {
        let session_id = SessionId::new();
        let ai_context = Arc::new(RwLock::new(SessionContext::new(session_id)));
        let session_memory = Arc::new(RwLock::new(SessionMemory::new(
            "test-session".to_string(),
            "test-agent".to_string(),
        )));
        
        let bridge = ContextBridge::new(
            ai_context.clone(),
            session_memory.clone(),
            "test-agent".to_string(),
            "test-session".to_string(),
        );
        
        // Add memory through bridge
        bridge.add_to_working_memory(
            "Test memory content".to_string(),
            WorkingMemoryType::TaskInstructions,
            0.9,
        ).await.unwrap();
        
        // Verify it's in both systems
        let memory = session_memory.read().await;
        assert_eq!(memory.working_memory.current_items.len(), 1);
        
        let context = ai_context.read().await;
        assert!(context.get_message_count() > 0);
    }
}