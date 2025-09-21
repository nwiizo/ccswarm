//! JSON-RPC 2.0 implementation for MCP client

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Builder for JSON-RPC objects
pub struct JsonRpcBuilder {
    jsonrpc: String,
}

impl Default for JsonRpcBuilder {
    fn default() -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
        }
    }
}

impl JsonRpcBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn request(self, id: RequestId, method: String, params: Option<Value>) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: self.jsonrpc,
            id,
            method,
            params,
        }
    }

    pub fn response_success(self, id: RequestId, result: Value) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: self.jsonrpc,
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn response_error(self, id: RequestId, error: JsonRpcError) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: self.jsonrpc,
            id,
            result: None,
            error: Some(error),
        }
    }

    pub fn notification(self, method: String, params: Option<Value>) -> JsonRpcNotification {
        JsonRpcNotification {
            jsonrpc: self.jsonrpc,
            method,
            params,
        }
    }
}

/// JSON-RPC 2.0 request ID
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

impl Default for RequestId {
    fn default() -> Self {
        RequestId::Number(1)
    }
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestId::Number(n) => write!(f, "{}", n),
            RequestId::String(s) => write!(f, "{}", s),
        }
    }
}

/// JSON-RPC 2.0 request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(id: RequestId, method: String, params: Option<Value>) -> Self {
        JsonRpcBuilder::new().request(id, method, params)
    }
}

/// JSON-RPC 2.0 response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a success response
    pub fn success(id: RequestId, result: Value) -> Self {
        JsonRpcBuilder::new().response_success(id, result)
    }

    /// Create an error response
    pub fn error(id: RequestId, error: JsonRpcError) -> Self {
        JsonRpcBuilder::new().response_error(id, error)
    }
}

/// JSON-RPC 2.0 notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

impl JsonRpcNotification {
    /// Create a new notification
    pub fn new(method: String, params: Option<Value>) -> Self {
        JsonRpcBuilder::new().notification(method, params)
    }
}

/// JSON-RPC 2.0 error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Create an error with the given code and message
    fn create_error(code: i32, message: String, data: Option<Value>) -> Self {
        Self {
            code,
            message,
            data,
        }
    }

    /// Parse error
    pub fn parse_error() -> Self {
        Self::create_error(-32700, "Parse error".to_string(), None)
    }

    /// Invalid request
    pub fn invalid_request() -> Self {
        Self::create_error(-32600, "Invalid Request".to_string(), None)
    }

    /// Method not found
    pub fn method_not_found(method: &str) -> Self {
        Self::create_error(-32601, format!("Method not found: {}", method), None)
    }

    /// Invalid parameters
    pub fn invalid_params(message: &str) -> Self {
        Self::create_error(-32602, format!("Invalid params: {}", message), None)
    }

    /// Internal error
    pub fn internal_error(message: &str) -> Self {
        Self::create_error(-32603, format!("Internal error: {}", message), None)
    }
}

/// JSON-RPC message (request, response, or notification)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

impl std::str::FromStr for JsonRpcMessage {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl JsonRpcMessage {
    /// Parse a JSON-RPC message from string (convenience method)
    pub fn parse(s: &str) -> Result<Self, serde_json::Error> {
        s.parse()
    }

    /// Serialize a JSON-RPC message to string
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
