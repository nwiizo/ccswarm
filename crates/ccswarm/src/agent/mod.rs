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
pub mod type_state;
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

use self::interleaved_thinking::{DecisionType, ThinkingStep, InterleavedThinkingEngine};
use crate::config::ClaudeConfig;
use crate::identity::boundary::TaskBoundaryChecker;
use crate::identity::boundary::TaskEvaluation;
use crate::identity::{AgentIdentity, AgentRole, IdentityMonitor, IdentityStatus};

/// Current status of an agent in its operational lifecycle
///
/// The agent status represents the current operational state and determines
/// what operations are available. Status transitions follow a defined state machine:
///
/// ```text
/// Initializing ──┐
///                ├─► Available ◄─┬─► Working ──► WaitingForReview
///                │               │                      │
///                ▼               │                      ▼
///              Error ◄───────────┴──────────────────► Available
///                │
///                ▼
///         ShuttingDown
/// ```
///
/// # State Descriptions
///
/// - **Initializing**: Agent is setting up resources and establishing identity
/// - **Available**: Agent is ready to accept new tasks
/// - **Working**: Agent is actively executing a task
/// - **WaitingForReview**: Task completed, waiting for quality review
/// - **Error**: Agent encountered an error and may need intervention
/// - **ShuttingDown**: Agent is performing cleanup and shutting down
///
/// # Examples
///
/// ```rust
/// use ccswarm::agent::AgentStatus;
///
/// let status = AgentStatus::Available;
/// match status {
///     AgentStatus::Available => println!("Agent ready for tasks"),
///     AgentStatus::Working => println!("Agent busy"),
///     AgentStatus::Error(msg) => println!("Agent error: {}", msg),
///     _ => println!("Agent in transition"),
/// }
/// ```
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
///
/// This is the primary agent implementation that interfaces with Claude Code
/// via the ACP (Agent Client Protocol). Each agent has its own identity,
/// workspace isolation, and specialized capabilities.
///
/// # Agent Lifecycle
///
/// 1. **Creation**: Agent is created with a specific role and workspace
/// 2. **Initialization**: Git worktree is set up, identity is established
/// 3. **Operation**: Agent executes tasks within its boundaries
/// 4. **Monitoring**: Continuous health and performance monitoring
/// 5. **Shutdown**: Graceful cleanup of resources
///
/// # Architecture
///
/// ```text
/// ┌─────────────────────────────────────┐
/// │           Claude Agent              │
/// ├─────────────────────────────────────┤
/// │  Identity & Boundary Management     │
/// │  ├─ Agent ID & Role                 │
/// │  ├─ Task Boundary Checking          │
/// │  └─ Identity Drift Monitoring       │
/// ├─────────────────────────────────────┤
/// │  Workspace Isolation                │
/// │  ├─ Git Worktree Management         │
/// │  ├─ Container Isolation (optional)  │
/// │  └─ Environment Variables           │
/// ├─────────────────────────────────────┤
/// │  Task Execution Engine              │
/// │  ├─ Interleaved Thinking            │
/// │  ├─ Agent Orchestration             │
/// │  └─ Claude Code Integration         │
/// ├─────────────────────────────────────┤
/// │  Learning & Adaptation              │
/// │  ├─ Personality Development         │
/// │  ├─ Phronesis (Practical Wisdom)    │
/// │  └─ Whiteboard Visualization        │
/// └─────────────────────────────────────┘
/// ```
///
/// # Examples
///
/// ## Creating a Frontend Agent
///
/// ```rust
/// use ccswarm::agent::ClaudeCodeAgent;
/// use ccswarm::identity::AgentRole;
/// use ccswarm::config::ClaudeConfig;
/// use std::path::Path;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let workspace = Path::new("/workspace");
/// let config = ClaudeConfig::default();
/// let role = AgentRole::Frontend {
///     technologies: vec!["React".to_string(), "TypeScript".to_string()],
///     responsibilities: vec!["UI Development".to_string()],
///     boundaries: vec!["Frontend only".to_string()],
/// };
///
/// let mut agent = ClaudeCodeAgent::new(
///     role,
///     workspace,
///     "feature",
///     config,
/// ).await?;
///
/// agent.initialize().await?;
/// # Ok(())
/// # }
/// ```
///
/// ## Task Execution with Boundary Checking
///
/// ```rust
/// use ccswarm::agent::{Task, Priority, TaskType};
///
/// # async fn example(mut agent: ccswarm::agent::ClaudeCodeAgent) -> Result<(), Box<dyn std::error::Error>> {
/// let task = Task::new(
///     "task-1".to_string(),
///     "Create a React component for user authentication".to_string(),
///     Priority::High,
///     TaskType::Development,
/// );
///
/// let result = agent.execute_task(task).await?;
/// if result.success {
///     println!("Task completed successfully");
/// }
/// # Ok(())
/// # }
/// ```
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
    ///
    /// # Arguments
    ///
    /// * `role` - The specialized role for this agent (Frontend, Backend, DevOps, etc.)
    /// * `workspace_root` - Root directory for agent workspaces
    /// * `branch_prefix` - Prefix for git branches created by this agent
    /// * `claude_config` - Configuration for Claude Code integration
    ///
    /// # Returns
    ///
    /// Returns a new `ClaudeCodeAgent` instance ready for initialization.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccswarm::agent::ClaudeCodeAgent;
    /// use ccswarm::identity::AgentRole;
    /// use ccswarm::config::ClaudeConfig;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let role = AgentRole::Backend {
    ///     technologies: vec!["Rust".to_string(), "Python".to_string()],
    ///     responsibilities: vec!["API Development".to_string()],
    ///     boundaries: vec!["Backend only".to_string()],
    /// };
    ///
    /// let agent = ClaudeCodeAgent::new(
    ///     role,
    ///     Path::new("/workspace"),
    ///     "feature",
    ///     ClaudeConfig::default(),
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
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

        let personality = AgentPersonality::new(agent_id.clone());

        let whiteboard = Whiteboard::new(agent_id.clone());
        let phronesis = PhronesisManager::new();

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
    ///
    /// This method performs the complete initialization sequence for the agent:
    ///
    /// 1. **Git Worktree Setup**: Creates an isolated git worktree for the agent
    /// 2. **Container Setup**: Optionally sets up container isolation
    /// 3. **Claude Configuration**: Generates CLAUDE.md and .claude.json files
    /// 4. **Identity Establishment**: Establishes agent identity with Claude Code
    /// 5. **Boundary Verification**: Tests that agent boundaries are working
    ///
    /// # Error Handling
    ///
    /// The initialization process can fail at several points:
    /// - Git operations (worktree creation, branch management)
    /// - File system operations (directory creation, file writing)
    /// - Claude Code communication (identity establishment)
    /// - Container setup (if using container isolation)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn example(mut agent: ccswarm::agent::ClaudeCodeAgent) -> Result<(), Box<dyn std::error::Error>> {
    /// // Initialize the agent - this must be called before executing tasks
    /// agent.initialize().await?;
    ///
    /// // Agent is now ready to execute tasks
    /// println!("Agent {} is ready", agent.identity.agent_id);
    /// # Ok(())
    /// # }
    /// ```
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
                self.worktree_path
                    .to_str()
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
    ///
    /// This is the main entry point for task execution. The method performs
    /// comprehensive validation and monitoring throughout the execution process.
    ///
    /// # Task Execution Pipeline
    ///
    /// 1. **Boundary Evaluation**: Check if task is within agent's capabilities
    /// 2. **Complexity Analysis**: Determine if task needs orchestration
    /// 3. **Execution Strategy**: Choose between simple or orchestrated execution
    /// 4. **Identity Monitoring**: Continuous monitoring during execution
    /// 5. **Result Processing**: Validate and report execution results
    ///
    /// # Boundary Checking
    ///
    /// Tasks are evaluated against agent boundaries and can result in:
    /// - **Accept**: Task is executed by this agent
    /// - **Delegate**: Task is forwarded to a more appropriate agent
    /// - **Clarify**: Additional information is needed
    /// - **Reject**: Task is outside agent's scope
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccswarm::agent::{Task, Priority, TaskType};
    ///
    /// # async fn example(mut agent: ccswarm::agent::ClaudeCodeAgent) -> Result<(), Box<dyn std::error::Error>> {
    /// let task = Task::new(
    ///     "ui-task-1".to_string(),
    ///     "Create a responsive navigation component".to_string(),
    ///     Priority::Medium,
    ///     TaskType::Development,
    /// ).with_details("Should work on mobile and desktop".to_string());
    ///
    /// match agent.execute_task(task).await {
    ///     Ok(result) if result.success => {
    ///         println!("Task completed: {}", result.output);
    ///     }
    ///     Ok(result) => {
    ///         println!("Task failed: {:?}", result.error);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Execution error: {}", e);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// This method should not panic under normal circumstances. All errors
    /// are properly handled and returned as `Result<TaskResult>`.
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
                    format!("Task rejected: {}", reason),
                    false
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
        let mut thinking_engine = InterleavedThinkingEngine::new().with_config(crate::agent::interleaved_thinking::ThinkingConfig {
            max_depth: 15,
            timeout_ms: 5000,
            parallel_thoughts: false,
        }); // Max 15 steps thinking configuration

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
        thinking_engine
            .process_observation(format!("{} ({})", initial_observation, self.identity.specialization.name()));

        // Prepare task prompt with identity header
        let mut prompt = claude::generate_task_prompt(&self.identity, &task);

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
            thinking_engine
                .process_observation(format!("{} ({})", observation, self.identity.specialization.name()));

            // Record thinking on whiteboard
            self.whiteboard
                .add_thought(&thought_trace_id, &format!("観察: {}", observation));

            // Monitor the response for identity
            let identity_status = monitor.monitor_response(&output).await?;
            self.handle_identity_status(identity_status, &mut monitor)
                .await?;

            // Handle thinking decision
            let thinking_step = ThinkingStep::new(
                observation.clone(),
                "Analysis".to_string(),
                DecisionType::Continue { reason: "Processing".to_string() }
            );

            match thinking_step.decision {
                DecisionType::Continue { reason } => {
                    tracing::debug!("Continuing execution: {}", reason);
                    self.whiteboard
                        .add_thought(&thought_trace_id, &format!("継続: {}", reason));
                    final_output = output;
                    if execution_count >= max_executions {
                        break;
                    }
                }
                DecisionType::Refine { refinement, reason } => {
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
                DecisionType::Complete { summary } => {
                    tracing::info!("Task completed: {}", summary);
                    self.whiteboard
                        .add_thought(&thought_trace_id, &format!("完了: {}", summary));
                    self.whiteboard.set_conclusion(&thought_trace_id, &summary);
                    final_output = output;
                    break;
                }
                DecisionType::Pivot {
                    new_direction,
                    reason,
                } => {
                    tracing::warn!("Pivoting approach: {} - {}", reason, new_direction);
                    self.whiteboard.add_thought(
                        &thought_trace_id,
                        &format!("方針転換: {} - {}", reason, new_direction),
                    );
                    self.whiteboard.annotate(
                        &thought_trace_id,
                        "大幅な方針変更",
                        AnnotationMarker::Important,
                    );
                    prompt = self.generate_pivot_prompt(&task, &new_direction);
                }
                DecisionType::RequestContext { questions } => {
                    tracing::info!("Additional context needed: {:?}", questions);
                    self.whiteboard
                        .add_thought(&thought_trace_id, &format!("追加情報必要: {:?}", questions));
                    // In a real implementation, this would request from orchestrator
                    // For now, we'll add questions to prompt and continue
                    prompt.push_str(&format!("\n\nPlease address: {}", questions.join(", ")));
                }
                DecisionType::Abort { reason } => {
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
        let _lesson = format!(
            "Task completed with thinking summary: {}",
            thinking_summary
        );
        self.phronesis.record_success(
            format!("Task {} completed successfully with interleaved thinking", task.id)
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
                    format!("Identity drift detected: {}", msg),
                    false
                );

                self.correct_identity_drift(monitor).await
            }
            IdentityStatus::BoundaryViolation(msg) => {
                // Record boundary violation
                self.phronesis.record_failure(
                    format!("Boundary violation: {}", msg)
                );
                Err(anyhow::anyhow!("Boundary violation detected: {}", msg))
            }
            IdentityStatus::CriticalFailure(msg) => {
                // Record critical failure
                self.phronesis.record_failure(
                    format!("Critical identity failure: {}", msg)
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
    ///
    /// This method implements the agent's learning system, updating the agent's
    /// personality and skills based on task completion. The learning system
    /// uses multiple dimensions:
    ///
    /// # Learning Dimensions
    ///
    /// - **Task Priority**: Higher priority tasks provide more experience
    /// - **Task Type**: Different types contribute to different skill areas
    /// - **Task Success**: Successful completion increases confidence
    /// - **Complexity**: More complex tasks provide proportionally more learning
    ///
    /// # Experience Points Calculation
    ///
    /// ```text
    /// Base Points = match priority {
    ///     Critical => 100,
    ///     High => 50,
    ///     Medium => 30,
    ///     Low => 10,
    /// }
    ///
    /// Multiplier = match task_type {
    ///     Development | Feature => 1.0,
    ///     Bugfix => 1.2,
    ///     Remediation => 1.5,
    ///     Testing => 0.8,
    ///     Documentation => 0.6,
    /// }
    ///
    /// Final XP = Base Points × Multiplier
    /// ```
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ccswarm::agent::{Task, Priority, TaskType};
    ///
    /// # fn example(mut agent: ccswarm::agent::ClaudeCodeAgent) {
    /// let high_priority_task = Task::new(
    ///     "critical-fix".to_string(),
    ///     "Fix security vulnerability".to_string(),
    ///     Priority::Critical,
    ///     TaskType::Bugfix,
    /// );
    ///
    /// // This will give significant experience due to high priority and bugfix type
    /// agent.update_agent_experience(&high_priority_task);
    /// # }
    /// ```
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
            skill.add_experience(adjusted_experience as f32);
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
    ///
    /// Performs a clean shutdown of the agent, ensuring all resources are
    /// properly released and any ongoing work is completed or safely aborted.
    ///
    /// # Shutdown Sequence
    ///
    /// 1. **Status Update**: Mark agent as shutting down
    /// 2. **Final Report**: Send final status report to coordination system
    /// 3. **Container Cleanup**: Clean up any container resources
    /// 4. **Resource Release**: Release file handles, network connections, etc.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # async fn example(mut agent: ccswarm::agent::ClaudeCodeAgent) -> Result<(), Box<dyn std::error::Error>> {
    /// // Perform graceful shutdown
    /// agent.shutdown().await?;
    /// println!("Agent shutdown completed");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Error Handling
    ///
    /// Shutdown errors are logged but typically not critical. The method
    /// will attempt to clean up as much as possible even if some steps fail.
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

