pub mod auto_create;
pub mod llm_quality_judge;
pub mod master_delegation;

#[cfg(test)]
mod llm_quality_judge_test;
#[cfg(test)]
mod review_test;

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

use self::llm_quality_judge::LLMQualityJudge;
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

    /// LLM-based quality judge
    quality_judge: Arc<RwLock<LLMQualityJudge>>,
}

/// Review history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewHistoryEntry {
    pub task_id: String,
    pub agent_id: String,
    pub review_date: DateTime<Utc>,
    pub issues_found: Vec<String>,
    pub remediation_task_id: Option<String>,
    pub review_passed: bool,
    pub iteration: u32,
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
    pub review_history: HashMap<String, Vec<ReviewHistoryEntry>>, // task_id -> review entries
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
            review_history: HashMap::new(),
        }));

        let quality_judge = Arc::new(RwLock::new(LLMQualityJudge::new()));

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
            quality_judge,
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
        let task_queue_tx = self.task_queue_tx.clone();
        let judge = self.quality_judge.clone();

        tokio::spawn(async move {
            loop {
                match bus.receive_message().await {
                    Ok(message) => {
                        if let Err(e) = Self::handle_agent_message(
                            message,
                            &agents,
                            &state,
                            &task_queue_tx,
                            &judge,
                        )
                        .await
                        {
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

        // Start quality review loop with LLM judge
        let agents = self.agents.clone();
        let standards = self.quality_standards.clone();
        let bus = self.coordination_bus.clone();
        let judge = self.quality_judge.clone();
        let worktree_manager = self.worktree_manager.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;

                if let Err(e) = Self::perform_llm_quality_review(
                    &agents,
                    &standards,
                    &bus,
                    &judge,
                    &worktree_manager,
                )
                .await
                {
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
        // For remediation tasks, use the assigned agent if specified
        if task.task_type == TaskType::Remediation && task.assigned_to.is_some() {
            return Ok(task.assigned_to.as_ref().unwrap().clone());
        }

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
            TaskType::Remediation => "Backend", // Fallback for unassigned remediation
            _ => "Backend",                     // Default
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

    /// Create a remediation task for quality issues
    #[allow(dead_code)]
    async fn create_remediation_task(
        &self,
        original_task_id: &str,
        agent_id: &str,
        issues: Vec<String>,
    ) -> Result<Task> {
        // Generate specific fix instructions based on the issues
        let instructions = self.generate_fix_instructions(&issues).await?;

        // Create remediation task
        let task_id = format!("remediate-{}-{}", original_task_id, Uuid::new_v4());

        let description = format!(
            "Fix quality issues in task {}: {}",
            original_task_id, instructions
        );

        let details = format!(
            "Quality issues found:\n{}\n\nSpecific instructions:\n{}",
            issues.join("\n- "),
            instructions
        );

        let remediation_task = Task::new(
            task_id,
            description,
            crate::agent::Priority::High, // High priority for quality fixes
            TaskType::Remediation,
        )
        .with_details(details)
        .assign_to(agent_id.to_string())
        .with_parent_task(original_task_id.to_string())
        .with_quality_issues(issues)
        .with_duration(1800); // 30 minutes estimate

        Ok(remediation_task)
    }

    /// Generate specific fix instructions using Master Claude's intelligence
    #[allow(dead_code)]
    async fn generate_fix_instructions(&self, issues: &[String]) -> Result<String> {
        // For now, generate instructions based on the issue types
        // In a full implementation, this would use Claude API to generate intelligent instructions

        let mut instructions = Vec::new();

        for issue in issues {
            let instruction = match issue.as_str() {
                "Low test coverage" => {
                    "1. Add unit tests to achieve at least 85% coverage\n\
                     2. Focus on edge cases and error paths\n\
                     3. Use test-driven development for any new code\n\
                     4. Run coverage report and ensure all critical paths are tested"
                }
                "High complexity" => {
                    "1. Break down complex functions into smaller, focused functions\n\
                     2. Extract repeated logic into helper functions\n\
                     3. Simplify conditional logic using early returns\n\
                     4. Consider using design patterns to reduce complexity"
                }
                "Security vulnerability" => {
                    "1. Review and fix all security warnings\n\
                     2. Validate all user inputs\n\
                     3. Use parameterized queries for database operations\n\
                     4. Update dependencies to latest secure versions"
                }
                "Missing documentation" => {
                    "1. Add comprehensive docstrings to all public functions\n\
                     2. Include parameter descriptions and return value documentation\n\
                     3. Add usage examples where appropriate\n\
                     4. Update README with any new functionality"
                }
                _ => {
                    "1. Review the specific issue and determine root cause\n\
                     2. Implement the most appropriate fix\n\
                     3. Add tests to prevent regression\n\
                     4. Document the changes made"
                }
            };

            instructions.push(format!("For '{}': \n{}", issue, instruction));
        }

        Ok(instructions.join("\n\n"))
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
        task_queue_tx: &Sender<Task>,
        judge: &Arc<RwLock<LLMQualityJudge>>,
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

                    // Check if this was a remediation task and trigger re-review
                    if task_id.starts_with("remediate-") {
                        // Find the original task ID from review history
                        let original_task_id =
                            s.review_history.iter().find_map(|(orig_id, entries)| {
                                entries
                                    .iter()
                                    .find(|e| e.remediation_task_id.as_ref() == Some(&task_id))
                                    .map(|_| orig_id.clone())
                            });

                        if let Some(orig_task_id) = original_task_id {
                            info!("Remediation task {} completed, scheduling re-review of original task {}", 
                                  task_id, orig_task_id);

                            // Mark the remediation as complete in history
                            if let Some(entries) = s.review_history.get_mut(&orig_task_id) {
                                if let Some(entry) = entries
                                    .iter_mut()
                                    .find(|e| e.remediation_task_id.as_ref() == Some(&task_id))
                                {
                                    entry.review_passed = true;
                                }
                            }

                            // TODO: Schedule immediate re-review of the original task
                            // For now, the periodic review will catch it
                        }
                    }
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
            AgentMessage::QualityIssue {
                agent_id,
                task_id,
                issues,
            } => {
                error!(
                    "Quality issues found in task {} by agent {}: {:?}",
                    task_id, agent_id, issues
                );

                // Create an enhanced remediation task with LLM-generated instructions
                let remediation_task_id = format!("remediate-{}-{}", task_id, Uuid::new_v4());

                // Get agent role for proper instruction generation
                let agent_role = if let Some(agent) = agents.get(&agent_id) {
                    agent.identity.specialization.name().to_string()
                } else {
                    "Unknown".to_string()
                };

                // Generate fix instructions from the issues
                let detailed_instructions = {
                    let judge_lock = judge.read().await;
                    // Convert string issues to QualityIssue objects for instruction generation
                    let quality_issues: Vec<llm_quality_judge::QualityIssue> = issues
                        .iter()
                        .map(|issue_str| {
                            let (category, description, fix) = match issue_str.as_str() {
                                s if s.contains("Low test coverage") => (
                                    llm_quality_judge::IssueCategory::TestCoverage,
                                    "Test coverage below requirements",
                                    "Add unit tests to achieve 85% coverage",
                                ),
                                s if s.contains("High complexity") => (
                                    llm_quality_judge::IssueCategory::CodeComplexity,
                                    "Code complexity too high",
                                    "Refactor to reduce cyclomatic complexity",
                                ),
                                s if s.contains("Security") => (
                                    llm_quality_judge::IssueCategory::Security,
                                    "Security issues detected",
                                    "Fix security vulnerabilities and validate inputs",
                                ),
                                s if s.contains("documentation") => (
                                    llm_quality_judge::IssueCategory::Documentation,
                                    "Documentation issues",
                                    "Add comprehensive documentation",
                                ),
                                s if s.contains("error handling") => (
                                    llm_quality_judge::IssueCategory::ErrorHandling,
                                    "Error handling issues",
                                    "Add comprehensive error handling",
                                ),
                                _ => (
                                    llm_quality_judge::IssueCategory::BestPractices,
                                    issue_str.as_str(),
                                    "Review and fix the reported issue",
                                ),
                            };

                            llm_quality_judge::QualityIssue {
                                severity: llm_quality_judge::IssueSeverity::High,
                                category,
                                description: description.to_string(),
                                suggested_fix: fix.to_string(),
                                affected_areas: vec![],
                                fix_effort: 30,
                            }
                        })
                        .collect();

                    judge_lock.generate_fix_instructions(&quality_issues, &agent_role)
                };

                let remediation_task = Task::new(
                    remediation_task_id,
                    format!(
                        "Fix quality issues in task {}: {}",
                        task_id,
                        issues.join(", ")
                    ),
                    crate::agent::Priority::High,
                    TaskType::Remediation,
                )
                .with_details(detailed_instructions)
                .assign_to(agent_id.clone())
                .with_parent_task(task_id.clone())
                .with_quality_issues(issues.clone())
                .with_duration(1800);

                // Send remediation task to the task queue
                task_queue_tx
                    .send(remediation_task.clone())
                    .await
                    .context("Failed to queue remediation task")?;

                // Track in review history
                let mut s = state.write().await;
                let review_entry = ReviewHistoryEntry {
                    task_id: task_id.clone(),
                    agent_id: agent_id.clone(),
                    review_date: Utc::now(),
                    issues_found: issues.clone(),
                    remediation_task_id: Some(remediation_task.id.clone()),
                    review_passed: false,
                    iteration: s
                        .review_history
                        .get(&task_id)
                        .map(|entries| entries.len() as u32 + 1)
                        .unwrap_or(1),
                };

                s.review_history
                    .entry(task_id.clone())
                    .or_insert_with(Vec::new)
                    .push(review_entry);

                info!("Created remediation task {} for agent {} to fix issues in task {} with detailed instructions", 
                     remediation_task.id, agent_id, task_id);
            }
            _ => {}
        }

        Ok(())
    }

    /// Perform LLM-based quality review on agent work
    async fn perform_llm_quality_review(
        agents: &DashMap<String, ClaudeCodeAgent>,
        standards: &QualityStandards,
        bus: &CoordinationBus,
        judge: &Arc<RwLock<LLMQualityJudge>>,
        _worktree_manager: &Arc<WorktreeManager>,
    ) -> Result<()> {
        info!("Performing LLM-based quality review");

        for entry in agents.iter() {
            let agent = entry.value();

            // Skip if agent hasn't completed any tasks
            if agent.task_history.is_empty() {
                continue;
            }

            // Review recent completed tasks
            for (task, result) in agent.task_history.iter().rev().take(5) {
                if result.success {
                    // Get agent workspace path
                    let workspace_path = agent.identity.workspace_path.clone();

                    // Perform LLM-based quality evaluation
                    let mut judge_guard = judge.write().await;
                    match judge_guard
                        .evaluate_task(
                            task,
                            result,
                            &agent.identity.specialization,
                            &workspace_path.to_string_lossy(),
                        )
                        .await
                    {
                        Ok(evaluation) => {
                            info!(
                                "Quality evaluation for task {}: score={:.2}, passes={}",
                                task.id, evaluation.overall_score, evaluation.passes_standards
                            );

                            // Check if quality standards are met
                            if !evaluation.passes_standards
                                || evaluation.overall_score < standards.min_test_coverage
                            {
                                // Convert evaluation to issues
                                let issues = judge_guard.evaluation_to_issues(&evaluation);

                                if !issues.is_empty() {
                                    warn!(
                                        "Task {} from agent {} failed quality standards: score={:.2}, confidence={:.2}",
                                        task.id, agent.identity.agent_id, evaluation.overall_score, evaluation.confidence
                                    );

                                    // Log detailed feedback
                                    info!("Quality feedback: {}", evaluation.feedback);

                                    // Send quality issue message with detailed information
                                    bus.send_message(AgentMessage::QualityIssue {
                                        agent_id: agent.identity.agent_id.clone(),
                                        task_id: task.id.clone(),
                                        issues,
                                    })
                                    .await?;

                                    // Store the detailed evaluation for remediation task creation
                                    // This could be enhanced to pass the full evaluation through the message
                                }
                            } else {
                                info!(
                                    "Task {} from agent {} passed quality review with score {:.2}",
                                    task.id, agent.identity.agent_id, evaluation.overall_score
                                );
                            }
                        }
                        Err(e) => {
                            error!(
                                "Failed to evaluate task {} for agent {}: {}",
                                task.id, agent.identity.agent_id, e
                            );
                            // Continue with next task
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Legacy quality review function (kept for backward compatibility)
    #[allow(dead_code)]
    async fn perform_quality_review(
        agents: &DashMap<String, ClaudeCodeAgent>,
        standards: &QualityStandards,
        bus: &CoordinationBus,
    ) -> Result<()> {
        info!("Performing basic quality review");

        for entry in agents.iter() {
            let agent = entry.value();

            // Skip if agent hasn't completed any tasks
            if agent.task_history.is_empty() {
                continue;
            }

            // Review recent completed tasks
            for (task, result) in agent.task_history.iter().rev().take(5) {
                if result.success {
                    // Basic placeholder logic
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
