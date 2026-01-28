//! Dynamic Subagent Spawning System
//!
//! Enables task-time dynamic subagent generation for parallel processing.
//! Inspired by Claude Code TeammateTool patterns.

use super::{SubagentError, SubagentResult, manager::SubagentManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for spawning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnerConfig {
    /// Maximum concurrent spawned agents
    pub max_concurrent: usize,
    /// Default timeout for spawned agents (ms)
    pub default_timeout_ms: u64,
    /// Whether to auto-cleanup idle agents
    pub auto_cleanup: bool,
    /// Idle timeout before cleanup (seconds)
    pub idle_timeout_secs: u64,
    /// Enable parallel spawning
    pub enable_parallel: bool,
}

impl Default for SpawnerConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            default_timeout_ms: 300_000, // 5 minutes
            auto_cleanup: true,
            idle_timeout_secs: 120,
            enable_parallel: true,
        }
    }
}

/// Context for spawning a subagent
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpawnContext {
    /// Parent agent or session ID
    pub parent_id: Option<String>,
    /// Inherited environment variables
    pub env_vars: HashMap<String, String>,
    /// Working directory for the spawned agent
    pub working_directory: Option<String>,
    /// Priority level (higher = more important)
    pub priority: i32,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SpawnContext {
    /// Create a new spawn context with parent ID
    pub fn with_parent(parent_id: impl Into<String>) -> Self {
        Self {
            parent_id: Some(parent_id.into()),
            ..Default::default()
        }
    }

    /// Set working directory
    pub fn working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }

    /// Set priority
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add a tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add environment variable
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.insert(key.into(), value.into());
        self
    }
}

/// Configuration for handoff between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffConfig {
    /// Enable handoff capability
    pub enabled: bool,
    /// Allowed handoff targets
    pub allowed_targets: Vec<String>,
    /// Whether to pass context on handoff
    pub pass_context: bool,
    /// Maximum handoff chain depth
    pub max_depth: usize,
    /// Current depth in handoff chain
    pub current_depth: usize,
}

impl Default for HandoffConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_targets: Vec::new(),
            pass_context: true,
            max_depth: 5,
            current_depth: 0,
        }
    }
}

impl HandoffConfig {
    /// Check if handoff is allowed to a target
    pub fn can_handoff_to(&self, target: &str) -> bool {
        if !self.enabled {
            return false;
        }
        if self.current_depth >= self.max_depth {
            return false;
        }
        self.allowed_targets.is_empty() || self.allowed_targets.iter().any(|t| t == target)
    }

    /// Create a config for the next handoff
    pub fn next_depth(&self) -> Self {
        Self {
            current_depth: self.current_depth + 1,
            ..self.clone()
        }
    }
}

/// Execution configuration for spawned agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Maximum retries on failure
    pub max_retries: u32,
    /// Delay between retries (ms)
    pub retry_delay_ms: u64,
    /// Whether this agent can spawn children
    pub can_spawn_children: bool,
    /// Maximum child agents
    pub max_children: usize,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            timeout_ms: Some(300_000),
            max_retries: 3,
            retry_delay_ms: 1000,
            can_spawn_children: false,
            max_children: 3,
        }
    }
}

/// Resource limits for spawned agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum tokens per request
    pub max_tokens: Option<usize>,
    /// Maximum memory usage (bytes)
    pub max_memory_bytes: Option<usize>,
    /// Maximum execution time (ms)
    pub max_execution_time_ms: Option<u64>,
    /// Rate limit (requests per minute)
    pub rate_limit_rpm: Option<u32>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_tokens: Some(200_000),
            max_memory_bytes: None,
            max_execution_time_ms: Some(600_000), // 10 minutes
            rate_limit_rpm: None,
        }
    }
}

/// Task to spawn an agent for
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnTask {
    /// Task ID
    pub id: String,
    /// Task description/prompt
    pub prompt: String,
    /// Task priority
    pub priority: i32,
    /// Expected result type
    pub expected_result: Option<String>,
    /// Additional context for the task
    pub context: HashMap<String, serde_json::Value>,
}

impl SpawnTask {
    /// Create a new spawn task
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            prompt: prompt.into(),
            priority: 0,
            expected_result: None,
            context: HashMap::new(),
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Add context
    pub fn with_context(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.context.insert(key.into(), value);
        self
    }

    /// Set custom task ID
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }
}

/// Result of spawning an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnResult {
    /// Instance ID of the spawned agent
    pub instance_id: String,
    /// Task ID if a task was assigned
    pub task_id: Option<String>,
    /// Status of the spawn operation
    pub status: SpawnStatus,
    /// Error message if failed
    pub error: Option<String>,
    /// Time taken to spawn (ms)
    pub spawn_time_ms: u64,
}

/// Status of a spawn operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpawnStatus {
    /// Spawning in progress
    Pending,
    /// Successfully spawned
    Success,
    /// Spawn failed
    Failed,
    /// Spawn was cancelled
    Cancelled,
}

/// Dynamic spawner for creating agents on-demand
pub struct DynamicSpawner {
    /// Reference to subagent manager
    manager: Arc<RwLock<SubagentManager>>,
    /// Spawner configuration
    config: SpawnerConfig,
    /// Active spawn operations
    active_spawns: Arc<RwLock<HashMap<String, SpawnResult>>>,
}

