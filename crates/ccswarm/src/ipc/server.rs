//! IPC HTTP Server
//!
//! Provides HTTP endpoints for daemon communication using Axum.
//!
//! ## Endpoints
//!
//! - `GET /health` - Health check
//! - `GET /status` - Orchestrator status
//! - `POST /shutdown` - Graceful shutdown
//! - `POST /task` - Submit a new task

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use tokio::sync::{RwLock, mpsc};
use tracing::{error, info};

use super::{
    HealthResponse, ShutdownRequest, ShutdownResponse, StatusResponse, TaskSubmitRequest,
    TaskSubmitResponse,
};
use crate::agent::{Priority, Task, TaskType};
use crate::orchestrator::ProactiveMaster;

/// Shared state for IPC handlers
pub struct IpcState {
    /// Reference to the ProactiveMaster
    pub master: Arc<RwLock<ProactiveMaster>>,
    /// Server start time
    pub start_time: Instant,
    /// Shutdown signal sender
    pub shutdown_tx: mpsc::Sender<()>,
    /// Pending tasks counter
    pub pending_tasks: Arc<RwLock<usize>>,
    /// Running tasks counter
    pub running_tasks: Arc<RwLock<usize>>,
    /// Completed tasks counter
    pub completed_tasks: Arc<RwLock<usize>>,
}

impl IpcState {
    /// Create a new IPC state
    pub fn new(master: Arc<RwLock<ProactiveMaster>>, shutdown_tx: mpsc::Sender<()>) -> Self {
        Self {
            master,
            start_time: Instant::now(),
            shutdown_tx,
            pending_tasks: Arc::new(RwLock::new(0)),
            running_tasks: Arc::new(RwLock::new(0)),
            completed_tasks: Arc::new(RwLock::new(0)),
        }
    }
}

/// IPC Server
pub struct IpcServer {
    /// Socket address to bind to
    addr: SocketAddr,
    /// Shared state
    state: Arc<IpcState>,
}

impl IpcServer {
    /// Create a new IPC server
    pub fn new(host: &str, port: u16, state: IpcState) -> Result<Self> {
        let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
        Ok(Self {
            addr,
            state: Arc::new(state),
        })
    }

    /// Build the router with all endpoints
    fn build_router(&self) -> Router {
        Router::new()
            .route("/health", get(health_handler))
            .route("/status", get(status_handler))
            .route("/shutdown", post(shutdown_handler))
            .route("/task", post(task_handler))
            .with_state(Arc::clone(&self.state))
    }

    /// Start the IPC server
    pub async fn start(self) -> Result<()> {
        let router = self.build_router();

        info!("Starting IPC server on {}", self.addr);

        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(listener, router)
            .await
            .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

        Ok(())
    }

