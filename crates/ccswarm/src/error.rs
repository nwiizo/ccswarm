use thiserror::Error;

/// Main error type for ccswarm with structured error handling
///
/// This enum provides comprehensive error types for all ccswarm operations,
/// with detailed context and proper error chaining using `thiserror`.
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

    /// Network communication error
    #[error("Network error: {message}")]
    Network {
        message: String,
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

    /// Template processing error
    #[error("Template error: {message}")]
    Template {
        message: String,
        template_name: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Extension system error
    #[error("Extension error [{extension_id}]: {message}")]
    Extension {
        extension_id: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Resource management error
    #[error("Resource error: {message}")]
    Resource {
        message: String,
        resource_type: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Authentication/authorization error
    #[error("Authentication error: {message}")]
    Auth {
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
    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Configuration {
            message: message.into(),
            source: None,
        }
    }

    /// Create an agent error
    pub fn agent<S: Into<String>, I: Into<String>>(agent_id: I, message: S) -> Self {
        Self::Agent {
            agent_id: agent_id.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a session error
    pub fn session<S: Into<String>, I: Into<String>>(session_id: I, message: S) -> Self {
        Self::Session {
            session_id: session_id.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a task error
    pub fn task<S: Into<String>, I: Into<String>>(task_id: I, message: S) -> Self {
        Self::Task {
            task_id: task_id.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a network error
    pub fn network<S: Into<String>>(message: S) -> Self {
        Self::Network {
            message: message.into(),
            source: None,
        }
    }

    /// Create a git error
    pub fn git<S: Into<String>>(message: S) -> Self {
        Self::Git {
            message: message.into(),
            source: None,
        }
    }

    /// Create a template error
    pub fn template<S: Into<String>>(message: S) -> Self {
        Self::Template {
            message: message.into(),
            template_name: None,
            source: None,
        }
    }

    /// Create a template error with template name
    pub fn template_with_name<S: Into<String>, N: Into<String>>(message: S, template_name: N) -> Self {
        Self::Template {
            message: message.into(),
            template_name: Some(template_name.into()),
            source: None,
        }
    }

    /// Create an extension error
    pub fn extension<S: Into<String>, I: Into<String>>(extension_id: I, message: S) -> Self {
        Self::Extension {
            extension_id: extension_id.into(),
            message: message.into(),
            source: None,
        }
    }

    /// Create a resource error
    pub fn resource<S: Into<String>>(message: S) -> Self {
        Self::Resource {
            message: message.into(),
            resource_type: None,
            source: None,
        }
    }

    /// Create a resource error with type
    pub fn resource_with_type<S: Into<String>, T: Into<String>>(message: S, resource_type: T) -> Self {
        Self::Resource {
            message: message.into(),
            resource_type: Some(resource_type.into()),
            source: None,
        }
    }

    /// Create an authentication error
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Self::Auth {
            message: message.into(),
            source: None,
        }
    }

    /// Create a user-friendly error with suggestion
    pub fn user_error<S: Into<String>>(message: S) -> Self {
        Self::UserError {
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create a user-friendly error with suggestion
    pub fn user_error_with_suggestion<S: Into<String>, T: Into<String>>(message: S, suggestion: T) -> Self {
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
            | Self::Network { source: s, .. }
            | Self::Git { source: s, .. }
            | Self::Template { source: s, .. }
            | Self::Extension { source: s, .. }
            | Self::Resource { source: s, .. }
            | Self::Auth { source: s, .. }
            | Self::Other { source: s, .. } => {
                *s = Some(Box::new(source));
            }
            _ => {}
        }
        self
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Network { .. }
                | Self::Io(_)
                | Self::Task { .. }
                | Self::Resource { .. }
        )
    }

    /// Check if this error should be retried
    pub fn should_retry(&self) -> bool {
        matches!(self, Self::Network { .. } | Self::Resource { .. })
    }

    /// Get error severity level
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Auth { .. } | Self::Configuration { .. } => ErrorSeverity::Critical,
            Self::Agent { .. } | Self::Session { .. } | Self::Extension { .. } => ErrorSeverity::High,
            Self::Task { .. } | Self::Git { .. } | Self::Template { .. } => ErrorSeverity::Medium,
            Self::Network { .. } | Self::Resource { .. } => ErrorSeverity::Low,
            Self::Io(_) | Self::SerdeJson(_) => ErrorSeverity::Medium,
            Self::UserError { .. } => ErrorSeverity::Info,
            Self::Other { .. } => ErrorSeverity::Medium,
        }
    }
}

/// Error severity levels for monitoring and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational - no action needed
    Info,
    /// Low severity - monitoring recommended
    Low,
    /// Medium severity - investigation needed
    Medium,
    /// High severity - immediate attention required
    High,
    /// Critical severity - system failure
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