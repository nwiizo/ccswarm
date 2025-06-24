//! Model Context Protocol (MCP) client implementation for ccswarm
//!
//! This module provides MCP client functionality to communicate with ai-session MCP servers,
//! enabling remote session management and control.

pub mod client;
pub mod jsonrpc;
pub mod transport;

pub use client::{AiSessionClient, McpClient};
pub use jsonrpc::{JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, RequestId};
pub use transport::{HttpTransport, InMemoryTransport, Transport, UnixSocketTransport};
