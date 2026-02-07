//! IPC (Inter-Process Communication) Module
//!
//! Provides HTTP-based IPC for daemon communication. The IPC server enables:
//! - Health checks
//! - Status queries
//! - Task submission
//! - Graceful shutdown
//!
//! This module allows the CLI to communicate with a running ccswarm daemon
//! without requiring the CLI to block waiting for the orchestrator.

pub mod server;

pub use server::{IpcServer, IpcState, start_ipc_server};

use serde::{Deserialize, Serialize};

/// Request to submit a new task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubmitRequest {
    /// Task description
    pub description: String,
    /// Task priority (low, medium, high, critical)
    #[serde(default = "default_priority")]
    pub priority: String,
    /// Task type (development, testing, etc.)
    #[serde(default = "default_task_type")]
    pub task_type: String,
    /// Additional details
    pub details: Option<String>,
}

fn default_priority() -> String {
    "medium".to_string()
}

fn default_task_type() -> String {
    "development".to_string()
}

/// Response from task submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubmitResponse {
    /// Whether the submission was successful
    pub success: bool,
    /// Task ID if successful
    pub task_id: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Uptime in seconds
    pub uptime_secs: u64,
    /// Version
    pub version: String,
}

/// Status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    /// Master ID
    pub master_id: String,
    /// Number of active agents
    pub active_agents: usize,
    /// Pending tasks count
    pub pending_tasks: usize,
    /// Running tasks count
    pub running_tasks: usize,
    /// Completed tasks count
    pub completed_tasks: usize,
    /// Current phase
    pub phase: String,
    /// Uptime in seconds
    pub uptime_secs: u64,
}

/// Shutdown request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownRequest {
    /// Reason for shutdown
    pub reason: Option<String>,
    /// Whether to force shutdown
    #[serde(default)]
    pub force: bool,
}

/// Shutdown response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownResponse {
    /// Whether shutdown was initiated
    pub success: bool,
    /// Message
    pub message: String,
}

/// Default IPC port
pub const DEFAULT_IPC_PORT: u16 = 8080;

/// Default IPC host
pub const DEFAULT_IPC_HOST: &str = "127.0.0.1";
