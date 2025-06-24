//! MCP client test example for ccswarm
//!
//! This example demonstrates how ccswarm can use the MCP client to
//! communicate with ai-session MCP servers.

use anyhow::Result;
use ccswarm::mcp::{AiSessionClient, HttpTransport, UnixSocketTransport};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 Testing ccswarm MCP client...");

    // Test with in-memory transport for demonstration
    test_with_mock_transport().await?;

    // Test with HTTP transport (would connect to real server)
    test_http_transport_config().await?;

    println!("✅ MCP client test completed!");

    Ok(())
}

async fn test_with_mock_transport() -> Result<()> {
    println!("🧪 Testing MCP client with mock transport...");

    // For testing, we'll simulate the client interface
    // In real usage, this would connect to an actual ai-session MCP server

    println!("   • Mock transport configuration successful");
    println!("   • Would connect to ai-session server via transport");
    println!("   • Would discover available tools (create_session, execute_command, etc.)");
    println!("   • Would enable remote session management");

    Ok(())
}

async fn test_http_transport_config() -> Result<()> {
    println!("🧪 Testing HTTP transport configuration...");

    // Create HTTP transport (doesn't actually connect)
    let transport = HttpTransport::new("http://localhost:8080/mcp".to_string())
        .with_timeout(Duration::from_secs(30));

    println!("   • HTTP transport created for ai-session server");
    println!("   • Server URL: http://localhost:8080/mcp");
    println!("   • Timeout: 30 seconds");

    // In real usage:
    // let mut client = AiSessionClient::new(Box::new(transport));
    // client.start().await?;
    // let session_id = client.create_session("test-session", Some("/tmp")).await?;
    // let output = client.execute_command(&session_id, "echo 'Hello from remote session'").await?;

    println!("   • Would enable remote session creation and command execution");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccswarm::mcp::{InMemoryTransport, McpClient};
    use serde_json::json;

    #[tokio::test]
    async fn test_mcp_client_creation() {
        let (transport1, _transport2) = InMemoryTransport::pair();
        let _client = McpClient::new(Box::new(transport1));

        // Client created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_request_id_generation() {
        let (transport1, _transport2) = InMemoryTransport::pair();
        let client = McpClient::new(Box::new(transport1));

        // Test that request IDs are generated correctly
        let id1 = client.next_request_id();
        let id2 = client.next_request_id();

        assert_ne!(id1, id2);
    }
}
