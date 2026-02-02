# Issue #3: Day 4-5 - CLI Command Implementation

## Overview
Implement CLI commands for operating Claude Code ACP.

## Task List

### 1. Add CLI Command Structure
- [ ] Add the following to `src/cli/mod.rs`:
```rust
#[derive(Subcommand)]
pub enum ClaudeACPCommands {
    Start { url: Option<String> },
    Test,
    Send { task: String },
    Stop,
    Status,
    Diagnose,
}
```

### 2. Implement Command Handlers
- [ ] `claude-acp start` - Start ACP connection
- [ ] `claude-acp test` - Connection test
- [ ] `claude-acp send` - Send task
- [ ] `claude-acp stop` - End connection
- [ ] `claude-acp status` - Display current state
- [ ] `claude-acp diagnose` - Troubleshooting

### 3. Output Format
- [ ] Colorful output (with emoji)
- [ ] Progress bar display
- [ ] Error message formatting

### 4. Help Messages
- [ ] Detailed help for each command
- [ ] Add usage examples
- [ ] Troubleshooting guide

## Acceptance Criteria
- [ ] `ccswarm claude-acp --help` displays help
- [ ] Each command works correctly
- [ ] Appropriate messages displayed on errors

## Estimated Time
8-10 hours

## Labels
- `task`
- `day-4-5`
- `cli`
- `claude-acp`
