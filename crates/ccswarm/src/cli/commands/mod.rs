/// Command modules using Rust best practices
///
/// This module uses the command pattern with zero-cost abstractions
/// and type-safe command execution.

use crate::error::Result;
use async_trait::async_trait;
use clap::Args;

/// Zero-cost abstraction for command execution
#[async_trait]
pub trait Command: Args + Send + Sync {
    /// Execute the command with type safety
    async fn execute(self) -> Result<()>;
}

/// Type-state pattern for command validation
pub struct Validated<T>(T);
pub struct Unvalidated<T>(T);

impl<T: Command> Unvalidated<T> {
    pub fn validate(self) -> Result<Validated<T>> {
        // Validation logic here
        Ok(Validated(self.0))
    }
}

impl<T: Command> Validated<T> {
    pub async fn run(self) -> Result<()> {
        self.0.execute().await
    }
}

pub mod init;
pub mod start;
pub mod task;
pub mod agent;
pub mod session;
pub mod sangha;

// Re-export commands for convenience
pub use init::InitCommand;
pub use start::StartCommand;
pub use task::TaskCommand;
pub use agent::AgentCommand;
pub use session::SessionCommand;
pub use sangha::SanghaCommand;