//! Parallel Executor for Subagent Tasks
//!
//! Manages parallel execution of tasks across multiple subagents
//! with result aggregation and error handling.

use super::{SubagentResult, spawner::SpawnTask};
use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;

/// Configuration for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelConfig {
    /// Maximum concurrent executions
    pub max_concurrent: usize,
    /// Default timeout per task (ms)
    pub default_timeout_ms: u64,
    /// Whether to fail fast on first error
    pub fail_fast: bool,
    /// Whether to retry failed tasks
    pub retry_failed: bool,
    /// Maximum retries per task
    pub max_retries: u32,
    /// Delay between retries (ms)
    pub retry_delay_ms: u64,
    /// Whether to collect partial results on timeout
    pub collect_partial_on_timeout: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 5,
            default_timeout_ms: 300_000, // 5 minutes
            fail_fast: false,
            retry_failed: true,
            max_retries: 2,
            retry_delay_ms: 1000,
            collect_partial_on_timeout: true,
        }
    }
}

/// Status of a parallel execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Execution is pending
    Pending,
    /// Execution is in progress
    Running,
    /// Execution completed successfully
    Completed,
    /// Execution failed
    Failed,
    /// Execution was cancelled
    Cancelled,
    /// Execution timed out
    TimedOut,
}

/// Result of a single task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    /// Task ID
    pub task_id: String,
    /// Agent instance ID
    pub agent_id: Option<String>,
    /// Execution status
    pub status: ExecutionStatus,
    /// Task result (if successful)
    pub result: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution duration (ms)
    pub duration_ms: u64,
    /// Number of retries attempted
    pub retries: u32,
}

impl TaskExecutionResult {
    /// Create a successful result
    pub fn success(
        task_id: &str,
        agent_id: &str,
        result: serde_json::Value,
        duration_ms: u64,
    ) -> Self {
        Self {
            task_id: task_id.to_string(),
            agent_id: Some(agent_id.to_string()),
            status: ExecutionStatus::Completed,
            result: Some(result),
            error: None,
            duration_ms,
            retries: 0,
        }
    }

    /// Create a failed result
    pub fn failure(task_id: &str, error: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            task_id: task_id.to_string(),
            agent_id: None,
            status: ExecutionStatus::Failed,
            result: None,
            error: Some(error.into()),
            duration_ms,
            retries: 0,
        }
    }

    /// Create a timeout result
    pub fn timeout(task_id: &str, duration_ms: u64) -> Self {
        Self {
            task_id: task_id.to_string(),
            agent_id: None,
            status: ExecutionStatus::TimedOut,
            result: None,
            error: Some("Task timed out".to_string()),
            duration_ms,
            retries: 0,
        }
    }

    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.status == ExecutionStatus::Completed
    }
}

/// Result of parallel execution batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelExecutionResult {
    /// Execution ID
    pub execution_id: String,
    /// Overall status
    pub status: ExecutionStatus,
    /// Individual task results
    pub task_results: Vec<TaskExecutionResult>,
    /// Total duration (ms)
    pub total_duration_ms: u64,
    /// Number of successful tasks
    pub successful_count: usize,
    /// Number of failed tasks
    pub failed_count: usize,
    /// Aggregated result (if applicable)
    pub aggregated_result: Option<serde_json::Value>,
}

impl ParallelExecutionResult {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.task_results.len();
        if total == 0 {
            return 0.0;
        }
        self.successful_count as f64 / total as f64
    }

    /// Get all successful results
    pub fn successful_results(&self) -> Vec<&TaskExecutionResult> {
        self.task_results
            .iter()
            .filter(|r| r.is_success())
            .collect()
    }

    /// Get all failed results
    pub fn failed_results(&self) -> Vec<&TaskExecutionResult> {
        self.task_results
            .iter()
            .filter(|r| !r.is_success())
            .collect()
    }
}

/// Strategy for aggregating results
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AggregationStrategy {
    /// Collect all results into an array
    CollectAll,
    /// Merge objects together
    MergeObjects,
    /// Take first successful result
    FirstSuccess,
    /// Take result with highest confidence
    HighestConfidence,
    /// Custom aggregation (use callback)
    Custom,
}

