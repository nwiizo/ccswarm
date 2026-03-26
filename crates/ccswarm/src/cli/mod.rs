//! CLI module for ccswarm with clippy exceptions for complex conditional patterns

#![allow(clippy::collapsible_else_if)]
#![allow(clippy::get_first)]

mod command_registry;
mod commands;
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

mod handlers;

pub use interactive_help::{InteractiveHelp, show_quick_help};
use output::{OutputFormatter, create_formatter};
pub use progress::{ProcessTracker, ProgressStyle, ProgressTracker, StatusLine};
pub use setup_wizard::SetupWizard;
pub use tutorial::InteractiveTutorial;

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn};

use crate::agent::{Priority, Task, TaskType};
use crate::config::CcswarmConfig;
use crate::execution::{ExecutionEngine, TaskStatus};
use crate::orchestrator::ProactiveMaster;

/// ccswarm - Claude Code integrated multi-agent system
#[derive(Parser)]
#[command(name = "ccswarm")]
#[command(about = "Claude Code multi-agent orchestration system")]
#[command(long_about = "ccswarm - AI Multi-Agent Orchestration System\n\n\
    Coordinate specialized AI agents (Frontend, Backend, DevOps, QA) using Claude Code.\n\
    Each agent works in isolated git worktrees with strict role boundaries.\n\n\
    Quick start:\n  \
      ccswarm quickstart --name MyProject\n  \
      ccswarm task execute \"Add user authentication\"\n\n\
    For more information:\n  \
      ccswarm doctor          Check system health\n  \
      ccswarm tutorial        Interactive walkthrough\n  \
      ccswarm help-topic      Extended help system")]
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

    /// Log format: text (default), ndjson
    #[arg(long, default_value = "text")]
    pub log_format: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new ccswarm project
    #[command(
        long_about = "Initialize a new ccswarm project in the current directory.\n\n\
        Creates ccswarm.json configuration and sets up agent worktrees.\n\n\
        Examples:\n  \
          ccswarm init --name MyApp\n  \
          ccswarm init --name MyApp --agents frontend,backend,qa"
    )]
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
    #[command(long_about = "Start the ccswarm orchestrator and agent processes.\n\n\
        The orchestrator coordinates task delegation and monitors agent health.\n\n\
        Examples:\n  \
          ccswarm start\n  \
          ccswarm start --daemon --port 9090\n  \
          ccswarm start --enable-acp")]
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

        /// Enable delegate mode (lead orchestrates only, no direct code execution)
        #[arg(long)]
        delegate: bool,

        /// Enable ACP (Agent Communication Protocol) WebSocket server
        #[arg(long)]
        enable_acp: bool,
    },

    /// Verify an auto-created application
    Verify {
        /// Path to the application to verify
        #[arg(default_value = "./")]
        path: PathBuf,

        /// Backend port for health checks
        #[arg(long, default_value = "3000")]
        backend_port: u16,

        /// Skip dependency installation
        #[arg(long)]
        skip_deps: bool,
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
    #[command(
        long_about = "Create, list, execute, merge, retry, and delete tasks.\n\n\
        Tasks are the primary unit of work in ccswarm. Each task is analyzed by the\n\
        Master Claude orchestrator and delegated to the appropriate specialist agent.\n\n\
        Examples:\n  \
          ccswarm task execute \"Fix login bug\"\n  \
          ccswarm task list --branches\n  \
          ccswarm task merge <id>"
    )]
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

    /// Interactive task creation and execution with AI-assisted clarification
    Interactive {
        /// Interaction mode: assistant, persona, quiet, passthrough
        #[arg(short, long, default_value = "assistant")]
        mode: String,

        /// Piece to use for execution (e.g., "default")
        #[arg(short, long)]
        piece: Option<String>,
    },

    /// Execute a task through a piece pipeline
    #[command(
        long_about = "Execute a task through a piece-based workflow pipeline.\n\n\
        Pieces define multi-step agent workflows (movements). The pipeline runner\n\
        executes each movement sequentially, passing context between steps.\n\n\
        Examples:\n  \
          ccswarm pipeline --task \"Fix README typo\" --piece default\n  \
          ccswarm pipeline --task \"Add tests\" --output-format json --verbose"
    )]
    Pipeline {
        /// Task description to execute
        #[arg(short, long)]
        task: String,

        /// Piece to use (default: "default")
        #[arg(short, long, default_value = "default")]
        piece: String,

        /// Output format: text, json, markdown
        #[arg(short, long, default_value = "text")]
        output_format: String,

        /// Timeout in seconds
        #[arg(long, default_value = "600")]
        timeout: u64,

        /// Verbose output with execution details
        #[arg(short, long)]
        verbose: bool,

        /// Write output to file
        #[arg(short = 'O', long)]
        output_file: Option<PathBuf>,
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
    #[command(long_about = "Diagnose and fix common ccswarm issues.\n\n\
        Checks Git setup, Claude Code CLI availability, API key configuration,\n\
        worktree health, and configuration file validity.\n\n\
        Examples:\n  \
          ccswarm doctor\n  \
          ccswarm doctor --fix\n  \
          ccswarm doctor --check-api")]
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

    /// Manage workflow pieces (list, eject, inspect)
    #[command(
        name = "piece",
        long_about = "Manage workflow pieces for agent task execution.\n\n\
        Pieces are YAML-defined workflow templates that specify movements (steps),\n\
        personas, policies, and output contracts.\n\n\
        Examples:\n  \
          ccswarm piece list\n  \
          ccswarm piece eject default\n  \
          ccswarm piece inspect default"
    )]
    Piece {
        #[command(subcommand)]
        action: PieceAction,
    },

    /// Manage external piece packages (repertoire)
    #[command(
        long_about = "Manage external piece packages from Git repositories.\n\n\
        Repertoire packages are collections of workflow pieces that can be installed\n\
        from any Git repository and used alongside builtin pieces.\n\n\
        Examples:\n  \
          ccswarm repertoire add https://github.com/user/my-pieces\n  \
          ccswarm repertoire list\n  \
          ccswarm repertoire remove my-pieces"
    )]
    Repertoire {
        #[command(subcommand)]
        action: RepertoireAction,
    },

    /// Sangha collective intelligence - propose and vote on changes
    #[command(long_about = "Democratic decision-making for agent swarms.\n\
        Proposals are persisted to coordination/proposals/ as JSON.\n\n\
        Examples:\n  \
          ccswarm sangha propose --title \"Add GraphQL\" --description \"...\"\n  \
          ccswarm sangha vote abc-123 --approve\n  \
          ccswarm sangha list\n  \
          ccswarm sangha status abc-123")]
    Sangha {
        #[command(subcommand)]
        action: SanghaAction,
    },

    /// Agent self-extension - propose and track capability extensions
    #[command(
        long_about = "Agents propose new capabilities tracked in coordination/extensions/.\n\n\
        Examples:\n  \
          ccswarm extend propose --title \"GraphQL resolver\" --agent backend\n  \
          ccswarm extend list\n  \
          ccswarm extend status ext-456\n  \
          ccswarm extend history"
    )]
    Extend {
        #[command(subcommand)]
        action: ExtendAction,
    },

    /// Search documentation and code
    #[command(long_about = "Search project documentation and source code.\n\n\
        Examples:\n  \
          ccswarm search docs \"authentication patterns\"\n  \
          ccswarm search code \"error handling\" --glob \"*.rs\"")]
    Search {
        #[command(subcommand)]
        action: SearchAction,
    },

    /// Agent evolution metrics and pattern analysis
    #[command(
        long_about = "Analyze agent performance from coordination/ history.\n\n\
        Examples:\n  \
          ccswarm evolution metrics\n  \
          ccswarm evolution patterns --agent frontend\n  \
          ccswarm evolution report --format json"
    )]
    Evolution {
        #[command(subcommand)]
        action: EvolutionAction,
    },

    /// Test harness to execute predefined scenarios and verify outcomes
    #[command(
        long_about = "Run harness scenarios to validate workflows end-to-end.\n\n\
        Scenarios are YAML files under .ccswarm/harness/ describing task, piece, and assertions.\n\n\
        Examples:\n  \
          ccswarm harness run\n  \
          ccswarm harness run --scenario .ccswarm/harness/add-login.yaml --report report.json --format json\n  \
          ccswarm harness list\n  \
          ccswarm harness plan\n  \
          ccswarm harness diff --baseline baseline.json\n  \
          ccswarm harness approve --report report.json --baseline baseline.json"
    )]
    Harness {
        #[command(subcommand)]
        action: HarnessAction,
    },

    /// Human-in-the-loop approval workflow
    #[command(long_about = "Approve or reject gated operations.\n\n\
        Examples:\n  \
          ccswarm approve plan --id run-abc123\n  \
          ccswarm approve deploy --id task-456 --reject --reason \"needs more tests\"\n  \
          ccswarm approve list --status pending")]
    Approve {
        #[command(subcommand)]
        action: ApproveAction,
    },
}