impl DynamicSpawner {
    /// Create a new dynamic spawner
    pub fn new(manager: Arc<RwLock<SubagentManager>>, config: SpawnerConfig) -> Self {
        Self {
            manager,
            config,
            active_spawns: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default configuration
    pub fn with_defaults(manager: Arc<RwLock<SubagentManager>>) -> Self {
        Self::new(manager, SpawnerConfig::default())
    }

    /// Spawn a single agent from a definition
    pub async fn spawn_agent(
        &self,
        definition_name: &str,
        _context: SpawnContext,
        task: Option<SpawnTask>,
    ) -> SubagentResult<SpawnResult> {
        let start = std::time::Instant::now();

        // Check concurrent limit
        {
            let spawns = self.active_spawns.read().await;
            let active = spawns
                .values()
                .filter(|s| s.status == SpawnStatus::Pending || s.status == SpawnStatus::Success)
                .count();
            if active >= self.config.max_concurrent {
                return Err(SubagentError::Validation(format!(
                    "Maximum concurrent spawns ({}) reached",
                    self.config.max_concurrent
                )));
            }
        }

        // Create the instance
        let manager = self.manager.read().await;
        let instance_id = manager.create_subagent(definition_name).await?;

        // Optionally assign a task
        let task_id = if let Some(t) = task {
            // Wait for agent to be available
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            let tid = manager.delegate_task(&instance_id, &t.prompt).await?;
            Some(tid)
        } else {
            None
        };

        let result = SpawnResult {
            instance_id: instance_id.clone(),
            task_id,
            status: SpawnStatus::Success,
            error: None,
            spawn_time_ms: start.elapsed().as_millis() as u64,
        };

        // Track the spawn
        {
            let mut spawns = self.active_spawns.write().await;
            spawns.insert(instance_id.clone(), result.clone());
        }

        tracing::info!(
            instance_id = %instance_id,
            spawn_time_ms = result.spawn_time_ms,
            "Agent spawned successfully"
        );

        Ok(result)
    }

    /// Spawn multiple agents in parallel
    pub async fn spawn_parallel(
        &self,
        definition_name: &str,
        tasks: Vec<SpawnTask>,
    ) -> SubagentResult<Vec<SpawnResult>> {
        if !self.config.enable_parallel {
            // Fall back to sequential
            let mut results = Vec::new();
            for task in tasks {
                let ctx = SpawnContext::default().priority(task.priority);
                let result = self.spawn_agent(definition_name, ctx, Some(task)).await?;
                results.push(result);
            }
            return Ok(results);
        }

        // Spawn in parallel
        let futures: Vec<_> = tasks
            .into_iter()
            .map(|task| {
                let ctx = SpawnContext::default().priority(task.priority);
                self.spawn_agent(definition_name, ctx, Some(task))
            })
            .collect();

        let results: Vec<SubagentResult<SpawnResult>> = futures::future::join_all(futures).await;

        // Collect successful results, log errors
        let mut successful = Vec::new();
        for result in results {
            match result {
                Ok(r) => successful.push(r),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to spawn parallel agent");
                }
            }
        }

        Ok(successful)
    }

    /// Get status of a spawn operation
    pub async fn get_spawn_status(&self, instance_id: &str) -> Option<SpawnResult> {
        let spawns = self.active_spawns.read().await;
        spawns.get(instance_id).cloned()
    }

    /// List active spawns
    pub async fn list_active(&self) -> Vec<SpawnResult> {
        let spawns = self.active_spawns.read().await;
        spawns.values().cloned().collect()
    }

    /// Cleanup completed/failed spawns
    pub async fn cleanup(&self) -> usize {
        let mut spawns = self.active_spawns.write().await;
        let before = spawns.len();
        spawns.retain(|_, s| s.status == SpawnStatus::Pending);
        before - spawns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_context_builder() {
        let ctx = SpawnContext::with_parent("parent-1")
            .working_directory("/tmp/work")
            .priority(5)
            .tag("test")
            .env("KEY", "value");

        assert_eq!(ctx.parent_id, Some("parent-1".to_string()));
        assert_eq!(ctx.working_directory, Some("/tmp/work".to_string()));
        assert_eq!(ctx.priority, 5);
        assert!(ctx.tags.contains(&"test".to_string()));
        assert_eq!(ctx.env_vars.get("KEY"), Some(&"value".to_string()));
    }

    #[test]
    fn test_handoff_config() {
        let mut config = HandoffConfig::default();
        config.allowed_targets = vec!["backend".to_string(), "frontend".to_string()];

        assert!(config.can_handoff_to("backend"));
        assert!(config.can_handoff_to("frontend"));
        assert!(!config.can_handoff_to("devops"));

        // Test depth limiting
        let mut deep = config.clone();
        deep.current_depth = deep.max_depth;
        assert!(!deep.can_handoff_to("backend"));
    }

    #[test]
    fn test_spawn_task_creation() {
        let task = SpawnTask::new("Implement feature X")
            .with_priority(10)
            .with_context("file", serde_json::json!("main.rs"));

        assert_eq!(task.priority, 10);
        assert!(task.context.contains_key("file"));
    }

    #[test]
    fn test_execution_config_defaults() {
        let config = ExecutionConfig::default();
        assert_eq!(config.max_retries, 3);
        assert!(!config.can_spawn_children);
    }
}
