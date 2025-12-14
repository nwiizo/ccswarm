//! Benchmark metrics collection

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Metrics collector for benchmarks
pub struct MetricsCollector {
    /// Collected metrics
    metrics: Arc<RwLock<Vec<BenchmarkMetrics>>>,
    /// Agent-specific metrics
    agent_metrics: Arc<RwLock<HashMap<String, AgentMetrics>>>,
}

/// Metrics for a benchmark run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    /// Run ID
    pub run_id: String,
    /// Suite ID
    pub suite_id: String,
    /// Agent ID
    pub agent_id: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Total tasks
    pub total_tasks: usize,
    /// Passed tasks
    pub passed_tasks: usize,
    /// Failed tasks
    pub failed_tasks: usize,
    /// Pass rate (0.0 - 1.0)
    pub pass_rate: f64,
    /// Average task duration in milliseconds
    pub avg_duration_ms: u64,
    /// Total duration in milliseconds
    pub total_duration_ms: u64,
    /// Points earned
    pub points_earned: u32,
    /// Maximum points
    pub max_points: u32,
    /// Score percentage (0.0 - 100.0)
    pub score: f64,
    /// Per-difficulty breakdown
    pub difficulty_breakdown: HashMap<String, DifficultyMetrics>,
    /// Per-type breakdown
    pub type_breakdown: HashMap<String, TypeMetrics>,
}

/// Metrics breakdown by difficulty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyMetrics {
    /// Total tasks at this difficulty
    pub total: usize,
    /// Passed tasks
    pub passed: usize,
    /// Average duration
    pub avg_duration_ms: u64,
}

/// Metrics breakdown by task type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMetrics {
    /// Total tasks of this type
    pub total: usize,
    /// Passed tasks
    pub passed: usize,
    /// Average duration
    pub avg_duration_ms: u64,
}

/// Aggregate metrics for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Agent ID
    pub agent_id: String,
    /// Total runs
    pub total_runs: usize,
    /// Total tasks attempted
    pub total_tasks: usize,
    /// Total tasks passed
    pub total_passed: usize,
    /// Overall pass rate
    pub overall_pass_rate: f64,
    /// Best score achieved
    pub best_score: f64,
    /// Average score
    pub avg_score: f64,
    /// Total time spent (ms)
    pub total_time_ms: u64,
    /// Per-suite performance
    pub suite_performance: HashMap<String, SuitePerformance>,
    /// Improvement trend (positive = improving)
    pub trend: f64,
    /// Last run timestamp
    pub last_run: DateTime<Utc>,
}

