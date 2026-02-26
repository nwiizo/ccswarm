use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, sleep};
use tracing::{error, info, warn};

use super::task_queue::{QueuedTask, TaskQueue};
use crate::agent::orchestrator::AgentOrchestrator;
use crate::agent::pool::AgentPool;
use crate::agent::{Task, TaskResult};
use crate::config::CcswarmConfig;
use crate::git::shell::ShellWorktreeManager;
use crate::orchestrator::master_delegation::MasterDelegationEngine;

/// Result of task execution
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecutionResult {
    pub task_id: String,
    pub success: bool,
    pub result: Option<TaskResult>,
    pub error: Option<String>,
    pub duration: Duration,
    pub agent_used: Option<String>,
    pub orchestration_used: bool,
}

/// Statistics for task execution
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecutionStats {
    pub tasks_executed: usize,
    pub tasks_succeeded: usize,
    pub tasks_failed: usize,
    pub average_duration: Duration,
    pub total_duration: Duration,
    pub orchestration_usage: f64, // Percentage of tasks that used orchestration
}

/// Worktree isolation configuration for task execution
#[derive(Clone)]
struct WorktreeConfig {
    /// Repository path for creating worktrees
    repo_path: Option<PathBuf>,
    /// Whether worktree isolation is enabled
    enabled: bool,
}

/// Information about a task-specific worktree
#[allow(dead_code)]
struct TaskWorktreeInfo {
    /// Path to the worktree
    path: PathBuf,
    /// Branch name created for this task (used for merge operations)
    branch: String,
    /// Path to the parent repository
    repo_path: PathBuf,
}

/// Main task execution engine
pub struct TaskExecutor {
    /// Task queue for managing tasks
    task_queue: Arc<TaskQueue>,
    /// Agent pool for task execution
    agent_pool: Arc<Mutex<AgentPool>>,
    /// Master delegation engine
    delegation_engine: Arc<Mutex<MasterDelegationEngine>>,
    /// Execution statistics
    stats: Arc<RwLock<ExecutionStats>>,
    /// Execution history
    execution_history: Arc<RwLock<Vec<ExecutionResult>>>,
    /// Maximum concurrent tasks
    max_concurrent_tasks: usize,
    /// Currently executing tasks
    active_executions: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
    /// Repository path for worktree isolation
    repo_path: Option<PathBuf>,
    /// Whether worktree isolation is enabled
    worktree_isolation: bool,
}

impl TaskExecutor {
    pub async fn new(config: &CcswarmConfig) -> anyhow::Result<Self> {
        let task_queue = Arc::new(TaskQueue::new());

        // Create agent pool and spawn configured agents
        let mut agent_pool = AgentPool::new().await?;
        for agent_type in config.agents.keys() {
            if let Err(e) = agent_pool.spawn_agent(agent_type, config).await {
                error!("Failed to spawn {} agent: {}", agent_type, e);
                // Continue with other agents rather than failing completely
            } else {
                info!("Successfully spawned {} agent", agent_type);
            }
        }
        let agent_pool = Arc::new(Mutex::new(agent_pool));

        let delegation_engine = Arc::new(Mutex::new(MasterDelegationEngine::new(
            crate::orchestrator::master_delegation::DelegationStrategy::Hybrid,
        )));

        let stats = Arc::new(RwLock::new(ExecutionStats {
            tasks_executed: 0,
            tasks_succeeded: 0,
            tasks_failed: 0,
            average_duration: Duration::from_secs(0),
            total_duration: Duration::from_secs(0),
            orchestration_usage: 0.0,
        }));

        // Determine repo path for worktree isolation
        let repo_path = config
            .project
            .repository
            .local_path
            .clone()
            .or_else(|| std::env::current_dir().ok());
        let worktree_isolation = config.project.repository.worktree_isolation;

        Ok(Self {
            task_queue,
            agent_pool,
            delegation_engine,
            stats,
            execution_history: Arc::new(RwLock::new(Vec::new())),
            max_concurrent_tasks: 5, // Configurable
            active_executions: Arc::new(RwLock::new(HashMap::new())),
            repo_path,
            worktree_isolation,
        })
    }

    /// Add a task to the execution queue
    pub async fn add_task(&self, task: Task) -> String {
        info!("Adding task to execution queue: {}", task.description);
        self.task_queue.add_task(task).await
    }