    /// Get the server address
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

/// Health check handler
async fn health_handler(State(state): State<Arc<IpcState>>) -> Json<HealthResponse> {
    let uptime_secs = state.start_time.elapsed().as_secs();

    Json(HealthResponse {
        status: "healthy".to_string(),
        uptime_secs,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Status handler
async fn status_handler(
    State(state): State<Arc<IpcState>>,
) -> Result<Json<StatusResponse>, StatusCode> {
    let master = state.master.read().await;
    let uptime_secs = state.start_time.elapsed().as_secs();

    let pending = *state.pending_tasks.read().await;
    let running = *state.running_tasks.read().await;
    let completed = *state.completed_tasks.read().await;

    Ok(Json(StatusResponse {
        master_id: master.id.clone(),
        active_agents: master.agents.len(),
        pending_tasks: pending,
        running_tasks: running,
        completed_tasks: completed,
        phase: "running".to_string(),
        uptime_secs,
    }))
}

/// Shutdown handler
async fn shutdown_handler(
    State(state): State<Arc<IpcState>>,
    Json(request): Json<ShutdownRequest>,
) -> Json<ShutdownResponse> {
    info!(
        "Shutdown requested: reason={:?}, force={}",
        request.reason, request.force
    );

    // Send shutdown signal
    match state.shutdown_tx.send(()).await {
        Ok(_) => {
            info!("Shutdown signal sent successfully");
            Json(ShutdownResponse {
                success: true,
                message: "Shutdown initiated".to_string(),
            })
        }
        Err(e) => {
            error!("Failed to send shutdown signal: {}", e);
            Json(ShutdownResponse {
                success: false,
                message: format!("Failed to initiate shutdown: {}", e),
            })
        }
    }
}

/// Task submission handler
async fn task_handler(
    State(state): State<Arc<IpcState>>,
    Json(request): Json<TaskSubmitRequest>,
) -> Result<Json<TaskSubmitResponse>, StatusCode> {
    info!("Task submission: {}", request.description);

    // Parse priority
    let priority = match request.priority.to_lowercase().as_str() {
        "low" => Priority::Low,
        "medium" => Priority::Medium,
        "high" => Priority::High,
        "critical" => Priority::Critical,
        _ => Priority::Medium,
    };

    // Parse task type
    let task_type = match request.task_type.to_lowercase().as_str() {
        "development" | "dev" => TaskType::Development,
        "testing" | "test" => TaskType::Testing,
        "documentation" | "docs" => TaskType::Documentation,
        "infrastructure" | "infra" => TaskType::Infrastructure,
        "bugfix" | "bug" => TaskType::Bugfix,
        "feature" => TaskType::Feature,
        _ => TaskType::Development,
    };

    // Create task
    let task_id = uuid::Uuid::new_v4().to_string();
    let mut task = Task::new(
        task_id.clone(),
        request.description.clone(),
        priority,
        task_type,
    );

    if let Some(details) = request.details {
        task = task.with_details(details);
    }

    // Get coordination bus and submit task
    let master = state.master.read().await;
    if let Some(bus) = master.get_coordination_bus() {
        use crate::coordination::AgentMessage;

        let message = AgentMessage::TaskAssignment {
            task_id: task_id.clone(),
            agent_id: "auto".to_string(), // Will be auto-assigned
            task_data: serde_json::to_value(&task).unwrap_or_else(|e| {
                tracing::warn!("Failed to serialize task: {e}");
                serde_json::Value::Null
            }),
        };

        if let Err(e) = bus.send_message(message).await {
            error!("Failed to submit task: {}", e);
            return Ok(Json(TaskSubmitResponse {
                success: false,
                task_id: None,
                error: Some(format!("Failed to submit task: {}", e)),
            }));
        }

        // Increment pending tasks counter
        {
            let mut pending = state.pending_tasks.write().await;
            *pending += 1;
        }

        info!("Task {} submitted successfully", task_id);
        Ok(Json(TaskSubmitResponse {
            success: true,
            task_id: Some(task_id),
            error: None,
        }))
    } else {
        Ok(Json(TaskSubmitResponse {
            success: false,
            task_id: None,
            error: Some("Orchestrator not running".to_string()),
        }))
    }
}

/// Start the IPC server in a background task
///
/// Returns a shutdown sender that can be used to stop the server
pub async fn start_ipc_server(
    host: &str,
    port: u16,
    master: Arc<RwLock<ProactiveMaster>>,
) -> Result<mpsc::Sender<()>> {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    let state = IpcState::new(Arc::clone(&master), shutdown_tx.clone());
    let server = IpcServer::new(host, port, state)?;
    let addr = server.addr;

    // Spawn server task
    tokio::spawn(async move {
        tokio::select! {
            result = server.start() => {
                if let Err(e) = result {
                    error!("IPC server error: {}", e);
                }
            }
            _ = shutdown_rx.recv() => {
                info!("IPC server shutdown signal received");
            }
        }
        info!("IPC server stopped");
    });

    info!("IPC server started on {}", addr);
    Ok(shutdown_tx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            uptime_secs: 100,
            version: "0.4.3".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_task_submit_request_defaults() {
        let json = r#"{"description": "Test task"}"#;
        let request: TaskSubmitRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.description, "Test task");
        assert_eq!(request.priority, "medium");
        assert_eq!(request.task_type, "development");
    }

    #[test]
    fn test_shutdown_request() {
        let json = r#"{"reason": "maintenance", "force": true}"#;
        let request: ShutdownRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.reason, Some("maintenance".to_string()));
        assert!(request.force);
    }
}
