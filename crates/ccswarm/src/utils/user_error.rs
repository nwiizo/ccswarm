use anyhow::{Context, Result};
use colored::Colorize;

use super::error_diagrams::{show_diagram, ErrorDiagrams};
use super::error_recovery::ErrorResolver;

/// User-friendly error wrapper that provides helpful context and solutions
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

        // Show visual diagram if available
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

    /// Display error and attempt auto-fix if requested
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

/// Common error patterns with helpful messages
pub struct CommonErrors;

impl CommonErrors {
    pub fn api_key_missing(provider: &str) -> UserError {
        UserError::new(format!("{} API key not found", provider))
            .with_details("The AI provider requires an API key to function")
            .suggest("Set the environment variable: export ANTHROPIC_API_KEY=your-key".to_string())
            .suggest("Add it to your .env file for persistence")
            .suggest("Visit https://console.anthropic.com to get your API key".to_string())
            .with_code("ENV001")
            .with_diagram(ErrorDiagrams::api_key_error())
    }

    pub fn session_not_found(session_id: &str) -> UserError {
        UserError::new("Session not found")
            .with_details(format!("No active session with ID: {}", session_id))
            .suggest("List all sessions: ccswarm session list")
            .suggest("Create a new session: ccswarm session create")
            .suggest("Check if the session was terminated")
            .with_code("SES001")
            .with_diagram(ErrorDiagrams::session_error())
            .auto_fixable()
    }

    pub fn agent_busy(agent_name: &str) -> UserError {
        UserError::new(format!("{} agent is busy", agent_name))
            .with_details("The agent is currently processing another task")
            .suggest("Wait for the current task to complete")
            .suggest(format!(
                "Check agent status: ccswarm agent status {}",
                agent_name
            ))
            .suggest("Use --force to interrupt (not recommended)")
            .with_code("AGT001")
            .with_diagram(ErrorDiagrams::agent_error())
    }

    pub fn config_not_found() -> UserError {
        UserError::new("Configuration file not found")
            .with_details("ccswarm.json is required to run")
            .suggest("Run setup wizard: ccswarm setup")
            .suggest("Create manually: ccswarm init --name MyProject")
            .suggest("Copy from example: cp examples/ccswarm.json .")
            .with_code("CFG001")
            .with_diagram(ErrorDiagrams::config_error())
            .auto_fixable()
    }

    pub fn git_not_initialized() -> UserError {
        UserError::new("Not a git repository")
            .with_details("ccswarm requires a git repository for agent isolation")
            .suggest("Initialize git: git init")
            .suggest("Clone existing repo: git clone <url>")
            .suggest("Use --no-git flag to skip (limited functionality)")
            .with_code("GIT001")
            .with_diagram(ErrorDiagrams::git_worktree_error())
            .auto_fixable()
    }

    pub fn permission_denied(path: &str) -> UserError {
        UserError::new("Permission denied")
            .with_details(format!("Cannot access: {}", path))
            .suggest("Check file permissions: ls -la")
            .suggest("Run with appropriate permissions")
            .suggest(format!("Change ownership: sudo chown $USER {}", path))
            .with_code("PRM001")
            .with_diagram(ErrorDiagrams::permission_error())
            .auto_fixable()
    }

    pub fn network_error(url: &str) -> UserError {
        UserError::new("Network connection failed")
            .with_details(format!("Cannot reach: {}", url))
            .suggest("Check your internet connection")
            .suggest("Verify the URL is correct")
            .suggest("Check if you're behind a proxy")
            .suggest("Try again in a few moments")
            .with_code("NET001")
            .with_diagram(ErrorDiagrams::network_error())
    }

    pub fn invalid_task_format() -> UserError {
        UserError::new("Invalid task format")
            .with_details("Task description must be clear and actionable")
            .suggest("Use imperative mood: 'Create user authentication'")
            .suggest("Add priority: --priority high")
            .suggest("Specify type: --type feature")
            .suggest("See examples: ccswarm task --examples")
            .with_code("TSK001")
            .with_diagram(ErrorDiagrams::task_error())
    }

    pub fn ai_response_error() -> UserError {
        UserError::new("AI response error")
            .with_details("The AI provider returned an unexpected response")
            .suggest("Check your API quota and limits")
            .suggest("Verify API key permissions")
            .suggest("Try a simpler request")
            .suggest("Check provider status page")
            .with_code("AI001")
    }

    pub fn worktree_conflict(branch: &str) -> UserError {
        UserError::new("Git worktree conflict")
            .with_details(format!("Branch '{}' is already checked out", branch))
            .suggest("List worktrees: git worktree list")
            .suggest("Remove unused worktree: git worktree remove <path>")
            .suggest("Use a different branch name")
            .suggest("Clean up with: ccswarm cleanup")
            .with_code("WRK001")
            .with_diagram(ErrorDiagrams::git_worktree_error())
            .auto_fixable()
    }
}

/// Extension trait to convert any error to user-friendly format
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

/// Helper to show progress with context
pub fn show_progress(message: &str) {
    println!("{} {}", "⏳".bright_yellow(), message.dimmed());
}
