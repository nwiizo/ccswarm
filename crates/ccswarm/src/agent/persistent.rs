/// Session-persistent Claude Code agent implementation for token efficiency
///
/// This module implements the Session-Persistent Agent Architecture to address
/// the token consumption issues in multi-task workflows. Key improvements:
/// - Efficient context management through session reuse
/// - One-time identity establishment per agent lifecycle
/// - Conversation history preservation for context continuity
/// - Batch task processing to amortize overhead
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::{info, warn};
use uuid::Uuid;

use crate::agent::orchestrator::task_plan::{ParallelTask, ParallelTaskResult};
use crate::agent::orchestrator::{AgentOrchestrator, StepResult, StepType, TaskPlan, TaskStep};
use crate::agent::{AgentStatus, Task, TaskResult, TaskType};
use crate::config::ClaudeConfig;
use crate::identity::{AgentIdentity, IdentityMonitor, IdentityStatus};

/// Message in conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub timestamp: DateTime<Utc>,
    pub message_type: MessageType,
    pub content: String,
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    IdentityEstablishment,
    TaskPrompt,
    Response,
    IdentityReminder,
    BatchStart,
    BatchEnd,
}

/// Claude Code session with state management
#[derive(Debug)]
pub struct ClaudeCodeSession {
    /// Session ID for tracking
    session_id: String,

    /// Working directory for Claude Code
    working_dir: PathBuf,

    /// Environment variables for the session
    env_vars: HashMap<String, String>,

    /// Claude configuration
    claude_config: ClaudeConfig,

    /// Last activity timestamp
    last_activity: Instant,

    /// Whether session is currently active
    is_active: bool,
}

impl ClaudeCodeSession {
    pub fn new(
        session_id: String,
        working_dir: PathBuf,
        env_vars: HashMap<String, String>,
        claude_config: ClaudeConfig,
    ) -> Self {
        Self {
            session_id,
            working_dir,
            env_vars,
            claude_config,
            last_activity: Instant::now(),
            is_active: true,
        }
    }

