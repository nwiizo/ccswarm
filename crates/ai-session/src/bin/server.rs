//! AI-Session HTTP Server - provides REST API for external command execution via curl

use ai_session::{SessionConfig, SessionManager};
use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;

/// Command line arguments
#[derive(Parser)]
#[command(name = "ai-session-server")]
#[command(about = "AI-Session HTTP server for external command execution")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Host to bind to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

/// Server state shared across requests
#[derive(Clone)]
struct AppState {
    manager: Arc<SessionManager>,
    sessions: Arc<RwLock<HashMap<String, String>>>, // name -> session_id mapping
}

/// Request to create a new session
#[derive(Deserialize)]
struct CreateSessionRequest {
    name: String,
    #[serde(default)]
    enable_ai_features: bool,
    #[serde(default)]
    working_directory: Option<String>,
    #[serde(default)]
    shell: Option<String>,
}

/// Request to execute a command
#[derive(Deserialize)]
struct ExecuteCommandRequest {
    command: String,
    #[serde(default = "default_timeout")]
    timeout_ms: u64,
}

fn default_timeout() -> u64 {
    5000
}

/// Response for session creation
#[derive(Serialize)]
struct SessionResponse {
    id: String,
    name: String,
    status: String,
    created_at: String,
}

/// Response for command execution
#[derive(Serialize)]
struct CommandResponse {
    success: bool,
    output: String,
    error: Option<String>,
    execution_time_ms: u64,
}

/// Response for session listing
#[derive(Serialize)]
struct SessionListResponse {
    sessions: Vec<SessionSummary>,
    total: usize,
}

#[derive(Serialize)]
struct SessionSummary {
    id: String,
    name: String,
    status: String,
    created_at: String,
    last_activity: String,
}

/// Error response
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 Starting AI-Session HTTP Server...");

    // Create application state
    let manager = Arc::new(SessionManager::new());
    let sessions = Arc::new(RwLock::new(HashMap::new()));

    let state = AppState { manager, sessions };

    // Build application routes
    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/sessions", get(list_sessions))
        .route("/sessions", post(create_session))
        .route("/sessions/:name", get(get_session))
        .route("/sessions/:name", delete(delete_session))
        .route("/sessions/:name/execute", post(execute_command))
        .route("/sessions/:name/status", get(get_session_status))
        .route("/sessions/:name/output", get(get_session_output))
        .with_state(state)
        .layer(CorsLayer::permissive());

    // Start server
    let bind_addr = format!("{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    println!("✓ Server listening on http://{}", bind_addr);
    println!("\n📖 API Endpoints:");
    println!("  GET    /health                     - Health check");
    println!("  GET    /sessions                   - List all sessions");
    println!("  POST   /sessions                   - Create new session");
    println!("  GET    /sessions/:name             - Get session details");
    println!("  DELETE /sessions/:name             - Delete session");
    println!("  POST   /sessions/:name/execute     - Execute command");
    println!("  GET    /sessions/:name/status      - Get session status");
    println!("  GET    /sessions/:name/output      - Get session output");

    println!("\n🔧 Example curl commands:");
    println!("  # Create session:");
    println!(
        "  curl -X POST http://{}:{}/sessions \\",
        args.host, args.port
    );
    println!("       -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"name\": \"dev\", \"enable_ai_features\": true}}'");
    println!();
    println!("  # Execute command:");
    println!(
        "  curl -X POST http://{}:{}/sessions/dev/execute \\",
        args.host, args.port
    );
    println!("       -H 'Content-Type: application/json' \\");
    println!("       -d '{{\"command\": \"echo Hello World\"}}'");
    println!();
    println!("  # Get session output:");
    println!(
        "  curl http://{}:{}/sessions/dev/output",
        args.host, args.port
    );

    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "ai-session-server",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// List all sessions
async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<SessionListResponse>, StatusCode> {
    let sessions_map = state.sessions.read().await;
    let session_list = state.manager.list_session_refs();

    let mut sessions = Vec::new();
    for session in session_list {
        // Find the name for this session
        let name = sessions_map
            .iter()
            .find(|(_, id)| **id == session.id.to_string())
            .map(|(name, _)| name.clone())
            .unwrap_or_else(|| session.id.to_string());

        sessions.push(SessionSummary {
            id: session.id.to_string(),
            name,
            status: format!("{:?}", session.status().await),
            created_at: session.created_at.to_rfc3339(),
            last_activity: session.last_activity.read().await.to_rfc3339(),
        });
    }

    Ok(Json(SessionListResponse {
        total: sessions.len(),
        sessions,
    }))
}

/// Create a new session
async fn create_session(
    State(state): State<AppState>,
    Json(req): Json<CreateSessionRequest>,
) -> Result<Json<SessionResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Check if session name already exists
    {
        let sessions = state.sessions.read().await;
        if sessions.contains_key(&req.name) {
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: format!("Session '{}' already exists", req.name),
                    code: "SESSION_EXISTS".to_string(),
                }),
            ));
        }
    }

    // Create session configuration
    let mut config = SessionConfig::default();
    config.enable_ai_features = req.enable_ai_features;

    if let Some(wd) = req.working_directory {
        config.working_directory = std::path::PathBuf::from(wd);
    }

    if let Some(shell) = req.shell {
        config.shell = Some(shell);
    }

    // Create the session
    let session = state
        .manager
        .create_session_with_config(config)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to create session: {}", e),
                    code: "SESSION_CREATION_FAILED".to_string(),
                }),
            )
        })?;

    // Start the session
    session.start().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to start session: {}", e),
                code: "SESSION_START_FAILED".to_string(),
            }),
        )
    })?;

    // Store the name mapping
    {
        let mut sessions = state.sessions.write().await;
        sessions.insert(req.name.clone(), session.id.to_string());
    }

    Ok(Json(SessionResponse {
        id: session.id.to_string(),
        name: req.name,
        status: format!("{:?}", session.status().await),
        created_at: session.created_at.to_rfc3339(),
    }))
}