/// Type alias for task executor function
pub type TaskExecutorFn = Box<
    dyn Fn(
            SpawnTask,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = SubagentResult<serde_json::Value>> + Send>,
        > + Send
        + Sync,
>;

/// Parallel executor for managing concurrent task execution
pub struct ParallelExecutor {
    /// Configuration
    config: ParallelConfig,
    /// Concurrency semaphore
    semaphore: Arc<Semaphore>,
    /// Active executions
    active_executions: Arc<RwLock<HashMap<String, ExecutionStatus>>>,
}

impl ParallelExecutor {
    /// Create a new parallel executor
    pub fn new(config: ParallelConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));
        Self {
            config,
            semaphore,
            active_executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ParallelConfig::default())
    }

    /// Execute tasks in parallel with a simple executor function
    pub async fn execute_parallel<F, Fut>(
        &self,
        tasks: Vec<SpawnTask>,
        executor: F,
    ) -> SubagentResult<ParallelExecutionResult>
    where
        F: Fn(SpawnTask) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = SubagentResult<serde_json::Value>> + Send + 'static,
    {
        let execution_id = uuid::Uuid::new_v4().to_string();
        let start = std::time::Instant::now();

        // Register execution
        {
            let mut active = self.active_executions.write().await;
            active.insert(execution_id.clone(), ExecutionStatus::Running);
        }

        let task_timeout = Duration::from_millis(self.config.default_timeout_ms);
        let mut futures = FuturesUnordered::new();

        for task in tasks {
            let semaphore = Arc::clone(&self.semaphore);
            let executor = executor.clone();
            let task_id = task.id.clone();

            let future = async move {
                // Acquire semaphore permit
                let _permit = semaphore.acquire().await;
                let task_start = std::time::Instant::now();

                let result = timeout(task_timeout, executor(task)).await;

                let duration_ms = task_start.elapsed().as_millis() as u64;

                match result {
                    Ok(Ok(value)) => TaskExecutionResult {
                        task_id,
                        agent_id: None,
                        status: ExecutionStatus::Completed,
                        result: Some(value),
                        error: None,
                        duration_ms,
                        retries: 0,
                    },
                    Ok(Err(e)) => TaskExecutionResult {
                        task_id,
                        agent_id: None,
                        status: ExecutionStatus::Failed,
                        result: None,
                        error: Some(e.to_string()),
                        duration_ms,
                        retries: 0,
                    },
                    Err(_) => TaskExecutionResult::timeout(&task_id, duration_ms),
                }
            };

            futures.push(future);
        }

        // Collect results
        let mut task_results = Vec::new();
        let mut first_error = None;

        while let Some(result) = futures.next().await {
            if self.config.fail_fast && !result.is_success() && first_error.is_none() {
                first_error = Some(result.error.clone().unwrap_or_default());
            }
            task_results.push(result);
        }

        let total_duration_ms = start.elapsed().as_millis() as u64;
        let successful_count = task_results.iter().filter(|r| r.is_success()).count();
        let failed_count = task_results.len() - successful_count;

        let status = if first_error.is_some() && self.config.fail_fast {
            ExecutionStatus::Failed
        } else {
            ExecutionStatus::Completed // Includes partial success when failed_count > 0
        };

        // Update active execution status
        {
            let mut active = self.active_executions.write().await;
            active.insert(execution_id.clone(), status);
        }

        Ok(ParallelExecutionResult {
            execution_id,
            status,
            task_results,
            total_duration_ms,
            successful_count,
            failed_count,
            aggregated_result: None,
        })
    }

    /// Execute with result aggregation
    pub async fn execute_with_aggregation<F, Fut>(
        &self,
        tasks: Vec<SpawnTask>,
        executor: F,
        strategy: AggregationStrategy,
    ) -> SubagentResult<ParallelExecutionResult>
    where
        F: Fn(SpawnTask) -> Fut + Send + Sync + Clone + 'static,
        Fut: std::future::Future<Output = SubagentResult<serde_json::Value>> + Send + 'static,
    {
        let mut result = self.execute_parallel(tasks, executor).await?;

        // Aggregate results
        result.aggregated_result = Some(self.aggregate_results(&result.task_results, strategy));

        Ok(result)
    }

    /// Aggregate results based on strategy
    fn aggregate_results(
        &self,
        results: &[TaskExecutionResult],
        strategy: AggregationStrategy,
    ) -> serde_json::Value {
        match strategy {
            AggregationStrategy::CollectAll => {
                let values: Vec<_> = results.iter().filter_map(|r| r.result.clone()).collect();
                serde_json::json!(values)
            }
            AggregationStrategy::MergeObjects => {
                let mut merged = serde_json::Map::new();
                for result in results {
                    if let Some(serde_json::Value::Object(obj)) = &result.result {
                        for (k, v) in obj {
                            merged.insert(k.clone(), v.clone());
                        }
                    }
                }
                serde_json::Value::Object(merged)
            }
            AggregationStrategy::FirstSuccess => results
                .iter()
                .find(|r| r.is_success())
                .and_then(|r| r.result.clone())
                .unwrap_or(serde_json::Value::Null),
            AggregationStrategy::HighestConfidence => {
                // Look for confidence field in results
                results
                    .iter()
                    .filter(|r| r.is_success())
                    .filter_map(|r| {
                        r.result.as_ref().and_then(|v| {
                            v.get("confidence")
                                .and_then(|c| c.as_f64())
                                .map(|conf| (conf, v.clone()))
                        })
                    })
                    .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(_, v)| v)
                    .unwrap_or(serde_json::Value::Null)
            }
            AggregationStrategy::Custom => {
                // Custom aggregation would be handled externally
                serde_json::Value::Null
            }
        }
    }

    /// Cancel an active execution
    pub async fn cancel(&self, execution_id: &str) -> bool {
        let mut active = self.active_executions.write().await;
        if let Some(status) = active.get_mut(execution_id) {
            if *status == ExecutionStatus::Running {
                *status = ExecutionStatus::Cancelled;
                return true;
            }
        }
        false
    }

    /// Get execution status
    pub async fn get_status(&self, execution_id: &str) -> Option<ExecutionStatus> {
        let active = self.active_executions.read().await;
        active.get(execution_id).copied()
    }

    /// List active executions
    pub async fn list_active(&self) -> Vec<(String, ExecutionStatus)> {
        let active = self.active_executions.read().await;
        active
            .iter()
            .filter(|(_, s)| **s == ExecutionStatus::Running)
            .map(|(id, s)| (id.clone(), *s))
            .collect()
    }

    /// Clean up completed executions
    pub async fn cleanup(&self) -> usize {
        let mut active = self.active_executions.write().await;
        let before = active.len();
        active.retain(|_, s| *s == ExecutionStatus::Running);
        before - active.len()
    }
}

