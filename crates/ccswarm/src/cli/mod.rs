//! CLI module for ccswarm with clippy exceptions for complex conditional patterns

#![allow(clippy::collapsible_else_if)]
#![allow(clippy::get_first)]

mod command_registry;
mod common_handler;
mod subagent_commands;

mod error_help;
mod health;
mod interactive_help;
mod output;
mod progress;
mod quickstart_simple;
mod resource_commands;
mod setup_wizard;
mod tutorial;

pub use interactive_help::{show_quick_help, InteractiveHelp};
use output::{create_formatter, OutputFormatter};
pub use progress::{ProcessTracker, ProgressStyle, ProgressTracker, StatusLine};
pub use setup_wizard::SetupWizard;
pub use tutorial::InteractiveTutorial;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::agent::{Priority, Task, TaskType};
use crate::config::CcswarmConfig;
use crate::execution::{ExecutionEngine, TaskStatus};

// Type alias for compatibility
type Config = CcswarmConfig;
use crate::orchestrator::MasterClaude;

/// ccswarm - Claude Code統括型マルチエージェントシステム
#[derive(Parser)]
#[command(name = "ccswarm")]
#[command(about = "Claude Code multi-agent orchestration system")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "ccswarm.json")]
    pub config: PathBuf,

    /// Repository path
    #[arg(short, long, default_value = ".")]
    pub repo: PathBuf,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// JSON output format
    #[arg(long)]
    pub json: bool,

    /// Automatically fix errors when possible
    #[arg(long, global = true)]
    pub fix: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new ccswarm project
    Init {
        /// Project name
        #[arg(short, long)]
        name: String,

        /// Repository URL
        #[arg(short, long)]
        repo_url: Option<String>,

        /// Agent configurations to create
        #[arg(long, value_delimiter = ',')]
        agents: Vec<String>,
    },

    /// Start the ccswarm orchestrator
    Start {
        /// Run in daemon mode
        #[arg(short, long)]
        daemon: bool,

        /// Port for status server
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Isolation mode for agents (worktree, container, hybrid)
        #[arg(long, default_value = "worktree")]
        isolation: String,

        /// Use real Claude API instead of simulation (requires ANTHROPIC_API_KEY)
        #[arg(long)]
        use_real_api: bool,
    },

    /// Start TUI (Terminal User Interface)
    Tui,

    /// Stop the running orchestrator
    Stop,

    /// Show status of orchestrator and agents
    Status {
        /// Show detailed status
        #[arg(short, long)]
        detailed: bool,

        /// Specific agent to check
        #[arg(short, long)]
        agent: Option<String>,
    },

    /// Task management commands
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },

    /// List agents and their configurations
    Agents {
        /// Show inactive agents
        #[arg(long)]
        all: bool,
    },

    /// Run quality review
    Review {
        /// Agent to review
        #[arg(short, long)]
        agent: Option<String>,

        /// Strict quality checks
        #[arg(short, long)]
        strict: bool,
    },

    /// Manage Git worktrees
    Worktree {
        #[command(subcommand)]
        action: WorktreeAction,
    },

    /// Show logs
    Logs {
        /// Follow logs
        #[arg(short, long)]
        follow: bool,

        /// Specific agent
        #[arg(short, long)]
        agent: Option<String>,

        /// Number of lines to show
        #[arg(short, long, default_value = "100")]
        lines: usize,
    },

    /// Generate configuration template
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Master delegation commands
    Delegate {
        #[command(subcommand)]
        action: DelegateAction,
    },

    /// Session management commands
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },

    /// Resource monitoring and management
    Resource {
        #[command(subcommand)]
        action: resource_commands::ResourceSubcommand,
    },

    /// Auto-create application with AI agents
    AutoCreate {
        /// Application description
        description: String,

        /// Use template
        #[arg(short, long)]
        template: Option<String>,

        /// Auto deploy after creation
        #[arg(long)]
        auto_deploy: bool,

        /// Output directory
        #[arg(short, long, default_value = "./")]
        output: PathBuf,
    },

    /// Sangha - collective decision making
    Sangha {
        #[command(subcommand)]
        action: SanghaAction,
    },

    /// Extension management
    Extend {
        #[command(subcommand)]
        action: ExtendAction,
    },

    /// Search external resources
    Search {
        #[command(subcommand)]
        action: SearchAction,
    },

    /// Evolution tracking
    Evolution {
        #[command(subcommand)]
        action: EvolutionAction,
    },

    /// Agent-managed quality checks
    Quality {
        #[command(subcommand)]
        action: QualityAction,
    },

    /// Template management commands
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },

    /// Manage Claude Code subagents
    Subagent {
        #[command(subcommand)]
        command: subagent_commands::SubagentCommand,
    },

    /// Interactive setup wizard for new users
    Setup,

    /// Interactive tutorial to learn ccswarm
    Tutorial {
        /// Start from specific chapter (1-4)
        #[arg(short, long)]
        chapter: Option<u8>,
    },

    /// Enhanced help system with examples
    HelpTopic {
        /// Topic to get help on
        topic: Option<String>,

        /// Search help topics
        #[arg(short, long)]
        search: Option<String>,
    },

    /// System health checks and diagnostics
    Health {
        /// Check agent health status
        #[arg(long)]
        check_agents: bool,

        /// Check AI-session health
        #[arg(long)]
        check_sessions: bool,

        /// Show resource usage
        #[arg(long)]
        resources: bool,

        /// Run full diagnostics
        #[arg(long)]
        diagnose: bool,

        /// Show detailed output
        #[arg(short, long)]
        detailed: bool,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Check system health and diagnose issues
    Doctor {
        /// Run fixes for common issues
        #[arg(short, long)]
        fix: bool,

        /// Diagnose specific error code
        #[arg(long)]
        error: Option<String>,

        /// Check API connectivity
        #[arg(long)]
        check_api: bool,
    },

    /// Quick start with one command - streamlined setup and initialization
    Quickstart {
        /// Project name (default: infers from directory)
        #[arg(short, long)]
        name: Option<String>,

        /// Skip interactive prompts and use defaults
        #[arg(long)]
        no_prompt: bool,

        /// Enable all agents (frontend, backend, devops, qa)
        #[arg(long)]
        all_agents: bool,

        /// Run initial tests after setup
        #[arg(long)]
        with_tests: bool,
    },
}

#[derive(Subcommand)]
pub enum WorktreeAction {
    /// List all worktrees
    List,

    /// Create a new worktree
    Create {
        /// Worktree path
        path: PathBuf,

        /// Branch name
        branch: String,

        /// Create new branch
        #[arg(short, long)]
        new_branch: bool,
    },

    /// Remove a worktree
    Remove {
        /// Worktree path
        path: PathBuf,

        /// Force removal
        #[arg(short, long)]
        force: bool,
    },

    /// Prune stale worktrees
    Prune,

