# MCP (Model Context Protocol) Implementation Plan

## Overview
MCP is a standardized protocol for AI applications to interact with external data sources and tools. It uses JSON-RPC 2.0 over either stdio (local) or HTTP/SSE (remote) transports.

## Core Components

### 1. JSON-RPC 2.0 Message Types

#### Request
```json
{
  "jsonrpc": "2.0",
  "id": "string | number",
  "method": "string",
  "params": {
    "[key: string]": "unknown"
  }
}
```

#### Response (Success)
```json
{
  "jsonrpc": "2.0",
  "id": "string | number",
  "result": {
    "[key: string]": "unknown"
  }
}
```

#### Response (Error)
```json
{
  "jsonrpc": "2.0",
  "id": "string | number",
  "error": {
    "code": "number",
    "message": "string",
    "data": "unknown"
  }
}
```

#### Notification
```json
{
  "jsonrpc": "2.0",
  "method": "string",
  "params": {
    "[key: string]": "unknown"
  }
}
```

### 2. MCP Methods

#### Tool Discovery
- Method: `tools/list`
- Parameters: `{ cursor?: string }`
- Returns: List of available tools with pagination

#### Tool Invocation
- Method: `tools/call`
- Parameters: `{ name: string, arguments: object }`
- Returns: Tool execution result

## Implementation Architecture

### For ai-session (MCP Server)

```rust
// ai-session/src/mcp/mod.rs
pub mod server;
pub mod transport;
pub mod tools;
pub mod jsonrpc;

// Main MCP server that exposes ai-session functionality
pub struct McpServer {
    session_manager: Arc<SessionManager>,
    transport: Box<dyn Transport>,
    tools: HashMap<String, Tool>,
}
```

#### Tools to Expose:
1. **execute_command**: Execute commands in ai-session
   ```json
   {
     "name": "execute_command",
     "description": "Execute a command in an AI session",
     "inputSchema": {
       "type": "object",
       "properties": {
         "session_id": { "type": "string" },
         "command": { "type": "string" }
       },
       "required": ["session_id", "command"]
     }
   }
   ```

2. **create_session**: Create new ai-session
   ```json
   {
     "name": "create_session",
     "description": "Create a new AI session",
     "inputSchema": {
       "type": "object",
       "properties": {
         "name": { "type": "string" },
         "working_directory": { "type": "string" }
       },
       "required": ["name"]
     }
   }
   ```

3. **get_session_info**: Get session information
   ```json
   {
     "name": "get_session_info",
     "description": "Get information about a session",
     "inputSchema": {
       "type": "object",
       "properties": {
         "session_id": { "type": "string" }
       },
       "required": ["session_id"]
     }
   }
   ```

### For ccswarm (MCP Client)

```rust
// ccswarm/src/mcp/mod.rs
pub mod client;
pub mod discovery;

// MCP client for ccswarm to connect to ai-session
pub struct McpClient {
    transport: Box<dyn Transport>,
    available_tools: Vec<Tool>,
}

impl McpClient {
    pub async fn discover_tools(&mut self) -> Result<Vec<Tool>>;
    pub async fn invoke_tool(&self, name: &str, args: Value) -> Result<Value>;
}
```

## Transport Layer

### Stdio Transport (Local)
```rust
pub struct StdioTransport {
    stdin: tokio::io::Stdin,
    stdout: tokio::io::Stdout,
}
```

### HTTP/SSE Transport (Remote)
```rust
pub struct HttpTransport {
    base_url: String,
    client: reqwest::Client,
}
```

## Implementation Steps

### Phase 1: JSON-RPC Foundation
1. Create JSON-RPC types and serialization
2. Implement message framing and parsing
3. Add request/response correlation

### Phase 2: MCP Server (ai-session)
1. Implement stdio transport
2. Create tool registry
3. Implement tools/list method
4. Implement tools/call method
5. Add execute_command tool
6. Add create_session tool
7. Add get_session_info tool

### Phase 3: MCP Client (ccswarm)
1. Implement MCP client
2. Add tool discovery
3. Integrate with orchestrator
4. Allow agents to use MCP tools

### Phase 4: Advanced Features
1. Add HTTP/SSE transport
2. Implement authentication (OAuth 2.1)
3. Add resource management
4. Add prompt templates

## Testing Strategy

1. Unit tests for JSON-RPC parsing
2. Integration tests for tool invocation
3. End-to-end tests with Claude Desktop
4. Performance benchmarks

## Security Considerations

1. Validate all tool inputs
2. Implement rate limiting
3. Add user consent mechanism
4. Sandbox command execution
5. Audit all tool invocations

## Configuration Example

### For Claude Desktop
```json
{
  "mcpServers": {
    "ai-session": {
      "command": "ai-session",
      "args": ["mcp-server", "--stdio"]
    }
  }
}
```

### For ccswarm
```toml
[mcp]
servers = [
  { name = "ai-session", command = "ai-session mcp-server --stdio" }
]
```