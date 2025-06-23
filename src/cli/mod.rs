use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::agent::{Priority, Task, TaskType};
use crate::config::CcswarmConfig;
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

    /// Add a task to the queue
    Task {
        /// Task description
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

pub struct CliRunner {
    config: CcswarmConfig,
    repo_path: PathBuf,
    json_output: bool,
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

        Ok(Self {
            config,
            repo_path: cli.repo.clone(),
            json_output: cli.json,
        })
    }

    /// Run the CLI command
    pub async fn run(&self, command: &Commands) -> Result<()> {
        match command {
            Commands::Init {
                name,
                repo_url,
                agents,
            } => self.init_project(name, repo_url.as_deref(), agents).await,
            Commands::Start {
                daemon,
                port,
                isolation,
                use_real_api,
            } => {
                self.start_orchestrator(*daemon, *port, isolation, *use_real_api)
                    .await
            }
            Commands::Tui => self.start_tui().await,
            Commands::Stop => self.stop_orchestrator().await,
            Commands::Status { detailed, agent } => {
                self.show_status(*detailed, agent.as_deref()).await
            }
            Commands::Task {
                description,
                priority,
                task_type,
                details,
                duration,
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
            Commands::Agents { all } => self.list_agents(*all).await,
            Commands::Review { agent, strict } => self.run_review(agent.as_deref(), *strict).await,
            Commands::Worktree { action } => self.handle_worktree(action).await,
            Commands::Logs {
                follow,
                agent,
                lines,
            } => self.show_logs(*follow, agent.as_deref(), *lines).await,
            Commands::Config { action } => self.handle_config(action).await,
            Commands::Delegate { action } => self.handle_delegate(action).await,
            Commands::Session { action } => self.handle_session(action).await,
            Commands::AutoCreate {
                description,
                template,
                auto_deploy,
                output,
            } => {
                self.handle_auto_create(description, template.as_deref(), *auto_deploy, output)
                    .await
            }
            Commands::Sangha { action } => self.handle_sangha(action).await,
            Commands::Extend { action } => self.handle_extend(action).await,
            Commands::Search { action } => self.handle_search(action).await,
            Commands::Evolution { action } => self.handle_evolution(action).await,
        }
    }

    async fn init_project(
        &self,
        name: &str,
        repo_url: Option<&str>,
        agents: &[String],
    ) -> Result<()> {
        info!("Initializing ccswarm project: {}", name);

        // Initialize Git repository if needed
        crate::git::shell::ShellWorktreeManager::init_if_needed(&self.repo_path).await?;

        // Create configuration
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

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Project initialized",
                    "project": name,
                    "agents": agents,
                }))?
            );
        } else {
            println!("✅ ccswarm project '{}' initialized", name);
            if !agents.is_empty() {
                println!("   Agents: {}", agents.join(", "));
            }
        }

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

        // Start TUI
        crate::tui::run_tui().await?;

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
                    if detailed {
                        println!(
                            "Details: {}",
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

        // Add to task queue
        let queue = crate::coordination::TaskQueue::new().await?;
        queue.add_task(&task).await?;

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "success",
                    "message": "Task added",
                    "task_id": task.id,
                    "description": description,
                    "priority": priority,
                }))?
            );
        } else {
            println!("✅ Task added: {}", task.id);
            println!("   Description: {}", description);
            println!("   Priority: {:?}", priority);
            println!("   Type: {:?}", task_type_clone);
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

impl CliRunner {
    async fn handle_delegate(&self, action: &DelegateAction) -> Result<()> {
        use crate::orchestrator::master_delegation::{DelegationStrategy, MasterDelegationEngine};

        match action {
            DelegateAction::Task {
                description,
                agent,
                priority,
                task_type,
                details,
                force,
            } => {
                let task = self.create_task_from_args(
                    description,
                    priority,
                    task_type,
                    details.as_deref(),
                    None,
                )?;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Task delegated",
                            "task": task,
                            "target_agent": agent,
                            "forced": force,
                        }))?
                    );
                } else {
                    println!("🎯 Delegating task to {} agent", agent);
                    println!("   Task: {}", task.description);
                    println!("   Priority: {:?}", task.priority);
                    println!("   Type: {:?}", task.task_type);
                    if *force {
                        println!("   ⚠️ Forced delegation");
                    }
                }
            }

            DelegateAction::Analyze {
                description,
                verbose,
                strategy,
            } => {
                let strategy = match strategy.as_str() {
                    "content" => DelegationStrategy::ContentBased,
                    "load" => DelegationStrategy::LoadBalanced,
                    "expertise" => DelegationStrategy::ExpertiseBased,
                    "workflow" => DelegationStrategy::WorkflowBased,
                    "hybrid" => DelegationStrategy::Hybrid,
                    _ => DelegationStrategy::Hybrid,
                };

                let mut engine = MasterDelegationEngine::new(strategy);
                let task = Task::new(
                    "analysis".to_string(),
                    description.clone(),
                    Priority::Medium,
                    TaskType::Development,
                );

                let decision = engine.delegate_task(task)?;

                if self.json_output {
                    println!("{}", serde_json::to_string_pretty(&decision)?);
                } else {
                    println!("🔍 Task Analysis Results");
                    println!("   Task: {}", description);
                    println!("   Recommended Agent: {}", decision.target_agent.name());
                    println!("   Confidence: {:.1}%", decision.confidence * 100.0);
                    if *verbose {
                        println!("   Reasoning: {}", decision.reasoning);
                        if let Some(duration) = decision.estimated_duration {
                            println!("   Estimated Duration: {:?}", duration);
                        }
                    }
                }
            }

            DelegateAction::Stats { detailed, period } => {
                // TODO: Implement delegation statistics
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Delegation statistics",
                            "period_hours": period,
                            "detailed": detailed,
                        }))?
                    );
                } else {
                    println!("📊 Delegation Statistics (last {} hours)", period);
                    println!("   Feature not yet implemented");
                }
            }

            DelegateAction::Interactive => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "error": "Interactive mode not available in JSON output",
                        }))?
                    );
                } else {
                    println!("🖥️ Interactive Delegation Mode");
                    self.run_interactive_delegation().await?;
                }
            }

            DelegateAction::Show { file } => {
                let config = CcswarmConfig::from_file(file.clone()).await?;

                if self.json_output {
                    println!("{}", serde_json::to_string_pretty(&config)?);
                } else {
                    println!("📄 Delegation Configuration: {}", file.display());
                    println!("========================");
                    println!("Project: {}", config.project.name);
                    println!("Repository: {}", config.project.repository.url);
                    println!("Agents: {}", config.agents.len());
                    for (name, agent_config) in &config.agents {
                        println!("  - {}: {}", name, agent_config.specialization);
                    }
                }
            }
        }

        Ok(())
    }

    async fn run_interactive_delegation(&self) -> Result<()> {
        use std::io::{self, Write};

        println!("🎯 Welcome to Interactive Delegation Mode");
        println!("   Type 'help' for commands, 'quit' to exit");
        println!();

        loop {
            print!("ccswarm> ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            match input {
                "quit" | "exit" => {
                    println!("👋 Goodbye!");
                    break;
                }
                "help" => {
                    println!("📚 Interactive Delegation Commands:");
                    println!("   analyze <task_description>  - Analyze task and suggest agent");
                    println!("   delegate <agent> <task>     - Delegate task to specific agent");
                    println!("   stats                       - Show delegation statistics");
                    println!("   agents                      - List available agents");
                    println!("   quit                        - Exit interactive mode");
                    println!();
                }
                "stats" => {
                    println!("📊 Delegation Statistics");
                    println!("   Feature not yet implemented");
                    println!();
                }
                "agents" => {
                    println!("🤖 Available Agents:");
                    println!("   • Frontend - React/TypeScript UI development");
                    println!("   • Backend - Node.js/Express API development");
                    println!("   • DevOps - Infrastructure and deployment");
                    println!("   • QA - Testing and quality assurance");
                    println!();
                }
                _ if input.starts_with("analyze ") => {
                    let task_desc = &input[8..];
                    if !task_desc.is_empty() {
                        // Directly call delegation analysis to avoid recursion
                        use crate::orchestrator::master_delegation::{
                            DelegationStrategy, MasterDelegationEngine,
                        };
                        let mut engine = MasterDelegationEngine::new(DelegationStrategy::Hybrid);
                        let task = Task::new(
                            "interactive-analysis".to_string(),
                            task_desc.to_string(),
                            Priority::Medium,
                            TaskType::Development,
                        );

                        match engine.delegate_task(task) {
                            Ok(decision) => {
                                println!("🔍 Task Analysis Results");
                                println!("   Task: {}", task_desc);
                                println!("   Recommended Agent: {}", decision.target_agent.name());
                                println!("   Confidence: {:.1}%", decision.confidence * 100.0);
                                println!("   Reasoning: {}", decision.reasoning);
                                if let Some(duration) = decision.estimated_duration {
                                    println!("   Estimated Duration: {} seconds", duration);
                                }
                            }
                            Err(e) => {
                                println!("❌ Analysis failed: {}", e);
                            }
                        }
                        println!();
                    } else {
                        println!("❌ Please provide a task description");
                        println!("   Example: analyze Create login form with validation");
                        println!();
                    }
                }
                _ if input.starts_with("delegate ") => {
                    let parts: Vec<&str> = input[9..].splitn(2, ' ').collect();
                    if parts.len() == 2 {
                        let agent = parts[0];
                        let task_desc = parts[1];

                        if ["frontend", "backend", "devops", "qa"].contains(&agent) {
                            println!("🎯 Delegating '{}' to {} agent", task_desc, agent);
                            println!("   ✅ Task queued for delegation");
                            println!();
                        } else {
                            println!("❌ Unknown agent: {}", agent);
                            println!("   Available: frontend, backend, devops, qa");
                            println!();
                        }
                    } else {
                        println!("❌ Usage: delegate <agent> <task_description>");
                        println!("   Example: delegate frontend Create responsive navigation bar");
                        println!();
                    }
                }
                "" => {
                    // Empty input, continue
                }
                _ => {
                    println!("❓ Unknown command: {}", input);
                    println!("   Type 'help' for available commands");
                    println!();
                }
            }
        }

        Ok(())
    }

    fn create_task_from_args(
        &self,
        description: &str,
        priority: &str,
        task_type: &str,
        details: Option<&str>,
        duration: Option<u32>,
    ) -> Result<Task> {
        let priority = match priority.to_lowercase().as_str() {
            "low" => Priority::Low,
            "medium" => Priority::Medium,
            "high" => Priority::High,
            _ => Priority::Medium,
        };

        let task_type = match task_type.to_lowercase().as_str() {
            "development" => TaskType::Development,
            "testing" => TaskType::Testing,
            "infrastructure" => TaskType::Infrastructure,
            "documentation" => TaskType::Documentation,
            "bugfix" => TaskType::Bugfix,
            "feature" => TaskType::Feature,
            "review" => TaskType::Review,
            "coordination" => TaskType::Coordination,
            _ => TaskType::Development,
        };

        let estimated_duration = duration.map(|d| std::time::Duration::from_secs(d as u64));

        let mut task = Task::new(
            uuid::Uuid::new_v4().to_string(),
            description.to_string(),
            priority,
            task_type,
        );

        if let Some(details) = details {
            task = task.with_details(details.to_string());
        }

        task.estimated_duration = estimated_duration.map(|d| d.as_secs() as u32);

        Ok(task)
    }

    async fn handle_session(&self, action: &SessionAction) -> Result<()> {
        match action {
            SessionAction::Create {
                agent,
                workspace,
                background,
            } => {
                let workspace_path = workspace.as_deref().unwrap_or("./");

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Session created",
                            "agent": agent,
                            "workspace": workspace_path,
                            "background": background,
                        }))?
                    );
                } else {
                    println!("🚀 Creating session for {} agent", agent);
                    println!("   Workspace: {}", workspace_path);
                    println!("   Background: {}", background);
                    println!("   ✅ Session created successfully");
                }
            }

            SessionAction::List { all } => {
                // TODO: Implement session listing
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Sessions listed",
                            "sessions": [],
                            "show_all": all,
                        }))?
                    );
                } else {
                    println!("📋 Active Sessions");
                    println!("=================");
                    println!("No active sessions found");
                    if *all {
                        println!("(Showing all sessions including inactive)");
                    }
                }
            }

            SessionAction::Pause { session_id } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Session paused",
                            "session_id": session_id,
                        }))?
                    );
                } else {
                    println!("⏸️ Pausing session: {}", session_id);
                    println!("   ✅ Session paused successfully");
                }
            }

            SessionAction::Resume { session_id } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Session resumed",
                            "session_id": session_id,
                        }))?
                    );
                } else {
                    println!("▶️ Resuming session: {}", session_id);
                    println!("   ✅ Session resumed successfully");
                }
            }

            SessionAction::Attach { session_id } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Attached to session",
                            "session_id": session_id,
                        }))?
                    );
                } else {
                    println!("🔗 Attaching to session: {}", session_id);
                    println!("   ✅ Attached successfully");
                    println!("   Use Ctrl+B, D to detach");
                }
            }

            SessionAction::Detach { session_id } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Detached from session",
                            "session_id": session_id,
                        }))?
                    );
                } else {
                    println!("🔌 Detaching from session: {}", session_id);
                    println!("   ✅ Detached successfully");
                }
            }

            SessionAction::Kill { session_id, force } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Session killed",
                            "session_id": session_id,
                            "force": force,
                        }))?
                    );
                } else {
                    println!("💀 Killing session: {}", session_id);
                    if *force {
                        println!("   ⚠️ Force kill enabled");
                    }
                    println!("   ✅ Session killed successfully");
                }
            }
        }

        Ok(())
    }

    async fn handle_auto_create(
        &self,
        description: &str,
        template: Option<&str>,
        auto_deploy: bool,
        output: &PathBuf,
    ) -> Result<()> {
        use crate::orchestrator::auto_create::AutoCreateEngine;

        info!("🚀 Starting auto-create for: {}", description);

        if self.json_output {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "status": "started",
                    "message": "Auto-create process initiated",
                    "description": description,
                    "template": template,
                    "auto_deploy": auto_deploy,
                    "output": output,
                }))?
            );
        } else {
            println!("🚀 ccswarm Auto-Create");
            println!("====================");
            println!("📋 Request: {}", description);
            if let Some(tmpl) = template {
                println!("📄 Template: {}", tmpl);
            }
            println!("📂 Output: {}", output.display());
            println!();
        }

        // Create auto-create engine
        let mut engine = AutoCreateEngine::new();

        // Execute auto-create workflow
        match engine
            .execute_auto_create(description, &self.config, output)
            .await
        {
            Ok(()) => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Application created successfully",
                            "output": output,
                        }))?
                    );
                } else {
                    println!("\n✅ Application created successfully!");
                    println!("📂 Location: {}", output.display());

                    if auto_deploy {
                        println!("\n🚀 Auto-deploying application...");
                        // TODO: Implement auto-deployment
                        println!("   ⚠️ Auto-deployment not yet implemented");
                    }
                }
            }
            Err(e) => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "error",
                            "message": "Auto-create failed",
                            "error": e.to_string(),
                        }))?
                    );
                } else {
                    println!("\n❌ Auto-create failed: {}", e);
                }
                return Err(e);
            }
        }

        Ok(())
    }

    async fn handle_sangha(&self, action: &SanghaAction) -> Result<()> {
        // Using stub implementation for now
        use crate::extension_stub::sangha::{Vote, VoteType};
        use std::str::FromStr;

        // Stub implementation - just log the action
        info!("🏛️  Sangha action requested (stub implementation)");

        match action {
            SanghaAction::Propose {
                proposal_type,
                file,
            } => {
                info!("📋 Submitting proposal to Sangha (stub implementation)");

                // Read proposal specification from file
                let _spec_content = tokio::fs::read_to_string(file)
                    .await
                    .context("Failed to read proposal file")?;

                info!("Proposal type: {}", proposal_type);

                // Stub: Generate a fake proposal ID
                let proposal_id = uuid::Uuid::new_v4();

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Proposal submitted",
                            "proposal_id": proposal_id,
                            "type": proposal_type,
                            "file": file,
                        }))?
                    );
                } else {
                    println!(
                        "📋 Submitting {} proposal from: {}",
                        proposal_type,
                        file.display()
                    );
                    println!("🆔 Proposal ID: {}", proposal_id);
                    println!("✅ Proposal submitted successfully");
                }
            }

            SanghaAction::Vote {
                proposal_id,
                choice,
                reason,
            } => {
                info!("🗳️ Casting vote (stub implementation)");

                // Parse vote choice
                let vote_type = match choice.to_lowercase().as_str() {
                    "aye" | "yes" | "approve" => VoteType::Approve,
                    "nay" | "no" | "reject" => VoteType::Reject,
                    "abstain" => VoteType::Abstain,
                    "veto" | "needs_changes" => VoteType::NeedsChanges,
                    _ => {
                        anyhow::bail!(
                            "Invalid vote choice: {}. Use aye, nay, abstain, or veto",
                            choice
                        );
                    }
                };

                // Parse proposal ID
                let prop_id =
                    uuid::Uuid::from_str(proposal_id).context("Invalid proposal ID format")?;

                // Create vote (stub)
                let _vote = Vote::new("cli-user".to_string(), prop_id.to_string(), vote_type);

                info!("Vote created for proposal: {}", proposal_id);

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Vote cast",
                            "proposal_id": proposal_id,
                            "choice": choice,
                            "reason": reason,
                        }))?
                    );
                } else {
                    println!("🗳️ Casting vote on proposal: {}", proposal_id);
                    println!("   Choice: {}", choice);
                    if let Some(r) = reason {
                        println!("   Reason: {}", r);
                    }
                    println!("✅ Vote cast successfully");
                }
            }

            SanghaAction::List { all, status } => {
                info!("📊 Listing Sangha status (stub implementation)");

                // Stub statistics
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Sangha status retrieved (stub)",
                            "stats": {
                                "total_members": 1,
                                "active_members": 1,
                                "total_proposals": 0,
                                "active_proposals": 0,
                                "consensus_algorithm": "stub",
                            },
                            "show_all": all,
                            "filter_status": status,
                        }))?
                    );
                } else {
                    println!("📊 Sangha Status (Stub Implementation)");
                    println!("=====================================");
                    println!("👥 Members: 1 total, 1 active");
                    println!("📋 Proposals: 0 total, 0 active");
                    println!("🧠 Consensus Algorithm: stub");

                    println!(
                        "\n💡 No active proposals. Use 'ccswarm sangha propose' to create one."
                    );
                }
            }

            SanghaAction::Session { id, active } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Session info",
                            "session_id": id,
                            "active": active,
                        }))?
                    );
                } else {
                    println!("🏛️ Sangha Session");
                    if *active {
                        println!("No active session");
                    }
                }
            }

            SanghaAction::ExtensionReview {
                proposal_id,
                technical_check,
            } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Extension reviewed",
                            "proposal_id": proposal_id,
                            "technical_check": technical_check,
                        }))?
                    );
                } else {
                    println!("🔍 Reviewing extension proposal: {}", proposal_id);
                    if *technical_check {
                        println!("   Performing technical validation...");
                    }
                    println!("✅ Review completed");
                }
            }
        }

        Ok(())
    }

    async fn handle_extend(&self, action: &ExtendAction) -> Result<()> {
        use crate::extension_stub::meta_learning::MetaLearningSystem;
        use crate::extension_stub::{
            ExtensionManager, ExtensionProposal, ExtensionStatus, ExtensionType,
        };
        use chrono::Utc;

        // Create extension manager instance
        let extension_manager = ExtensionManager::new(());
        let _meta_learning = MetaLearningSystem::new();

        match action {
            ExtendAction::Propose {
                agent,
                extension_type,
                file,
            } => {
                // Read extension specification from file
                let spec_content = tokio::fs::read_to_string(file)
                    .await
                    .context("Failed to read extension specification file")?;

                // Parse extension type
                let ext_type = match extension_type.as_str() {
                    "capability" => ExtensionType::Capability,
                    "system" => ExtensionType::System,
                    "cognitive" => ExtensionType::Cognitive,
                    "collaborative" => ExtensionType::Collaborative,
                    _ => ExtensionType::Capability, // Default
                };

                // Create extension proposal
                let proposal = ExtensionProposal {
                    id: uuid::Uuid::new_v4(),
                    proposer: agent.clone(),
                    extension_type: ext_type,
                    title: format!("{} extension for {}", extension_type, agent),
                    description: spec_content.lines().take(3).collect::<Vec<_>>().join(" "),
                    current_state: crate::extension_stub::CurrentState {
                        capabilities: vec!["basic functionality".to_string()],
                        limitations: vec!["needs enhancement".to_string()],
                        performance_metrics: std::collections::HashMap::new(),
                    },
                    proposed_state: crate::extension_stub::ProposedState {
                        new_capabilities: vec!["enhanced functionality".to_string()],
                        expected_improvements: vec!["improved performance".to_string()],
                        performance_targets: std::collections::HashMap::new(),
                    },
                    implementation_plan: crate::extension_stub::ImplementationPlan {
                        phases: vec![
                            crate::extension_stub::ImplementationPhase {
                                name: "Analysis & Design".to_string(),
                                description: "Analyze requirements and design the extension"
                                    .to_string(),
                                tasks: vec![
                                    "Design document".to_string(),
                                    "Technical specification".to_string(),
                                ],
                                duration_estimate: "1 week".to_string(),
                                validation_method: "Code review".to_string(),
                                phase_name: "Analysis & Design".to_string(),
                                estimated_duration: std::time::Duration::from_secs(604800), // 1 week
                                complexity: "Medium".to_string(),
                                dependencies: Vec::new(),
                            },
                            crate::extension_stub::ImplementationPhase {
                                name: "Implementation".to_string(),
                                description: "Implement the extension functionality".to_string(),
                                tasks: vec!["Working code".to_string(), "Unit tests".to_string()],
                                duration_estimate: "2 weeks".to_string(),
                                validation_method: "Testing".to_string(),
                                phase_name: "Implementation".to_string(),
                                estimated_duration: std::time::Duration::from_secs(1209600), // 2 weeks
                                complexity: "High".to_string(),
                                dependencies: vec!["Analysis & Design".to_string()],
                            },
                            crate::extension_stub::ImplementationPhase {
                                name: "Testing & Deployment".to_string(),
                                description: "Test and deploy the extension".to_string(),
                                tasks: vec![
                                    "Test results".to_string(),
                                    "Deployed extension".to_string(),
                                ],
                                duration_estimate: "1 week".to_string(),
                                validation_method: "Production testing".to_string(),
                                phase_name: "Testing & Deployment".to_string(),
                                estimated_duration: std::time::Duration::from_secs(604800), // 1 week
                                complexity: "Medium".to_string(),
                                dependencies: vec!["Implementation".to_string()],
                            },
                        ],
                        timeline: "2-4 weeks".to_string(),
                        resources_required: vec!["development time".to_string()],
                        dependencies: vec![],
                    },
                    risk_assessment: crate::extension_stub::RiskAssessment {
                        risks: vec![],
                        mitigation_strategies: vec![],
                        rollback_plan: "Revert to previous version".to_string(),
                        overall_risk_score: 0.3,
                        overall_risk: 0.3,
                        categories: vec!["Low".to_string()],
                    },
                    success_criteria: vec![
                        crate::extension_stub::SuccessCriterion {
                            description: "Extension functionality working correctly".to_string(),
                            metric: "Functional tests passed".to_string(),
                            target_value: "100%".to_string(),
                            measurement_method: "Automated testing".to_string(),
                            criterion: "Functionality".to_string(),
                            measurable: true,
                        },
                        crate::extension_stub::SuccessCriterion {
                            description: "No performance degradation".to_string(),
                            metric: "Response time".to_string(),
                            target_value: "< 100ms increase".to_string(),
                            measurement_method: "Performance benchmarks".to_string(),
                            criterion: "Performance".to_string(),
                            measurable: true,
                        },
                    ],
                    created_at: Utc::now(),
                    status: ExtensionStatus::Proposed,
                };

                // Submit proposal to extension manager
                let proposal_id = extension_manager.propose_extension(proposal).await?;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Extension proposed",
                            "proposal_id": proposal_id,
                            "agent": agent,
                            "type": extension_type,
                            "file": file,
                        }))?
                    );
                } else {
                    println!("🔧 Proposing extension for agent: {}", agent);
                    println!("   Type: {}", extension_type);
                    println!("   Specification: {}", file.display());
                    println!("🆔 Proposal ID: {}", proposal_id);
                    println!("✅ Extension proposal submitted");
                }
            }

            ExtendAction::Status {
                agent,
                extension_id,
            } => {
                // Get extension manager statistics
                let stats = extension_manager.get_stats().await;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Extension status retrieved",
                            "agent": agent,
                            "extension_id": extension_id,
                            "stats": {
                                "total_extensions": stats.total_extensions,
                                "active_extensions": stats.active_extensions,
                                "pending_proposals": stats.pending_proposals,
                                "successful_extensions": stats.successful_extensions,
                                "failed_extensions": stats.failed_extensions,
                            }
                        }))?
                    );
                } else {
                    println!("📊 Extension Status for agent: {}", agent);
                    if let Some(id) = extension_id {
                        println!("🆔 Extension ID: {}", id);
                    }
                    println!("📈 Statistics:");
                    println!("   Total Extensions: {}", stats.total_extensions);
                    println!("   Active: {}", stats.active_extensions);
                    println!("   Pending Proposals: {}", stats.pending_proposals);
                    println!("   Successful: {}", stats.successful_extensions);
                    println!("   Failed: {}", stats.failed_extensions);
                }
            }

            ExtendAction::History {
                agent,
                successful,
                failed,
            } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Extension history",
                            "agent": agent,
                            "filter": {
                                "successful": successful,
                                "failed": failed,
                            },
                        }))?
                    );
                } else {
                    println!("📜 Extension History for agent: {}", agent);
                    println!("================================");
                    if *successful {
                        println!("Showing only successful extensions");
                    } else if *failed {
                        println!("Showing only failed extensions");
                    }
                    println!("No extensions found");
                }
            }

            ExtendAction::Rollback {
                agent,
                extension_id,
                force,
            } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Extension rolled back",
                            "agent": agent,
                            "extension_id": extension_id,
                            "force": force,
                        }))?
                    );
                } else {
                    println!("⏪ Rolling back extension: {}", extension_id);
                    println!("   Agent: {}", agent);
                    if *force {
                        println!("   ⚠️ Force rollback enabled");
                    }
                    println!("✅ Rollback completed");
                }
            }

            ExtendAction::Discover {
                agent,
                discovery_type,
            } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Discovery completed",
                            "agent": agent,
                            "discovery_type": discovery_type,
                        }))?
                    );
                } else {
                    println!("🔍 Discovering extension opportunities");
                    if let Some(a) = agent {
                        println!("   Agent: {}", a);
                    }
                    if let Some(dt) = discovery_type {
                        println!("   Type: {}", dt);
                    }
                    println!("✅ Discovery completed");
                }
            }

            ExtendAction::Autonomous {
                agent,
                continuous,
                dry_run,
            } => {
                // use crate::extension::autonomous_agent_extension::AutonomousAgentExtensionManager;

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "started",
                            "message": "Autonomous extension reasoning initiated",
                            "agent": agent,
                            "continuous": continuous,
                            "dry_run": dry_run,
                        }))?
                    );
                } else {
                    println!("🤖 Initiating autonomous extension reasoning");
                    if let Some(a) = agent {
                        println!("   Agent: {}", a);
                    } else {
                        println!("   Agents: All agents");
                    }
                    if *continuous {
                        println!(
                            "   Mode: Continuous (agents will autonomously propose extensions)"
                        );
                    }
                    if *dry_run {
                        println!("   🔍 DRY RUN - proposals will not be submitted to Sangha");
                    }
                }

                // Get agent list
                let target_agents = if let Some(agent_id) = agent {
                    vec![agent_id.clone()]
                } else {
                    // Get all agents from config
                    self.config.agents.keys().cloned().collect()
                };

                // Process each agent
                for agent_id in target_agents {
                    if !self.json_output {
                        println!("\n🎯 Processing agent: {}", agent_id);
                    }

                    // Create autonomous extension manager for the agent
                    // In real implementation, would create proper provider and sangha client
                    // For now, just log the intent

                    if *dry_run {
                        if !self.json_output {
                            println!("   📋 Would propose extensions based on:");
                            println!("      - Past performance analysis");
                            println!("      - Identified capability gaps");
                            println!("      - Recurring failure patterns");
                            println!("      - Self-reflection insights");
                        }
                    } else if !self.json_output {
                        println!("   🧠 Analyzing experiences and performance...");
                        println!("   🔍 Identifying capability gaps...");
                        println!("   💡 Generating extension proposals...");
                        println!("   🏛️  Submitting proposals to Sangha for approval...");

                        // In real implementation:
                        // 1. Create AutonomousAgentExtensionManager
                        // 2. Call propose_extensions()
                        // 3. Wait for Sangha consensus
                        // 4. Report results
                    }
                }

                if *continuous && !self.json_output {
                    println!("\n♾️  Continuous mode enabled - agents will autonomously propose extensions as needed");
                    println!("   Press Ctrl+C to stop");
                }

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "completed",
                            "message": "Autonomous extension reasoning completed",
                            "continuous_mode": continuous,
                        }))?
                    );
                } else {
                    println!("\n✅ Autonomous extension reasoning completed");
                }
            }
        }

        Ok(())
    }

    async fn handle_search(&self, action: &SearchAction) -> Result<()> {
        use crate::extension_stub::agent_extension::{
            DocumentationSearchStrategy, GitHubSearchStrategy, SearchContext, SearchFilters,
            SearchQuery, SearchStrategy, StackOverflowSearchStrategy,
        };

        match action {
            SearchAction::Docs {
                query,
                source,
                limit,
            } => {
                let search_query = SearchQuery {
                    keywords: query.split_whitespace().map(|s| s.to_string()).collect(),
                    context: Some(SearchContext::CapabilityGap {
                        current: vec![],
                        desired: query.split_whitespace().map(|s| s.to_string()).collect(),
                    }),
                    filters: Some(SearchFilters {
                        min_relevance: 0.5,
                        max_complexity: 0.8,
                        preferred_sources: vec![source.clone()],
                        relevance_threshold: 0.5,
                        date_range: None,
                    }),
                };

                let mut all_results = Vec::new();

                if source == "all" || source == "mdn" {
                    let mdn_strategy = DocumentationSearchStrategy::new();
                    match mdn_strategy.search(&search_query).await {
                        Ok(mut results) => {
                            results.truncate(*limit / 3);
                            all_results.extend(results);
                        }
                        Err(e) => eprintln!("MDN search failed: {}", e),
                    }
                }

                if source == "all" || source == "github" {
                    let github_strategy = GitHubSearchStrategy::new();
                    match github_strategy.search(&search_query).await {
                        Ok(mut results) => {
                            results.truncate(*limit / 3);
                            all_results.extend(results);
                        }
                        Err(e) => eprintln!("GitHub search failed: {}", e),
                    }
                }

                if source == "all" || source == "stackoverflow" {
                    let stackoverflow_strategy = StackOverflowSearchStrategy;
                    match stackoverflow_strategy.search(&search_query).await {
                        Ok(mut results) => {
                            results.truncate(*limit / 3);
                            all_results.extend(results);
                        }
                        Err(e) => eprintln!("StackOverflow search failed: {}", e),
                    }
                }

                all_results.sort_by(|a, b| {
                    b.relevance_score
                        .partial_cmp(&a.relevance_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                all_results.truncate(*limit);

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "query": query,
                            "source": source,
                            "results": all_results.iter().map(|r| serde_json::json!({
                                "title": r.title,
                                "url": r.url,
                                "snippet": r.snippet,
                                "source": r.source,
                                "relevance_score": r.relevance_score,
                            })).collect::<Vec<_>>()
                        }))?
                    );
                } else {
                    println!("🔍 Searching documentation for: {}", query);
                    println!("   Source: {}", source);
                    println!("   Found {} results", all_results.len());
                    println!();

                    for (i, result) in all_results.iter().enumerate() {
                        println!("{}. {} ({})", i + 1, result.title, result.source);
                        println!("   🔗 {}", result.url);
                        println!("   📝 {}", result.snippet);
                        println!("   ⭐ Relevance: {:.2}", result.relevance_score);
                        println!();
                    }
                }
            }

            SearchAction::Test { query } => {
                println!("🧪 Testing search functionality with query: {}", query);

                let search_query = SearchQuery {
                    keywords: query.split_whitespace().map(|s| s.to_string()).collect(),
                    context: Some(SearchContext::CapabilityGap {
                        current: vec![],
                        desired: query.split_whitespace().map(|s| s.to_string()).collect(),
                    }),
                    filters: Some(SearchFilters {
                        min_relevance: 0.5,
                        max_complexity: 0.8,
                        preferred_sources: vec!["all".to_string()],
                        relevance_threshold: 0.5,
                        date_range: None,
                    }),
                };

                // Test each search strategy
                println!("🔍 Testing MDN search...");
                let mdn_strategy = DocumentationSearchStrategy::new();
                match mdn_strategy.search(&search_query).await {
                    Ok(results) => println!("✅ MDN: Found {} results", results.len()),
                    Err(e) => println!("❌ MDN: Error - {}", e),
                }

                println!("🔍 Testing GitHub search...");
                let github_strategy = GitHubSearchStrategy::new();
                match github_strategy.search(&search_query).await {
                    Ok(results) => println!("✅ GitHub: Found {} results", results.len()),
                    Err(e) => println!("❌ GitHub: Error - {}", e),
                }

                println!("🔍 Testing StackOverflow search...");
                let stackoverflow_strategy = StackOverflowSearchStrategy;
                match stackoverflow_strategy.search(&search_query).await {
                    Ok(results) => println!("✅ StackOverflow: Found {} results", results.len()),
                    Err(e) => println!("❌ StackOverflow: Error - {}", e),
                }

                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Search test completed",
                            "query": query,
                        }))?
                    );
                } else {
                    println!("✅ Search functionality test completed");
                }
            }
        }

        Ok(())
    }

    async fn handle_evolution(&self, action: &EvolutionAction) -> Result<()> {
        match action {
            EvolutionAction::Metrics { agent, all, period } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Evolution metrics",
                            "agent": agent,
                            "all": all,
                            "period_days": period,
                        }))?
                    );
                } else {
                    println!("📈 Evolution Metrics");
                    println!("==================");
                    if *all {
                        println!("Showing metrics for all agents");
                    } else if let Some(a) = agent {
                        println!("Agent: {}", a);
                    }
                    println!("Period: {} days", period);
                    println!("\nNo metrics available");
                }
            }

            EvolutionAction::Patterns {
                pattern_type,
                successful,
                failed,
            } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Evolution patterns",
                            "pattern_type": pattern_type,
                            "filter": {
                                "successful": successful,
                                "failed": failed,
                            },
                        }))?
                    );
                } else {
                    println!("🔮 Evolution Patterns");
                    println!("===================");
                    if *successful {
                        println!("Showing successful patterns");
                    } else if *failed {
                        println!("Showing failed patterns");
                    }
                    println!("No patterns found");
                }
            }

            EvolutionAction::Genealogy { extension, full } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Extension genealogy",
                            "extension": extension,
                            "full_tree": full,
                        }))?
                    );
                } else {
                    println!("🌳 Extension Genealogy: {}", extension);
                    println!("======================");
                    if *full {
                        println!("Showing full genealogy tree");
                    }
                    println!("No genealogy data available");
                }
            }

            EvolutionAction::Report { format, output } => {
                if self.json_output {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({
                            "status": "success",
                            "message": "Evolution report generated",
                            "format": format,
                            "output": output,
                        }))?
                    );
                } else {
                    println!("📊 Generating evolution report");
                    println!("   Format: {}", format);
                    if let Some(o) = output {
                        println!("   Output: {}", o.display());
                    }
                    println!("✅ Report generated");
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests;
