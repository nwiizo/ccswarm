use thiserror::Error;

/// Errors that can occur during session operations
#[derive(Debug, Error)]
pub enum SessionError {
    /// Session with the specified ID was not found
    #[error("Session not found: {id}")]
    NotFound { id: String },

    /// Session is in an invalid state for the requested operation
    #[error("Invalid session state: {state} for operation: {operation}")]
    InvalidState { state: String, operation: String },

    /// Failed to acquire a lock on session data
    #[error("Failed to acquire session lock: {reason}")]
    LockError { reason: String },

    /// Session creation failed
    #[error("Failed to create session: {reason}")]
    CreationFailed { reason: String },

    /// Session termination failed
    #[error("Failed to terminate session: {reason}")]
    TerminationFailed { reason: String },

    /// Resource monitoring error
    #[error("Resource monitoring error: {reason}")]
    ResourceError { reason: String },

    /// Context bridge error
    #[error("Context bridge error: {reason}")]
    ContextBridgeError { reason: String },

    /// Persistence error
    #[error("Session persistence error: {reason}")]
    PersistenceError { reason: String },

    /// Pool management error
    #[error("Session pool error: {reason}")]
    PoolError { reason: String },

    /// Memory management error
    #[error("Session memory error: {reason}")]
    MemoryError { reason: String },

    /// Generic error wrapping other error types
    #[error("Session error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Result type alias for session operations
pub type SessionResult<T> = Result<T, SessionError>;

/// Extension trait for converting mutex errors to SessionError
pub trait LockResultExt<T> {
    /// Convert a poisoned mutex error to a SessionError
    fn map_lock_error(self) -> SessionResult<T>;
}

impl<T> LockResultExt<T> for std::sync::LockResult<T> {
    fn map_lock_error(self) -> SessionResult<T> {
        self.map_err(|e| SessionError::LockError {
            reason: format!("Mutex poisoned: {}", e),
        })
    }
}
