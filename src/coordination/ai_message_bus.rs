/// Integration bridge for ai-session's MessageBus with ccswarm's coordination system
///
/// This module replaces the file-based JSON coordination with ai-session's
/// high-performance MessageBus, enabling:
/// - Real-time inter-agent communication
/// - Event-driven task delegation
/// - Distributed decision making
/// - Lower latency coordination (<100ms)
use anyhow::{Context as AnyhowContext, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

use ai_session::coordination::{AgentId, AgentMessage, MessageBus};

use crate::agent::{Task, TaskResult};
use crate::identity::AgentRole;
use crate::orchestrator::{DelegationDecision, MasterClaude};

/// Enhanced message types for ccswarm coordination
/// NOTE: These extend ai-session's AgentMessage for ccswarm-specific needs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CCSwarmMessage {
    /// Agent message from ai-session
    Base(AgentMessage),
    /// Ccswarm-specific delegation message
    Delegation {
        task_id: String,
        agent_id: AgentId,
        decision: Box<DelegationDecision>,
    },
    /// Quality review request
    QualityReview {
        task_id: String,
        agent_id: AgentId,
        review_type: String,
    },
    /// Session management message
    SessionCommand {
        session_id: String,
        command: SessionCommandType,
    },
}

/// Session command types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionCommandType {
    Start,
    Stop,
    Pause,
    Resume,
    Cleanup,
}

/// Bridge between ai-session's MessageBus and ccswarm's coordination
pub struct AICoordinationBridge {
    /// The underlying ai-session MessageBus
    message_bus: Arc<MessageBus>,

    /// Reference to Master Claude for delegation decisions
    master_handler: Arc<RwLock<MasterClaude>>,

    /// Mapping from ccswarm agent IDs to ai-session AgentIds
    agent_map: Arc<RwLock<HashMap<String, AgentId>>>,

    /// Active task tracking
    active_tasks: Arc<RwLock<HashMap<String, TrackedTask>>>,

    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
}

/// Tracked task information
#[derive(Debug, Clone)]
struct TrackedTask {
    task_id: String,
    agent_id: String,
    #[allow(dead_code)]
    ai_agent_id: AgentId,
    task: Task,
    started_at: chrono::DateTime<chrono::Utc>,
    status: TaskStatus,
}

#[derive(Debug, Clone)]
enum TaskStatus {
    Assigned,
    InProgress,
    #[allow(dead_code)]
    Completed(TaskResult),
    #[allow(dead_code)]
    Failed(String),
}

impl AICoordinationBridge {
    /// Create a new coordination bridge
    pub fn new(message_bus: Arc<MessageBus>, master_handler: Arc<RwLock<MasterClaude>>) -> Self {
        Self {
            message_bus,
            master_handler,
            agent_map: Arc::new(RwLock::new(HashMap::new())),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx: None,
        }
    }

