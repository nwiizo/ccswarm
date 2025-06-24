//! JSON-RPC 2.0 implementation for MCP

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// JSON-RPC version constant
pub const JSONRPC_VERSION: &str = "2.0";

/// JSON-RPC request ID type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestId::String(s) => write!(f, "{}", s),
            RequestId::Number(n) => write!(f, "{}", n),
        }
    }
}

/// JSON-RPC request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request
    pub fn new(id: RequestId, method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            method,
            params,
        }
    }
}

/// JSON-RPC error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ServerError = -32000,
}

/// JSON-RPC error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Create a new JSON-RPC error
    pub fn new(code: ErrorCode, message: String, data: Option<Value>) -> Self {
        Self {
            code: code as i32,
            message,
            data,
        }
    }

    /// Create a parse error
    pub fn parse_error() -> Self {
        Self::new(
            ErrorCode::ParseError,
            "Parse error".to_string(),
            None,
        )
    }

    /// Create an invalid request error
    pub fn invalid_request() -> Self {
        Self::new(
            ErrorCode::InvalidRequest,
            "Invalid Request".to_string(),
            None,
        )
    }

    /// Create a method not found error
    pub fn method_not_found(method: &str) -> Self {
        Self::new(
            ErrorCode::MethodNotFound,
            format!("Method not found: {}", method),
            None,
        )
    }

    /// Create an invalid params error
    pub fn invalid_params(message: String) -> Self {
        Self::new(
            ErrorCode::InvalidParams,
            message,
            None,
        )
    }

    /// Create an internal error
    pub fn internal_error(message: String) -> Self {
        Self::new(
            ErrorCode::InternalError,
            message,
            None,
        )
    }
}

/// JSON-RPC response
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
    /// Create a successful response
    pub fn success(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: RequestId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC notification (no ID, no response expected)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcNotification {
    /// Create a new JSON-RPC notification
    pub fn new(method: String, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            method,
            params,
        }
    }
}

/// JSON-RPC message that can be either request, response, or notification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_request_serialization() {
        let request = JsonRpcRequest::new(
            RequestId::Number(1),
            "test_method".to_string(),
            Some(json!({"param": "value"})),
        );
        
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains(r#""jsonrpc":"2.0""#));
        assert!(json.contains(r#""id":1"#));
        assert!(json.contains(r#""method":"test_method""#));
        assert!(json.contains(r#""params":{"param":"value"}"#));
    }

    #[test]
    fn test_response_success_serialization() {
        let response = JsonRpcResponse::success(
            RequestId::String("abc".to_string()),
            json!({"result": "success"}),
        );
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""jsonrpc":"2.0""#));
        assert!(json.contains(r#""id":"abc""#));
        assert!(json.contains(r#""result":{"result":"success"}"#));
        assert!(!json.contains("error"));
    }

    #[test]
    fn test_response_error_serialization() {
        let response = JsonRpcResponse::error(
            RequestId::Number(2),
            JsonRpcError::method_not_found("unknown_method"),
        );
        
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""jsonrpc":"2.0""#));
        assert!(json.contains(r#""id":2"#));
        assert!(json.contains(r#""code":-32601"#));
        assert!(json.contains("Method not found"));
        assert!(!json.contains("result"));
    }

    #[test]
    fn test_notification_serialization() {
        let notification = JsonRpcNotification::new(
            "status_update".to_string(),
            Some(json!({"status": "ready"})),
        );
        
        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains(r#""jsonrpc":"2.0""#));
        assert!(json.contains(r#""method":"status_update""#));
        assert!(json.contains(r#""params":{"status":"ready"}"#));
        assert!(!json.contains("id"));
    }

    #[test]
    fn test_message_deserialization() {
        // Test request deserialization
        let request_json = r#"{"jsonrpc":"2.0","id":1,"method":"test","params":{}}"#;
        let message: JsonRpcMessage = serde_json::from_str(request_json).unwrap();
        assert!(matches!(message, JsonRpcMessage::Request(_)));

        // Test response deserialization
        let response_json = r#"{"jsonrpc":"2.0","id":1,"result":{"status":"ok"}}"#;
        let message: JsonRpcMessage = serde_json::from_str(response_json).unwrap();
        assert!(matches!(message, JsonRpcMessage::Response(_)));

        // Test notification deserialization
        let notification_json = r#"{"jsonrpc":"2.0","method":"notify","params":null}"#;
        let message: JsonRpcMessage = serde_json::from_str(notification_json).unwrap();
        assert!(matches!(message, JsonRpcMessage::Notification(_)));
    }
}