/// Performance on a specific suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuitePerformance {
    /// Suite ID
    pub suite_id: String,
    /// Number of attempts
    pub attempts: usize,
    /// First score (baseline)
    pub first_score: f64,
    /// Best score
    pub best_score: f64,
    /// Last score
    pub last_score: f64,
    /// Improvement from first to last
    pub improvement: f64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            agent_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record benchmark metrics
    pub async fn record(&self, metrics: BenchmarkMetrics) {
        // Update metrics history
        {
            let mut history = self.metrics.write().await;
            history.push(metrics.clone());
        }

        // Update agent metrics
        self.update_agent_metrics(&metrics).await;
    }

    /// Update agent metrics
    async fn update_agent_metrics(&self, metrics: &BenchmarkMetrics) {
        let mut agent_map = self.agent_metrics.write().await;

        let agent = agent_map
            .entry(metrics.agent_id.clone())
            .or_insert_with(|| AgentMetrics {
                agent_id: metrics.agent_id.clone(),
                total_runs: 0,
                total_tasks: 0,
                total_passed: 0,
                overall_pass_rate: 0.0,
                best_score: 0.0,
                avg_score: 0.0,
                total_time_ms: 0,
                suite_performance: HashMap::new(),
                trend: 0.0,
                last_run: Utc::now(),
            });

        // Update counts
        agent.total_runs += 1;
        agent.total_tasks += metrics.total_tasks;
        agent.total_passed += metrics.passed_tasks;
        agent.total_time_ms += metrics.total_duration_ms;
        agent.last_run = metrics.timestamp;

        // Update rates
        if agent.total_tasks > 0 {
            agent.overall_pass_rate = agent.total_passed as f64 / agent.total_tasks as f64;
        }

        // Update scores
        if metrics.score > agent.best_score {
            agent.best_score = metrics.score;
        }
        agent.avg_score = (agent.avg_score * (agent.total_runs - 1) as f64 + metrics.score)
            / agent.total_runs as f64;

        // Update suite performance
        let suite_perf = agent
            .suite_performance
            .entry(metrics.suite_id.clone())
            .or_insert_with(|| SuitePerformance {
                suite_id: metrics.suite_id.clone(),
                attempts: 0,
                first_score: 0.0,
                best_score: 0.0,
                last_score: 0.0,
                improvement: 0.0,
            });

        // Record first score on first attempt
        if suite_perf.attempts == 0 {
            suite_perf.first_score = metrics.score;
        }
        suite_perf.attempts += 1;
        if metrics.score > suite_perf.best_score {
            suite_perf.best_score = metrics.score;
        }
        suite_perf.last_score = metrics.score;
        suite_perf.improvement = metrics.score - suite_perf.first_score;
    }

    /// Get metrics for an agent
    pub async fn get_agent_metrics(&self, agent_id: &str) -> Option<AgentMetrics> {
        let agent_map = self.agent_metrics.read().await;
        agent_map.get(agent_id).cloned()
    }

    /// Get all agent metrics
    pub async fn get_all_agent_metrics(&self) -> Vec<AgentMetrics> {
        let agent_map = self.agent_metrics.read().await;
        agent_map.values().cloned().collect()
    }

    /// Get metrics history
    pub async fn get_history(&self, limit: Option<usize>) -> Vec<BenchmarkMetrics> {
        let history = self.metrics.read().await;
        match limit {
            Some(n) => history.iter().rev().take(n).cloned().collect(),
            None => history.clone(),
        }
    }

    /// Get metrics for a specific suite
    pub async fn get_suite_metrics(&self, suite_id: &str) -> Vec<BenchmarkMetrics> {
        let history = self.metrics.read().await;
        history
            .iter()
            .filter(|m| m.suite_id == suite_id)
            .cloned()
            .collect()
    }

    /// Calculate leaderboard for a suite
    pub async fn get_leaderboard(&self, suite_id: &str) -> Vec<LeaderboardEntry> {
        let agent_map = self.agent_metrics.read().await;
        let mut entries: Vec<LeaderboardEntry> = agent_map
            .values()
            .filter_map(|agent| {
                agent
                    .suite_performance
                    .get(suite_id)
                    .map(|perf| LeaderboardEntry {
                        agent_id: agent.agent_id.clone(),
                        best_score: perf.best_score,
                        attempts: perf.attempts,
                        improvement: perf.improvement,
                    })
            })
            .collect();

        entries.sort_by(|a, b| {
            b.best_score
                .partial_cmp(&a.best_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        entries
    }

    /// Get metrics count
    pub async fn metrics_count(&self) -> usize {
        let history = self.metrics.read().await;
        history.len()
    }

    /// Clear all metrics
    pub async fn clear(&self) {
        let mut history = self.metrics.write().await;
        history.clear();
        let mut agent_map = self.agent_metrics.write().await;
        agent_map.clear();
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    /// Agent ID
    pub agent_id: String,
    /// Best score
    pub best_score: f64,
    /// Number of attempts
    pub attempts: usize,
    /// Improvement from first attempt
    pub improvement: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metrics(agent_id: &str, suite_id: &str, score: f64) -> BenchmarkMetrics {
        BenchmarkMetrics {
            run_id: uuid::Uuid::new_v4().to_string(),
            suite_id: suite_id.to_string(),
            agent_id: agent_id.to_string(),
            timestamp: Utc::now(),
            total_tasks: 10,
            passed_tasks: (score / 10.0) as usize,
            failed_tasks: 10 - (score / 10.0) as usize,
            pass_rate: score / 100.0,
            avg_duration_ms: 1000,
            total_duration_ms: 10000,
            points_earned: (score / 10.0) as u32 * 10,
            max_points: 100,
            score,
            difficulty_breakdown: HashMap::new(),
            type_breakdown: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_collector_creation() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.metrics_count().await, 0);
    }

    #[tokio::test]
    async fn test_record_metrics() {
        let collector = MetricsCollector::new();
        let metrics = create_test_metrics("agent-1", "suite-1", 85.0);

        collector.record(metrics).await;
        assert_eq!(collector.metrics_count().await, 1);
    }

    #[tokio::test]
    async fn test_agent_metrics_update() {
        let collector = MetricsCollector::new();

        collector
            .record(create_test_metrics("agent-1", "suite-1", 70.0))
            .await;
        collector
            .record(create_test_metrics("agent-1", "suite-1", 80.0))
            .await;

        let agent = collector.get_agent_metrics("agent-1").await.unwrap();
        assert_eq!(agent.total_runs, 2);
        assert_eq!(agent.best_score, 80.0);
    }

    #[tokio::test]
    async fn test_suite_performance_tracking() {
        let collector = MetricsCollector::new();

        collector
            .record(create_test_metrics("agent-1", "suite-1", 60.0))
            .await;
        collector
            .record(create_test_metrics("agent-1", "suite-1", 75.0))
            .await;
        collector
            .record(create_test_metrics("agent-1", "suite-1", 80.0))
            .await;

        let agent = collector.get_agent_metrics("agent-1").await.unwrap();
        let perf = agent.suite_performance.get("suite-1").unwrap();

        assert_eq!(perf.attempts, 3);
        assert_eq!(perf.best_score, 80.0);
        assert_eq!(perf.last_score, 80.0);
        assert_eq!(perf.improvement, 20.0); // 80 - 60
    }

    #[tokio::test]
    async fn test_get_history() {
        let collector = MetricsCollector::new();

        for i in 0..5 {
            collector
                .record(create_test_metrics("agent", "suite", (i * 10) as f64))
                .await;
        }

        let all = collector.get_history(None).await;
        assert_eq!(all.len(), 5);

        let limited = collector.get_history(Some(3)).await;
        assert_eq!(limited.len(), 3);
    }

    #[tokio::test]
    async fn test_get_suite_metrics() {
        let collector = MetricsCollector::new();

        collector
            .record(create_test_metrics("agent-1", "suite-1", 70.0))
            .await;
        collector
            .record(create_test_metrics("agent-1", "suite-2", 80.0))
            .await;
        collector
            .record(create_test_metrics("agent-2", "suite-1", 75.0))
            .await;

        let suite1 = collector.get_suite_metrics("suite-1").await;
        assert_eq!(suite1.len(), 2);
    }

    #[tokio::test]
    async fn test_leaderboard() {
        let collector = MetricsCollector::new();

        collector
            .record(create_test_metrics("agent-1", "suite-1", 80.0))
            .await;
        collector
            .record(create_test_metrics("agent-2", "suite-1", 90.0))
            .await;
        collector
            .record(create_test_metrics("agent-3", "suite-1", 75.0))
            .await;

        let leaderboard = collector.get_leaderboard("suite-1").await;

        assert_eq!(leaderboard.len(), 3);
        assert_eq!(leaderboard[0].agent_id, "agent-2"); // Highest score
        assert_eq!(leaderboard[1].agent_id, "agent-1");
        assert_eq!(leaderboard[2].agent_id, "agent-3");
    }

    #[tokio::test]
    async fn test_get_all_agent_metrics() {
        let collector = MetricsCollector::new();

        collector
            .record(create_test_metrics("agent-1", "suite-1", 80.0))
            .await;
        collector
            .record(create_test_metrics("agent-2", "suite-1", 70.0))
            .await;

        let all = collector.get_all_agent_metrics().await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn test_clear_metrics() {
        let collector = MetricsCollector::new();

        collector
            .record(create_test_metrics("agent-1", "suite-1", 80.0))
            .await;
        assert_eq!(collector.metrics_count().await, 1);

        collector.clear().await;
        assert_eq!(collector.metrics_count().await, 0);
    }

    #[test]
    fn test_difficulty_metrics() {
        let dm = DifficultyMetrics {
            total: 5,
            passed: 4,
            avg_duration_ms: 2000,
        };

        assert_eq!(dm.total, 5);
        assert_eq!(dm.passed, 4);
    }

    #[test]
    fn test_leaderboard_entry() {
        let entry = LeaderboardEntry {
            agent_id: "agent-1".to_string(),
            best_score: 95.0,
            attempts: 5,
            improvement: 15.0,
        };

        assert_eq!(entry.best_score, 95.0);
    }
}
