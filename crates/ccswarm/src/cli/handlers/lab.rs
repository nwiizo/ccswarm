//! `ccswarm lab` — experimental / research commands grouped away from the primary flow.
//! Dispatches to existing sangha/extend/search/evolution handlers.

use super::super::*;

impl CliRunner {
    pub(crate) async fn handle_lab(&self, action: &LabAction) -> Result<()> {
        match action {
            LabAction::Sangha { action } => self.handle_sangha(action).await,
            LabAction::Extend { action } => self.handle_extend(action).await,
            LabAction::Evolution { action } => self.handle_evolution(action).await,
            LabAction::Search { action } => self.handle_search_cmd(action).await,
        }
    }
}
