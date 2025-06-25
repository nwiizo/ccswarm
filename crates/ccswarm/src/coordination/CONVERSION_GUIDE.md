# CCSwarm <-> AI-Session Message Conversion Guide

## Overview

The `conversion.rs` module provides seamless bidirectional conversion between ccswarm's `AgentMessage` and ai-session's `AgentMessage` types. This enables ccswarm to leverage ai-session's high-performance message bus while maintaining backward compatibility with existing ccswarm code.

## Key Components

### 1. Type Aliases

To avoid naming conflicts, we use type aliases:

```rust
use ai_session::coordination::AgentMessage as AISessionMessage;
use crate::coordination::AgentMessage as CCSwarmMessage;
```

### 2. UnifiedAgentInfo

A bridge structure that maintains mappings between ccswarm and ai-session agent identities:

```rust
pub struct UnifiedAgentInfo {
    pub ccswarm_id: String,              // e.g., "frontend-specialist"
    pub ai_session_id: AISessionAgentId, // UUID-based ID
    pub role: AgentRole,                 // Frontend, Backend, DevOps, QA
    pub capabilities: Vec<String>,       // Agent capabilities
    pub metadata: serde_json::Value,     // Additional info
}
```

### 3. AgentMappingRegistry

Maintains bidirectional mappings between agent IDs:

```rust
let registry = AgentMappingRegistry::new();
registry.register(agent_info).await;

// Look up IDs in either direction
let ai_id = registry.get_ai_session_id("frontend-specialist").await;
let ccswarm_id = registry.get_ccswarm_id(&ai_session_id).await;
```

### 4. Conversion Functions

Helper functions for easy message conversion:

```rust
// CCSwarm to AI-Session
let ai_msg = convert_to_ai_session(ccswarm_msg, &registry).await?;

// AI-Session to CCSwarm
let ccswarm_msg = convert_from_ai_session(ai_msg, &registry).await?;
```

## Message Type Mappings

| CCSwarm Message | AI-Session Message | Notes |
|-----------------|-------------------|-------|
| TaskCompleted | TaskCompleted | Direct mapping with result conversion |
| StatusUpdate | StatusUpdate | Status enum converted to string |
| RequestAssistance | HelpRequest | Task ID not preserved (AI-session doesn't include it) |
| Heartbeat | StatusUpdate | Mapped as status="heartbeat" |
| InterAgentMessage | Custom | Preserved in Custom message payload |
| TaskProgress | *(Not in CCSwarm)* | Mapped to InterAgentMessage |

## Usage Example

```rust
// Set up registry and register agents
let registry = AgentMappingRegistry::new();
let agent_info = UnifiedAgentInfo::from_ccswarm_agent(&agent);
registry.register(agent_info).await;

// Convert messages seamlessly
let ccswarm_msg = CCSwarmMessage::StatusUpdate {
    agent_id: "frontend-specialist".to_string(),
    status: AgentStatus::Working,
    metrics: json!({"tasks": 5}),
};

let ai_msg = convert_to_ai_session(ccswarm_msg, &registry).await?;
// Now ai_msg can be sent through ai-session's MessageBus
```

## Integration with AI-Session MessageBus

The conversion module integrates with `ai_message_bus.rs` to provide high-performance message routing:

```rust
// In AICoordinationBridge
let ai_msg = convert_to_ai_session(ccswarm_msg, &self.agent_registry).await?;
self.message_bus.publish_to_agent(&target_agent_id, ai_msg).await?;
```

## Error Handling

The module defines `ConversionError` for handling conversion failures:

- `IncompatibleType`: Message types that can't be converted
- `MissingField`: Required field not present
- `InvalidField`: Field contains invalid value
- `SerializationError`: JSON serialization failures

## Best Practices

1. **Always register agents** before attempting message conversion
2. **Handle missing mappings** gracefully - agents might not be registered yet
3. **Preserve metadata** when possible for debugging and tracing
4. **Use helper functions** instead of trait methods for cleaner code
5. **Test round-trip conversions** to ensure data integrity

## Testing

Run the comprehensive test suite:

```bash
cargo test -p ccswarm conversion::tests
```

Run the demo example:

```bash
cargo run --example conversion_demo
```