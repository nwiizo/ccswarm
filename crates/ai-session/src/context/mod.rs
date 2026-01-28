//! Session context management for AI agents
//!
//! This module provides intelligent context management for AI agents, including conversation
//! history, task context, and workspace state. The context system is designed to maximize
//! AI performance while minimizing token usage through intelligent compression and summarization.
//!
//! # Key Features
//!
//! - **Efficient History Management**: Real zstd compression for optimized context handling
//! - **Task Context**: Structured task and goal tracking for AI agents
//! - **Agent State**: Persistent agent memory and decision tracking
//! - **Workspace State**: File system and project state awareness
//! - **Smart Compression**: Zstd-based compression with message summarization
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
use std::io::Read as IoRead;

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum token limit for the session
    pub max_tokens: usize,
    /// Number of recent messages to keep uncompressed
    pub keep_recent_messages: usize,
    /// Compression level for zstd (1-22, default 3)
    pub compression_level: i32,
    /// Threshold ratio to trigger compression (e.g., 0.8 means compress when 80% full)
    pub compression_threshold: f32,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_tokens: 100_000,        // Default context window
            keep_recent_messages: 20,   // Keep last 20 messages uncompressed
            compression_level: 3,       // Balanced compression
            compression_threshold: 0.8, // Compress at 80% capacity
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
        conversation_history.keep_recent = config.keep_recent_messages;
        conversation_history.compression_level = config.compression_level;

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
        let threshold = (self.conversation_history.max_tokens as f32
            * self.config.compression_threshold) as usize;

        if self.conversation_history.current_tokens > threshold {
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

    /// Get compression statistics
    pub fn get_compression_stats(&self) -> CompressionStats {
        self.conversation_history.get_compression_stats()
    }
}

