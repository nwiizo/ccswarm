use anyhow::{Context, Result};
use colored::Colorize;

use super::error_diagrams::show_diagram;
use super::error_recovery::ErrorResolver;
use super::user_error_macros::{ErrorCategory, ErrorFactory};

/// User-friendly error wrapper - now using factory pattern to eliminate duplication
/// Reduced from 272 lines to ~120 lines
pub struct UserError {
    pub title: String,
    pub details: String,
    pub suggestions: Vec<String>,
    pub error_code: Option<String>,
    pub source: Option<anyhow::Error>,
    pub diagram: Option<String>,
    pub can_auto_fix: bool,
}

impl UserError {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            details: String::new(),
            suggestions: Vec::new(),
            error_code: None,
            source: None,
            diagram: None,
            can_auto_fix: false,
        }
    }

    // Builder methods
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = details.into();
        self
    }

    pub fn suggest(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = Some(code.into());
        self
    }

    pub fn caused_by(mut self, error: anyhow::Error) -> Self {
        self.source = Some(error);
        self
    }

    pub fn with_diagram(mut self, diagram: String) -> Self {
        self.diagram = Some(diagram);
        self
    }

    pub fn auto_fixable(mut self) -> Self {
        self.can_auto_fix = true;
        self
    }

    pub fn display(&self) {
        eprintln!();
        eprintln!("{} {}", "❌".red(), self.title.bright_red().bold());

        if !self.details.is_empty() {
            eprintln!();
            eprintln!("   {}", self.details.white());
        }

        if let Some(diagram) = &self.diagram {
            show_diagram(diagram.clone());
        }

        if !self.suggestions.is_empty() {
            eprintln!();
            eprintln!("   {}", "💡 Try this:".bright_yellow());
            for (i, suggestion) in self.suggestions.iter().enumerate() {
                eprintln!("   {}. {}", i + 1, suggestion.bright_white());
            }
        }

        if self.can_auto_fix {
            eprintln!();
            eprintln!(
                "   {} {}",
                "🔧".bright_blue(),
                "Auto-fix available! Run with --fix flag".bright_blue()
            );
        }

        if let Some(code) = &self.error_code {
            eprintln!();
            eprintln!("   {} {}", "Error code:".dimmed(), code.dimmed());
            eprintln!(
                "   {} ccswarm doctor --error {}",
                "More info:".dimmed(),
                code.dimmed()
            );
        }

        if let Some(source) = &self.source {
            if std::env::var("RUST_LOG").is_ok() {
                eprintln!();
                eprintln!("   {}", "Technical details:".dimmed());
                eprintln!("   {}", format!("{:?}", source).dimmed());
            }
        }

        eprintln!();
    }

    pub async fn display_and_fix(&self, auto_fix: bool) -> Result<()> {
        self.display();
        if auto_fix && self.can_auto_fix {
            if let Some(code) = &self.error_code {
                let resolver = ErrorResolver::new();
                resolver.resolve_interactive(code).await?;
            }
        }
        Ok(())
    }
}

/// Common errors - now using factory pattern (60+ methods reduced to 10)
pub struct CommonErrors;

impl CommonErrors {
    // Define all error methods using the factory pattern
    pub fn api_key_missing(provider: &str) -> UserError {
        ErrorFactory::create(
            ErrorCategory::Environment,
            format!("{} API key not found", provider),
            "The AI provider requires an API key to function",
            vec![
                format!(
                    "Set the environment variable: export {}_API_KEY=your-key",
                    provider.to_uppercase()
                ),
                "Add it to your .env file for persistence".to_string(),
                format!("Visit the {} console to get your API key", provider),
            ],
            1,
        )
    }

    pub fn session_not_found(session_id: &str) -> UserError {
        ErrorFactory::create(
            ErrorCategory::Session,
            "Session not found",
            format!("No active session with ID: {}", session_id),
            vec![
                "List all sessions: ccswarm session list".to_string(),
                "Create a new session: ccswarm session create".to_string(),
                "Check if the session was terminated".to_string(),
            ],
            1,
        )
    }

