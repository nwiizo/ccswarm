//! Utility functions and helpers

pub mod async_error_boundary;
pub mod async_error_boundary_simple;
pub mod async_macros;
pub mod command;
pub mod command_macros;
pub mod consolidated_error_handling;
pub mod error;
pub mod error_diagrams;
pub mod error_handling_macros;
pub mod error_recovery;
pub mod generic_handler;
pub mod macros;
pub mod user_error;
pub mod user_error_macros;

#[cfg(test)]
mod error_tests;

pub use async_error_boundary::{with_retry, AsyncCircuitBreaker, ConcurrentBoundary};
pub use async_error_boundary_simple::{boundary, boundary_with_fallback};
pub use command::CommandExecutor;
pub use consolidated_error_handling::{
    AgentError, CCSwarmError, ConfigError, ErrorContext, ErrorContextExt, GlobalErrorHandler,
    NetworkError, OrchestrationError, SessionError, TaskError,
};
pub use error::ResultExt;
pub use error_diagrams::{show_diagram, ErrorDiagrams};
pub use error_recovery::{ErrorRecoveryDB, ErrorResolver, RecoveryAction};
pub use user_error::{show_progress, CommonErrors, UserError, UserErrorExt};
