//! ccswarm integration module - Advanced AI agent coordination

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::agent::{AgentTask, AgentType, TaskType};

/// ccswarm agent coordinator
pub struct AgentCoordinator {
    /// Agent registry
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    /// Task queue
    task_queue: Arc<RwLock<TaskQueue>>,
    /// Message bus
    message_bus: mpsc::Sender<CoordinationMessage>,
    /// Session manager reference
    _session_manager: Arc<crate::SessionManager>,
}

/// Agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent ID
    pub id: String,
    /// Agent name
    pub name: String,
    /// Agent type
    pub agent_type: AgentType,
    /// Session ID
    pub session_id: String,
    /// Server URL
    pub server_url: String,
    /// Current status
    pub status: AgentStatus,
    /// Capabilities
    pub capabilities: Vec<TaskType>,
    /// Current workload
    pub current_tasks: Vec<String>,
}

/// Agent status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Ready to accept tasks
    Available,
    /// Currently working
    Busy,
    /// Temporarily unavailable
    Paused,
    /// Not responding
    Offline,
    /// Error state
    Error,
}

/// Task queue for coordinating work
pub struct TaskQueue {
    /// Pending tasks
    pending: Vec<AgentTask>,
    /// Active tasks
    active: HashMap<String, TaskAssignment>,
    /// Completed tasks
    completed: Vec<CompletedTask>,
    /// Task dependencies
    dependencies: HashMap<String, Vec<String>>,
}

/// Task assignment details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    /// Task
    pub task: AgentTask,
    /// Assigned agent
    pub agent_id: String,
    /// Assignment time
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    /// Start time
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Retry count
    pub retry_count: u32,
}

/// Completed task record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTask {
    /// Original task
    pub task: AgentTask,
    /// Agent that completed it
    pub agent_id: String,
    /// Completion time
    pub completed_at: chrono::DateTime<chrono::Utc>,
    /// Result
    pub result: TaskResult,
    /// Total time in ms
    pub duration_ms: u64,
}

/// Task result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Success status
    pub success: bool,
    /// Output
    pub output: Option<String>,
    /// Error message
    pub error: Option<String>,
    /// Artifacts produced
    pub artifacts: HashMap<String, serde_json::Value>,
}

/// Coordination message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationMessage {
    /// Message ID
    pub id: String,
    /// Message type
    pub msg_type: CoordinationMessageType,
    /// Source agent
    pub from: String,
    /// Target agent (optional)
    pub to: Option<String>,
    /// Payload
    pub payload: serde_json::Value,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Coordination message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinationMessageType {
    /// Agent registration
    AgentRegistration,
    /// Agent status update
    StatusUpdate,
    /// Task assignment
    TaskAssignment,
    /// Task accepted
    TaskAccepted,
    /// Task rejected
    TaskRejected,
    /// Task progress
    TaskProgress,
    /// Task completed
    TaskCompleted,
    /// Task failed
    TaskFailed,
    /// Request for help
    HelpRequest,
    /// Knowledge sharing
    KnowledgeShare,
    /// System announcement
    SystemAnnouncement,
}

impl AgentCoordinator {
    /// Create new coordinator
    pub fn new(
        session_manager: Arc<crate::SessionManager>,
    ) -> (Self, mpsc::Receiver<CoordinationMessage>) {
        let (tx, rx) = mpsc::channel(1000);

        let coordinator = Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            task_queue: Arc::new(RwLock::new(TaskQueue {
                pending: Vec::new(),
                active: HashMap::new(),
                completed: Vec::new(),
                dependencies: HashMap::new(),
            })),
            message_bus: tx,
            _session_manager: session_manager,
        };

