/// Consolidated error handling using error diagrams and patterns
///
/// This module provides a unified approach to error handling across ccswarm.

use anyhow::{Context, Result};
use std::fmt;
use thiserror::Error;
use tracing::{error, warn};

use crate::utils::{ErrorDiagrams, show_diagram};
use crate::utils::error_recovery::{ErrorRecoveryDB, RecoveryAction};

/// Consolidated error types for ccswarm
#[derive(Error, Debug)]
pub enum CCSwarmError {
    #[error("Session error: {0}")]
    Session(#[from] SessionError),
    
    #[error("Orchestration error: {0}")]
    Orchestration(#[from] OrchestrationError),
    
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),
    
    #[error("Task error: {0}")]
    Task(#[from] TaskError),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigError),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Session-specific errors
#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Session not found: {id}")]
    NotFound { id: String },
    
    #[error("Session creation failed: {reason}")]
    CreationFailed { reason: String },
    
    #[error("Session pool exhausted for role: {role}")]
    PoolExhausted { role: String },
    
    #[error("Session validation failed: {details}")]
    ValidationFailed { details: String },
    
    #[error("Session timeout after {duration:?}")]
    Timeout { duration: std::time::Duration },
    
    #[error("Session compression failed: {reason}")]
    CompressionFailed { reason: String },
}

/// Orchestration-specific errors
#[derive(Error, Debug)]
pub enum OrchestrationError {
    #[error("No suitable agent for task: {task_id}")]
    NoSuitableAgent { task_id: String },
    
    #[error("Task delegation failed: {reason}")]
    DelegationFailed { reason: String },
    
    #[error("Circular dependency detected: {task_ids:?}")]
    CircularDependency { task_ids: Vec<String> },
    
    #[error("Consensus not reached: {votes_for}/{votes_total}")]
    ConsensusNotReached { votes_for: usize, votes_total: usize },
    
    #[error("Quality check failed: {score:.2} < {threshold:.2}")]
    QualityCheckFailed { score: f64, threshold: f64 },
}

/// Agent-specific errors
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Agent not available: {name}")]
    NotAvailable { name: String },
    
    #[error("Agent role violation: {agent} attempted {forbidden_action}")]
    RoleViolation { agent: String, forbidden_action: String },
    
    #[error("Agent overloaded: {current_load:.2} > {max_load:.2}")]
    Overloaded { current_load: f64, max_load: f64 },
    
    #[error("Agent initialization failed: {reason}")]
    InitializationFailed { reason: String },
}

/// Task-specific errors
#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Task execution failed: {task_id} - {reason}")]
    ExecutionFailed { task_id: String, reason: String },
    
    #[error("Task dependencies not met: {missing:?}")]
    DependenciesNotMet { missing: Vec<String> },
    
    #[error("Task already assigned to: {agent}")]
    AlreadyAssigned { agent: String },
    
    #[error("Invalid task state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    #[error("Invalid value for {field}: {value}")]
    InvalidValue { field: String, value: String },
    
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: String },
    
    #[error("Configuration parse error: {details}")]
    ParseError { details: String },
}

/// Network-related errors
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed to {endpoint}: {reason}")]
    ConnectionFailed { endpoint: String, reason: String },
    
    #[error("Request timeout to {endpoint} after {duration:?}")]
    RequestTimeout { endpoint: String, duration: std::time::Duration },
    
    #[error("Invalid response from {endpoint}: {details}")]
    InvalidResponse { endpoint: String, details: String },
}

/// Error context with diagnostic information
pub struct ErrorContext {
    pub error: CCSwarmError,
    pub stack_trace: Vec<String>,
    pub recovery_actions: Vec<RecoveryAction>,
    pub diagram: Option<String>,
}

impl ErrorContext {
    /// Create error context with automatic diagnosis
    pub fn new(error: CCSwarmError) -> Self {
        let recovery_actions = Self::suggest_recovery_actions(&error);
        let diagram = Self::generate_error_diagram(&error);
        
        Self {
            error,
            stack_trace: Vec::new(),
            recovery_actions,
            diagram,
        }
    }

    /// Add stack frame
    pub fn with_frame(mut self, frame: impl Into<String>) -> Self {
        self.stack_trace.push(frame.into());
        self
    }

