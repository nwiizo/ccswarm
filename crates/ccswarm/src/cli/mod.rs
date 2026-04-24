//! CLI module for ccswarm with clippy exceptions for complex conditional patterns

#![allow(clippy::collapsible_else_if)]
#![allow(clippy::get_first)]

mod command_registry;
mod commands;

mod error_help;
mod output;
mod progress;
mod quickstart_simple;

pub(crate) mod handlers;

use output::{OutputFormatter, create_formatter};
pub use progress::{ProcessTracker, ProgressStyle, ProgressTracker, StatusLine};

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::agent::{Priority, Task, TaskType};
use crate::config::CcswarmConfig;

/// ccswarm - Claude Code integrated multi-agent system
#[derive(Parser)]
#[command(name = "ccswarm")]
#[command(about = "Claude Code multi-agent orchestration system")]
#[command(
    long_about = "ccswarm — turn tasks into PR-ready diffs with quality gates.\n\n\
    One task in, one quality-gated change out: plan → implement → review → fix,\n\
    reproducibly. Provider-agnostic (Claude Code, Codex, GitHub Copilot CLI).\n\n\
    Primary flow:\n  \
      ccswarm                                  # interactive task entry\n  \
      ccswarm pipeline --task \"...\"            # single-shot run\n  \
      ccswarm queue add \"...\"; ccswarm queue drain  # batch\n\n\
    Inspection:\n  \
      ccswarm doctor           Environment and provider CLI checks\n  \
      ccswarm runs list        Past pipeline runs\n  \
      ccswarm tail             Follow the current run's event stream\n  \
      ccswarm cost <run-id>    Duration + token breakdown"
)]
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

    /// Generate .claude/agents/*.md from facets or validate existing definitions
    #[command(name = "agent-gen")]
    AgentGen {
        #[command(subcommand)]
        action: AgentGenAction,
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

    /// Interactive task creation and execution with AI-assisted clarification
    Interactive {
        /// Interaction mode: assistant, persona, quiet, passthrough
        #[arg(short, long, default_value = "assistant")]
        mode: String,

        /// Flow to use for execution (e.g., "default")
        #[arg(short, long)]
        flow: Option<String>,
    },

    /// Execute a task through a flow pipeline
    #[command(
        long_about = "Execute a task through a flow-based workflow pipeline.\n\n\
        Flows define multi-step agent workflows (stages). The pipeline runner\n\
        executes each stage sequentially, passing context between steps.\n\n\
        Examples:\n  \
          ccswarm pipeline --task \"Fix README typo\" --flow default\n  \
          ccswarm pipeline --task \"Add tests\" --output-format json --verbose"
    )]
    Pipeline {
        /// Task description to execute
        #[arg(short, long)]
        task: String,

        /// Flow to use (default: "default")
        #[arg(short, long, default_value = "default")]
        flow: String,

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

        /// Continue from a previous run (reuse Claude Code session)
        #[arg(long, name = "continue")]
        continue_from: Option<String>,

        /// Execute in isolated git worktree
        #[arg(long)]
        isolate: bool,

        /// Budget limit in USD per stage
        #[arg(long)]
        budget: Option<f64>,

        /// Cumulative input+output token cap across the whole run.
        /// Aborts the flow after the stage that crosses the threshold.
        /// Provider-agnostic (works for claude and codex).
        #[arg(long)]
        run_budget_tokens: Option<u64>,

        /// Model override for all stages
        #[arg(long, name = "model")]
        model_override: Option<String>,

        /// Auto-commit changes after successful execution
        #[arg(long)]
        auto_commit: bool,

        /// Create GitHub PR after successful execution (requires gh cli)
        #[arg(long)]
        create_pr: bool,

        /// Print the composed prompt for each stage and exit without spawning the
        /// provider CLI. Useful for reviewing what ccswarm would send before paying
        /// for the round-trip.
        #[arg(long)]
        dry_run: bool,
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

    /// Manage workflow flows (list, eject, inspect)
    #[command(
        name = "flow",
        long_about = "Manage workflow flows for agent task execution.\n\n\
        Flows are YAML-defined workflow templates that specify stages (steps),\n\
        personas, policies, and output contracts.\n\n\
        Examples:\n  \
          ccswarm flow list\n  \
          ccswarm flow eject default\n  \
          ccswarm flow inspect default"
    )]
    Flow {
        #[command(subcommand)]
        action: FlowAction,
    },

    /// Manage external flow packages (repertoire)
    #[command(
        long_about = "Manage external flow packages from Git repositories.\n\n\
        Repertoire packages are collections of workflow flows that can be installed\n\
        from any Git repository and used alongside builtin flows.\n\n\
        Examples:\n  \
          ccswarm repertoire add https://github.com/user/my-flows\n  \
          ccswarm repertoire list\n  \
          ccswarm repertoire remove my-flows"
    )]
    Repertoire {
        #[command(subcommand)]
        action: RepertoireAction,
    },

    /// Experimental / research commands (sangha, extend, evolution, search)
    #[command(long_about = "Experimental features grouped under `lab`:\n  \
          ccswarm lab sangha propose ...      Collective voting on proposals\n  \
          ccswarm lab extend propose ...      Agent self-extension tracking\n  \
          ccswarm lab evolution report        Agent performance analytics\n  \
          ccswarm lab search docs \"...\"       Ripgrep over docs/ and source\n\n\
        These sit below the primary flow (task / pipeline / queue / runs). They exist for \
        research and may change without notice.")]
    Lab {
        #[command(subcommand)]
        action: LabAction,
    },

    /// Test harness to execute predefined scenarios and verify outcomes
    #[command(
        long_about = "Run harness scenarios to validate workflows end-to-end.\n\n\
        Scenarios are YAML files under .ccswarm/harness/ describing task, flow, and assertions.\n\n\
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

    /// Session management - list, inspect, and manage pipeline sessions
    #[command(
        long_about = "Browse and manage pipeline sessions stored in .ccswarm/runs/.\n\n\
        Sessions are created automatically when running pipelines. Each session\n\
        records events as NDJSON and produces a summary on completion.\n\n\
        Examples:\n  \
          ccswarm session list\n  \
          ccswarm session list --all\n  \
          ccswarm session view <session-id>"
    )]
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },

    /// View past pipeline runs recorded in .ccswarm/runs/
    #[command(
        long_about = "Browse pipeline run history stored in .ccswarm/runs/.\n\n\
        Each run directory contains events.ndjson and summary.json produced\n\
        by the pipeline runner.\n\n\
        Examples:\n  \
          ccswarm run list\n  \
          ccswarm run view <run-id>"
    )]
    Run {
        #[command(subcommand)]
        action: RunAction,
    },

    /// List all registered facets (personas, policies, knowledge)
    #[command(
        long_about = "Show built-in and project-local facets available to flows.\n\n\
        Examples:\n  \
          ccswarm facets\n  \
          ccswarm facets personas\n  \
          ccswarm facets policies --detailed"
    )]
    Facets {
        /// Facet type filter: personas | policies | knowledge | all (default)
        #[arg(default_value = "all")]
        kind: String,

        /// Show description/role inline
        #[arg(short, long)]
        detailed: bool,
    },

    /// Queue tasks and drain them through the pipeline in batch
    #[command(
        long_about = "Accumulate tasks in .ccswarm/queue.yaml, then process them all.\n\n\
        Examples:\n  \
          ccswarm queue add \"Add login form\"\n  \
          ccswarm queue add --from-issue 42\n  \
          ccswarm queue list\n  \
          ccswarm queue drain --timeout 600"
    )]
    Queue {
        #[command(subcommand)]
        action: QueueAction,
    },

    /// Fully autonomous mode — no y/n prompts, auto-commit, auto-PR
    #[command(
        long_about = "Self-driving loop: pull tasks → pipeline → auto-fix → auto-commit → auto-PR → repeat.\n\n\
        All interactive prompts are suppressed. Use `--watch` to keep polling the queue \
        for new tasks; otherwise stops when the queue drains.\n\n\
        Safety: aborts on exceeded iteration count, first hard failure if --stop-on-error, \
        or exceeded wall-clock budget.\n\n\
        Examples:\n  \
          ccswarm auto                          # drain queue once, auto-everything\n  \
          ccswarm auto --task \"Add X\"           # single task, no queue needed\n  \
          ccswarm auto --watch --poll-secs 30   # daemon-like: keep running\n  \
          ccswarm auto --max-iterations 5 --stop-on-error"
    )]
    Auto {
        /// Single task to execute (bypass the queue)
        #[arg(short, long)]
        task: Option<String>,

        /// Flow to use (default: "default")
        #[arg(short, long, default_value = "default")]
        flow: String,

        /// Keep polling the queue for new tasks even after it empties
        #[arg(long)]
        watch: bool,

        /// Poll interval in seconds when --watch is set
        #[arg(long, default_value_t = 30)]
        poll_secs: u64,

        /// Maximum tasks to process in one invocation (0 = unlimited)
        #[arg(long, default_value_t = 0)]
        max_iterations: usize,

        /// Wall-clock budget in seconds for the whole session (0 = unlimited)
        #[arg(long, default_value_t = 0)]
        wall_budget_secs: u64,

        /// Abort the loop on the first task that fails
        #[arg(long)]
        stop_on_error: bool,

        /// Per-task timeout in seconds
        #[arg(long, default_value_t = 600)]
        timeout: u64,

        /// Create a GitHub PR after each successful task (requires gh cli)
        #[arg(long)]
        create_pr: bool,
    },

    /// Revert-advisory for a past run (shows git commits since run started)
    #[command(
        long_about = "Inspect a past run and show the git commits that may need reverting.\n\n\
        This command is intentionally advisory: it never rewrites history on its own.\n\
        Copy the suggested `git revert` commands and run them yourself."
    )]
    Undo {
        /// Run ID (default: most recent)
        run_id: Option<String>,
    },

    /// Replay a past run: re-execute the recorded task through the pipeline
    #[command(
        long_about = "Extract the task from a past run's summary.json and re-run it.\n\n\
        Examples:\n  \
          ccswarm replay <run-id>\n  \
          ccswarm replay <run-id> --flow review-fix"
    )]
    Replay {
        /// Run ID (default: most recent)
        run_id: Option<String>,

        /// Override the flow used for replay (default: same flow as original)
        #[arg(short, long)]
        flow: Option<String>,

        /// Timeout in seconds
        #[arg(long, default_value = "600")]
        timeout: u64,
    },

    /// Show token / duration breakdown for a past run
    #[command(
        long_about = "Aggregate per-stage and per-agent metrics from events.ndjson.\n\n\
        Examples:\n  \
          ccswarm cost\n  \
          ccswarm cost <run-id>"
    )]
    Cost {
        /// Run ID (default: most recent)
        run_id: Option<String>,
    },

    /// Tail a pipeline run's event stream (live when still running)
    #[command(
        long_about = "Follow the NDJSON event log for a run with pretty formatting.\n\n\
        Examples:\n  \
          ccswarm tail\n  \
          ccswarm tail <run-id>\n  \
          ccswarm tail <run-id> --no-follow"
    )]
    Tail {
        /// Run ID (default: most recent run in .ccswarm/runs/)
        run_id: Option<String>,

        /// Do not follow new events; print existing log and exit
        #[arg(long)]
        no_follow: bool,
    },

    /// Create a new project and run pipeline in one command
    #[command(
        long_about = "Scaffold a new project: create directory, git init, run pipeline.\n\n\
        Examples:\n  \
          ccswarm scaffold --dir /tmp/myapp --task \"Create a todo app\"\n  \
          ccswarm scaffold --dir ./myapp --task \"Build a REST API\" --flow quick"
    )]
    Scaffold {
        /// Directory to create
        #[arg(short, long)]
        dir: PathBuf,

        /// Task description
        #[arg(short, long)]
        task: String,

        /// Flow to use (default: "default")
        #[arg(short, long, default_value = "default")]
        flow: String,

        /// Timeout in seconds
        #[arg(long, default_value = "600")]
        timeout: u64,
    },
}