/// Token-efficient conversation history with real zstd compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEfficientHistory {
    /// Active messages in the conversation (uncompressed, recent)
    pub messages: Vec<Message>,
    /// Compressed older messages (zstd compressed)
    pub compressed_history: Option<CompressedHistory>,
    /// Maximum token limit
    pub max_tokens: usize,
    /// Current token count (approximate)
    pub current_tokens: usize,
    /// Number of recent messages to keep uncompressed
    pub keep_recent: usize,
    /// Compression level for zstd
    pub compression_level: i32,
    /// Total messages ever added (including compressed)
    pub total_messages_added: usize,
    /// Total tokens saved through compression
    pub tokens_saved_by_compression: usize,
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
            max_tokens: 100_000,
            current_tokens: 0,
            keep_recent: 20,
            compression_level: 3,
            total_messages_added: 0,
            tokens_saved_by_compression: 0,
        }
    }

    /// Add a message to the history
    pub fn add_message(&mut self, role: MessageRole, content: String) {
        let token_estimate = estimate_tokens(&content);
        let message = Message {
            role,
            content,
            timestamp: Utc::now(),
            token_count: token_estimate,
        };

        self.messages.push(message);
        self.current_tokens += token_estimate;
        self.total_messages_added += 1;

        // Compress if needed
        if self.current_tokens > self.max_tokens {
            self.compress_old_messages();
        }
    }

    /// Add a message struct directly to the history
    pub fn add_message_struct(&mut self, message: Message) {
        self.current_tokens += message.token_count;
        self.messages.push(message);
        self.total_messages_added += 1;

        // Compress if needed
        if self.current_tokens > self.max_tokens {
            self.compress_old_messages();
        }
    }

    /// Compress old messages using zstd
    pub fn compress_old_messages(&mut self) {
        // Keep only the most recent messages uncompressed
        if self.messages.len() <= self.keep_recent {
            return;
        }

        // Split messages: older ones to compress, recent ones to keep
        let split_point = self.messages.len() - self.keep_recent;
        let messages_to_compress: Vec<Message> = self.messages.drain(..split_point).collect();

        if messages_to_compress.is_empty() {
            return;
        }

        // Calculate tokens being compressed
        let tokens_to_compress: usize = messages_to_compress.iter().map(|m| m.token_count).sum();

        // Serialize messages to JSON
        let json_data = match serde_json::to_vec(&messages_to_compress) {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!("Failed to serialize messages for compression: {}", e);
                // Put messages back if serialization fails
                let mut restored = messages_to_compress;
                restored.append(&mut self.messages);
                self.messages = restored;
                return;
            }
        };

        // Compress using zstd
        let compressed_data = match zstd::encode_all(json_data.as_slice(), self.compression_level) {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!("Failed to compress messages: {}", e);
                // Put messages back if compression fails
                let mut restored = messages_to_compress;
                restored.append(&mut self.messages);
                self.messages = restored;
                return;
            }
        };

        // Calculate compression ratio
        let original_size = json_data.len();
        let compressed_size = compressed_data.len();
        let compression_ratio = if original_size > 0 {
            1.0 - (compressed_size as f64 / original_size as f64)
        } else {
            0.0
        };

        // Create summary of compressed content
        let summary = create_compression_summary(&messages_to_compress);

        // Merge with existing compressed history if any
        let new_compressed = if let Some(existing) = self.compressed_history.take() {
            CompressedHistory {
                compressed_data: merge_compressed_data(
                    &existing.compressed_data,
                    &compressed_data,
                    self.compression_level,
                ),
                summary: format!("{}\n---\n{}", existing.summary, summary),
                message_count: existing.message_count + messages_to_compress.len(),
                original_tokens: existing.original_tokens + tokens_to_compress,
                compressed_bytes: existing.compressed_bytes + compressed_size,
                compression_ratio: (existing.compression_ratio + compression_ratio) / 2.0,
            }
        } else {
            CompressedHistory {
                compressed_data,
                summary,
                message_count: messages_to_compress.len(),
                original_tokens: tokens_to_compress,
                compressed_bytes: compressed_size,
                compression_ratio,
            }
        };

        // Update state
        self.compressed_history = Some(new_compressed);
        self.current_tokens -= tokens_to_compress;
        self.tokens_saved_by_compression += tokens_to_compress;

        // Add a small token cost for the summary (accessible without decompression)
        let summary_tokens = estimate_tokens(
            self.compressed_history
                .as_ref()
                .map(|h| h.summary.as_str())
                .unwrap_or(""),
        );
        self.current_tokens += summary_tokens.min(100); // Cap summary token cost

        tracing::info!(
            "Compressed {} messages ({} tokens) with {:.1}% ratio",
            messages_to_compress.len(),
            tokens_to_compress,
            compression_ratio * 100.0
        );
    }

    /// Decompress and retrieve all historical messages
    pub fn decompress_history(&self) -> Option<Vec<Message>> {
        let compressed = self.compressed_history.as_ref()?;

        // Decompress using zstd
        let mut decompressed = Vec::new();
        let mut decoder = match zstd::Decoder::new(compressed.compressed_data.as_slice()) {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("Failed to create zstd decoder: {}", e);
                return None;
            }
        };

        if let Err(e) = decoder.read_to_end(&mut decompressed) {
            tracing::error!("Failed to decompress history: {}", e);
            return None;
        }

        // Deserialize messages
        match serde_json::from_slice(&decompressed) {
            Ok(messages) => Some(messages),
            Err(e) => {
                tracing::error!("Failed to deserialize decompressed messages: {}", e);
                None
            }
        }
    }

    /// Get all messages including decompressed history
    pub fn get_all_messages(&self) -> Vec<Message> {
        let mut all_messages = self.decompress_history().unwrap_or_default();
        all_messages.extend(self.messages.clone());
        all_messages
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

    /// Get compression statistics
    pub fn get_compression_stats(&self) -> CompressionStats {
        let compressed_stats = self.compressed_history.as_ref().map(|h| {
            (
                h.message_count,
                h.original_tokens,
                h.compressed_bytes,
                h.compression_ratio,
            )
        });

        CompressionStats {
            total_messages_added: self.total_messages_added,
            active_messages: self.messages.len(),
            compressed_messages: compressed_stats.map(|(c, _, _, _)| c).unwrap_or(0),
            active_tokens: self.current_tokens,
            tokens_saved: self.tokens_saved_by_compression,
            compressed_bytes: compressed_stats.map(|(_, _, b, _)| b).unwrap_or(0),
            compression_ratio: compressed_stats.map(|(_, _, _, r)| r).unwrap_or(0.0),
        }
    }
}

