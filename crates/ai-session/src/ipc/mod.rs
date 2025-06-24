//! Native IPC implementation using Unix domain sockets

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use uuid::Uuid;

/// IPC message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcMessage {
    /// Message ID
    pub id: String,
    /// Message type
    pub msg_type: IpcMessageType,
    /// Payload
    pub payload: serde_json::Value,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// IPC message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpcMessageType {
    /// Create session
    CreateSession,
    /// Execute command
    ExecuteCommand,
    /// Get output
    GetOutput,
    /// Get status
    GetStatus,
    /// List sessions
    ListSessions,
    /// Delete session
    DeleteSession,
    /// Response
    Response,
    /// Error
    Error,
    /// Event notification
    Event,
}

/// IPC server for native communication
pub struct IpcServer {
    /// Socket path
    socket_path: PathBuf,
    /// Session manager
    session_manager: Arc<crate::SessionManager>,
}

impl IpcServer {
    /// Create new IPC server
    pub fn new(socket_path: PathBuf, session_manager: Arc<crate::SessionManager>) -> Self {
        Self {
            socket_path,
            session_manager,
        }
    }

    /// Start the IPC server
    pub async fn start(&self) -> Result<()> {
        // Remove existing socket if it exists
        if self.socket_path.exists() {
            tokio::fs::remove_file(&self.socket_path).await?;
        }

        // Create parent directory if needed
        if let Some(parent) = self.socket_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Bind to socket
        let listener = UnixListener::bind(&self.socket_path)?;
        log::info!("IPC server listening on {:?}", self.socket_path);

        loop {
            let (stream, _) = listener.accept().await?;
            let session_manager = self.session_manager.clone();

            // Handle connection in separate task
            tokio::spawn(async move {
                if let Err(e) = handle_client(stream, session_manager).await {
                    log::error!("Client handler error: {}", e);
                }
            });
        }
    }
}