    /// Initialize the coordination bridge and start listening for messages
    pub async fn initialize(&mut self) -> Result<()> {
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Subscribe to all agent messages
        let receiver = self.message_bus.subscribe_all();

        // Clone references for the async task
        let master_handler = Arc::clone(&self.master_handler);
        let agent_map = Arc::clone(&self.agent_map);
        let active_tasks = Arc::clone(&self.active_tasks);
        let message_bus = Arc::clone(&self.message_bus);

        // Spawn message handling task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Shutting down AI coordination bridge");
                        break;
                    }
                    else => {
                        // Use blocking receive in a separate task to avoid blocking tokio runtime
                        if let Ok(message) = receiver.try_recv() {
                            if let Err(e) = Self::handle_agent_message(
                                message,
                                &master_handler,
                                &agent_map,
                                &active_tasks,
                                &message_bus,
                            ).await {
                                tracing::error!("Error handling agent message: {}", e);
                            }
                        } else {
                            // Sleep briefly to avoid busy waiting
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                        }
                    }
                }
            }
        });

        tracing::info!("AI coordination bridge initialized");
        Ok(())
    }

    /// Register a ccswarm agent with the message bus
    pub async fn register_agent(
        &self,
        ccswarm_agent_id: String,
        agent_role: AgentRole,
    ) -> Result<AgentId> {
        let ai_agent_id = AgentId::new();

        // Store mapping
        {
            let mut agent_map = self.agent_map.write().await;
            agent_map.insert(ccswarm_agent_id.clone(), ai_agent_id.clone());
        }

        // Send registration message
        let registration = AgentMessage::Registration {
            agent_id: ai_agent_id.clone(),
            capabilities: vec![agent_role.name().to_string()],
            metadata: serde_json::json!({
                "ccswarm_agent_id": ccswarm_agent_id,
                "role": agent_role.name(),
                "registered_at": Utc::now().to_rfc3339(),
            }),
        };

        // Use a dummy agent ID to send registration for monitoring
        let dummy_agent = AgentId::new();
        self.message_bus
            .publish_to_agent(&dummy_agent, registration)
            .await
            .ok(); // Ignore error

        tracing::info!(
            "Registered agent {} as {} with AI agent ID: {}",
            ccswarm_agent_id,
            agent_role.name(),
            ai_agent_id
        );

        Ok(ai_agent_id)
    }

    /// Delegate a task to an agent via the message bus
    pub async fn delegate_task(
        &self,
        task: Task,
        agent_id: &str,
        decision: DelegationDecision,
    ) -> Result<()> {
        let agent_map = self.agent_map.read().await;
        let ai_agent_id = agent_map
            .get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent {} not registered", agent_id))?
            .clone();
        drop(agent_map);

        // Track the task
        let tracked_task = TrackedTask {
            task_id: task.id.clone(),
            agent_id: agent_id.to_string(),
            ai_agent_id: ai_agent_id.clone(),
            task: task.clone(),
            started_at: Utc::now(),
            status: TaskStatus::Assigned,
        };

        {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.insert(task.id.clone(), tracked_task);
        }

        // Send task assignment message
        let assignment = AgentMessage::TaskAssignment {
            task_id: ai_session::coordination::TaskId::new(),
            agent_id: ai_agent_id.clone(),
            task_data: serde_json::json!({
                "description": task.description,
                "priority": task.priority,
                "task_type": task.task_type,
                "details": task.details,
                "delegation_reason": decision.reasoning,
                "confidence": decision.confidence,
            }),
        };

        self.message_bus
            .publish_to_agent(&ai_agent_id, assignment)
            .await?;

        tracing::info!(
            "Delegated task {} to agent {} with confidence {}",
            task.id,
            agent_id,
            decision.confidence
        );

        Ok(())
    }

    /// Handle incoming agent messages
    async fn handle_agent_message(
        msg: AgentMessage,
        master_handler: &Arc<RwLock<MasterClaude>>,
        agent_map: &Arc<RwLock<HashMap<String, AgentId>>>,
        active_tasks: &Arc<RwLock<HashMap<String, TrackedTask>>>,
        message_bus: &Arc<MessageBus>,
    ) -> Result<()> {
        match msg {
            AgentMessage::TaskCompleted {
                agent_id,
                task_id,
                result,
            } => {
                Self::handle_task_completion(
                    agent_id,
                    task_id.to_string(),
                    result,
                    master_handler,
                    active_tasks,
                )
                .await?;
            }

            AgentMessage::TaskProgress {
                agent_id,
                task_id,
                progress,
                message,
            } => {
                Self::handle_task_progress(
                    agent_id,
                    task_id.to_string(),
                    progress,
                    Some(message),
                    active_tasks,
                )
                .await?;
            }

            AgentMessage::HelpRequest {
                agent_id,
                context,
                priority,
            } => {
                let priority_f32 = match priority {
                    ai_session::coordination::MessagePriority::Low => 0.25,
                    ai_session::coordination::MessagePriority::Normal => 0.5,
                    ai_session::coordination::MessagePriority::High => 0.75,
                    ai_session::coordination::MessagePriority::Critical => 1.0,
                };
                Self::handle_help_request(
                    agent_id,
                    serde_json::Value::String(context),
                    priority_f32,
                    master_handler,
                    agent_map,
                    message_bus,
                )
                .await?;
            }

            AgentMessage::StatusUpdate {
                agent_id,
                status,
                metrics,
            } => {
                Self::handle_status_update(agent_id, status, Some(metrics)).await?;
            }

            _ => {
                // Other message types can be handled as needed
                tracing::debug!("Received message type: {:?}", msg);
            }
        }

        Ok(())
    }

    /// Handle task completion messages
    async fn handle_task_completion(
        agent_id: AgentId,
        task_id: String,
        result_data: serde_json::Value,
        master_handler: &Arc<RwLock<MasterClaude>>,
        active_tasks: &Arc<RwLock<HashMap<String, TrackedTask>>>,
    ) -> Result<()> {
        // Parse task result
        let task_result: TaskResult =
            serde_json::from_value(result_data).context("Failed to parse task result")?;

        // Update task status
        {
            let mut tasks = active_tasks.write().await;
            if let Some(tracked_task) = tasks.get_mut(&task_id) {
                tracked_task.status = TaskStatus::Completed(task_result.clone());
            }
        }

        // Notify Master Claude
        let master = master_handler.read().await;
        master.handle_task_completion(&task_id, task_result).await?;

        tracing::info!("Task {} completed by agent {}", task_id, agent_id);
        Ok(())
    }

    /// Handle task progress updates
    async fn handle_task_progress(
        _agent_id: AgentId,
        task_id: String,
        progress: f32,
        message: Option<String>,
        active_tasks: &Arc<RwLock<HashMap<String, TrackedTask>>>,
    ) -> Result<()> {
        let tasks = active_tasks.read().await;
        if let Some(tracked_task) = tasks.get(&task_id) {
            tracing::info!(
                "Task {} progress: {}% - {}",
                task_id,
                (progress * 100.0) as u32,
                message.unwrap_or_default()
            );

            // Could update task status to InProgress if needed
            if matches!(tracked_task.status, TaskStatus::Assigned) {
                drop(tasks);
                let mut tasks = active_tasks.write().await;
                if let Some(tracked_task) = tasks.get_mut(&task_id) {
                    tracked_task.status = TaskStatus::InProgress;
                }
            }
        }

        Ok(())
    }

    /// Handle help requests from agents
    async fn handle_help_request(
        agent_id: AgentId,
        context: serde_json::Value,
        priority: f32,
        master_handler: &Arc<RwLock<MasterClaude>>,
        agent_map: &Arc<RwLock<HashMap<String, AgentId>>>,
        message_bus: &Arc<MessageBus>,
    ) -> Result<()> {
        tracing::info!(
            "Help request from agent {} with priority {}",
            agent_id,
            priority
        );

        // Find the ccswarm agent ID
        let agent_map_guard = agent_map.read().await;
        let ccswarm_agent_id = agent_map_guard
            .iter()
            .find(|(_, ai_id)| **ai_id == agent_id)
            .map(|(cc_id, _)| cc_id.clone());
        drop(agent_map_guard);

        if let Some(ccswarm_id) = ccswarm_agent_id {
            // Let Master Claude handle the help request
            let master = master_handler.read().await;
            let response = master.handle_help_request(&ccswarm_id, context).await?;

            // Send response back via message bus
            let help_response = AgentMessage::Custom {
                message_type: "help_response".to_string(),
                data: response,
            };

            message_bus
                .publish_to_agent(&agent_id, help_response)
                .await?;
        }

        Ok(())
    }

    /// Handle status updates from agents
    async fn handle_status_update(
        agent_id: AgentId,
        status: String,
        metrics: Option<serde_json::Value>,
    ) -> Result<()> {
        tracing::debug!(
            "Status update from agent {}: {} (metrics: {:?})",
            agent_id,
            status,
            metrics
        );

        // Could store status for monitoring/dashboard
        Ok(())
    }

    /// Get active tasks summary
    pub async fn get_active_tasks_summary(&self) -> Vec<TaskSummary> {
        let tasks = self.active_tasks.read().await;
        tasks
            .values()
            .map(|tracked| TaskSummary {
                task_id: tracked.task_id.clone(),
                agent_id: tracked.agent_id.clone(),
                description: tracked.task.description.clone(),
                status: format!("{:?}", tracked.status),
                started_at: tracked.started_at,
                duration: Utc::now() - tracked.started_at,
            })
            .collect()
    }

    /// Broadcast a message to all agents
    pub async fn broadcast_message(&self, from_agent: AgentId, message: String) -> Result<()> {
        let broadcast_msg = ai_session::coordination::BroadcastMessage {
            id: uuid::Uuid::new_v4(),
            from: from_agent.clone(),
            content: message,
            priority: ai_session::coordination::MessagePriority::Normal,
            timestamp: chrono::Utc::now(),
        };
        self.message_bus.broadcast(from_agent, broadcast_msg)?;
        Ok(())
    }

    /// Send a message to a specific agent
    pub async fn send_to_agent(&self, agent_id: &str, message: AgentMessage) -> Result<()> {
        let agent_map = self.agent_map.read().await;
        let ai_agent_id = agent_map
            .get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent {} not registered", agent_id))?;

        self.message_bus
            .publish_to_agent(ai_agent_id, message)
            .await?;
        Ok(())
    }

    /// Shutdown the coordination bridge
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        Ok(())
    }

    /// Create a task delegation message
    pub fn create_delegation_message(
        &self,
        task_id: String,
        agent_id: AgentId,
        decision: DelegationDecision,
    ) -> CCSwarmMessage {
        CCSwarmMessage::Delegation {
            task_id,
            agent_id,
            decision: Box::new(decision),
        }
    }

    /// Create a quality review message
    pub fn create_quality_review_message(
        &self,
        task_id: String,
        agent_id: AgentId,
        review_type: String,
    ) -> CCSwarmMessage {
        CCSwarmMessage::QualityReview {
            task_id,
            agent_id,
            review_type,
        }
    }
}

