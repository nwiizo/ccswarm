//! Multi-agent coordination functionality

use anyhow::Result;
use crossbeam_channel::{Receiver, Sender};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::core::AISession;

/// Agent identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct AgentId(Uuid);

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentId {
    /// Create a new agent ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Multi-agent session coordinator
pub struct MultiAgentSession {
    /// Active agent sessions
    pub agents: Arc<DashMap<AgentId, Arc<AISession>>>,
    /// Message bus for inter-agent communication
    pub message_bus: Arc<MessageBus>,
    /// Task distributor
    pub task_distributor: Arc<TaskDistributor>,
    /// Resource manager
    pub resource_manager: Arc<ResourceManager>,
}

impl Default for MultiAgentSession {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiAgentSession {
    /// Create a new multi-agent session
    pub fn new() -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
            message_bus: Arc::new(MessageBus::new()),
            task_distributor: Arc::new(TaskDistributor::new()),
            resource_manager: Arc::new(ResourceManager::new()),
        }
    }

    /// Register an agent
    pub fn register_agent(&self, agent_id: AgentId, session: Arc<AISession>) -> Result<()> {
        self.agents.insert(agent_id.clone(), session);
        self.message_bus.register_agent(agent_id)?;
        Ok(())
    }

    /// Unregister an agent
    pub fn unregister_agent(&self, agent_id: &AgentId) -> Result<()> {
        self.agents.remove(agent_id);
        self.message_bus.unregister_agent(agent_id)?;
        Ok(())
    }

    /// Get an agent session
    pub fn get_agent(&self, agent_id: &AgentId) -> Option<Arc<AISession>> {
        self.agents.get(agent_id).map(|entry| entry.clone())
    }

    /// List all agents
    pub fn list_agents(&self) -> Vec<AgentId> {
        self.agents
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Send a message to an agent
    pub async fn send_message(&self, from: AgentId, to: AgentId, message: Message) -> Result<()> {
        self.message_bus.send_message(from, to, message)
    }

    /// Broadcast a message to all agents
    pub async fn broadcast(&self, from: AgentId, message: BroadcastMessage) -> Result<()> {
        self.message_bus.broadcast(from, message)
    }
}

/// Message bus for inter-agent communication
pub struct MessageBus {
    /// Message channels for each agent
    channels: DashMap<AgentId, (Sender<Message>, Receiver<Message>)>,
    /// Broadcast channel
    broadcast_sender: Sender<BroadcastMessage>,
    _broadcast_receiver: Receiver<BroadcastMessage>,
    /// Agent message channels for ccswarm integration
    agent_channels: DashMap<AgentId, (Sender<AgentMessage>, Receiver<AgentMessage>)>,
    /// All messages channel for monitoring
    all_messages_sender: Sender<AgentMessage>,
    all_messages_receiver: Receiver<AgentMessage>,
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBus {
    /// Create a new message bus
    pub fn new() -> Self {
        let (broadcast_sender, broadcast_receiver) = crossbeam_channel::unbounded();
        let (all_messages_sender, all_messages_receiver) = crossbeam_channel::unbounded();
        Self {
            channels: DashMap::new(),
            broadcast_sender,
            _broadcast_receiver: broadcast_receiver,
            agent_channels: DashMap::new(),
            all_messages_sender,
            all_messages_receiver,
        }
    }

    /// Register an agent
    pub fn register_agent(&self, agent_id: AgentId) -> Result<()> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        self.channels.insert(agent_id.clone(), (sender, receiver));

        // Also register agent message channel
        let (agent_sender, agent_receiver) = crossbeam_channel::unbounded();
        self.agent_channels
            .insert(agent_id, (agent_sender, agent_receiver));
        Ok(())
    }

    /// Unregister an agent
    pub fn unregister_agent(&self, agent_id: &AgentId) -> Result<()> {
        self.channels.remove(agent_id);
        self.agent_channels.remove(agent_id);
        Ok(())
    }

    /// Send a message to a specific agent
    pub fn send_message(&self, _from: AgentId, to: AgentId, message: Message) -> Result<()> {
        if let Some(channel) = self.channels.get(&to) {
            channel.0.send(message)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Agent not found: {}", to))
        }
    }

    /// Broadcast a message to all agents
    pub fn broadcast(&self, _from: AgentId, message: BroadcastMessage) -> Result<()> {
        self.broadcast_sender.send(message)?;
        Ok(())
    }

    /// Get receiver for an agent
    pub fn get_receiver(&self, agent_id: &AgentId) -> Option<Receiver<Message>> {
        self.channels.get(agent_id).map(|entry| entry.1.clone())
    }

    /// Subscribe to all messages (for monitoring)
    pub fn subscribe_all(&self) -> Receiver<AgentMessage> {
        self.all_messages_receiver.clone()
    }

    /// Publish a message to a specific agent
    pub async fn publish_to_agent(&self, agent_id: &AgentId, message: AgentMessage) -> Result<()> {
        // Send to the specific agent
        if let Some(channel) = self.agent_channels.get(agent_id) {
            channel.0.send(message.clone())?;
        } else {
            return Err(anyhow::anyhow!("Agent not found: {}", agent_id));
        }

        // Also send to the all messages channel for monitoring
        self.all_messages_sender.send(message)?;

        Ok(())
    }

    /// Get agent message receiver for a specific agent
    pub fn get_agent_receiver(&self, agent_id: &AgentId) -> Option<Receiver<AgentMessage>> {
        self.agent_channels
            .get(agent_id)
            .map(|entry| entry.1.clone())
    }
}

/// Inter-agent message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID
    pub id: Uuid,
    /// Sender agent
    pub from: AgentId,
    /// Message type
    pub message_type: MessageType,
    /// Message payload
    pub payload: serde_json::Value,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Task assignment
    TaskAssignment,
    /// Status update
    StatusUpdate,
    /// Data sharing
    DataShare,
    /// Coordination request
    CoordinationRequest,
    /// Response
    Response,
    /// Custom message
    Custom(String),
}

