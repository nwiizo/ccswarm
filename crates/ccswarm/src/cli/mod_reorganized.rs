//! CLI module for ccswarm with clippy exceptions for complex conditional patterns

#![allow(clippy::collapsible_else_if)]
#![allow(clippy::get_first)]

// Module declarations
mod command_handler;
mod command_registry;
mod commands;
mod error_help;
mod handlers;
mod health;
mod interactive_help;
mod output;
mod progress;
mod quickstart_simple;
mod resource_commands;
mod setup_wizard;
mod tutorial;

// Public exports
pub use commands::Commands;
pub use interactive_help::{show_quick_help, InteractiveHelp};
pub use output::{create_formatter, OutputFormatter};
pub use progress::{ProcessTracker, ProgressStyle, ProgressTracker, StatusLine};
pub use setup_wizard::SetupWizard;
pub use tutorial::InteractiveTutorial;

// Standard library imports
use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use tracing::warn;

// Internal imports
use crate::config::CcswarmConfig;
use crate::execution::ExecutionEngine;

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

/// Main CLI runner that handles command execution
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
            self::handlers::config::create_default_config(&cli.repo)?
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
}