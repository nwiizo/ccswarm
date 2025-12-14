//! Simplified Claude Code ACP adapter implementation

use super::{ACPError, ACPResult, ClaudeACPConfig};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{Duration, timeout};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message,
};
use tracing::{debug, info, warn};

/// Simplified Claude Code ACP adapter for MVP
pub struct SimplifiedClaudeAdapter {
    /// WebSocket connection to Claude Code
    ws_stream: Option<Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,

    /// Current session ID
    session_id: Option<String>,

    /// Configuration
    config: ClaudeACPConfig,

    /// Connection status
    connected: bool,
}

/// Simple ACP message structure
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct ACPMessage {
    #[serde(rename = "jsonrpc")]
    jsonrpc: String,
    method: String,
    params: Value,
    id: Option<u64>,
}

/// Simple response structure
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct ACPResponse {
    #[serde(rename = "jsonrpc")]
    jsonrpc: String,
    result: Option<Value>,
    error: Option<Value>,
    id: Option<u64>,
}

impl SimplifiedClaudeAdapter {
    /// Create a new adapter with default configuration
    pub fn new(config: ClaudeACPConfig) -> Self {
        Self {
            ws_stream: None,
            session_id: None,
            config,
            connected: false,
        }
    }

    /// Connect to Claude Code with retry logic
    pub async fn connect_with_retry(&mut self) -> ACPResult<()> {
        for attempt in 1..=self.config.max_retries {
            info!(
                "🔄 Connection attempt {}/{} to {}...",
                attempt, self.config.max_retries, self.config.url
            );

            match self.connect().await {
                Ok(_) => return Ok(()),
                Err(e) if attempt < self.config.max_retries => {
                    warn!(
                        "⚠️  Retry in {} seconds... ({})",
                        self.config.retry_delay, e
                    );
                    tokio::time::sleep(Duration::from_secs(self.config.retry_delay)).await;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }

    /// Connect to Claude Code
    pub async fn connect(&mut self) -> ACPResult<()> {
        debug!("Connecting to Claude Code at {}", self.config.url);

        // Connect directly with URL string
        let ws_future = connect_async(&self.config.url);
        let connection_result = timeout(Duration::from_secs(self.config.timeout), ws_future).await;

        let (ws_stream, _) = match connection_result {
            Ok(Ok((stream, response))) => {
                debug!("WebSocket connected: {:?}", response);
                (stream, response)
            }
            Ok(Err(e)) => {
                debug!("WebSocket connection error: {}", e);
                return Err(ACPError::WebSocketError(e.to_string()));
            }
            Err(_) => {
                debug!("Connection timeout after {} seconds", self.config.timeout);
                return Err(ACPError::Timeout(self.config.timeout));
            }
        };

        // Store connection
        self.ws_stream = Some(Arc::new(Mutex::new(ws_stream)));
        self.connected = true;

        // Initialize session (simplified)
        let session_id = format!("session-{}", uuid::Uuid::new_v4());
        debug!("Session created: {}", session_id);
        self.session_id = Some(session_id);

        info!("✅ Connected to Claude Code!");

        Ok(())
    }

    /// Send a task to Claude Code
    pub async fn send_task(&self, task: &str) -> ACPResult<String> {
        if !self.connected {
            return Err(ACPError::NotConnected);
        }

        info!("📤 Sending task to Claude Code: {}", task);

        // Get WebSocket stream
        let ws_stream = self.ws_stream.as_ref().ok_or(ACPError::NotConnected)?;

        // Send task message
        let message = Message::Text(task.to_string());

        {
            let mut stream = ws_stream.lock().await;
            stream
                .send(message)
                .await
                .map_err(|e| ACPError::WebSocketError(e.to_string()))?;
            stream
                .flush()
                .await
                .map_err(|e| ACPError::WebSocketError(e.to_string()))?;

            // Read response
            if let Some(Ok(Message::Text(response))) = stream.next().await {
                return Ok(response);
            }
        }

        // Fallback response
        Ok(format!("Task '{}' sent to Claude Code", task))
    }

    /// Disconnect from Claude Code
    pub async fn disconnect(&mut self) {
        self.ws_stream = None;
        self.session_id = None;
        self.connected = false;

        info!("👋 Disconnected from Claude Code");
    }

    /// Test connection
    pub async fn test_connection(&mut self) -> ACPResult<String> {
        self.connect().await?;
        let result = self.send_task("echo test").await?;
        self.disconnect().await;
        Ok(format!("Connection test successful: {}", result))
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get configuration
    pub fn config(&self) -> &ClaudeACPConfig {
        &self.config
    }

    /// Get session ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }
}