/// Agent message for ccswarm integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    /// Agent registration
    Registration {
        agent_id: AgentId,
        capabilities: Vec<String>,
        metadata: serde_json::Value,
    },
    /// Task assignment to agent
    TaskAssignment {
        task_id: TaskId,
        agent_id: AgentId,
        task_data: serde_json::Value,
    },
    /// Task completion notification
    TaskCompleted {
        agent_id: AgentId,
        task_id: TaskId,
        result: serde_json::Value,
    },
    /// Task progress update
    TaskProgress {
        agent_id: AgentId,
        task_id: TaskId,
        progress: f32,
        message: String,
    },
    /// Help request from agent
    HelpRequest {
        agent_id: AgentId,
        context: String,
        priority: MessagePriority,
    },
    /// Status update from agent
    StatusUpdate {
        agent_id: AgentId,
        status: String,
        metrics: serde_json::Value,
    },
    /// Custom message type
    Custom {
        message_type: String,
        data: serde_json::Value,
    },
}

/// Broadcast message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastMessage {
    /// Message ID
    pub id: Uuid,
    /// Sender agent
    pub from: AgentId,
    /// Message content
    pub content: String,
    /// Priority
    pub priority: MessagePriority,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Message priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Task distributor for workload management
pub struct TaskDistributor {
    /// Task queue
    task_queue: Arc<RwLock<Vec<Task>>>,
    /// Agent capabilities
    agent_capabilities: Arc<DashMap<AgentId, Vec<String>>>,
    /// Task assignments
    assignments: Arc<DashMap<TaskId, AgentId>>,
}

impl Default for TaskDistributor {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskDistributor {
    /// Create a new task distributor
    pub fn new() -> Self {
        Self {
            task_queue: Arc::new(RwLock::new(Vec::new())),
            agent_capabilities: Arc::new(DashMap::new()),
            assignments: Arc::new(DashMap::new()),
        }
    }

