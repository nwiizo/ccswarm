//! MCP tool definitions and registry

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name (must be unique)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// JSON Schema for input parameters
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Content returned by the tool
    pub content: Vec<ToolContent>,
    /// Whether the tool execution resulted in an error
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

/// Content type for tool results
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { 
        data: String,  // base64 encoded
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

/// Tool handler function type
pub type ToolHandler = Arc<dyn Fn(Value) -> Result<ToolResult> + Send + Sync>;

/// Registry for MCP tools
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
    handlers: HashMap<String, ToolHandler>,
}

impl ToolRegistry {
    /// Create a new tool registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            handlers: HashMap::new(),
        }
    }

    /// Register a new tool
    pub fn register<F>(&mut self, tool: Tool, handler: F) -> Result<()>
    where
        F: Fn(Value) -> Result<ToolResult> + Send + Sync + 'static,
    {
        if self.tools.contains_key(&tool.name) {
            return Err(anyhow::anyhow!("Tool '{}' already registered", tool.name));
        }

        let name = tool.name.clone();
        self.tools.insert(name.clone(), tool);
        self.handlers.insert(name, Arc::new(handler));
        
        Ok(())
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// List all registered tools
    pub fn list_tools(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }

    /// Invoke a tool
    pub fn invoke(&self, name: &str, arguments: Value) -> Result<ToolResult> {
        let handler = self.handlers.get(name)
            .ok_or_else(|| anyhow::anyhow!("Tool '{}' not found", name))?;
        
        handler(arguments)
    }

    /// Create built-in ai-session tools
    pub fn with_builtin_tools(session_manager: Arc<crate::SessionManager>) -> Self {
        let mut registry = Self::new();

        // Register execute_command tool
        let sm_clone = session_manager.clone();
        let execute_command_tool = Tool {
            name: "execute_command".to_string(),
            description: "Execute a command in an AI session".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "ID of the session to execute command in"
                    },
                    "command": {
                        "type": "string",
                        "description": "Command to execute"
                    }
                },
                "required": ["session_id", "command"]
            }),
        };

        registry.register(execute_command_tool, move |args| {
            let session_id_str = args.get("session_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing session_id"))?;
            
            let command = args.get("command")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing command"))?;

            // Parse session ID
            let session_id = crate::core::SessionId::parse_str(session_id_str)
                .map_err(|e| anyhow::anyhow!("Invalid session_id: {}", e))?;

            // Execute command asynchronously
            let sm = sm_clone.clone();
            let cmd = command.to_string();
            
            // Use blocking runtime for sync context
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async move {
                    // Get the session
                    let session = sm.get_session(&session_id)
                        .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

                    // Check session status
                    let status = session.status().await;
                    if status != crate::core::SessionStatus::Running {
                        return Err(anyhow::anyhow!("Session is not running: {:?}", status));
                    }

                    // Send command with newline
                    session.send_input(&format!("{}\n", cmd)).await?;

                    // Wait a bit for command to execute
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                    // Read output
                    let output = session.read_output().await?;
                    let output_str = String::from_utf8_lossy(&output);

                    Ok(ToolResult {
                        content: vec![ToolContent::Text {
                            text: output_str.to_string(),
                        }],
                        is_error: false,
                    })
                })
            });

            result
        }).unwrap();

        // Register create_session tool
        let sm_clone = session_manager.clone();
        let create_session_tool = Tool {
            name: "create_session".to_string(),
            description: "Create a new AI session".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Name for the new session"
                    },
                    "working_directory": {
                        "type": "string",
                        "description": "Working directory for the session",
                        "default": "."
                    }
                },
                "required": ["name"]
            }),
        };

        registry.register(create_session_tool, move |args| {
            let name = args.get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing name"))?;
            
            let working_dir = args.get("working_directory")
                .and_then(|v| v.as_str())
                .unwrap_or(".");

            // Create session configuration
            let mut config = crate::core::SessionConfig::default();
            config.name = Some(name.to_string());
            config.working_directory = std::path::PathBuf::from(working_dir);
            config.enable_ai_features = true;

            // Create session asynchronously
            let sm = sm_clone.clone();
            
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async move {
                    // Create the session
                    let session = sm.create_session_with_config(config).await?;
                    
                    // Start the session
                    session.start().await?;
                    
                    // Wait for session to be ready
                    let mut retries = 10;
                    while retries > 0 {
                        let status = session.status().await;
                        if status == crate::core::SessionStatus::Running {
                            break;
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        retries -= 1;
                    }
                    
                    let status = session.status().await;
                    if status != crate::core::SessionStatus::Running {
                        return Err(anyhow::anyhow!("Failed to start session: {:?}", status));
                    }

                    Ok(ToolResult {
                        content: vec![ToolContent::Text {
                            text: format!("Created and started session '{}' with ID: {}", 
                                         name, session.id),
                        }],
                        is_error: false,
                    })
                })
            });

            result
        }).unwrap();

        // Register get_session_info tool
        let sm_clone = session_manager.clone();
        let get_session_info_tool = Tool {
            name: "get_session_info".to_string(),
            description: "Get information about a session".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "ID of the session to get info for"
                    }
                },
                "required": ["session_id"]
            }),
        };

        registry.register(get_session_info_tool, move |args| {
            let session_id_str = args.get("session_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing session_id"))?;

            // Parse session ID
            let session_id = crate::core::SessionId::parse_str(session_id_str)
                .map_err(|e| anyhow::anyhow!("Invalid session_id: {}", e))?;

            // Get session info asynchronously
            let sm = sm_clone.clone();
            
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async move {
                    // Get the session
                    let session = sm.get_session(&session_id)
                        .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

                    // Get session information
                    let status = session.status().await;
                    let context = session.context.read().await;
                    let metadata = session.metadata.read().await;
                    
                    let info = json!({
                        "id": session.id.to_string(),
                        "name": session.config.name,
                        "status": format!("{:?}", status),
                        "working_directory": session.config.working_directory.display().to_string(),
                        "created_at": session.created_at.to_rfc3339(),
                        "ai_features_enabled": session.config.enable_ai_features,
                        "context_token_count": context.conversation_history.current_tokens,
                        "metadata": metadata.clone(),
                    });

                    Ok(ToolResult {
                        content: vec![ToolContent::Text {
                            text: serde_json::to_string_pretty(&info)
                                .unwrap_or_else(|_| format!("{:?}", info)),
                        }],
                        is_error: false,
                    })
                })
            });

            result
        }).unwrap();

        registry
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry() {
        let mut registry = ToolRegistry::new();
        
        let tool = Tool {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "input": {"type": "string"}
                }
            }),
        };

        registry.register(tool.clone(), |args| {
            Ok(ToolResult {
                content: vec![ToolContent::Text {
                    text: format!("Received: {:?}", args),
                }],
                is_error: false,
            })
        }).unwrap();

        // Test tool retrieval
        assert!(registry.get_tool("test_tool").is_some());
        assert!(registry.get_tool("nonexistent").is_none());

        // Test tool listing
        let tools = registry.list_tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "test_tool");

        // Test tool invocation
        let result = registry.invoke("test_tool", json!({"input": "hello"})).unwrap();
        assert!(!result.is_error);
        assert_eq!(result.content.len(), 1);
    }

    #[test]
    fn test_duplicate_tool_registration() {
        let mut registry = ToolRegistry::new();
        
        let tool = Tool {
            name: "duplicate".to_string(),
            description: "A tool".to_string(),
            input_schema: json!({}),
        };

        registry.register(tool.clone(), |_| {
            Ok(ToolResult {
                content: vec![],
                is_error: false,
            })
        }).unwrap();

        // Second registration should fail
        let result = registry.register(tool, |_| {
            Ok(ToolResult {
                content: vec![],
                is_error: false,
            })
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_tool_content_serialization() {
        let text_content = ToolContent::Text {
            text: "Hello, world!".to_string(),
        };

        let json = serde_json::to_string(&text_content).unwrap();
        assert!(json.contains(r#""type":"text""#));
        assert!(json.contains(r#""text":"Hello, world!""#));

        let image_content = ToolContent::Image {
            data: "base64data".to_string(),
            mime_type: "image/png".to_string(),
        };

        let json = serde_json::to_string(&image_content).unwrap();
        assert!(json.contains(r#""type":"image""#));
        assert!(json.contains(r#""data":"base64data""#));
        assert!(json.contains(r#""mimeType":"image/png""#));
    }
}