//! Model Context Protocol (MCP) implementation for ai-session

pub mod jsonrpc;
pub mod server;
pub mod tools;
pub mod transport;

pub use jsonrpc::{JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
pub use server::McpServer;
pub use tools::{Tool, ToolRegistry};
pub use transport::{StdioTransport, Transport};
