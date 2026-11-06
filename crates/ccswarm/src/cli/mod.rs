//! CLI module for ccswarm with clippy exceptions for complex conditional patterns

#![allow(clippy::collapsible_else_if)]
#![allow(clippy::get_first)]

mod command_registry;

mod output;
mod quickstart_simple;

pub(crate) mod handlers;

use output::{OutputFormatter, create_formatter};

use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::Write;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::config::CcswarmConfig;

/// ccswarm - Claude Code integrated multi-agent system
#[derive(Parser)]
#[command(name = "ccswarm")]
#[command(about = "Claude Code multi-agent orchestration system")]
#[command(
    long_about = "ccswarm — turn tasks into PR-ready diffs with quality gates.\n\n\
    One task in, one quality-gated change out: plan → implement → review → fix,\n\
    reproducibly. Supports Claude Code and Codex; Copilot configuration names\n\
    are accepted only to fail fast with migration guidance.\n\n\
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

    /// Default provider for stages that don't pin one in flow YAML.
    ///
    /// Executable providers are `claude` and `codex`. `copilot` remains
    /// parseable for configuration compatibility but fails fast with guidance.
    /// Overrides `CCSWARM_PROVIDER`.
    #[arg(long, global = true)]
    pub provider: Option<String>,

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
        Checks Git setup, Herdr, provider CLIs, API key configuration,\n\
        worktree health, and configuration file validity.\n\n\
        Examples:\n  \
          ccswarm doctor\n  \
          ccswarm doctor --fix\n  \
          ccswarm doctor --check-api")]
    Doctor {
        /// Run fixes for common issues
        #[arg(short, long)]
        fix: bool,

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

        /// Pause before each commit until `ccswarm approve commit --id <run-id>`
        #[arg(long)]
        require_approval: bool,

        /// Seconds to wait for a commit approval before failing the task
        #[arg(long, default_value_t = 600)]
        approval_timeout: u64,
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

    /// Herdr plugin entry points (see docs/HERDR_INTEGRATION.md)
    #[command(
        long_about = "Entry points used by the ccswarm Herdr plugin in herdr-plugin/.\n\n\
        Examples:\n  \
          ccswarm herdr action run-board   # plugin action: open the run board\n  \
          ccswarm herdr show runs          # plugin pane: recent runs"
    )]
    Herdr {
        #[command(subcommand)]
        action: HerdrAction,
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
        /// Also create a Sangha proposal for consensus approval
        #[arg(long)]
        auto_sangha: bool,
    },
    /// Generate an extension proposal from local workflow context
    AutoPropose {
        /// Target agent: frontend | backend | devops | qa | all
        #[arg(short, long, default_value = "all")]
        agent: String,
        /// Optional reason or observed gap to include in the proposal
        #[arg(short, long)]
        reason: Option<String>,
        /// Do not create a linked Sangha proposal
        #[arg(long = "no-auto-sangha", action = clap::ArgAction::SetFalse, default_value_t = true)]
        auto_sangha: bool,
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
    /// Approve or reject a pending commit gate (unattended runs)
    Commit {
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
        /// Load task body from a tracker issue
        #[arg(long, value_name = "ISSUE")]
        from_issue: Option<String>,
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
    /// Manually release a claimed task
    Release {
        /// Queue task ID to release
        id: String,
        /// Release reason recorded in the queue file
        #[arg(long)]
        reason: String,
    },
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
        /// Run queue reconciliation and exit without dispatching new work
        #[arg(long)]
        reconcile_only: bool,
        /// Pause before each commit until `ccswarm approve commit --id <run-id>`
        #[arg(long)]
        require_approval: bool,
        /// Seconds to wait for a commit approval before failing the task
        #[arg(long, default_value_t = 600)]
        approval_timeout: u64,
    },
}

#[derive(Subcommand)]
pub enum HerdrAction {
    /// Open a ccswarm plugin pane in the focused pane's directory. Herdr
    /// plugin actions call this with HERDR_PLUGIN_CONTEXT_JSON set.
    Action {
        #[arg(value_enum)]
        view: HerdrView,
    },
    /// Show a view in the current pane; popups wait for Enter before closing.
    Show {
        #[arg(value_enum)]
        view: HerdrView,
    },
}

/// Views provided by the ccswarm Herdr plugin.
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum HerdrView {
    /// Follow the current run's events (`ccswarm tail`)
    RunBoard,
    /// Recent runs (`ccswarm run list`)
    Runs,
    /// Environment checks (`ccswarm doctor`)
    Doctor,
}

impl HerdrView {
    /// Plugin pane entrypoint ID in herdr-plugin/herdr-plugin.toml.
    pub fn entrypoint(self) -> &'static str {
        match self {
            Self::RunBoard => "run-board",
            Self::Runs => "runs",
            Self::Doctor => "doctor",
        }
    }

    /// Whether the manifest places this view in a popup, which Herdr opens
    /// over the active pane; split panes need an explicit target pane.
    pub fn is_popup(self) -> bool {
        !matches!(self, Self::RunBoard)
    }
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

pub struct CliRunner {
    config: CcswarmConfig,
    repo_path: PathBuf,
    json_output: bool,
    formatter: OutputFormatter,
    /// `--provider` flag: default provider for stages that don't pin one in
    /// flow YAML. Overrides the CCSWARM_PROVIDER env var.
    default_provider: Option<crate::providers::ProviderKind>,
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
            if !cli.json {
                warn!("Configuration file not found, using defaults");
            }
            create_default_config(&cli.repo)?
        };

        let formatter = create_formatter(cli.json);

        // Fail fast on an unknown provider name rather than silently running
        // everything on the claude default.
        let default_provider = match cli.provider.as_deref() {
            Some(name) => Some(crate::providers::ProviderKind::parse(name).ok_or_else(|| {
                anyhow::anyhow!(
                    "unknown provider '{}' (expected: claude | codex | copilot)",
                    name
                )
            })?),
            None => None,
        };

        Ok(Self {
            config,
            repo_path: cli.repo.clone(),
            json_output: cli.json,
            formatter,
            default_provider,
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
        handlers::scaffold::handle_scaffold(dir, task, flow, timeout, self.default_provider).await
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
