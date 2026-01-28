//! Unified message bus - Consolidates multiple communication channels

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast, mpsc};
use uuid::Uuid;

/// Unified message type that encompasses all communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnifiedMessage {
    /// Session management message
    Session(SessionMessage),
    /// Agent coordination message
    Coordination(CoordinationMessage),
    /// Task management message
    Task(TaskMessage),
    /// System event message
    Event(EventMessage),
    /// Direct agent-to-agent message
    Direct(DirectMessage),
    /// IPC message
    Ipc(IpcMessage),
}

/// Session management messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: String,
    pub session_id: String,
    pub msg_type: SessionMessageType,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionMessageType {
    Created,
    Started,
    Stopped,
    Output,
    Input,
    StatusChange,
}

/// Agent coordination messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationMessage {
    pub id: String,
    pub from_agent: String,
    pub to_agent: Option<String>,
    pub msg_type: CoordinationMessageType,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationMessageType {
    TaskAssignment,
    TaskAccepted,
    TaskRejected,
    TaskCompleted,
    StatusUpdate,
    HelpRequest,
    KnowledgeShare,
}

/// Task management messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMessage {
    pub id: String,
    pub task_id: String,
    pub msg_type: TaskMessageType,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskMessageType {
    Created,
    Assigned,
    Started,
    Progress,
    Completed,
    Failed,
    Cancelled,
}

/// System event messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMessage {
    pub id: String,
    pub source: String,
    pub event_type: EventType,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    SystemStartup,
    SystemShutdown,
    AgentConnected,
    AgentDisconnected,
    Error,
    Warning,
    Info,
}

/// Direct agent-to-agent messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectMessage {
    pub id: String,
    pub from_agent: String,
    pub to_agent: String,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// IPC messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    pub id: String,
    pub msg_type: String,
    pub payload: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Unified message bus that handles all communication
pub struct UnifiedBus {
    /// Broadcast channel for all messages
    broadcast_tx: broadcast::Sender<UnifiedMessage>,
    /// Point-to-point channels for direct messages
    direct_channels: Arc<RwLock<HashMap<String, mpsc::Sender<UnifiedMessage>>>>,
    /// Topic subscribers
    topic_subscribers: Arc<RwLock<HashMap<String, Vec<mpsc::Sender<UnifiedMessage>>>>>,
    /// Message history
    message_history: Arc<RwLock<Vec<UnifiedMessage>>>,
    /// History size limit
    history_limit: usize,
}

