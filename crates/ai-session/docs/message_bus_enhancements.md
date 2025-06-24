# MessageBus Enhancements for ccswarm Integration

This document describes the MessageBus enhancements implemented for ccswarm integration in the ai-session library.

## Overview

The MessageBus has been enhanced with additional functionality to support ccswarm's multi-agent orchestration requirements. These enhancements enable more sophisticated agent communication patterns and monitoring capabilities.

## New Features

### 1. AgentMessage Enum

A new comprehensive message type system has been added to support all ccswarm communication patterns:

```rust
pub enum AgentMessage {
    /// Agent registration with capabilities
    Registration {
        agent_id: AgentId,
        capabilities: Vec<String>,
        metadata: serde_json::Value,
    },
    
    /// Task assignment to specific agent
    TaskAssignment {
        task_id: TaskId,
        agent_id: AgentId,
        task_data: serde_json::Value,
    },
    
    /// Task completion notification
    TaskCompleted {
        agent_id: AgentId,
        task_id: TaskId,
        result: serde_json::Value,
    },
    
    /// Progress updates during task execution
    TaskProgress {
        agent_id: AgentId,
        task_id: TaskId,
        progress: f32,  // 0.0 to 1.0
        message: String,
    },
    
    /// Agent requesting help from others
    HelpRequest {
        agent_id: AgentId,
        context: String,
        priority: MessagePriority,
    },
    
    /// General status updates
    StatusUpdate {
        agent_id: AgentId,
        status: String,
        metrics: serde_json::Value,
    },
    
    /// Extensible custom messages
    Custom {
        message_type: String,
        data: serde_json::Value,
    },
}
```

### 2. Subscribe All Method

Monitor all messages across the system:

```rust
/// Subscribe to all messages (for monitoring)
pub fn subscribe_all(&self) -> Receiver<AgentMessage>
```

This method returns a receiver that gets a copy of every `AgentMessage` sent through the system, enabling:
- System-wide monitoring and logging
- Analytics and performance tracking
- Debugging multi-agent interactions
- Audit trails

### 3. Publish to Agent Method

Send messages directly to specific agents:

```rust
/// Publish a message to a specific agent
pub async fn publish_to_agent(&self, agent_id: &AgentId, message: AgentMessage) -> Result<()>
```

Features:
- Sends the message to the specified agent's dedicated channel
- Also sends a copy to the all-messages channel for monitoring
- Returns an error if the agent is not registered
- Async for non-blocking operation

## Usage Examples

### Basic Registration and Task Flow

```rust
use ai_session::{AgentId, AgentMessage, MultiAgentSession, TaskId};

// Create multi-agent session
let multi_session = MultiAgentSession::new();
let message_bus = &multi_session.message_bus;

// Register agent
let agent_id = AgentId::new();
multi_session.register_agent(agent_id.clone(), session)?;

// Agent announces its capabilities
message_bus.publish_to_agent(
    &agent_id,
    AgentMessage::Registration {
        agent_id: agent_id.clone(),
        capabilities: vec!["frontend".to_string(), "react".to_string()],
        metadata: serde_json::json!({ "version": "1.0" }),
    }
).await?;

// Assign a task
let task_id = TaskId::new();
message_bus.publish_to_agent(
    &agent_id,
    AgentMessage::TaskAssignment {
        task_id: task_id.clone(),
        agent_id: agent_id.clone(),
        task_data: serde_json::json!({
            "type": "build_component",
            "name": "UserDashboard"
        }),
    }
).await?;

// Report progress
message_bus.publish_to_agent(
    &agent_id,
    AgentMessage::TaskProgress {
        agent_id: agent_id.clone(),
        task_id: task_id.clone(),
        progress: 0.5,
        message: "Component structure complete".to_string(),
    }
).await?;

// Complete task
message_bus.publish_to_agent(
    &agent_id,
    AgentMessage::TaskCompleted {
        agent_id: agent_id.clone(),
        task_id: task_id.clone(),
        result: serde_json::json!({
            "success": true,
            "files": ["UserDashboard.tsx", "UserDashboard.css"]
        }),
    }
).await?;
```

### Monitoring All Messages

```rust
// Subscribe to all messages
let all_messages = message_bus.subscribe_all();

// Spawn monitoring task
tokio::spawn(async move {
    while let Ok(msg) = all_messages.recv() {
        match msg {
            AgentMessage::Registration { agent_id, capabilities, .. } => {
                log::info!("Agent {} registered with capabilities: {:?}", 
                          agent_id, capabilities);
            }
            AgentMessage::TaskCompleted { agent_id, task_id, .. } => {
                log::info!("Task {} completed by agent {}", task_id, agent_id);
            }
            AgentMessage::HelpRequest { agent_id, context, priority } => {
                log::warn!("Agent {} needs help (priority {:?}): {}", 
                          agent_id, priority, context);
            }
            // Handle other message types...
            _ => {}
        }
    }
});
```

### Inter-Agent Collaboration

```rust
// Frontend agent needs help
message_bus.publish_to_agent(
    &backend_agent_id,
    AgentMessage::HelpRequest {
        agent_id: frontend_agent_id.clone(),
        context: "Need API endpoint for user authentication".to_string(),
        priority: MessagePriority::High,
    }
).await?;

// Backend agent responds with custom message
message_bus.publish_to_agent(
    &frontend_agent_id,
    AgentMessage::Custom {
        message_type: "api_specification".to_string(),
        data: serde_json::json!({
            "endpoint": "/api/v1/auth",
            "methods": ["POST"],
            "schema": { /* ... */ }
        }),
    }
).await?;
```

## Integration with ccswarm

These enhancements enable ccswarm to:

1. **Track Agent Capabilities**: Agents can register their skills and specializations
2. **Coordinate Task Distribution**: Master can assign tasks based on agent capabilities
3. **Monitor Progress**: Real-time tracking of task execution across all agents
4. **Enable Collaboration**: Agents can request help and share information
5. **Collect Metrics**: System-wide monitoring for performance and debugging

## Performance Considerations

- Messages are sent through unbounded channels for minimal latency
- The `subscribe_all()` receiver gets cloned messages to avoid blocking
- Async operations prevent blocking on message sends
- DashMap provides thread-safe concurrent access to agent channels

## Future Enhancements

Potential future improvements could include:
- Message persistence for replay and recovery
- Priority queues for message delivery
- Message filtering and routing rules
- Dead letter queues for failed deliveries
- Message batching for efficiency