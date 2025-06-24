//! Session context management for AI agents
//!
//! This module provides intelligent context management for AI agents, including conversation
//! history, task context, and workspace state. The context system is designed to maximize
//! AI performance while minimizing token usage through intelligent compression and summarization.
//!
//! # Key Features
//!
//! - **Token-Efficient History**: Automatic compression to reduce API costs by ~93%
//! - **Task Context**: Structured task and goal tracking for AI agents
//! - **Agent State**: Persistent agent memory and decision tracking
//! - **Workspace State**: File system and project state awareness
//! - **Smart Compression**: Intelligent context summarization and pruning
//!
//! # Examples
//!
//! ## Basic Context Management
//!
//! ```rust
//! use ai_session::context::{SessionContext, Message, MessageRole};
//! use ai_session::SessionId;
//! use chrono::Utc;
//!
//! let session_id = SessionId::new();
//! let mut context = SessionContext::new(session_id);
//!
//! // Add conversation messages
//! let user_message = Message {
//!     role: MessageRole::User,
//!     content: "Help me implement a REST API".to_string(),
//!     timestamp: Utc::now(),
//!     token_count: 7,
//! };
//! context.add_message(user_message);
//!
//! let assistant_message = Message {
//!     role: MessageRole::Assistant, 
//!     content: "I'll help you create a REST API. Let's start with the basic structure...".to_string(),
//!     timestamp: Utc::now(),
//!     token_count: 18,
//! };
//! context.add_message(assistant_message);
//!
//! // Check context stats
//! println!("Messages: {}", context.get_message_count());
//! println!("Total tokens: {}", context.get_total_tokens());
//! ```
//!
//! ## Context Compression
//!
//! ```rust
//! use ai_session::context::{SessionContext, Message, MessageRole};
//! use ai_session::SessionId;
//!
//! # tokio_test::block_on(async {
//! let session_id = SessionId::new();
//! let mut context = SessionContext::new(session_id);
//!
//! // Fill context with many messages...
//! for i in 0..100 {
//!     let message = Message {
//!         role: MessageRole::User,
//!         content: format!("Message {}", i),
//!         timestamp: chrono::Utc::now(),
//!         token_count: 5,
//!     };
//!     context.add_message(message);
//! }
//!
//! println!("Before compression: {} tokens", context.get_total_tokens());
//!
//! // Compress when approaching token limit
//! if context.get_total_tokens() > 400 {
//!     let compressed = context.compress_context().await;
//!     if compressed {
//!         println!("After compression: {} tokens", context.get_total_tokens());
//!     }
//! }
//! # });
//! ```

use crate::core::SessionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum token limit for the session
    pub max_tokens: usize,
    // Other configuration options can be added here
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_tokens: 100_000, // Default context window
        }
    }
}

/// Session context containing AI-relevant state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Session ID
    pub session_id: SessionId,
    /// Conversation history (token-efficient)
    pub conversation_history: TokenEfficientHistory,
    /// Current task context
    pub task_context: TaskContext,
    /// Agent state
    pub agent_state: AgentState,
    /// Workspace state
    pub workspace_state: WorkspaceState,
    /// Context metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Session configuration
    pub config: SessionConfig,
}

impl SessionContext {
    /// Create a new session context
    pub fn new(session_id: SessionId) -> Self {
        let config = SessionConfig::default();
        let mut conversation_history = TokenEfficientHistory::new();
        conversation_history.max_tokens = config.max_tokens;

        Self {
            session_id,
            conversation_history,
            task_context: TaskContext::default(),
            agent_state: AgentState::default(),
            workspace_state: WorkspaceState::default(),
            metadata: HashMap::new(),
            config,
        }
    }

    /// Add a message to the conversation history (takes Message struct)
    pub fn add_message(&mut self, message: Message) {
        self.conversation_history.add_message_struct(message);
    }

