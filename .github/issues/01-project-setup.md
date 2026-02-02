# Issue #1: Day 1 - Project Setup and Dependency Addition

## Overview
Perform basic setup for Claude Code ACP integration.

## Task List

### 1. Add Dependencies
- [ ] Add the following to `Cargo.toml`:
```toml
[dependencies]
agent-client-protocol = "0.3.1"
jsonrpc-core = "18.0"
tokio-tungstenite = "0.20"

[features]
claude-acp = []
```

### 2. Create Module Structure
- [ ] Create `src/acp_claude/` directory
- [ ] Create `src/acp_claude/mod.rs`
- [ ] Create `src/acp_claude/adapter.rs`
- [ ] Create `src/acp_claude/client.rs`
- [ ] Create `src/acp_claude/config.rs`

### 3. Basic Type Definitions
- [ ] Define `ClaudeACPConfig` struct
- [ ] Define `ClaudeACPError` error type
- [ ] Define `ACPResult<T>` type alias

### 4. Feature Flag Setup
- [ ] Conditionally compile module with `#[cfg(feature = "claude-acp")]`
- [ ] Add feature flag to `main.rs`

## Acceptance Criteria
- [ ] `cargo build --features claude-acp` succeeds
- [ ] Existing build `cargo build` also succeeds
- [ ] New modules import correctly

## Estimated Time
4-6 hours

## Labels
- `task`
- `day-1`
- `setup`
- `claude-acp`
