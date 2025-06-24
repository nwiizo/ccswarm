# AI-Session Developer Guide

A practical guide for developers integrating ai-session into their projects and understanding the ccswarm ecosystem.

## Table of Contents

- [Overview](#overview)
- [Installation & Setup](#installation--setup)
- [Integration Patterns](#integration-patterns)
- [ccswarm Integration](#ccswarm-integration)
- [Performance Considerations](#performance-considerations)
- [Testing & Debugging](#testing--debugging)
- [Common Patterns](#common-patterns)
- [Migration Guide](#migration-guide)

## Overview

ai-session is a revolutionary terminal session management library designed for AI agents and modern development workflows. It provides:

- **93% API cost reduction** through intelligent context compression
- **Native PTY implementation** for cross-platform compatibility
- **Multi-agent coordination** via message bus architecture
- **Advanced observability** for decision tracking and performance analysis

### When to Use ai-session

✅ **Perfect for:**
- AI agent development platforms
- Multi-agent coordination systems
- Development environment automation
- CI/CD pipeline orchestration
- Terminal multiplexer replacements

❌ **Not ideal for:**
- Simple script automation (use regular process spawning)
- Single-command executions
- Non-interactive batch processing

## Installation & Setup

### Add to Your Project

```toml
[dependencies]
ai-session = "0.3.5"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```

### Feature Flags

```toml
[dependencies]
ai-session = { version = "0.3.5", features = ["cli", "server"] }
```

Available features:
- `cli` - Command-line interface utilities
- `server` - HTTP server for external integration
- `testing` - Mock utilities for testing

### Environment Setup

```bash
# Required for some features
export RUST_LOG=info

# Optional: Configure ai-session
export AI_SESSION_LOG_LEVEL=debug
export AI_SESSION_COMPRESSION=true
```

## Integration Patterns

### Pattern 1: Simple Session Management

Best for: Basic terminal automation, development tools

```rust
use ai_session::{SessionManager, SessionConfig};
use anyhow::Result;

pub struct SimpleTerminalManager {
    manager: SessionManager,
}

impl SimpleTerminalManager {
    pub fn new() -> Self {
        Self {
            manager: SessionManager::new(),
        }
    }
    
    pub async fn execute_command(&self, command: &str) -> Result<String> {
        let session = self.manager.create_session().await?;
        session.start().await?;
        
        session.send_input(&format!("{}\n", command)).await?;
        
        // Wait for execution
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        let output = session.read_output().await?;
        let result = String::from_utf8_lossy(&output).to_string();
        
        session.stop().await?;
        Ok(result)
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<()> {
    let terminal = SimpleTerminalManager::new();
    let output = terminal.execute_command("ls -la").await?;
    println!("Output: {}", output);
    Ok(())
}
```

### Pattern 2: Long-Running Agent Sessions

Best for: AI agents, interactive development environments

```rust
use ai_session::{SessionManager, SessionConfig, ContextConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AgentSession {
    session: Arc<ai_session::AISession>,
    context_history: Arc<RwLock<Vec<String>>>,
}

impl AgentSession {
    pub async fn new(role: &str, working_dir: &str) -> anyhow::Result<Self> {
        let manager = SessionManager::new();
        
        let mut config = SessionConfig::default();
        config.enable_ai_features = true;
        config.agent_role = Some(role.to_string());
        config.working_directory = working_dir.into();
        config.context_config = ContextConfig {
            max_tokens: 8192,
            compression_threshold: 0.8,
        };
        
        let session = manager.create_session_with_config(config).await?;
        session.start().await?;
        
        Ok(Self {
            session,
            context_history: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    pub async fn execute_with_context(&self, command: &str) -> anyhow::Result<String> {
        // Add to context history
        {
            let mut history = self.context_history.write().await;
            history.push(format!("Command: {}", command));
        }
        
        // Execute command
        self.session.send_input(&format!("{}\n", command)).await?;
        
        // Wait and read output
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let output = self.session.read_output().await?;
        let output_str = String::from_utf8_lossy(&output).to_string();
        
        // Add output to context
        {
            let mut history = self.context_history.write().await;
            history.push(format!("Output: {}", output_str));
            
            // Keep only recent history to manage memory
            if history.len() > 50 {
                history.drain(0..10); // Remove oldest 10 entries
            }
        }
        
        Ok(output_str)
    }
    
    pub async fn get_context_summary(&self) -> String {
        let history = self.context_history.read().await;
        if history.len() > 5 {
            format!("Recent commands:\n{}", history.iter().rev().take(5).cloned().collect::<Vec<_>>().join("\n"))
        } else {
            history.join("\n")
        }
    }
}

// Usage
#[tokio::main] 
async fn main() -> anyhow::Result<()> {
    let agent = AgentSession::new("rust-developer", "/tmp/project").await?;
    
    // Execute a series of development commands
    let commands = vec![
        "cargo new my-project --bin",
        "cd my-project", 
        "cargo add tokio --features full",
        "cargo check",
    ];
    
    for cmd in commands {
        let output = agent.execute_with_context(cmd).await?;
        println!("Command: {}\nOutput: {}\n", cmd, output);
    }
    
    // Get context summary
    let summary = agent.get_context_summary().await;
    println!("Session summary:\n{}", summary);
    
    Ok(())
}
```

### Pattern 3: Multi-Agent Coordination

Best for: Complex projects requiring multiple specialized agents

```rust
use ai_session::{SessionManager, SessionConfig};
use ai_session::coordination::{MultiAgentSession, AgentId, Message, MessageType};
use std::sync::Arc;
use std::collections::HashMap;

pub struct ProjectCoordinator {
    coordinator: Arc<MultiAgentSession>,
    agents: HashMap<String, AgentId>,
    manager: SessionManager,
}

impl ProjectCoordinator {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            coordinator: Arc::new(MultiAgentSession::new()),
            agents: HashMap::new(),
            manager: SessionManager::new(),
        })
    }
    
    pub async fn add_agent(&mut self, name: &str, role: &str, working_dir: &str) -> anyhow::Result<()> {
        let mut config = SessionConfig::default();
        config.enable_ai_features = true;
        config.agent_role = Some(role.to_string());
        config.working_directory = working_dir.into();
        
        let session = self.manager.create_session_with_config(config).await?;
        let agent_id = AgentId::new();
        
        self.coordinator.register_agent(agent_id.clone(), session)?;
        self.agents.insert(name.to_string(), agent_id);
        
        Ok(())
    }
    
    pub async fn coordinate_task(&self, task_description: &str) -> anyhow::Result<()> {
        // Start all agents
        self.coordinator.start_all_agents().await?;
        
        // Distribute task information to all agents
        if let Some(frontend_id) = self.agents.get("frontend") {
            if let Some(backend_id) = self.agents.get("backend") {
                // Frontend requests API specification from backend
                let api_request = Message {
                    from: frontend_id.clone(),
                    message_type: MessageType::TaskRequest,
                    payload: serde_json::json!({
                        "task": task_description,
                        "request_type": "api_specification",
                        "priority": "high"
                    }),
                    priority: ai_session::coordination::MessagePriority::High,
                    timestamp: chrono::Utc::now(),
                };
                
                self.coordinator.send_message(
                    frontend_id.clone(),
                    backend_id.clone(),
                    api_request
                ).await?;
            }
        }
        
        // Broadcast task start to all agents
        if let Some(frontend_id) = self.agents.get("frontend") {
            let broadcast = ai_session::coordination::BroadcastMessage {
                id: uuid::Uuid::new_v4(),
                from: frontend_id.clone(),
                content: format!("Starting coordinated task: {}", task_description),
                priority: ai_session::coordination::MessagePriority::Normal,
                timestamp: chrono::Utc::now(),
                metadata: Some(serde_json::json!({
                    "task_type": "coordinated",
                    "estimated_duration": "30m"
                })),
            };
            
            self.coordinator.broadcast(frontend_id.clone(), broadcast).await?;
        }
        
        Ok(())
    }
}

// Usage
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut coordinator = ProjectCoordinator::new().await?;
    
    // Add specialized agents
    coordinator.add_agent("frontend", "react-developer", "/project/frontend").await?;
    coordinator.add_agent("backend", "rust-developer", "/project/backend").await?;
    coordinator.add_agent("qa", "test-engineer", "/project").await?;
    
    // Coordinate a complex task
    coordinator.coordinate_task("Implement user authentication system").await?;
    
    println!("Task coordination initiated");
    Ok(())
}
```

## ccswarm Integration

ai-session is designed to integrate seamlessly with ccswarm's multi-agent orchestration system.

### Using SessionManagerAdapter

```rust
use ai_session::ccswarm::SessionManagerAdapter;
use ccswarm::{AgentRole, AutoAcceptConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let workspace_root = PathBuf::from("/path/to/project");
    let adapter = SessionManagerAdapter::new(workspace_root);
    
    // Create ccswarm-compatible agent session
    let agent_session = adapter.create_agent_session(
        "frontend-agent-001".to_string(),
        AgentRole::Frontend {
            framework: "react".to_string(),
            languages: vec!["typescript".to_string(), "css".to_string()],
        },
        PathBuf::from("/path/to/project/frontend"),
        Some("Frontend development agent".to_string()),
        true, // Enable AI features
    ).await?;
    
    // Configure auto-accept for safe operations
    let auto_accept_config = AutoAcceptConfig {
        enabled: true,
        risk_threshold: 3,
        protected_patterns: vec![
            "package.json".to_string(),
            "*.env".to_string(),
        ],
    };
    
    // Start and configure the session
    {
        let mut session = agent_session.lock().await;
        session.start().await?;
        session.configure_auto_accept(auto_accept_config);
        
        // Execute ccswarm-specific commands
        session.execute_command("npm install").await?;
        session.execute_command("npm run build").await?;
    }
    
    // Get performance statistics
    {
        let session = agent_session.lock().await;
        let stats = session.get_performance_stats().await;
        println!("Token savings: {:.1}%", stats.token_savings_percentage);
        println!("Commands executed: {}", stats.commands_executed);
    }
    
    Ok(())
}
```

### Context Integration

```rust
use ai_session::{SessionContext, ContextMessage, MessageRole};
use ai_session::SessionId;

async fn integrate_with_ccswarm_context() -> anyhow::Result<()> {
    let session_id = SessionId::new();
    let mut context = SessionContext::new(session_id);
    
    // Add ccswarm system message
    let system_message = ContextMessage {
        role: MessageRole::System,
        content: "You are a specialized AI agent in the ccswarm multi-agent system. Your role is frontend development using React and TypeScript.".to_string(),
        timestamp: chrono::Utc::now(),
        token_count: 30,
    };
    context.add_message(system_message);
    
    // Add task context
    let task_message = ContextMessage {
        role: MessageRole::User,
        content: "Implement a user registration form with validation".to_string(),
        timestamp: chrono::Utc::now(),
        token_count: 10,
    };
    context.add_message(task_message);
    
    // Use ccswarm-specific context methods
    println!("Context messages: {}", context.get_message_count());
    println!("Total tokens: {}", context.get_total_tokens());
    
    // Compress context for efficiency
    if context.get_total_tokens() > 100 {
        context.compress_context().await;
        println!("Context compressed to {} tokens", context.get_total_tokens());
    }
    
    Ok(())
}
```

## Performance Considerations

### Memory Management

```rust
use ai_session::{SessionManager, SessionConfig};
use std::sync::Arc;
use tokio::sync::Semaphore;

// Session pool for high-throughput scenarios
pub struct OptimizedSessionPool {
    manager: SessionManager,
    semaphore: Arc<Semaphore>,
    config: SessionConfig,
}

impl OptimizedSessionPool {
    pub fn new(max_concurrent: usize) -> Self {
        let mut config = SessionConfig::default();
        config.output_buffer_size = 512 * 1024; // 512KB buffer
        config.compress_output = true;
        config.context_config.compression_threshold = 0.7; // Compress earlier
        
        Self {
            manager: SessionManager::new(),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            config,
        }
    }
    
    pub async fn execute_batch<F>(&self, commands: Vec<String>, processor: F) -> anyhow::Result<Vec<String>>
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        let processor = Arc::new(processor);
        let mut handles = Vec::new();
        
        for command in commands {
            let permit = self.semaphore.clone().acquire_owned().await?;
            let session = self.manager.create_session_with_config(self.config.clone()).await?;
            let processor = processor.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = permit; // Keep permit alive
                
                session.start().await?;
                session.send_input(&format!("{}\n", command)).await?;
                
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                let output = session.read_output().await?;
                let result = String::from_utf8_lossy(&output).to_string();
                
                session.stop().await?;
                
                Ok::<String, anyhow::Error>(processor(&result))
            });
            
            handles.push(handle);
        }
        
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await??);
        }
        
        Ok(results)
    }
}
```

### Context Optimization

```rust
use ai_session::context::{SessionContext, CompressionStrategy};

async fn optimize_context_usage() -> anyhow::Result<()> {
    let session_id = ai_session::SessionId::new();
    let mut context = SessionContext::new(session_id);
    
    // Configure for high-efficiency operation
    context.config.max_tokens = 4096;
    context.config.compression_threshold = 0.6;
    
    // Monitor and compress proactively
    let mut message_count = 0;
    
    loop {
        // Simulate adding messages
        message_count += 1;
        
        // Check compression needs every 10 messages
        if message_count % 10 == 0 {
            let tokens = context.get_total_tokens();
            let threshold = (context.config.max_tokens as f64 * context.config.compression_threshold) as usize;
            
            if tokens > threshold {
                println!("Compressing context: {} tokens -> ", tokens);
                context.compress_context().await;
                println!("{} tokens", context.get_total_tokens());
            }
        }
        
        if message_count >= 100 {
            break;
        }
    }
    
    Ok(())
}
```

## Testing & Debugging

### Unit Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ai_session::{SessionManager, SessionConfig};
    
    #[tokio::test]
    async fn test_session_creation() {
        let manager = SessionManager::new();
        let session = manager.create_session().await.unwrap();
        
        assert_eq!(session.status().await, ai_session::SessionStatus::Initializing);
    }
    
    #[tokio::test]
    async fn test_command_execution() {
        let manager = SessionManager::new();
        let session = manager.create_session().await.unwrap();
        
        session.start().await.unwrap();
        session.send_input("echo 'test'\n").await.unwrap();
        
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let output = session.read_output().await.unwrap();
        let output_str = String::from_utf8_lossy(&output);
        
        assert!(output_str.contains("test"));
        
        session.stop().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_context_compression() {
        use ai_session::context::{SessionContext, Message, MessageRole};
        
        let session_id = ai_session::SessionId::new();
        let mut context = SessionContext::new(session_id);
        
        // Add many messages to trigger compression
        for i in 0..50 {
            let message = Message {
                role: MessageRole::User,
                content: format!("Test message {}", i),
                timestamp: chrono::Utc::now(),
                token_count: 5,
            };
            context.add_message(message);
        }
        
        let initial_tokens = context.get_total_tokens();
        assert!(initial_tokens > 200);
        
        let compressed = context.compress_context().await;
        if compressed {
            assert!(context.get_total_tokens() < initial_tokens);
        }
    }
}
```

### Debugging Configuration

```rust
use ai_session::{SessionConfig, ContextConfig};

fn create_debug_config() -> SessionConfig {
    let mut config = SessionConfig::default();
    
    // Enable verbose logging
    config.environment.insert("RUST_LOG".to_string(), "debug".to_string());
    config.environment.insert("AI_SESSION_DEBUG".to_string(), "1".to_string());
    
    // Increase buffer sizes for detailed output
    config.output_buffer_size = 10 * 1024 * 1024; // 10MB
    
    // Reduce compression threshold for testing
    config.context_config.compression_threshold = 0.5;
    
    // Set conservative timeout
    config.timeout = Some(std::time::Duration::from_secs(30));
    
    config
}

// Enable debug logging
async fn setup_debug_logging() {
    ai_session::init_logging();
    
    // You can also use env_logger for more control
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
}
```

## Common Patterns

### Error Handling

```rust
use ai_session::{SessionManager, SessionError};

async fn robust_session_handling() -> anyhow::Result<()> {
    let manager = SessionManager::new();
    
    match manager.create_session().await {
        Ok(session) => {
            match session.start().await {
                Ok(_) => {
                    // Session started successfully
                    println!("Session ready");
                }
                Err(e) => {
                    eprintln!("Failed to start session: {}", e);
                    
                    // Handle specific error types
                    if let Some(session_error) = e.downcast_ref::<SessionError>() {
                        match session_error {
                            SessionError::PtyError(msg) => {
                                eprintln!("PTY error: {}", msg);
                                // Maybe retry with different PTY settings
                            }
                            SessionError::ProcessError(msg) => {
                                eprintln!("Process error: {}", msg);
                                // Check system resources
                            }
                            _ => eprintln!("Other session error: {}", session_error),
                        }
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to create session: {}", e);
        }
    }
    
    Ok(())
}
```

### Resource Cleanup

```rust
use ai_session::{SessionManager, AISession};
use std::sync::Arc;

// RAII wrapper for automatic cleanup
pub struct ManagedSession {
    session: Arc<AISession>,
}

impl ManagedSession {
    pub async fn new(manager: &SessionManager) -> anyhow::Result<Self> {
        let session = manager.create_session().await?;
        session.start().await?;
        
        Ok(Self { session })
    }
    
    pub async fn execute(&self, command: &str) -> anyhow::Result<String> {
        self.session.send_input(&format!("{}\n", command)).await?;
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        
        let output = self.session.read_output().await?;
        Ok(String::from_utf8_lossy(&output).to_string())
    }
}

impl Drop for ManagedSession {
    fn drop(&mut self) {
        let session = self.session.clone();
        tokio::spawn(async move {
            let _ = session.stop().await;
        });
    }
}

// Usage
async fn example_usage() -> anyhow::Result<()> {
    let manager = SessionManager::new();
    
    {
        let session = ManagedSession::new(&manager).await?;
        let output = session.execute("ls -la").await?;
        println!("Output: {}", output);
        // Session automatically cleaned up when dropped
    }
    
    Ok(())
}
```

## Migration Guide

### From tmux

```rust
// Before (tmux)
// tmux new-session -d -s mysession
// tmux send-keys -t mysession "echo hello" Enter
// tmux capture-pane -t mysession -p

// After (ai-session)
use ai_session::{SessionManager, SessionConfig};

async fn migrate_from_tmux() -> anyhow::Result<()> {
    let manager = SessionManager::new();
    
    let mut config = SessionConfig::default();
    config.name = Some("mysession".to_string());
    
    let session = manager.create_session_with_config(config).await?;
    session.start().await?;
    
    session.send_input("echo hello\n").await?;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    
    let output = session.read_output().await?;
    println!("{}", String::from_utf8_lossy(&output));
    
    session.stop().await?;
    Ok(())
}
```

### From subprocess

```rust
// Before (subprocess)
// use std::process::Command;
// let output = Command::new("ls").arg("-la").output()?;

// After (ai-session) - for persistent sessions
use ai_session::{SessionManager, SessionConfig};

async fn migrate_from_subprocess() -> anyhow::Result<()> {
    let manager = SessionManager::new();
    let session = manager.create_session().await?;
    
    session.start().await?;
    session.send_input("ls -la\n").await?;
    
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let output = session.read_output().await?;
    
    println!("{}", String::from_utf8_lossy(&output));
    
    session.stop().await?;
    Ok(())
}
```

## Best Practices

1. **Use session pooling** for high-throughput applications
2. **Enable compression** for AI feature to save on API costs
3. **Set appropriate timeouts** to prevent hanging
4. **Clean up sessions** explicitly or use RAII patterns
5. **Monitor context size** and compress proactively
6. **Use the HTTP API** for language-agnostic integration
7. **Test with realistic workloads** before production
8. **Configure logging** appropriately for your environment

## Troubleshooting

### Common Issues

**Session won't start:**
- Check if working directory exists
- Verify shell path is correct
- Ensure PTY size is valid (not 0x0)

**High memory usage:**
- Enable context compression
- Reduce context window size
- Clean up terminated sessions regularly

**Performance issues:**
- Use session pooling
- Enable output compression
- Batch commands when possible

**HTTP server connection issues:**
- Verify server is running
- Check firewall settings
- Ensure correct port configuration

For more detailed troubleshooting, see the [API Reference](API_REFERENCE.md#troubleshooting).