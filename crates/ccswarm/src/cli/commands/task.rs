//! Task command handler

use anyhow::Result;
use async_trait::async_trait;

use crate::cli::command_handler::CommandHandler;
use crate::cli::{CliRunner, TaskAction};

pub struct TaskCommand {
    pub action: TaskAction,
}

#[async_trait]
impl CommandHandler for TaskCommand {
    async fn execute(&self, runner: &CliRunner) -> Result<()> {
        // Delegate to existing implementation
        runner.handle_task(&self.action).await
    }
    
    fn name(&self) -> &'static str {
        "task"
    }
}