    /// Register agent capabilities
    pub fn register_capabilities(&self, agent_id: AgentId, capabilities: Vec<String>) {
        self.agent_capabilities.insert(agent_id, capabilities);
    }

    /// Submit a task
    pub async fn submit_task(&self, task: Task) -> Result<()> {
        self.task_queue.write().await.push(task);
        Ok(())
    }

    /// Assign tasks to agents
    pub async fn distribute_tasks(&self) -> Result<Vec<(TaskId, AgentId)>> {
        let mut assignments = Vec::new();
        let mut queue = self.task_queue.write().await;

        // Simple round-robin distribution
        // In a real implementation, this would use sophisticated matching
        let agents: Vec<AgentId> = self
            .agent_capabilities
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        if agents.is_empty() {
            return Ok(assignments);
        }

        let mut agent_index = 0;
        while let Some(task) = queue.pop() {
            let agent_id = &agents[agent_index % agents.len()];
            self.assignments.insert(task.id.clone(), agent_id.clone());
            assignments.push((task.id, agent_id.clone()));
            agent_index += 1;
        }

        Ok(assignments)
    }
}

/// Task identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TaskId(Uuid);

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskId {
    /// Create a new task ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task ID
    pub id: TaskId,
    /// Task name
    pub name: String,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Task payload
    pub payload: serde_json::Value,
    /// Priority
    pub priority: TaskPriority,
    /// Created at
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Resource manager for preventing conflicts
pub struct ResourceManager {
    /// File locks
    file_locks: Arc<DashMap<String, AgentId>>,
    /// API rate limits
    rate_limits: Arc<DashMap<String, RateLimit>>,
    /// Shared memory pool
    shared_memory: Arc<DashMap<String, Vec<u8>>>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            file_locks: Arc::new(DashMap::new()),
            rate_limits: Arc::new(DashMap::new()),
            shared_memory: Arc::new(DashMap::new()),
        }
    }

    /// Acquire a file lock
    pub fn acquire_file_lock(&self, path: &str, agent_id: AgentId) -> Result<()> {
        match self.file_locks.entry(path.to_string()) {
            dashmap::mapref::entry::Entry::Occupied(_) => {
                Err(anyhow::anyhow!("File already locked: {}", path))
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                entry.insert(agent_id);
                Ok(())
            }
        }
    }

    /// Release a file lock
    pub fn release_file_lock(&self, path: &str, agent_id: &AgentId) -> Result<()> {
        if let Some((_, owner)) = self.file_locks.remove(path)
            && owner != *agent_id
        {
            return Err(anyhow::anyhow!("Not the lock owner"));
        }
        Ok(())
    }

    /// Check rate limit
    pub fn check_rate_limit(&self, resource: &str) -> bool {
        if let Some(limit) = self.rate_limits.get(resource) {
            limit.can_proceed()
        } else {
            true
        }
    }

    /// Write to shared memory
    pub fn write_shared_memory(&self, key: &str, data: Vec<u8>) {
        self.shared_memory.insert(key.to_string(), data);
    }

    /// Read from shared memory
    pub fn read_shared_memory(&self, key: &str) -> Option<Vec<u8>> {
        self.shared_memory.get(key).map(|entry| entry.clone())
    }
}

/// Rate limit information
#[derive(Debug, Clone)]
pub struct RateLimit {
    /// Maximum requests per interval
    pub max_requests: usize,
    /// Time interval
    pub interval: std::time::Duration,
    /// Current count
    pub current_count: Arc<RwLock<usize>>,
    /// Last reset time
    pub last_reset: Arc<RwLock<std::time::Instant>>,
}

