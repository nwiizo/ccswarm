//! Arpeggio batch processing for Piece/Movement workflows.
//!
//! Enables running a piece across multiple inputs in batch, either sequentially
//! or in parallel. Inspired by takt's arpeggio (batch execution) feature.
//!
//! An "arpeggio" takes a single piece definition and applies it to a sequence
//! of tasks, collecting results.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Configuration for an arpeggio (batch) execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArpeggioConfig {
    /// Piece to execute for each item
    pub piece_name: String,
    /// Maximum concurrent executions (1 = sequential)
    #[serde(default = "default_concurrency")]
    pub max_concurrency: usize,
    /// Whether to stop on first failure
    #[serde(default)]
    pub fail_fast: bool,
    /// Maximum total execution time in seconds
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    /// Variables shared across all items
    #[serde(default)]
    pub shared_variables: HashMap<String, String>,
}

fn default_concurrency() -> usize {
    1
}

impl Default for ArpeggioConfig {
    fn default() -> Self {
        Self {
            piece_name: "default".to_string(),
            max_concurrency: 1,
            fail_fast: false,
            timeout_secs: None,
            shared_variables: HashMap::new(),
        }
    }
}

/// A single item in an arpeggio batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArpeggioItem {
    /// Unique identifier for this item
    pub id: String,
    /// Task text for this item
    pub task_text: String,
    /// Item-specific variables (merged with shared variables)
    #[serde(default)]
    pub variables: HashMap<String, String>,
}

/// Result of a single item execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArpeggioItemResult {
    /// Item ID
    pub id: String,
    /// Whether it succeeded
    pub success: bool,
    /// Output summary
    pub output: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Error message if failed
    pub error: Option<String>,
}

/// Aggregate result of an arpeggio execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArpeggioResult {
    /// Piece that was executed
    pub piece_name: String,
    /// Individual item results
    pub items: Vec<ArpeggioItemResult>,
    /// Total items processed
    pub total: usize,
    /// Successful items
    pub succeeded: usize,
    /// Failed items
    pub failed: usize,
    /// Skipped items (due to fail_fast)
    pub skipped: usize,
    /// Total execution duration in milliseconds
    pub total_duration_ms: u64,
    /// Started at
    pub started_at: DateTime<Utc>,
    /// Completed at
    pub completed_at: DateTime<Utc>,
}

impl ArpeggioResult {
    /// Whether all items succeeded
    pub fn all_succeeded(&self) -> bool {
        self.failed == 0 && self.skipped == 0
    }

    /// Success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.succeeded as f64 / self.total as f64) * 100.0
    }
}

/// Arpeggio executor for batch processing
pub struct ArpeggioExecutor {
    config: ArpeggioConfig,
}

impl ArpeggioExecutor {
    pub fn new(config: ArpeggioConfig) -> Self {
        Self { config }
    }

