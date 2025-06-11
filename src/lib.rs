pub mod agent;
pub mod auto_accept;
pub mod cli;
pub mod config;
pub mod coordination;
pub mod git;
pub mod identity;
pub mod monitoring;
pub mod orchestrator;
pub mod providers;
pub mod session;
pub mod streaming;
pub mod tmux;
pub mod tui;
pub mod workspace;

#[cfg(test)]
mod tests;

pub use agent::*;
pub use identity::*;
pub use orchestrator::*;
