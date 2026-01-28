pub mod converter;
pub mod delegation;
pub mod dynamic_generation;
pub mod manager;
pub mod parallel_executor;
/// Claude Code Native Subagent Integration Module
///
/// This module provides integration with Claude Code's native subagent feature,
/// allowing ccswarm to leverage built-in subagent capabilities for improved
/// context management and parallel processing.
pub mod parser;
pub mod spawner;
pub mod workload_balancer;

// Re-export key types for easier access
// Note: MultiAgentExecutor temporarily disabled until ai-session integration is restored
pub use parallel_executor::{
    AggregationStrategy, ExecutionStatus, ParallelConfig, ParallelExecutionResult,
    ParallelExecutor, TaskExecutionResult,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents a Claude Code subagent definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentDefinition {
    /// Unique name for the subagent
    pub name: String,

    /// Description of the subagent's purpose
    pub description: String,

    /// List of tools available to the subagent
    pub tools: SubagentTools,

    /// Capabilities and specializations
    pub capabilities: Vec<String>,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,

    /// Execution configuration for this subagent
    #[serde(default)]
    pub execution_config: spawner::ExecutionConfig,

    /// Resource limits for this subagent
    #[serde(default)]
    pub resource_limits: spawner::ResourceLimits,

    /// Spawn context for dynamic spawning
    #[serde(default)]
    pub spawn_context: Option<spawner::SpawnContext>,

    /// Handoff configuration for agent-to-agent transfers
    #[serde(default)]
    pub handoff_config: Option<spawner::HandoffConfig>,
}

/// Tools available to a subagent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentTools {
    /// Standard tools (file operations, command execution, etc.)
    pub standard: Vec<String>,

    /// Semantic tools (symbol analysis, refactoring, etc.)
    #[serde(default)]
    pub semantic: Vec<String>,

    /// Memory tools (knowledge persistence)
    #[serde(default)]
    pub memory: Vec<String>,

    /// Custom tools specific to this subagent
    #[serde(default)]
    pub custom: Vec<String>,
}

/// Configuration for the subagent system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentConfig {
    /// Directory containing subagent definitions
    pub agents_dir: PathBuf,

    /// Enable dynamic subagent generation
    #[serde(default = "default_true")]
    pub enable_dynamic_generation: bool,

    /// Maximum number of concurrent subagents
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_agents: usize,

    /// Enable hybrid mode (use both old and new systems)
    #[serde(default)]
    pub hybrid_mode: bool,
}

fn default_true() -> bool {
    true
}

fn default_max_concurrent() -> usize {
    5
}

impl Default for SubagentConfig {
    fn default() -> Self {
        Self {
            agents_dir: PathBuf::from(".claude/agents"),
            enable_dynamic_generation: true,
            max_concurrent_agents: 5,
            hybrid_mode: false,
        }
    }
}

/// Result type for subagent operations
pub type SubagentResult<T> = Result<T, SubagentError>;

/// Errors that can occur in subagent operations
#[derive(Debug, thiserror::Error)]
pub enum SubagentError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Subagent not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Delegation error: {0}")]
    Delegation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Execution error: {0}")]
    ExecutionError(String),
}