    /// Enable worktree isolation for this executor
    pub fn enable_worktree_isolation(&mut self, repo_path: PathBuf) {
        self.repo_path = Some(repo_path);
        self.worktree_isolation = true;
    }

    /// Start the execution engine
    pub async fn start(&self) -> anyhow::Result<()> {
        info!("Starting task execution engine");

        // Start task processing loop
        let task_queue = self.task_queue.clone();
        let agent_pool = self.agent_pool.clone();
        let delegation_engine = self.delegation_engine.clone();
        let stats = self.stats.clone();
        let execution_history = self.execution_history.clone();
        let active_executions = self.active_executions.clone();
        let max_concurrent = self.max_concurrent_tasks;
        let wt_config = Arc::new(WorktreeConfig {
            repo_path: self.repo_path.clone(),
            enabled: self.worktree_isolation,
        });

        tokio::spawn(async move {
            Self::execution_loop(
                task_queue,
                agent_pool,
                delegation_engine,
                stats,
                execution_history,
                active_executions,
                max_concurrent,
                wt_config,
            )
            .await;
        });

        Ok(())
    }

    /// Main execution loop
    #[allow(clippy::too_many_arguments)]
    async fn execution_loop(
        task_queue: Arc<TaskQueue>,
        agent_pool: Arc<Mutex<AgentPool>>,
        delegation_engine: Arc<Mutex<MasterDelegationEngine>>,
        stats: Arc<RwLock<ExecutionStats>>,
        execution_history: Arc<RwLock<Vec<ExecutionResult>>>,
        active_executions: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
        max_concurrent: usize,
        wt_config: Arc<WorktreeConfig>,
    ) {
        let mut interval = interval(Duration::from_secs(1)); // Check every second

        loop {
            interval.tick().await;

            // Clean up completed tasks
            Self::cleanup_completed_tasks(&active_executions).await;

            // Check if we can start new tasks
            let active_count = {
                let active = active_executions.read().await;
                active.len()
            };

            if active_count >= max_concurrent {
                continue; // Wait for some tasks to complete
            }

            // Get next task from queue
            if let Some(queued_task) = task_queue.get_next_task().await {
                info!("Processing task: {}", queued_task.task.description);

                // Clone necessary data for the execution task
                let task_queue_clone = task_queue.clone();
                let agent_pool_clone = agent_pool.clone();
                let delegation_engine_clone = delegation_engine.clone();
                let stats_clone = stats.clone();
                let execution_history_clone = execution_history.clone();
                let task_clone = queued_task.clone();
                let wt_config_clone = wt_config.clone();

                // Spawn execution task
                let handle = tokio::spawn(async move {
                    Self::execute_single_task(
                        task_clone,
                        task_queue_clone,
                        agent_pool_clone,
                        delegation_engine_clone,
                        stats_clone,
                        execution_history_clone,
                        &wt_config_clone,
                    )
                    .await;
                });

                // Store the handle
                {
                    let mut active = active_executions.write().await;
                    active.insert(queued_task.task.id.clone(), handle);
                }
            }

            // Small delay to prevent busy waiting
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Execute a single task with optional worktree isolation
    async fn execute_single_task(
        queued_task: QueuedTask,
        task_queue: Arc<TaskQueue>,
        agent_pool: Arc<Mutex<AgentPool>>,
        delegation_engine: Arc<Mutex<MasterDelegationEngine>>,
        stats: Arc<RwLock<ExecutionStats>>,
        execution_history: Arc<RwLock<Vec<ExecutionResult>>>,
        wt_config: &WorktreeConfig,
    ) {
        let task_id = queued_task.task.id.clone();
        let start_time = Instant::now();

        info!(
            "Executing task: {} - {}",
            task_id, queued_task.task.description
        );

        // Setup worktree isolation if enabled
        let worktree_info = if wt_config.enabled {
            if let Some(ref repo) = wt_config.repo_path {
                match Self::setup_task_worktree(repo, &task_id).await {
                    Ok(info) => {
                        info!(
                            "Created worktree for task {}: {}",
                            task_id,
                            info.path.display()
                        );
                        Some(info)
                    }
                    Err(e) => {
                        warn!(
                            "Failed to create worktree for task {}, continuing without isolation: {}",
                            task_id, e
                        );
                        None
                    }
                }
            } else {
                warn!("Worktree isolation enabled but no repo_path configured, skipping");
                None
            }
        } else {
            None
        };

        // Determine best agent for the task
        let (agent_id, use_orchestration) = {
            let mut engine = delegation_engine.lock().await;
            match engine.delegate_task(queued_task.task.clone()) {
                Ok(decision) => {
                    let agent_name = decision.target_agent.name().to_lowercase();
                    let complex = Self::is_complex_task(&queued_task.task);
                    (agent_name, complex)
                }
                Err(e) => {
                    error!("Delegation failed for task {}: {}", task_id, e);
                    // Cleanup worktree on failure
                    if let Some(ref wt) = worktree_info {
                        Self::cleanup_task_worktree(wt).await;
                    }
                    Self::record_execution_failure(
                        &task_queue,
                        &stats,
                        &execution_history,
                        &task_id,
                        format!("Delegation failed: {}", e),
                        start_time.elapsed(),
                    )
                    .await;
                    return;
                }
            }
        };

        // Verify agent exists in pool before execution
        {
            let pool = agent_pool.lock().await;
            if !pool.has_agent(&agent_id) {
                error!(
                    "Delegated task '{}' to agent '{}' but agent not found in pool",
                    task_id, agent_id
                );
                // Cleanup worktree on failure
                if let Some(ref wt) = worktree_info {
                    Self::cleanup_task_worktree(wt).await;
                }
                Self::record_execution_failure(
                    &task_queue,
                    &stats,
                    &execution_history,
                    &task_id,
                    format!("Agent '{}' not found in pool", agent_id),
                    start_time.elapsed(),
                )
                .await;
                return;
            }
        }

        // Mark task as assigned
        if let Err(e) = task_queue.assign_task(&task_id, &agent_id).await {
            error!("Failed to assign task {}: {}", task_id, e);
            return;
        }

        // Start task execution
        if let Err(e) = task_queue.start_task_execution(&task_id, &agent_id).await {
            error!("Failed to start task execution {}: {}", task_id, e);
            return;
        }

        // Execute the task
        let execution_result = if use_orchestration {
            info!("Using orchestration for complex task: {}", task_id);
            Self::execute_with_orchestration(&queued_task.task, &agent_pool, &agent_id).await
        } else {
            info!("Using direct execution for task: {}", task_id);
            Self::execute_directly(&queued_task.task, &agent_pool, &agent_id).await
        };

        let duration = start_time.elapsed();

        match execution_result {
            Ok(result) => {
                info!("Task {} completed successfully", task_id);

                // Mark task as completed
                if let Err(e) = task_queue.complete_task(&task_id, result.clone()).await {
                    error!("Failed to mark task as completed {}: {}", task_id, e);
                }

                // Record success
                Self::record_execution_success(
                    &stats,
                    &execution_history,
                    &task_id,
                    result,
                    duration,
                    &agent_id,
                    use_orchestration,
                )
                .await;

                // Auto-create PR if task ran in a worktree
                if let Some(ref wt) = worktree_info {
                    Self::auto_create_pr(&wt.branch, &queued_task.task.description, &wt.repo_path)
                        .await;
                }
            }
            Err(e) => {
                error!("Task {} failed: {}", task_id, e);

                // Mark task as failed
                if let Err(err) = task_queue.fail_task(&task_id, e.to_string()).await {
                    error!("Failed to mark task as failed {}: {}", task_id, err);
                }

                // Record failure
                Self::record_execution_failure(
                    &task_queue,
                    &stats,
                    &execution_history,
                    &task_id,
                    e.to_string(),
                    duration,
                )
                .await;
            }
        }

        // Cleanup worktree after execution
        if let Some(ref wt) = worktree_info {
            Self::cleanup_task_worktree(wt).await;
        }
    }

    /// Setup a git worktree for isolated task execution
    async fn setup_task_worktree(
        repo_path: &Path,
        task_id: &str,
    ) -> anyhow::Result<TaskWorktreeInfo> {
        let manager = ShellWorktreeManager::new(repo_path.to_path_buf())?;

        // Sanitize task_id for branch name
        let safe_id: String = task_id
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        let branch_name = format!("task/{}", safe_id);

        // Determine worktree path
        let worktrees_base = repo_path
            .parent()
            .map(|p| p.join(".ccswarm-worktrees"))
            .unwrap_or_else(|| repo_path.join(".ccswarm-worktrees"));

        let worktree_path = worktrees_base.join(&safe_id);

        // Create worktrees directory if needed
        tokio::fs::create_dir_all(&worktrees_base).await?;

        // Create the worktree
        let _wt = manager
            .create_worktree(&worktree_path, &branch_name)
            .await?;

        Ok(TaskWorktreeInfo {
            path: worktree_path,
            branch: branch_name,
            repo_path: repo_path.to_path_buf(),
        })
    }

    /// Cleanup a task worktree after execution
    async fn cleanup_task_worktree(info: &TaskWorktreeInfo) {
        if let Ok(manager) = ShellWorktreeManager::new(info.repo_path.clone()) {
            if let Err(e) = manager.remove_worktree(&info.path).await {
                warn!(
                    "Failed to remove worktree at {}: {}",
                    info.path.display(),
                    e
                );
            } else {
                info!("Cleaned up worktree at {}", info.path.display());
            }
        }
    }

    /// Auto-create a pull request for a completed task's branch
    async fn auto_create_pr(branch: &str, task_description: &str, repo_path: &Path) {
        // First check if `gh` CLI is available
        let gh_available = tokio::process::Command::new("gh")
            .arg("--version")
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !gh_available {
            info!(
                "GitHub CLI not available, skipping auto-PR for branch '{}'",
                branch
            );
            return;
        }

        // Push the branch first
        let push_result = tokio::process::Command::new("git")
            .args(["push", "-u", "origin", branch])
            .current_dir(repo_path)
            .output()
            .await;

        if let Err(e) = push_result {
            warn!("Failed to push branch '{}': {}", branch, e);
            return;
        }

        // Truncate description for PR title
        let title = if task_description.len() > 70 {
            format!("{}...", &task_description[..67])
        } else {
            task_description.to_string()
        };

        let body = format!(
            "## Summary\n\n- {}\n\n## Notes\n\nAuto-created by ccswarm task executor.\nBranch: `{}`",
            task_description, branch
        );

        let pr_result = tokio::process::Command::new("gh")
            .args([
                "pr", "create", "--title", &title, "--body", &body, "--head", branch,
            ])
            .current_dir(repo_path)
            .output()
            .await;

        match pr_result {
            Ok(output) if output.status.success() => {
                let url = String::from_utf8_lossy(&output.stdout);
                info!("Auto-created PR for task: {}", url.trim());
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                warn!("Failed to create PR: {}", stderr.trim());
            }
            Err(e) => {
                warn!("Failed to run gh pr create: {}", e);
            }
        }
    }

    /// Execute task with orchestration
    async fn execute_with_orchestration(
        task: &Task,
        agent_pool: &Arc<Mutex<AgentPool>>,
        agent_id: &str,
    ) -> anyhow::Result<TaskResult> {
        let pool = agent_pool.lock().await;

        match pool.orchestrate_task(task).await {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow::anyhow!(
                "Orchestration failed for task '{}' (agent '{}'): {}",
                task.id,
                agent_id,
                e
            )),
        }
    }

    /// Execute task directly with an agent
    async fn execute_directly(
        task: &Task,
        agent_pool: &Arc<Mutex<AgentPool>>,
        agent_id: &str,
    ) -> anyhow::Result<TaskResult> {
        let pool = agent_pool.lock().await;

        match pool.execute_task_with_agent(agent_id, task).await {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow::anyhow!(
                "Direct execution failed for task '{}' (agent '{}'): {}",
                task.id,
                agent_id,
                e
            )),
        }
    }

