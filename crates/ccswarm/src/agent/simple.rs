use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

use crate::agent::{AgentStatus, Task, TaskResult};
use crate::config::ClaudeConfig;
use crate::identity::{AgentIdentity, AgentRole};
use crate::workspace::SimpleWorkspaceManager;

// Orchestrator implementation in separate file
#[path = "simple_orchestrator.rs"]
mod simple_orchestrator;

/// Simple agent without Git integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleClaudeAgent {
    /// Agent identity
    pub identity: AgentIdentity,

    /// Workspace path
    pub workspace_path: PathBuf,

    /// Claude configuration
    pub claude_config: ClaudeConfig,

    /// Current status
    pub status: AgentStatus,

    /// Current task
    pub current_task: Option<Task>,

    /// Task history
    pub task_history: Vec<(Task, TaskResult)>,

    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
}

impl SimpleClaudeAgent {
    /// Create a new simple agent
    pub async fn new(
        role: AgentRole,
        workspace_root: &std::path::Path,
        claude_config: ClaudeConfig,
    ) -> Result<Self> {
        let agent_id = format!("{}-agent-{}", role.name().to_lowercase(), Uuid::new_v4());
        let session_id = Uuid::new_v4().to_string();
        let workspace_path = workspace_root
            .parent()
            .map(|p| p.join("worktrees").join(&agent_id))
            .unwrap_or_else(|| workspace_root.join(".worktrees").join(&agent_id));

        let identity = AgentIdentity {
            agent_id: agent_id.clone(),
            specialization: role,
            workspace_path: workspace_path.clone(),
            env_vars: Self::create_env_vars(&agent_id, &session_id),
            session_id,
            parent_process_id: std::process::id().to_string(),
            initialized_at: Utc::now(),
        };

        let agent = Self {
            identity,
            workspace_path,
            claude_config,
            status: AgentStatus::Initializing,
            current_task: None,
            task_history: Vec::new(),
            last_activity: Utc::now(),
        };

        Ok(agent)
    }

    /// Create environment variables
    fn create_env_vars(
        agent_id: &str,
        session_id: &str,
    ) -> std::collections::HashMap<String, String> {
        let mut env_vars = std::collections::HashMap::new();
        env_vars.insert("CCSWARM_AGENT_ID".to_string(), agent_id.to_string());
        env_vars.insert("CCSWARM_SESSION_ID".to_string(), session_id.to_string());
        env_vars.insert(
            "CCSWARM_ROLE".to_string(),
            agent_id.split('-').next().unwrap_or("unknown").to_string(),
        );
        env_vars
    }

    /// Initialize the agent
    pub async fn initialize(&mut self, workspace_manager: &SimpleWorkspaceManager) -> Result<()> {
        tracing::info!("Initializing simple agent: {}", self.identity.agent_id);

        // Create workspace
        workspace_manager
            .create_workspace(&self.identity.agent_id)
            .await?;

        // Generate CLAUDE.md
        let claude_md_content = crate::agent::claude::generate_claude_md_content(&self.identity);
        workspace_manager
            .setup_claude_config(&self.identity.agent_id, &claude_md_content)
            .await?;

        self.status = AgentStatus::Available;
        self.last_activity = Utc::now();

        tracing::info!(
            "Simple agent {} initialized successfully",
            self.identity.agent_id
        );
        Ok(())
    }

    /// Execute a task
    pub async fn execute_task(&mut self, task: Task) -> Result<TaskResult> {
        tracing::info!(
            "Agent {} executing task: {}",
            self.identity.agent_id,
            task.id
        );

        self.status = AgentStatus::Working;
        self.current_task = Some(task.clone());
        self.last_activity = Utc::now();

        let start_time = std::time::Instant::now();

        // Simulated task execution
        let result = self.simulate_task_execution(&task).await?;

        let duration = start_time.elapsed();
        let task_result = if result.is_success {
            TaskResult::success(
                serde_json::json!({
                    "agent_id": self.identity.agent_id,
                    "task_id": task.id,
                    "result": result.output,
                    "notes": result.notes
                }),
                duration,
            )
        } else {
            TaskResult::failure(result.error_message, duration)
        };

        // Add to task history
        self.task_history.push((task, task_result.clone()));
        self.current_task = None;
        self.status = AgentStatus::Available;
        self.last_activity = Utc::now();

        Ok(task_result)
    }

    /// Simulate task execution
    async fn simulate_task_execution(&self, task: &Task) -> Result<SimulatedResult> {
        // Evaluate task based on agent specialization
        let can_handle = self.can_handle_task(task);

        if !can_handle {
            return Ok(SimulatedResult {
                is_success: false,
                error_message: format!(
                    "Task '{}' is outside my specialization as a {} agent",
                    task.description,
                    self.identity.specialization.name()
                ),
                output: String::new(),
                notes: vec![
                    "Task delegation recommended".to_string(),
                    format!("Should be handled by appropriate specialist agent"),
                ],
            });
        }

        // Simulate success
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(SimulatedResult {
            is_success: true,
            error_message: String::new(),
            output: format!("Successfully completed: {}", task.description),
            notes: vec![
                format!("Processed by {} agent", self.identity.specialization.name()),
                "Task completed within agent boundaries".to_string(),
            ],
        })
    }

    /// Check if the task can be handled by this agent
    fn can_handle_task(&self, task: &Task) -> bool {
        let role_name = self.identity.specialization.name().to_lowercase();
        let description = task.description.to_lowercase();
        let task_id = task.id.to_lowercase();

        match role_name.as_str() {
            "frontend" => {
                task_id.contains("frontend")
                    || description.contains("html")
                    || description.contains("css")
                    || description.contains("javascript")
                    || description.contains("ui")
                    || description.contains("react")
                    || description.contains("component")
                    || description.contains("frontend")
            }
            "backend" => {
                task_id.contains("backend")
                    || description.contains("node")
                    || description.contains("express")
                    || description.contains("package.json")
                    || description.contains("api")
                    || description.contains("server")
                    || description.contains("database")
                    || description.contains("backend")
            }
            "devops" => {
                task_id.contains("deploy")
                    || description.contains("script")
                    || description.contains("readme")
                    || description.contains("documentation")
                    || description.contains("deploy")
                    || description.contains("infrastructure")
                    || description.contains("ci/cd")
                    || description.contains("docker")
            }
            "qa" => {
                description.contains("test")
                    || description.contains("quality")
                    || description.contains("bug")
                    || description.contains("validation")
            }
            _ => true, // Master agent can handle all tasks
        }
    }

    /// Update status
    pub fn update_status(&mut self, status: AgentStatus) {
        self.status = status;
        self.last_activity = Utc::now();
    }
}

/// Simulated task execution result
#[derive(Debug)]
struct SimulatedResult {
    is_success: bool,
    error_message: String,
    output: String,
    notes: Vec<String>,
}