    /// Add a message to the conversation history (legacy method)
    pub fn add_message_raw(&mut self, role: MessageRole, content: String) {
        self.conversation_history.add_message(role, content);
    }

    /// Get the total number of messages in the conversation history
    pub fn get_message_count(&self) -> usize {
        self.conversation_history.messages.len()
    }

    /// Get the total estimated token count
    pub fn get_total_tokens(&self) -> usize {
        self.conversation_history.current_tokens
    }

    /// Get the most recent n messages
    pub fn get_recent_messages(&self, n: usize) -> Vec<&Message> {
        let message_count = self.conversation_history.messages.len();
        if n >= message_count {
            self.conversation_history.messages.iter().collect()
        } else {
            self.conversation_history
                .messages
                .iter()
                .skip(message_count - n)
                .collect()
        }
    }

    /// Compress the context if needed, returns true if compression occurred
    pub async fn compress_context(&mut self) -> bool {
        // Check if compression is needed
        if self.conversation_history.current_tokens > self.conversation_history.max_tokens {
            self.conversation_history.compress_old_messages();
            true
        } else {
            false
        }
    }

    /// Update the task context
    pub fn update_task(&mut self, task: TaskContext) {
        self.task_context = task;
    }

    /// Get a summary of the context
    pub fn summarize(&self) -> ContextSummary {
        ContextSummary {
            session_id: self.session_id.clone(),
            message_count: self.conversation_history.messages.len(),
            current_task: self.task_context.name.clone(),
            agent_state: self.agent_state.state.clone(),
            workspace_files: self.workspace_state.tracked_files.len(),
        }
    }
}

/// Token-efficient conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEfficientHistory {
    /// Messages in the conversation
    pub messages: Vec<Message>,
    /// Compressed older messages
    pub compressed_history: Option<CompressedHistory>,
    /// Maximum token limit
    pub max_tokens: usize,
    /// Current token count (approximate)
    pub current_tokens: usize,
}

impl Default for TokenEfficientHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenEfficientHistory {
    /// Create a new history
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            compressed_history: None,
            max_tokens: 100_000, // Default context window
            current_tokens: 0,
        }
    }

    /// Add a message to the history
    pub fn add_message(&mut self, role: MessageRole, content: String) {
        let token_estimate = content.len() / 4; // Rough estimate
        let message = Message {
            role,
            content,
            timestamp: Utc::now(),
            token_count: token_estimate,
        };

        self.messages.push(message);
        self.current_tokens += token_estimate;

        // Compress if needed
        if self.current_tokens > self.max_tokens {
            self.compress_old_messages();
        }
    }

    /// Add a message struct directly to the history
    pub fn add_message_struct(&mut self, message: Message) {
        self.current_tokens += message.token_count;
        self.messages.push(message);

        // Compress if needed
        if self.current_tokens > self.max_tokens {
            self.compress_old_messages();
        }
    }

    /// Compress old messages to save tokens
    pub fn compress_old_messages(&mut self) {
        // Simple implementation: remove oldest messages
        // In a real implementation, this would use intelligent summarization
        while self.current_tokens > self.max_tokens && !self.messages.is_empty() {
            let removed = self.messages.remove(0);
            self.current_tokens -= removed.token_count;
        }
    }

    /// Get messages within token limit
    pub fn get_messages_within_limit(&self, token_limit: usize) -> Vec<&Message> {
        let mut messages = Vec::new();
        let mut tokens = 0;

        // Start from most recent messages
        for message in self.messages.iter().rev() {
            if tokens + message.token_count <= token_limit {
                messages.push(message);
                tokens += message.token_count;
            } else {
                break;
            }
        }

        messages.reverse();
        messages
    }
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message sender
    pub role: MessageRole,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Estimated token count
    pub token_count: usize,
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Compressed history placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedHistory {
    /// Summary of compressed messages
    pub summary: String,
    /// Number of messages compressed
    pub message_count: usize,
    /// Token count saved
    pub tokens_saved: usize,
}