    /// Determine if a task is complex enough to need orchestration
    fn is_complex_task(task: &Task) -> bool {
        let description = task.description.to_lowercase();
        let details = task.details.as_deref().unwrap_or("").to_lowercase();
        let combined = format!("{} {}", description, details);

        // Complex indicators
        let complexity_keywords = [
            "implement",
            "create",
            "build",
            "design",
            "develop",
            "integrate",
            "migrate",
            "refactor",
            "comprehensive",
            "multiple",
            "several",
            "complete",
            "full",
            "and",
            "then",
            "also",
            "plus",
            "step",
        ];

        let keyword_count = complexity_keywords
            .iter()
            .filter(|&keyword| combined.contains(keyword))
            .count();

        // High priority tasks or those with many complexity indicators
        keyword_count >= 3
            || matches!(
                task.priority,
                crate::agent::Priority::High | crate::agent::Priority::Critical
            )
    }

    /// Record successful execution
    async fn record_execution_success(
        stats: &Arc<RwLock<ExecutionStats>>,
        execution_history: &Arc<RwLock<Vec<ExecutionResult>>>,
        task_id: &str,
        result: TaskResult,
        duration: Duration,
        agent_id: &str,
        orchestration_used: bool,
    ) {
        let execution_result = ExecutionResult {
            task_id: task_id.to_string(),
            success: true,
            result: Some(result),
            error: None,
            duration,
            agent_used: Some(agent_id.to_string()),
            orchestration_used,
        };

        // Update stats
        {
            let mut stats_guard = stats.write().await;
            stats_guard.tasks_executed += 1;
            stats_guard.tasks_succeeded += 1;
            stats_guard.total_duration += duration;
            stats_guard.average_duration =
                stats_guard.total_duration / stats_guard.tasks_executed as u32;

            let orchestration_count = execution_history
                .read()
                .await
                .iter()
                .filter(|r| r.orchestration_used)
                .count()
                + if orchestration_used { 1 } else { 0 };
            stats_guard.orchestration_usage =
                (orchestration_count as f64) / (stats_guard.tasks_executed as f64) * 100.0;
        }

        // Add to history
        {
            let mut history = execution_history.write().await;
            history.push(execution_result);

            // Keep history size manageable
            if history.len() > 1000 {
                history.drain(0..100); // Remove oldest 100 entries
            }
        }
    }

