use crate::error::CCSwarmError;
use std::fmt;

/// Consolidated error handling utilities
pub struct ErrorContext {
    context: Vec<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            context: Vec::new(),
        }
    }

    pub fn add_context(&mut self, ctx: impl Into<String>) {
        self.context.push(ctx.into());
    }

    pub fn wrap_error(&self, error: CCSwarmError) -> CCSwarmError {
        let context_str = self.context.join(" -> ");
        CCSwarmError::Other {
            message: format!("{}: {}", context_str, error),
            source: None,
        }
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Result type with consolidated error handling
pub type ConsolidatedResult<T> = Result<T, ConsolidatedError>;

#[derive(Debug)]
pub struct ConsolidatedError {
    pub error: CCSwarmError,
    pub context: Vec<String>,
}

impl fmt::Display for ConsolidatedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;
        if !self.context.is_empty() {
            write!(f, "\nContext: {}", self.context.join(" -> "))?;
        }
        Ok(())
    }
}

impl std::error::Error for ConsolidatedError {}

impl From<CCSwarmError> for ConsolidatedError {
    fn from(error: CCSwarmError) -> Self {
        Self {
            error,
            context: Vec::new(),
        }
    }
}

// Use error types from main error module (don't re-export to avoid name conflicts)

// Additional error types
#[derive(Debug)]
pub struct AgentError(pub String);
#[derive(Debug)]
pub struct ConfigError(pub String);
#[derive(Debug)]
pub struct NetworkError(pub String);
#[derive(Debug)]
pub struct OrchestrationError(pub String);
#[derive(Debug)]
pub struct SessionError(pub String);
#[derive(Debug)]
pub struct TaskError(pub String);

impl std::error::Error for AgentError {}
impl std::error::Error for ConfigError {}
impl std::error::Error for NetworkError {}
impl std::error::Error for OrchestrationError {}
impl std::error::Error for SessionError {}
impl std::error::Error for TaskError {}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Agent error: {}", self.0)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Config error: {}", self.0)
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Network error: {}", self.0)
    }
}

impl fmt::Display for OrchestrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Orchestration error: {}", self.0)
    }
}

impl fmt::Display for SessionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Session error: {}", self.0)
    }
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task error: {}", self.0)
    }
}

/// Error context extension trait
pub trait ErrorContextExt {
    fn context(self, msg: &str) -> Self;
}

impl ErrorContextExt for CCSwarmError {
    fn context(self, msg: &str) -> Self {
        CCSwarmError::Other {
            message: format!("{}: {}", msg, self),
            source: None,
        }
    }
}

/// Global error handler
pub struct GlobalErrorHandler;

impl GlobalErrorHandler {
    pub fn handle(error: CCSwarmError) {
        eprintln!("Global error: {}", error);
    }
}
