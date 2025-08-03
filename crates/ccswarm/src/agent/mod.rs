pub mod backend_status;
pub mod claude;
pub mod interleaved_thinking;
pub mod isolation;
pub mod orchestrator;
pub mod persistent;
pub mod personality;
pub mod phronesis;
pub mod pool;
pub mod search_agent;
pub mod simple;
pub mod task;
pub mod task_builder;
pub mod whiteboard;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;
use uuid::Uuid;

pub use backend_status::{BackendStatus, BackendStatusExt};
pub use isolation::{IsolationConfig, IsolationMode};
pub use personality::{AgentPersonality, PersonalityTraits, Skill, TaskApproach, WorkingStyle};
pub use phronesis::{LearningEventType, PhronesisManager, PracticalWisdom, WisdomCategory};
pub use task::{Priority, Task, TaskResult, TaskType};
pub use task_builder::TaskBuilder;
pub use whiteboard::{AnnotationMarker, EntryType, Whiteboard, WhiteboardEntry};

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

    /// Isolation mode for this agent
    pub isolation_mode: IsolationMode,

    /// Container ID if running in container mode
    pub container_id: Option<String>,

    /// Agent's personality formed by skills and experiences
    pub personality: AgentPersonality,

    /// Whiteboard for thought visualization
    pub whiteboard: Whiteboard,

    /// Practical wisdom (phronesis) manager
    pub phronesis: PhronesisManager,
}

impl ClaudeCodeAgent {
    /// Create a new agent with the given configuration
    pub async fn new(
        role: AgentRole,
        workspace_root: &std::path::Path,
        branch_prefix: &str,
        claude_config: ClaudeConfig,
    ) -> Result<Self> {
        Self::new_with_isolation(
            role,
            workspace_root,
            branch_prefix,
            claude_config,
            IsolationMode::default(),
        )
        .await
    }