    /// Record failed execution
    async fn record_execution_failure(
        _task_queue: &Arc<TaskQueue>,
        stats: &Arc<RwLock<ExecutionStats>>,
        execution_history: &Arc<RwLock<Vec<ExecutionResult>>>,
        task_id: &str,
        error: String,
        duration: Duration,
    ) {
        let execution_result = ExecutionResult {
            task_id: task_id.to_string(),
            success: false,
            result: None,
            error: Some(error),
            duration,
            agent_used: None,
            orchestration_used: false,
        };

        // Update stats
        {
            let mut stats_guard = stats.write().await;
            stats_guard.tasks_executed += 1;
            stats_guard.tasks_failed += 1;
            stats_guard.total_duration += duration;
            if stats_guard.tasks_executed > 0 {
                stats_guard.average_duration =
                    stats_guard.total_duration / stats_guard.tasks_executed as u32;
            }
        }

        // Add to history
        {
            let mut history = execution_history.write().await;
            history.push(execution_result);

            if history.len() > 1000 {
                history.drain(0..100);
            }
        }
    }

    /// Clean up completed task handles
    async fn cleanup_completed_tasks(
        active_executions: &Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
    ) {
        let mut to_remove = Vec::new();

        {
            let active = active_executions.read().await;
            for (task_id, handle) in active.iter() {
                if handle.is_finished() {
                    to_remove.push(task_id.clone());
                }
            }
        }

        if !to_remove.is_empty() {
            let mut active = active_executions.write().await;
            for task_id in to_remove {
                active.remove(&task_id);
            }
        }
    }