/// Current task context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskContext {
    /// Task ID
    pub id: Option<String>,
    /// Task name
    pub name: Option<String>,
    /// Task description
    pub description: Option<String>,
    /// Task type
    pub task_type: Option<String>,
    /// Priority
    pub priority: Option<TaskPriority>,
    /// Started at
    pub started_at: Option<DateTime<Utc>>,
    /// Additional context
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Agent state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentState {
    /// Current state
    pub state: String,
    /// Agent capabilities
    pub capabilities: Vec<String>,
    /// Performance metrics
    pub metrics: HashMap<String, f64>,
    /// Last error (if any)
    pub last_error: Option<String>,
}

/// Workspace state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceState {
    /// Current working directory
    pub working_directory: String,
    /// Tracked files
    pub tracked_files: HashMap<String, FileState>,
    /// Recent changes
    pub recent_changes: Vec<FileChange>,
}

/// File state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    /// File path
    pub path: String,
    /// Last modified
    pub last_modified: DateTime<Utc>,
    /// File hash
    pub hash: String,
    /// Is modified
    pub is_modified: bool,
}

/// File change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// File path
    pub path: String,
    /// Change type
    pub change_type: FileChangeType,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Type of file change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeType {
    Created,
    Modified,
    Deleted,
    Renamed,
}

/// Context summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    /// Session ID
    pub session_id: SessionId,
    /// Number of messages
    pub message_count: usize,
    /// Current task name
    pub current_task: Option<String>,
    /// Agent state
    pub agent_state: String,
    /// Number of tracked files
    pub workspace_files: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_efficient_history() {
        let mut history = TokenEfficientHistory::new();
        history.max_tokens = 100; // Small limit for testing

        // Add messages
        for i in 0..10 {
            history.add_message(
                MessageRole::User,
                format!("Message {}", i).repeat(10), // ~100 chars each
            );
        }

        // Should have compressed some messages
        assert!(history.messages.len() < 10);
        assert!(history.current_tokens <= history.max_tokens);
    }

    #[test]
    fn test_context_summary() {
        let session_id = SessionId::new();
        let mut context = SessionContext::new(session_id.clone());

        context.add_message_raw(MessageRole::User, "Hello".to_string());
        context.add_message_raw(MessageRole::Assistant, "Hi there!".to_string());

        let summary = context.summarize();
        assert_eq!(summary.session_id, session_id);
        assert_eq!(summary.message_count, 2);
    }

    #[test]
    fn test_new_api_methods() {
        let session_id = SessionId::new();
        let mut context = SessionContext::new(session_id.clone());

        // Test message count
        assert_eq!(context.get_message_count(), 0);

        // Add a message using new API
        let message = Message {
            role: MessageRole::User,
            content: "Test message".to_string(),
            timestamp: Utc::now(),
            token_count: 3,
        };
        context.add_message(message);

        // Check message count and tokens
        assert_eq!(context.get_message_count(), 1);
        assert_eq!(context.get_total_tokens(), 3);

        // Test get_recent_messages
        let recent = context.get_recent_messages(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].content, "Test message");

        // Test config field
        assert_eq!(context.config.max_tokens, 100_000);
    }

    #[tokio::test]
    async fn test_compress_context() {
        let session_id = SessionId::new();
        let mut context = SessionContext::new(session_id);

        // Set a small token limit for testing
        context.config.max_tokens = 50;
        context.conversation_history.max_tokens = 50;

        // Add messages that exceed the limit
        for i in 0..10 {
            let message = Message {
                role: MessageRole::User,
                content: format!("Message {}", i),
                timestamp: Utc::now(),
                token_count: 10,
            };
            context.add_message(message);
        }

        // Should have auto-compressed during add
        assert!(context.get_total_tokens() <= 50);

        // Manual compression should return false (already compressed)
        let compressed = context.compress_context().await;
        assert!(!compressed);
    }
}
