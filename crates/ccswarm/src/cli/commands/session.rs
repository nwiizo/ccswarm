//! Session command definitions (stub).
//!
//! The primary session CLI is defined in `cli/mod.rs` as `Commands::Session`
//! with `SessionAction`, and implemented in `cli/handlers/session.rs`.
//! This module preserves the `SessionCommand` type for backward compatibility
//! with the command-trait pattern used by other commands.

use crate::error::Result;
use async_trait::async_trait;
use clap::Args;

/// Session management commands (command-trait wrapper).
///
/// Note: The actual session handling is routed via `Commands::Session`
/// in `cli/mod.rs` and handled by `handlers/session.rs`.
#[derive(Debug, Clone, Args)]
pub struct SessionCommand {
    #[command(subcommand)]
    pub action: SessionCommandAction,
}

/// Legacy session actions for the command-trait pattern.
#[derive(Debug, Clone, clap::Subcommand)]
pub enum SessionCommandAction {
    /// List sessions
    List,
    /// Show session stats
    Stats,
    /// Create new session
    Create { name: String },
}

#[async_trait]
impl super::Command for SessionCommand {
    async fn execute(self) -> Result<()> {
        match self.action {
            SessionCommandAction::List => {
                tracing::info!("Listing sessions — use `ccswarm session list` instead");
            }
            SessionCommandAction::Stats => {
                tracing::info!("Showing session statistics");
            }
            SessionCommandAction::Create { name } => {
                tracing::info!("Creating session: {}", name);
            }
        }
        Ok(())
    }
}