    /// Get task queue reference
    pub fn get_task_queue(&self) -> &Arc<TaskQueue> {
        &self.task_queue
    }

    /// Get execution statistics
    pub async fn get_stats(&self) -> ExecutionStats {
        self.stats.read().await.clone()
    }

    /// Get execution history
    pub async fn get_execution_history(&self, limit: Option<usize>) -> Vec<ExecutionResult> {
        let history = self.execution_history.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.clone(),
        }
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: &str, reason: Option<String>) -> anyhow::Result<()> {
        // Cancel in queue
        self.task_queue.cancel_task(task_id, reason).await?;

        // Cancel active execution if running
        let mut active = self.active_executions.write().await;
        if let Some(handle) = active.remove(task_id) {
            handle.abort();
        }

        Ok(())
    }
}

/// Global execution engine instance
#[derive(Clone)]
pub struct ExecutionEngine {
    executor: Arc<TaskExecutor>,
}

impl ExecutionEngine {
    pub async fn new(config: &CcswarmConfig) -> anyhow::Result<Self> {
        let executor = Arc::new(TaskExecutor::new(config).await?);
        Ok(Self { executor })
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        self.executor.start().await
    }

    pub fn get_executor(&self) -> &Arc<TaskExecutor> {
        &self.executor
    }
}
