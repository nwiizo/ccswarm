//! Transport layer for MCP communication

use anyhow::Result;
use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

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

/// HTTP/SSE transport for remote MCP communication (stub for future implementation)
pub struct HttpTransport {
    #[allow(dead_code)]
    base_url: String,
    #[allow(dead_code)]
    client: reqwest::Client,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send(&mut self, _message: JsonRpcMessage) -> Result<()> {
        // TODO: Implement HTTP/SSE transport
        unimplemented!("HTTP transport not yet implemented")
    }

    async fn receive(&mut self) -> Result<Option<JsonRpcMessage>> {
        // TODO: Implement HTTP/SSE transport
        unimplemented!("HTTP transport not yet implemented")
    }

    async fn close(&mut self) -> Result<()> {
        // TODO: Implement HTTP/SSE transport
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

    #[test]
    fn test_http_transport_creation() {
        let _transport = HttpTransport::new("http://localhost:8080".to_string());
        // Transport created successfully
    }
}