#[derive(Subcommand)]
pub enum RepertoireAction {
    /// Install a piece package from a Git repository
    Add {
        /// Git URL of the piece package
        url: String,
    },

    /// List installed piece packages
    List,

    /// Remove an installed piece package
    Remove {
        /// Package name to remove
        name: String,
    },
}

#[derive(Subcommand)]
pub enum SanghaAction {
    /// Create a proposal for collective voting
    Propose {
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        description: String,
        /// feature | refactor | policy | tooling
        #[arg(long, default_value = "feature")]
        proposal_type: String,
    },
    /// Vote on a proposal
    Vote {
        /// Proposal ID
        id: String,
        /// Approve (omit to reject)
        #[arg(long)]
        approve: bool,
        #[arg(short, long)]
        reason: Option<String>,
    },
    /// List proposals
    List {
        /// Filter: open | accepted | rejected
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Show proposal details and vote tally
    Status {
        /// Proposal ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum ExtendAction {
    /// Propose a capability extension
    Propose {
        #[arg(short, long)]
        title: String,
        #[arg(short, long)]
        description: String,
        /// Target agent: frontend | backend | devops | qa | all
        #[arg(short, long, default_value = "all")]
        agent: String,
    },
    /// List extensions
    List {
        /// Filter: proposed | approved | active | deprecated
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Show extension details
    Status {
        /// Extension ID
        id: String,
    },
    /// Show recent extension history
    History {
        /// Max entries to show
        #[arg(default_value = "20")]
        limit: usize,
    },
}

#[derive(Subcommand)]
pub enum SearchAction {
    /// Search documentation files
    Docs {
        /// Search query
        query: String,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Search source code
    Code {
        /// Search query
        query: String,
        /// File glob filter (e.g. "*.rs")
        #[arg(short, long)]
        glob: Option<String>,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

#[derive(Subcommand)]
pub enum EvolutionAction {
    /// Show agent performance metrics from coordination/agent-status/
    Metrics {
        #[arg(short, long)]
        agent: Option<String>,
        /// text | json
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Analyze task success/failure patterns from coordination/task-queue/
    Patterns {
        #[arg(short, long)]
        agent: Option<String>,
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    /// Generate evolution report
    Report {
        /// text | json | markdown
        #[arg(short, long, default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum ApproveAction {
    /// Approve or reject a plan gate
    Plan {
        #[arg(long)]
        id: String,
        #[arg(long)]
        reject: bool,
        #[arg(long)]
        reason: Option<String>,
    },
    /// Approve or reject a risky edit gate
    RiskyEdit {
        #[arg(long)]
        id: String,
        #[arg(long)]
        reject: bool,
        #[arg(long)]
        reason: Option<String>,
    },
    /// Approve or reject a deploy gate
    Deploy {
        #[arg(long)]
        id: String,
        #[arg(long)]
        reject: bool,
        #[arg(long)]
        reason: Option<String>,
    },
    /// Approve or reject a merge gate
    Merge {
        #[arg(long)]
        id: String,
        #[arg(long)]
        reject: bool,
        #[arg(long)]
        reason: Option<String>,
    },
    /// List approval requests
    List {
        #[arg(short, long)]
        status: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum HarnessAction {
    /// Run scenarios from a file or directory (defaults to .ccswarm/harness/)
    Run {
        /// Scenario YAML file
        #[arg(short, long)]
        scenario: Option<PathBuf>,
        /// Directory containing scenarios
        #[arg(short, long)]
        dir: Option<PathBuf>,
        /// Write report to file
        #[arg(short, long)]
        report: Option<PathBuf>,
        /// Report format (json|text|markdown)
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Parallel jobs (0 or omitted = auto)
        #[arg(short = 'j', long, default_value_t = 0)]
        jobs: usize,
    },

    /// List discovered scenarios under .ccswarm/harness/
    List,

    /// Show expanded execution plan without running
    Plan {
        /// Scenario YAML file
        #[arg(short, long)]
        scenario: Option<PathBuf>,
        /// Directory containing scenarios
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },

    /// Compare current results against a baseline report
    Diff {
        /// Baseline JSON file (created by harness run --report)
        #[arg(long)]
        baseline: PathBuf,
        /// Scenario YAML file (optional; if omitted, use .ccswarm/harness/)
        #[arg(short, long)]
        scenario: Option<PathBuf>,
        /// Directory containing scenarios (optional)
        #[arg(short, long)]
        dir: Option<PathBuf>,
        /// Output format (json|text|markdown)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Approve current results as new baseline
    Approve {
        /// Source report JSON (current run)
        #[arg(long)]
        report: PathBuf,
        /// Destination baseline JSON
        #[arg(long)]
        baseline: PathBuf,
        /// Overwrite without prompt
        #[arg(long)]
        force: bool,
    },

    /// Create a sample harness scenario file
    Init {
        /// Output file path (default: .ccswarm/harness/sample.yaml)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Scenario name to embed
        #[arg(short, long, default_value = "sample-task")]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum PieceAction {
    /// List all available pieces (builtin and custom)
    List,

    /// Eject a builtin piece to a local YAML file for customization
    Eject {
        /// Name of the piece to eject
        name: String,

        /// Output directory (default: .ccswarm/pieces/)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Inspect a piece's structure and movements
    Inspect {
        /// Name of the piece to inspect
        name: String,
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

        /// Show worktree branch information for each task
        #[arg(long)]
        branches: bool,
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

    /// Merge a completed task's worktree branch into main
    Merge {
        /// Task ID to merge
        task_id: String,

        /// Delete the worktree branch after merge
        #[arg(long, default_value = "true")]
        cleanup: bool,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Retry a failed task
    Retry {
        /// Task ID to retry
        task_id: String,

        /// Force retry even if task is not in failed state
        #[arg(short, long)]
        force: bool,
    },

    /// Delete a task and its associated worktree
    Delete {
        /// Task ID to delete
        task_id: String,

        /// Force deletion even if task is in progress
        #[arg(short, long)]
        force: bool,
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

    /// Show configuration details
    Show {
        /// Configuration file
        #[arg(short, long, default_value = "ccswarm.json")]
        file: PathBuf,
        /// Show a single agent configuration
        #[arg(short, long)]
        agent: Option<String>,
        /// Print raw JSON (ignored for JSON output)
        #[arg(long)]
        raw: bool,
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

pub struct CliRunner {
    config: CcswarmConfig,
    repo_path: PathBuf,
    json_output: bool,
    formatter: OutputFormatter,
    execution_engine: Option<ExecutionEngine>,
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

        // Only initialize execution engine for commands that need it.
        // Many commands (doctor, piece, repertoire, help, config, etc.)
        // don't need agents spawned, so skip expensive initialization.
        let execution_engine = if Self::command_needs_engine(&cli.command) {
            match ExecutionEngine::new(&config).await {
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
            }
        } else {
            None
        };

        Ok(Self {
            config,
            repo_path: cli.repo.clone(),
            json_output: cli.json,
            formatter,
            execution_engine,
        })
    }

    /// Check if a command requires the execution engine (agent spawning).
    /// Commands that only read config/display info should NOT spawn agents.
    fn command_needs_engine(command: &Commands) -> bool {
        matches!(
            command,
            Commands::Start { .. }
                | Commands::Task { .. }
                | Commands::Tui
                | Commands::AutoCreate { .. }
                | Commands::Pipeline { .. }
                | Commands::Interactive { .. }
                | Commands::Delegate { .. }
        )
    }

    /// Run the CLI command
    pub async fn run(&self, command: &Commands) -> Result<()> {
        // Use the command registry for centralized command handling
        let registry = self::command_registry::get_command_registry();
        registry.execute(self, command).await
    }
}

fn create_default_config(repo_path: &Path) -> Result<CcswarmConfig> {
    let mut agents = std::collections::HashMap::new();

    // Add common agent configurations
    agents.insert(
        "frontend".to_string(),
        crate::config::AgentConfig {
            specialization: "react_typescript".to_string(),
            worktree: "../worktrees/frontend-agent".to_string(),
            branch: "feature/frontend-ui".to_string(),
            claude_config: crate::config::ClaudeConfig::for_agent("frontend"),
            claude_md_template: "frontend_specialist".to_string(),
        },
    );

    agents.insert(
        "backend".to_string(),
        crate::config::AgentConfig {
            specialization: "node_microservices".to_string(),
            worktree: "../worktrees/backend-agent".to_string(),
            branch: "feature/backend-api".to_string(),
            claude_config: crate::config::ClaudeConfig::for_agent("backend"),
            claude_md_template: "backend_specialist".to_string(),
        },
    );

    agents.insert(
        "devops".to_string(),
        crate::config::AgentConfig {
            specialization: "aws_kubernetes".to_string(),
            worktree: "../worktrees/devops-agent".to_string(),
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
                ..Default::default()
            },
            master_claude: crate::config::MasterClaudeConfig {
                role: "technical_lead".to_string(),
                quality_threshold: 0.90,
                think_mode: crate::config::ThinkMode::UltraThink,
                permission_level: "supervised".to_string(),
                claude_config: crate::config::ClaudeConfig::for_master(),
                enable_proactive_mode: true, // Enabled by default
                proactive_frequency: 30,     // 30 second interval
                high_frequency: 15,          // High frequency 15 second interval
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
            worktree: "../worktrees/frontend-agent".to_string(),
            branch: "feature/frontend".to_string(),
            claude_config: crate::config::ClaudeConfig::for_agent("frontend"),
            claude_md_template: "frontend_specialist".to_string(),
        },
    );

    Ok(config)
}