    /// Suggest recovery actions based on error type
    fn suggest_recovery_actions(error: &CCSwarmError) -> Vec<RecoveryAction> {
        match error {
            CCSwarmError::Session(SessionError::PoolExhausted { role }) => vec![
                RecoveryAction::Retry {
                    max_attempts: 3,
                    backoff_ms: 1000,
                },
                RecoveryAction::Fallback {
                    description: format!("Use general-purpose agent instead of {}", role),
                },
                RecoveryAction::ScaleUp {
                    resource: "session_pool".to_string(),
                    factor: 1.5,
                },
            ],
            
            CCSwarmError::Orchestration(OrchestrationError::NoSuitableAgent { .. }) => vec![
                RecoveryAction::Retry {
                    max_attempts: 2,
                    backoff_ms: 500,
                },
                RecoveryAction::QueueTask {
                    priority: "high".to_string(),
                },
            ],
            
            CCSwarmError::Agent(AgentError::Overloaded { .. }) => vec![
                RecoveryAction::BackPressure {
                    duration_ms: 5000,
                },
                RecoveryAction::LoadBalance {
                    strategy: "least_loaded".to_string(),
                },
            ],
            
            CCSwarmError::Network(NetworkError::ConnectionFailed { .. }) => vec![
                RecoveryAction::Retry {
                    max_attempts: 5,
                    backoff_ms: 2000,
                },
                RecoveryAction::CircuitBreak {
                    threshold: 3,
                    timeout_ms: 30000,
                },
            ],
            
            _ => vec![RecoveryAction::Log {
                level: "error".to_string(),
            }],
        }
    }

    /// Generate error diagram
    fn generate_error_diagram(error: &CCSwarmError) -> Option<String> {
        match error {
            CCSwarmError::Session(SessionError::PoolExhausted { role }) => {
                Some(ErrorDiagrams::session_pool_exhausted(role))
            },
            
            CCSwarmError::Orchestration(OrchestrationError::CircularDependency { task_ids }) => {
                Some(ErrorDiagrams::circular_dependency(task_ids))
            },
            
            CCSwarmError::Agent(AgentError::RoleViolation { agent, forbidden_action }) => {
                Some(ErrorDiagrams::role_violation(agent, forbidden_action))
            },
            
            CCSwarmError::Network(NetworkError::ConnectionFailed { endpoint, reason }) => {
                Some(ErrorDiagrams::connection_failed(endpoint, reason))
            },
            
            _ => None,
        }
    }

    /// Display error with full context
    pub fn display(&self) -> String {
        let mut output = format!("Error: {}\n", self.error);
        
        if !self.stack_trace.is_empty() {
            output.push_str("\nStack Trace:\n");
            for (i, frame) in self.stack_trace.iter().enumerate() {
                output.push_str(&format!("  {}: {}\n", i + 1, frame));
            }
        }
        
        if !self.recovery_actions.is_empty() {
            output.push_str("\nSuggested Recovery Actions:\n");
            for action in &self.recovery_actions {
                output.push_str(&format!("  - {}\n", action));
            }
        }
        
        if let Some(diagram) = &self.diagram {
            output.push_str("\nError Diagram:\n");
            output.push_str(diagram);
        }
        
        output
    }
}

/// Extension trait for Result types
pub trait ErrorContextExt<T> {
    /// Add error context with automatic diagnosis
    fn with_error_context(self, frame: impl Into<String>) -> Result<T>;
    
    /// Convert to CCSwarmError with context
    fn into_ccswarm_error(self) -> Result<T, ErrorContext>;
}

impl<T, E> ErrorContextExt<T> for Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn with_error_context(self, frame: impl Into<String>) -> Result<T> {
        self.context(frame.into())
    }
    
    fn into_ccswarm_error(self) -> Result<T, ErrorContext> {
        self.map_err(|e| {
            let error = CCSwarmError::Unknown(e.into().to_string());
            ErrorContext::new(error)
        })
    }
}

/// Global error handler with recovery
pub struct GlobalErrorHandler {
    recovery_db: ErrorRecoveryDB,
    error_count: std::sync::atomic::AtomicUsize,
    circuit_breakers: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, bool>>>,
}