impl Clone for ParallelExecutor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            semaphore: Arc::clone(&self.semaphore),
            active_executions: Arc::clone(&self.active_executions),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_execution_result() {
        let success = TaskExecutionResult::success(
            "task-1",
            "agent-1",
            serde_json::json!({"result": "ok"}),
            100,
        );
        assert!(success.is_success());

        let failure = TaskExecutionResult::failure("task-2", "something went wrong", 50);
        assert!(!failure.is_success());

        let timeout = TaskExecutionResult::timeout("task-3", 30000);
        assert!(!timeout.is_success());
        assert_eq!(timeout.status, ExecutionStatus::TimedOut);
    }

    #[test]
    fn test_parallel_execution_result() {
        let result = ParallelExecutionResult {
            execution_id: "exec-1".to_string(),
            status: ExecutionStatus::Completed,
            task_results: vec![
                TaskExecutionResult::success("t1", "a1", serde_json::json!({}), 100),
                TaskExecutionResult::failure("t2", "error", 50),
                TaskExecutionResult::success("t3", "a2", serde_json::json!({}), 150),
            ],
            total_duration_ms: 200,
            successful_count: 2,
            failed_count: 1,
            aggregated_result: None,
        };

        assert!((result.success_rate() - 0.666).abs() < 0.01);
        assert_eq!(result.successful_results().len(), 2);
        assert_eq!(result.failed_results().len(), 1);
    }

    #[tokio::test]
    async fn test_parallel_executor() {
        let executor = ParallelExecutor::with_defaults();

        let tasks = vec![
            SpawnTask::new("Task 1"),
            SpawnTask::new("Task 2"),
            SpawnTask::new("Task 3"),
        ];

        // Simple executor that returns the prompt
        let result = executor
            .execute_parallel(tasks, |task| async move {
                Ok(serde_json::json!({ "prompt": task.prompt }))
            })
            .await
            .unwrap();

        assert_eq!(result.task_results.len(), 3);
        assert_eq!(result.successful_count, 3);
        assert_eq!(result.failed_count, 0);
    }

    #[test]
    fn test_aggregation_collect_all() {
        let executor = ParallelExecutor::with_defaults();
        let results = vec![
            TaskExecutionResult::success("t1", "a1", serde_json::json!({"v": 1}), 100),
            TaskExecutionResult::success("t2", "a2", serde_json::json!({"v": 2}), 100),
        ];

        let aggregated = executor.aggregate_results(&results, AggregationStrategy::CollectAll);
        assert!(aggregated.is_array());
        assert_eq!(aggregated.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_aggregation_merge_objects() {
        let executor = ParallelExecutor::with_defaults();
        let results = vec![
            TaskExecutionResult::success("t1", "a1", serde_json::json!({"a": 1}), 100),
            TaskExecutionResult::success("t2", "a2", serde_json::json!({"b": 2}), 100),
        ];

        let aggregated = executor.aggregate_results(&results, AggregationStrategy::MergeObjects);
        assert!(aggregated.is_object());
        assert_eq!(aggregated["a"], 1);
        assert_eq!(aggregated["b"], 2);
    }
}
