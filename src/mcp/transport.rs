//! Transport layer for MCP client communication

use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tokio::net::UnixStream;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use super::jsonrpc::JsonRpcMessage;

/// Transport trait for MCP communication
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a JSON-RPC message
    async fn send(&mut self, message: &JsonRpcMessage) -> Result<()>;
    
    /// Receive a JSON-RPC message
    async fn receive(&mut self) -> Result<Option<JsonRpcMessage>>;
    
    /// Close the transport
    async fn close(&mut self) -> Result<()>;
}

/// HTTP transport for MCP communication
pub struct HttpTransport {
    base_url: String,
    client: reqwest::Client,
    timeout: Duration,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(base_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
            
        Self {
            base_url,
            client,
            timeout: Duration::from_secs(30),
        }
    }
    
    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn send(&mut self, message: &JsonRpcMessage) -> Result<()> {
        let json_str = message.to_string()?;
        
        let response = self.client
            .post(&self.base_url)
            .header("Content-Type", "application/json")
            .body(json_str)
            .timeout(self.timeout)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "HTTP request failed with status: {}", 
                response.status()
            ));
        }
        
        Ok(())
    }
    
    async fn receive(&mut self) -> Result<Option<JsonRpcMessage>> {
        // For HTTP, we don't have a persistent connection to receive from
        // This would be used in a request-response pattern
        Ok(None)
    }
    
    async fn close(&mut self) -> Result<()> {
        // HTTP connections are stateless
        Ok(())
    }
}

/// Unix domain socket transport for MCP communication
pub struct UnixSocketTransport {
    stream: Option<UnixStream>,
    reader: Option<BufReader<tokio::net::unix::OwnedReadHalf>>,
    writer: Option<tokio::net::unix::OwnedWriteHalf>,
    socket_path: String,
}

impl UnixSocketTransport {
    /// Create a new Unix socket transport
    pub fn new(socket_path: String) -> Self {
        Self {
            stream: None,
            reader: None,
            writer: None,
            socket_path,
        }
    }
    
    /// Connect to the Unix socket
    pub async fn connect(&mut self) -> Result<()> {
        let stream = UnixStream::connect(&self.socket_path).await?;
        let (read_half, write_half) = stream.into_split();
        
        self.reader = Some(BufReader::new(read_half));
        self.writer = Some(write_half);
        
        Ok(())
    }
}

#[async_trait]
impl Transport for UnixSocketTransport {
    async fn send(&mut self, message: &JsonRpcMessage) -> Result<()> {
        let writer = self.writer.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Unix socket not connected"))?;
            
        let json_str = message.to_string()?;
        let message_with_delimiter = format!("{}\n", json_str);
        
        writer.write_all(message_with_delimiter.as_bytes()).await?;
        writer.flush().await?;
        
        Ok(())
    }
    
    async fn receive(&mut self) -> Result<Option<JsonRpcMessage>> {
        let reader = self.reader.as_mut()
            .ok_or_else(|| anyhow::anyhow!("Unix socket not connected"))?;
            
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line).await?;
        
        if bytes_read == 0 {
            // Connection closed
            return Ok(None);
        }
        
        let message = JsonRpcMessage::from_str(line.trim())?;
        Ok(Some(message))
    }
    
    async fn close(&mut self) -> Result<()> {
        if let Some(writer) = self.writer.take() {
            drop(writer);
        }
        if let Some(reader) = self.reader.take() {
            drop(reader);
        }
        self.stream = None;
        
        Ok(())
    }
}

/// In-memory transport for testing
pub struct InMemoryTransport {
    send_queue: tokio::sync::mpsc::UnboundedSender<JsonRpcMessage>,
    recv_queue: tokio::sync::mpsc::UnboundedReceiver<JsonRpcMessage>,
}

impl InMemoryTransport {
    /// Create a new in-memory transport pair
    pub fn pair() -> (Self, Self) {
        let (tx1, rx1) = tokio::sync::mpsc::unbounded_channel();
        let (tx2, rx2) = tokio::sync::mpsc::unbounded_channel();
        
        let transport1 = Self {
            send_queue: tx2,
            recv_queue: rx1,
        };
        
        let transport2 = Self {
            send_queue: tx1,
            recv_queue: rx2,
        };
        
        (transport1, transport2)
    }
}

#[async_trait]
impl Transport for InMemoryTransport {
    async fn send(&mut self, message: &JsonRpcMessage) -> Result<()> {
        self.send_queue.send(message.clone())
            .map_err(|e| anyhow::anyhow!("Failed to send message: {}", e))?;
        Ok(())
    }
    
    async fn receive(&mut self) -> Result<Option<JsonRpcMessage>> {
        match self.recv_queue.recv().await {
            Some(message) => Ok(Some(message)),
            None => Ok(None),
        }
    }
    
    async fn close(&mut self) -> Result<()> {
        self.recv_queue.close();
        Ok(())
    }
}