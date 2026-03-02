use thiserror::Error;

/// Main error type for ccswarm with structured error handling
///
/// # Examples
///
/// ```rust
/// use ccswarm::error::CCSwarmError;
///
/// // Creating a configuration error
/// let config_error = CCSwarmError::Configuration {
///     message: "Invalid agent configuration".to_string(),
///     source: None,
/// };
/// ```
#[derive(Error, Debug)]
pub enum CCSwarmError {
    /// IO operation failed
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization failed
    #[error("JSON error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// Configuration related error
    #[error("Configuration error: {message}")]
    Configuration {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Agent operation failed
    #[error("Agent error [{agent_id}]: {message}")]
    Agent {
        agent_id: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Session management error
    #[error("Session error [{session_id}]: {message}")]
    Session {
        session_id: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Task execution error
    #[error("Task error [{task_id}]: {message}")]
    Task {
        task_id: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Orchestrator coordination error
    #[error("Orchestrator error: {message}")]
    Orchestrator {
        message: String,
        task_id: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Git operation error
    #[error("Git error: {message}")]
    Git {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// User-facing error with helpful message
    #[error("{message}")]
    UserError {
        message: String,
        suggestion: Option<String>,
    },

    /// Generic error for cases not covered above
    #[error("{message}")]
    Other {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl From<String> for CCSwarmError {
    fn from(error: String) -> Self {
        Self::Other {
            message: error,
            source: None,
        }
    }
}

impl From<&str> for CCSwarmError {
    fn from(error: &str) -> Self {
        Self::Other {
            message: error.to_string(),
            source: None,
        }
    }
}

/// Result type alias for ccswarm operations
pub type Result<T> = std::result::Result<T, CCSwarmError>;

/// Helper trait for creating structured errors
pub trait ErrorContext<T> {
    /// Add context to an error
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;

    /// Add context with error source
    fn with_context_and_source<F, E>(self, f: F, source: E) -> Result<T>
    where
        F: FnOnce() -> String,
        E: std::error::Error + Send + Sync + 'static;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| CCSwarmError::Other {
            message: f(),
            source: Some(Box::new(e)),
        })
    }

    fn with_context_and_source<F, S>(self, f: F, source: S) -> Result<T>
    where
        F: FnOnce() -> String,
        S: std::error::Error + Send + Sync + 'static,
    {
        self.map_err(|_| CCSwarmError::Other {
            message: f(),
            source: Some(Box::new(source)),
        })
    }
}

/// Convenience methods for creating specific error types
impl CCSwarmError {
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
            source: None,
        }
    }

    pub fn agent<S: Into<String>, I: Into<String>>(agent_id: I, message: S) -> Self {
        Self::Agent {
            agent_id: agent_id.into(),
            message: message.into(),
            source: None,
        }
    }

    pub fn session<S: Into<String>, I: Into<String>>(session_id: I, message: S) -> Self {
        Self::Session {
            session_id: session_id.into(),
            message: message.into(),
            source: None,
        }
    }

    pub fn orchestrator<S: Into<String>>(message: S, task_id: Option<String>) -> Self {
        Self::Orchestrator {
            message: message.into(),
            task_id,
            source: None,
        }
    }

    pub fn task<S: Into<String>, I: Into<String>>(task_id: I, message: S) -> Self {
        Self::Task {
            task_id: task_id.into(),
            message: message.into(),
            source: None,
        }
    }

    pub fn git<S: Into<String>>(message: S) -> Self {
        Self::Git {
            message: message.into(),
            source: None,
        }
    }

    pub fn user_error<S: Into<String>>(message: S) -> Self {
        Self::UserError {
            message: message.into(),
            suggestion: None,
        }
    }

    pub fn user_error_with_suggestion<S: Into<String>, T: Into<String>>(
        message: S,
        suggestion: T,
    ) -> Self {
        Self::UserError {
            message: message.into(),
            suggestion: Some(suggestion.into()),
        }
    }

    /// Add a source error to this error
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        match &mut self {
            Self::Configuration { source: s, .. }
            | Self::Agent { source: s, .. }
            | Self::Session { source: s, .. }
            | Self::Task { source: s, .. }
            | Self::Git { source: s, .. }
            | Self::Orchestrator { source: s, .. }
            | Self::Other { source: s, .. } => {
                *s = Some(Box::new(source));
            }
            _ => {}
        }
        self
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(self, Self::Io(_) | Self::Task { .. })
    }

    /// Check if this error should be retried
    pub fn should_retry(&self) -> bool {
        match self {
            Self::Io(io_err) => {
                matches!(
                    io_err.kind(),
                    std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::ConnectionAborted
                        | std::io::ErrorKind::BrokenPipe
                        | std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::Interrupted
                        | std::io::ErrorKind::WouldBlock
                )
            }
            _ => false,
        }
    }

    /// Get the suggested delay before retrying this error
    pub fn suggested_retry_delay(&self) -> std::time::Duration {
        match self {
            Self::Io(_) => std::time::Duration::from_millis(500),
            _ => std::time::Duration::from_secs(1),
        }
    }

    /// Get the maximum number of retries recommended for this error
    pub fn max_retries(&self) -> u32 {
        if !self.should_retry() {
            return 0;
        }
        match self {
            Self::Io(_) => 2,
            _ => 0,
        }
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Configuration { .. } => ErrorSeverity::Critical,
            Self::Agent { .. } | Self::Session { .. } | Self::Orchestrator { .. } => {
                ErrorSeverity::High
            }
            Self::Task { .. } | Self::Git { .. } => ErrorSeverity::Medium,
            Self::Io(_) | Self::SerdeJson(_) => ErrorSeverity::Medium,
            Self::UserError { .. } => ErrorSeverity::Info,
            Self::Other { .. } => ErrorSeverity::Medium,
        }
    }

    /// Check if this error should be treated as fatal
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::Configuration { .. } | Self::Orchestrator { .. })
    }
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Low => write!(f, "LOW"),
            Self::Medium => write!(f, "MEDIUM"),
            Self::High => write!(f, "HIGH"),
            Self::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Convert CCSwarmError to user-friendly error messages with suggestions