impl GlobalErrorHandler {
    pub fn new() -> Self {
        Self {
            recovery_db: ErrorRecoveryDB::new(),
            error_count: std::sync::atomic::AtomicUsize::new(0),
            circuit_breakers: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Handle error with automatic recovery
    pub async fn handle_error(&self, context: ErrorContext) -> Result<()> {
        // Increment error count
        self.error_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        // Log error with context
        error!("{}", context.display());
        
        // Show error diagram if available
        if let Some(diagram) = &context.diagram {
            show_diagram("error", diagram);
        }
        
        // Execute recovery actions
        for action in &context.recovery_actions {
            match action {
                RecoveryAction::Retry { max_attempts, backoff_ms } => {
                    warn!("Retrying operation: max_attempts={}, backoff={}ms", max_attempts, backoff_ms);
                }
                
                RecoveryAction::CircuitBreak { threshold, timeout_ms } => {
                    let key = format!("{:?}", context.error);
                    let mut breakers = self.circuit_breakers.write().await;
                    breakers.insert(key, true);
                    
                    tokio::spawn(async move {
                        tokio::time::sleep(tokio::time::Duration::from_millis(*timeout_ms)).await;
                        // Reset circuit breaker after timeout
                    });
                }
                
                _ => {
                    // Handle other recovery actions
                }
            }
        }
        
        Ok(())
    }

    /// Get error statistics
    pub fn get_error_stats(&self) -> ErrorStats {
        ErrorStats {
            total_errors: self.error_count.load(std::sync::atomic::Ordering::Relaxed),
            // Additional stats would be tracked here
        }
    }
}

#[derive(Debug)]
pub struct ErrorStats {
    pub total_errors: usize,
}

/// Utility functions for common error patterns
pub mod patterns {
    use super::*;

    /// Wrap operation with error context
    pub async fn with_context<F, T>(
        operation: F,
        context: &str,
    ) -> Result<T, ErrorContext>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        operation.await.map_err(|e| {
            let error = CCSwarmError::Unknown(e.to_string());
            ErrorContext::new(error).with_frame(context)
        })
    }

    /// Execute with automatic retry on specific errors
    pub async fn retry_on_error<F, T>(
        operation: F,
        should_retry: impl Fn(&CCSwarmError) -> bool,
        max_retries: usize,
    ) -> Result<T>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
    {
        let mut attempts = 0;
        
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if let Some(ccswarm_err) = e.downcast_ref::<CCSwarmError>() {
                        if attempts < max_retries && should_retry(ccswarm_err) {
                            attempts += 1;
                            warn!("Retrying after error (attempt {}/{}): {}", attempts, max_retries, ccswarm_err);
                            tokio::time::sleep(tokio::time::Duration::from_millis(1000 * attempts as u64)).await;
                            continue;
                        }
                    }
                    return Err(e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context_creation() {
        let error = CCSwarmError::Session(SessionError::PoolExhausted {
            role: "frontend".to_string(),
        });
        
        let context = ErrorContext::new(error);
        
        assert!(!context.recovery_actions.is_empty());
        assert!(context.diagram.is_some());
    }

    #[test]
    fn test_error_display() {
        let error = CCSwarmError::Agent(AgentError::RoleViolation {
            agent: "frontend-agent".to_string(),
            forbidden_action: "database access".to_string(),
        });
        
        let context = ErrorContext::new(error)
            .with_frame("orchestrator::delegate_task")
            .with_frame("agent::execute");
        
        let display = context.display();
        assert!(display.contains("Role violation"));
        assert!(display.contains("Stack Trace"));
        assert!(display.contains("Recovery Actions"));
    }

    #[tokio::test]
    async fn test_global_error_handler() {
        let handler = GlobalErrorHandler::new();
        
        let error = CCSwarmError::Network(NetworkError::ConnectionFailed {
            endpoint: "api.example.com".to_string(),
            reason: "timeout".to_string(),
        });
        
        let context = ErrorContext::new(error);
        let result = handler.handle_error(context).await;
        
        assert!(result.is_ok());
        assert_eq!(handler.get_error_stats().total_errors, 1);
    }
}