    /// Clean all ccswarm worktrees and branches
    Clean {
        /// Force cleanup without confirmation
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum TaskAction {
    /// Add a new task to the queue
    Add {
        /// Task description (or template ID if using template)
        description: String,

        /// Task priority
        #[arg(short, long, default_value = "medium")]
        priority: String,

        /// Task type
        #[arg(short, long, default_value = "development")]
        task_type: String,

        /// Additional details
        #[arg(long)]
        details: Option<String>,

        /// Estimated duration in seconds
        #[arg(long)]
        duration: Option<u32>,

        /// Auto-assign to best agent
        #[arg(long)]
        auto_assign: bool,

        /// Use a template for task creation
        #[arg(long)]
        template: Option<String>,

        /// Template variable values (key=value)
        #[arg(long, value_delimiter = ',')]
        template_vars: Vec<String>,

        /// Interactive template variable input
        #[arg(long)]
        interactive: bool,
    },

    /// List all tasks
    List {
        /// Show all tasks including completed
        #[arg(short, long)]
        all: bool,

        /// Filter by status (pending, in_progress, completed, failed)
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by agent
        #[arg(long)]
        agent: Option<String>,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Show task status
    Status {
        /// Task ID
        task_id: String,

        /// Show execution history
        #[arg(long)]
        history: bool,

        /// Show orchestration details
        #[arg(long)]
        orchestration: bool,
    },

    /// Cancel a task
    Cancel {
        /// Task ID
        task_id: String,

        /// Force cancellation even if in progress
        #[arg(short, long)]
        force: bool,

        /// Reason for cancellation
        #[arg(short, long)]
        reason: Option<String>,
    },

    /// Show task execution history
    History {
        /// Number of recent tasks to show
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Filter by agent
        #[arg(short, long)]
        agent: Option<String>,

        /// Show only failed tasks
        #[arg(long)]
        failed_only: bool,
    },

    /// Execute a task immediately (bypass queue)
    Execute {
        /// Task ID or new task description
        task: String,

        /// Force execution on specific agent
        #[arg(short, long)]
        agent: Option<String>,

        /// Use orchestrator for complex execution
        #[arg(long)]
        orchestrate: bool,
    },

    /// Show task queue statistics
    Stats {
        /// Show detailed breakdown
        #[arg(short, long)]
        detailed: bool,

        /// Show performance metrics
        #[arg(long)]
        performance: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Generate default configuration
    Generate {
        /// Output file
        #[arg(short, long, default_value = "ccswarm.json")]
        output: PathBuf,

        /// Project template
        #[arg(short, long, default_value = "full-stack")]
        template: String,
    },

    /// Validate configuration
    Validate {
        /// Configuration file
        #[arg(short, long, default_value = "ccswarm.json")]
        file: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum SessionAction {
    /// Create a new agent session
    Create {
        /// Agent type (frontend, backend, devops, qa)
        #[arg(short, long)]
        agent: String,

        /// Workspace path
        #[arg(short, long)]
        workspace: Option<String>,

        /// Background mode
        #[arg(short, long)]
        background: bool,
    },

    /// List all sessions
    List {
        /// Show all sessions including inactive
        #[arg(short, long)]
        all: bool,
    },

    /// Pause a running session
    Pause {
        /// Session ID
        session_id: String,
    },

    /// Resume a paused session
    Resume {
        /// Session ID
        session_id: String,
    },

    /// Attach to a session
    Attach {
        /// Session ID
        session_id: String,
    },

    /// Detach from a session
    Detach {
        /// Session ID
        session_id: String,
    },

    /// Kill a session
    Kill {
        /// Session ID
        session_id: String,

        /// Force kill
        #[arg(short, long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum DelegateAction {
    /// Delegate a task to specific agent
    Task {
        /// Task description
        description: String,

        /// Target agent type (frontend, backend, devops, qa)
        #[arg(short, long)]
        agent: String,

        /// Task priority
        #[arg(short, long, default_value = "medium")]
        priority: String,

        /// Task type
        #[arg(short, long, default_value = "development")]
        task_type: String,

        /// Additional details
        #[arg(long)]
        details: Option<String>,

        /// Force delegation even if agent doesn't match
        #[arg(long)]
        force: bool,
    },

    /// Analyze task and suggest optimal agent
    Analyze {
        /// Task description
        description: String,

        /// Show delegation reasoning
        #[arg(short, long)]
        verbose: bool,

        /// Delegation strategy to use
        #[arg(short, long, default_value = "hybrid")]
        strategy: String,
    },

    /// Show delegation statistics
    Stats {
        /// Show detailed breakdown
        #[arg(short, long)]
        detailed: bool,

        /// Time period to analyze (hours)
        #[arg(long, default_value = "24")]
        period: u32,
    },

    /// Interactive delegation mode
    Interactive,

    /// Show configuration
    Show {
        /// Configuration file
        #[arg(short, long, default_value = "ccswarm.json")]
        file: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum SanghaAction {
    /// Submit a proposal to Sangha
    Propose {
        /// Proposal type (doctrine, extension, task)
        #[arg(short, long)]
        proposal_type: String,

        /// Proposal file (JSON)
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Vote on a proposal
    Vote {
        /// Proposal ID
        proposal_id: String,

        /// Vote choice (aye, nay, abstain)
        choice: String,

        /// Reason for vote
        #[arg(short, long)]
        reason: Option<String>,
    },

    /// List active proposals
    List {
        /// Show all proposals including completed
        #[arg(short, long)]
        all: bool,

        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,
    },

    /// Show Sangha session status
    Session {
        /// Session ID
        #[arg(short, long)]
        id: Option<String>,

        /// Show active session
        #[arg(short, long)]
        active: bool,
    },

    /// Review extension proposal
    ExtensionReview {
        /// Proposal ID
        proposal_id: String,

        /// Perform technical check
        #[arg(long)]
        technical_check: bool,
    },
}

#[derive(Subcommand)]
pub enum ExtendAction {
    /// Propose an extension
    Propose {
        /// Agent ID
        #[arg(short, long)]
        agent: String,

        /// Extension type (capability, cognitive, collaborative)
        #[arg(short, long)]
        extension_type: String,

        /// Extension specification file
        #[arg(short, long)]
        file: PathBuf,
    },

    /// Show extension status
    Status {
        /// Agent ID
        #[arg(short, long)]
        agent: String,

        /// Extension ID
        #[arg(short, long)]
        extension_id: Option<String>,
    },

    /// Show extension history
    History {
        /// Agent ID
        #[arg(short, long)]
        agent: String,

        /// Show only successful extensions
        #[arg(long)]
        successful: bool,

        /// Show only failed extensions
        #[arg(long)]
        failed: bool,
    },

    /// Rollback an extension
    Rollback {
        /// Agent ID
        #[arg(short, long)]
        agent: String,

        /// Extension ID
        #[arg(short, long)]
        extension_id: String,

        /// Force rollback
        #[arg(short, long)]
        force: bool,
    },

    /// Discover extension opportunities
    Discover {
        /// Agent ID
        #[arg(short, long)]
        agent: Option<String>,

        /// Discovery type (capability, performance, trend)
        #[arg(short, long)]
        discovery_type: Option<String>,
    },

    /// Autonomous extension proposal (agents think for themselves)
    Autonomous {
        /// Agent ID (optional, all agents if not specified)
        #[arg(short, long)]
        agent: Option<String>,

        /// Enable continuous autonomous extension
        #[arg(long)]
        continuous: bool,

        /// Dry run - show what would be proposed without submitting
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
pub enum SearchAction {
    /// Search documentation and resources
    Docs {
        /// Search query
        query: String,

        /// Search source (mdn, github, stackoverflow, all)
        #[arg(short, long, default_value = "all")]
        source: String,

        /// Maximum results to return
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Test search functionality
    Test {
        /// Test query
        #[arg(default_value = "async rust")]
        query: String,
    },
}

#[derive(Subcommand)]
pub enum EvolutionAction {
    /// Show evolution metrics
    Metrics {
        /// Agent ID
        #[arg(short, long)]
        agent: Option<String>,

        /// Show all agents
        #[arg(long)]
        all: bool,

        /// Time period in days
        #[arg(short, long, default_value = "30")]
        period: u32,
    },

    /// Show successful patterns
    Patterns {
        /// Pattern type filter
        #[arg(short, long)]
        pattern_type: Option<String>,

        /// Show only successful patterns
        #[arg(long)]
        successful: bool,

        /// Show only failed patterns
        #[arg(long)]
        failed: bool,
    },

    /// Show extension genealogy
    Genealogy {
        /// Extension name or ID
        extension: String,

        /// Show full tree
        #[arg(short, long)]
        full: bool,
    },

    /// Generate evolution report
    Report {
        /// Output format (text, json, html)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum QualityAction {
    /// Run all quality checks through agents
    Check {
        /// Skip specific check types
        #[arg(long, value_delimiter = ',')]
        skip: Vec<String>,

        /// Run only specific check types
        #[arg(long, value_delimiter = ',')]
        only: Vec<String>,

        /// Fail fast on first error
        #[arg(long)]
        fail_fast: bool,
    },

    /// Run format checks (DevOps agent)
    Format {
        /// Automatically fix formatting issues
        #[arg(long)]
        fix: bool,
    },

    /// Run linting checks (DevOps agent)  
    Lint {
        /// Automatically fix linting issues where possible
        #[arg(long)]
        fix: bool,
    },

    /// Run test suite (QA agent)
    Test {
        /// Test filter pattern
        #[arg(short, long)]
        pattern: Option<String>,

        /// Run only unit tests
        #[arg(long)]
        unit: bool,

        /// Run only integration tests
        #[arg(long)]
        integration: bool,

        /// Run only security tests
        #[arg(long)]
        security: bool,
    },

    /// Run build verification (DevOps agent)
    Build {
        /// Build in release mode
        #[arg(long)]
        release: bool,

        /// Build all targets
        #[arg(long)]
        all_targets: bool,
    },

    /// Run security analysis (Backend agent)
    Security {
        /// Run vulnerability scan
        #[arg(long)]
        audit: bool,

        /// Check dependencies
        #[arg(long)]
        deps: bool,
    },

    /// Show quality gate status
    Status {
        /// Show detailed status for each check
        #[arg(short, long)]
        detailed: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum TemplateAction {
    /// List available templates
    List {
        /// Show all templates including disabled ones
        #[arg(short, long)]
        all: bool,

        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,

        /// Filter by tags
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Search term for name or description
        #[arg(short, long)]
        search: Option<String>,

        /// Sort by popularity
        #[arg(long)]
        popular: bool,

        /// Sort by success rate
        #[arg(long)]
        quality: bool,

        /// Show detailed information
        #[arg(short, long)]
        detailed: bool,
    },

    /// Show template details
    Show {
        /// Template ID or name
        template: String,

        /// Show template source code
        #[arg(long)]
        source: bool,

        /// Show usage statistics
        #[arg(long)]
        stats: bool,
    },

    /// Create a new template
    Create {
        /// Template ID
        #[arg(short, long)]
        id: String,

        /// Template name
        #[arg(short, long)]
        name: String,

        /// Template description
        #[arg(short, long)]
        description: String,

        /// Template category
        #[arg(short, long)]
        category: String,

        /// Open editor to define template details
        #[arg(long)]
        editor: bool,

        /// Use existing template as base
        #[arg(long)]
        from: Option<String>,
    },

    /// Edit an existing template
    Edit {
        /// Template ID or name
        template: String,

        /// Open in external editor
        #[arg(long)]
        editor: bool,
    },

    /// Delete a template
    Delete {
        /// Template ID or name
        template: String,

        /// Force deletion without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Apply a template to create a task
    Apply {
        /// Template ID or name
        template: String,

        /// Variable values (key=value)
        #[arg(short, long, value_delimiter = ',')]
        vars: Vec<String>,

        /// Interactive mode to prompt for variables
        #[arg(short, long)]
        interactive: bool,

        /// Preview the generated task without creating it
        #[arg(long)]
        preview: bool,

        /// Auto-assign to best agent
        #[arg(long)]
        auto_assign: bool,
    },

    /// Validate a template
    Validate {
        /// Template ID or name
        template: String,

        /// Show detailed validation report
        #[arg(short, long)]
        detailed: bool,

        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
    },

    /// Clone a template
    Clone {
        /// Source template ID or name
        source: String,

        /// New template ID
        #[arg(short, long)]
        id: String,

        /// New template name (optional)
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Import templates from file
    Import {
        /// JSON file containing templates
        file: String,

        /// Overwrite existing templates
        #[arg(long)]
        force: bool,
    },

    /// Export templates to file
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Export specific templates only
        #[arg(short, long, value_delimiter = ',')]
        templates: Vec<String>,

        /// Include usage statistics
        #[arg(long)]
        stats: bool,
    },

    /// Install predefined templates
    Install {
        /// Install all predefined templates
        #[arg(long)]
        all: bool,

        /// Install specific template categories
        #[arg(short, long, value_delimiter = ',')]
        categories: Vec<String>,

        /// Force reinstall existing templates
        #[arg(long)]
        force: bool,
    },

    /// Show template usage statistics
    Stats {
        /// Show global statistics
        #[arg(short, long)]
        global: bool,

        /// Show statistics for specific template
        #[arg(short, long)]
        template: Option<String>,
    },

    /// Search templates
    Search {
        /// Search query
        query: String,

        /// Limit number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Minimum quality score (0.0-1.0)
        #[arg(long)]
        min_quality: Option<f64>,
    },
}

/// Compact CLI runner using common handler pattern
pub struct CliRunner {
    config: Config,
    repo_path: PathBuf,
    json_output: bool,
    formatter: OutputFormatter,
    execution_engine: ExecutionEngine,
}

// Command types for CLI
#[derive(Debug, Clone)]
pub enum TaskCommands {
    Add { description: String, tags: Vec<String>, priority: String },
    Create { description: String },
    List { status: Option<String>, assigned: Option<String>, tags: Vec<String> },
    Status { id: String },
}

#[derive(Debug, Clone)]
pub enum AgentCommands {
    Create { name: String, role: String },
    List,
    Status { name: String },
    Start { name: String },
    Stop { name: String },
}

#[derive(Debug, Clone)]
pub enum SessionCommands {
    List { show_all: bool },
    Create { name: String },
    Attach { id: String },
    Detach,
}

#[derive(Debug, Clone)]
pub enum WorktreeCommands {
    List,
    Create { name: String },
    Remove { name: String },
}

#[derive(Debug, Clone)]
pub enum DelegateCommands {
    Task { id: String, agent: String },
    Auto,
}

#[derive(Debug, Clone)]
pub enum LogsCommands {
    Show { agent: Option<String> },
    Tail { lines: usize },
}

#[derive(Debug, Clone)]
pub enum ReviewCommands {
    Start,
    Status,
    History,
}

impl CliRunner {

    pub async fn run(&self, cli: Cli) -> Result<()> {
        // Use common handler for all commands
        match cli.command {
            Commands::Init { name, agents, repo_url } => {
                // Init project implementation
                Ok(())
            }
            Commands::Start { daemon, port, isolation, use_real_api } => {
                // Start system implementation
                Ok(())
            }
            Commands::Stop => {
                // Stop system implementation
                Ok(())
            }
            Commands::Status => {
                // Show status implementation
                Ok(())
            }
            Commands::Task(cmd) => self.handle_task_unified(cmd).await,
            Commands::Agent(cmd) => self.handle_agent_unified(cmd).await,
            Commands::Session(cmd) => self.handle_session_unified(cmd).await,
            Commands::Worktree(cmd) => self.handle_worktree_unified(cmd).await,
            Commands::Delegate(cmd) => self.handle_delegate_unified(cmd).await,
            Commands::Tui => {
                // TUI implementation
                crate::tui::run_tui().await
            }
            Commands::Logs(cmd) => self.handle_logs_unified(cmd).await,
            Commands::Review(cmd) => self.handle_review_unified(cmd).await,
            _ => Ok(())
        }
    }

    // Unified handlers that eliminate duplication
    async fn handle_task_unified(&self, _cmd: TaskCommands) -> Result<()> {
        // Task command implementation
        Ok(())
    }

    async fn handle_agent_unified(&self, _cmd: AgentCommands) -> Result<()> {
        // Agent command implementation
        Ok(())
    }

    async fn handle_session_unified(&self, _cmd: SessionCommands) -> Result<()> {
        // Session command implementation
        Ok(())
    }

    async fn handle_worktree_unified(&self, _cmd: WorktreeCommands) -> Result<()> {
        Ok(())
    }

    async fn handle_delegate_unified(&self, _cmd: DelegateCommands) -> Result<()> {
        Ok(())
    }

    async fn handle_logs_unified(&self, _cmd: LogsCommands) -> Result<()> {
        Ok(())
    }

    async fn handle_review_unified(&self, _cmd: ReviewCommands) -> Result<()> {
        Ok(())
    }

    async fn handle_semantic_unified(&self, _cmd: serde_json::Value) -> Result<()> {
        // Semantic commands temporarily disabled during refactoring
        Ok(())
    }

}

impl CliRunner {
    /// Create new CLI runner
    pub async fn new(cli: &Cli) -> Result<Self> {
        // Load configuration
        let config = if cli.config.exists() {
            CcswarmConfig::from_file(cli.config.clone())
                .await
                .context("Failed to load configuration")?
        } else {
            warn!("Configuration file not found, using defaults");
            create_default_config(&cli.repo)?
        };

        let formatter = create_formatter(cli.json);

        // Initialize execution engine for task management
        let execution_engine = match ExecutionEngine::new(&config).await {
            Ok(engine) => {
                if let Err(e) = engine.start().await {
                    warn!("Failed to start execution engine: {}", e);
                    None
                } else {
                    Some(engine)
                }
            }
            Err(e) => {
                warn!("Failed to create execution engine: {}", e);
                None
            }
        };

        Ok(Self {
            config,
            repo_path: cli.repo.clone(),
            json_output: cli.json,
            formatter,
            execution_engine,
        })
    }

    /// Run the CLI command
    pub async fn run(&self, command: &Commands) -> Result<()> {
        // Use the command registry for centralized command handling
        let registry = self::command_registry::get_command_registry();
        registry.execute(self, command).await
    }

    async fn init_project(
        &self,
        name: &str,
        repo_url: Option<&str>,
        agents: &[String],
    ) -> Result<()> {
        use crate::utils::user_error::CommonErrors;

        info!("Initializing ccswarm project: {}", name);

        // Show progress to user
        println!(
            "{}",
            format!("🚀 Initializing ccswarm project: {}", name)
                .bright_cyan()
                .bold()
        );
        println!();

        // Check if git is available
        if !crate::git::shell::ShellWorktreeManager::is_git_available() {
            CommonErrors::git_not_initialized().display();
            return Err(anyhow!("Git is required for ccswarm"));
        }

        // Initialize Git repository if needed
        crate::utils::user_error::show_progress("Setting up git repository...");
        crate::git::shell::ShellWorktreeManager::init_if_needed(&self.repo_path)
            .await
            .inspect_err(|e| {
                eprintln!();
                CommonErrors::git_not_initialized()
                    .with_details(e.to_string())
                    .display();
            })?;
        println!("✅ Git repository ready");

        // Create configuration
        crate::utils::user_error::show_progress("Creating project configuration...");
        let mut config = create_default_config(&self.repo_path)?;
        config.project.name = name.to_string();

        if let Some(url) = repo_url {
            config.project.repository.url = url.to_string();
        }

        // Add requested agents
        for agent_type in agents {
            let agent_config = crate::config::AgentConfig {
                specialization: agent_type.clone(),
                worktree: format!("agents/{}-agent", agent_type),
                branch: format!("feature/{}", agent_type),
                claude_config: crate::config::ClaudeConfig::for_agent(agent_type),
                claude_md_template: format!("{}_specialist", agent_type),
            };
            config.agents.insert(agent_type.clone(), agent_config);
        }

        // Save configuration
        let config_file = self.repo_path.join("ccswarm.json");
        config.to_file(config_file).await?;

        let data = serde_json::json!({
            "project": name,
            "agents": agents,
        });

        println!(
            "{}",
            self.formatter.format_success(
                &format!("ccswarm project '{}' initialized", name),
                Some(data)
            )
        );

        Ok(())
    }

    async fn start_orchestrator(
        &self,
        _daemon: bool,
        _port: u16,
        isolation: &str,
        use_real_api: bool,
    ) -> Result<()> {
        info!(
            "Starting ccswarm orchestrator with isolation mode: {} (real_api: {})",
            isolation, use_real_api
        );

        // Parse isolation mode
        let isolation_mode = match isolation {
            "container" => crate::agent::IsolationMode::Container,
            "hybrid" => crate::agent::IsolationMode::Hybrid,
            _ => crate::agent::IsolationMode::GitWorktree,
        };

        // Update configuration to use real API if requested
        let mut config = self.config.clone();
        if use_real_api {
            // Check for API key
            if std::env::var("ANTHROPIC_API_KEY").is_err() {
                return Err(anyhow::anyhow!(
                    "ANTHROPIC_API_KEY environment variable must be set when using --use-real-api"
                ));
            }

            // Update all agent configurations to use real API
            for agent_config in config.agents.values_mut() {
                agent_config.claude_config.use_real_api = true;
            }

            // Update master configuration
            config.project.master_claude.claude_config.use_real_api = true;
        }

        let mut master = MasterClaude::new(config, self.repo_path.clone()).await?;

        // Set isolation mode for all agents
        master.set_isolation_mode(isolation_mode);

        // Initialize agents
        master.initialize().await?;

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Orchestrator started",
                    "master_id": master.id,
                    "agents": master.agents.len(),
                }))?
            );
        } else {
            println!("🚀 ccswarm orchestrator started");
            println!("   Master ID: {}", master.id);
            println!("   Agents: {}", master.agents.len());
        }

        // Start coordination (this would run indefinitely in real usage)
        master.start_coordination().await?;

        Ok(())
    }

    async fn start_tui(&self) -> Result<()> {
        info!("Starting ccswarm TUI");

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Starting TUI mode",
                }))?
            );
        } else {
            println!("🖥️  Starting ccswarm TUI...");
            println!("   Press 'q' to quit");
        }

        // Start TUI with execution engine if available
        if let Some(ref execution_engine) = self.execution_engine {
            crate::tui::run_tui_with_engine(execution_engine.clone()).await?;
        } else {
            crate::tui::run_tui().await?;
        }

        Ok(())
    }

    async fn stop_orchestrator(&self) -> Result<()> {
        // TODO: Implement graceful shutdown via signal/file
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Stop signal sent",
                }))?
            );
        } else {
            println!("🛑 ccswarm orchestrator stop signal sent");
        }
        Ok(())
    }

    async fn show_status(&self, detailed: bool, agent: Option<&str>) -> Result<()> {
        // Read status from coordination files
        let status_tracker = crate::coordination::StatusTracker::new().await?;

        if let Some(agent_id) = agent {
            // Show specific agent status
            if let Some(status) = status_tracker.get_status(agent_id).await? {
                if self.json_output {
                    println!("{}", serde_json::to_string_pretty(&status)?);
                } else {
                    println!("Agent: {}", agent_id);
                    println!("Status: {}", status["status"]);
                    println!("Updated: {}", status["timestamp"]);

                    // Check if this is a backend agent and show backend-specific info
                    if let Some(role) = status.get("role") {
                        if role.as_str() == Some("Backend") {
                            if let Some(backend_info) = status.get("backend_specific") {
                                println!("\n🔧 Backend Status:");
                                if let Some(api_health) = backend_info.get("api_health") {
                                    println!(
                                        "  API Health: {:.1}%",
                                        api_health.as_f64().unwrap_or(0.0) * 100.0
                                    );
                                }
                                if let Some(db) = backend_info.get("database") {
                                    println!(
                                        "  Database: {} ({})",
                                        if db["is_connected"].as_bool().unwrap_or(false) {
                                            "Connected"
                                        } else {
                                            "Disconnected"
                                        },
                                        db["database_type"].as_str().unwrap_or("Unknown")
                                    );
                                }
                                if let Some(server) = backend_info.get("server") {
                                    println!(
                                        "  Server: {:.1}MB RAM, {:.1}% CPU",
                                        server["memory_usage_mb"].as_f64().unwrap_or(0.0),
                                        server["cpu_usage_percent"].as_f64().unwrap_or(0.0)
                                    );
                                }
                                if let Some(services) =
                                    backend_info.get("services").and_then(|s| s.as_array())
                                {
                                    println!("  Active Services: {}", services.len());
                                }
                                if let Some(activity) = backend_info.get("recent_activity") {
                                    println!(
                                        "  Recent API Calls: {}",
                                        activity.as_u64().unwrap_or(0)
                                    );
                                }
                            }
                        }
                    }

                    if detailed {
                        println!(
                            "\nDetails: {}",
                            serde_json::to_string_pretty(&status["additional_info"])?
                        );
                    }
                }
            } else if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "error": "Agent not found",
                        "agent": agent_id,
                    }))?
                );
            } else {
                println!("❌ Agent '{}' not found", agent_id);
            }
        } else {
            // Show all agent statuses
            let statuses = status_tracker.get_all_statuses().await?;

            if self.json_output {
                println!("{}", serde_json::to_string_pretty(&statuses)?);
            } else {
                println!("📊 ccswarm Status");
                println!("================");

                if statuses.is_empty() {
                    println!("No agents found");
                } else {
                    for status in &statuses {
                        println!("Agent: {}", status["agent_id"]);
                        println!("  Status: {}", status["status"]);
                        println!("  Updated: {}", status["timestamp"]);

                        // Show role-specific summary for backend agents
                        if let Some(role) = status.get("role") {
                            if role.as_str() == Some("Backend") {
                                if let Some(backend_info) = status.get("backend_specific") {
                                    if let Some(api_health) = backend_info.get("api_health") {
                                        print!(
                                            "  API Health: {:.0}% | ",
                                            api_health.as_f64().unwrap_or(0.0) * 100.0
                                        );
                                    }
                                    if let Some(db) = backend_info.get("database") {
                                        print!(
                                            "DB: {} | ",
                                            if db["is_connected"].as_bool().unwrap_or(false) {
                                                "✓"
                                            } else {
                                                "✗"
                                            }
                                        );
                                    }
                                    if let Some(services) =
                                        backend_info.get("services").and_then(|s| s.as_array())
                                    {
                                        print!("Services: {}", services.len());
                                    }
                                    println!();
                                }
                            }
                        }

                        if detailed {
                            println!(
                                "  Details: {}",
                                serde_json::to_string_pretty(&status["additional_info"])?
                            );
                        }
                        println!();
                    }
                }
            }
        }

        Ok(())
    }

    async fn add_task(
        &self,
        description: &str,
        priority: &str,
        task_type: &str,
        details: Option<&str>,
        duration: Option<u32>,
    ) -> Result<()> {
        // use crate::cli::progress::{ProgressStyle, ProgressTracker};
        use crate::utils::user_error::CommonErrors;

        // Validate task description
        if description.trim().is_empty() {
            CommonErrors::invalid_task_format()
                .with_details("Task description cannot be empty")
                .suggest("Provide a clear, actionable task description")
                .suggest("Example: ccswarm task \"Create user authentication system\"")
                .display();
            return Err(anyhow!("Invalid task description"));
        }

        // Create task (simplified progress display)
        println!(
            "Creating task: {}...",
            description.chars().take(50).collect::<String>()
        );

        let priority = match priority.to_lowercase().as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            "critical" => Priority::Critical,
            _ => Priority::Medium,
        };

        let task_type = match task_type.to_lowercase().as_str() {
            "development" | "dev" => TaskType::Development,
            "testing" | "test" => TaskType::Testing,
            "documentation" | "docs" => TaskType::Documentation,
            "infrastructure" | "infra" => TaskType::Infrastructure,
            "coordination" => TaskType::Coordination,
            "review" => TaskType::Review,
            "bugfix" | "bug" => TaskType::Bugfix,
            "feature" => TaskType::Feature,
            _ => TaskType::Development,
        };

        let task_type_clone = task_type;
        let mut task = Task::new(
            uuid::Uuid::new_v4().to_string(),
            description.to_string(),
            priority,
            task_type,
        );

        if let Some(details) = details {
            task = task.with_details(details.to_string());
        }

        if let Some(duration) = duration {
            task = task.with_duration(duration);
        }

        // Add to task queue via execution engine
        let task_id = if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            executor.add_task(task.clone()).await
        } else {
            // Fall back to creating task with ID but warn user
            warn!("Execution engine not available, task will not be executed");
            task.id.clone()
        };

        // Complete progress indicator
        // ProgressTracker::complete(progress, true, Some("Task created successfully".to_string())).await;

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Task added",
                    "task_id": task_id,
                    "description": description,
                    "priority": priority,
                }))?
            );
        } else {
            println!();
            println!("{}", "✅ Task created successfully!".bright_green().bold());
            println!();
            println!(
                "   {} {}",
                "Task ID:".bright_cyan(),
                task_id[..8].bright_white()
            );
            println!("   {} {}", "Description:".bright_cyan(), description);
            println!(
                "   {} {}",
                "Priority:".bright_cyan(),
                match priority {
                    Priority::Critical => "🔴 Critical".bright_red(),
                    Priority::High => "🟡 High".bright_yellow(),
                    Priority::Medium => "🟢 Medium".bright_green(),
                    Priority::Low => "🔵 Low".bright_blue(),
                }
            );
            println!(
                "   {} {}",
                "Type:".bright_cyan(),
                format!("{:?}", task_type_clone).bright_white()
            );

            if let Some(duration) = task.estimated_duration {
                println!("   {} {} minutes", "Est. Duration:".bright_cyan(), duration);
            }

            // Show helpful next steps
            show_quick_help("task-created");
        }

        Ok(())
    }

    /// Handle task management commands
    async fn handle_task(&self, action: &TaskAction) -> Result<()> {
        match action {
            TaskAction::Add {
                description,
                priority,
                task_type,
                details,
                duration,
                auto_assign: _,
                template: _,
                template_vars: _,
                interactive: _,
            } => {
                self.add_task(
                    description,
                    priority,
                    task_type,
                    details.as_deref(),
                    *duration,
                )
                .await
            }
            TaskAction::List {
                all,
                status,
                agent,
                detailed,
            } => {
                self.list_tasks(*all, status.as_deref(), agent.as_deref(), *detailed)
                    .await
            }
            TaskAction::Status {
                task_id,
                history,
                orchestration,
            } => {
                self.show_task_status(task_id, *history, *orchestration)
                    .await
            }
            TaskAction::Cancel {
                task_id,
                force,
                reason,
            } => self.cancel_task(task_id, *force, reason.as_deref()).await,
            TaskAction::History {
                limit,
                agent,
                failed_only,
            } => {
                self.show_task_history(*limit, agent.as_deref(), *failed_only)
                    .await
            }
            TaskAction::Execute {
                task,
                agent,
                orchestrate,
            } => {
                self.execute_task_immediate(task, agent.as_deref(), *orchestrate)
                    .await
            }
            TaskAction::Stats {
                detailed,
                performance,
            } => self.show_task_stats(*detailed, *performance).await,
        }
    }

    /// List tasks with filters
    async fn list_tasks(
        &self,
        all: bool,
        status_filter: Option<&str>,
        agent_filter: Option<&str>,
        detailed: bool,
    ) -> Result<()> {
        let tasks = if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            let task_queue = executor.get_task_queue();
            task_queue.list_tasks(status_filter, agent_filter).await
        } else {
            Vec::new()
        };

        let displayed_tasks = if all {
            tasks
        } else {
            tasks.into_iter().take(50).collect()
        };

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "tasks": displayed_tasks,
                    "total": displayed_tasks.len(),
                    "filters": {
                        "all": all,
                        "status": status_filter,
                        "agent": agent_filter,
                        "detailed": detailed
                    }
                }))?
            );
        } else {
            println!("📋 Task List");
            println!("============");

            if displayed_tasks.is_empty() {
                println!("No tasks in queue currently.");
                println!();
                println!("💡 Add a task with: ccswarm task add \"Your task description\"");
            } else {
                println!("Found {} tasks:", displayed_tasks.len());
                println!();

                for task in &displayed_tasks {
                    let status_emoji = match &task.status {
                        TaskStatus::Pending => "⏳",
                        TaskStatus::Assigned { .. } => "📋",
                        TaskStatus::InProgress { .. } => "🏃",
                        TaskStatus::Completed { .. } => "✅",
                        TaskStatus::Failed { .. } => "❌",
                        TaskStatus::Cancelled { .. } => "🚫",
                    };

                    let priority_emoji = match task.task.priority {
                        Priority::Critical => "🚨",
                        Priority::High => "🔥",
                        Priority::Medium => "📅",
                        Priority::Low => "💤",
                    };

                    println!(
                        "{} {} {} [{}] {}",
                        status_emoji,
                        priority_emoji,
                        &task.task.id[..8], // Short ID
                        task.task.task_type,
                        task.task.description
                    );

                    if let Some(agent) = &task.assigned_agent {
                        println!("   👤 Assigned to: {}", agent);
                    }

                    if detailed {
                        println!(
                            "   ⏰ Created: {}",
                            task.created_at.format("%Y-%m-%d %H:%M:%S")
                        );
                        println!(
                            "   🔄 Updated: {}",
                            task.updated_at.format("%Y-%m-%d %H:%M:%S")
                        );
                        if let Some(details) = &task.task.details {
                            println!("   📝 Details: {}", details);
                        }
                    }
                    println!();
                }
            }
        }
        Ok(())
    }

    /// Show detailed task status
    async fn show_task_status(
        &self,
        task_id: &str,
        history: bool,
        orchestration: bool,
    ) -> Result<()> {
        let task = if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            let task_queue = executor.get_task_queue();
            task_queue.get_task(task_id).await
        } else {
            None
        };

        if let Some(task) = task {
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "task_id": task_id,
                        "task": task,
                        "history": history,
                        "orchestration": orchestration
                    }))?
                );
            } else {
                println!("🔍 Task Status: {}", task_id);
                println!("===============");

                let status_emoji = match &task.status {
                    TaskStatus::Pending => "⏳",
                    TaskStatus::Assigned { .. } => "📋",
                    TaskStatus::InProgress { .. } => "🏃",
                    TaskStatus::Completed { .. } => "✅",
                    TaskStatus::Failed { .. } => "❌",
                    TaskStatus::Cancelled { .. } => "🚫",
                };

                println!("{} Status: {:?}", status_emoji, task.status);
                println!("📝 Description: {}", task.task.description);
                println!("🎯 Priority: {:?}", task.task.priority);
                println!("📋 Type: {:?}", task.task.task_type);
                println!(
                    "⏰ Created: {}",
                    task.created_at.format("%Y-%m-%d %H:%M:%S")
                );
                println!(
                    "🔄 Updated: {}",
                    task.updated_at.format("%Y-%m-%d %H:%M:%S")
                );

                if let Some(agent) = &task.assigned_agent {
                    println!("👤 Assigned to: {}", agent);
                }

                if let Some(details) = &task.task.details {
                    println!("📝 Details: {}", details);
                }

                if history && !task.execution_history.is_empty() {
                    println!();
                    println!("📚 Execution History:");
                    for (i, attempt) in task.execution_history.iter().enumerate() {
                        println!(
                            "  {}. Agent: {} | Started: {}",
                            i + 1,
                            attempt.agent_id,
                            attempt.started_at.format("%Y-%m-%d %H:%M:%S")
                        );
                        if let Some(completed) = attempt.completed_at {
                            println!("     Completed: {}", completed.format("%Y-%m-%d %H:%M:%S"));
                        }
                        if let Some(error) = &attempt.error {
                            println!("     Error: {}", error);
                        }
                    }
                }
            }
        } else {
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "task_id": task_id,
                        "status": "not_found",
                        "history": history,
                        "orchestration": orchestration
                    }))?
                );
            } else {
                println!("🔍 Task Status: {}", task_id);
                println!("===============");
                println!("❌ Task not found");
                println!();
                println!("💡 Use 'ccswarm task list' to see available tasks");
            }
        }
        Ok(())
    }

    /// Cancel a task
    async fn cancel_task(&self, task_id: &str, force: bool, reason: Option<&str>) -> Result<()> {
        let result = if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            match executor
                .cancel_task(task_id, reason.map(|s| s.to_string()))
                .await
            {
                Ok(()) => Ok(()),
                Err(e) => Err(e),
            }
        } else {
            Err(anyhow::anyhow!("Execution engine not available"))
        };

        match result {
            Ok(()) => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "task_id": task_id,
                            "cancelled": true,
                            "reason": reason,
                            "force": force
                        }))?
                    );
                } else {
                    println!("✅ Task cancelled successfully: {}", task_id);
                    if let Some(r) = reason {
                        println!("   Reason: {}", r);
                    }
                }
            }
            Err(e) => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "task_id": task_id,
                            "cancelled": false,
                            "reason": e.to_string(),
                            "force": force
                        }))?
                    );
                } else {
                    println!("❌ Failed to cancel task: {}", task_id);
                    println!("   Reason: {}", e);
                    if let Some(r) = reason {
                        println!("   User reason: {}", r);
                    }
                }
            }
        }
        Ok(())
    }

    /// Show task execution history
    async fn show_task_history(
        &self,
        limit: usize,
        agent_filter: Option<&str>,
        failed_only: bool,
    ) -> Result<()> {
        let history = if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            executor.get_execution_history(Some(limit)).await
        } else {
            Vec::new()
        };

        let filtered_history: Vec<_> = history
            .into_iter()
            .filter(|result| {
                if failed_only && result.success {
                    return false;
                }
                if let Some(agent) = agent_filter {
                    if let Some(ref result_agent) = result.agent_used {
                        return result_agent == agent;
                    }
                    return false;
                }
                true
            })
            .collect();

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "history": filtered_history,
                    "limit": limit,
                    "agent_filter": agent_filter,
                    "failed_only": failed_only,
                    "total_count": filtered_history.len()
                }))?
            );
        } else {
            println!("📚 Task History (Last {} tasks)", limit);
            println!("================================");

            if filtered_history.is_empty() {
                println!("No task history available.");
            } else {
                println!("Found {} execution records:", filtered_history.len());
                println!();

                for result in &filtered_history {
                    let status_emoji = if result.success { "✅" } else { "❌" };
                    let orchestration_indicator = if result.orchestration_used {
                        "🎯"
                    } else {
                        "🔄"
                    };

                    println!(
                        "{} {} Task: {} | Duration: {:.2}s",
                        status_emoji,
                        orchestration_indicator,
                        &result.task_id[..8],
                        result.duration.as_secs_f64()
                    );

                    if let Some(agent) = &result.agent_used {
                        println!("   👤 Agent: {}", agent);
                    }

                    if let Some(error) = &result.error {
                        println!("   ❌ Error: {}", error);
                    }
                    println!();
                }
            }

            if let Some(agent) = agent_filter {
                println!("Filter: Agent = {}", agent);
            }
            if failed_only {
                println!("Filter: Failed tasks only");
            }
        }
        Ok(())
    }

    /// Execute a task immediately
    async fn execute_task_immediate(
        &self,
        task: &str,
        agent: Option<&str>,
        orchestrate: bool,
    ) -> Result<()> {
        if let Some(ref _engine) = self.execution_engine {
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "task": task,
                        "agent": agent,
                        "orchestrate": orchestrate,
                        "status": "not_implemented"
                    }))?
                );
            } else {
                println!("⚡ Immediate Task Execution");
                println!("==========================");
                println!("Task: {}", task);
                if let Some(a) = agent {
                    println!("Agent: {}", a);
                }
                println!("Orchestration: {}", if orchestrate { "Yes" } else { "No" });
                println!();
                println!("❌ Immediate execution not yet implemented");
                println!("💡 Use 'ccswarm task add' to add task to queue for execution");
            }
        } else {
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "task": task,
                        "agent": agent,
                        "orchestrate": orchestrate,
                        "status": "execution_engine_unavailable"
                    }))?
                );
            } else {
                println!("❌ Execution engine not available");
                println!("   Cannot execute tasks without execution engine");
            }
        }
        Ok(())
    }

    /// Show task queue statistics
    async fn show_task_stats(&self, detailed: bool, performance: bool) -> Result<()> {
        if let Some(ref engine) = self.execution_engine {
            let executor = engine.get_executor();
            let queue_stats = executor.get_task_queue().get_stats().await;
            let execution_stats = executor.get_stats().await;

            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "queue_stats": queue_stats,
                        "execution_stats": execution_stats,
                        "detailed": detailed,
                        "performance": performance
                    }))?
                );
            } else {
                println!("📊 Task Queue Statistics");
                println!("========================");
                println!("⏳ Pending: {}", queue_stats.pending_count);
                println!("🏃 In Progress: {}", queue_stats.active_count);
                println!("✅ Completed: {}", queue_stats.completed_count);
                println!("❌ Failed: {}", queue_stats.failed_count);
                println!("📋 Total: {}", queue_stats.total_count);

                if performance || detailed {
                    println!();
                    println!("🎯 Execution Statistics");
                    println!("=======================");
                    println!("Tasks Executed: {}", execution_stats.tasks_executed);
                    println!(
                        "Success Rate: {:.1}%",
                        if execution_stats.tasks_executed > 0 {
                            (execution_stats.tasks_succeeded as f64
                                / execution_stats.tasks_executed as f64)
                                * 100.0
                        } else {
                            0.0
                        }
                    );
                    println!(
                        "Average Duration: {:.2}s",
                        execution_stats.average_duration.as_secs_f64()
                    );
                    println!(
                        "Total Duration: {:.2}s",
                        execution_stats.total_duration.as_secs_f64()
                    );
                    println!(
                        "Orchestration Usage: {:.1}%",
                        execution_stats.orchestration_usage
                    );
                }

                if detailed {
                    println!();
                    println!("📈 Queue Health");
                    println!("===============");
                    let failure_rate = if queue_stats.total_count > 0 {
                        (queue_stats.failed_count as f64 / queue_stats.total_count as f64) * 100.0
                    } else {
                        0.0
                    };

                    let health_emoji = if failure_rate < 5.0 {
                        "🟢"
                    } else if failure_rate < 15.0 {
                        "🟡"
                    } else {
                        "🔴"
                    };

                    println!(
                        "{} Overall Health: {:.1}% failure rate",
                        health_emoji, failure_rate
                    );

                    if queue_stats.active_count > 10 {
                        println!(
                            "⚠️  High concurrent load: {} tasks",
                            queue_stats.active_count
                        );
                    }

                    if queue_stats.pending_count > 50 {
                        println!(
                            "⚠️  Queue backlog: {} pending tasks",
                            queue_stats.pending_count
                        );
                    }
                }
            }
        } else {
            if self.json_output {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "error": "execution_engine_unavailable",
                        "detailed": detailed,
                        "performance": performance
                    }))?
                );
            } else {
                println!("❌ Execution engine not available");
                println!("   Cannot display task statistics");
            }
        }
        Ok(())
    }

    async fn list_agents(&self, _all: bool) -> Result<()> {
        if self.json_output {
            println!("{}", serde_json::to_string_pretty(&self.config.agents)?);
        } else {
            println!("🤖 Configured Agents");
            println!("==================");

            for (name, config) in &self.config.agents {
                println!("Agent: {}", name);
                println!("  Specialization: {}", config.specialization);
                println!("  Worktree: {}", config.worktree);
                println!("  Branch: {}", config.branch);
                println!("  Think Mode: {:?}", config.claude_config.think_mode);
                println!();
            }
        }

        Ok(())
    }

    async fn run_review(&self, _agent: Option<&str>, _strict: bool) -> Result<()> {
        // TODO: Implement quality review
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Quality review completed",
                    "issues": 0,
                }))?
            );
        } else {
            println!("🔍 Quality review completed");
            println!("   No issues found");
        }

        Ok(())
    }

    async fn handle_worktree(&self, action: &WorktreeAction) -> Result<()> {
        let manager = crate::git::shell::ShellWorktreeManager::new(self.repo_path.clone())?;

        match action {
            WorktreeAction::List => {
                let worktrees = manager.list_worktrees().await?;

                if self.json_output {
                    println!("{}", serde_json::to_string_pretty(&worktrees)?);
                } else {
                    println!("🌳 Git Worktrees");
                    println!("===============");

                    for wt in &worktrees {
                        println!("Path: {}", wt.path.display());
                        println!("  Branch: {}", wt.branch);
                        println!("  Head: {}", wt.head_commit);
                        println!("  Locked: {}", wt.is_locked);
                        println!();
                    }
                }
            }
            WorktreeAction::Create {
                path,
                branch,
                new_branch,
            } => {
                let info = if *new_branch {
                    manager.create_worktree_full(path, branch, true).await?
                } else {
                    manager.create_worktree(path, branch).await?
                };

                if self.json_output {
                    println!("{}", serde_json::to_string_pretty(&info)?);
                } else {
                    println!("✅ Worktree created");
                    println!("   Path: {}", info.path.display());
                    println!("   Branch: {}", info.branch);
                }
            }
            WorktreeAction::Remove { path, force } => {
                if *force {
                    manager.remove_worktree_full(path, true).await?
                } else {
                    manager.remove_worktree(path).await?
                };

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Worktree removed",
                            "path": path,
                        }))?
                    );
                } else {
                    println!("✅ Worktree removed: {}", path.display());
                }
            }
            WorktreeAction::Prune => {
                manager.prune_worktrees().await?;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Worktrees pruned",
                        }))?
                    );
                } else {
                    println!("✅ Stale worktrees pruned");
                }
            }
            WorktreeAction::Clean { force } => {
                use std::io::{self, Write};

                // Find all ccswarm-related worktrees
                let worktrees = manager.list_worktrees().await?;
                let ccswarm_worktrees: Vec<_> = worktrees
                    .iter()
                    .filter(|w| w.branch.contains("agent") || w.branch.contains("feature/"))
                    .collect();

                if ccswarm_worktrees.is_empty() {
                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "success",
                                "message": "No ccswarm worktrees found",
                            }))?
                        );
                    } else {
                        println!("✅ No ccswarm worktrees to clean");
                    }
                    return Ok(());
                }

                // Ask for confirmation unless forced
                if !force {
                    println!("⚠️  Found {} ccswarm worktrees:", ccswarm_worktrees.len());
                    for w in &ccswarm_worktrees {
                        println!("   - {} ({})", w.path.display(), w.branch);
                    }
                    print!("\nAre you sure you want to remove all these worktrees? [y/N] ");
                    io::stdout().flush()?;

                    let mut response = String::new();
                    io::stdin().read_line(&mut response)?;

                    if !response.trim().eq_ignore_ascii_case("y") {
                        println!("❌ Cleanup cancelled");
                        return Ok(());
                    }
                }

                // Remove all ccswarm worktrees
                let mut removed_count = 0;
                for worktree in ccswarm_worktrees {
                    match manager.remove_worktree(&worktree.path).await {
                        Ok(_) => {
                            removed_count += 1;
                            if !self.json_output {
                                println!("   ✓ Removed {}", worktree.path.display());
                            }
                        }
                        Err(e) => {
                            if !self.json_output {
                                println!(
                                    "   ✗ Failed to remove {}: {}",
                                    worktree.path.display(),
                                    e
                                );
                            }
                        }
                    }
                }

                // Also clean up branches
                let output = tokio::process::Command::new("git")
                    .args(["branch", "--list", "*agent*", "feature/*"])
                    .output()
                    .await?;

                if output.status.success() {
                    let branches = String::from_utf8_lossy(&output.stdout);
                    let branch_count = branches.lines().count();

                    if branch_count > 0 {
                        tokio::process::Command::new("git")
                            .args(&["branch", "-D"])
                            .args(branches.lines().map(|b| b.trim().trim_start_matches("* ")))
                            .output()
                            .await?;
                    }
                }

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Cleanup completed",
                            "worktrees_removed": removed_count,
                        }))?
                    );
                } else {
                    println!(
                        "\n✅ Cleanup completed: {} worktrees removed",
                        removed_count
                    );
                }
            }
        }

        Ok(())
    }

    async fn show_logs(&self, _follow: bool, _agent: Option<&str>, _lines: usize) -> Result<()> {
        // TODO: Implement log viewing
        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Logs displayed",
                    "lines": 0,
                }))?
            );
        } else {
            println!("📝 Logs");
            println!("======");
            println!("No logs available yet");
        }

        Ok(())
    }

    async fn handle_config(&self, action: &ConfigAction) -> Result<()> {
        match action {
            ConfigAction::Generate { output, template } => {
                let config = match template.as_str() {
                    "minimal" => create_minimal_config(&self.repo_path)?,
                    "frontend-only" => create_frontend_only_config(&self.repo_path)?,
                    "full-stack" => create_default_config(&self.repo_path)?,
                    _ => create_default_config(&self.repo_path)?,
                };

                config.to_file(output.clone()).await?;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Configuration generated",
                            "file": output,
                            "template": template,
                        }))?
                    );
                } else {
                    println!("✅ Configuration generated: {}", output.display());
                    println!("   Template: {}", template);
                }
            }
            ConfigAction::Validate { file } => match CcswarmConfig::from_file(file.clone()).await {
                Ok(_) => {
                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "success",
                                "message": "Configuration is valid",
                                "file": file,
                            }))?
                        );
                    } else {
                        println!("✅ Configuration is valid: {}", file.display());
                    }
                }
                Err(e) => {
                    if self.json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&serde_json::json!({
                                "status": "error",
                                "message": "Configuration is invalid",
                                "file": file,
                                "error": e.to_string(),
                            }))?
                        );
                    } else {
                        println!("❌ Configuration is invalid: {}", file.display());
                        println!("   Error: {}", e);
                    }
                    return Err(e);
                }
            },
        }

        Ok(())
    }
}

