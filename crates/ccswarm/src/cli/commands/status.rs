//! Status command handler

use anyhow::Result;
use async_trait::async_trait;

use crate::cli::command_handler::CommandHandler;
use crate::cli::CliRunner;

pub struct StatusCommand {
    pub detailed: bool,
    pub agent: Option<String>,
}

#[async_trait]
impl CommandHandler for StatusCommand {
    async fn execute(&self, runner: &CliRunner) -> Result<()> {
        // Delegate to existing implementation
        runner.show_status(self.detailed, self.agent.as_deref()).await
    }
    
    fn name(&self) -> &'static str {
        "status"
    }
}