impl RateLimit {
    /// Check if request can proceed
    pub fn can_proceed(&self) -> bool {
        // Simplified implementation
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_agent_session() {
        let multi_session = MultiAgentSession::new();
        let _agent_id = AgentId::new();

        // Would need a mock session for testing
        assert_eq!(multi_session.list_agents().len(), 0);
    }

    #[test]
    fn test_message_bus() {
        let bus = MessageBus::new();
        let agent1 = AgentId::new();
        let agent2 = AgentId::new();

        bus.register_agent(agent1.clone()).unwrap();
        bus.register_agent(agent2.clone()).unwrap();

        let message = Message {
            id: Uuid::new_v4(),
            from: agent1.clone(),
            message_type: MessageType::StatusUpdate,
            payload: serde_json::json!({"status": "ready"}),
            timestamp: chrono::Utc::now(),
        };

        bus.send_message(agent1, agent2.clone(), message).unwrap();

        if let Some(receiver) = bus.get_receiver(&agent2) {
            assert!(receiver.try_recv().is_ok());
        }
    }

    #[tokio::test]
    async fn test_agent_message_publish() {
        let bus = MessageBus::new();
        let agent1 = AgentId::new();
        let agent2 = AgentId::new();

        bus.register_agent(agent1.clone()).unwrap();
        bus.register_agent(agent2.clone()).unwrap();

        // Subscribe to all messages
        let all_receiver = bus.subscribe_all();

        // Create a registration message
        let registration_msg = AgentMessage::Registration {
            agent_id: agent1.clone(),
            capabilities: vec!["frontend".to_string(), "react".to_string()],
            metadata: serde_json::json!({"version": "1.0"}),
        };

        // Publish to agent2
        bus.publish_to_agent(&agent2, registration_msg.clone())
            .await
            .unwrap();

        // Check agent2 received the message
        if let Some(receiver) = bus.get_agent_receiver(&agent2) {
            let received = receiver.try_recv().unwrap();
            match received {
                AgentMessage::Registration { agent_id, .. } => {
                    assert_eq!(agent_id, agent1);
                }
                _ => panic!("Unexpected message type"),
            }
        }

        // Check all_messages channel received it too
        let all_msg = all_receiver.try_recv().unwrap();
        match all_msg {
            AgentMessage::Registration { agent_id, .. } => {
                assert_eq!(agent_id, agent1);
            }
            _ => panic!("Unexpected message type"),
        }
    }

    #[tokio::test]
    async fn test_all_agent_message_variants() {
        let bus = MessageBus::new();
        let agent1 = AgentId::new();
        bus.register_agent(agent1.clone()).unwrap();

        // Test all message variants
        let messages = vec![
            AgentMessage::Registration {
                agent_id: agent1.clone(),
                capabilities: vec!["test".to_string()],
                metadata: serde_json::json!({}),
            },
            AgentMessage::TaskAssignment {
                task_id: TaskId::new(),
                agent_id: agent1.clone(),
                task_data: serde_json::json!({"task": "test"}),
            },
            AgentMessage::TaskCompleted {
                agent_id: agent1.clone(),
                task_id: TaskId::new(),
                result: serde_json::json!({"success": true}),
            },
            AgentMessage::TaskProgress {
                agent_id: agent1.clone(),
                task_id: TaskId::new(),
                progress: 0.5,
                message: "Halfway done".to_string(),
            },
            AgentMessage::HelpRequest {
                agent_id: agent1.clone(),
                context: "Need help with React".to_string(),
                priority: MessagePriority::High,
            },
            AgentMessage::StatusUpdate {
                agent_id: agent1.clone(),
                status: "active".to_string(),
                metrics: serde_json::json!({"cpu": 50, "memory": 1024}),
            },
            AgentMessage::Custom {
                message_type: "test_message".to_string(),
                data: serde_json::json!({"foo": "bar"}),
            },
        ];

        for msg in messages {
            bus.publish_to_agent(&agent1, msg).await.unwrap();
        }

        // Verify all messages were received
        if let Some(receiver) = bus.get_agent_receiver(&agent1) {
            let mut count = 0;
            while receiver.try_recv().is_ok() {
                count += 1;
            }
            assert_eq!(count, 7); // All 7 message variants
        }
    }
}
