//! Agent-specific session management for ccswarm integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent types supported by ai-session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// Claude Code agent
    ClaudeCode,
    /// Aider agent
    Aider,
    /// OpenAI Codex
    Codex,
    /// Custom LLM agent
    Custom,
    /// Standard shell (no AI)
    Shell,
}

/// Agent-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent type
    pub agent_type: AgentType,
    /// Startup command (e.g., "claude", "aider")
    pub startup_command: Option<String>,
    /// Prompt prefix for commands
    pub prompt_prefix: Option<String>,
    /// Prompt suffix for commands
    pub prompt_suffix: Option<String>,
    /// Response parsing pattern
    pub response_pattern: Option<String>,
    /// Timeout for responses
    pub response_timeout_ms: u64,
    /// Enable streaming output
    pub streaming: bool,
    /// Custom settings
    pub custom_settings: HashMap<String, serde_json::Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_type: AgentType::Shell,
            startup_command: None,
            prompt_prefix: None,
            prompt_suffix: None,
            response_pattern: None,
            response_timeout_ms: 30000,
            streaming: false,
            custom_settings: HashMap::new(),
        }
    }
}

impl AgentConfig {
    /// Create Claude Code agent configuration
    pub fn claude_code() -> Self {
        Self {
            agent_type: AgentType::ClaudeCode,
            startup_command: Some("claude".to_string()),
            prompt_prefix: None,
            prompt_suffix: Some("\n".to_string()),
            response_pattern: Some(r"(?s).*\n\n(.+)$".to_string()),
            response_timeout_ms: 60000,
            streaming: true,
            custom_settings: HashMap::new(),
        }
    }

    /// Create Aider agent configuration
    pub fn aider() -> Self {
        Self {
            agent_type: AgentType::Aider,
            startup_command: Some("aider".to_string()),
            prompt_prefix: None,
            prompt_suffix: Some("\n".to_string()),
            response_pattern: Some(r"(?s).*\n\n(.+)$".to_string()),
            response_timeout_ms: 60000,
            streaming: true,
            custom_settings: HashMap::new(),
        }
    }
}

/// Agent session wrapper with enhanced capabilities
pub struct AgentSession {
    /// Base session ID
    pub session_id: String,
    /// Agent configuration
    pub config: AgentConfig,
    /// Conversation history
    pub history: Vec<ConversationTurn>,
    /// Current context tokens
    pub context_tokens: usize,
    /// Active task
    pub current_task: Option<AgentTask>,
}

/// Single conversation turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// User input
    pub input: String,
    /// Agent response
    pub response: String,
    /// Token count
    pub tokens: usize,
    /// Execution time in ms
    pub execution_time_ms: u64,
}

/// Task assigned to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    /// Task ID
    pub id: String,
    /// Task type
    pub task_type: TaskType,
    /// Task description
    pub description: String,
    /// Task parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Task status
    pub status: TaskStatus,
    /// Parent task (for subtasks)
    pub parent_task_id: Option<String>,
    /// Dependencies
    pub depends_on: Vec<String>,
}

/// Task types for agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// Code generation
    CodeGeneration,
    /// Code review
    CodeReview,
    /// Debugging
    Debugging,
    /// Documentation
    Documentation,
    /// Testing
    Testing,
    /// Refactoring
    Refactoring,
    /// Research
    Research,
    /// Custom task
    Custom,
}

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Pending execution
    Pending,
    /// Currently executing
    InProgress,
    /// Completed successfully
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
    /// Blocked by dependencies
    Blocked,
}

/// Message for inter-agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    /// Message ID
    pub id: String,
    /// Source agent
    pub from: String,
    /// Target agent
    pub to: String,
    /// Message type
    pub message_type: MessageType,
    /// Message content
    pub content: serde_json::Value,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Priority
    pub priority: MessagePriority,
}

/// Message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Task assignment
    TaskAssignment,
    /// Task result
    TaskResult,
    /// Information sharing
    Information,
    /// Query
    Query,
    /// Response
    Response,
    /// Error
    Error,
    /// Status update
    StatusUpdate,
}

/// Message priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Agent workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Workflow ID
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Workflow steps
    pub steps: Vec<WorkflowStep>,
    /// Current step index
    pub current_step: usize,
    /// Workflow status
    pub status: WorkflowStatus,
    /// Workflow context
    pub context: HashMap<String, serde_json::Value>,
}

/// Single workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step name
    pub name: String,
    /// Agent to execute
    pub agent: String,
    /// Task to execute
    pub task: AgentTask,
    /// Success condition
    pub success_condition: Option<String>,
    /// Retry policy
    pub retry_policy: RetryPolicy,
    /// Next steps based on outcome
    pub next_steps: HashMap<String, usize>,
}

/// Retry policy for workflow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Delay between retries in ms
    pub retry_delay_ms: u64,
    /// Exponential backoff
    pub exponential_backoff: bool,
}

/// Workflow status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Not started
    NotStarted,
    /// Running
    Running,
    /// Paused
    Paused,
    /// Completed
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}
