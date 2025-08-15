/// Common utilities for session management
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Unified message handler for all session operations
pub trait SessionMessageHandler: Send + Sync {
    async fn handle(&self, message: SessionMessage) -> Result<SessionResponse>;
}

/// Common session message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionMessage {
    Coordination(CoordinationMessage),
    Efficiency(EfficiencyMessage),
    Task(TaskMessage),
    Control(ControlMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationMessage {
    pub sender: String,
    pub content: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMessage {
    pub tokens_saved: usize,
    pub compression_ratio: f64,
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMessage {
    pub task_id: String,
    pub command: String,
    pub agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMessage {
    pub command: ControlCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlCommand {
    Start,
    Stop,
    Restart,
    Status,
}

/// Common response type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    pub success: bool,
    pub data: serde_json::Value,
    pub message: Option<String>,
}

/// Base session handler implementation
pub struct BaseSessionHandler<T> {
    pub data: Arc<RwLock<T>>,
    pub sender: mpsc::Sender<SessionMessage>,
    pub receiver: Arc<RwLock<mpsc::Receiver<SessionMessage>>>,
}

impl<T: Send + Sync + 'static> BaseSessionHandler<T> {
    pub fn new(data: T) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            data: Arc::new(RwLock::new(data)),
            sender: tx,
            receiver: Arc::new(RwLock::new(rx)),
        }
    }
    
    pub async fn send_message(&self, message: SessionMessage) -> Result<()> {
        self.sender.send(message).await?;
        Ok(())
    }
    
    pub async fn process_messages<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(SessionMessage) -> Result<SessionResponse> + Send,
    {
        let mut receiver = self.receiver.write().await;
        while let Some(msg) = receiver.recv().await {
            let _ = handler(msg)?;
        }
        Ok(())
    }
}

/// Statistics collector for all session types
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct SessionStatistics {
    pub total_messages: usize,
    pub tokens_saved: usize,
    pub compression_ratio: f64,
    pub tasks_executed: usize,
    pub errors: usize,
    pub uptime_seconds: u64,
}

impl SessionStatistics {
    pub fn update(&mut self, update_type: StatUpdate) {
        match update_type {
            StatUpdate::Message => self.total_messages += 1,
            StatUpdate::TokensSaved(n) => self.tokens_saved += n,
            StatUpdate::Compression(ratio) => self.compression_ratio = ratio,
            StatUpdate::TaskExecuted => self.tasks_executed += 1,
            StatUpdate::Error => self.errors += 1,
            StatUpdate::Uptime(secs) => self.uptime_seconds = secs,
        }
    }
    
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|_| serde_json::json!({}))
    }
}

#[derive(Debug, Clone)]
pub enum StatUpdate {
    Message,
    TokensSaved(usize),
    Compression(f64),
    TaskExecuted,
    Error,
    Uptime(u64),
}

/// Efficiency calculator
pub struct EfficiencyCalculator;

impl EfficiencyCalculator {
    pub fn calculate_token_savings(original: usize, compressed: usize) -> f64 {
        if original == 0 {
            return 0.0;
        }
        ((original - compressed) as f64 / original as f64) * 100.0
    }
    
    pub fn calculate_compression_ratio(original: usize, compressed: usize) -> f64 {
        if compressed == 0 {
            return 0.0;
        }
        original as f64 / compressed as f64
    }
    
    pub fn estimate_time_saved(tokens_saved: usize) -> f64 {
        // Estimate: 1000 tokens = 1 second of processing
        tokens_saved as f64 / 1000.0
    }
}