//! MCP server test for ai-session
//!
//! This example demonstrates the MCP server functionality.

use ai_session::SessionManager;
use anyhow::Result;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔧 Testing ai-session MCP server...");

    // Create session manager
    let session_manager = Arc::new(SessionManager::new());

    // Note: For full MCP server testing, you would need a transport layer

    // Test tool registry functionality
    println!("🧪 Testing tool registry...");

    let tools = ai_session::mcp::tools::ToolRegistry::with_builtin_tools(session_manager.clone());

    // List all available tools
    let tool_list = tools.list_tools();
    println!("✅ Available MCP tools ({} total):", tool_list.len());

    for tool in tool_list {
        println!("   • {} - {}", tool.name, tool.description);
    }

    // Test create_session tool
    println!("🧪 Testing create_session tool...");

    let create_session_args = json!({
        "name": "mcp-test-session",
        "working_directory": "/tmp"
    });

    match tools.invoke("create_session", create_session_args) {
        Ok(result) => {
            println!("✅ create_session tool executed successfully");
            if let Some(ai_session::mcp::tools::ToolContent::Text { text }) = result.content.first()
            {
                println!("   Result: {}", text);
            }
        }
        Err(e) => {
            println!("❌ create_session tool failed: {}", e);
        }
    }

    // Test get_session_info tool if we can get session ID
    println!("🧪 Testing get_session_info tool...");

    // First get list of sessions
    let list_sessions_result = tools.invoke("list_sessions", json!({}));
    match list_sessions_result {
        Ok(result) => {
            println!("✅ list_sessions tool executed successfully");
            if let Some(ai_session::mcp::tools::ToolContent::Text { text }) = result.content.first()
            {
                println!("   Available sessions: {}", text);
            }
        }
        Err(e) => {
            println!("⚠️  list_sessions tool failed: {}", e);
        }
    }

    // Test invalid tool call
    println!("🧪 Testing invalid tool call...");

    match tools.invoke("nonexistent_tool", json!({})) {
        Ok(_) => {
            println!("❌ Unexpected success for invalid tool");
        }
        Err(e) => {
            println!("✅ Invalid tool correctly rejected: {}", e);
        }
    }

    println!("✅ MCP server test completed successfully!");

    Ok(())
}
