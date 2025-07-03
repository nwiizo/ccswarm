//! Init command handler

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;

use crate::cli::command_handler::CommandHandler;
use crate::cli::CliRunner;
use crate::config::CcswarmConfig;

pub struct InitCommand {
    pub name: String,
    pub repo_url: Option<String>,
    pub agents: Vec<String>,
}

#[async_trait]
impl CommandHandler for InitCommand {
    async fn execute(&self, runner: &CliRunner) -> Result<()> {
        println!("Initializing ccswarm project: {}", self.name);
        
        // Create configuration
        let config = CcswarmConfig::default();
        
        // Save configuration
        let config_path = runner.repo_path.join("ccswarm.json");
        config.to_file(config_path.clone()).await
            .context("Failed to save configuration")?;
        
        let success_msg = runner.formatter.format_success(
            &format!("Project '{}' initialized successfully at {}", self.name, config_path.display()),
            None
        );
        println!("{}", success_msg);
        
        // Show next steps
        println!("\nNext steps:");
        println!("  1. Configure agents in ccswarm.json");
        println!("  2. Run 'ccswarm start' to begin orchestration");
        println!("  3. Use 'ccswarm task' to create tasks");
        
        Ok(())
    }
    
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(anyhow!("Project name cannot be empty"));
        }
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "init"
    }
}