        (coordinator, rx)
    }

    /// Register an agent
    pub async fn register_agent(&self, agent_info: AgentInfo) -> Result<()> {
        let agent_id = agent_info.id.clone();

        let mut agents = self.agents.write().await;
        agents.insert(agent_id.clone(), agent_info.clone());

        // Send registration message
        let msg = CoordinationMessage {
            id: Uuid::new_v4().to_string(),
            msg_type: CoordinationMessageType::AgentRegistration,
            from: agent_id,
            to: None,
            payload: serde_json::to_value(&agent_info)?,
            timestamp: chrono::Utc::now(),
        };

        self.message_bus.send(msg).await?;
        Ok(())
    }

    /// Submit a task
    pub async fn submit_task(&self, task: AgentTask) -> Result<()> {
        let mut queue = self.task_queue.write().await;

        // Check dependencies
        if !task.depends_on.is_empty() {
            queue
                .dependencies
                .insert(task.id.clone(), task.depends_on.clone());
        }

        // Add to pending queue
        queue.pending.push(task);

        // Trigger task assignment
        drop(queue);
        self.assign_pending_tasks().await?;

        Ok(())
    }

    /// Assign pending tasks to available agents
    async fn assign_pending_tasks(&self) -> Result<()> {
        let agents = self.agents.read().await;

        // Find available agents
        let available_agents: Vec<_> = agents
            .values()
            .filter(|a| a.status == AgentStatus::Available)
            .collect();

        if available_agents.is_empty() {
            return Ok(());
        }

        // Get tasks to assign
        let mut tasks_to_assign = Vec::new();
        {
            let queue = self.task_queue.read().await;
            for task in queue.pending.iter() {
                // Check dependencies
                if !self.has_unmet_dependencies(task, &queue.completed).await {
                    // Find suitable agent
                    if let Some(agent) = self.find_suitable_agent(task, &available_agents).await {
                        tasks_to_assign.push((task.clone(), agent.id.clone()));
                    }
                }
            }
        }

        // Now assign the tasks
        if !tasks_to_assign.is_empty() {
            let mut queue = self.task_queue.write().await;

            for (task, agent_id) in tasks_to_assign {
                let assignment = TaskAssignment {
                    task: task.clone(),
                    agent_id: agent_id.clone(),
                    assigned_at: chrono::Utc::now(),
                    started_at: None,
                    retry_count: 0,
                };

                queue.active.insert(task.id.clone(), assignment.clone());
                queue.pending.retain(|t| t.id != task.id);

                // Send assignment message
                let msg = CoordinationMessage {
                    id: Uuid::new_v4().to_string(),
                    msg_type: CoordinationMessageType::TaskAssignment,
                    from: "coordinator".to_string(),
                    to: Some(agent_id),
                    payload: serde_json::to_value(&assignment)?,
                    timestamp: chrono::Utc::now(),
                };

                self.message_bus.send(msg).await?;
            }
        }

        Ok(())
    }

    /// Check if task has unmet dependencies
    async fn has_unmet_dependencies(&self, task: &AgentTask, completed: &[CompletedTask]) -> bool {
        if task.depends_on.is_empty() {
            return false;
        }

        let completed_ids: Vec<_> = completed.iter().map(|t| &t.task.id).collect();
        task.depends_on
            .iter()
            .any(|dep| !completed_ids.contains(&dep))
    }

    /// Find suitable agent for task
    async fn find_suitable_agent<'a>(
        &self,
        task: &AgentTask,
        agents: &[&'a AgentInfo],
    ) -> Option<&'a AgentInfo> {
        agents
            .iter()
            .find(|agent| agent.capabilities.contains(&task.task_type))
            .copied()
    }

    /// Handle task completion
    pub async fn complete_task(&self, task_id: String, result: TaskResult) -> Result<()> {
        let mut queue = self.task_queue.write().await;

        if let Some(assignment) = queue.active.remove(&task_id) {
            let completed = CompletedTask {
                task: assignment.task,
                agent_id: assignment.agent_id.clone(),
                completed_at: chrono::Utc::now(),
                duration_ms: assignment
                    .started_at
                    .map(|start| (chrono::Utc::now() - start).num_milliseconds() as u64)
                    .unwrap_or(0),
                result,
            };

            queue.completed.push(completed.clone());

            // Send completion message
            let msg = CoordinationMessage {
                id: Uuid::new_v4().to_string(),
                msg_type: CoordinationMessageType::TaskCompleted,
                from: assignment.agent_id,
                to: None,
                payload: serde_json::to_value(&completed)?,
                timestamp: chrono::Utc::now(),
            };

            self.message_bus.send(msg).await?;

            // Check if this unblocks any tasks
            drop(queue);
            self.assign_pending_tasks().await?;
        }

        Ok(())
    }

    /// Get agent status
    pub async fn get_agent_status(&self, agent_id: &str) -> Option<AgentInfo> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// Get task queue status
    pub async fn get_queue_status(&self) -> Result<QueueStatus> {
        let queue = self.task_queue.read().await;

        Ok(QueueStatus {
            pending_count: queue.pending.len(),
            active_count: queue.active.len(),
            completed_count: queue.completed.len(),
            pending_tasks: queue.pending.clone(),
            active_tasks: queue.active.values().cloned().collect(),
            recent_completed: queue.completed.iter().rev().take(10).cloned().collect(),
        })
    }
}

