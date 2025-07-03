//! Individual command implementations

pub mod init;
pub mod status;
pub mod task;

use anyhow::Result;
use async_trait::async_trait;
use super::command_handler::CommandHandler;
use super::CliRunner;

/// Base implementation for simple commands
pub struct BaseCommand {
    pub name: &'static str,
    pub handler: fn(&CliRunner) -> Result<()>,
}

#[async_trait]
impl CommandHandler for BaseCommand {
    async fn execute(&self, runner: &CliRunner) -> Result<()> {
        (self.handler)(runner)
    }
    
    fn name(&self) -> &'static str {
        self.name
    }
}