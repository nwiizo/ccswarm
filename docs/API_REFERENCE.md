# AI-Session API Reference

Complete API reference for the ai-session library and its integration with ccswarm. This document provides comprehensive examples for practical usage.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Core API](#core-api)
- [HTTP Server API](#http-server-api)
- [Context Management](#context-management)
- [Multi-Agent Coordination](#multi-agent-coordination)
- [ccswarm Integration](#ccswarm-integration)
- [Examples by Use Case](#examples-by-use-case)
- [Performance Optimizations](#performance-optimizations)
- [Troubleshooting](#troubleshooting)

## Overview

ai-session provides a revolutionary terminal session management system optimized for AI agents. Key benefits:

- **93% API cost reduction** through intelligent context compression
- **Native PTY implementation** for cross-platform compatibility
- **Multi-agent coordination** with message bus architecture
- **Security and isolation** with capability-based access control
- **Advanced observability** for decision tracking and performance analysis

## Quick Start

### Basic Session Management

```rust
use ai_session::{SessionManager, SessionConfig, ContextConfig};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    ai_session::init_logging();

    // Create session manager
    let manager = SessionManager::new();
    
    // Create and configure session
    let mut config = SessionConfig::default();
    config.enable_ai_features = true;
    config.context_config.max_tokens = 8192;
    config.working_directory = std::env::current_dir()?;
    
    // Create session
    let session = manager.create_session_with_config(config).await?;
    
    // Start session
    session.start().await?;
    
    // Execute commands
    session.send_input("echo 'Hello AI Session!'\n").await?;
    
    // Read output
    let output = session.read_output().await?;
    println!("Output: {}", String::from_utf8_lossy(&output));
    
    // Clean up
    session.stop().await?;
    
    Ok(())
}
```

### Using the HTTP Server

Start the server:
```bash
# Start ai-session HTTP server
cargo run --bin ai-session-server -- --port 3000

# Or use the pre-built binary
ai-session-server --port 3000 --host 0.0.0.0
```

Create and use sessions via HTTP:
```bash
# Create a new session
curl -X POST http://localhost:3000/sessions \
     -H 'Content-Type: application/json' \
     -d '{
       "name": "dev-session",
       "enable_ai_features": true,
       "working_directory": "/path/to/project"
     }'

# Execute a command
curl -X POST http://localhost:3000/sessions/dev-session/execute \
     -H 'Content-Type: application/json' \
     -d '{"command": "cargo build"}'

# Get session output
curl http://localhost:3000/sessions/dev-session/output

# List all sessions
curl http://localhost:3000/sessions

# Delete session
curl -X DELETE http://localhost:3000/sessions/dev-session
```

## Core API

### SessionManager

The SessionManager is your main entry point for managing sessions.

```rust
use ai_session::{SessionManager, SessionConfig, SessionId};
use std::path::PathBuf;

let manager = SessionManager::new();

// Create session with default config
let session = manager.create_session().await?;

// Create session with custom config
let mut config = SessionConfig::default();
config.name = Some("frontend-agent".to_string());
config.working_directory = PathBuf::from("/path/to/frontend");
config.enable_ai_features = true;
config.agent_role = Some("frontend-developer".to_string());

let session = manager.create_session_with_config(config).await?;

// Restore session from persistence
let session_id = SessionId::parse_str("550e8400-e29b-41d4-a716-446655440000")?;
let created_at = chrono::Utc::now();
let restored = manager.restore_session(session_id, config, created_at).await?;

// List all sessions
let session_ids = manager.list_sessions();
for id in session_ids {
    println!("Session: {}", id);
}

// Get specific session
if let Some(session) = manager.get_session(&session_id) {
    println!("Found session: {}", session.id);
}

// Remove session
manager.remove_session(&session_id).await?;

// Cleanup terminated sessions
let removed_count = manager.cleanup_terminated().await?;
println!("Cleaned up {} terminated sessions", removed_count);
```

### AISession

Individual session management with AI capabilities.

```rust
use ai_session::{AISession, SessionConfig, SessionStatus};
use std::time::Duration;

// Create session
let session = AISession::new(config).await?;

// Start the session
session.start().await?;

// Check status
match session.status().await {
    SessionStatus::Running => println!("Session is running"),
    SessionStatus::Error => println!("Session has errors"),
    _ => println!("Session status: {:?}", session.status().await),
}

// Send input with error handling
match session.send_input("ls -la\n").await {
    Ok(_) => {
        // Wait for command to execute
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Read output
        let output = session.read_output().await?;
        let output_str = String::from_utf8_lossy(&output);
        println!("Command output:\n{}", output_str);
    }
    Err(e) => eprintln!("Failed to send input: {}", e),
}

// Work with metadata
session.set_metadata("user_id".to_string(), serde_json::json!("user123")).await?;
session.set_metadata("project".to_string(), serde_json::json!({
    "name": "my-app",
    "version": "1.0.0"
})).await?;

// Retrieve metadata
if let Some(user_id) = session.get_metadata("user_id").await {
    println!("User ID: {}", user_id);
}

// Get AI context
let context = session.get_ai_context().await?;
println!("Context messages: {}", context.get_message_count());
println!("Total tokens: {}", context.get_total_tokens());

// Stop session
session.stop().await?;
```

### Session Configuration

Comprehensive configuration options:

```rust
use ai_session::{SessionConfig, ContextConfig};
use std::collections::HashMap;
use std::time::Duration;

let mut config = SessionConfig::default();

// Basic settings
config.name = Some("backend-api-session".to_string());
config.working_directory = "/path/to/backend".into();
config.shell = Some("/bin/zsh".to_string());

// Alternative shell command
config.shell_command = Some("fish".to_string());

// PTY configuration
config.pty_size = (30, 120); // rows, cols
config.output_buffer_size = 4 * 1024 * 1024; // 4MB buffer

// Timeouts
config.timeout = Some(Duration::from_secs(300)); // 5 minute timeout

// Output processing
config.compress_output = true;  // Enable compression
config.parse_output = true;     // Enable semantic parsing

// AI features
config.enable_ai_features = true;
config.agent_role = Some("backend-developer".to_string());

// Advanced context configuration
config.context_config = ContextConfig {
    max_tokens: 8192,           // Increased token limit
    compression_threshold: 0.7,  // Compress when 70% full
};

// Environment variables
config.environment.insert("NODE_ENV".to_string(), "development".to_string());
config.environment.insert("RUST_LOG".to_string(), "debug".to_string());
config.environment.insert("DATABASE_URL".to_string(), "postgresql://localhost/db".to_string());

// Use the configuration
let session = manager.create_session_with_config(config).await?;
```

## HTTP Server API

### Endpoints Reference

#### Health Check
```http
GET /health
```
Response:
```json
{
  "status": "healthy",
  "service": "ai-session-server",
  "version": "0.3.5",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### Session Management

**Create Session**
```http
POST /sessions
Content-Type: application/json

{
  "name": "my-session",
  "enable_ai_features": true,
  "working_directory": "/path/to/project",
  "shell": "/bin/bash"
}
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "my-session",
  "status": "Running",
  "created_at": "2024-01-15T10:30:00Z"
}
```

**List Sessions**
```http
GET /sessions
```

Response:
```json
{
  "sessions": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "my-session",
      "status": "Running",
      "created_at": "2024-01-15T10:30:00Z",
      "last_activity": "2024-01-15T10:35:00Z"
    }
  ],
  "total": 1
}
```

**Get Session Details**
```http
GET /sessions/{name}
```

**Delete Session**
```http
DELETE /sessions/{name}
```

#### Command Execution

**Execute Command**
```http
POST /sessions/{name}/execute
Content-Type: application/json

{
  "command": "cargo build --release",
  "timeout_ms": 30000
}
```

Response:
```json
{
  "success": true,
  "output": "Compiling my-app v1.0.0\nFinished release [optimized] target(s)",
  "error": null,
  "execution_time_ms": 2500
}
```

**Get Session Status**
```http
GET /sessions/{name}/status
```

Response:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "my-session",
  "status": "Running",
  "created_at": "2024-01-15T10:30:00Z",
  "last_activity": "2024-01-15T10:35:00Z",
  "config": {
    "enable_ai_features": true,
    "working_directory": "/path/to/project",
    "pty_size": [24, 80]
  }
}
```

**Get Session Output**
```http
GET /sessions/{name}/output
```

Response:
```json
{
  "session_name": "my-session",
  "output": "Welcome to the session\nLast command output here",
  "raw_output": "Welcome to the session\r\nLast command output here\r\n",
  "timestamp": "2024-01-15T10:35:00Z",
  "size_bytes": 1024
}
```

### HTTP Client Example

```rust
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let base_url = "http://localhost:3000";

    // Create session
    let create_response = client
        .post(&format!("{}/sessions", base_url))
        .json(&json!({
            "name": "rust-dev",
            "enable_ai_features": true,
            "working_directory": "/tmp/rust-project"
        }))
        .send()
        .await?;

    println!("Session created: {}", create_response.status());

    // Execute command
    let exec_response = client
        .post(&format!("{}/sessions/rust-dev/execute", base_url))
        .json(&json!({
            "command": "cargo --version"
        }))
        .send()
        .await?;

    let result: serde_json::Value = exec_response.json().await?;
    println!("Command result: {}", result);

    // Clean up
    client
        .delete(&format!("{}/sessions/rust-dev", base_url))
        .send()
        .await?;

    Ok(())
}
```

## Context Management

### Advanced Context Operations

```rust
use ai_session::context::{SessionContext, Message, MessageRole};
use chrono::Utc;

// Create context
let mut context = SessionContext::new(session_id);

// Add different types of messages
let user_message = Message {
    role: MessageRole::User,
    content: "Implement a REST API for user management".to_string(),
    timestamp: Utc::now(),
    token_count: 8,
};
context.add_message(user_message);

let assistant_message = Message {
    role: MessageRole::Assistant,
    content: "I'll help you implement a REST API. Let me start by creating the basic structure...".to_string(),
    timestamp: Utc::now(),
    token_count: 20,
};
context.add_message(assistant_message);

let tool_message = Message {
    role: MessageRole::Tool,
    content: "cargo build completed successfully".to_string(),
    timestamp: Utc::now(),
    token_count: 5,
};
context.add_message(tool_message);

// Work with message history
println!("Total messages: {}", context.get_message_count());
println!("Total tokens: {}", context.get_total_tokens());

// Get recent messages
let recent = context.get_recent_messages(5);
for msg in recent {
    println!("[{}] {}: {}", 
        msg.timestamp.format("%H:%M:%S"), 
        msg.role, 
        msg.content
    );
}

// Get messages within token limit
let limited_messages = context.get_messages_within_limit(100);
println!("Messages within 100 tokens: {}", limited_messages.len());

// Compress context when needed
if context.get_total_tokens() > context.config.max_tokens {
    let compressed = context.compress_context().await;
    if compressed {
        println!("Context compressed from {} to {} tokens", 
            context.get_total_tokens(), 
            context.get_total_tokens()
        );
    }
}

// Get context summary
let summary = context.get_summary().await;
println!("Summary - Messages: {}, Tokens: {}, Compression: {:.2}%", 
    summary.total_messages,
    summary.total_tokens,
    summary.compression_ratio * 100.0
);
```

### Context Configuration

```rust
use ai_session::context::{ContextConfig, CompressionStrategy};

let context_config = ContextConfig {
    max_tokens: 8192,
    compression_threshold: 0.8,
    compression_strategy: CompressionStrategy::Intelligent,
    preserve_system_messages: true,
    preserve_recent_count: 10,
    summarization_model: Some("gpt-3.5-turbo".to_string()),
};

// Apply to session
let mut session_config = SessionConfig::default();
session_config.context_config = context_config;
```

## Multi-Agent Coordination

### Setting Up Multi-Agent System

```rust
use ai_session::coordination::{
    MultiAgentSession, AgentId, MessageType, BroadcastMessage, 
    TaskDistributor, Task, TaskPriority
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let coordinator = Arc::new(MultiAgentSession::new());
    let manager = SessionManager::new();

    // Create specialized agent sessions
    let frontend_id = AgentId::new();
    let backend_id = AgentId::new();
    let qa_id = AgentId::new();

    // Frontend agent
    let mut frontend_config = SessionConfig::default();
    frontend_config.agent_role = Some("frontend-developer".to_string());
    frontend_config.enable_ai_features = true;
    frontend_config.working_directory = "/project/frontend".into();
    
    let frontend_session = manager.create_session_with_config(frontend_config).await?;
    coordinator.register_agent(frontend_id.clone(), frontend_session)?;

    // Backend agent
    let mut backend_config = SessionConfig::default();
    backend_config.agent_role = Some("backend-developer".to_string());
    backend_config.enable_ai_features = true;
    backend_config.working_directory = "/project/backend".into();
    
    let backend_session = manager.create_session_with_config(backend_config).await?;
    coordinator.register_agent(backend_id.clone(), backend_session)?;

    // QA agent
    let mut qa_config = SessionConfig::default();
    qa_config.agent_role = Some("qa-engineer".to_string());
    qa_config.enable_ai_features = true;
    qa_config.working_directory = "/project".into();
    
    let qa_session = manager.create_session_with_config(qa_config).await?;
    coordinator.register_agent(qa_id.clone(), qa_session)?;

    // Start all sessions
    coordinator.start_all_agents().await?;

    println!("Multi-agent system ready with {} agents", 
        coordinator.get_agent_count());

    Ok(())
}
```

### Message Passing Between Agents

```rust
use ai_session::coordination::{Message, MessageType, MessagePriority};

// Direct message from frontend to backend
let api_request = Message {
    from: frontend_id.clone(),
    message_type: MessageType::DataShare,
    payload: serde_json::json!({
        "request_type": "api_schema",
        "endpoints": [
            {"path": "/api/users", "method": "GET"},
            {"path": "/api/users", "method": "POST"},
            {"path": "/api/users/{id}", "method": "PUT"}
        ]
    }),
    priority: MessagePriority::High,
    timestamp: Utc::now(),
};

coordinator.send_message(frontend_id.clone(), backend_id.clone(), api_request).await?;

// Broadcast to all agents
let build_notification = BroadcastMessage {
    id: uuid::Uuid::new_v4(),
    from: backend_id.clone(),
    content: "Backend API build completed. Ready for frontend integration.".to_string(),
    priority: MessagePriority::Normal,
    timestamp: Utc::now(),
    metadata: Some(serde_json::json!({
        "build_time": "2.3s",
        "tests_passed": 15,
        "coverage": "92%"
    })),
};

coordinator.broadcast(backend_id.clone(), build_notification).await?;

// Send notification to QA
let qa_request = Message {
    from: backend_id.clone(),
    message_type: MessageType::TaskRequest,
    payload: serde_json::json!({
        "task": "integration_testing",
        "description": "Test the new user management API endpoints",
        "priority": "high",
        "estimated_duration": "30m"
    }),
    priority: MessagePriority::High,
    timestamp: Utc::now(),
};

coordinator.send_message(backend_id.clone(), qa_id.clone(), qa_request).await?;
```

### Task Distribution

```rust
use ai_session::coordination::{TaskDistributor, Task, TaskId, TaskPriority};
use std::time::Duration;

let distributor = TaskDistributor::new();

// Register agent capabilities
distributor.register_capabilities(
    frontend_id.clone(),
    vec![
        "react".to_string(),
        "typescript".to_string(),
        "css".to_string(),
        "ui-design".to_string(),
        "accessibility".to_string()
    ]
);

distributor.register_capabilities(
    backend_id.clone(),
    vec![
        "rust".to_string(),
        "api-design".to_string(),
        "database".to_string(),
        "authentication".to_string(),
        "performance".to_string()
    ]
);

distributor.register_capabilities(
    qa_id.clone(),
    vec![
        "testing".to_string(),
        "automation".to_string(),
        "security-testing".to_string(),
        "performance-testing".to_string()
    ]
);

// Create and submit tasks
let tasks = vec![
    Task {
        id: TaskId::new(),
        description: "Create user registration form with validation".to_string(),
        requirements: vec!["react".to_string(), "typescript".to_string()],
        priority: TaskPriority::High,
        estimated_effort: Duration::from_hours(4),
        metadata: serde_json::json!({
            "component_type": "form",
            "validation_rules": ["email", "password_strength"],
            "accessibility_level": "AA"
        }),
    },
    Task {
        id: TaskId::new(),
        description: "Implement user authentication API endpoints".to_string(),
        requirements: vec!["rust".to_string(), "api-design".to_string(), "authentication".to_string()],
        priority: TaskPriority::High,
        estimated_effort: Duration::from_hours(6),
        metadata: serde_json::json!({
            "endpoints": ["/login", "/logout", "/refresh"],
            "auth_method": "JWT",
            "security_level": "high"
        }),
    },
    Task {
        id: TaskId::new(),
        description: "Create comprehensive test suite for user management".to_string(),
        requirements: vec!["testing".to_string(), "automation".to_string()],
        priority: TaskPriority::Medium,
        estimated_effort: Duration::from_hours(8),
        metadata: serde_json::json!({
            "test_types": ["unit", "integration", "e2e"],
            "coverage_target": "95%"
        }),
    },
];

// Submit tasks
for task in tasks {
    distributor.submit_task(task).await?;
}

// Distribute tasks to appropriate agents
let assignments = distributor.distribute_tasks().await?;

for assignment in assignments {
    println!("Task '{}' assigned to agent: {}", 
        assignment.task.description, 
        assignment.agent_id
    );
    
    // Send task to assigned agent
    let task_message = Message {
        from: AgentId::system(), // System message
        message_type: MessageType::TaskAssignment,
        payload: serde_json::to_value(&assignment.task)?,
        priority: MessagePriority::from_task_priority(assignment.task.priority),
        timestamp: Utc::now(),
    };
    
    coordinator.send_message(
        AgentId::system(),
        assignment.agent_id,
        task_message
    ).await?;
}
```

## ccswarm Integration

### Using ai-session in ccswarm

```rust
use ai_session::ccswarm::{SessionManagerAdapter, CcswarmConfig};
use ccswarm::{AgentRole, AutoAcceptConfig};

// Create adapter for ccswarm integration
let workspace_root = PathBuf::from("/path/to/project");
let adapter = SessionManagerAdapter::new(workspace_root);

// Create agent session using ccswarm configuration
let agent_session = adapter.create_agent_session(
    "frontend-agent-001".to_string(),
    AgentRole::Frontend {
        framework: "react".to_string(),
        languages: vec!["typescript".to_string(), "css".to_string()],
    },
    PathBuf::from("/path/to/project/frontend"),
    Some("Frontend development agent specialized in React/TypeScript".to_string()),
    true, // Enable AI features
).await?;

// Configure auto-accept for the agent
let auto_accept_config = AutoAcceptConfig {
    enabled: true,
    risk_threshold: 3,
    protected_patterns: vec![
        "package.json".to_string(),
        "*.env".to_string(),
    ],
};

// Start the agent session
{
    let mut session = agent_session.lock().await;
    session.start().await?;
    session.configure_auto_accept(auto_accept_config);
}

// Execute ccswarm-specific commands
{
    let session = agent_session.lock().await;
    session.execute_command("npm install react@latest").await?;
    session.execute_command("npm run build").await?;
}

// Get session statistics (93% token savings)
{
    let session = agent_session.lock().await;
    let stats = session.get_performance_stats().await;
    println!("Token savings: {:.1}%", stats.token_savings_percentage);
    println!("Context compression ratio: {:.2}", stats.compression_ratio);
    println!("Commands executed: {}", stats.commands_executed);
}
```

### ccswarm-specific Context Extensions

```rust
use ai_session::{SessionContext, ContextMessage, MessageRole};

// Create context with ccswarm extensions
let mut context = SessionContext::new(session_id);

// Add ccswarm-specific message
let ccswarm_message = ContextMessage {
    role: MessageRole::System,
    content: "You are a frontend specialist agent in the ccswarm system. Focus on React/TypeScript development.".to_string(),
    timestamp: Utc::now(),
    token_count: 25,
};
context.add_message(ccswarm_message);

// Use ccswarm-specific context methods
println!("Message count: {}", context.get_message_count());
println!("Total tokens: {}", context.get_total_tokens());

// Get recent messages for ccswarm coordination
let recent_messages = context.get_recent_messages(10);
for msg in recent_messages {
    println!("{}: {}", msg.role, msg.content);
}

// Compress context using ccswarm's intelligent algorithm
let was_compressed = context.compress_context().await;
if was_compressed {
    println!("Context compressed for token efficiency");
}
```

## Examples by Use Case

### Use Case 1: CI/CD Pipeline Integration

```rust
use ai_session::{SessionManager, SessionConfig};

async fn run_ci_pipeline() -> Result<()> {
    let manager = SessionManager::new();
    
    // Create CI session
    let mut config = SessionConfig::default();
    config.name = Some("ci-pipeline".to_string());
    config.working_directory = "/ci/workspace".into();
    config.enable_ai_features = false; // CI doesn't need AI features
    config.timeout = Some(Duration::from_secs(1800)); // 30 minutes
    
    let session = manager.create_session_with_config(config).await?;
    session.start().await?;
    
    // Run pipeline steps
    let steps = vec![
        "git clone https://github.com/user/repo.git .",
        "cargo build --release",
        "cargo test",
        "cargo clippy -- -D warnings",
        "cargo fmt -- --check",
    ];
    
    for (i, step) in steps.iter().enumerate() {
        println!("Running step {}: {}", i + 1, step);
        
        session.send_input(&format!("{}\n", step)).await?;
        
        // Wait for completion
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        let output = session.read_output().await?;
        let output_str = String::from_utf8_lossy(&output);
        
        // Check for failure
        if output_str.contains("error") || output_str.contains("failed") {
            eprintln!("Step {} failed: {}", i + 1, output_str);
            break;
        }
        
        println!("Step {} completed successfully", i + 1);
    }
    
    session.stop().await?;
    Ok(())
}
```

### Use Case 2: Development Environment Setup

```rust
async fn setup_dev_environment() -> Result<()> {
    let manager = SessionManager::new();
    
    // Create development session
    let mut config = SessionConfig::default();
    config.name = Some("dev-setup".to_string());
    config.enable_ai_features = true;
    config.agent_role = Some("development-assistant".to_string());
    
    let session = manager.create_session_with_config(config).await?;
    session.start().await?;
    
    // Development setup commands
    let setup_commands = vec![
        // Rust setup
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y",
        "source ~/.cargo/env",
        "rustup update",
        "rustup component add clippy rustfmt",
        
        // Node.js setup
        "curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -",
        "sudo apt-get install -y nodejs",
        "npm install -g typescript ts-node",
        
        // Project initialization
        "cargo new my-project --bin",
        "cd my-project",
        "cargo add tokio --features full",
        "cargo add serde --features derive",
    ];
    
    for cmd in setup_commands {
        println!("Executing: {}", cmd);
        session.send_input(&format!("{}\n", cmd)).await?;
        
        // Wait and check output
        tokio::time::sleep(Duration::from_secs(3)).await;
        let output = session.read_output().await?;
        println!("Output: {}", String::from_utf8_lossy(&output));
    }
    
    session.stop().await?;
    Ok(())
}
```

### Use Case 3: Multi-Language Project Management

```rust
async fn manage_polyglot_project() -> Result<()> {
    let coordinator = Arc::new(MultiAgentSession::new());
    let manager = SessionManager::new();
    
    // Rust backend agent
    let rust_agent = {
        let mut config = SessionConfig::default();
        config.agent_role = Some("rust-backend".to_string());
        config.working_directory = "/project/backend".into();
        config.enable_ai_features = true;
        
        let session = manager.create_session_with_config(config).await?;
        let agent_id = AgentId::new();
        coordinator.register_agent(agent_id.clone(), session)?;
        agent_id
    };
    
    // TypeScript frontend agent
    let ts_agent = {
        let mut config = SessionConfig::default();
        config.agent_role = Some("typescript-frontend".to_string());
        config.working_directory = "/project/frontend".into();
        config.enable_ai_features = true;
        
        let session = manager.create_session_with_config(config).await?;
        let agent_id = AgentId::new();
        coordinator.register_agent(agent_id.clone(), session)?;
        agent_id
    };
    
    // Python ML agent
    let python_agent = {
        let mut config = SessionConfig::default();
        config.agent_role = Some("python-ml".to_string());
        config.working_directory = "/project/ml".into();
        config.enable_ai_features = true;
        
        let session = manager.create_session_with_config(config).await?;
        let agent_id = AgentId::new();
        coordinator.register_agent(agent_id.clone(), session)?;
        agent_id
    };
    
    // Start all agents
    coordinator.start_all_agents().await?;
    
    // Coordinate development tasks
    let schema_update = Message {
        from: rust_agent.clone(),
        message_type: MessageType::DataShare,
        payload: serde_json::json!({
            "schema_version": "2.0",
            "new_endpoints": ["/api/predictions", "/api/models"],
            "breaking_changes": []
        }),
        priority: MessagePriority::High,
        timestamp: Utc::now(),
    };
    
    // Notify frontend of API changes
    coordinator.send_message(rust_agent.clone(), ts_agent.clone(), schema_update.clone()).await?;
    
    // Notify ML service of new API
    coordinator.send_message(rust_agent.clone(), python_agent.clone(), schema_update).await?;
    
    // Broadcast build completion
    let build_complete = BroadcastMessage {
        id: uuid::Uuid::new_v4(),
        from: rust_agent.clone(),
        content: "Backend API v2.0 deployed successfully".to_string(),
        priority: MessagePriority::Normal,
        timestamp: Utc::now(),
        metadata: Some(serde_json::json!({
            "version": "2.0.0",
            "deployment_time": "45s",
            "health_check": "passed"
        })),
    };
    
    coordinator.broadcast(rust_agent, build_complete).await?;
    
    Ok(())
}
```

## Performance Optimizations

### Session Pooling

```rust
use ai_session::{SessionManager, SessionConfig};
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct SessionPool {
    manager: SessionManager,
    config: SessionConfig,
    semaphore: Arc<Semaphore>,
    max_sessions: usize,
}

impl SessionPool {
    pub fn new(max_sessions: usize, config: SessionConfig) -> Self {
        Self {
            manager: SessionManager::new(),
            config,
            semaphore: Arc::new(Semaphore::new(max_sessions)),
            max_sessions,
        }
    }
    
    pub async fn acquire(&self) -> Result<Arc<AISession>> {
        // Wait for available slot
        let _permit = self.semaphore.acquire().await?;
        
        // Create new session
        let session = self.manager.create_session_with_config(self.config.clone()).await?;
        session.start().await?;
        
        Ok(session)
    }
    
    pub async fn execute_with_session<F, R>(&self, task: F) -> Result<R>
    where
        F: FnOnce(Arc<AISession>) -> futures::future::BoxFuture<'static, Result<R>>,
        R: Send + 'static,
    {
        let session = self.acquire().await?;
        let result = task(session.clone()).await?;
        
        // Session is automatically cleaned up when dropped
        session.stop().await?;
        
        Ok(result)
    }
}

// Usage example
async fn use_session_pool() -> Result<()> {
    let config = SessionConfig {
        enable_ai_features: true,
        context_config: ContextConfig {
            max_tokens: 4096,
            compression_threshold: 0.8,
        },
        ..Default::default()
    };
    
    let pool = SessionPool::new(5, config);
    
    // Execute task with pooled session
    let result = pool.execute_with_session(|session| {
        Box::pin(async move {
            session.send_input("cargo build\n").await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
            let output = session.read_output().await?;
            Ok(String::from_utf8_lossy(&output).to_string())
        })
    }).await?;
    
    println!("Build output: {}", result);
    Ok(())
}
```

### Context Optimization

```rust
use ai_session::context::{SessionContext, CompressionStrategy};

async fn optimize_context_usage() -> Result<()> {
    let mut context = SessionContext::new(session_id);
    
    // Configure aggressive compression for high-throughput scenarios
    context.config.max_tokens = 4096;
    context.config.compression_threshold = 0.6; // Compress earlier
    context.config.compression_strategy = CompressionStrategy::Aggressive;
    
    // Add messages with token estimation
    let messages = vec![
        ("System", "You are a helpful assistant", 6),
        ("User", "How do I optimize Rust performance?", 7),
        ("Assistant", "Here are several Rust optimization techniques...", 45),
        ("Tool", "cargo build --release completed in 2.3s", 8),
    ];
    
    for (role, content, estimated_tokens) in messages {
        let message = Message {
            role: MessageRole::from_str(role)?,
            content: content.to_string(),
            timestamp: Utc::now(),
            token_count: estimated_tokens,
        };
        
        context.add_message(message);
        
        // Auto-compress when approaching limit
        if context.get_total_tokens() > (context.config.max_tokens as f64 * context.config.compression_threshold) as usize {
            context.compress_context().await;
        }
    }
    
    println!("Final token count: {}", context.get_total_tokens());
    println!("Compression ratio: {:.2}", context.get_compression_ratio());
    
    Ok(())
}
```

### Batch Operations

```rust
async fn batch_command_execution() -> Result<()> {
    let session = create_optimized_session().await?;
    
    let commands = vec![
        "echo 'Starting batch operations'",
        "cargo fmt",
        "cargo clippy",
        "cargo test",
        "cargo build --release",
        "echo 'Batch operations completed'",
    ];
    
    // Send all commands in batch
    let batch_command = commands.join(" && ");
    session.send_input(&format!("{}\n", batch_command)).await?;
    
    // Wait for completion with progress monitoring
    let mut total_output = String::new();
    let start_time = std::time::Instant::now();
    
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        let output = session.read_output().await?;
        let output_str = String::from_utf8_lossy(&output);
        total_output.push_str(&output_str);
        
        // Check for completion
        if output_str.contains("Batch operations completed") {
            break;
        }
        
        // Timeout protection
        if start_time.elapsed() > Duration::from_secs(300) {
            eprintln!("Batch operation timeout");
            break;
        }
    }
    
    println!("Batch completed in {:?}", start_time.elapsed());
    println!("Total output length: {} chars", total_output.len());
    
    Ok(())
}
```

## Troubleshooting

### Common Issues and Solutions

#### Session Won't Start
```rust
async fn troubleshoot_session_start() -> Result<()> {
    let session = manager.create_session().await?;
    
    match session.start().await {
        Ok(_) => println!("Session started successfully"),
        Err(e) => {
            eprintln!("Failed to start session: {}", e);
            
            // Check common issues
            
            // 1. Working directory exists
            if !session.config.working_directory.exists() {
                eprintln!("Working directory does not exist: {:?}", 
                    session.config.working_directory);
                std::fs::create_dir_all(&session.config.working_directory)?;
                
                // Retry
                session.start().await?;
            }
            
            // 2. Shell exists
            if let Some(shell) = &session.config.shell {
                if !std::path::Path::new(shell).exists() {
                    eprintln!("Shell not found: {}", shell);
                    // Fall back to default shell
                    let mut new_config = session.config.clone();
                    new_config.shell = None; // Use default
                    
                    let new_session = manager.create_session_with_config(new_config).await?;
                    new_session.start().await?;
                }
            }
            
            // 3. PTY size issues
            if session.config.pty_size.0 == 0 || session.config.pty_size.1 == 0 {
                eprintln!("Invalid PTY size: {:?}", session.config.pty_size);
                let mut new_config = session.config.clone();
                new_config.pty_size = (24, 80); // Safe default
                
                let new_session = manager.create_session_with_config(new_config).await?;
                new_session.start().await?;
            }
        }
    }
    
    Ok(())
}
```

#### High Memory Usage
```rust
async fn monitor_memory_usage() -> Result<()> {
    let session = create_session().await?;
    
    // Monitor memory usage
    let memory_monitor = tokio::spawn(async move {
        loop {
            let context = session.get_ai_context().await.unwrap();
            let token_count = context.get_total_tokens();
            let message_count = context.get_message_count();
            
            if token_count > 6000 {
                println!("High token usage: {} tokens, {} messages", 
                    token_count, message_count);
                
                // Force compression
                context.compress_context().await;
                
                let new_token_count = context.get_total_tokens();
                println!("After compression: {} tokens", new_token_count);
            }
            
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
    
    // Your main logic here
    // ...
    
    memory_monitor.abort();
    Ok(())
}
```

#### Connection Issues with HTTP Server
```rust
async fn test_http_server_connection() -> Result<()> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    
    let base_url = "http://localhost:3000";
    
    // Test health endpoint
    match client.get(&format!("{}/health", base_url)).send().await {
        Ok(response) => {
            if response.status().is_success() {
                println!("Server is healthy");
            } else {
                eprintln!("Server returned error: {}", response.status());
            }
        }
        Err(e) => {
            eprintln!("Cannot connect to server: {}", e);
            eprintln!("Make sure ai-session-server is running on port 3000");
            eprintln!("Start with: ai-session-server --port 3000");
        }
    }
    
    Ok(())
}
```

### Debug Configuration

```rust
use ai_session::{SessionConfig, ContextConfig};

fn create_debug_config() -> SessionConfig {
    let mut config = SessionConfig::default();
    
    // Enable detailed logging
    config.environment.insert("RUST_LOG".to_string(), "debug".to_string());
    config.environment.insert("AI_SESSION_DEBUG".to_string(), "1".to_string());
    
    // Increase buffer sizes for debugging
    config.output_buffer_size = 10 * 1024 * 1024; // 10MB
    
    // Reduce compression threshold for testing
    config.context_config.compression_threshold = 0.5;
    
    // Shorter timeout for faster feedback
    config.timeout = Some(Duration::from_secs(60));
    
    config
}
```

## Best Practices

1. **Always clean up sessions** - Use RAII or explicit cleanup
2. **Monitor token usage** - Set up compression thresholds appropriately
3. **Use session pooling** for high-throughput applications
4. **Enable AI features** only when needed to save resources
5. **Set appropriate timeouts** to prevent hanging operations
6. **Use structured logging** for better observability
7. **Test with realistic data sizes** to avoid production surprises
8. **Configure compression** based on your token budget
9. **Use the HTTP API** for language-agnostic integration
10. **Monitor performance metrics** regularly

For more detailed examples and advanced usage patterns, see the examples directory and integration tests in the ai-session crate.