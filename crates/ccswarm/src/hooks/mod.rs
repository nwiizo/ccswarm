//! Hook System for ccswarm
//!
//! Provides PreToolUse/PostToolUse patterns inspired by Claude Agent SDK.
//! Allows interception and modification of execution flow at key points.
//!
//! # Hook Types
//! - **ExecutionHooks**: Pre/post execution and error handling
//! - **ToolHooks**: Pre/post tool use interception
//!
//! # Example
//! ```ignore
//! use ccswarm::hooks::{HookRegistry, LoggingHook};
//!
//! let mut registry = HookRegistry::new();
//! registry.register_execution_hook(Arc::new(LoggingHook::new()));
//! ```

mod builtin;
mod registry;

pub use builtin::{LoggingHook, SecurityHook};
pub use registry::HookRegistry;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookResult {
    /// Continue with normal execution
    Continue,
    /// Continue with modified input
    ContinueWith(serde_json::Value),
    /// Skip this operation (non-error)
    Skip { reason: String },
    /// Deny this operation (error)
    Deny { reason: String },
    /// Abort the entire task
    Abort { reason: String },
}

impl HookResult {
    /// Check if execution should continue
    pub fn should_continue(&self) -> bool {
        matches!(self, HookResult::Continue | HookResult::ContinueWith(_))
    }

    /// Check if operation was denied
    pub fn is_denied(&self) -> bool {
        matches!(self, HookResult::Deny { .. })
    }

    /// Check if task was aborted
    pub fn is_aborted(&self) -> bool {
        matches!(self, HookResult::Abort { .. })
    }
}

/// Context passed to hooks
#[derive(Debug, Clone)]
pub struct HookContext {
    /// Agent ID executing the operation
    pub agent_id: String,
    /// Session ID if available
    pub session_id: Option<String>,
    /// Task ID if available
    pub task_id: Option<String>,
    /// Working directory
    pub working_directory: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl HookContext {
    /// Create a new hook context
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            session_id: None,
            task_id: None,
            working_directory: None,
            metadata: HashMap::new(),
        }
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set task ID
    pub fn with_task(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    /// Set working directory
    pub fn with_working_directory(mut self, dir: impl Into<String>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Input for pre-execution hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreExecutionInput {
    /// Task description
    pub task_description: String,
    /// Task type
    pub task_type: String,
    /// Priority level
    pub priority: String,
    /// Additional task details
    pub details: Option<String>,
}

/// Input for post-execution hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostExecutionInput {
    /// Task description
    pub task_description: String,
    /// Whether execution succeeded
    pub success: bool,
    /// Execution output
    pub output: serde_json::Value,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

/// Input for error hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnErrorInput {
    /// Error message
    pub error_message: String,
    /// Error type/category
    pub error_type: String,
    /// Whether error is recoverable
    pub is_recoverable: bool,
    /// Stack trace if available
    pub stack_trace: Option<String>,
}

/// Input for pre-tool-use hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreToolUseInput {
    /// Tool name being invoked
    pub tool_name: String,
    /// Tool arguments
    pub arguments: serde_json::Value,
    /// Tool description
    pub description: Option<String>,
}

/// Input for post-tool-use hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostToolUseInput {
    /// Tool name that was invoked
    pub tool_name: String,
    /// Tool arguments that were used
    pub arguments: serde_json::Value,
    /// Whether tool execution succeeded
    pub success: bool,
    /// Tool result
    pub result: serde_json::Value,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
}

/// Execution lifecycle hooks
#[async_trait]
pub trait ExecutionHooks: Send + Sync {
    /// Called before task execution begins
    async fn pre_execution(&self, input: PreExecutionInput, ctx: HookContext) -> HookResult;

    /// Called after task execution completes
    async fn post_execution(&self, input: PostExecutionInput, ctx: HookContext) -> HookResult;

    /// Called when an error occurs during execution
    async fn on_error(&self, input: OnErrorInput, ctx: HookContext) -> HookResult;

    /// Hook name for identification
    fn name(&self) -> &str;

    /// Hook priority (higher = runs earlier)
    fn priority(&self) -> i32 {
        0
    }
}

/// Tool usage hooks
#[async_trait]
pub trait ToolHooks: Send + Sync {
    /// Called before a tool is used
    async fn pre_tool_use(&self, input: PreToolUseInput, ctx: HookContext) -> HookResult;

    /// Called after a tool is used
    async fn post_tool_use(&self, input: PostToolUseInput, ctx: HookContext) -> HookResult;

    /// Hook name for identification
    fn name(&self) -> &str;

    /// Hook priority (higher = runs earlier)
    fn priority(&self) -> i32 {
        0
    }
}

/// Combined hook trait for convenience
#[async_trait]
pub trait AllHooks: ExecutionHooks + ToolHooks {}

// Auto-implement AllHooks for types that implement both traits
impl<T: ExecutionHooks + ToolHooks> AllHooks for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_result_checks() {
        assert!(HookResult::Continue.should_continue());
        assert!(HookResult::ContinueWith(serde_json::json!({})).should_continue());
        assert!(
            !HookResult::Skip {
                reason: "test".into()
            }
            .should_continue()
        );
        assert!(
            HookResult::Deny {
                reason: "test".into()
            }
            .is_denied()
        );
        assert!(
            HookResult::Abort {
                reason: "test".into()
            }
            .is_aborted()
        );
    }

    #[test]
    fn test_hook_context_builder() {
        let ctx = HookContext::new("agent-1")
            .with_session("session-1")
            .with_task("task-1")
            .with_working_directory("/tmp")
            .with_metadata("key", serde_json::json!("value"));

        assert_eq!(ctx.agent_id, "agent-1");
        assert_eq!(ctx.session_id, Some("session-1".to_string()));
        assert_eq!(ctx.task_id, Some("task-1".to_string()));
        assert_eq!(ctx.working_directory, Some("/tmp".to_string()));
        assert!(ctx.metadata.contains_key("key"));
    }
}