/// Task summary for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub task_id: String,
    pub agent_id: String,
    pub description: String,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub duration: chrono::Duration,
}

/// Extension trait for MasterClaude integration
/// NOTE: These are placeholder methods until MasterClaude implements proper coordination
#[async_trait::async_trait]
trait MasterClaudeCoordination {
    async fn handle_task_completion(&self, task_id: &str, result: TaskResult) -> Result<()>;
    async fn handle_help_request(
        &self,
        agent_id: &str,
        context: serde_json::Value,
    ) -> Result<serde_json::Value>;
}

// Temporary implementation for MasterClaude
#[async_trait::async_trait]
impl MasterClaudeCoordination for MasterClaude {
    async fn handle_task_completion(&self, task_id: &str, result: TaskResult) -> Result<()> {
        // TODO: Implement actual task completion handling
        tracing::info!("Task {} completed with result: {:?}", task_id, result);
        Ok(())
    }

    async fn handle_help_request(
        &self,
        agent_id: &str,
        context: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // TODO: Implement actual help request handling
        tracing::info!("Help request from agent {}: {:?}", agent_id, context);
        Ok(serde_json::json!({
            "response": "Help acknowledged",
            "suggestions": ["Continue with current approach", "Check documentation"],
        }))
    }
}

#[cfg(test)]
mod tests {

    // TODO: Add tests when MasterClaude has a testable constructor
    // Currently commented out as MasterClaude requires CcswarmConfig and repo_path

    // #[tokio::test]
    // async fn test_coordination_bridge_creation() {
    //     let message_bus = Arc::new(MessageBus::new());
    //     let master_handler = Arc::new(RwLock::new(MasterClaude::default()));
    //
    //     let bridge = AICoordinationBridge::new(message_bus, master_handler);
    //     assert!(bridge.shutdown_tx.is_none());
    // }

    // #[tokio::test]
    // async fn test_agent_registration() {
    //     let message_bus = Arc::new(MessageBus::new());
    //     let master_handler = Arc::new(RwLock::new(MasterClaude::default()));
    //
    //     let bridge = AICoordinationBridge::new(message_bus, master_handler);
    //
    //     let ai_agent_id = bridge.register_agent(
    //         "test-agent-123".to_string(),
    //         crate::identity::default_frontend_role(),
    //     ).await.unwrap();
    //
    //     let agent_map = bridge.agent_map.read().await;
    //     assert!(agent_map.contains_key("test-agent-123"));
    // }
}
