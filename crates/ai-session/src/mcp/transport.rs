//! Transport layer for MCP communication

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

#[cfg(feature = "mcp")]
use futures_util::{SinkExt, StreamExt};
#[cfg(feature = "mcp")]
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use super::jsonrpc::JsonRpcMessage;

/// Transport trait for MCP communication
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a JSON-RPC message
    async fn send(&mut self, message: JsonRpcMessage) -> Result<()>;

    /// Receive a JSON-RPC message
    async fn receive(&mut self) -> Result<Option<JsonRpcMessage>>;

    /// Close the transport
    async fn close(&mut self) -> Result<()>;
}

/// Standard I/O transport for local MCP communication
pub struct StdioTransport {
    stdin_reader: BufReader<tokio::io::Stdin>,
    stdout: tokio::io::Stdout,
    shutdown_rx: mpsc::Receiver<()>,
}

impl StdioTransport {
    /// Create a new stdio transport
    pub fn new(shutdown_rx: mpsc::Receiver<()>) -> Self {
        Self {
            stdin_reader: BufReader::new(tokio::io::stdin()),
            stdout: tokio::io::stdout(),
            shutdown_rx,
        }
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn send(&mut self, message: JsonRpcMessage) -> Result<()> {
        let json = serde_json::to_string(&message)?;
        self.stdout.write_all(json.as_bytes()).await?;
        self.stdout.write_all(b"\n").await?;
        self.stdout.flush().await?;
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<JsonRpcMessage>> {
        let mut line = String::new();

        tokio::select! {
            _ = self.shutdown_rx.recv() => {
                return Ok(None);
            }
            result = self.stdin_reader.read_line(&mut line) => {
                match result {
                    Ok(0) => Ok(None), // EOF
                    Ok(_) => {
                        let trimmed = line.trim();
                        if trimmed.is_empty() {
                            return Ok(None);
                        }
                        let message: JsonRpcMessage = serde_json::from_str(trimmed)?;
                        Ok(Some(message))
                    }
                    Err(e) => Err(e.into()),
                }
            }
        }
    }

    async fn close(&mut self) -> Result<()> {
        // Nothing special needed for stdio
        Ok(())
    }
}

#[cfg(feature = "mcp")]
type WebSocketPair = (
    futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        Message,
    >,
    futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
);

/// HTTP/WebSocket transport for remote MCP communication
#[cfg(feature = "mcp")]
pub struct HttpTransport {
    base_url: String,
    client: reqwest::Client,
    websocket: Option<WebSocketPair>,
    is_connected: bool,
}

#[cfg(feature = "mcp")]
impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
            websocket: None,
            is_connected: false,
        }
    }

    /// Connect to the WebSocket endpoint
    async fn connect(&mut self) -> Result<()> {
        if self.is_connected {
            return Ok(());
        }

        // Convert HTTP URL to WebSocket URL
        let ws_url = self
            .base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://")
            + "/mcp";

        let (ws_stream, _) = connect_async(&ws_url)
            .await
            .map_err(|e| anyhow!("Failed to connect to WebSocket: {}", e))?;

        let (sink, stream) = ws_stream.split();
        self.websocket = Some((sink, stream));
        self.is_connected = true;

        Ok(())
    }

    /// Send a JSON-RPC message over HTTP POST
    async fn send_http(&self, message: &JsonRpcMessage) -> Result<JsonRpcMessage> {
        let response = self
            .client
            .post(format!("{}/mcp", self.base_url))
            .header("Content-Type", "application/json")
            .json(message)
            .send()
            .await
            .map_err(|e| anyhow!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "HTTP request failed with status: {}",
                response.status()
            ));
        }

        let response_json: JsonRpcMessage = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse JSON response: {}", e))?;

        Ok(response_json)
    }
}

#[cfg(feature = "mcp")]
#[async_trait]
impl Transport for HttpTransport {
    async fn send(&mut self, message: JsonRpcMessage) -> Result<()> {
        // Try WebSocket first, fall back to HTTP
        if let Some((sink, _)) = &mut self.websocket {
            let json = serde_json::to_string(&message)?;
            sink.send(Message::Text(json))
                .await
                .map_err(|e| anyhow!("WebSocket send failed: {}", e))?;
            Ok(())
        } else {
            // For HTTP, we need to handle request/response pattern
            // Store the message for later retrieval or handle immediately
            match &message {
                JsonRpcMessage::Request { .. } => {
                    let _response = self.send_http(&message).await?;
                    // Response will be handled by receive() method
                    Ok(())
                }
                JsonRpcMessage::Notification { .. } => {
                    // Send notification via HTTP POST without expecting response
                    let _response = self
                        .client
                        .post(format!("{}/mcp/notify", self.base_url))
                        .header("Content-Type", "application/json")
                        .json(&message)
                        .send()
                        .await
                        .map_err(|e| anyhow!("HTTP notification failed: {}", e))?;
                    Ok(())
                }
                JsonRpcMessage::Response { .. } => {
                    // Responses are typically not sent by clients
                    Err(anyhow!("Cannot send response message via HTTP transport"))
                }
            }
        }
    }

    async fn receive(&mut self) -> Result<Option<JsonRpcMessage>> {
        // Ensure connection is established
        if !self.is_connected {
            self.connect().await?;
        }

        if let Some((_, stream)) = &mut self.websocket {
            match stream.next().await {
                Some(Ok(Message::Text(text))) => {
                    let message: JsonRpcMessage = serde_json::from_str(&text)
                        .map_err(|e| anyhow!("Failed to parse WebSocket message: {}", e))?;
                    Ok(Some(message))
                }
                Some(Ok(Message::Close(_))) => {
                    self.is_connected = false;
                    self.websocket = None;
                    Ok(None)
                }
                Some(Ok(_)) => {
                    // Ignore other message types (binary, ping, pong)
                    self.receive().await // Recursively wait for next text message
                }
                Some(Err(e)) => Err(anyhow!("WebSocket error: {}", e)),
                None => {
                    // Stream ended
                    self.is_connected = false;
                    self.websocket = None;
                    Ok(None)
                }
            }
        } else {
            // For HTTP-only mode, we'd need to implement server-sent events or polling
            // For now, return None to indicate no messages available
            Ok(None)
        }
    }

    async fn close(&mut self) -> Result<()> {
        if let Some((mut sink, _)) = self.websocket.take() {
            sink.send(Message::Close(None))
                .await
                .map_err(|e| anyhow!("Failed to close WebSocket: {}", e))?;
        }
        self.is_connected = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stdio_transport_creation() {
        let (_tx, rx) = mpsc::channel(1);
        let _transport = StdioTransport::new(rx);
        // Transport created successfully
    }

    #[cfg(feature = "mcp")]
    #[test]
    fn test_http_transport_creation() {
        let _transport = HttpTransport::new("http://localhost:8080".to_string());
        // Transport created successfully
    }
}
