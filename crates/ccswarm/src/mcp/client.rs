//! MCP client implementation for ccswarm

use anyhow::Result;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use super::jsonrpc::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, RequestId};
use super::transport::Transport;

/// MCP tool definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// MCP tool result
#[derive(Debug, Clone)]
pub struct McpToolResult {
    pub content: Vec<McpContent>,
    pub is_error: bool,
}

/// MCP content type
#[derive(Debug, Clone)]
pub enum McpContent {
    Text { text: String },
    Image { data: String, mime_type: String },
}

/// MCP client for communicating with ai-session servers
pub struct McpClient {
    /// Transport for communication
    transport: Arc<Mutex<Box<dyn Transport>>>,
    /// Request ID counter
    request_id: AtomicI64,
    /// Pending requests (waiting for responses)
    pending_requests:
        Arc<RwLock<HashMap<RequestId, tokio::sync::oneshot::Sender<JsonRpcResponse>>>>,
    /// Available tools
    tools: Arc<RwLock<HashMap<String, McpTool>>>,
    /// Client configuration
    config: McpClientConfig,
    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
}

/// MCP client configuration
#[derive(Debug, Clone)]
pub struct McpClientConfig {
    /// Request timeout
    pub request_timeout: Duration,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Client name for identification
    pub client_name: String,
    /// Client version
    pub client_version: String,
}

impl Default for McpClientConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            max_concurrent_requests: 100,
            client_name: "ccswarm".to_string(),
            client_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

impl McpClient {
    /// Create a new MCP client
    pub fn new(transport: Box<dyn Transport>) -> Self {
        Self::with_config(transport, McpClientConfig::default())
    }

    /// Create a new MCP client with configuration
    pub fn with_config(transport: Box<dyn Transport>, config: McpClientConfig) -> Self {
        Self {
            transport: Arc::new(Mutex::new(transport)),
            request_id: AtomicI64::new(1),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            config,
            shutdown_tx: None,
        }
    }

    /// Start the MCP client
    pub async fn start(&mut self) -> Result<()> {
        info!(
            "Starting MCP client: {}/{}",
            self.config.client_name, self.config.client_version
        );

        // Initialize with the server
        self.initialize().await?;

        // Discover available tools
        self.discover_tools().await?;

        // Start message handling loop
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let transport = self.transport.clone();
        let pending_requests = self.pending_requests.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Handle shutdown signal
                    _ = shutdown_rx.recv() => {
                        info!("MCP client shutting down");
                        break;
                    }

                    // Handle incoming messages
                    message_result = async {
                        let mut transport = transport.lock().await;
                        transport.receive().await
                    } => {
                        match message_result {
                            Ok(Some(message)) => {
                                if let Err(e) = Self::handle_message(message, &pending_requests).await {
                                    error!("Error handling message: {}", e);
                                }
                            }
                            Ok(None) => {
                                warn!("Transport closed");
                                break;
                            }
                            Err(e) => {
                                error!("Error receiving message: {}", e);
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Shutdown the MCP client
    pub async fn shutdown(&mut self) -> Result<()> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(()).await;
        }

        // Close transport
        let mut transport = self.transport.lock().await;
        transport.close().await?;

        Ok(())
    }

    /// Initialize the MCP session
    async fn initialize(&self) -> Result<()> {
        let request = JsonRpcRequest::new(
            self.next_request_id(),
            "initialize".to_string(),
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "clientInfo": {
                    "name": self.config.client_name,
                    "version": self.config.client_version
                }
            })),
        );

        let response = self.send_request(request).await?;

        if response.error.is_some() {
            return Err(anyhow::anyhow!("Initialize failed: {:?}", response.error));
        }

