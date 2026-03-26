//! ccswarm - AI-powered multi-agent orchestration system

#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::module_inception)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::type_complexity)]
#![allow(clippy::option_if_let_else)]

pub mod agent;
pub mod cli;
pub mod config;
pub mod coordination;
pub mod error;
pub mod events;
pub mod git;
pub mod hooks;
pub mod identity;
pub mod resource;
pub mod session;
pub mod utils;
pub mod workflow;
pub mod workspace;