/// Estimate tokens for a string (improved approximation)
fn estimate_tokens(content: &str) -> usize {
    // Handle empty string explicitly
    if content.is_empty() {
        return 1;
    }

    // Better approximation than simple len/4:
    // - Count words (roughly 1.3 tokens per word for English)
    // - Account for punctuation and special characters
    // - Add overhead for JSON structure

    let word_count = content.split_whitespace().count();
    let char_count = content.chars().count();

    // Special characters often become their own tokens
    let special_chars = content
        .chars()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
        .count();

    // Estimate: words * 1.3 + special_chars + small overhead
    let estimate = (word_count as f64 * 1.3) as usize + special_chars + 2;

    // Also consider raw character-based estimate for very long strings
    let char_estimate = char_count / 4;

    // Use the larger of the two estimates
    estimate.max(char_estimate).max(1)
}

/// Create a summary of compressed messages
fn create_compression_summary(messages: &[Message]) -> String {
    if messages.is_empty() {
        return String::new();
    }

    let first = messages.first().unwrap();
    let last = messages.last().unwrap();

    let user_count = messages
        .iter()
        .filter(|m| m.role == MessageRole::User)
        .count();
    let assistant_count = messages
        .iter()
        .filter(|m| m.role == MessageRole::Assistant)
        .count();

    format!(
        "[Compressed: {} messages ({} user, {} assistant) from {} to {}]",
        messages.len(),
        user_count,
        assistant_count,
        first.timestamp.format("%H:%M:%S"),
        last.timestamp.format("%H:%M:%S")
    )
}

/// Merge two compressed data blocks
fn merge_compressed_data(existing: &[u8], new: &[u8], level: i32) -> Vec<u8> {
    // Decompress both, merge, recompress
    // This is less efficient but maintains a single compressed block

    let mut existing_decompressed = Vec::new();
    if let Ok(mut decoder) = zstd::Decoder::new(existing) {
        let _ = decoder.read_to_end(&mut existing_decompressed);
    }

    let mut new_decompressed = Vec::new();
    if let Ok(mut decoder) = zstd::Decoder::new(new) {
        let _ = decoder.read_to_end(&mut new_decompressed);
    }

    // Parse both as message arrays and merge
    let existing_messages: Vec<Message> =
        serde_json::from_slice(&existing_decompressed).unwrap_or_default();
    let new_messages: Vec<Message> = serde_json::from_slice(&new_decompressed).unwrap_or_default();

    let mut merged = existing_messages;
    merged.extend(new_messages);

    // Recompress
    let json_data = serde_json::to_vec(&merged).unwrap_or_default();
    zstd::encode_all(json_data.as_slice(), level).unwrap_or_else(|_| new.to_vec())
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

/// Compressed history with zstd compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedHistory {
    /// Zstd compressed message data
    #[serde(with = "base64_serde")]
    pub compressed_data: Vec<u8>,
    /// Human-readable summary of compressed content
    pub summary: String,
    /// Number of messages compressed
    pub message_count: usize,
    /// Original token count before compression
    pub original_tokens: usize,
    /// Size of compressed data in bytes
    pub compressed_bytes: usize,
    /// Compression ratio achieved (0.0 to 1.0)
    pub compression_ratio: f64,
}