        info!("MCP client initialized successfully");
        Ok(())
    }

    /// Discover available tools from the server
    async fn discover_tools(&self) -> Result<()> {
        let request = JsonRpcRequest::new(self.next_request_id(), "tools/list".to_string(), None);

        let response = self.send_request(request).await?;

        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("Tools discovery failed: {:?}", error));
        }

        if let Some(result) = response.result
            && let Some(tools_array) = result.get("tools").and_then(|v| v.as_array())
        {
            let mut tools = self.tools.write().await;

            for tool_value in tools_array {
                if let Ok(tool) = serde_json::from_value::<McpTool>(tool_value.clone()) {
                    debug!("Discovered tool: {}", tool.name);
                    tools.insert(tool.name.clone(), tool);
                }
            }

            info!("Discovered {} tools", tools.len());
        }

        Ok(())
    }

    /// Execute a tool on the server
    pub async fn execute_tool(&self, tool_name: &str, arguments: Value) -> Result<McpToolResult> {
        // Verify tool exists
        {
            let tools = self.tools.read().await;
            if !tools.contains_key(tool_name) {
                return Err(anyhow::anyhow!("Tool '{}' not found", tool_name));
            }
        }

        let request = JsonRpcRequest::new(
            self.next_request_id(),
            "tools/call".to_string(),
            Some(json!({
                "name": tool_name,
                "arguments": arguments
            })),
        );

        let response = self.send_request(request).await?;

        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("Tool execution failed: {:?}", error));
        }

        let result = response
            .result
            .ok_or_else(|| anyhow::anyhow!("No result from tool execution"))?;

        // Parse tool result
        let content = result
            .get("content")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| {
                if let Some(text) = v.get("text").and_then(|t| t.as_str()) {
                    Some(McpContent::Text {
                        text: text.to_string(),
                    })
                } else if let (Some(data), Some(mime_type)) = (
                    v.get("data").and_then(|d| d.as_str()),
                    v.get("mimeType").and_then(|m| m.as_str()),
                ) {
                    Some(McpContent::Image {
                        data: data.to_string(),
                        mime_type: mime_type.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        let is_error = result
            .get("isError")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Ok(McpToolResult { content, is_error })
    }

    /// Get list of available tools
    pub async fn list_tools(&self) -> Vec<McpTool> {
        let tools = self.tools.read().await;
        tools.values().cloned().collect()
    }

    /// Send a JSON-RPC request and wait for response
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        // Store the pending request
        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(request.id.clone(), response_tx);
        }

        // Send the request
        {
            let mut transport = self.transport.lock().await;
            transport.send(&JsonRpcMessage::Request(request)).await?;
        }

        // Wait for response with timeout
        let response = timeout(self.config.request_timeout, response_rx)
            .await
            .map_err(|_| anyhow::anyhow!("Request timeout"))?
            .map_err(|_| anyhow::anyhow!("Response channel closed"))?;

        Ok(response)
    }

    /// Handle incoming messages
    async fn handle_message(
        message: JsonRpcMessage,
        pending_requests: &Arc<
            RwLock<HashMap<RequestId, tokio::sync::oneshot::Sender<JsonRpcResponse>>>,
        >,
    ) -> Result<()> {
        match message {
            JsonRpcMessage::Response(response) => {
                // Find and complete pending request
                let mut pending = pending_requests.write().await;
                if let Some(sender) = pending.remove(&response.id) {
                    let _ = sender.send(response);
                } else {
                    warn!("Received response for unknown request: {}", response.id);
                }
            }
            JsonRpcMessage::Notification(notification) => {
                debug!("Received notification: {}", notification.method);
                // Handle notifications (currently just log them)
            }
            JsonRpcMessage::Request(request) => {
                warn!("Client received unexpected request: {}", request.method);
                // Clients don't typically handle requests
            }
        }

        Ok(())
    }

    /// Generate next request ID
    pub fn next_request_id(&self) -> RequestId {
        RequestId::Number(self.request_id.fetch_add(1, Ordering::SeqCst))
    }
}

/// High-level MCP client API for ai-session integration
pub struct AiSessionClient {
    mcp_client: McpClient,
}

impl AiSessionClient {
    /// Create a new ai-session client
    pub fn new(transport: Box<dyn Transport>) -> Self {
        Self {
            mcp_client: McpClient::new(transport),
        }
    }

    /// Start the client
    pub async fn start(&mut self) -> Result<()> {
        self.mcp_client.start().await
    }

    /// Shutdown the client
    pub async fn shutdown(&mut self) -> Result<()> {
        self.mcp_client.shutdown().await
    }

    /// Create a new ai-session
    pub async fn create_session(
        &self,
        name: &str,
        working_directory: Option<&str>,
    ) -> Result<String> {
        let mut args = json!({ "name": name });
        if let Some(wd) = working_directory {
            args["working_directory"] = json!(wd);
        }

        let result = self.mcp_client.execute_tool("create_session", args).await?;

        if result.is_error {
            return Err(anyhow::anyhow!("Failed to create session"));
        }

        // Extract session ID from result
        if let Some(McpContent::Text { text }) = result.content.first() {
            // Parse session ID from response text
            if let Some(start) = text.find("ID: ") {
                let id_start = start + 4;
                if let Some(end) = text[id_start..].find(char::is_whitespace) {
                    return Ok(text[id_start..id_start + end].to_string());
                } else {
                    return Ok(text[id_start..].to_string());
                }
            }
        }

        Err(anyhow::anyhow!(
            "Could not extract session ID from response"
        ))
    }

    /// Execute a command in an ai-session
    pub async fn execute_command(&self, session_id: &str, command: &str) -> Result<String> {
        let args = json!({
            "session_id": session_id,
            "command": command
        });

        let result = self
            .mcp_client
            .execute_tool("execute_command", args)
            .await?;

        if result.is_error {
            return Err(anyhow::anyhow!("Command execution failed"));
        }

        // Extract output from result
        if let Some(McpContent::Text { text }) = result.content.first() {
            Ok(text.clone())
        } else {
            Ok(String::new())
        }
    }

    /// Get information about a session
    pub async fn get_session_info(&self, session_id: &str) -> Result<Value> {
        let args = json!({ "session_id": session_id });

        let result = self
            .mcp_client
            .execute_tool("get_session_info", args)
            .await?;

        if result.is_error {
            return Err(anyhow::anyhow!("Failed to get session info"));
        }

        // Parse JSON from result
        if let Some(McpContent::Text { text }) = result.content.first() {
            serde_json::from_str(text)
                .map_err(|e| anyhow::anyhow!("Failed to parse session info: {}", e))
        } else {
            Ok(json!({}))
        }
    }
}