    pub fn agent_busy(agent_name: &str) -> UserError {
        ErrorFactory::create(
            ErrorCategory::Agent,
            format!("{} agent is busy", agent_name),
            "The agent is currently processing another task",
            vec![
                "Wait for the current task to complete".to_string(),
                format!("Check agent status: ccswarm agent status {}", agent_name),
                "Use --force to interrupt (not recommended)".to_string(),
            ],
            1,
        )
    }

    pub fn config_not_found() -> UserError {
        ErrorFactory::create(
            ErrorCategory::Configuration,
            "Configuration file not found",
            "ccswarm.json is required to run",
            vec![
                "Run setup wizard: ccswarm setup".to_string(),
                "Create manually: ccswarm init --name MyProject".to_string(),
                "Copy from example: cp examples/ccswarm.json .".to_string(),
            ],
            1,
        )
    }

    pub fn git_not_initialized() -> UserError {
        ErrorFactory::create(
            ErrorCategory::Git,
            "Not a git repository",
            "ccswarm requires a git repository for agent isolation",
            vec![
                "Initialize git: git init".to_string(),
                "Clone existing repo: git clone <url>".to_string(),
                "Use --no-git flag to skip (limited functionality)".to_string(),
            ],
            1,
        )
    }

    pub fn permission_denied(path: &str) -> UserError {
        ErrorFactory::create(
            ErrorCategory::Permission,
            "Permission denied",
            format!("Cannot access: {}", path),
            vec![
                "Check file permissions: ls -la".to_string(),
                "Run with appropriate permissions".to_string(),
                format!("Change ownership: sudo chown $USER {}", path),
            ],
            1,
        )
    }

    pub fn network_error(url: &str) -> UserError {
        ErrorFactory::create(
            ErrorCategory::Network,
            "Network connection failed",
            format!("Cannot reach: {}", url),
            vec![
                "Check your internet connection".to_string(),
                "Verify the URL is correct".to_string(),
                "Check if you're behind a proxy".to_string(),
                "Try again in a few moments".to_string(),
            ],
            1,
        )
    }

    pub fn invalid_task_format() -> UserError {
        ErrorFactory::create(
            ErrorCategory::Task,
            "Invalid task format",
            "Task description must be clear and actionable",
            vec![
                "Use imperative mood: 'Create user authentication'".to_string(),
                "Add priority: --priority high".to_string(),
                "Specify type: --type feature".to_string(),
                "See examples: ccswarm task --examples".to_string(),
            ],
            1,
        )
    }

    pub fn ai_response_error() -> UserError {
        ErrorFactory::create(
            ErrorCategory::AI,
            "AI response error",
            "The AI provider returned an unexpected response",
            vec![
                "Check your API quota and limits".to_string(),
                "Verify API key permissions".to_string(),
                "Try a simpler request".to_string(),
                "Check provider status page".to_string(),
            ],
            1,
        )
    }

    pub fn worktree_conflict(branch: &str) -> UserError {
        ErrorFactory::create(
            ErrorCategory::Worktree,
            "Git worktree conflict",
            format!("Branch '{}' is already checked out", branch),
            vec![
                "List worktrees: git worktree list".to_string(),
                "Remove unused worktree: git worktree remove <path>".to_string(),
                "Use a different branch name".to_string(),
                "Clean up with: ccswarm cleanup".to_string(),
            ],
            1,
        )
    }
}

/// Extension trait remains the same
pub trait UserErrorExt<T> {
    fn user_context(self, title: &str) -> Result<T>;
    fn with_suggestion(self, suggestion: &str) -> Result<T>;
}

impl<T> UserErrorExt<T> for Result<T> {
    fn user_context(self, title: &str) -> Result<T> {
        self.context(title.to_string())
    }

    fn with_suggestion(self, suggestion: &str) -> Result<T> {
        match self {
            Ok(v) => Ok(v),
            Err(e) => {
                let user_err = UserError::new(e.to_string())
                    .suggest(suggestion)
                    .caused_by(e);
                user_err.display();
                std::process::exit(1);
            }
        }
    }
}

pub fn show_progress(message: &str) {
    println!("{} {}", "⏳".bright_yellow(), message.dimmed());
}
