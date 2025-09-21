//! Claude Code ACP (Agent Client Protocol) Integration
//!
//! This module provides the main integration with Claude Code through ACP protocol.

pub mod adapter;
pub mod config;
pub mod error;

pub use adapter::SimplifiedClaudeAdapter;
pub use config::ClaudeACPConfig;
pub use error::{ACPError, ACPResult};

use tracing::info;

/// Initialize Claude ACP integration
pub async fn init() -> ACPResult<()> {
    info!("🚀 Initializing Claude Code ACP integration");
    Ok(())
}
