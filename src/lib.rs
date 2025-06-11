pub mod agent;
pub mod cli;
pub mod config;
pub mod coordination;
pub mod git;
pub mod identity;
pub mod orchestrator;
pub mod tui;
pub mod workspace;

#[cfg(test)]
mod tests;

pub use agent::*;
pub use identity::*;
pub use orchestrator::*;
