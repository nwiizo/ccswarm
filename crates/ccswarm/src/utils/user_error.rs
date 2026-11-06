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

    pub fn api_key_missing(provider: &str) -> UserError {
        UserError::new(
            format!("API key missing for {}", provider),
            format!("The {} API key is required but not found", provider),
        )
        .with_suggestion(format!(
            "Set the environment variable for {} API key",
            provider
        ))
    }
}

/// Show progress message to user
pub fn show_progress(message: &str) {
    print!("⏳ {}... ", message.cyan());
    use std::io::{self, Write};
    let _ = io::stdout().flush();
}