impl CCSwarmError {
    pub fn to_user_error(&self) -> crate::utils::user_error::UserError {
        use crate::utils::user_error::UserError;

        match self {
            Self::Configuration { message, .. } => {
                UserError::new("Configuration Error", message.as_str())
                    .with_suggestion("Check your ccswarm.json file. Run 'ccswarm doctor' to diagnose configuration issues.")
            }
            Self::Agent { agent_id, message, .. } => {
                UserError::new(format!("Agent Error [{}]", agent_id), message.as_str())
                    .with_suggestion(format!(
                        "Check agent '{}' status with 'ccswarm status --agent {}'. Try 'ccswarm doctor --fix' to resolve common issues.",
                        agent_id, agent_id
                    ))
            }
            Self::Session { session_id, message, .. } => {
                UserError::new(format!("Session Error [{}]", session_id), message.as_str())
                    .with_suggestion("Run 'ccswarm session list' to check active sessions. Try restarting with 'ccswarm stop && ccswarm start'.")
            }
            Self::Task { task_id, message, .. } => {
                UserError::new(format!("Task Error [{}]", task_id), message.as_str())
                    .with_suggestion(format!(
                        "Check task status with 'ccswarm task list'. Retry with 'ccswarm task retry {}'.",
                        task_id
                    ))
            }
            Self::Git { message, .. } => {
                UserError::new("Git Error", message.as_str())
                    .with_suggestion("Ensure you're in a Git repository. Run 'git status' to check, or 'ccswarm doctor' for diagnostics.")
            }
            Self::Orchestrator { message, .. } => {
                UserError::new("Orchestrator Error", message.as_str())
                    .with_suggestion("Try restarting the orchestrator: 'ccswarm stop && ccswarm start'.")
            }
            Self::Io(io_err) => {
                UserError::new("I/O Error", &io_err.to_string())
                    .with_suggestion("Check file permissions and disk space. Ensure the target path exists.")
            }
            Self::SerdeJson(json_err) => {
                UserError::new("JSON Parse Error", &json_err.to_string())
                    .with_suggestion("Check your configuration file for syntax errors. Use a JSON validator to find issues.")
            }
            Self::UserError { message, suggestion } => {
                let err = UserError::new("Error", message.as_str());
                if let Some(s) = suggestion {
                    err.with_suggestion(s.clone())
                } else {
                    err
                }
            }
            Self::Other { message, .. } => {
                UserError::new("Error", message.as_str())
                    .with_suggestion("Run 'ccswarm doctor' for diagnostics. Use '--verbose' for more details.")
            }
        }
    }
}
