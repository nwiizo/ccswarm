use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, sleep};
use tracing::{error, info};

use super::task_queue::{QueuedTask, TaskQueue};
use crate::agent::orchestrator::AgentOrchestrator;
use crate::agent::pool::AgentPool;
use crate::agent::{Task, TaskResult};
use crate::config::CcswarmConfig;
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

        Ok(Self {
            task_queue,
            agent_pool,
            delegation_engine,
            stats,
            execution_history: Arc::new(RwLock::new(Vec::new())),
            max_concurrent_tasks: 5, // Configurable
            active_executions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Add a task to the execution queue
    pub async fn add_task(&self, task: Task) -> String {
        info!("Adding task to execution queue: {}", task.description);
        self.task_queue.add_task(task).await
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

        tokio::spawn(async move {
            Self::execution_loop(
                task_queue,
                agent_pool,
                delegation_engine,
                stats,
                execution_history,
                active_executions,
                max_concurrent,
            )
            .await;
        });

        Ok(())
    }

    /// Main execution loop
    async fn execution_loop(
        task_queue: Arc<TaskQueue>,
        agent_pool: Arc<Mutex<AgentPool>>,
        delegation_engine: Arc<Mutex<MasterDelegationEngine>>,
        stats: Arc<RwLock<ExecutionStats>>,
        execution_history: Arc<RwLock<Vec<ExecutionResult>>>,
        active_executions: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
        max_concurrent: usize,
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

                // Spawn execution task
                let handle = tokio::spawn(async move {
                    Self::execute_single_task(
                        task_clone,
                        task_queue_clone,
                        agent_pool_clone,
                        delegation_engine_clone,
                        stats_clone,
                        execution_history_clone,
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

    /// Execute a single task
    async fn execute_single_task(
        queued_task: QueuedTask,
        task_queue: Arc<TaskQueue>,
        agent_pool: Arc<Mutex<AgentPool>>,
        delegation_engine: Arc<Mutex<MasterDelegationEngine>>,
        stats: Arc<RwLock<ExecutionStats>>,
        execution_history: Arc<RwLock<Vec<ExecutionResult>>>,
    ) {
        let task_id = queued_task.task.id.clone();
        let start_time = Instant::now();

        info!(
            "Executing task: {} - {}",
            task_id, queued_task.task.description
        );

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
    }

    /// Execute task with orchestration
    async fn execute_with_orchestration(
        task: &Task,
        agent_pool: &Arc<Mutex<AgentPool>>,
        _agent_id: &str,
    ) -> anyhow::Result<TaskResult> {
        let pool = agent_pool.lock().await;

        // Use the orchestrator interface
        match pool.orchestrate_task(task).await {
            Ok(result) => Ok(result),
            Err(e) => Err(anyhow::anyhow!("Orchestration failed: {}", e)),
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
            Err(e) => Err(anyhow::anyhow!("Direct execution failed: {}", e)),
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
