//! Benchmark Integration Module
//!
//! Provides a framework for evaluating AI agent performance using
//! standardized benchmarks (SWE-Bench, τ-Bench style evaluations).

mod metrics;
mod runner;
mod task;

pub use metrics::{AgentMetrics, BenchmarkMetrics, MetricsCollector};
pub use runner::{BenchmarkResult, BenchmarkRunner, RunnerConfig};
pub use task::{BenchmarkTask, TaskDifficulty, TaskType};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Benchmark suite containing multiple tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    /// Suite ID
    pub id: String,
    /// Suite name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Version
    pub version: String,
    /// Tasks in the suite
    pub tasks: Vec<BenchmarkTask>,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Created at
    pub created_at: DateTime<Utc>,
}

impl BenchmarkSuite {
    /// Create a new benchmark suite
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            version: "1.0.0".to_string(),
            tasks: Vec::new(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    /// Add a task to the suite
    pub fn with_task(mut self, task: BenchmarkTask) -> Self {
        self.tasks.push(task);
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Get task count
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    /// Get tasks by difficulty
    pub fn tasks_by_difficulty(&self, difficulty: TaskDifficulty) -> Vec<&BenchmarkTask> {
        self.tasks
            .iter()
            .filter(|t| t.difficulty == difficulty)
            .collect()
    }

    /// Get tasks by type
    pub fn tasks_by_type(&self, task_type: TaskType) -> Vec<&BenchmarkTask> {
        self.tasks
            .iter()
            .filter(|t| t.task_type == task_type)
            .collect()
    }

    /// Get total expected points
    pub fn total_points(&self) -> u32 {
        self.tasks.iter().map(|t| t.points).sum()
    }
}

/// Registry for managing benchmark suites
pub struct BenchmarkRegistry {
    /// Registered suites
    suites: Arc<RwLock<HashMap<String, BenchmarkSuite>>>,
    /// Completed runs
    runs: Arc<RwLock<Vec<BenchmarkRun>>>,
}

/// A completed benchmark run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkRun {
    /// Run ID
    pub id: String,
    /// Suite ID
    pub suite_id: String,
    /// Agent ID that was benchmarked
    pub agent_id: String,
    /// Started at
    pub started_at: DateTime<Utc>,
    /// Completed at
    pub completed_at: Option<DateTime<Utc>>,
    /// Results per task
    pub results: Vec<BenchmarkResult>,
    /// Overall score
    pub score: f64,
    /// Pass rate
    pub pass_rate: f64,
    /// Total duration in milliseconds
    pub duration_ms: u64,
}

impl BenchmarkRun {
    /// Calculate summary statistics
    pub fn summary(&self) -> BenchmarkSummary {
        let total = self.results.len();
        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = total - passed;

        let total_duration: u64 = self.results.iter().map(|r| r.duration_ms).sum();
        let avg_duration = if total > 0 {
            total_duration / total as u64
        } else {
            0
        };

        let earned_points: u32 = self
            .results
            .iter()
            .filter(|r| r.passed)
            .map(|r| r.points_earned)
            .sum();

        let max_points: u32 = self.results.iter().map(|r| r.max_points).sum();

        BenchmarkSummary {
            total_tasks: total,
            passed,
            failed,
            pass_rate: self.pass_rate,
            score: self.score,
            earned_points,
            max_points,
            avg_duration_ms: avg_duration,
            total_duration_ms: total_duration,
        }
    }
}

/// Summary of a benchmark run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    /// Total number of tasks
    pub total_tasks: usize,
    /// Number of passed tasks
    pub passed: usize,
    /// Number of failed tasks
    pub failed: usize,
    /// Pass rate (0.0 - 1.0)
    pub pass_rate: f64,
    /// Overall score (0.0 - 100.0)
    pub score: f64,
    /// Points earned
    pub earned_points: u32,
    /// Maximum possible points
    pub max_points: u32,
    /// Average duration per task
    pub avg_duration_ms: u64,
    /// Total duration
    pub total_duration_ms: u64,
}

