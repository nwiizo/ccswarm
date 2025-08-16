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

use ai_session::coordination::{AgentId, AgentMessage as AISessionMessage, MessageBus};

use crate::agent::{Task, TaskResult};
use crate::coordination::conversion::{AgentMappingRegistry, UnifiedAgentInfo};
use crate::coordination::AgentMessage as CCSwarmAgentMessage;
use crate::identity::AgentRole;
use crate::orchestrator::MasterClaude;
use crate::orchestrator::master_delegation::DelegationDecision;

/// Enhanced message types for ccswarm coordination
/// NOTE: These extend ai-session's AgentMessage for ccswarm-specific needs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CCSwarmMessage {
    /// Agent message from ai-session
    Base(CCSwarmAgentMessage),
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

    /// Agent mapping registry for ID conversions
    agent_registry: Arc<AgentMappingRegistry>,

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
            agent_registry: Arc::new(AgentMappingRegistry::new()),
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
        let agent_registry = Arc::clone(&self.agent_registry);
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
                                &agent_registry,
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

        // Create unified agent info
        let agent_info = UnifiedAgentInfo {
            ccswarm_id: ccswarm_agent_id.clone(),
            ai_session_id: ai_agent_id.clone(),
            role: agent_role.clone(),
            capabilities: vec![agent_role.name().to_string()],
            metadata: serde_json::json!({
                "ccswarm_agent_id": ccswarm_agent_id,
                "role": agent_role.name(),
                "registered_at": Utc::now().to_rfc3339(),
            }),
        };

        // Register in the mapping registry
        self.agent_registry.register(agent_info.clone()).await;

        // Send registration message
        let registration = AISessionMessage::Registration {
            agent_id: ai_agent_id.clone(),
            capabilities: agent_info.capabilities,
            metadata: agent_info.metadata,
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
        let ai_agent_id = self
            .agent_registry
            .get_ai_session_id(agent_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Agent {} not registered", agent_id))?;

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
        let assignment = AISessionMessage::TaskAssignment {
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
        msg: AISessionMessage,
        master_handler: &Arc<RwLock<MasterClaude>>,
        agent_registry: &Arc<AgentMappingRegistry>,
        active_tasks: &Arc<RwLock<HashMap<String, TrackedTask>>>,
        message_bus: &Arc<MessageBus>,
    ) -> Result<()> {
        match msg {
            AISessionMessage::TaskCompleted {
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

            AISessionMessage::TaskProgress {
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

            AISessionMessage::HelpRequest {
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
                    agent_registry,
                    message_bus,
                )
                .await?;
            }

            AISessionMessage::StatusUpdate {
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
        agent_registry: &Arc<AgentMappingRegistry>,
        message_bus: &Arc<MessageBus>,
    ) -> Result<()> {
        tracing::info!(
            "Help request from agent {} with priority {}",
            agent_id,
            priority
        );

        // Find the ccswarm agent ID
        let ccswarm_agent_id = agent_registry.get_ccswarm_id(&agent_id).await;

        if let Some(ccswarm_id) = ccswarm_agent_id {
            // Let Master Claude handle the help request
            let master = master_handler.read().await;
            let response = master.handle_help_request(&ccswarm_id, context).await?;

            // Send response back via message bus
            let help_response = AISessionMessage::Custom {
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
    pub async fn send_to_agent(&self, agent_id: &str, message: AISessionMessage) -> Result<()> {
        let ai_agent_id = self
            .agent_registry
            .get_ai_session_id(agent_id)
            .await
            .ok_or_else(|| anyhow::anyhow!("Agent {} not registered", agent_id))?;

        self.message_bus
            .publish_to_agent(&ai_agent_id, message)
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
        tracing::info!("Task {} completed with result: {:?}", task_id, result);

        // Update orchestrator state
        // Update orchestrator state - simplified access
        // TODO: Implement proper state management

        if result.success {
            // state.successful_tasks += 1;
            tracing::info!("Task {} completed successfully", task_id);
        } else {
            // state.failed_tasks += 1;
            let error_msg = result.error.as_deref().unwrap_or("Unknown error");
            tracing::error!("Task {} failed: {}", task_id, error_msg);

            // Coordination bus notification disabled - field not available
            // TODO: Implement coordination bus for MasterClaude
            tracing::warn!("Task failure notification skipped - coordination bus not available");
        }

        // Task list management disabled - field not available
        // TODO: Implement task list management for MasterClaude

        Ok(())
    }

    async fn handle_help_request(
        &self,
        agent_id: &str,
        context: serde_json::Value,
    ) -> Result<serde_json::Value> {
        tracing::info!("Help request from agent {}: {:?}", agent_id, context);

        // Extract context information
        let current_task = context.get("current_task").and_then(|v| v.as_str());
        let error_message = context.get("error").and_then(|v| v.as_str());
        let _stuck_reason = context.get("stuck_reason").and_then(|v| v.as_str());

        // Generate intelligent suggestions based on agent role and context
        // TODO: Implement agent access for MasterClaude
        let suggestions = {
            // Default suggestions when agent access is not available
            match agent_id {
                "Frontend" => vec![
                    "Check browser console for errors",
                    "Verify component imports and exports",
                    "Test with different browsers/versions",
                    "Review CSS/styling conflicts",
                ],
                "Backend" => vec![
                    "Check server logs for errors",
                    "Verify database connections",
                    "Test API endpoints independently",
                    "Review authentication/authorization",
                ],
                "DevOps" => vec![
                    "Check container/deployment logs",
                    "Verify environment variables",
                    "Test infrastructure connectivity",
                    "Review CI/CD pipeline status",
                ],
                "QA" => vec![
                    "Review test coverage reports",
                    "Check for flaky tests",
                    "Verify test data setup",
                    "Review testing environment config",
                ],
                "Search" => vec![
                    "Verify gemini CLI is installed and accessible",
                    "Check search query syntax and filters",
                    "Review search API rate limits",
                    "Try different search terms or sources",
                ],
                _ => vec![
                    "Break down the problem into smaller steps",
                    "Check documentation and examples",
                    "Review recent changes that might be related",
                    "Ask for clarification on requirements",
                    "Check logs for detailed errors",
                    "Verify environment configuration",
                ],
            }
        };

        // Add context-specific suggestions
        let mut contextual_suggestions = suggestions;
        if let Some(error) = error_message {
            if error.contains("compilation") || error.contains("build") {
                contextual_suggestions.push("Check for syntax errors and missing dependencies");
            } else if error.contains("network") || error.contains("connection") {
                contextual_suggestions
                    .push("Verify network connectivity and endpoint availability");
            } else if error.contains("permission") || error.contains("access") {
                contextual_suggestions.push("Check file permissions and access rights");
            }
        }

        // Send help response through coordination bus
        let help_response = serde_json::json!({
            "response": "Help request processed",
            "suggestions": contextual_suggestions,
            "next_steps": [
                "Try the suggested approaches",
                "Report back with results",
                "Request specific guidance if still stuck"
            ],
            "context": {
                "agent_id": agent_id,
                "timestamp": chrono::Utc::now(),
                "task": current_task
            }
        });

        // Coordination bus notification disabled - field not available
        // TODO: Implement coordination bus for MasterClaude
        tracing::warn!("Help response broadcast skipped - coordination bus not available");

        Ok(help_response)
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