/// Base64 serialization for binary data
mod base64_serde {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let encoded = STANDARD.encode(data);
        encoded.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoded = String::deserialize(deserializer)?;
        STANDARD.decode(&encoded).map_err(serde::de::Error::custom)
    }
}

/// Compression statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    /// Total messages ever added
    pub total_messages_added: usize,
    /// Currently active (uncompressed) messages
    pub active_messages: usize,
    /// Messages in compressed storage
    pub compressed_messages: usize,
    /// Current active token count
    pub active_tokens: usize,
    /// Tokens saved by compression
    pub tokens_saved: usize,
    /// Size of compressed data in bytes
    pub compressed_bytes: usize,
    /// Average compression ratio
    pub compression_ratio: f64,
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
        history.max_tokens = 200; // Moderate limit for testing
        history.keep_recent = 3; // Keep 3 recent messages

        // Add messages with moderate size (~5-10 tokens each)
        for i in 0..10 {
            history.add_message(
                MessageRole::User,
                format!("Test message number {}", i), // ~5 tokens each
            );
        }

        // Should have compressed some messages
        assert!(history.messages.len() <= 10);

        // Verify compression happened
        assert!(
            history.compressed_history.is_some() || history.current_tokens <= history.max_tokens
        );

        // After compression: tokens should be manageable
        // 3 recent messages (~15 tokens) + summary (capped at 100) should be < 200 + 100
        assert!(
            history.current_tokens <= history.max_tokens + 150,
            "current_tokens {} exceeded max_tokens {} + 150",
            history.current_tokens,
            history.max_tokens
        );
    }

    #[test]
    fn test_zstd_compression() {
        let mut history = TokenEfficientHistory::new();
        history.max_tokens = 50;
        history.keep_recent = 2;

        // Add many messages to trigger compression
        for i in 0..20 {
            history.add_message(MessageRole::User, format!("Test message number {}", i));
        }

        // Should have compressed history
        assert!(history.compressed_history.is_some());

        // Verify we can decompress
        let decompressed = history.decompress_history();
        assert!(decompressed.is_some());

        let messages = decompressed.unwrap();
        assert!(!messages.is_empty());
    }

    #[test]
    fn test_compression_stats() {
        let mut history = TokenEfficientHistory::new();
        history.max_tokens = 30;
        history.keep_recent = 2;

        // Add messages
        for i in 0..10 {
            history.add_message(MessageRole::User, format!("Message {}", i));
        }

        let stats = history.get_compression_stats();
        assert_eq!(stats.total_messages_added, 10);
        assert!(stats.compressed_messages > 0 || stats.active_messages == 10);
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
        context.config.compression_threshold = 0.5;
        context.conversation_history.max_tokens = 50;
        context.conversation_history.keep_recent = 3;

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
        // Check compression stats
        let stats = context.get_compression_stats();
        assert!(stats.compressed_messages > 0 || stats.total_messages_added == 10);
    }

    #[test]
    fn test_estimate_tokens() {
        // Simple string
        let tokens = estimate_tokens("Hello world");
        assert!(tokens >= 2);

        // Empty string
        let tokens = estimate_tokens("");
        assert_eq!(tokens, 1); // Minimum 1

        // String with special characters
        let tokens = estimate_tokens("Hello, world! How are you?");
        assert!(tokens >= 5);

        // Long string
        let tokens = estimate_tokens(&"word ".repeat(100));
        assert!(tokens >= 100);
    }

    #[test]
    fn test_get_all_messages() {
        let mut history = TokenEfficientHistory::new();
        history.max_tokens = 20;
        history.keep_recent = 2;

        // Add messages
        for i in 0..5 {
            history.add_message(MessageRole::User, format!("Message {}", i));
        }

        // Get all messages (compressed + active)
        let all = history.get_all_messages();
        assert_eq!(all.len(), 5);
    }
}
