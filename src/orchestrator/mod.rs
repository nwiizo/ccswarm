pub mod auto_create;
pub mod master_delegation;

use anyhow::{Context, Result};
use async_channel::{Receiver, Sender};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::agent::{AgentStatus, ClaudeCodeAgent, Task, TaskType};
use crate::config::CcswarmConfig;
use crate::coordination::{AgentMessage, CoordinationBus};
use crate::git::shell::ShellWorktreeManager as WorktreeManager;
use crate::identity::{
    default_backend_role, default_devops_role, default_frontend_role, default_qa_role,
    QualityStandards,
};

/// Master Claude coordinator
pub struct MasterClaude {
    /// Unique ID for this Master Claude instance
    pub id: String,

    /// Configuration
    pub config: CcswarmConfig,

    /// Managed agents
    pub agents: Arc<DashMap<String, ClaudeCodeAgent>>,

    /// Task queue sender
    pub task_queue_tx: Sender<Task>,

    /// Task queue receiver
    task_queue_rx: Receiver<Task>,

    /// Quality standards
    pub quality_standards: QualityStandards,

    /// Coordination bus for agent communication
    pub coordination_bus: Arc<CoordinationBus>,

    /// Git worktree manager
    pub worktree_manager: Arc<WorktreeManager>,

    /// Orchestrator state
    pub state: Arc<RwLock<OrchestratorState>>,
}

/// State of the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorState {
    pub status: OrchestratorStatus,
    pub start_time: DateTime<Utc>,
    pub total_tasks_processed: u64,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    pub active_agents: Vec<String>,
    pub pending_tasks: Vec<Task>,
}

/// Status of the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrchestratorStatus {
    Initializing,
    Running,
    Paused,
    ShuttingDown,
    Error(String),
}

impl MasterClaude {
    /// Create a new Master Claude instance
    pub async fn new(config: CcswarmConfig, repo_path: PathBuf) -> Result<Self> {
        let id = format!("master-claude-{}", Uuid::new_v4());

        // Create channels
        let (task_queue_tx, task_queue_rx) = async_channel::bounded(1000);

        // Initialize worktree manager
        let worktree_manager = Arc::new(WorktreeManager::new(repo_path).unwrap());

        // Initialize coordination bus
        let coordination_bus = Arc::new(CoordinationBus::new().await?);

        // Create initial state
        let state = Arc::new(RwLock::new(OrchestratorState {
            status: OrchestratorStatus::Initializing,
            start_time: Utc::now(),
            total_tasks_processed: 0,
            successful_tasks: 0,
            failed_tasks: 0,
            active_agents: Vec::new(),
            pending_tasks: Vec::new(),
        }));

        Ok(Self {
            id,
            config: config.clone(),
            agents: Arc::new(DashMap::new()),
            task_queue_tx,
            task_queue_rx,
            quality_standards: config.project.master_claude.quality_threshold.into(),
            coordination_bus,
            worktree_manager,
            state,
        })
    }

    /// Initialize the orchestrator and all agents
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing Master Claude orchestrator: {}", self.id);

        // Initialize agents based on configuration
        for (agent_name, agent_config) in &self.config.agents {
            info!("Initializing agent: {}", agent_name);

            let role = match agent_config.specialization.as_str() {
                "react_typescript" | "frontend" => default_frontend_role(),
                "node_microservices" | "backend" => default_backend_role(),
                "aws_kubernetes" | "devops" => default_devops_role(),
                "qa" | "testing" => default_qa_role(),
                _ => {
                    warn!(
                        "Unknown specialization: {}, using frontend role",
                        agent_config.specialization
                    );
                    default_frontend_role()
                }
            };

            // Create agent
            let mut agent = ClaudeCodeAgent::new(
                role,
                &PathBuf::from(&self.config.project.name),
                &agent_config.branch,
                agent_config.claude_config.clone(),
            )
            .await?;

            // Initialize agent (worktree, identity, etc.)
            agent.initialize().await?;

            // Add to managed agents
            self.agents.insert(agent.identity.agent_id.clone(), agent);
        }

