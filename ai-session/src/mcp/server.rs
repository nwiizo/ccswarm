//! MCP server implementation for ai-session

use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, warn};

use super::jsonrpc::{
    JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId,
};
use super::tools::{Tool, ToolRegistry};
use super::transport::Transport;

/// MCP server that exposes ai-session functionality
pub struct McpServer {
    /// Session manager for ai-session operations
    #[allow(dead_code)]
    session_manager: Arc<crate::SessionManager>,
    /// Transport for communication
    transport: Arc<Mutex<Box<dyn Transport>>>,
    /// Tool registry
    tools: Arc<ToolRegistry>,
    /// Shutdown signal sender
    shutdown_tx: mpsc::Sender<()>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(
        session_manager: Arc<crate::SessionManager>,
        transport: Box<dyn Transport>,
        shutdown_tx: mpsc::Sender<()>,
    ) -> Self {
        let tools = Arc::new(ToolRegistry::with_builtin_tools(session_manager.clone()));

        Self {
            session_manager,
            transport: Arc::new(Mutex::new(transport)),
            tools,
            shutdown_tx,
        }
    }

    /// Run the MCP server
    pub async fn run(&self) -> Result<()> {
        info!("Starting MCP server");

        // Send initialization notification
        self.send_notification("initialized", None).await?;

        loop {
            let message = {
                let mut transport = self.transport.lock().await;
                transport.receive().await?
            };

            match message {
                Some(JsonRpcMessage::Request(request)) => {
                    debug!("Received request: {}", request.method);
                    if let Err(e) = self.handle_request(request).await {
                        error!("Error handling request: {}", e);
                    }
                }
                Some(JsonRpcMessage::Notification(notification)) => {
                    debug!("Received notification: {}", notification.method);
                    if let Err(e) = self.handle_notification(notification).await {
                        error!("Error handling notification: {}", e);
                    }
                }
                Some(JsonRpcMessage::Response(_)) => {
                    warn!("Server received unexpected response message");
                }
                None => {
                    info!("Transport closed, shutting down MCP server");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a JSON-RPC request
    async fn handle_request(&self, request: JsonRpcRequest) -> Result<()> {
        let response = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id, request.params).await,
            "tools/list" => self.handle_tools_list(request.id, request.params).await,
            "tools/call" => self.handle_tools_call(request.id, request.params).await,
            _ => {
                JsonRpcResponse::error(request.id, JsonRpcError::method_not_found(&request.method))
            }
        };

        self.send_response(response).await
    }

    /// Handle a JSON-RPC notification
    async fn handle_notification(&self, notification: JsonRpcNotification) -> Result<()> {
        match notification.method.as_str() {
            "shutdown" => {
                info!("Received shutdown notification");
                self.shutdown_tx.send(()).await?;
            }
            _ => {
                debug!("Ignoring unknown notification: {}", notification.method);
            }
        }
        Ok(())
    }

    /// Handle initialize request
    async fn handle_initialize(&self, id: RequestId, _params: Option<Value>) -> JsonRpcResponse {
        let result = json!({
            "protocolVersion": "0.1.0",
            "capabilities": {
                "tools": {
                    "listSupported": true,
                    "callSupported": true
                }
            },
            "serverInfo": {
                "name": "ai-session-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        JsonRpcResponse::success(id, result)
    }

    /// Handle tools/list request
    async fn handle_tools_list(&self, id: RequestId, params: Option<Value>) -> JsonRpcResponse {
        // Extract cursor for pagination if provided
        let _cursor = params
            .as_ref()
            .and_then(|p| p.get("cursor"))
            .and_then(|c| c.as_str());

        let tools: Vec<&Tool> = self.tools.list_tools();

        // TODO: Implement pagination
        let result = json!({
            "tools": tools,
            "nextCursor": null
        });

        JsonRpcResponse::success(id, result)
    }

    /// Handle tools/call request
    async fn handle_tools_call(&self, id: RequestId, params: Option<Value>) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing parameters".to_string()),
                );
            }
        };

        let name = match params.get("name").and_then(|n| n.as_str()) {
            Some(n) => n,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing tool name".to_string()),
                );
            }
        };

        let empty_args = json!({});
        let arguments = params.get("arguments").unwrap_or(&empty_args);

        match self.tools.invoke(name, arguments.clone()) {
            Ok(result) => JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()),
            Err(e) => JsonRpcResponse::error(id, JsonRpcError::internal_error(e.to_string())),
        }
    }

    /// Send a response
    async fn send_response(&self, response: JsonRpcResponse) -> Result<()> {
        let mut transport = self.transport.lock().await;
        transport.send(JsonRpcMessage::Response(response)).await
    }

    /// Send a notification
    async fn send_notification(&self, method: &str, params: Option<Value>) -> Result<()> {
        let notification = JsonRpcNotification::new(method.to_string(), params);
        let mut transport = self.transport.lock().await;
        transport
            .send(JsonRpcMessage::Notification(notification))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::transport::StdioTransport;
    use crate::SessionManager;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let session_manager = Arc::new(SessionManager::new());
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        let transport = Box::new(StdioTransport::new(shutdown_rx));

        let _server = McpServer::new(session_manager, transport, shutdown_tx);
        // Server created successfully
    }

    #[test]
    fn test_initialize_response() {
        // Test will be implemented when we can mock transport
    }
}
