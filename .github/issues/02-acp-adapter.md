# Issue #2: Day 2-3 - Claude Code ACP Adapter Implementation

## Overview
Implement the ACP adapter for communicating with Claude Code.

## Task List

### 1. Implement ClaudeCodeAdapter
- [ ] Implement the following in `adapter.rs`:
```rust
pub struct ClaudeCodeAdapter {
    connection: Option<ClientSideConnection>,
    session_id: Option<String>,
    config: ClaudeACPConfig,
}
```

### 2. Connection Management
- [ ] Implement `connect()` method
  - [ ] Establish WebSocket connection
  - [ ] Initialize ACP protocol
  - [ ] Create session
- [ ] Implement `disconnect()` method
- [ ] Implement `is_connected()` method

### 3. Task Sending
- [ ] Implement `send_task()` method
  - [ ] Create PromptRequest
  - [ ] Process response
  - [ ] Error handling

### 4. Auto-retry Functionality
- [ ] Implement `connect_with_retry()` method
- [ ] Implement exponential backoff
- [ ] Timeout handling

### 5. Configuration Management
- [ ] Load configuration from environment variables
- [ ] Set default values
- [ ] Configuration validation

## Test Cases
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_adapter_creation() { /* ... */ }

    #[tokio::test]
    async fn test_config_from_env() { /* ... */ }

    #[tokio::test]
    async fn test_retry_logic() { /* ... */ }
}
```

## Acceptance Criteria
- [ ] Adapter initializes correctly
- [ ] Connection to mock server succeeds
- [ ] Task send/receive works
- [ ] Retry logic functions

## Estimated Time
8-12 hours

## Labels
- `task`
- `day-2-3`
- `implementation`
- `claude-acp`
