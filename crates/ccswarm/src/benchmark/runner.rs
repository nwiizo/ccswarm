//! Benchmark runner

use super::metrics::{BenchmarkMetrics, MetricsCollector};
use super::task::{BenchmarkTask, TaskContext, TaskDifficulty};
use super::{BenchmarkRun, BenchmarkSuite};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for the benchmark runner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerConfig {
    /// Maximum parallel tasks
    pub max_parallel: usize,
    /// Default timeout per task in seconds
    pub default_timeout_secs: u64,
    /// Whether to continue on failure
    pub continue_on_failure: bool,
    /// Whether to shuffle task order
    pub shuffle_tasks: bool,
    /// Retry failed tasks
    pub retry_failed: bool,
    /// Maximum retries
    pub max_retries: u32,
    /// Collect detailed metrics
    pub detailed_metrics: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            max_parallel: 1,
            default_timeout_secs: 300,
            continue_on_failure: true,
            shuffle_tasks: false,
            retry_failed: false,
            max_retries: 2,
            detailed_metrics: true,
        }
    }
}

/// Result of a single task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Task ID
    pub task_id: String,
    /// Whether passed
    pub passed: bool,
    /// Points earned
    pub points_earned: u32,
    /// Maximum points
    pub max_points: u32,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Output produced
    pub output: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Handler for executing tasks
#[async_trait::async_trait]
pub trait TaskHandler: Send + Sync {
    /// Execute a task and return the result
    async fn execute(&self, task: &BenchmarkTask, context: &TaskContext) -> BenchmarkResult;
}

/// Default task handler (for testing)
pub struct DefaultTaskHandler;

#[async_trait::async_trait]
impl TaskHandler for DefaultTaskHandler {
    async fn execute(&self, task: &BenchmarkTask, _context: &TaskContext) -> BenchmarkResult {
        // Simulate task execution
        let duration_ms = 100 + (task.difficulty.expected_minutes() as u64 * 10);

        // Mock success based on difficulty
        let passed = match task.difficulty {
            TaskDifficulty::Easy => true,
            TaskDifficulty::Medium => true,
            TaskDifficulty::Hard => rand_success(0.7),
            TaskDifficulty::Expert => rand_success(0.5),
        };

        BenchmarkResult {
            task_id: task.id.clone(),
            passed,
            points_earned: if passed { task.points } else { 0 },
            max_points: task.points,
            duration_ms,
            output: if passed {
                Some("Task completed successfully".to_string())
            } else {
                None
            },
            error: if !passed {
                Some("Task failed".to_string())
            } else {
                None
            },
        }
    }
}

fn rand_success(probability: f64) -> bool {
    // Simple deterministic "random" for tests
    probability >= 0.5
}

/// Benchmark runner
pub struct BenchmarkRunner {
    /// Configuration
    config: RunnerConfig,
    /// Task handler
    handler: Arc<dyn TaskHandler>,
    /// Metrics collector
    metrics: Arc<MetricsCollector>,
    /// Active runs
    active_runs: Arc<RwLock<HashMap<String, BenchmarkRun>>>,
}

impl BenchmarkRunner {
    /// Create a new runner with default handler
    pub fn new(config: RunnerConfig) -> Self {
        Self::with_handler(config, Arc::new(DefaultTaskHandler))
    }

