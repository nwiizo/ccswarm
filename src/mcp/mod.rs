//! Model Context Protocol (MCP) client implementation for ccswarm
//!
//! This module provides MCP client functionality to communicate with ai-session MCP servers,
//! enabling remote session management and control.

pub mod client;
pub mod jsonrpc;
pub mod transport;

pub use client::{McpClient, AiSessionClient};
pub use jsonrpc::{JsonRpcMessage, JsonRpcRequest, JsonRpcResponse, JsonRpcError, RequestId};
pub use transport::{Transport, HttpTransport, UnixSocketTransport, InMemoryTransport};