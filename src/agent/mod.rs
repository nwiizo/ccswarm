pub mod claude;
pub mod interleaved_thinking;
pub mod persistent;
pub mod pool;
pub mod simple;
pub mod task;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;
use uuid::Uuid;

pub use task::{Priority, Task, TaskResult, TaskType};

use self::interleaved_thinking::{Decision, InterleavedThinkingEngine};
use crate::config::ClaudeConfig;
use crate::identity::boundary::TaskBoundaryChecker;
use crate::identity::boundary::TaskEvaluation;
use crate::identity::{AgentIdentity, AgentRole, IdentityMonitor, IdentityStatus};

/// Current status of an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    /// Agent is being initialized
    Initializing,
    /// Agent is available for tasks
    Available,
    /// Agent is currently working on a task
    Working,
    /// Agent has completed work and is waiting for review
    WaitingForReview,
    /// Agent encountered an error
    Error(String),
    /// Agent is shutting down
    ShuttingDown,
}

/// Core Claude Code agent structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeAgent {
    /// Agent identity information
    pub identity: AgentIdentity,

    /// Git worktree path
    pub worktree_path: PathBuf,

    /// Git branch name
    pub branch_name: String,

    /// Claude configuration
    pub claude_config: ClaudeConfig,

    /// Current agent status
    pub status: AgentStatus,

    /// Current task being worked on
    pub current_task: Option<Task>,

    /// Task history
    pub task_history: Vec<(Task, TaskResult)>,

    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
}

impl ClaudeCodeAgent {
    /// Create a new agent with the given configuration
    pub async fn new(
        role: AgentRole,
        workspace_root: &std::path::Path,
        branch_prefix: &str,
        claude_config: ClaudeConfig,
    ) -> Result<Self> {
        let agent_id = format!("{}-agent-{}", role.name().to_lowercase(), Uuid::new_v4());
        let session_id = Uuid::new_v4().to_string();
        let worktree_path = workspace_root.join(format!("agents/{}", &agent_id));
        let branch_name = format!("{}/{}", branch_prefix, &agent_id);

        let identity = AgentIdentity {
            agent_id: agent_id.clone(),
            specialization: role,
            workspace_path: worktree_path.clone(),
            env_vars: Self::create_env_vars(&agent_id, &session_id),
            session_id,
            parent_process_id: std::process::id().to_string(),
            initialized_at: Utc::now(),
        };

        let agent = Self {
            identity,
            worktree_path,
            branch_name,
            claude_config,
            status: AgentStatus::Initializing,
            current_task: None,
            task_history: Vec::new(),
            last_activity: Utc::now(),
        };

        Ok(agent)
    }

    /// Create environment variables for agent identity
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

    /// Initialize the agent (setup worktree, identity, etc.)
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing agent: {}", self.identity.agent_id);

        // Setup Git worktree
        self.setup_worktree().await?;

        // Generate and write CLAUDE.md
        self.generate_claude_md().await?;

        // Setup Claude configuration
        self.setup_claude_config().await?;

        // Establish identity
        self.establish_identity().await?;

        // Run boundary verification
        self.verify_boundaries().await?;

        self.status = AgentStatus::Available;
        self.last_activity = Utc::now();