    /// Create with custom handler
    pub fn with_handler(config: RunnerConfig, handler: Arc<dyn TaskHandler>) -> Self {
        Self {
            config,
            handler,
            metrics: Arc::new(MetricsCollector::new()),
            active_runs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get metrics collector
    pub fn metrics(&self) -> &Arc<MetricsCollector> {
        &self.metrics
    }

    /// Run a benchmark suite for an agent
    pub async fn run(&self, suite: &BenchmarkSuite, agent_id: &str) -> BenchmarkRun {
        let run_id = uuid::Uuid::new_v4().to_string();
        let started_at = Utc::now();

        // Initialize run
        let mut run = BenchmarkRun {
            id: run_id.clone(),
            suite_id: suite.id.clone(),
            agent_id: agent_id.to_string(),
            started_at,
            completed_at: None,
            results: Vec::new(),
            score: 0.0,
            pass_rate: 0.0,
            duration_ms: 0,
        };

        // Track active run
        {
            let mut active = self.active_runs.write().await;
            active.insert(run_id.clone(), run.clone());
        }

        // Execute tasks
        let context = TaskContext::default();
        let mut results = Vec::new();
        let mut total_duration: u64 = 0;

        for task in &suite.tasks {
            let result = self.execute_task(task, &context).await;
            total_duration += result.duration_ms;
            results.push(result);
        }

        // Calculate statistics
        let total_tasks = results.len();
        let passed_tasks = results.iter().filter(|r| r.passed).count();
        let total_points: u32 = results.iter().map(|r| r.max_points).sum();
        let earned_points: u32 = results.iter().map(|r| r.points_earned).sum();

        run.results = results;
        run.duration_ms = total_duration;
        run.pass_rate = if total_tasks > 0 {
            passed_tasks as f64 / total_tasks as f64
        } else {
            0.0
        };
        run.score = if total_points > 0 {
            (earned_points as f64 / total_points as f64) * 100.0
        } else {
            0.0
        };
        run.completed_at = Some(Utc::now());

        // Record metrics
        if self.config.detailed_metrics {
            self.record_metrics(&run, suite).await;
        }

        // Remove from active runs
        {
            let mut active = self.active_runs.write().await;
            active.remove(&run_id);
        }

        run
    }

    /// Execute a single task
    async fn execute_task(&self, task: &BenchmarkTask, context: &TaskContext) -> BenchmarkResult {
        let mut attempts = 0;
        let max_attempts = if self.config.retry_failed {
            self.config.max_retries + 1
        } else {
            1
        };

        loop {
            let result = self.handler.execute(task, context).await;

            if result.passed || attempts >= max_attempts - 1 {
                return result;
            }

            attempts += 1;
        }
    }

    /// Record metrics for a run
    async fn record_metrics(&self, run: &BenchmarkRun, suite: &BenchmarkSuite) {
        let mut difficulty_breakdown: HashMap<String, super::metrics::DifficultyMetrics> =
            HashMap::new();
        let mut type_breakdown: HashMap<String, super::metrics::TypeMetrics> = HashMap::new();

        for (result, task) in run.results.iter().zip(suite.tasks.iter()) {
            // Difficulty breakdown
            let diff_key = format!("{:?}", task.difficulty);
            let diff_entry =
                difficulty_breakdown
                    .entry(diff_key)
                    .or_insert(super::metrics::DifficultyMetrics {
                        total: 0,
                        passed: 0,
                        avg_duration_ms: 0,
                    });
            diff_entry.total += 1;
            if result.passed {
                diff_entry.passed += 1;
            }
            diff_entry.avg_duration_ms =
                (diff_entry.avg_duration_ms * (diff_entry.total - 1) as u64 + result.duration_ms)
                    / diff_entry.total as u64;

            // Type breakdown
            let type_key = format!("{:?}", task.task_type);
            let type_entry =
                type_breakdown
                    .entry(type_key)
                    .or_insert(super::metrics::TypeMetrics {
                        total: 0,
                        passed: 0,
                        avg_duration_ms: 0,
                    });
            type_entry.total += 1;
            if result.passed {
                type_entry.passed += 1;
            }
            type_entry.avg_duration_ms =
                (type_entry.avg_duration_ms * (type_entry.total - 1) as u64 + result.duration_ms)
                    / type_entry.total as u64;
        }

        let metrics = BenchmarkMetrics {
            run_id: run.id.clone(),
            suite_id: run.suite_id.clone(),
            agent_id: run.agent_id.clone(),
            timestamp: run.started_at,
            total_tasks: run.results.len(),
            passed_tasks: run.results.iter().filter(|r| r.passed).count(),
            failed_tasks: run.results.iter().filter(|r| !r.passed).count(),
            pass_rate: run.pass_rate,
            avg_duration_ms: if run.results.is_empty() {
                0
            } else {
                run.duration_ms / run.results.len() as u64
            },
            total_duration_ms: run.duration_ms,
            points_earned: run.results.iter().map(|r| r.points_earned).sum(),
            max_points: run.results.iter().map(|r| r.max_points).sum(),
            score: run.score,
            difficulty_breakdown,
            type_breakdown,
        };

        self.metrics.record(metrics).await;
    }

    /// Get active run count
    pub async fn active_run_count(&self) -> usize {
        let active = self.active_runs.read().await;
        active.len()
    }

    /// Check if a run is active
    pub async fn is_run_active(&self, run_id: &str) -> bool {
        let active = self.active_runs.read().await;
        active.contains_key(run_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_suite() -> BenchmarkSuite {
        BenchmarkSuite::new("test-suite", "Test Suite")
            .with_task(
                BenchmarkTask::new("task-1", "Easy Task")
                    .with_difficulty(TaskDifficulty::Easy)
                    .with_points(10),
            )
            .with_task(
                BenchmarkTask::new("task-2", "Medium Task")
                    .with_difficulty(TaskDifficulty::Medium)
                    .with_points(20),
            )
    }

    #[test]
    fn test_runner_config_default() {
        let config = RunnerConfig::default();
        assert_eq!(config.max_parallel, 1);
        assert!(config.continue_on_failure);
    }

    #[test]
    fn test_benchmark_result_creation() {
        let result = BenchmarkResult {
            task_id: "task-1".to_string(),
            passed: true,
            points_earned: 10,
            max_points: 10,
            duration_ms: 100,
            output: Some("Success".to_string()),
            error: None,
        };

        assert!(result.passed);
        assert_eq!(result.points_earned, 10);
    }

    #[tokio::test]
    async fn test_runner_creation() {
        let runner = BenchmarkRunner::new(RunnerConfig::default());
        assert_eq!(runner.active_run_count().await, 0);
    }

    #[tokio::test]
    async fn test_run_suite() {
        let runner = BenchmarkRunner::new(RunnerConfig::default());
        let suite = create_test_suite();

        let run = runner.run(&suite, "test-agent").await;

        assert_eq!(run.suite_id, "test-suite");
        assert_eq!(run.agent_id, "test-agent");
        assert_eq!(run.results.len(), 2);
        assert!(run.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_run_metrics_recorded() {
        let config = RunnerConfig {
            detailed_metrics: true,
            ..Default::default()
        };
        let runner = BenchmarkRunner::new(config);
        let suite = create_test_suite();

        runner.run(&suite, "agent-1").await;

        let metrics_count = runner.metrics().metrics_count().await;
        assert_eq!(metrics_count, 1);
    }

    #[tokio::test]
    async fn test_run_score_calculation() {
        let runner = BenchmarkRunner::new(RunnerConfig::default());
        let suite = create_test_suite();

        let run = runner.run(&suite, "agent-1").await;

        // All easy/medium tasks pass with DefaultTaskHandler
        assert!(run.pass_rate > 0.0);
        assert!(run.score > 0.0);
    }

    #[tokio::test]
    async fn test_run_duration() {
        let runner = BenchmarkRunner::new(RunnerConfig::default());
        let suite = create_test_suite();

        let run = runner.run(&suite, "agent-1").await;

        assert!(run.duration_ms > 0);
    }

    #[tokio::test]
    async fn test_active_run_tracking() {
        // This is hard to test without async control, but we can verify the mechanism works
        let runner = BenchmarkRunner::new(RunnerConfig::default());
        assert!(!runner.is_run_active("nonexistent").await);
    }

    #[tokio::test]
    async fn test_run_with_retry() {
        let config = RunnerConfig {
            retry_failed: true,
            max_retries: 2,
            ..Default::default()
        };
        let runner = BenchmarkRunner::new(config);
        let suite = create_test_suite();

        let run = runner.run(&suite, "agent-1").await;
        assert_eq!(run.results.len(), 2);
    }

    #[tokio::test]
    async fn test_multiple_runs() {
        let runner = BenchmarkRunner::new(RunnerConfig::default());
        let suite = create_test_suite();

        let run1 = runner.run(&suite, "agent-1").await;
        let run2 = runner.run(&suite, "agent-2").await;

        assert_ne!(run1.id, run2.id);
        assert_ne!(run1.agent_id, run2.agent_id);
    }

    #[tokio::test]
    async fn test_custom_handler() {
        struct CustomHandler;

        #[async_trait::async_trait]
        impl TaskHandler for CustomHandler {
            async fn execute(
                &self,
                task: &BenchmarkTask,
                _context: &TaskContext,
            ) -> BenchmarkResult {
                BenchmarkResult {
                    task_id: task.id.clone(),
                    passed: true,
                    points_earned: 100,
                    max_points: 100,
                    duration_ms: 50,
                    output: Some("Custom output".to_string()),
                    error: None,
                }
            }
        }

        let runner =
            BenchmarkRunner::with_handler(RunnerConfig::default(), Arc::new(CustomHandler));
        let suite = create_test_suite();

        let run = runner.run(&suite, "agent-1").await;

        // All tasks should earn 100 points with custom handler
        for result in &run.results {
            assert_eq!(result.points_earned, 100);
        }
    }

    #[test]
    fn test_result_with_error() {
        let result = BenchmarkResult {
            task_id: "failed-task".to_string(),
            passed: false,
            points_earned: 0,
            max_points: 20,
            duration_ms: 5000,
            output: None,
            error: Some("Timeout exceeded".to_string()),
        };

        assert!(!result.passed);
        assert!(result.error.is_some());
    }
}
