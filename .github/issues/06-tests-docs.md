# Issue #6: Day 9 - Unit Tests and Documentation

## Overview
Create tests and documentation for Claude Code ACP integration.

## Task List

### 1. Unit Tests
- [ ] Tests for `adapter.rs`:
```rust
#[cfg(test)]
mod adapter_tests {
    #[tokio::test]
    async fn test_adapter_creation() { }

    #[tokio::test]
    async fn test_connect() { }

    #[tokio::test]
    async fn test_send_task() { }

    #[tokio::test]
    async fn test_retry_logic() { }
}
```

### 2. Configuration Tests
- [ ] Configuration file loading tests
- [ ] Environment variable loading tests
- [ ] Default value tests

### 3. Mock Server
- [ ] Create test mock server
- [ ] Implement various response patterns
- [ ] Simulate error cases

### 4. Documentation
- [ ] Update README.md
- [ ] API documentation (rustdoc)
- [ ] Add usage examples
- [ ] Troubleshooting guide

### 5. Configuration Examples
- [ ] Create `.ccswarm.yml.example`
- [ ] Add environment variable examples to `.env.example`

## Acceptance Criteria
- [ ] `cargo test --features claude-acp` succeeds
- [ ] Test coverage 80%+
- [ ] `cargo doc --features claude-acp` generates documentation
- [ ] README.md has usage instructions

## Estimated Time
6-8 hours

## Labels
- `task`
- `day-9`
- `testing`
- `documentation`
- `claude-acp`
