//! ccswarm - AI-powered multi-agent orchestration system

#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::module_inception)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::type_complexity)]
#![allow(clippy::vec_init_then_push)]
#![allow(clippy::let_and_return)]
#![allow(clippy::manual_is_ascii_check)]
#![allow(clippy::option_if_let_else)]

#[cfg(feature = "claude-acp")]
pub mod acp_claude;
pub mod agent;
pub mod auto_accept;
pub mod benchmark;
pub mod cli;
pub mod config;
// Temporarily disabled due to Docker dependency issues
// pub mod container;
pub mod coordination;
pub mod error;
pub mod execution;
pub mod extension;
pub mod extension_stub;
pub mod git;
pub mod hitl;
pub mod hooks;
pub mod identity;
pub mod mcp;
pub mod memory;
pub mod monitoring;
pub mod orchestrator;
pub mod providers;
pub mod resource;
pub mod sangha;
pub mod security;
pub mod semantic;
pub mod session;
pub mod streaming;
pub mod subagent;
pub mod template;
pub mod tmux;
pub mod tracing;
pub mod traits;
pub mod tui;
pub mod utils;
pub mod workflow;
pub mod workspace;
