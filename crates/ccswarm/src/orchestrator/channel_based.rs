use crate::agent::Task;
use crate::error::{CCSwarmError, Result};
use std::collections::HashMap;
/// Channel-based orchestration using Rust best practices
///
/// This module implements efficient message-passing concurrency
/// without shared state, leveraging Rust's ownership system.
use tokio::sync::{mpsc, oneshot};

/// Messages for the orchestrator
#[derive(Debug)]
pub enum OrchestratorMsg {
    /// Submit a new task
    SubmitTask {
        task: Task,
        reply: oneshot::Sender<Result<String>>,
    },
    /// Assign task to agent
    AssignAgent {
        task_id: String,
        agent_id: String,
        reply: oneshot::Sender<Result<()>>,
    },
    /// Get current status
    GetStatus {
        reply: oneshot::Sender<OrchestratorStatus>,
    },
    /// Shutdown orchestrator
    Shutdown,
}

/// Orchestrator status information
#[derive(Debug, Clone)]
pub struct OrchestratorStatus {
    pub pending_tasks: usize,
    pub active_tasks: usize,
    pub completed_tasks: usize,
    pub active_agents: Vec<String>,
}

/// Channel-based orchestrator handle
#[derive(Clone)]
pub struct OrchestratorHandle {
    tx: mpsc::Sender<OrchestratorMsg>,
}

impl OrchestratorHandle {
    /// Submit a task and get its ID
    pub async fn submit_task(&self, task: Task) -> Result<String> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(OrchestratorMsg::SubmitTask { task, reply })
            .await
            .map_err(|_| CCSwarmError::orchestrator("Orchestrator channel closed", None))?;

        rx.await
            .map_err(|_| CCSwarmError::orchestrator("Failed to receive reply", None))?
    }

    /// Assign a task to an agent
    pub async fn assign_agent(&self, task_id: String, agent_id: String) -> Result<()> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(OrchestratorMsg::AssignAgent {
                task_id,
                agent_id,
                reply,
            })
            .await
            .map_err(|_| CCSwarmError::orchestrator("Orchestrator channel closed", None))?;

        rx.await
            .map_err(|_| CCSwarmError::orchestrator("Failed to receive reply", None))?
    }

    /// Get current orchestrator status
    pub async fn get_status(&self) -> Result<OrchestratorStatus> {
        let (reply, rx) = oneshot::channel();
        self.tx
            .send(OrchestratorMsg::GetStatus { reply })
            .await
            .map_err(|_| CCSwarmError::orchestrator("Orchestrator channel closed", None))?;

        rx.await
            .map_err(|_| CCSwarmError::orchestrator("Failed to receive status", None))
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        self.tx
            .send(OrchestratorMsg::Shutdown)
            .await
            .map_err(|_| CCSwarmError::orchestrator("Orchestrator already shut down", None))?;
        Ok(())
    }
}

/// The actual orchestrator that processes messages
pub struct Orchestrator {
    rx: mpsc::Receiver<OrchestratorMsg>,
    pending_tasks: Vec<Task>,
    active_tasks: HashMap<String, (Task, String)>, // task_id -> (task, agent_id)
    completed_tasks: Vec<String>,
    active_agents: HashMap<String, Option<String>>, // agent_id -> task_id
}

impl Orchestrator {
    /// Create a new orchestrator with a handle for external communication
    pub fn new() -> (Self, OrchestratorHandle) {
        let (tx, rx) = mpsc::channel(100);

        let orchestrator = Self {
            rx,
            pending_tasks: Vec::new(),
            active_tasks: HashMap::new(),
            completed_tasks: Vec::new(),
            active_agents: HashMap::new(),
        };

        let handle = OrchestratorHandle { tx };

        (orchestrator, handle)
    }

    /// Run the orchestrator event loop
    pub async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                OrchestratorMsg::SubmitTask { task, reply } => {
                    let task_id = self.handle_submit_task(task);
                    let _ = reply.send(Ok(task_id));
                }
                OrchestratorMsg::AssignAgent {
                    task_id,
                    agent_id,
                    reply,
                } => {
                    let result = self.handle_assign_agent(task_id, agent_id);
                    let _ = reply.send(result);
                }
                OrchestratorMsg::GetStatus { reply } => {
                    let status = self.get_current_status();
                    let _ = reply.send(status);
                }
                OrchestratorMsg::Shutdown => {
                    tracing::info!("Orchestrator shutting down");
                    break;
                }
            }
        }

        tracing::info!("Orchestrator event loop terminated");
    }

    fn handle_submit_task(&mut self, task: Task) -> String {
        let task_id = task.id.clone();
        self.pending_tasks.push(task);
        tracing::debug!("Task {} submitted", task_id);
        task_id
    }

    fn handle_assign_agent(&mut self, task_id: String, agent_id: String) -> Result<()> {
        // Find the task in pending
        let task_index = self
            .pending_tasks
            .iter()
            .position(|t| t.id == task_id)
            .ok_or_else(|| CCSwarmError::orchestrator("Task not found", None))?;

        let task = self.pending_tasks.remove(task_index);

        // Verify agent is available
        if let Some(current_task) = self.active_agents.get(&agent_id) {
            if current_task.is_some() {
                return Err(CCSwarmError::orchestrator("Agent is busy", None));
            }
        }

        // Assign the task
        self.active_tasks
            .insert(task_id.clone(), (task, agent_id.clone()));
        self.active_agents
            .insert(agent_id.clone(), Some(task_id.clone()));

        tracing::info!("Task {} assigned to agent {}", task_id, agent_id);
        Ok(())
    }

    fn get_current_status(&self) -> OrchestratorStatus {
        OrchestratorStatus {
            pending_tasks: self.pending_tasks.len(),
            active_tasks: self.active_tasks.len(),
            completed_tasks: self.completed_tasks.len(),
            active_agents: self
                .active_agents
                .iter()
                .filter(|(_, task)| task.is_some())
                .map(|(agent_id, _)| agent_id.clone())
                .collect(),
        }
    }
}

/// Spawn the orchestrator as a background task
pub fn spawn_orchestrator() -> OrchestratorHandle {
    let (orchestrator, handle) = Orchestrator::new();

    tokio::spawn(async move {
        orchestrator.run().await;
    });

    handle
}
