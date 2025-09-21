use colored::Colorize;
use std::fmt;

/// User-friendly error messages
#[derive(Debug, Clone)]
pub struct UserError {
    pub title: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub details: Option<String>,
}

impl UserError {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            suggestion: None,
            details: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn display(&self) {
        eprintln!("\n{} {}", "✗".red().bold(), self.title.red().bold());
        eprintln!("  {}", self.message);

        if let Some(ref details) = self.details {
            eprintln!("\n  {}", "Details:".yellow());
            eprintln!("  {}", details);
        }

        if let Some(ref suggestion) = self.suggestion {
            eprintln!("\n  {} {}", "💡".yellow(), "Suggestion:".yellow().bold());
            eprintln!("  {}", suggestion.green());
        }
        eprintln!();
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.title, self.message)
    }
}

impl std::error::Error for UserError {}

/// Common error scenarios with helpful messages
pub struct CommonErrors;

impl CommonErrors {
    pub fn git_not_initialized() -> UserError {
        UserError::new(
            "Git repository not initialized",
            "ccswarm requires a Git repository to manage agent worktrees",
        )
        .with_suggestion("Run 'git init' in your project directory first")
    }

    pub fn config_not_found() -> UserError {
        UserError::new(
            "Configuration file not found",
            "ccswarm.json is required to run this command",
        )
        .with_suggestion("Run 'ccswarm init' to create a configuration file")
    }

    pub fn invalid_agent_type(agent_type: &str) -> UserError {
        UserError::new(
            format!("Invalid agent type: {}", agent_type),
            "Supported agent types are: frontend, backend, devops, qa",
        )
        .with_suggestion("Try 'ccswarm agents spawn --type frontend'".to_string())
    }

    pub fn session_not_found(session_id: &str) -> UserError {
        UserError::new(
            format!("Session not found: {}", session_id),
            "The specified session does not exist or has been terminated",
        )
        .with_suggestion("Run 'ccswarm session list' to see active sessions")
    }

    pub fn port_in_use(port: u16) -> UserError {
        UserError::new(
            format!("Port {} is already in use", port),
            "Another process is using this port",
        )
        .with_suggestion("Try a different port with --port option".to_string())
    }

    pub fn invalid_task_format() -> UserError {
        UserError::new(
            "Invalid task format",
            "The task format is not valid",
        )
        .with_suggestion("Use format: 'task description [priority] [type]'")
    }

    pub fn api_key_missing(provider: &str) -> UserError {
        UserError::new(
            format!("API key missing for {}", provider),
            format!("The {} API key is required but not found", provider),
        )
        .with_suggestion(format!("Set the environment variable for {} API key", provider))
    }
}

/// Show progress message to user
pub fn show_progress(message: &str) {
    print!("⏳ {}... ", message.cyan());
    use std::io::{self, Write};
    let _ = io::stdout().flush();
}

/// Extension trait for user errors
pub trait UserErrorExt {
    fn to_user_error(self) -> UserError;
}

impl<E: std::error::Error> UserErrorExt for E {
    fn to_user_error(self) -> UserError {
        UserError::new("Error", self.to_string())
    }
}