/// Get session details
async fn get_session(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<SessionResponse>, (StatusCode, Json<ErrorResponse>)> {
    let session = get_session_by_name(&state, &name).await?;

    Ok(Json(SessionResponse {
        id: session.id.to_string(),
        name,
        status: format!("{:?}", session.status().await),
        created_at: session.created_at.to_rfc3339(),
    }))
}

/// Delete a session
async fn delete_session(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let session = get_session_by_name(&state, &name).await?;

    // Stop the session
    session.stop().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to stop session: {}", e),
                code: "SESSION_STOP_FAILED".to_string(),
            }),
        )
    })?;

    // Remove from manager
    state
        .manager
        .remove_session(&session.id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to remove session: {}", e),
                    code: "SESSION_REMOVAL_FAILED".to_string(),
                }),
            )
        })?;

    // Remove from name mapping
    {
        let mut sessions = state.sessions.write().await;
        sessions.remove(&name);
    }

    Ok(Json(serde_json::json!({
        "message": format!("Session '{}' deleted successfully", name),
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Execute a command in a session
async fn execute_command(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(req): Json<ExecuteCommandRequest>,
) -> Result<Json<CommandResponse>, (StatusCode, Json<ErrorResponse>)> {
    let session = get_session_by_name(&state, &name).await?;

    let start_time = std::time::Instant::now();

    // Send command to session
    session
        .send_input(&format!("{}\n", req.command))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to send command: {}", e),
                    code: "COMMAND_SEND_FAILED".to_string(),
                }),
            )
        })?;

    // Wait a bit for command execution
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    // Read output
    let output = session.read_output().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to read output: {}", e),
                code: "OUTPUT_READ_FAILED".to_string(),
            }),
        )
    })?;

    let execution_time = start_time.elapsed().as_millis() as u64;
    let output_str = String::from_utf8_lossy(&output).to_string();

    Ok(Json(CommandResponse {
        success: true,
        output: clean_terminal_output(&output_str),
        error: None,
        execution_time_ms: execution_time,
    }))
}

/// Get session status
async fn get_session_status(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let session = get_session_by_name(&state, &name).await?;

    Ok(Json(serde_json::json!({
        "id": session.id,
        "name": name,
        "status": format!("{:?}", session.status().await),
        "created_at": session.created_at.to_rfc3339(),
        "last_activity": session.last_activity.read().await.to_rfc3339(),
        "config": {
            "enable_ai_features": session.config.enable_ai_features,
            "working_directory": session.config.working_directory.display().to_string(),
            "pty_size": session.config.pty_size
        }
    })))
}

/// Get session output (read latest output buffer)
async fn get_session_output(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<ErrorResponse>)> {
    let session = get_session_by_name(&state, &name).await?;

    let output = session.read_output().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to read output: {}", e),
                code: "OUTPUT_READ_FAILED".to_string(),
            }),
        )
    })?;

    let output_str = String::from_utf8_lossy(&output).to_string();

    Ok(Json(serde_json::json!({
        "session_name": name,
        "output": clean_terminal_output(&output_str),
        "raw_output": output_str,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "size_bytes": output.len()
    })))
}

/// Helper function to get session by name
async fn get_session_by_name(
    state: &AppState,
    name: &str,
) -> Result<std::sync::Arc<ai_session::AISession>, (StatusCode, Json<ErrorResponse>)> {
    // Get session ID from name mapping
    let session_id = {
        let sessions = state.sessions.read().await;
        sessions.get(name).cloned()
    };

    let session_id = session_id.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Session '{}' not found", name),
                code: "SESSION_NOT_FOUND".to_string(),
            }),
        )
    })?;

    // Parse session ID and get session from manager
    let session_id = ai_session::SessionId::parse_str(&session_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid session ID format: {}", session_id),
                code: "INVALID_SESSION_ID".to_string(),
            }),
        )
    })?;

    state.manager.get_session(&session_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Session '{}' not found in manager", name),
                code: "SESSION_NOT_FOUND".to_string(),
            }),
        )
    })
}

/// Clean terminal escape sequences and control characters for display
fn clean_terminal_output(output: &str) -> String {
    // Remove ANSI escape sequences and control characters
    let ansi_escape = regex::Regex::new(r"\x1b\[[0-9;]*[mK]").unwrap();
    let control_chars = regex::Regex::new(r"[\x00-\x1f\x7f]").unwrap();

    let cleaned = ansi_escape.replace_all(output, "");
    let cleaned = control_chars.replace_all(&cleaned, "");

    // Remove empty lines and excessive whitespace
    cleaned
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim())
        .collect::<Vec<_>>()
        .join("\n")
}
