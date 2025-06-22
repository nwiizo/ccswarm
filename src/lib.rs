//! ccswarm - AI-powered multi-agent orchestration system

#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::module_inception)]
#![allow(clippy::needless_borrows_for_generic_args)]

pub mod agent;
pub mod auto_accept;
pub mod cli;
pub mod config;
// Temporarily disabled due to Docker dependency issues
// pub mod container;
pub mod coordination;
// Temporarily disabled due to compilation issues
// pub mod extension;
pub mod extension_stub;
pub mod git;
pub mod identity;
pub mod monitoring;
pub mod orchestrator;
pub mod providers;
// Temporarily disabled due to compilation issues  
// pub mod sangha;
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