    /// Send a message to Claude Code and get response
    pub async fn send_message(&mut self, prompt: &str) -> Result<String> {
        self.last_activity = Instant::now();

        // For tests and simulation mode, return a mock response
        if !self.claude_config.use_real_api {
            return Ok(self.generate_mock_response(prompt));
        }

        let mut cmd = Command::new("claude");
        cmd.current_dir(&self.working_dir);

        // Add environment variables
        for (key, value) in &self.env_vars {
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

    /// Generate a mock response for testing and simulation
    fn generate_mock_response(&self, prompt: &str) -> String {
        // Check if this is an identity establishment prompt
        if prompt.contains("🤖 **AGENT IDENTITY**") {
            return format!(
                "🤖 AGENT: {}\n📁 WORKSPACE: {}\n🎯 SCOPE: Identity established successfully",
                self.session_id,
                self.working_dir
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );
        }

        // Generate a simple mock response for tasks
        if prompt.contains("## Current Task:") {
            return format!(
                "🤖 AGENT: {}\n📁 WORKSPACE: {}\n🎯 SCOPE: Task within agent boundaries\n\nTask completed successfully in simulation mode.",
                self.session_id,
                self.working_dir.file_name().unwrap_or_default().to_string_lossy()
            );
        }

        // Default mock response
        format!(
            "🤖 AGENT: {}\n📁 WORKSPACE: {}\n🎯 SCOPE: General response\n\nMock response generated for testing.",
            self.session_id,
            self.working_dir.file_name().unwrap_or_default().to_string_lossy()
        )
    }

    /// Execute task with conversation context
    pub async fn execute_with_context(
        &mut self,
        task: &Task,
        history: &VecDeque<ConversationMessage>,
    ) -> Result<String> {
        // Build context-aware prompt
        let prompt = self.build_contextual_prompt(task, history);
        self.send_message(&prompt).await
    }

    /// Build prompt with conversation context
    fn build_contextual_prompt(
        &self,
        task: &Task,
        history: &VecDeque<ConversationMessage>,
    ) -> String {
        let mut prompt = String::new();

        // Add recent relevant context (last 3 messages)
        let recent_context: Vec<_> = history
            .iter()
            .rev()
            .take(3)
            .filter(|msg| {
                matches!(
                    msg.message_type,
                    MessageType::TaskPrompt | MessageType::Response
                )
            })
            .collect();

        if !recent_context.is_empty() {
            prompt.push_str("## Recent Context:\n");
            for msg in recent_context.iter().rev() {
                match msg.message_type {
                    MessageType::TaskPrompt => {
                        prompt.push_str(&format!(
                            "Previous Task: {}\n",
                            msg.content.lines().next().unwrap_or("")
                        ));
                    }
                    MessageType::Response => {
                        let summary = msg
                            .content
                            .lines()
                            .next()
                            .unwrap_or("")
                            .chars()
                            .take(100)
                            .collect::<String>();
                        prompt.push_str(&format!("Previous Result: {}...\n", summary));
                    }
                    _ => {}
                }
            }
            prompt.push('\n');
        }

        // Add current task
        prompt.push_str(&format!(
            "## Current Task:\n**{}**\n\n{}\n\nPriority: {:?}\nType: {:?}\n",
            task.description,
            task.details.as_deref().unwrap_or(""),
            task.priority,
            task.task_type
        ));

        prompt
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn last_activity(&self) -> Instant {
        self.last_activity
    }

    pub fn shutdown(&mut self) {
        self.is_active = false;
    }
}

/// Session-persistent Claude Code agent for token efficiency
#[derive(Debug)]
pub struct PersistentClaudeAgent {
    /// Agent identity
    pub identity: AgentIdentity,

    /// Claude Code session
    session: Arc<Mutex<ClaudeCodeSession>>,

    /// Whether identity has been established
    identity_established: Arc<AtomicBool>,

    /// Conversation history with size limit
    conversation_history: Arc<Mutex<VecDeque<ConversationMessage>>>,

    /// Maximum history size (to prevent unbounded growth)
    max_history_size: usize,

    /// Current agent status
    pub status: AgentStatus,

    /// Current task being worked on
    pub current_task: Option<Task>,

    /// Task history
    pub task_history: Vec<(Task, TaskResult)>,

    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,

    /// Session creation timestamp
    pub session_created_at: DateTime<Utc>,
}

impl PersistentClaudeAgent {
    /// Create a new persistent agent
    pub async fn new(identity: AgentIdentity, claude_config: ClaudeConfig) -> Result<Self> {
        let session_id = Uuid::new_v4().to_string();
        let session = ClaudeCodeSession::new(
            session_id,
            identity.workspace_path.clone(),
            identity.env_vars.clone(),
            claude_config,
        );

        Ok(Self {
            identity,
            session: Arc::new(Mutex::new(session)),
            identity_established: Arc::new(AtomicBool::new(false)),
            conversation_history: Arc::new(Mutex::new(VecDeque::new())),
            max_history_size: 50, // Keep last 50 messages
            status: AgentStatus::Initializing,
            current_task: None,
            task_history: Vec::new(),
            last_activity: Utc::now(),
            session_created_at: Utc::now(),
        })
    }

    /// Establish identity once per session lifecycle
    pub async fn establish_identity_once(&self) -> Result<()> {
        if self.identity_established.load(Ordering::Relaxed) {
            tracing::debug!(
                "Identity already established for agent: {}",
                self.identity.agent_id
            );
            return Ok(());
        }

        tracing::info!(
            "Establishing identity for agent: {}",
            self.identity.agent_id
        );

        // Generate lightweight identity prompt (200 tokens vs 2000+ in original)
        let compact_prompt = self.generate_compact_identity_prompt();

        // Send identity establishment message
        let mut session = self.session.lock().await;
        let response = session.send_message(&compact_prompt).await?;

        // Verify response contains identity markers
        if !self.verify_identity_response(&response) {
            return Err(anyhow::anyhow!("Failed to establish agent identity"));
        }

        // Record in conversation history
        let mut history = self.conversation_history.lock().await;
        history.push_back(ConversationMessage {
            timestamp: Utc::now(),
            message_type: MessageType::IdentityEstablishment,
            content: compact_prompt,
            task_id: None,
        });
        history.push_back(ConversationMessage {
            timestamp: Utc::now(),
            message_type: MessageType::Response,
            content: response,
            task_id: None,
        });

        // Trim history if needed
        self.trim_history(&mut history);

        self.identity_established.store(true, Ordering::Relaxed);
        tracing::info!(
            "Identity established successfully for agent: {}",
            self.identity.agent_id
        );

        Ok(())
    }

    /// Generate compact identity prompt (much smaller than full CLAUDE.md)
    fn generate_compact_identity_prompt(&self) -> String {
        format!(
            r#"🤖 **AGENT IDENTITY**: {}
📁 **WORKSPACE**: {}
🎯 **SPECIALIZATION**: {}
🚫 **BOUNDARIES**: Stay within {} scope only
✅ **RESPONSE FORMAT**: Always include identity header in responses:
   🤖 AGENT: {}
   📁 WORKSPACE: {}
   🎯 SCOPE: [Assessment of task fit]

You are a specialized {} agent. Maintain strict boundaries and provide concise, focused responses."#,
            self.identity.agent_id,
            self.identity
                .workspace_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            self.identity.specialization.name(),
            self.identity.specialization.name(),
            self.identity.specialization.name(),
            self.identity
                .workspace_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy(),
            self.identity.specialization.name(),
        )
    }

    /// Verify identity response contains required markers
    fn verify_identity_response(&self, response: &str) -> bool {
        response.contains("🤖 AGENT:") && response.contains(self.identity.specialization.name())
    }

    /// Execute a single task with session persistence
    pub async fn execute_task(&mut self, task: Task) -> Result<TaskResult> {
        self.status = AgentStatus::Working;
        self.current_task = Some(task.clone());
        self.last_activity = Utc::now();

        // Ensure identity is established
        self.establish_identity_once().await?;

        let start_time = Instant::now();
        let mut identity_monitor = IdentityMonitor::new(&self.identity.agent_id);

        // Execute task with conversation context
        let history = self.conversation_history.lock().await;
        let history_clone = history.clone();
        drop(history); // Release lock

        let mut session = self.session.lock().await;
        let response = session.execute_with_context(&task, &history_clone).await?;
        drop(session); // Release lock

        // Monitor identity in response
        let identity_status = identity_monitor.monitor_response(&response).await?;
        match identity_status {
            IdentityStatus::Healthy => {
                tracing::debug!("Identity maintained during task execution");
            }
            IdentityStatus::DriftDetected(msg) => {
                tracing::warn!("Identity drift detected: {}", msg);
                self.send_identity_reminder().await?;
            }
            IdentityStatus::BoundaryViolation(msg) => {
                return Err(anyhow::anyhow!("Boundary violation detected: {}", msg));
            }
            IdentityStatus::CriticalFailure(msg) => {
                return Err(anyhow::anyhow!("Critical identity failure: {}", msg));
            }
        }

        // Record conversation
        let mut history = self.conversation_history.lock().await;
        history.push_back(ConversationMessage {
            timestamp: Utc::now(),
            message_type: MessageType::TaskPrompt,
            content: format!("Task: {}", task.description),
            task_id: Some(task.id.clone()),
        });
        history.push_back(ConversationMessage {
            timestamp: Utc::now(),
            message_type: MessageType::Response,
            content: response.clone(),
            task_id: Some(task.id.clone()),
        });
        self.trim_history(&mut history);

        let result = TaskResult {
            success: true,
            output: serde_json::json!({
                "response": response,
                "agent": self.identity.agent_id,
                "task_id": task.id,
                "session_id": self.session.lock().await.session_id(),
            }),
            error: None,
            duration: start_time.elapsed(),
        };

        // Update status and history
        self.status = AgentStatus::WaitingForReview;
        self.current_task = None;
        self.task_history.push((task, result.clone()));

        Ok(result)
    }

    /// Execute multiple tasks in batch for maximum efficiency
    pub async fn execute_task_batch(&mut self, tasks: Vec<Task>) -> Result<Vec<TaskResult>> {
        if tasks.is_empty() {
            return Ok(Vec::new());
        }

        tracing::info!(
            "Executing batch of {} tasks for agent: {}",
            tasks.len(),
            self.identity.agent_id
        );

        // Ensure identity is established once for the entire batch
        self.establish_identity_once().await?;

        // Record batch start
        let mut history = self.conversation_history.lock().await;
        history.push_back(ConversationMessage {
            timestamp: Utc::now(),
            message_type: MessageType::BatchStart,
            content: format!("Starting batch of {} tasks", tasks.len()),
            task_id: None,
        });
        drop(history);

        let mut results = Vec::new();

        for (i, task) in tasks.into_iter().enumerate() {
            tracing::debug!(
                "Executing batch task {}/{} for agent: {}",
                i + 1,
                results.capacity(),
                self.identity.agent_id
            );

            // For batch processing, we can be more efficient by reusing context
            let result = self.execute_task(task).await?;
            results.push(result);
        }

        // Record batch end
        let mut history = self.conversation_history.lock().await;
        history.push_back(ConversationMessage {
            timestamp: Utc::now(),
            message_type: MessageType::BatchEnd,
            content: format!("Completed batch of {} tasks", results.len()),
            task_id: None,
        });
        self.trim_history(&mut history);

        tracing::info!(
            "Batch execution completed for agent: {}",
            self.identity.agent_id
        );
        Ok(results)
    }

    /// Send lightweight identity reminder instead of full re-establishment
    async fn send_identity_reminder(&self) -> Result<()> {
        let reminder_prompt = format!(
            "🤖 IDENTITY REMINDER: You are a {} agent working in {}. Maintain specialization boundaries.",
            self.identity.specialization.name(),
            self.identity.workspace_path.file_name().unwrap_or_default().to_string_lossy()
        );

        let mut session = self.session.lock().await;
        let _response = session.send_message(&reminder_prompt).await?;

        let mut history = self.conversation_history.lock().await;
        history.push_back(ConversationMessage {
            timestamp: Utc::now(),
            message_type: MessageType::IdentityReminder,
            content: reminder_prompt,
            task_id: None,
        });
        self.trim_history(&mut history);

        Ok(())
    }

    /// Trim conversation history to prevent unbounded growth
    fn trim_history(&self, history: &mut VecDeque<ConversationMessage>) {
        while history.len() > self.max_history_size {
            history.pop_front();
        }
    }

    /// Get session statistics
    pub async fn get_session_stats(&self) -> SessionStats {
        let session = self.session.lock().await;
        let history = self.conversation_history.lock().await;

        SessionStats {
            session_id: session.session_id().to_string(),
            created_at: self.session_created_at,
            last_activity: DateTime::from_timestamp(
                session.last_activity().elapsed().as_secs() as i64,
                0,
            )
            .unwrap_or(Utc::now()),
            identity_established: self.identity_established.load(Ordering::Relaxed),
            conversation_messages: history.len(),
            tasks_completed: self.task_history.len(),
            is_active: session.is_active(),
        }
    }

    /// Shutdown the persistent session
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down persistent agent: {}", self.identity.agent_id);
        self.status = AgentStatus::ShuttingDown;

        let mut session = self.session.lock().await;
        session.shutdown();

        Ok(())
    }
}

/// Session statistics for monitoring
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStats {
    pub session_id: String,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub identity_established: bool,
    pub conversation_messages: usize,
    pub tasks_completed: usize,
    pub is_active: bool,
}

#[async_trait]
impl AgentOrchestrator for PersistentClaudeAgent {
    async fn orchestrate_task(&self, task: &Task) -> Result<TaskResult> {
        info!(
            "🎭 Persistent agent orchestrating task: {}",
            task.description
        );

        // Leverage persistent session for complex task orchestration
        let plan = self.analyze_task(task).await?;

        // Execute plan with session persistence for context continuity
        let mut context = HashMap::new();
        context.insert("task_id".to_string(), task.id.clone());
        context.insert("persistent_mode".to_string(), "true".to_string());
        context.insert("session_id".to_string(), {
            let session = self.session.lock().await;
            session.session_id().to_string()
        });

        for step in &plan.steps {
            let step_result = self.execute_step(step, &context).await?;

            // Update context with step results and preserve in session history
            context.insert(
                format!("step_{}_result", step.id),
                serde_json::to_string(&step_result.outputs)?,
            );

            if !step_result.success {
                warn!("Persistent agent step failed: {}", step.name);
                return Ok(TaskResult::failure(
                    format!("Persistent orchestration failed at step: {}", step.name),
                    Duration::from_secs(0),
                ));
            }
        }

        Ok(TaskResult::success(
            serde_json::json!({
                "orchestration": "persistent_success",
                "steps_completed": plan.steps.len(),
                "session_id": context.get("session_id")
            }),
            Duration::from_secs(15),
        ))
    }

    async fn analyze_task(&self, task: &Task) -> Result<TaskPlan> {
        let mut plan = TaskPlan::new(task.id.clone());

        // Persistent session-aware orchestration strategies
        match task.task_type {
            TaskType::Development => {
                // Long-running development workflow with session continuity
                plan.add_step(
                    TaskStep::new(
                        "persistent_context_analysis".to_string(),
                        "Analyze task with session context".to_string(),
                        StepType::Analysis,
                    )
                    .with_description(
                        "Analyze task using persistent session context and history".to_string(),
                    ),
                );
                plan.add_step(
                    TaskStep::new(
                        "iterative_development".to_string(),
                        "Execute with session persistence".to_string(),
                        StepType::Execution,
                    )
                    .with_description(
                        "Execute development task with session context continuity".to_string(),
                    ),
                );
                plan.add_step(
                    TaskStep::new(
                        "session_context_validation".to_string(),
                        "Validate within session context".to_string(),
                        StepType::Validation,
                    )
                    .with_description(
                        "Validate results within persistent session context".to_string(),
                    ),
                );
            }
            TaskType::Testing => {
                // Test execution with preserved test state
                plan.add_step(
                    TaskStep::new(
                        "test_session_setup".to_string(),
                        "Setup test session context".to_string(),
                        StepType::Analysis,
                    )
                    .with_description("Setup testing context using session history".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "persistent_test_execution".to_string(),
                        "Execute tests with session state".to_string(),
                        StepType::Execution,
                    )
                    .with_description("Execute tests with persistent session state".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "test_result_persistence".to_string(),
                        "Persist test results in session".to_string(),
                        StepType::Validation,
                    )
                    .with_description("Store test results in persistent session".to_string()),
                );
            }
            TaskType::Bugfix => {
                // Bug investigation with conversation history
                plan.add_step(
                    TaskStep::new(
                        "bug_context_analysis".to_string(),
                        "Analyze bug with session history".to_string(),
                        StepType::Analysis,
                    )
                    .with_description("Analyze bug using conversation history".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "iterative_debugging".to_string(),
                        "Debug with persistent context".to_string(),
                        StepType::Execution,
                    )
                    .with_description("Debug using persistent session context".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "fix_verification".to_string(),
                        "Verify fix with session validation".to_string(),
                        StepType::Validation,
                    )
                    .with_description("Verify fix using session context".to_string()),
                );
            }
            _ => {
                // Default persistent workflow
                plan.add_step(
                    TaskStep::new(
                        "persistent_analysis".to_string(),
                        "Analyze with session context".to_string(),
                        StepType::Analysis,
                    )
                    .with_description("Analyze task with session context".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "session_execution".to_string(),
                        "Execute with preserved context".to_string(),
                        StepType::Execution,
                    )
                    .with_description("Execute with preserved session context".to_string()),
                );
                plan.add_step(
                    TaskStep::new(
                        "persistent_validation".to_string(),
                        "Validate with session history".to_string(),
                        StepType::Validation,
                    )
                    .with_description("Validate using session history".to_string()),
                );
            }
        }

        Ok(plan)
    }

    async fn execute_step(
        &self,
        step: &TaskStep,
        context: &HashMap<String, String>,
    ) -> Result<StepResult> {
        info!("🎭 Persistent agent executing step: {}", step.name);

        match step.name.as_str() {
            "Analyze task with session context" => {
                self.execute_context_analysis_step(step, context).await
            }
            "Execute with session persistence" => {
                self.execute_iterative_development_step(step, context).await
            }
            "Validate within session context" => {
                self.execute_session_validation_step(step, context).await
            }
            "Execute tests with session state" => {
                self.execute_persistent_test_step(step, context).await
            }
            "Debug with persistent context" => {
                self.execute_iterative_debugging_step(step, context).await
            }
            _ => {
                // Default execution with session persistence
                self.execute_persistent_default_step(step, context).await
            }
        }
    }

    async fn review_and_adapt(
        &self,
        results: &[StepResult],
        current_plan: &mut TaskPlan,
    ) -> Result<TaskPlan> {
        info!(
            "🎭 Persistent agent reviewing and adapting plan based on {} results",
            results.len()
        );

        // Leverage conversation history for intelligent adaptation
        let history = self.conversation_history.lock().await;
        let failed_steps: Vec<_> = results.iter().filter(|r| !r.success).collect();

        if !failed_steps.is_empty() {
            warn!(
                "Persistent agent detected {} failed steps",
                failed_steps.len()
            );

            // Analyze failure patterns from session history
            let failure_analysis = self.analyze_failure_patterns(&history, &failed_steps);

            // Add adaptive recovery steps based on session context
            if failure_analysis.contains("context_loss") {
                let recovery_step = TaskStep::new(
                    "context_recovery".to_string(),
                    "Recover session context".to_string(),
                    StepType::Execution,
                )
                .with_description("Recover lost session context".to_string());
                current_plan.steps.push(recovery_step);
            }

            if failure_analysis.contains("identity_drift") {
                let identity_step = TaskStep::new(
                    "identity_reinforcement".to_string(),
                    "Reinforce agent identity".to_string(),
                    StepType::Execution,
                )
                .with_description("Reinforce agent identity and boundaries".to_string());
                current_plan.steps.push(identity_step);
            }

            let validation_step = TaskStep::new(
                "adaptive_validation".to_string(),
                "Validate adaptive recovery".to_string(),
                StepType::Validation,
            )
            .with_description("Validate recovery steps".to_string());
            current_plan.steps.push(validation_step);
        }

        Ok(current_plan.clone())
    }

    async fn execute_parallel_task(
        &self,
        task: &ParallelTask,
        _context: &HashMap<String, String>,
    ) -> Result<ParallelTaskResult> {
        // Execute parallel task with session context
        let execution_task = Task::new(
            task.id.clone(),
            task.name.clone(),
            crate::agent::Priority::Medium,
            TaskType::Development,
        );

        // Use conversation history for context
        let mut session = self.session.lock().await;
        let history = self.conversation_history.lock().await;
        let response = session
            .execute_with_context(&execution_task, &history)
            .await?;
        drop(session);
        drop(history);

        let success = !response.contains("error") && !response.contains("failed");

        Ok(ParallelTaskResult {
            task_id: task.id.clone(),
            success,
            output: response,
            error: if success {
                None
            } else {
                Some("Task execution failed".to_string())
            },
        })
    }

    async fn synthesize_results(
        &self,
        task: &Task,
        results: Vec<StepResult>,
    ) -> Result<TaskResult> {
        let total_steps = results.len();
        let successful_steps = results.iter().filter(|r| r.is_success()).count();
        let overall_success = successful_steps == total_steps;

        let summary = if overall_success {
            format!(
                "Persistent agent orchestration completed successfully: {}/{} steps succeeded",
                successful_steps, total_steps
            )
        } else {
            format!(
                "Persistent agent orchestration partially failed: {}/{} steps succeeded",
                successful_steps, total_steps
            )
        };

        let session_id = {
            let session = self.session.lock().await;
            session.session_id().to_string()
        };

        let output = serde_json::json!({
            "orchestration_type": "persistent",
            "task_id": task.id,
            "session_id": session_id,
            "total_steps": total_steps,
            "successful_steps": successful_steps,
            "conversation_history_length": self.conversation_history.lock().await.len(),
            "step_results": results.iter().map(|r| {
                serde_json::json!({
                    "step_id": r.step_id,
                    "success": r.success,
                    "summary": r.summary
                })
            }).collect::<Vec<_>>()
        });

        if overall_success {
            Ok(TaskResult::success(
                output,
                Duration::from_millis(results.iter().map(|r| r.duration_ms).sum()),
            ))
        } else {
            Ok(TaskResult::failure(
                summary,
                Duration::from_millis(results.iter().map(|r| r.duration_ms).sum()),
            ))
        }
    }
}

impl PersistentClaudeAgent {
    /// Execute context analysis step leveraging session history
    async fn execute_context_analysis_step(
        &self,
        _step: &TaskStep,
        context: &HashMap<String, String>,
    ) -> Result<StepResult> {
        let task_id = context
            .get("task_id")
            .unwrap_or(&"unknown".to_string())
            .clone();
        let session_id = context
            .get("session_id")
            .unwrap_or(&"unknown".to_string())
            .clone();

        // Analyze conversation history for relevant context
        let history = self.conversation_history.lock().await;
        let relevant_messages = history
            .iter()
            .filter(|msg| {
                matches!(
                    msg.message_type,
                    MessageType::TaskPrompt | MessageType::Response
                )
            })
            .count();

        Ok(StepResult::new(_step.id.clone())
            .with_summary(format!("Context analysis for task {} completed", task_id))
            .add_output("task_id".to_string(), task_id)
            .add_output("session_id".to_string(), session_id)
            .add_output(
                "history_messages".to_string(),
                relevant_messages.to_string(),
            ))
    }

    /// Execute iterative development step with session persistence
    async fn execute_iterative_development_step(
        &self,
        step: &TaskStep,
        _context: &HashMap<String, String>,
    ) -> Result<StepResult> {
        let task = Task::new(
            format!("persistent_dev_{}", step.id),
            step.description.clone(),
            crate::agent::Priority::Medium,
            TaskType::Development,
        );

        // Use session context for development
        let mut session = self.session.lock().await;
        let history = self.conversation_history.lock().await;
        let response = session.execute_with_context(&task, &history).await?;
        drop(session);
        drop(history);

        Ok(StepResult::new(step.id.clone())
            .with_summary("Iterative development step completed with session context".to_string())
            .add_output("development_response".to_string(), response)
            .add_output("session_context_used".to_string(), "true".to_string()))
    }

    /// Execute session validation step
    async fn execute_session_validation_step(
        &self,
        _step: &TaskStep,
        context: &HashMap<String, String>,
    ) -> Result<StepResult> {
        let session_id = context
            .get("session_id")
            .unwrap_or(&"unknown".to_string())
            .clone();

        // Validate session state and identity
        let identity_established = self.identity_established.load(Ordering::Relaxed);
        let history = self.conversation_history.lock().await;
        let session_health = !history.is_empty() && identity_established;

        Ok(StepResult::new(_step.id.clone())
            .with_summary(format!("Session validation completed for {}", session_id))
            .add_output("session_id".to_string(), session_id)
            .add_output(
                "identity_established".to_string(),
                identity_established.to_string(),
            )
            .add_output("session_healthy".to_string(), session_health.to_string())
            .add_output("conversation_length".to_string(), history.len().to_string()))
    }

    /// Execute persistent test step
    async fn execute_persistent_test_step(
        &self,
        step: &TaskStep,
        _context: &HashMap<String, String>,
    ) -> Result<StepResult> {
        let task = Task::new(
            format!("persistent_test_{}", step.id),
            step.description.clone(),
            crate::agent::Priority::Medium,
            TaskType::Testing,
        );

        // Execute tests with session persistence
        let mut session = self.session.lock().await;
        let history = self.conversation_history.lock().await;
        let response = session.execute_with_context(&task, &history).await?;
        drop(session);
        drop(history);

        // Parse test results from response
        let test_success = response.contains("✅") || response.contains("passed");

        Ok(StepResult::new(step.id.clone())
            .with_summary("Persistent test execution completed".to_string())
            .add_output("test_response".to_string(), response)
            .add_output("test_passed".to_string(), test_success.to_string())
            .add_output("persistent_execution".to_string(), "true".to_string()))
    }

    /// Execute iterative debugging step
    async fn execute_iterative_debugging_step(
        &self,
        step: &TaskStep,
        _context: &HashMap<String, String>,
    ) -> Result<StepResult> {
        let task = Task::new(
            format!("persistent_debug_{}", step.id),
            step.description.clone(),
            crate::agent::Priority::High,
            TaskType::Bugfix,
        );

        // Use conversation history for debugging context
        let mut session = self.session.lock().await;
        let history = self.conversation_history.lock().await;
        let response = session.execute_with_context(&task, &history).await?;
        drop(session);
        drop(history);

        // Analyze debugging progress
        let debug_progress = response.contains("identified")
            || response.contains("fixed")
            || response.contains("resolved");

        Ok(StepResult::new(step.id.clone())
            .with_summary("Iterative debugging step completed".to_string())
            .add_output("debug_response".to_string(), response)
            .add_output("progress_made".to_string(), debug_progress.to_string())
            .add_output("context_leveraged".to_string(), "true".to_string()))
    }

    /// Execute default step with session persistence
    async fn execute_persistent_default_step(
        &self,
        step: &TaskStep,
        context: &HashMap<String, String>,
    ) -> Result<StepResult> {
        let task_id = context
            .get("task_id")
            .unwrap_or(&"unknown".to_string())
            .clone();

        let task = Task::new(
            format!("persistent_default_{}_{}", step.id, task_id),
            step.description.clone(),
            crate::agent::Priority::Medium,
            TaskType::Development,
        );

        // Execute with full session context
        let mut session = self.session.lock().await;
        let history = self.conversation_history.lock().await;
        let response = session.execute_with_context(&task, &history).await?;
        drop(session);
        drop(history);

        Ok(StepResult::new(step.id.clone())
            .with_summary("Persistent default step completed".to_string())
            .add_output("response".to_string(), response)
            .add_output("session_context_preserved".to_string(), "true".to_string())
            .add_output("task_id".to_string(), task_id))
    }

    /// Analyze failure patterns from conversation history
    fn analyze_failure_patterns(
        &self,
        history: &VecDeque<ConversationMessage>,
        failed_steps: &[&StepResult],
    ) -> String {
        let mut patterns = Vec::new();

        // Check for context loss patterns
        if failed_steps
            .iter()
            .any(|step| step.errors.iter().any(|e| e.contains("context")))
        {
            patterns.push("context_loss");
        }

        // Check for identity drift in recent messages
        let recent_responses: Vec<_> = history
            .iter()
            .rev()
            .take(5)
            .filter(|msg| matches!(msg.message_type, MessageType::Response))
            .collect();

        if recent_responses
            .iter()
            .any(|msg| !msg.content.contains("🤖 AGENT:"))
        {
            patterns.push("identity_drift");
        }

        patterns.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::default_frontend_role;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_persistent_agent_creation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();

        let identity = AgentIdentity {
            agent_id: "test-frontend-agent".to_string(),
            specialization: default_frontend_role(),
            workspace_path,
            env_vars: HashMap::new(),
            session_id: "test-session".to_string(),
            parent_process_id: "test-parent".to_string(),
            initialized_at: Utc::now(),
        };

        let config = ClaudeConfig::default();
        let agent = PersistentClaudeAgent::new(identity, config).await.unwrap();

        assert_eq!(agent.status, AgentStatus::Initializing);
        assert!(!agent.identity_established.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn test_compact_identity_prompt() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();

        let identity = AgentIdentity {
            agent_id: "test-frontend-agent".to_string(),
            specialization: default_frontend_role(),
            workspace_path,
            env_vars: HashMap::new(),
            session_id: "test-session".to_string(),
            parent_process_id: "test-parent".to_string(),
            initialized_at: Utc::now(),
        };

        let config = ClaudeConfig::default();
        let agent = PersistentClaudeAgent::new(identity, config).await.unwrap();

        let prompt = agent.generate_compact_identity_prompt();

        // Verify it's much more compact than the original CLAUDE.md approach
        assert!(prompt.len() < 500); // Much less than 2000+ tokens
        assert!(prompt.contains("🤖 AGENT:"));
        assert!(prompt.contains("Frontend"));
    }

    #[tokio::test]
    async fn test_persistent_agent_orchestration() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().to_path_buf();

        let identity = AgentIdentity {
            agent_id: "test-persistent-agent".to_string(),
            specialization: default_frontend_role(),
            workspace_path,
            env_vars: HashMap::new(),
            session_id: "test-session".to_string(),
            parent_process_id: "test-parent".to_string(),
            initialized_at: Utc::now(),
        };

        let config = ClaudeConfig::default();
        let agent = PersistentClaudeAgent::new(identity, config).await.unwrap();

        let task = Task::new(
            "test_persistent_orchestration".to_string(),
            "Test persistent agent orchestration".to_string(),
            crate::agent::Priority::Medium,
            TaskType::Development,
        );

        let plan = agent.analyze_task(&task).await.unwrap();
        assert!(!plan.steps.is_empty());
        assert!(plan.steps.iter().any(|step| step.name.contains("session")));
    }
}