    /// Create a new agent with specific isolation mode
    pub async fn new_with_isolation(
        role: AgentRole,
        workspace_root: &std::path::Path,
        branch_prefix: &str,
        claude_config: ClaudeConfig,
        isolation_mode: IsolationMode,
    ) -> Result<Self> {
        let agent_id = format!("{}-agent-{}", role.name().to_lowercase(), Uuid::new_v4());
        let session_id = Uuid::new_v4().to_string();
        let worktree_path = workspace_root.join(format!("agents/{}", &agent_id));
        let branch_name = format!("{}/{}", branch_prefix, &agent_id);

        let identity = AgentIdentity {
            agent_id: agent_id.clone(),
            specialization: role.clone(),
            workspace_path: worktree_path.clone(),
            env_vars: Self::create_env_vars(&agent_id, &session_id),
            session_id,
            parent_process_id: std::process::id().to_string(),
            initialized_at: Utc::now(),
        };

        let personality = AgentPersonality::new(agent_id.clone(), &role);

        let whiteboard = Whiteboard::new(agent_id.clone());
        let phronesis = PhronesisManager::new(agent_id.clone());

        let agent = Self {
            identity,
            worktree_path,
            branch_name,
            claude_config,
            status: AgentStatus::Initializing,
            current_task: None,
            task_history: Vec::new(),
            last_activity: Utc::now(),
            isolation_mode,
            container_id: None,
            personality,
            whiteboard,
            phronesis,
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
        tracing::info!(
            "Initializing agent: {} with isolation mode: {:?}",
            self.identity.agent_id,
            self.isolation_mode
        );

        // Setup Git worktree (always needed for code synchronization)
        self.setup_worktree().await?;

        // If using container isolation, create and configure container
        if self.isolation_mode.requires_docker() {
            self.setup_container().await?;
        }

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
                self.worktree_path.to_str()
                    .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in worktree path"))?,
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
            AgentRole::Search { .. } => (
                "Verify search capabilities".to_string(),
                TaskType::Assistance,
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

    /// Setup container for agent execution
    async fn setup_container(&mut self) -> Result<()> {
        // Temporarily disabled due to container module compilation issues
        /*
        use crate::container::{ContainerConfig, ContainerProvider};

        tracing::info!("Setting up container for agent: {}", self.identity.agent_id);

        let provider = crate::container::docker::DockerContainerProvider::new().await?;

        // Create container configuration based on agent role
        let mut config = ContainerConfig::for_agent(
            self.identity.specialization.name(),
            &self.identity.agent_id,
        );

        // Add worktree as volume mount
        config.add_volume(
            self.worktree_path.to_string_lossy().to_string(),
            "/workspace".to_string(),
            false, // read-write
        );

        // Add environment variables
        for (key, value) in &self.identity.env_vars {
            config.env.insert(key.clone(), value.clone());
        }

        // Create and start container
        let container = provider
            .create_container(&format!("ccswarm-{}", self.identity.agent_id), &config)
            .await?;
        provider.start_container(&container.id).await?;

        // Install Claude CLI in container
        self.install_claude_in_container(&container.id, &provider)
            .await?;

        self.container_id = Some(container.id);

        tracing::info!(
            "Container setup complete for agent: {}",
            self.identity.agent_id
        );
        */

        // Placeholder implementation
        tracing::info!("Container setup skipped (disabled)");
        Ok(())
    }

    /// Install Claude CLI in the container  
    #[allow(dead_code)]
    async fn install_claude_in_container(&self, _container_id: &str, _provider: &()) -> Result<()> {
        // Temporarily disabled - container functionality not available
        tracing::info!("Claude CLI installation skipped (disabled)");
        Ok(())
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

                // Check if this is a complex task that needs orchestration
                if self.is_complex_task(&task) {
                    tracing::info!("Task identified as complex, using agent orchestrator");
                    self.execute_task_with_orchestration(task.clone()).await?
                } else {
                    self.execute_task_with_monitoring(task.clone()).await?
                }
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

                // Record rejection as a learning event
                self.phronesis.record_learning_event(
                    LearningEventType::Discovery {
                        finding: format!("Task outside boundaries: {}", reason),
                        significance: "Boundary protection activated".to_string(),
                    },
                    &format!("Task rejected for: {}", task.description),
                    std::collections::HashMap::new(),
                    crate::agent::phronesis::LearningOutcome {
                        lesson_learned: format!(
                            "This type of task should be handled by a different agent: {}",
                            reason
                        ),
                        actionable_insight: Some("Delegate to appropriate specialist".to_string()),
                        applicable_situations: vec!["Task delegation".to_string()],
                    },
                );

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

        // Create whiteboard section for this task
        let task_section_id = self
            .whiteboard
            .create_section(&format!("Task: {}", task.description));

        // Record initial task analysis on whiteboard
        let initial_note = self.whiteboard.add_note(
            &format!(
                "タスク開始: {}. タイプ: {:?}, 優先度: {:?}",
                task.description, task.task_type, task.priority
            ),
            vec!["task_start".to_string()],
        );
        self.whiteboard
            .add_to_section(&task_section_id, &initial_note);

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

        // Start a thought trace on whiteboard
        let thought_trace_id = self.whiteboard.start_thought_trace();
        self.whiteboard
            .add_to_section(&task_section_id, &thought_trace_id);

        loop {
            execution_count += 1;
            let output = self.execute_claude_command(&prompt).await?;

            // Record execution attempt on whiteboard
            let exec_note = self.whiteboard.add_note(
                &format!(
                    "実行試行 #{}: 出力長 {} 文字",
                    execution_count,
                    output.len()
                ),
                vec!["execution".to_string()],
            );
            self.whiteboard.add_to_section(&task_section_id, &exec_note);

            // Process the output through thinking engine
            let observation = self.extract_execution_observation(&output);
            let thinking_step = thinking_engine
                .process_observation(&observation, self.identity.specialization.name())
                .await?;

            // Record thinking on whiteboard
            self.whiteboard
                .add_thought(&thought_trace_id, &format!("観察: {}", observation));

            // Monitor the response for identity
            let identity_status = monitor.monitor_response(&output).await?;
            self.handle_identity_status(identity_status, &mut monitor)
                .await?;

            // Handle thinking decision
            match thinking_step.decision {
                Decision::Continue { reason } => {
                    tracing::debug!("Continuing execution: {}", reason);
                    self.whiteboard
                        .add_thought(&thought_trace_id, &format!("継続: {}", reason));
                    final_output = output;
                    if execution_count >= max_executions {
                        break;
                    }
                }
                Decision::Refine { refinement, reason } => {
                    tracing::info!("Refining approach: {} - {}", reason, refinement);
                    self.whiteboard.add_thought(
                        &thought_trace_id,
                        &format!("改善: {} - {}", reason, refinement),
                    );

                    // Record refinement as hypothesis
                    let hypothesis_id = self.whiteboard.add_hypothesis(&refinement, 0.7);
                    self.whiteboard
                        .add_to_section(&task_section_id, &hypothesis_id);

                    prompt = self.refine_prompt(&prompt, &refinement, &task);
                    final_output = output; // Keep last output
                }
                Decision::Complete { summary } => {
                    tracing::info!("Task completed: {}", summary);
                    self.whiteboard
                        .add_thought(&thought_trace_id, &format!("完了: {}", summary));
                    self.whiteboard.set_conclusion(&thought_trace_id, &summary);
                    final_output = output;
                    break;
                }
                Decision::Pivot {
                    new_approach,
                    reason,
                } => {
                    tracing::warn!("Pivoting approach: {} - {}", reason, new_approach);
                    self.whiteboard.add_thought(
                        &thought_trace_id,
                        &format!("方針転換: {} - {}", reason, new_approach),
                    );
                    self.whiteboard.annotate(
                        &thought_trace_id,
                        "大幅な方針変更",
                        AnnotationMarker::Important,
                    );
                    prompt = self.generate_pivot_prompt(&task, &new_approach);
                }
                Decision::RequestContext { questions } => {
                    tracing::info!("Additional context needed: {:?}", questions);
                    self.whiteboard
                        .add_thought(&thought_trace_id, &format!("追加情報必要: {:?}", questions));
                    // In a real implementation, this would request from orchestrator
                    // For now, we'll add questions to prompt and continue
                    prompt.push_str(&format!("\n\nPlease address: {}", questions.join(", ")));
                }
                Decision::Abort { reason } => {
                    self.whiteboard
                        .add_thought(&thought_trace_id, &format!("中断: {}", reason));
                    self.whiteboard.annotate(
                        &thought_trace_id,
                        "タスク中断",
                        AnnotationMarker::Important,
                    );
                    return Err(anyhow::anyhow!("Task aborted: {}", reason));
                }
            }

            if execution_count >= max_executions {
                break;
            }
        }

        // Generate thinking summary
        let thinking_summary = thinking_engine.get_thinking_summary();

        // Update agent's experience based on task completion
        self.update_agent_experience(&task);

        // Record success in phronesis system
        let lesson = format!(
            "Task completed in {} steps with {:.1}% confidence",
            thinking_summary.total_steps,
            thinking_summary.avg_confidence * 100.0
        );
        self.phronesis.record_success(
            &task.id,
            "Interleaved thinking with whiteboard",
            &format!("Task completed in {} iterations", execution_count),
            &lesson,
        );

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

    /// Check if a task is complex and needs orchestration
    fn is_complex_task(&self, task: &Task) -> bool {
        // A task is complex if it:
        // 1. Has multiple explicit steps in the description
        // 2. Contains keywords indicating multi-step work
        // 3. Has high priority and involves multiple components
        // 4. Is a feature or infrastructure task (typically multi-step)

        let description_lower = task.description.to_lowercase();
        let details_lower = task.details.as_deref().unwrap_or("").to_lowercase();
        let combined_text = format!("{} {}", description_lower, details_lower);

        // Check for multi-step indicators
        let multi_step_keywords = [
            "implement",
            "create",
            "build",
            "setup",
            "configure",
            "integrate",
            "migrate",
            "refactor",
            "design",
            "develop",
            "and then",
            "first",
            "next",
            "finally",
            "step",
        ];

        let has_multi_step_keywords = multi_step_keywords
            .iter()
            .any(|keyword| combined_text.contains(keyword));

        // Check for complexity indicators
        let complexity_indicators = combined_text.contains("multiple")
            || combined_text.contains("several")
            || combined_text.contains("comprehensive")
            || combined_text.contains("full")
            || combined_text.contains("complete");

        // Task type based complexity
        let complex_task_types = matches!(
            task.task_type,
            TaskType::Feature | TaskType::Infrastructure | TaskType::Development
        );

        // High priority tasks often need careful orchestration
        let high_priority = matches!(task.priority, Priority::High | Priority::Critical);

        // Decision logic
        (has_multi_step_keywords && complexity_indicators)
            || (complex_task_types && high_priority)
            || (complexity_indicators && complex_task_types)
    }

    /// Execute task with agent-level orchestration
    async fn execute_task_with_orchestration(&mut self, task: Task) -> Result<TaskResult> {
        use self::orchestrator::{AgentOrchestrator, OrchestrationBuilder, ParallelTask, TaskPlan};
        use async_trait::async_trait;

        tracing::info!(
            "Agent {} orchestrating complex task: {}",
            self.identity.agent_id,
            task.description
        );

        // Create an orchestrator instance for this agent
        struct AgentTaskOrchestrator<'a> {
            agent: &'a mut ClaudeCodeAgent,
        }

        #[async_trait]
        impl<'a> AgentOrchestrator for AgentTaskOrchestrator<'a> {
            async fn analyze_task(&self, task: &Task) -> Result<TaskPlan> {
                let mut plan = TaskPlan::new(task.id.clone());

                // Add context from master orchestration
                if let Some(parent_task_id) = &task.parent_task_id {
                    plan.update_context("parent_task".to_string(), parent_task_id.clone());
                }
                plan.update_context(
                    "agent_role".to_string(),
                    self.agent.identity.specialization.name().to_string(),
                );
                plan.update_context("task_type".to_string(), format!("{:?}", task.task_type));

                // Create analysis step based on agent specialization
                let analysis_tasks = match &self.agent.identity.specialization {
                    crate::identity::AgentRole::Frontend { .. } => vec![
                        OrchestrationBuilder::parallel_task(
                            "analyze_ui_requirements",
                            "Analyze UI Requirements",
                            "Identify UI components and user interactions needed",
                            true,
                        ),
                        OrchestrationBuilder::parallel_task(
                            "check_design_system",
                            "Check Design System",
                            "Review existing design patterns and components",
                            false,
                        ),
                    ],
                    crate::identity::AgentRole::Backend { .. } => vec![
                        OrchestrationBuilder::parallel_task(
                            "analyze_api_requirements",
                            "Analyze API Requirements",
                            "Identify API endpoints and data models needed",
                            true,
                        ),
                        OrchestrationBuilder::parallel_task(
                            "check_dependencies",
                            "Check Dependencies",
                            "Review existing services and dependencies",
                            true,
                        ),
                    ],
                    crate::identity::AgentRole::DevOps { .. } => vec![
                        OrchestrationBuilder::parallel_task(
                            "analyze_infrastructure",
                            "Analyze Infrastructure",
                            "Identify infrastructure requirements",
                            true,
                        ),
                        OrchestrationBuilder::parallel_task(
                            "check_deployment",
                            "Check Deployment",
                            "Review deployment pipeline and configurations",
                            true,
                        ),
                    ],
                    crate::identity::AgentRole::QA { .. } => vec![
                        OrchestrationBuilder::parallel_task(
                            "analyze_test_requirements",
                            "Analyze Test Requirements",
                            "Identify test scenarios and coverage needs",
                            true,
                        ),
                        OrchestrationBuilder::parallel_task(
                            "check_test_framework",
                            "Check Test Framework",
                            "Review existing test infrastructure",
                            false,
                        ),
                    ],
                    crate::identity::AgentRole::Master { .. } => {
                        vec![OrchestrationBuilder::parallel_task(
                            "analyze_overall_impact",
                            "Analyze Overall Impact",
                            "Assess cross-team dependencies and coordination needs",
                            true,
                        )]
                    }
                    crate::identity::AgentRole::Search { .. } => vec![
                        OrchestrationBuilder::parallel_task(
                            "analyze_search_query",
                            "Analyze Search Query",
                            "Parse and optimize the search query",
                            true,
                        ),
                        OrchestrationBuilder::parallel_task(
                            "identify_search_sources",
                            "Identify Search Sources",
                            "Determine best sources for information",
                            false,
                        ),
                    ],
                };

                let analysis_step = OrchestrationBuilder::analysis_step(
                    "step1_analysis",
                    "Initial Analysis",
                    analysis_tasks,
                );
                plan.add_step(analysis_step);

                // Add execution step
                let mut execution_step = OrchestrationBuilder::execution_step(
                    "step2_execution",
                    "Main Implementation",
                    vec!["step1_analysis"],
                );
                execution_step.add_parallel_task(OrchestrationBuilder::parallel_task(
                    "implement_solution",
                    "Implement Solution",
                    &task.description,
                    true,
                ));
                plan.add_step(execution_step);

                // Add validation step
                let mut validation_step = OrchestrationBuilder::validation_step(
                    "step3_validation",
                    "Validate Implementation",
                )
                .depends_on("step2_execution".to_string());
                validation_step.add_parallel_task(OrchestrationBuilder::parallel_task(
                    "validate_solution",
                    "Validate Solution",
                    "Ensure implementation meets requirements",
                    true,
                ));
                plan.add_step(validation_step);

                Ok(plan)
            }

            async fn execute_parallel_task(
                &self,
                task: &ParallelTask,
                context: &std::collections::HashMap<String, String>,
            ) -> Result<crate::agent::orchestrator::task_plan::ParallelTaskResult> {
                // Execute the task using agent's capabilities
                let prompt = format!(
                    "## Parallel Task: {}\n{}\n\nContext:\n{:?}",
                    task.name, task.command, context
                );

                let response = self.agent.execute_claude_command(&prompt).await?;

                Ok(crate::agent::orchestrator::task_plan::ParallelTaskResult {
                    task_id: task.id.clone(),
                    success: !response.contains("error") && !response.contains("failed"),
                    output: response,
                    error: None,
                })
            }

            async fn synthesize_results(
                &self,
                task: &Task,
                results: Vec<crate::agent::orchestrator::task_plan::StepResult>,
            ) -> Result<TaskResult> {
                let all_success = results.iter().all(|r| r.is_success());
                let total_duration: u64 = results.iter().map(|r| r.duration_ms).sum();

                // Create comprehensive output
                let mut output_sections = vec![];

                for (i, result) in results.iter().enumerate() {
                    output_sections.push(format!(
                        "Step {}: {} - {}",
                        i + 1,
                        if result.is_success() { "✓" } else { "✗" },
                        result.summary
                    ));
                }

                let final_output = format!(
                    "Orchestrated task execution complete:\n\n{}\n\nTotal steps: {}\nSuccessful: {}\nTotal duration: {}ms",
                    output_sections.join("\n"),
                    results.len(),
                    results.iter().filter(|r| r.is_success()).count(),
                    total_duration
                );

                if all_success {
                    Ok(TaskResult::success(
                        serde_json::json!({
                            "task_id": task.id,
                            "agent": self.agent.identity.agent_id,
                            "orchestrated": true,
                            "steps_completed": results.len(),
                            "output": final_output,
                        }),
                        std::time::Duration::from_millis(total_duration),
                    ))
                } else {
                    let errors: Vec<String> =
                        results.iter().flat_map(|r| &r.errors).cloned().collect();

                    Ok(TaskResult::failure(
                        format!("Orchestration failed: {}", errors.join(", ")),
                        std::time::Duration::from_millis(total_duration),
                    ))
                }
            }
        }

        // Create orchestrator and execute
        let orchestrator = AgentTaskOrchestrator { agent: self };
        orchestrator.orchestrate_task(&task).await
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
        &mut self,
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

                // Record drift as learning event
                self.phronesis.record_learning_event(
                    LearningEventType::Refinement {
                        original_approach: "Standard execution".to_string(),
                        improved_approach: "Identity boundary reinforcement".to_string(),
                    },
                    &format!("Identity drift: {}", msg),
                    std::collections::HashMap::new(),
                    crate::agent::phronesis::LearningOutcome {
                        lesson_learned: "Identity boundaries need reinforcement".to_string(),
                        actionable_insight: Some(
                            "Add stronger identity markers in prompts".to_string(),
                        ),
                        applicable_situations: vec!["Identity maintenance".to_string()],
                    },
                );

                self.correct_identity_drift(monitor).await
            }
            IdentityStatus::BoundaryViolation(msg) => {
                // Record boundary violation
                self.phronesis.record_failure(
                    "current_task",
                    "BoundaryViolation",
                    &msg,
                    "Task attempted to cross agent boundaries",
                );
                Err(anyhow::anyhow!("Boundary violation detected: {}", msg))
            }
            IdentityStatus::CriticalFailure(msg) => {
                // Record critical failure
                self.phronesis.record_failure(
                    "current_task",
                    "CriticalIdentityFailure",
                    &msg,
                    "Critical identity system failure requires investigation",
                );
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
        match self.isolation_mode {
            IsolationMode::Container => self.execute_claude_in_container(prompt).await,
            IsolationMode::GitWorktree => self.execute_claude_in_worktree(prompt).await,
            IsolationMode::Hybrid => {
                // Try container first, fall back to worktree
                match self.execute_claude_in_container(prompt).await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        tracing::warn!(
                            "Container execution failed, falling back to worktree: {}",
                            e
                        );
                        self.execute_claude_in_worktree(prompt).await
                    }
                }
            }
        }
    }

    /// Execute Claude Code command in git worktree
    async fn execute_claude_in_worktree(&self, prompt: &str) -> Result<String> {
        // Check if we should use real API instead of simulation
        if self.claude_config.use_real_api {
            return self.execute_claude_real_api(prompt).await;
        }

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

    /// Execute Claude using real API
    async fn execute_claude_real_api(&self, prompt: &str) -> Result<String> {
        use crate::providers::claude_api::ClaudeApiClient;

        tracing::info!(
            "Using real Claude API for agent: {}",
            self.identity.agent_id
        );

        // Create API client
        let api_client = ClaudeApiClient::new(None)?;

        // Format the full prompt with agent identity
        let full_prompt = format!(
            "{}\n\n{}",
            claude::generate_identity_header(&self.identity),
            prompt
        );

        // Make API call
        let response = api_client
            .simple_completion(
                &self.claude_config.model,
                &full_prompt,
                4096, // Max tokens
            )
            .await?;

        // Format response to match expected output format
        if self.claude_config.json_output {
            // Wrap response in JSON format similar to CLI output
            let json_response = serde_json::json!({
                "response": response,
                "agent": self.identity.agent_id,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });
            Ok(serde_json::to_string(&json_response)?)
        } else {
            Ok(response)
        }
    }

    /// Execute Claude Code command in container
    async fn execute_claude_in_container(&self, prompt: &str) -> Result<String> {
        // Temporarily disabled - container functionality not available
        // Fall back to real API implementation
        self.execute_claude_real_api(prompt).await
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
            "personality": {
                "description": self.personality.describe_personality(),
                "working_style": self.personality.working_style,
                "traits": self.personality.traits,
                "composability_score": self.personality.composability_score(),
                "skills": self.personality.skills.iter().map(|(name, skill)| {
                    serde_json::json!({
                        "name": name,
                        "category": "general",
                        "level": skill.level,
                        "experience": skill.experience_points,
                    })
                }).collect::<Vec<_>>(),
            },
            "whiteboard_summary": self.whiteboard.summarize(),
            "phronesis_summary": self.phronesis.summarize(),
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

    /// Update agent's experience based on completed task
    pub fn update_agent_experience(&mut self, task: &Task) {
        // タスクタイプに基づいて経験値を付与
        let experience_points = match task.priority {
            Priority::Critical => 100,
            Priority::High => 50,
            Priority::Medium => 30,
            Priority::Low => 10,
        };

        // タスクタイプに関連するスキルを特定して経験値を追加
        // Simplified: give experience to all skills, with some variation by task type
        let experience_multiplier = match task.task_type {
            TaskType::Development | TaskType::Feature | TaskType::Infrastructure => 1.0,
            TaskType::Testing | TaskType::Review => 0.8,
            TaskType::Documentation => 0.6,
            TaskType::Coordination => 0.7,
            TaskType::Bugfix | TaskType::Bug => 1.2,
            TaskType::Remediation => 1.5,
            TaskType::Assistance => 0.9,
            TaskType::Research => 0.7,
        };

        let adjusted_experience = (experience_points as f32 * experience_multiplier) as u32;

        for skill in self.personality.skills.values_mut() {
            skill.add_experience(adjusted_experience);
            tracing::debug!(
                "Added {} experience to skill: {}",
                adjusted_experience,
                skill.name
            );
        }

        // 個性の説明を更新してログ出力
        tracing::info!(
            "Agent {} personality update: {}",
            self.identity.agent_id,
            self.personality.describe_personality()
        );
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

        // Clean up container if using container isolation
        if let Some(container_id) = &self.container_id {
            tracing::info!("Cleaning up container: {}", container_id);
            match self.cleanup_container(container_id).await {
                Ok(_) => tracing::info!("Container cleanup complete"),
                Err(e) => tracing::warn!("Container cleanup failed: {}", e),
            }
        }

        Ok(())
    }

    /// Clean up container resources
    async fn cleanup_container(&self, _container_id: &str) -> Result<()> {
        // Temporarily disabled - container functionality not available
        tracing::info!("Container cleanup skipped (disabled)");
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
