# ccswarm Integration API Documentation

This document describes the API extensions added to ai-session for ccswarm integration.

## SessionContext API Extensions

The `SessionContext` struct now provides the following additional methods required by ccswarm:

### 1. Message Count
```rust
pub fn get_message_count(&self) -> usize
```
Returns the total number of messages in the conversation history.

### 2. Total Token Count
```rust
pub fn get_total_tokens(&self) -> usize
```
Returns the current estimated token count for all messages in the context.

### 3. Recent Messages
```rust
pub fn get_recent_messages(&self, n: usize) -> Vec<&Message>
```
Returns the last `n` messages from the conversation history. If `n` is greater than the total number of messages, returns all messages.

### 4. Context Compression
```rust
pub async fn compress_context(&mut self) -> bool
```
Attempts to compress the context if it exceeds the configured token limit. Returns `true` if compression occurred, `false` otherwise.

### 5. New Message API
```rust
pub fn add_message(&mut self, message: Message)
```
Adds a message using the `Message` struct directly. This is the preferred method for ccswarm integration.

The legacy method is still available:
```rust
pub fn add_message_raw(&mut self, role: MessageRole, content: String)
```

### 6. Session Configuration
The `SessionContext` now includes a public `config` field:
```rust
pub struct SessionContext {
    // ... other fields ...
    pub config: SessionConfig,
}

pub struct SessionConfig {
    pub max_tokens: usize,
    // Other configuration options can be added here
}
```

## Message Structure

The `Message` struct has all public fields as required:

```rust
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub token_count: usize,
}

pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}
```

## Usage Example

```rust
use ai_session::{SessionContext, ContextMessage, MessageRole, SessionId};
use chrono::Utc;

// Create a new session context
let mut context = SessionContext::new(SessionId::new());

// Add a message using the new API
let message = ContextMessage {
    role: MessageRole::User,
    content: "Hello, AI!".to_string(),
    timestamp: Utc::now(),
    token_count: 3,
};
context.add_message(message);

// Get message count and tokens
println!("Messages: {}", context.get_message_count());
println!("Total tokens: {}", context.get_total_tokens());

// Get recent messages
let recent = context.get_recent_messages(5);
for msg in recent {
    println!("{:?}: {}", msg.role, msg.content);
}

// Compress context if needed
let compressed = context.compress_context().await;
if compressed {
    println!("Context was compressed to fit token limit");
}

// Access configuration
println!("Max tokens: {}", context.config.max_tokens);
```

## Exports

All required types are exported from the main library:

```rust
pub use ai_session::{
    SessionContext,
    ContextMessage,  // Message from context module
    MessageRole,
    ContextSessionConfig,  // SessionConfig from context module
};
```

Note: There are two different `Message` types in ai-session:
- `ContextMessage` - For conversation history in SessionContext
- `CoordinationMessage` - For inter-agent communication in MessageBus

Make sure to use `ContextMessage` when working with SessionContext.

## Related Documentation

### ccswarm Documentation
- **[ccswarm Documentation Hub](../../../docs/README.md)** - Master documentation index
- **[ccswarm Architecture](../../../docs/ARCHITECTURE.md)** - Overall system design
- **[Session Commands](../../../.claude/commands/session.md)** - ccswarm session management
- **[Task Management](../../../.claude/commands/task.md)** - How tasks use sessions

### AI-Session Documentation
- **[AI-Session Documentation Hub](README.md)** - Complete AI-Session documentation
- **[API Guide](API_GUIDE.md)** - Complete API reference
- **[Architecture](ARCHITECTURE.md)** - AI-Session system design
- **[CLI Guide](CLI_GUIDE.md)** - Standalone AI-Session commands

### Development Resources
- **[Project Rules](../../../.claude/commands/project-rules.md)** - Coding standards
- **[Development Guide](../../../docs/DEVELOPER_GUIDE.md)** - Development workflows