#[derive(Subcommand)]
pub enum RepertoireAction {
    /// Install a flow package from a Git repository
    Add {
        /// Git URL of the flow package
        url: String,
    },

    /// List installed flow packages
    List,

    /// Remove an installed flow package
    Remove {
        /// Package name to remove
        name: String,
    },
}

#[derive(Subcommand)]
pub enum LabAction {
    /// Collective voting on proposals
    Sangha {
        #[command(subcommand)]
        action: SanghaAction,
    },
    /// Agent self-extension tracking
    Extend {
        #[command(subcommand)]
        action: ExtendAction,
    },
    /// Agent performance analytics
    Evolution {
        #[command(subcommand)]
        action: EvolutionAction,
    },
    /// Ripgrep over docs/ and source
    Search {
        #[command(subcommand)]
        action: SearchAction,
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
pub enum QueueAction {
    /// Append a task to the queue
    Add {
        /// Task description (ignored if --from-issue or --file is given).
        /// Pass `-` to read the task body from stdin.
        #[arg(default_value = "")]
        task: String,
        /// Load task body from a GitHub issue via `gh issue view`
        #[arg(long, value_name = "NUMBER")]
        from_issue: Option<u64>,
        /// Load task body from a file (convenient for long prompts)
        #[arg(long, value_name = "PATH")]
        file: Option<std::path::PathBuf>,
        /// Flow to run when this task is drained (default: "default")
        #[arg(short, long)]
        flow: Option<String>,
    },
    /// Show queued tasks
    List,
    /// Clear pending tasks from the queue
    Clear,
    /// Execute all queued tasks through the pipeline.
    /// `drain` runs unattended by default — all commit/PR prompts are suppressed so
    /// the queue fully empties without user input. Pass `--interactive` to restore
    /// the old per-task y/n prompts.
    Drain {
        /// Flow override; by default each task uses its per-task flow or "default"
        #[arg(short, long)]
        flow: Option<String>,
        /// Timeout per task in seconds
        #[arg(long, default_value = "600")]
        timeout: u64,
        /// Stop on the first failure (default: continue)
        #[arg(long)]
        fail_fast: bool,
        /// Restore per-task y/n prompts (default: off, unattended)
        #[arg(long)]
        interactive: bool,
        /// Also create a GitHub PR for each successful task (requires `gh` CLI)
        #[arg(long)]
        create_pr: bool,
    },
}

#[derive(Subcommand)]
pub enum RunAction {
    /// List past pipeline runs (sorted by date, newest first)
    List,
    /// View details and events for a specific run
    View {
        /// Run ID to inspect
        id: String,
    },
    /// Compare the timeline of two runs
    Diff {
        /// Baseline run ID
        a: String,
        /// Candidate run ID
        b: String,
    },
}

#[derive(Subcommand)]
pub enum AgentGenAction {
    /// Generate a .claude/agents/*.md file from facets
    Generate {
        /// Agent name (used as filename)
        name: String,
        /// Persona facet to use (e.g. "coder", "reviewer")
        #[arg(short, long)]
        persona: Option<String>,
        /// Policy facet to use (e.g. "coding", "security")
        #[arg(long)]
        policy: Option<String>,
        /// Short description for the agent
        #[arg(short, long)]
        description: Option<String>,
        /// Model to use (default: sonnet)
        #[arg(short, long, default_value = "sonnet")]
        model: String,
        /// Output directory (default: .claude/agents/)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Validate existing .claude/agents/*.md files
    Validate {
        /// Specific file to validate (default: all in .claude/agents/)
        path: Option<PathBuf>,
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
pub enum FlowAction {
    /// List all available flows (builtin and custom)
    List,

    /// Eject a builtin flow to a local YAML file for customization
    Eject {
        /// Name of the flow to eject
        name: String,

        /// Output directory (default: .ccswarm/flows/)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Inspect a flow's structure and stages
    Inspect {
        /// Name of the flow to inspect
        name: String,
    },

    /// Check a flow YAML file for structural validity
    Check {
        /// Name of builtin/custom flow, or path to a YAML file
        target: String,
    },

    /// Scaffold a new flow YAML file
    New {
        /// Name of the new flow (used as filename)
        name: String,

        /// Template: "minimal" (1 stage) or "faceted" (plan+implement+review)
        #[arg(short, long, default_value = "minimal")]
        template: String,

        /// Output directory (default: .ccswarm/flows/)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Render composed prompts for each stage in a flow
    Render {
        /// Name of builtin/custom flow, or path to a YAML file
        target: String,

        /// Render only the stage with this ID
        #[arg(short, long)]
        stage: Option<String>,
    },

    /// Suggest an appropriate builtin flow for a given task description
    Suggest {
        /// Task description to classify
        task: String,
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
    /// List all sessions (pipeline runs) with metadata extracted from event logs
    List {
        /// Show all sessions including those without summary
        #[arg(short, long)]
        all: bool,
    },

    /// View details and events for a specific session
    View {
        /// Session/run ID to inspect
        id: String,
    },

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

        Ok(Self {
            config,
            repo_path: cli.repo.clone(),
            json_output: cli.json,
            formatter,
        })
    }

    /// Run the CLI command
    pub async fn run(&self, command: &Commands) -> Result<()> {
        // Use the command registry for centralized command handling
        let registry = self::command_registry::get_command_registry();
        registry.execute(self, command).await
    }

    /// Handle scaffold command
    pub async fn handle_scaffold(
        &self,
        dir: &std::path::Path,
        task: &str,
        flow: &str,
        timeout: u64,
    ) -> Result<()> {
        handlers::scaffold::handle_scaffold(dir, task, flow, timeout).await
    }

    /// Handle agent-gen subcommands
    pub async fn handle_agent_gen(&self, action: &AgentGenAction) -> Result<()> {
        match action {
            AgentGenAction::Generate {
                name,
                persona,
                policy,
                description,
                model,
                output,
            } => {
                let output_dir = output
                    .clone()
                    .unwrap_or_else(|| self.repo_path.join(".claude/agents"));
                let path = handlers::agent_gen::generate_agent_definition(
                    name,
                    persona.as_deref(),
                    policy.as_deref(),
                    description.as_deref(),
                    model,
                    &output_dir,
                )
                .await?;
                println!("Generated agent definition: {}", path.display());
                Ok(())
            }
            AgentGenAction::Validate { path } => {
                let agents_dir = self.repo_path.join(".claude/agents");
                let paths: Vec<PathBuf> = if let Some(p) = path {
                    vec![p.clone()]
                } else {
                    let mut entries = Vec::new();
                    let mut dir = tokio::fs::read_dir(&agents_dir).await?;
                    while let Some(entry) = dir.next_entry().await? {
                        let p = entry.path();
                        if p.extension().is_some_and(|ext| ext == "md") {
                            entries.push(p);
                        }
                    }
                    entries
                };

                let mut all_valid = true;
                for p in &paths {
                    let issues = handlers::agent_gen::validate_agent_definition(p).await?;
                    if issues.is_empty() {
                        println!("  {} OK", p.display());
                    } else {
                        all_valid = false;
                        println!("  {} {} issue(s):", p.display(), issues.len());
                        for issue in &issues {
                            println!("    - {}", issue);
                        }
                    }
                }
                if all_valid {
                    println!("All agent definitions are valid.");
                }
                Ok(())
            }
        }
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
