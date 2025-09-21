//! Error types for Claude Code ACP integration

use thiserror::Error;

/// Claude ACP error types
#[derive(Error, Debug, Clone)]
pub enum ACPError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    #[error("Claude Code not running or not accessible")]
    ServiceNotAvailable,

    #[error("Not connected to Claude Code")]
    NotConnected,

    #[error("Invalid response from Claude Code: {0}")]
    InvalidResponse(String),

    #[error("Task execution failed: {0}")]
    TaskFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("JSON parsing error: {0}")]
    JsonError(String),
}

/// Result type for ACP operations
pub type ACPResult<T> = Result<T, ACPError>;

impl From<serde_json::Error> for ACPError {
    fn from(err: serde_json::Error) -> Self {
        ACPError::JsonError(err.to_string())
    }
}

impl From<std::io::Error> for ACPError {
    fn from(err: std::io::Error) -> Self {
        ACPError::ConnectionError(err.to_string())
    }
}