impl BenchmarkRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            suites: Arc::new(RwLock::new(HashMap::new())),
            runs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a benchmark suite
    pub async fn register(&self, suite: BenchmarkSuite) {
        let mut suites = self.suites.write().await;
        suites.insert(suite.id.clone(), suite);
    }

    /// Get a suite by ID
    pub async fn get_suite(&self, id: &str) -> Option<BenchmarkSuite> {
        let suites = self.suites.read().await;
        suites.get(id).cloned()
    }

    /// List all suite IDs
    pub async fn list_suites(&self) -> Vec<String> {
        let suites = self.suites.read().await;
        suites.keys().cloned().collect()
    }

    /// Record a completed run
    pub async fn record_run(&self, run: BenchmarkRun) {
        let mut runs = self.runs.write().await;
        runs.push(run);
    }

    /// Get runs for a suite
    pub async fn get_runs_for_suite(&self, suite_id: &str) -> Vec<BenchmarkRun> {
        let runs = self.runs.read().await;
        runs.iter()
            .filter(|r| r.suite_id == suite_id)
            .cloned()
            .collect()
    }

    /// Get runs for an agent
    pub async fn get_runs_for_agent(&self, agent_id: &str) -> Vec<BenchmarkRun> {
        let runs = self.runs.read().await;
        runs.iter()
            .filter(|r| r.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Get best run for an agent on a suite
    pub async fn get_best_run(&self, suite_id: &str, agent_id: &str) -> Option<BenchmarkRun> {
        let runs = self.runs.read().await;
        runs.iter()
            .filter(|r| r.suite_id == suite_id && r.agent_id == agent_id)
            .max_by(|a, b| {
                a.score
                    .partial_cmp(&b.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    /// Get suite count
    pub async fn suite_count(&self) -> usize {
        let suites = self.suites.read().await;
        suites.len()
    }

    /// Get total run count
    pub async fn run_count(&self) -> usize {
        let runs = self.runs.read().await;
        runs.len()
    }
}

impl Default for BenchmarkRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined benchmark suites
pub struct PredefinedSuites;

impl PredefinedSuites {
    /// Create a basic coding tasks suite
    pub fn basic_coding() -> BenchmarkSuite {
        BenchmarkSuite::new("basic-coding", "Basic Coding Tasks")
            .with_description("Simple programming tasks for initial evaluation")
            .with_task(
                BenchmarkTask::new("fizzbuzz", "Implement FizzBuzz")
                    .with_type(TaskType::Implementation)
                    .with_difficulty(TaskDifficulty::Easy)
                    .with_points(10)
                    .with_expected_output(
                        "1, 2, Fizz, 4, Buzz, Fizz, 7, 8, Fizz, Buzz, 11, Fizz, 13, 14, FizzBuzz",
                    ),
            )
            .with_task(
                BenchmarkTask::new("palindrome", "Check if string is palindrome")
                    .with_type(TaskType::Implementation)
                    .with_difficulty(TaskDifficulty::Easy)
                    .with_points(10),
            )
            .with_task(
                BenchmarkTask::new("fibonacci", "Generate Fibonacci sequence")
                    .with_type(TaskType::Implementation)
                    .with_difficulty(TaskDifficulty::Easy)
                    .with_points(10),
            )
    }

    /// Create a bug fix suite
    pub fn bug_fixes() -> BenchmarkSuite {
        BenchmarkSuite::new("bug-fixes", "Bug Fix Tasks")
            .with_description("Tasks requiring identifying and fixing bugs")
            .with_task(
                BenchmarkTask::new("off-by-one", "Fix off-by-one error in loop")
                    .with_type(TaskType::BugFix)
                    .with_difficulty(TaskDifficulty::Easy)
                    .with_points(15),
            )
            .with_task(
                BenchmarkTask::new("null-check", "Add missing null checks")
                    .with_type(TaskType::BugFix)
                    .with_difficulty(TaskDifficulty::Medium)
                    .with_points(20),
            )
            .with_task(
                BenchmarkTask::new("race-condition", "Fix race condition")
                    .with_type(TaskType::BugFix)
                    .with_difficulty(TaskDifficulty::Hard)
                    .with_points(30),
            )
    }

    /// Create a refactoring suite
    pub fn refactoring() -> BenchmarkSuite {
        BenchmarkSuite::new("refactoring", "Refactoring Tasks")
            .with_description("Code improvement and refactoring tasks")
            .with_task(
                BenchmarkTask::new("extract-function", "Extract repeated code to function")
                    .with_type(TaskType::Refactoring)
                    .with_difficulty(TaskDifficulty::Medium)
                    .with_points(20),
            )
            .with_task(
                BenchmarkTask::new("apply-pattern", "Apply design pattern")
                    .with_type(TaskType::Refactoring)
                    .with_difficulty(TaskDifficulty::Hard)
                    .with_points(30),
            )
    }

    /// Get all predefined suites
    pub fn all() -> Vec<BenchmarkSuite> {
        vec![Self::basic_coding(), Self::bug_fixes(), Self::refactoring()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_suite_creation() {
        let suite = BenchmarkSuite::new("test-suite", "Test Suite");
        assert_eq!(suite.id, "test-suite");
        assert_eq!(suite.name, "Test Suite");
        assert_eq!(suite.task_count(), 0);
    }

    #[test]
    fn test_suite_with_tasks() {
        let suite = BenchmarkSuite::new("tasks", "Tasks Suite")
            .with_task(BenchmarkTask::new("task-1", "Task 1"))
            .with_task(BenchmarkTask::new("task-2", "Task 2"));

        assert_eq!(suite.task_count(), 2);
    }

    #[test]
    fn test_suite_total_points() {
        let suite = BenchmarkSuite::new("points", "Points Test")
            .with_task(BenchmarkTask::new("t1", "Task 1").with_points(10))
            .with_task(BenchmarkTask::new("t2", "Task 2").with_points(20));

        assert_eq!(suite.total_points(), 30);
    }

    #[test]
    fn test_tasks_by_difficulty() {
        let suite = BenchmarkSuite::new("diff", "Difficulty Test")
            .with_task(BenchmarkTask::new("e1", "Easy 1").with_difficulty(TaskDifficulty::Easy))
            .with_task(BenchmarkTask::new("h1", "Hard 1").with_difficulty(TaskDifficulty::Hard))
            .with_task(BenchmarkTask::new("e2", "Easy 2").with_difficulty(TaskDifficulty::Easy));

        let easy = suite.tasks_by_difficulty(TaskDifficulty::Easy);
        assert_eq!(easy.len(), 2);

        let hard = suite.tasks_by_difficulty(TaskDifficulty::Hard);
        assert_eq!(hard.len(), 1);
    }

    #[test]
    fn test_tasks_by_type() {
        let suite = BenchmarkSuite::new("type", "Type Test")
            .with_task(BenchmarkTask::new("impl", "Impl").with_type(TaskType::Implementation))
            .with_task(BenchmarkTask::new("fix", "Fix").with_type(TaskType::BugFix));

        let impl_tasks = suite.tasks_by_type(TaskType::Implementation);
        assert_eq!(impl_tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let registry = BenchmarkRegistry::new();
        assert_eq!(registry.suite_count().await, 0);
    }

    #[tokio::test]
    async fn test_registry_register_suite() {
        let registry = BenchmarkRegistry::new();
        let suite = BenchmarkSuite::new("test", "Test");

        registry.register(suite).await;
        assert_eq!(registry.suite_count().await, 1);
    }

    #[tokio::test]
    async fn test_registry_get_suite() {
        let registry = BenchmarkRegistry::new();
        let suite = BenchmarkSuite::new("my-suite", "My Suite");

        registry.register(suite).await;

        let retrieved = registry.get_suite("my-suite").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "My Suite");
    }

    #[tokio::test]
    async fn test_registry_record_run() {
        let registry = BenchmarkRegistry::new();

        let run = BenchmarkRun {
            id: "run-1".to_string(),
            suite_id: "suite-1".to_string(),
            agent_id: "agent-1".to_string(),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            results: vec![],
            score: 85.0,
            pass_rate: 0.9,
            duration_ms: 5000,
        };

        registry.record_run(run).await;
        assert_eq!(registry.run_count().await, 1);
    }

    #[tokio::test]
    async fn test_registry_get_runs_for_agent() {
        let registry = BenchmarkRegistry::new();

        let run1 = BenchmarkRun {
            id: "run-1".to_string(),
            suite_id: "suite-1".to_string(),
            agent_id: "agent-1".to_string(),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            results: vec![],
            score: 80.0,
            pass_rate: 0.8,
            duration_ms: 4000,
        };

        let run2 = BenchmarkRun {
            id: "run-2".to_string(),
            suite_id: "suite-2".to_string(),
            agent_id: "agent-1".to_string(),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            results: vec![],
            score: 90.0,
            pass_rate: 0.95,
            duration_ms: 3000,
        };

        registry.record_run(run1).await;
        registry.record_run(run2).await;

        let runs = registry.get_runs_for_agent("agent-1").await;
        assert_eq!(runs.len(), 2);
    }

    #[test]
    fn test_benchmark_run_summary() {
        let run = BenchmarkRun {
            id: "run".to_string(),
            suite_id: "suite".to_string(),
            agent_id: "agent".to_string(),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            results: vec![
                BenchmarkResult {
                    task_id: "t1".to_string(),
                    passed: true,
                    points_earned: 10,
                    max_points: 10,
                    duration_ms: 1000,
                    output: None,
                    error: None,
                },
                BenchmarkResult {
                    task_id: "t2".to_string(),
                    passed: false,
                    points_earned: 0,
                    max_points: 20,
                    duration_ms: 2000,
                    output: None,
                    error: Some("Failed".to_string()),
                },
            ],
            score: 33.3,
            pass_rate: 0.5,
            duration_ms: 3000,
        };

        let summary = run.summary();
        assert_eq!(summary.total_tasks, 2);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.earned_points, 10);
        assert_eq!(summary.max_points, 30);
    }

    #[test]
    fn test_predefined_suites() {
        let suites = PredefinedSuites::all();
        assert_eq!(suites.len(), 3);

        let basic = &suites[0];
        assert_eq!(basic.id, "basic-coding");
        assert!(basic.task_count() > 0);
    }

    #[test]
    fn test_predefined_basic_coding() {
        let suite = PredefinedSuites::basic_coding();
        assert_eq!(suite.id, "basic-coding");
        assert!(suite.task_count() >= 3);
    }

    #[test]
    fn test_predefined_bug_fixes() {
        let suite = PredefinedSuites::bug_fixes();
        assert_eq!(suite.id, "bug-fixes");
        assert!(!suite.tasks_by_type(TaskType::BugFix).is_empty());
    }

    #[test]
    fn test_predefined_refactoring() {
        let suite = PredefinedSuites::refactoring();
        assert_eq!(suite.id, "refactoring");
        assert!(!suite.tasks_by_type(TaskType::Refactoring).is_empty());
    }
}