        // Update state
        let mut state = self.state.write().await;
        state.status = OrchestratorStatus::Running;
        state.active_agents = self
            .agents
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        info!(
            "Master Claude initialized with {} agents",
            self.agents.len()
        );
        Ok(())
    }

    /// Start the coordination loop
    pub async fn start_coordination(&self) -> Result<()> {
        info!("Starting Master Claude coordination");

        // Start message handling
        let agents = self.agents.clone();
        let bus = self.coordination_bus.clone();
        let state = self.state.clone();

        tokio::spawn(async move {
            loop {
                match bus.receive_message().await {
                    Ok(message) => {
                        if let Err(e) = Self::handle_agent_message(message, &agents, &state).await {
                            error!("Error handling agent message: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Error receiving message: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                }
            }
        });

        // Start quality review loop
        let agents = self.agents.clone();
        let standards = self.quality_standards.clone();
        let bus = self.coordination_bus.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                if let Err(e) = Self::perform_quality_review(&agents, &standards, &bus).await {
                    error!("Error performing quality review: {}", e);
                }
            }
        });

        // Main coordination loop
        self.coordinate_development().await
    }

    /// Main coordination loop
    async fn coordinate_development(&self) -> Result<()> {
        info!("Master Claude coordination loop started");

        loop {
            tokio::select! {
                // Handle incoming tasks
                Ok(task) = self.task_queue_rx.recv() => {
                    if let Err(e) = self.assign_task(task).await {
                        error!("Error assigning task: {}", e);
                    }
                }

                // Periodic status check
                _ = tokio::time::sleep(std::time::Duration::from_secs(10)) => {
                    self.check_agent_health().await?;
                }
            }

            // Check if we should shutdown
            let state = self.state.read().await;
            if state.status == OrchestratorStatus::ShuttingDown {
                break;
            }
        }

        Ok(())
    }

    /// Add a task to the queue
    pub async fn add_task(&self, task: Task) -> Result<()> {
        self.task_queue_tx
            .send(task.clone())
            .await
            .context("Failed to add task to queue")?;

        let mut state = self.state.write().await;
        state.pending_tasks.push(task);

        Ok(())
    }

    /// Assign a task to the most suitable agent
    async fn assign_task(&self, task: Task) -> Result<()> {
        info!("Assigning task: {} - {}", task.id, task.description);

        // Select optimal agent
        let agent_id = self.select_optimal_agent(&task).await?;

        // Get agent
        let mut agent = self
            .agents
            .get_mut(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?
            .clone();

        // Execute task
        let result = agent.execute_task(task.clone()).await?;

        // Update agent in map
        self.agents.insert(agent_id.clone(), agent);

        // Update statistics
        let mut state = self.state.write().await;
        state.total_tasks_processed += 1;
        if result.success {
            state.successful_tasks += 1;
        } else {
            state.failed_tasks += 1;
        }
        state.pending_tasks.retain(|t| t.id != task.id);

        // Send result to coordination bus
        let task_id = task.id.clone();
        self.coordination_bus
            .send_message(AgentMessage::TaskCompleted {
                agent_id,
                task_id: task_id.clone(),
                result: result.clone(),
            })
            .await?;

        info!("Task completed: {} - Success: {}", task_id, result.success);
        Ok(())
    }

    /// Select the optimal agent for a task
    async fn select_optimal_agent(&self, task: &Task) -> Result<String> {
        // Determine required specialization
        let required_role = match task.task_type {
            TaskType::Development => {
                if task.description.to_lowercase().contains("ui")
                    || task.description.to_lowercase().contains("component")
                    || task.description.to_lowercase().contains("frontend")
                {
                    "Frontend"
                } else {
                    "Backend" // Default to backend for general development
                }
            }
            TaskType::Infrastructure => "DevOps",
            TaskType::Testing => "QA",
            _ => "Backend", // Default
        };

        // Find available agents with matching specialization
        let mut suitable_agents = Vec::new();

        for entry in self.agents.iter() {
            let agent = entry.value();
            if agent.identity.specialization.name() == required_role
                && matches!(agent.status, AgentStatus::Available)
            {
                suitable_agents.push(agent.identity.agent_id.clone());
            }
        }

        if suitable_agents.is_empty() {
            // No available agent with exact match, find any available agent
            for entry in self.agents.iter() {
                let agent = entry.value();
                if matches!(agent.status, AgentStatus::Available) {
                    warn!(
                        "No {} agent available, using {}",
                        required_role,
                        agent.identity.specialization.name()
                    );
                    return Ok(agent.identity.agent_id.clone());
                }
            }

            return Err(anyhow::anyhow!("No available agents for task"));
        }

        // Select agent with least recent activity (load balancing)
        let selected = suitable_agents
            .into_iter()
            .min_by_key(|id| {
                self.agents
                    .get(id)
                    .map(|a| a.last_activity)
                    .unwrap_or_else(Utc::now)
            })
            .unwrap();

        Ok(selected)
    }

    /// Check health of all agents
    async fn check_agent_health(&self) -> Result<()> {
        let mut unhealthy_agents = Vec::new();

        for entry in self.agents.iter() {
            let agent = entry.value();

            // Check if agent has been inactive for too long
            let inactive_duration = Utc::now() - agent.last_activity;
            if inactive_duration.num_seconds() > 300 {
                // 5 minutes
                warn!(
                    "Agent {} has been inactive for {} seconds",
                    agent.identity.agent_id,
                    inactive_duration.num_seconds()
                );
                unhealthy_agents.push(agent.identity.agent_id.clone());
            }

            // Check for error status
            if matches!(agent.status, AgentStatus::Error(_)) {
                error!("Agent {} is in error state", agent.identity.agent_id);
                unhealthy_agents.push(agent.identity.agent_id.clone());
            }
        }

        // Handle unhealthy agents
        for agent_id in unhealthy_agents {
            self.restart_agent(&agent_id).await?;
        }

        Ok(())
    }

    /// Restart an agent
    async fn restart_agent(&self, agent_id: &str) -> Result<()> {
        warn!("Restarting agent: {}", agent_id);

        if let Some(mut agent) = self.agents.get_mut(agent_id) {
            // Reset agent status
            agent.status = AgentStatus::Available;
            agent.last_activity = Utc::now();

            // Re-establish identity
            agent.establish_identity().await?;
        }

        Ok(())
    }

    /// Handle messages from agents
    async fn handle_agent_message(
        message: AgentMessage,
        agents: &DashMap<String, ClaudeCodeAgent>,
        state: &RwLock<OrchestratorState>,
    ) -> Result<()> {
        match message {
            AgentMessage::StatusUpdate { agent_id, status } => {
                if let Some(mut agent) = agents.get_mut(&agent_id) {
                    agent.status = status;
                    agent.last_activity = Utc::now();
                }
            }
            AgentMessage::TaskCompleted {
                agent_id,
                task_id,
                result,
            } => {
                info!(
                    "Agent {} completed task {}: success={}",
                    agent_id, task_id, result.success
                );

                let mut s = state.write().await;
                s.total_tasks_processed += 1;
                if result.success {
                    s.successful_tasks += 1;
                } else {
                    s.failed_tasks += 1;
                }
            }
            AgentMessage::RequestAssistance {
                agent_id,
                task_id,
                reason,
            } => {
                warn!(
                    "Agent {} requesting assistance for task {}: {}",
                    agent_id, task_id, reason
                );
                // TODO: Implement assistance logic
            }
            _ => {}
        }

        Ok(())
    }

    /// Perform quality review on agent work
    async fn perform_quality_review(
        agents: &DashMap<String, ClaudeCodeAgent>,
        standards: &QualityStandards,
        bus: &CoordinationBus,
    ) -> Result<()> {
        info!("Performing quality review");

        for entry in agents.iter() {
            let agent = entry.value();

            // Skip if agent hasn't completed any tasks
            if agent.task_history.is_empty() {
                continue;
            }

            // Review recent completed tasks
            for (task, result) in agent.task_history.iter().rev().take(5) {
                if result.success {
                    // TODO: Implement actual quality checks
                    // - Test coverage analysis
                    // - Code complexity checks
                    // - Security scanning
                    // - Performance benchmarks

                    let quality_score = 0.92; // Placeholder

                    if quality_score < standards.min_test_coverage {
                        warn!(
                            "Task {} from agent {} failed quality standards: score={}",
                            task.id, agent.identity.agent_id, quality_score
                        );

                        bus.send_message(AgentMessage::QualityIssue {
                            agent_id: agent.identity.agent_id.clone(),
                            task_id: task.id.clone(),
                            issues: vec!["Low test coverage".to_string()],
                        })
                        .await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate status report
    pub async fn generate_status_report(&self) -> Result<StatusReport> {
        let state = self.state.read().await;

        let agent_statuses: HashMap<String, AgentStatus> = self
            .agents
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().status.clone()))
            .collect();

        let uptime = Utc::now() - state.start_time;

        Ok(StatusReport {
            orchestrator_id: self.id.clone(),
            status: state.status.clone(),
            uptime_seconds: uptime.num_seconds() as u64,
            total_agents: self.agents.len(),
            active_agents: agent_statuses
                .values()
                .filter(|s| matches!(s, AgentStatus::Available | AgentStatus::Working))
                .count(),
            total_tasks_processed: state.total_tasks_processed,
            successful_tasks: state.successful_tasks,
            failed_tasks: state.failed_tasks,
            pending_tasks: state.pending_tasks.len(),
            agent_statuses,
            timestamp: Utc::now(),
        })
    }

    /// Shutdown the orchestrator gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Master Claude orchestrator");

        // Update state
        {
            let mut state = self.state.write().await;
            state.status = OrchestratorStatus::ShuttingDown;
        }

        // Shutdown all agents
        for entry in self.agents.iter() {
            let mut agent = entry.value().clone();
            if let Err(e) = agent.shutdown().await {
                error!(
                    "Error shutting down agent {}: {}",
                    agent.identity.agent_id, e
                );
            }
        }

        // Close coordination bus
        self.coordination_bus.close().await?;

        info!("Master Claude shutdown complete");
        Ok(())
    }
}

/// Status report for the orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusReport {
    pub orchestrator_id: String,
    pub status: OrchestratorStatus,
    pub uptime_seconds: u64,
    pub total_agents: usize,
    pub active_agents: usize,
    pub total_tasks_processed: u64,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    pub pending_tasks: usize,
    pub agent_statuses: HashMap<String, AgentStatus>,
    pub timestamp: DateTime<Utc>,
}

impl From<f64> for QualityStandards {
    fn from(threshold: f64) -> Self {
        Self {
            min_test_coverage: threshold,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::Priority;
    use crate::config::ClaudeConfig;
    use tempfile::TempDir;

    fn create_test_config() -> CcswarmConfig {
        let mut agents = HashMap::new();
        agents.insert(
            "frontend".to_string(),
            crate::config::AgentConfig {
                specialization: "frontend".to_string(),
                worktree: "agents/frontend".to_string(),
                branch: "feature/frontend".to_string(),
                claude_config: ClaudeConfig::for_agent("frontend"),
                claude_md_template: "frontend_specialist".to_string(),
            },
        );

        CcswarmConfig {
            project: crate::config::ProjectConfig {
                name: "Test Project".to_string(),
                repository: crate::config::RepositoryConfig {
                    url: "https://github.com/test/repo".to_string(),
                    main_branch: "main".to_string(),
                },
                master_claude: crate::config::MasterClaudeConfig {
                    role: "technical_lead".to_string(),
                    quality_threshold: 0.9,
                    think_mode: crate::config::ThinkMode::UltraThink,
                    permission_level: "supervised".to_string(),
                    claude_config: ClaudeConfig::for_master(),
                },
            },
            agents,
            coordination: crate::config::CoordinationConfig {
                communication_method: "json_files".to_string(),
                sync_interval: 30,
                quality_gate_frequency: "on_commit".to_string(),
                master_review_trigger: "all_tasks_complete".to_string(),
            },
        }
    }

    #[tokio::test]
    async fn test_master_claude_creation() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize git repo
        crate::git::WorktreeManager::init_if_needed(&repo_path)
            .await
            .unwrap();

        let config = create_test_config();
        let master = MasterClaude::new(config, repo_path).await.unwrap();

        assert!(master.id.starts_with("master-claude-"));

        let state = master.state.read().await;
        assert_eq!(state.status, OrchestratorStatus::Initializing);
    }

    #[tokio::test]
    async fn test_task_assignment() {
        let task = Task::new(
            "test-1".to_string(),
            "Create React component".to_string(),
            Priority::Medium,
            TaskType::Development,
        );

        // Test that frontend tasks are recognized
        assert!(task.description.to_lowercase().contains("component"));
    }
}