/// Handle client connection
async fn handle_client(
    stream: UnixStream,
    session_manager: Arc<crate::SessionManager>,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                // Parse message
                let msg: IpcMessage = match serde_json::from_str(&line) {
                    Ok(msg) => msg,
                    Err(e) => {
                        let error_response = IpcMessage {
                            id: Uuid::new_v4().to_string(),
                            msg_type: IpcMessageType::Error,
                            payload: serde_json::json!({
                                "error": format!("Invalid message format: {}", e)
                            }),
                            timestamp: chrono::Utc::now(),
                        };
                        writer
                            .write_all(serde_json::to_string(&error_response)?.as_bytes())
                            .await?;
                        writer.write_all(b"\n").await?;
                        writer.flush().await?;
                        continue;
                    }
                };

                // Process message
                let response = process_message(msg, &session_manager).await?;

                // Send response
                writer
                    .write_all(serde_json::to_string(&response)?.as_bytes())
                    .await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
            }
            Err(e) => {
                log::error!("Read error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Process IPC message
async fn process_message(
    msg: IpcMessage,
    session_manager: &Arc<crate::SessionManager>,
) -> Result<IpcMessage> {
    match msg.msg_type {
        IpcMessageType::CreateSession => {
            let ai_features = msg.payload["enable_ai_features"].as_bool().unwrap_or(false);

            let mut config = crate::core::SessionConfig::default();
            config.enable_ai_features = ai_features;

            let session = session_manager.create_session_with_config(config).await?;

            Ok(IpcMessage {
                id: msg.id,
                msg_type: IpcMessageType::Response,
                payload: serde_json::json!({
                    "success": true,
                    "session_id": session.id.to_string(),
                }),
                timestamp: chrono::Utc::now(),
            })
        }

        IpcMessageType::ExecuteCommand => {
            let session_id = msg.payload["session"].as_str().unwrap_or("");
            let command = msg.payload["command"].as_str().unwrap_or("");

            let session_id = crate::core::SessionId::parse_str(session_id)?;

            if let Some(session) = session_manager.get_session(&session_id) {
                session.send_input(&format!("{}\n", command)).await?;

                Ok(IpcMessage {
                    id: msg.id,
                    msg_type: IpcMessageType::Response,
                    payload: serde_json::json!({
                        "success": true,
                    }),
                    timestamp: chrono::Utc::now(),
                })
            } else {
                Ok(IpcMessage {
                    id: msg.id,
                    msg_type: IpcMessageType::Error,
                    payload: serde_json::json!({
                        "error": "Session not found"
                    }),
                    timestamp: chrono::Utc::now(),
                })
            }
        }

        IpcMessageType::GetOutput => {
            let session_id = msg.payload["session"].as_str().unwrap_or("");
            let last_n = msg.payload["last_n"].as_u64().unwrap_or(100) as usize;

            let session_id = crate::core::SessionId::parse_str(session_id)?;

            if let Some(session) = session_manager.get_session(&session_id) {
                let output = session.read_output().await?;
                let output_str = String::from_utf8_lossy(&output);
                let all_lines: Vec<&str> = output_str.lines().collect();
                let lines: Vec<String> = all_lines
                    .iter()
                    .rev()
                    .take(last_n)
                    .rev()
                    .map(|s| s.to_string())
                    .collect();

                Ok(IpcMessage {
                    id: msg.id,
                    msg_type: IpcMessageType::Response,
                    payload: serde_json::json!({
                        "output": lines,
                    }),
                    timestamp: chrono::Utc::now(),
                })
            } else {
                Ok(IpcMessage {
                    id: msg.id,
                    msg_type: IpcMessageType::Error,
                    payload: serde_json::json!({
                        "error": "Session not found"
                    }),
                    timestamp: chrono::Utc::now(),
                })
            }
        }

        IpcMessageType::GetStatus => {
            let session_id = msg.payload["session"].as_str().unwrap_or("");
            let session_id = crate::core::SessionId::parse_str(session_id)?;

            if let Some(session) = session_manager.get_session(&session_id) {
                let status = session.status().await;
                Ok(IpcMessage {
                    id: msg.id,
                    msg_type: IpcMessageType::Response,
                    payload: serde_json::to_value(status)?,
                    timestamp: chrono::Utc::now(),
                })
            } else {
                Ok(IpcMessage {
                    id: msg.id,
                    msg_type: IpcMessageType::Error,
                    payload: serde_json::json!({
                        "error": "Session not found"
                    }),
                    timestamp: chrono::Utc::now(),
                })
            }
        }

        IpcMessageType::ListSessions => {
            let sessions = session_manager.list_sessions();
            let session_ids: Vec<String> = sessions.iter().map(|id| id.to_string()).collect();
            Ok(IpcMessage {
                id: msg.id,
                msg_type: IpcMessageType::Response,
                payload: serde_json::json!({
                    "sessions": session_ids,
                }),
                timestamp: chrono::Utc::now(),
            })
        }

        IpcMessageType::DeleteSession => {
            let session_id = msg.payload["session"].as_str().unwrap_or("");
            let session_id = crate::core::SessionId::parse_str(session_id)?;

            session_manager.remove_session(&session_id).await?;

            Ok(IpcMessage {
                id: msg.id,
                msg_type: IpcMessageType::Response,
                payload: serde_json::json!({
                    "success": true,
                }),
                timestamp: chrono::Utc::now(),
            })
        }

        _ => Ok(IpcMessage {
            id: msg.id,
            msg_type: IpcMessageType::Error,
            payload: serde_json::json!({
                "error": "Unsupported message type"
            }),
            timestamp: chrono::Utc::now(),
        }),
    }
}

/// IPC client for native communication
pub struct IpcClient {
    /// Socket path
    socket_path: PathBuf,
}

impl IpcClient {
    /// Create new IPC client
    pub fn new(socket_path: PathBuf) -> Self {
        Self { socket_path }
    }

    /// Send message and get response
    pub async fn send_message(&self, msg: IpcMessage) -> Result<IpcMessage> {
        let stream = UnixStream::connect(&self.socket_path).await?;
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send message
        writer
            .write_all(serde_json::to_string(&msg)?.as_bytes())
            .await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        // Read response
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: IpcMessage = serde_json::from_str(&line)?;
        Ok(response)
    }

    /// Create session
    pub async fn create_session(&self, name: &str, enable_ai_features: bool) -> Result<()> {
        let msg = IpcMessage {
            id: Uuid::new_v4().to_string(),
            msg_type: IpcMessageType::CreateSession,
            payload: serde_json::json!({
                "name": name,
                "enable_ai_features": enable_ai_features,
            }),
            timestamp: chrono::Utc::now(),
        };

        self.send_message(msg).await?;
        Ok(())
    }

    /// Execute command
    pub async fn execute_command(&self, session: &str, command: &str) -> Result<()> {
        let msg = IpcMessage {
            id: Uuid::new_v4().to_string(),
            msg_type: IpcMessageType::ExecuteCommand,
            payload: serde_json::json!({
                "session": session,
                "command": command,
            }),
            timestamp: chrono::Utc::now(),
        };

        self.send_message(msg).await?;
        Ok(())
    }

    /// Get output
    pub async fn get_output(&self, session: &str, last_n: usize) -> Result<Vec<String>> {
        let msg = IpcMessage {
            id: Uuid::new_v4().to_string(),
            msg_type: IpcMessageType::GetOutput,
            payload: serde_json::json!({
                "session": session,
                "last_n": last_n,
            }),
            timestamp: chrono::Utc::now(),
        };

        let response = self.send_message(msg).await?;
        let output = response.payload["output"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid output format"))?
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();

        Ok(output)
    }
}

/// Get default socket path
pub fn default_socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("ai-session.sock")
}