impl UnifiedBus {
    /// Create new unified bus
    pub fn new(history_limit: usize) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1024);

        Self {
            broadcast_tx,
            direct_channels: Arc::new(RwLock::new(HashMap::new())),
            topic_subscribers: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(Vec::new())),
            history_limit,
        }
    }

    /// Subscribe to all messages
    pub fn subscribe_all(&self) -> broadcast::Receiver<UnifiedMessage> {
        self.broadcast_tx.subscribe()
    }

    /// Subscribe to specific topic
    pub async fn subscribe_topic(&self, topic: &str) -> mpsc::Receiver<UnifiedMessage> {
        let (tx, rx) = mpsc::channel(256);
        let mut subscribers = self.topic_subscribers.write().await;
        subscribers.entry(topic.to_string()).or_default().push(tx);
        rx
    }

    /// Register direct channel for agent
    pub async fn register_agent(&self, agent_id: &str) -> mpsc::Receiver<UnifiedMessage> {
        let (tx, rx) = mpsc::channel(256);
        let mut channels = self.direct_channels.write().await;
        channels.insert(agent_id.to_string(), tx);
        rx
    }

    /// Unregister agent
    pub async fn unregister_agent(&self, agent_id: &str) {
        let mut channels = self.direct_channels.write().await;
        channels.remove(agent_id);
    }

    /// Send message to bus
    pub async fn send(&self, message: UnifiedMessage) -> Result<()> {
        // Add to history
        {
            let mut history = self.message_history.write().await;
            history.push(message.clone());

            // Trim history if needed
            if history.len() > self.history_limit {
                let drain_end = history.len() - self.history_limit;
                history.drain(0..drain_end);
            }
        }

        // Broadcast to all subscribers
        let _ = self.broadcast_tx.send(message.clone());

        // Send to topic subscribers
        if let Some(topic) = self.get_message_topic(&message) {
            let subscribers = self.topic_subscribers.read().await;
            if let Some(subs) = subscribers.get(&topic) {
                for sub in subs {
                    let _ = sub.send(message.clone()).await;
                }
            }
        }

        // Send direct messages
        if let UnifiedMessage::Direct(ref msg) = message {
            let channels = self.direct_channels.read().await;
            if let Some(tx) = channels.get(&msg.to_agent) {
                let _ = tx.send(message).await;
            }
        }

        Ok(())
    }

    /// Get message topic for routing
    fn get_message_topic(&self, message: &UnifiedMessage) -> Option<String> {
        match message {
            UnifiedMessage::Session(_) => Some("session".to_string()),
            UnifiedMessage::Coordination(_) => Some("coordination".to_string()),
            UnifiedMessage::Task(_) => Some("task".to_string()),
            UnifiedMessage::Event(_) => Some("event".to_string()),
            UnifiedMessage::Direct(_) => None, // Direct messages don't use topics
            UnifiedMessage::Ipc(_) => Some("ipc".to_string()),
        }
    }

    /// Get message history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<UnifiedMessage> {
        let history = self.message_history.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.clone(),
        }
    }

    /// Create session message
    pub fn create_session_message(
        session_id: &str,
        msg_type: SessionMessageType,
        payload: serde_json::Value,
    ) -> UnifiedMessage {
        UnifiedMessage::Session(SessionMessage {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            msg_type,
            payload,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Create coordination message
    pub fn create_coordination_message(
        from_agent: &str,
        to_agent: Option<&str>,
        msg_type: CoordinationMessageType,
        payload: serde_json::Value,
    ) -> UnifiedMessage {
        UnifiedMessage::Coordination(CoordinationMessage {
            id: Uuid::new_v4().to_string(),
            from_agent: from_agent.to_string(),
            to_agent: to_agent.map(|s| s.to_string()),
            msg_type,
            payload,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Create task message
    pub fn create_task_message(
        task_id: &str,
        msg_type: TaskMessageType,
        payload: serde_json::Value,
    ) -> UnifiedMessage {
        UnifiedMessage::Task(TaskMessage {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            msg_type,
            payload,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Create event message
    pub fn create_event_message(
        source: &str,
        event_type: EventType,
        payload: serde_json::Value,
    ) -> UnifiedMessage {
        UnifiedMessage::Event(EventMessage {
            id: Uuid::new_v4().to_string(),
            source: source.to_string(),
            event_type,
            payload,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Create direct message
    pub fn create_direct_message(
        from_agent: &str,
        to_agent: &str,
        content: &str,
    ) -> UnifiedMessage {
        UnifiedMessage::Direct(DirectMessage {
            id: Uuid::new_v4().to_string(),
            from_agent: from_agent.to_string(),
            to_agent: to_agent.to_string(),
            content: content.to_string(),
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unified_bus() {
        let bus = UnifiedBus::new(100);

        // Subscribe to all messages
        let mut all_rx = bus.subscribe_all();

        // Subscribe to session topic
        let mut session_rx = bus.subscribe_topic("session").await;

        // Register an agent
        let mut agent_rx = bus.register_agent("agent1").await;

        // Send a session message
        let msg = UnifiedBus::create_session_message(
            "session1",
            SessionMessageType::Created,
            serde_json::json!({"status": "ok"}),
        );
        bus.send(msg.clone()).await.unwrap();

        // Check all subscriber received it
        let received = all_rx.recv().await.unwrap();
        match received {
            UnifiedMessage::Session(s) => assert_eq!(s.session_id, "session1"),
            _ => panic!("Wrong message type"),
        }

        // Check topic subscriber received it
        let received = session_rx.recv().await.unwrap();
        match received {
            UnifiedMessage::Session(s) => assert_eq!(s.session_id, "session1"),
            _ => panic!("Wrong message type"),
        }

        // Send direct message
        let direct_msg = UnifiedBus::create_direct_message("agent2", "agent1", "Hello agent1");
        bus.send(direct_msg).await.unwrap();

        // Check agent received it
        let received = agent_rx.recv().await.unwrap();
        match received {
            UnifiedMessage::Direct(d) => {
                assert_eq!(d.to_agent, "agent1");
                assert_eq!(d.content, "Hello agent1");
            }
            _ => panic!("Wrong message type"),
        }
    }
}
