use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::ExecutionSessionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: ExecutionSessionId,
    messages: Vec<Message>,
    total_messages_added: usize,
    active_tokens: usize,
    metadata: HashMap<String, serde_json::Value>,
}

impl SessionContext {
    pub fn new(session_id: ExecutionSessionId) -> Self {
        Self {
            session_id,
            messages: Vec::new(),
            total_messages_added: 0,
            active_tokens: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn add_message_raw(&mut self, role: MessageRole, content: String) {
        let token_count = estimate_tokens(&content);
        self.messages.push(Message {
            role,
            content,
            timestamp: Utc::now(),
            token_count,
        });
        self.total_messages_added += 1;
        self.active_tokens = self.active_tokens.saturating_add(token_count);
        self.compact_if_needed();
    }

    pub async fn compress_context(&mut self) {
        self.compact_if_needed();
    }

    pub fn get_recent_messages(&self, n: usize) -> Vec<&Message> {
        let count = self.messages.len();
        if n >= count {
            self.messages.iter().collect()
        } else {
            self.messages.iter().skip(count - n).collect()
        }
    }

    pub fn get_compression_stats(&self) -> CompressionStats {
        CompressionStats {
            total_messages_added: self.total_messages_added,
            active_messages: self.messages.len(),
            compressed_messages: self
                .total_messages_added
                .saturating_sub(self.messages.len()),
            active_tokens: self.active_tokens,
            tokens_saved: 0,
            compressed_bytes: 0,
            compression_ratio: 0.0,
        }
    }

    fn compact_if_needed(&mut self) {
        const KEEP_RECENT: usize = 40;
        if self.messages.len() <= KEEP_RECENT {
            return;
        }
        let remove_count = self.messages.len() - KEEP_RECENT;
        let removed_tokens: usize = self
            .messages
            .drain(..remove_count)
            .map(|message| message.token_count)
            .sum();
        self.active_tokens = self.active_tokens.saturating_sub(removed_tokens);
    }
}

fn estimate_tokens(content: &str) -> usize {
    if content.is_empty() {
        return 1;
    }
    let word_count = content.split_whitespace().count();
    let char_count = content.chars().count();
    let special_chars = content
        .chars()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
        .count();
    ((word_count as f64 * 1.3) as usize + special_chars + 2)
        .max(char_count / 4)
        .max(1)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub token_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    pub total_messages_added: usize,
    pub active_messages: usize,
    pub compressed_messages: usize,
    pub active_tokens: usize,
    pub tokens_saved: usize,
    pub compressed_bytes: usize,
    pub compression_ratio: f64,
}