/// Queue status summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    /// Number of pending tasks
    pub pending_count: usize,
    /// Number of active tasks
    pub active_count: usize,
    /// Number of completed tasks
    pub completed_count: usize,
    /// Pending tasks
    pub pending_tasks: Vec<AgentTask>,
    /// Active task assignments
    pub active_tasks: Vec<TaskAssignment>,
    /// Recent completed tasks
    pub recent_completed: Vec<CompletedTask>,
}

/// Smart prompt builder for different agent types
pub struct PromptBuilder {
    agent_type: AgentType,
    context: HashMap<String, String>,
}

impl PromptBuilder {
    /// Create new prompt builder
    pub fn new(agent_type: AgentType) -> Self {
        Self {
            agent_type,
            context: HashMap::new(),
        }
    }

    /// Add context
    pub fn with_context(mut self, key: &str, value: &str) -> Self {
        self.context.insert(key.to_string(), value.to_string());
        self
    }

    /// Build prompt for task
    pub fn build_task_prompt(&self, task: &AgentTask) -> String {
        match self.agent_type {
            AgentType::ClaudeCode => self.build_claude_prompt(task),
            AgentType::Aider => self.build_aider_prompt(task),
            _ => self.build_generic_prompt(task),
        }
    }

    fn build_claude_prompt(&self, task: &AgentTask) -> String {
        let mut prompt = String::new();

        // Add context if available
        if let Some(project) = self.context.get("project") {
            prompt.push_str(&format!("Project: {}\n\n", project));
        }

        // Task description
        prompt.push_str(&format!("Task: {}\n", task.description));

        // Add specific instructions based on task type
        match task.task_type {
            TaskType::CodeGeneration => {
                prompt.push_str("\nPlease generate the requested code with:\n");
                prompt.push_str("- Clear documentation\n");
                prompt.push_str("- Error handling\n");
                prompt.push_str("- Unit tests\n");
            }
            TaskType::CodeReview => {
                prompt.push_str("\nPlease review the code for:\n");
                prompt.push_str("- Correctness\n");
                prompt.push_str("- Performance\n");
                prompt.push_str("- Security issues\n");
                prompt.push_str("- Best practices\n");
            }
            TaskType::Debugging => {
                prompt.push_str("\nPlease debug the issue by:\n");
                prompt.push_str("- Identifying the root cause\n");
                prompt.push_str("- Suggesting fixes\n");
                prompt.push_str("- Preventing similar issues\n");
            }
            _ => {}
        }

        prompt
    }

    fn build_aider_prompt(&self, task: &AgentTask) -> String {
        // Aider-specific prompt format
        let mut prompt = format!("/ask {}\n", task.description);

        if let Some(files) = task.parameters.get("files") {
            if let Some(files_list) = files.as_array() {
                for file in files_list {
                    if let Some(file_str) = file.as_str() {
                        prompt.push_str(&format!("/add {}\n", file_str));
                    }
                }
            }
        }

        prompt
    }

    fn build_generic_prompt(&self, task: &AgentTask) -> String {
        task.description.clone()
    }
}