    /// Execute an arpeggio over a list of items.
    ///
    /// In the current implementation, executes sequentially.
    /// Future versions will support true parallel execution via tokio tasks.
    pub async fn execute(&self, items: Vec<ArpeggioItem>) -> Result<ArpeggioResult> {
        let started_at = Utc::now();
        let total = items.len();
        let start_instant = std::time::Instant::now();

        info!(
            "Starting arpeggio: piece={}, items={}, concurrency={}",
            self.config.piece_name, total, self.config.max_concurrency
        );

        let mut results = Vec::new();
        let mut succeeded = 0;
        let mut failed = 0;
        let mut skipped = 0;

        for item in &items {
            // Check fail_fast
            if self.config.fail_fast && failed > 0 {
                skipped += 1;
                debug!("Skipping item '{}' due to fail_fast", item.id);
                continue;
            }

            // Check timeout
            if let Some(timeout_secs) = self.config.timeout_secs
                && start_instant.elapsed().as_secs() >= timeout_secs
            {
                warn!("Arpeggio timeout reached after {} seconds", timeout_secs);
                skipped += total - results.len() - skipped;
                break;
            }

            let item_result = self.execute_item(item).await;
            match item_result {
                Ok(result) => {
                    if result.success {
                        succeeded += 1;
                    } else {
                        failed += 1;
                    }
                    results.push(result);
                }
                Err(e) => {
                    failed += 1;
                    results.push(ArpeggioItemResult {
                        id: item.id.clone(),
                        success: false,
                        output: String::new(),
                        duration_ms: 0,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        let completed_at = Utc::now();
        let total_duration_ms = start_instant.elapsed().as_millis() as u64;

        info!(
            "Arpeggio complete: {}/{} succeeded, {} failed, {} skipped",
            succeeded, total, failed, skipped
        );

        Ok(ArpeggioResult {
            piece_name: self.config.piece_name.clone(),
            items: results,
            total,
            succeeded,
            failed,
            skipped,
            total_duration_ms,
            started_at,
            completed_at,
        })
    }

    /// Execute a single item
    async fn execute_item(&self, item: &ArpeggioItem) -> Result<ArpeggioItemResult> {
        let start = std::time::Instant::now();

        debug!(
            "Executing item '{}': {}",
            item.id,
            item.task_text.chars().take(80).collect::<String>()
        );

        // Merge shared variables with item variables
        let mut _variables = self.config.shared_variables.clone();
        _variables.extend(item.variables.clone());

        // In production, this would call PieceEngine.execute_piece()
        // For now, simulate execution
        let output = format!(
            "Executed piece '{}' for item '{}': {}",
            self.config.piece_name, item.id, item.task_text
        );

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(ArpeggioItemResult {
            id: item.id.clone(),
            success: true,
            output,
            duration_ms,
            error: None,
        })
    }

    /// Create items from a list of task strings
    pub fn items_from_strings(tasks: &[String]) -> Vec<ArpeggioItem> {
        tasks
            .iter()
            .enumerate()
            .map(|(i, task)| ArpeggioItem {
                id: format!("item-{}", i),
                task_text: task.clone(),
                variables: HashMap::new(),
            })
            .collect()
    }

    /// Create items from a newline-separated string
    pub fn items_from_text(text: &str) -> Vec<ArpeggioItem> {
        let tasks: Vec<String> = text
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect();
        Self::items_from_strings(&tasks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arpeggio_config_default() {
        let config = ArpeggioConfig::default();
        assert_eq!(config.max_concurrency, 1);
        assert!(!config.fail_fast);
        assert!(config.timeout_secs.is_none());
    }

    #[tokio::test]
    async fn test_arpeggio_execute_sequential() {
        let config = ArpeggioConfig {
            piece_name: "test".to_string(),
            ..ArpeggioConfig::default()
        };
        let executor = ArpeggioExecutor::new(config);

        let items = vec![
            ArpeggioItem {
                id: "a".to_string(),
                task_text: "Task A".to_string(),
                variables: HashMap::new(),
            },
            ArpeggioItem {
                id: "b".to_string(),
                task_text: "Task B".to_string(),
                variables: HashMap::new(),
            },
        ];

        let result = executor.execute(items).await.unwrap();
        assert_eq!(result.total, 2);
        assert_eq!(result.succeeded, 2);
        assert_eq!(result.failed, 0);
        assert!(result.all_succeeded());
        assert_eq!(result.success_rate(), 100.0);
    }

    #[test]
    fn test_items_from_strings() {
        let tasks = vec![
            "Fix bug #1".to_string(),
            "Fix bug #2".to_string(),
            "Fix bug #3".to_string(),
        ];
        let items = ArpeggioExecutor::items_from_strings(&tasks);
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].id, "item-0");
        assert_eq!(items[2].task_text, "Fix bug #3");
    }

    #[test]
    fn test_items_from_text() {
        let text = "Fix login page\nUpdate API docs\n\nRefactor auth module\n";
        let items = ArpeggioExecutor::items_from_text(text);
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_result_metrics() {
        let result = ArpeggioResult {
            piece_name: "test".to_string(),
            items: vec![],
            total: 10,
            succeeded: 7,
            failed: 2,
            skipped: 1,
            total_duration_ms: 5000,
            started_at: Utc::now(),
            completed_at: Utc::now(),
        };

        assert!(!result.all_succeeded());
        assert!((result.success_rate() - 70.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_fail_fast() {
        // This test verifies the fail_fast flag structure
        let config = ArpeggioConfig {
            piece_name: "test".to_string(),
            fail_fast: true,
            ..ArpeggioConfig::default()
        };
        let executor = ArpeggioExecutor::new(config);

        let items = vec![ArpeggioItem {
            id: "ok".to_string(),
            task_text: "Good task".to_string(),
            variables: HashMap::new(),
        }];

        // Since our simulated executor always succeeds, this should pass
        let result = executor.execute(items).await.unwrap();
        assert_eq!(result.succeeded, 1);
        assert_eq!(result.skipped, 0);
    }

    #[test]
    fn test_shared_variables() {
        let mut shared = HashMap::new();
        shared.insert("repo".to_string(), "myorg/myapp".to_string());

        let config = ArpeggioConfig {
            shared_variables: shared,
            ..ArpeggioConfig::default()
        };

        assert_eq!(config.shared_variables.get("repo").unwrap(), "myorg/myapp");
    }
}
