# Message Conversion Design for MasterClaude Integration

## Overview

This document describes the type conversion strategy implemented to enable MasterClaude integration with ai-session's coordination system. The design prioritizes minimal breaking changes while enabling full ai-session capabilities.

## Key Components

### 1. AgentMessage Enum Mapping

The two systems have different AgentMessage enums with limited overlap:

**CCSwarm AgentMessage variants:**
- StatusUpdate
- TaskCompleted
- RequestAssistance
- QualityIssue
- Coordination
- Heartbeat
- InterAgentMessage
- TaskGenerated

**AI-Session AgentMessage variants:**
- Registration
- TaskAssignment
- TaskCompleted
- TaskProgress
- HelpRequest
- StatusUpdate
- Custom

Only `TaskCompleted` and `StatusUpdate` have direct equivalents.

### 2. Conversion Module (`coordination/conversion.rs`)

The conversion module provides:

#### Type Aliases
```rust
use ai_session::coordination::AgentMessage as AISessionMessage;
use crate::coordination::AgentMessage as CCSwarmMessage;
```

#### Unified Agent Information
```rust
pub struct UnifiedAgentInfo {
    pub ccswarm_id: String,
    pub ai_session_id: AISessionAgentId,
    pub role: AgentRole,
    pub capabilities: Vec<String>,
    pub metadata: serde_json::Value,
}
```

#### Agent Mapping Registry
```rust
pub struct AgentMappingRegistry {
    ccswarm_to_ai: Arc<RwLock<HashMap<String, AISessionAgentId>>>,
    ai_to_ccswarm: Arc<RwLock<HashMap<AISessionAgentId, String>>>,
    agent_info: Arc<RwLock<HashMap<String, UnifiedAgentInfo>>>,
}
```

#### Conversion Traits
```rust
pub trait IntoAISessionMessage {
    fn into_ai_session(self, registry: &AgentMappingRegistry) 
        -> impl Future<Output = Result<AISessionMessage, ConversionError>>;
}

pub trait FromAISessionMessage: Sized {
    fn from_ai_session(msg: AISessionMessage, registry: &AgentMappingRegistry) 
        -> impl Future<Output = Result<Self, ConversionError>>;
}
```

### 3. Agent Attribute Access (`orchestrator/agent_access.rs`)

Standardizes how agent properties are accessed:

```rust
pub trait AgentAttributeAccess {
    fn agent_id(&self) -> &str;
    fn role(&self) -> &AgentRole;
    fn specialization(&self) -> &str;
    fn capabilities(&self) -> Vec<String>;
    fn has_capability(&self, capability: &str) -> bool;
}
```

This resolves the difference between:
- CCSwarm: `agent.identity.agent_id`, `agent.identity.specialization`
- AI-Session: Expects separate agent ID and capabilities

### 4. Message Conversion Examples

#### Direct Mappings
- `TaskCompleted` ↔ `TaskCompleted` (with field conversions)
- `StatusUpdate` ↔ `StatusUpdate` (with status string conversion)
- `RequestAssistance` → `HelpRequest`
- `HelpRequest` → `RequestAssistance`

#### Custom Message Handling
Messages without direct equivalents are wrapped:
- CCSwarm → AI-Session: Wrapped in `Custom` variant
- AI-Session → CCSwarm: Wrapped in `Coordination` variant with `Custom` type

### 5. Integration in AI Message Bus

The `ai_message_bus.rs` module is updated to:
- Use type aliases to avoid conflicts
- Replace HashMap-based agent mapping with `AgentMappingRegistry`
- Convert messages at boundaries between systems
- Maintain backward compatibility

## Implementation Benefits

1. **No Breaking Changes**: Existing CCSwarm code continues to work
2. **Clear Boundaries**: Explicit conversions make data flow visible
3. **Error Handling**: Conversion errors are caught and handled
4. **Extensibility**: New message types can be added easily
5. **Performance**: Minimal overhead from conversions

## Usage Example

```rust
// Register agent with unified info
let agent_info = UnifiedAgentInfo::from_ccswarm_agent(&agent);
registry.register(agent_info).await;

// Convert CCSwarm message to AI-Session
let ai_msg = ccswarm_msg.into_ai_session(&registry).await?;

// Send through AI-Session message bus
message_bus.publish_to_agent(&ai_agent_id, ai_msg).await?;

// Receive and convert back
let received = message_bus.get_agent_receiver(&ai_agent_id).try_recv()?;
let ccswarm_msg = CCSwarmMessage::from_ai_session(received, &registry).await?;
```

## Future Enhancements

1. **Optimize Conversions**: Cache frequently used conversions
2. **Message Batching**: Convert multiple messages efficiently
3. **Schema Evolution**: Version the conversion logic
4. **Metrics**: Track conversion performance and errors
5. **Direct Integration**: Gradually migrate to native ai-session messages

## Testing

The conversion layer includes comprehensive tests:
- Bidirectional conversion tests
- Error handling tests
- Agent mapping tests
- Integration tests with message bus

See `tests/test_message_conversion.rs` for examples.