        tracing::info!("Agent {} initialized successfully", self.identity.agent_id);
        Ok(())
    }

    /// Setup Git worktree for this agent
    async fn setup_worktree(&self) -> Result<()> {
        // Create parent directory if needed
        if let Some(parent) = self.worktree_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .context("Failed to create parent directory")?;
        }

        // Create branch
        let output = Command::new("git")
            .args(["checkout", "-b", &self.branch_name])
            .output()
            .await
            .context("Failed to create branch")?;

        if !output.status.success()
            && !String::from_utf8_lossy(&output.stderr).contains("already exists")
        {
            return Err(anyhow::anyhow!(
                "Failed to create branch: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Create worktree
        let output = Command::new("git")
            .args([
                "worktree",
                "add",
                self.worktree_path.to_str().unwrap(),
                &self.branch_name,
            ])
            .output()
            .await
            .context("Failed to create worktree")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to create worktree: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        tracing::info!("Git worktree created at: {}", self.worktree_path.display());
        Ok(())
    }

    /// Generate CLAUDE.md for this agent
    async fn generate_claude_md(&self) -> Result<()> {
        let content = claude::generate_claude_md_content(&self.identity);
        let claude_md_path = self.worktree_path.join("CLAUDE.md");

        tokio::fs::write(&claude_md_path, content)
            .await
            .context("Failed to write CLAUDE.md")?;

        tracing::info!("CLAUDE.md generated for agent: {}", self.identity.agent_id);
        Ok(())
    }

    /// Setup Claude configuration (.claude.json)
    async fn setup_claude_config(&self) -> Result<()> {
        let config_path = self.worktree_path.join(".claude.json");
        let config_json = serde_json::to_string_pretty(&self.claude_config)?;

        tokio::fs::write(&config_path, config_json)
            .await
            .context("Failed to write .claude.json")?;

        tracing::info!(".claude.json created for agent: {}", self.identity.agent_id);
        Ok(())
    }

    /// Establish agent identity with Claude Code
    pub async fn establish_identity(&mut self) -> Result<()> {
        let prompt = claude::generate_identity_establishment_prompt(&self.identity);

        // Execute identity establishment
        let response = self.execute_claude_command(&prompt).await?;

        // Verify response contains required identity markers
        if !claude::verify_identity_response(&response, &self.identity) {
            return Err(anyhow::anyhow!("Failed to establish agent identity"));
        }

        tracing::info!("Identity established for agent: {}", self.identity.agent_id);
        Ok(())
    }

    /// Verify agent boundaries are properly set
    async fn verify_boundaries(&self) -> Result<()> {
        // Test with a task that should be accepted
        let test_task = self.create_test_task_for_role();
        let checker = TaskBoundaryChecker::new(self.identity.specialization.clone());

        match checker.evaluate_task(&test_task).await {
            TaskEvaluation::Accept { .. } => {
                tracing::info!(
                    "Boundary verification passed for agent: {}",
                    self.identity.agent_id
                );
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Boundary verification failed")),
        }
    }

    /// Create a test task appropriate for this agent's role
    fn create_test_task_for_role(&self) -> Task {
        let (description, task_type) = match &self.identity.specialization {
            AgentRole::Frontend { .. } => (
                "Verify React component rendering".to_string(),
                TaskType::Development,
            ),
            AgentRole::Backend { .. } => (
                "Verify API endpoint functionality".to_string(),
                TaskType::Development,
            ),
            AgentRole::DevOps { .. } => (
                "Verify Docker container build".to_string(),
                TaskType::Infrastructure,
            ),
            AgentRole::QA { .. } => ("Verify test suite execution".to_string(), TaskType::Testing),
            AgentRole::Master { .. } => (
                "Verify coordination capabilities".to_string(),
                TaskType::Coordination,
            ),
        };

        Task::new(
            Uuid::new_v4().to_string(),
            description,
            Priority::Low,
            task_type,
        )
        .with_details("Boundary verification test".to_string())
    }

    /// Execute a task with full identity and boundary checking
    pub async fn execute_task(&mut self, task: Task) -> Result<TaskResult> {
        self.status = AgentStatus::Working;
        self.current_task = Some(task.clone());
        self.last_activity = Utc::now();

        // Evaluate task boundaries
        let checker = TaskBoundaryChecker::new(self.identity.specialization.clone());
        let evaluation = checker.evaluate_task(&task).await;

        let result = match evaluation {
            TaskEvaluation::Accept { reason } => {
                tracing::info!("Task accepted by {}: {}", self.identity.agent_id, reason);
                self.execute_task_with_monitoring(task.clone()).await?
            }
            TaskEvaluation::Delegate {
                target_agent,
                suggestion,
                ..
            } => {
                tracing::info!(
                    "Task delegated by {} to {}",
                    self.identity.agent_id,
                    target_agent
                );
                TaskResult {
                    success: false,
                    output: serde_json::json!({
                        "action": "delegated",
                        "target": target_agent,
                        "suggestion": suggestion,
                    }),
                    error: None,
                    duration: std::time::Duration::from_secs(0),
                }
            }
            TaskEvaluation::Clarify { questions, .. } => {
                tracing::info!(
                    "Task requires clarification from {}",
                    self.identity.agent_id
                );
                TaskResult {
                    success: false,
                    output: serde_json::json!({
                        "action": "clarification_needed",
                        "questions": questions,
                    }),
                    error: None,
                    duration: std::time::Duration::from_secs(0),
                }
            }
            TaskEvaluation::Reject { reason } => {
                tracing::warn!("Task rejected by {}: {}", self.identity.agent_id, reason);
                TaskResult {
                    success: false,
                    output: serde_json::json!({}),
                    error: Some(format!("Task rejected: {}", reason)),
                    duration: std::time::Duration::from_secs(0),
                }
            }
        };

        // Update status and history
        self.status = if result.success {
            AgentStatus::WaitingForReview
        } else {
            AgentStatus::Available
        };
        self.current_task = None;
        self.task_history.push((task, result.clone()));

        // Report status
        self.report_status(&result).await?;

        Ok(result)
    }

    /// Execute task with identity monitoring and interleaved thinking
    async fn execute_task_with_monitoring(&mut self, task: Task) -> Result<TaskResult> {
        let start_time = std::time::Instant::now();
        let mut monitor = IdentityMonitor::new(&self.identity.agent_id);
        let mut thinking_engine = InterleavedThinkingEngine::new().with_config(15, 0.6); // Max 15 steps, 0.6 confidence threshold

        // Initial thinking step - analyze the task
        let initial_observation = format!(
            "Starting task: {}. Type: {:?}, Priority: {:?}",
            task.description, task.task_type, task.priority
        );
        let initial_step = thinking_engine
            .process_observation(&initial_observation, self.identity.specialization.name())
            .await?;

        // Prepare task prompt with identity header
        let mut prompt = claude::generate_task_prompt(&self.identity, &task);

        // Add thinking context if we need clarification
        if let Decision::RequestContext { questions } = &initial_step.decision {
            prompt.push_str(&format!(
                "\n\nPlease consider these aspects: {}",
                questions.join(", ")
            ));
        }

        // Execute Claude Code with progressive refinement
        let mut final_output = String::new();
        let mut execution_count = 0;
        let max_executions = 3;

        loop {
            execution_count += 1;
            let output = self.execute_claude_command(&prompt).await?;

            // Process the output through thinking engine
            let observation = self.extract_execution_observation(&output);
            let thinking_step = thinking_engine
                .process_observation(&observation, self.identity.specialization.name())
                .await?;

            // Monitor the response for identity
            let identity_status = monitor.monitor_response(&output).await?;
            self.handle_identity_status(identity_status, &mut monitor)
                .await?;

            // Handle thinking decision
            match thinking_step.decision {
                Decision::Continue { reason } => {
                    tracing::debug!("Continuing execution: {}", reason);
                    final_output = output;
                    if execution_count >= max_executions {
                        break;
                    }
                }
                Decision::Refine { refinement, reason } => {
                    tracing::info!("Refining approach: {} - {}", reason, refinement);
                    prompt = self.refine_prompt(&prompt, &refinement, &task);
                    final_output = output; // Keep last output
                }
                Decision::Complete { summary } => {
                    tracing::info!("Task completed: {}", summary);
                    final_output = output;
                    break;
                }
                Decision::Pivot {
                    new_approach,
                    reason,
                } => {
                    tracing::warn!("Pivoting approach: {} - {}", reason, new_approach);
                    prompt = self.generate_pivot_prompt(&task, &new_approach);
                }
                Decision::RequestContext { questions } => {
                    tracing::info!("Additional context needed: {:?}", questions);
                    // In a real implementation, this would request from orchestrator
                    // For now, we'll add questions to prompt and continue
                    prompt.push_str(&format!("\n\nPlease address: {}", questions.join(", ")));
                }
                Decision::Abort { reason } => {
                    return Err(anyhow::anyhow!("Task aborted: {}", reason));
                }
            }

            if execution_count >= max_executions {
                break;
            }
        }

        // Generate thinking summary
        let thinking_summary = thinking_engine.get_thinking_summary();

        Ok(TaskResult {
            success: true,
            output: serde_json::json!({
                "response": final_output,
                "agent": self.identity.agent_id,
                "task_id": task.id,
                "thinking_summary": thinking_summary,
                "execution_iterations": execution_count,
            }),
            error: None,
            duration: start_time.elapsed(),
        })
    }

    /// Extract observation from execution output
    fn extract_execution_observation(&self, output: &str) -> String {
        // Look for key indicators in output
        if output.contains("error") || output.contains("Error") {
            format!(
                "Execution encountered errors: {}",
                output
                    .lines()
                    .find(|l| l.contains("error"))
                    .unwrap_or("unknown error")
            )
        } else if output.contains("success") || output.contains("completed") {
            "Execution completed successfully".to_string()
        } else if output.contains("created") || output.contains("generated") {
            "New artifacts generated".to_string()
        } else {
            format!("Execution output: {} characters", output.len())
        }
    }

    /// Handle identity status from monitoring
    async fn handle_identity_status(
        &self,
        status: IdentityStatus,
        monitor: &mut IdentityMonitor,
    ) -> Result<()> {
        match status {
            IdentityStatus::Healthy => {
                tracing::debug!("Identity maintained during task execution");
                Ok(())
            }
            IdentityStatus::DriftDetected(msg) => {
                tracing::warn!("Identity drift detected: {}", msg);
                self.correct_identity_drift(monitor).await
            }
            IdentityStatus::BoundaryViolation(msg) => {
                Err(anyhow::anyhow!("Boundary violation detected: {}", msg))
            }
            IdentityStatus::CriticalFailure(msg) => {
                Err(anyhow::anyhow!("Critical identity failure: {}", msg))
            }
        }
    }

    /// Refine prompt based on thinking engine feedback
    fn refine_prompt(&self, original_prompt: &str, refinement: &str, task: &Task) -> String {
        format!(
            "{}\n\n## Refinement\n{}\n\nPlease apply this refinement while maintaining focus on: {}",
            original_prompt, refinement, task.description
        )
    }

    /// Generate pivot prompt for new approach
    fn generate_pivot_prompt(&self, task: &Task, new_approach: &str) -> String {
        format!(
            "{}\n\n## New Approach\n{}\n\nTask: {}\nType: {:?}\nPriority: {:?}",
            claude::generate_identity_header(&self.identity),
            new_approach,
            task.description,
            task.task_type,
            task.priority
        )
    }

    /// Execute Claude Code command
    async fn execute_claude_command(&self, prompt: &str) -> Result<String> {
        let mut cmd = Command::new("claude");

        // Set working directory
        cmd.current_dir(&self.worktree_path);

        // Add environment variables
        for (key, value) in &self.identity.env_vars {
            cmd.env(key, value);
        }

        // Add Claude arguments
        cmd.arg("-p").arg(prompt);

        if self.claude_config.json_output {
            cmd.arg("--json");
        }

        if self.claude_config.dangerous_skip {
            cmd.arg("--dangerously-skip-permissions");
        }

        // Add think mode if specified
        if let Some(think_mode) = &self.claude_config.think_mode {
            cmd.arg("--think").arg(think_mode.to_string());
        }

        // Execute command
        let output = cmd
            .output()
            .await
            .context("Failed to execute Claude Code")?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "Claude Code execution failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Correct identity drift
    async fn correct_identity_drift(&self, monitor: &mut IdentityMonitor) -> Result<()> {
        let correction_prompt = monitor.generate_correction_prompt(
            &self.worktree_path.to_string_lossy(),
            self.identity.specialization.name(),
        );

        let _ = self.execute_claude_command(&correction_prompt).await?;
        tracing::info!(
            "Identity drift correction applied for agent: {}",
            self.identity.agent_id
        );
        Ok(())
    }

    /// Report agent status to coordination system
    async fn report_status(&self, result: &TaskResult) -> Result<()> {
        let status_report = serde_json::json!({
            "agent_id": self.identity.agent_id,
            "specialization": self.identity.specialization.name(),
            "status": self.status,
            "current_task": self.current_task,
            "last_result": result,
            "timestamp": Utc::now(),
            "worktree": self.worktree_path.to_string_lossy(),
            "branch": self.branch_name,
        });

        // Write to coordination directory
        let status_file = PathBuf::from("coordination/agent-status")
            .join(format!("{}.json", self.identity.agent_id));

        if let Some(parent) = status_file.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&status_file, serde_json::to_string_pretty(&status_report)?).await?;

        Ok(())
    }

    /// Shutdown the agent gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down agent: {}", self.identity.agent_id);
        self.status = AgentStatus::ShuttingDown;

        // Final status report
        self.report_status(&TaskResult {
            success: true,
            output: serde_json::json!({"action": "shutdown"}),
            error: None,
            duration: std::time::Duration::from_secs(0),
        })
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::default_frontend_role;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_agent_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = temp_dir.path().to_path_buf();

        let config = ClaudeConfig::default();
        let agent = ClaudeCodeAgent::new(default_frontend_role(), &workspace, "feature", config)
            .await
            .unwrap();

        assert!(agent.identity.agent_id.starts_with("frontend-agent-"));
        assert_eq!(agent.status, AgentStatus::Initializing);
    }
}

#[cfg(test)]
mod interleaved_thinking_test;