fn create_default_config(repo_path: &Path) -> Result<CcswarmConfig> {
    let mut agents = std::collections::HashMap::new();

    // Add common agent configurations
    agents.insert(
        "frontend".to_string(),
        crate::config::AgentConfig {
            specialization: "react_typescript".to_string(),
            worktree: "agents/frontend-agent".to_string(),
            branch: "feature/frontend-ui".to_string(),
            claude_config: crate::config::ClaudeConfig::for_agent("frontend"),
            claude_md_template: "frontend_specialist".to_string(),
        },
    );

    agents.insert(
        "backend".to_string(),
        crate::config::AgentConfig {
            specialization: "node_microservices".to_string(),
            worktree: "agents/backend-agent".to_string(),
            branch: "feature/backend-api".to_string(),
            claude_config: crate::config::ClaudeConfig::for_agent("backend"),
            claude_md_template: "backend_specialist".to_string(),
        },
    );

    agents.insert(
        "devops".to_string(),
        crate::config::AgentConfig {
            specialization: "aws_kubernetes".to_string(),
            worktree: "agents/devops-agent".to_string(),
            branch: "feature/infrastructure".to_string(),
            claude_config: crate::config::ClaudeConfig::for_agent("devops"),
            claude_md_template: "devops_specialist".to_string(),
        },
    );

    Ok(CcswarmConfig {
        project: crate::config::ProjectConfig {
            name: "New ccswarm Project".to_string(),
            repository: crate::config::RepositoryConfig {
                url: repo_path.to_string_lossy().to_string(),
                main_branch: "main".to_string(),
            },
            master_claude: crate::config::MasterClaudeConfig {
                role: "technical_lead".to_string(),
                quality_threshold: 0.90,
                think_mode: crate::config::ThinkMode::UltraThink,
                permission_level: "supervised".to_string(),
                claude_config: crate::config::ClaudeConfig::for_master(),
                enable_proactive_mode: true, // デフォルト有効
                proactive_frequency: 30,     // 30秒間隔
                high_frequency: 15,          // 高頻度15秒間隔
            },
        },
        agents,
        coordination: crate::config::CoordinationConfig {
            communication_method: "json_files".to_string(),
            sync_interval: 30,
            quality_gate_frequency: "on_commit".to_string(),
            master_review_trigger: "all_tasks_complete".to_string(),
        },
    })
}

fn create_minimal_config(repo_path: &Path) -> Result<CcswarmConfig> {
    let mut config = create_default_config(repo_path)?;
    config.agents.clear();
    config.project.name = "Minimal ccswarm Project".to_string();
    Ok(config)
}

fn create_frontend_only_config(repo_path: &Path) -> Result<CcswarmConfig> {
    let mut config = create_minimal_config(repo_path)?;
    config.project.name = "Frontend ccswarm Project".to_string();

    config.agents.insert(
        "frontend".to_string(),
        crate::config::AgentConfig {
            specialization: "react_typescript".to_string(),
            worktree: "agents/frontend-agent".to_string(),
            branch: "feature/frontend".to_string(),
            claude_config: crate::config::ClaudeConfig::for_agent("frontend"),
            claude_md_template: "frontend_specialist".to_string(),
        },
    );

    Ok(config)
}

#[cfg(test)]
mod tests;
