use crate::error::Result;
use async_trait::async_trait;
use clap::Args;

/// Sangha consensus commands
#[derive(Debug, Clone, Args)]
pub struct SanghaCommand {
    #[command(subcommand)]
    pub action: SanghaAction,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum SanghaAction {
    /// Propose a change
    Propose { title: String, description: String },
    /// Vote on a proposal
    Vote {
        proposal_id: String,
        #[arg(long)]
        approve: bool,
    },
    /// List proposals
    List,
}

#[async_trait]
impl super::Command for SanghaCommand {
    async fn execute(self) -> Result<()> {
        match self.action {
            SanghaAction::Propose { title, description } => {
                tracing::info!("Creating proposal: {} - {}", title, description);
            }
            SanghaAction::Vote {
                proposal_id,
                approve,
            } => {
                let vote = if approve { "approve" } else { "reject" };
                tracing::info!("Voting to {} proposal {}", vote, proposal_id);
            }
            SanghaAction::List => {
                tracing::info!("Listing all proposals");
            }
        }
        Ok(())
    }
}
