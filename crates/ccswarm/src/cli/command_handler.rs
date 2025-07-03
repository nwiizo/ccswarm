//! Command handler trait and implementations for CLI commands

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use super::CliRunner;

/// Trait for handling CLI commands
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// Execute the command
    async fn execute(&self, runner: &CliRunner) -> Result<()>;
    
    /// Validate command arguments
    fn validate(&self) -> Result<()> {
        Ok(())
    }
    
    /// Get command name for logging
    fn name(&self) -> &'static str;
}

/// Context passed to command handlers
pub struct CommandContext<'a> {
    pub runner: &'a CliRunner,
    pub formatter: &'a super::OutputFormatter,
}

impl<'a> CommandContext<'a> {
    pub fn new(runner: &'a CliRunner) -> Self {
        Self {
            runner,
            formatter: &runner.formatter,
        }
    }
}

/// Macro to implement common command handler boilerplate
#[macro_export]
macro_rules! impl_command_handler {
    ($name:ident, $execute_fn:ident, $cmd_name:literal) => {
        #[async_trait]
        impl CommandHandler for $name {
            async fn execute(&self, runner: &CliRunner) -> Result<()> {
                runner.$execute_fn(self).await
            }
            
            fn name(&self) -> &'static str {
                $cmd_name
            }
        }
    };
}

/// Registry for command handlers
pub struct CommandRegistry {
    handlers: std::collections::HashMap<String, Arc<dyn CommandHandler>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }
    
    pub fn register(&mut self, name: String, handler: Arc<dyn CommandHandler>) {
        self.handlers.insert(name, handler);
    }
    
    pub fn get(&self, name: &str) -> Option<Arc<dyn CommandHandler>> {
        self.handlers.get(name).cloned()
    }
}