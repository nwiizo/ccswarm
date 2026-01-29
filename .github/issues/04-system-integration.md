# Issue #4: Day 6-7 - Integration with Existing System

## Overview
Integrate Claude Code ACP with ccswarm's existing task system.

## Task List

### 1. Integration with MasterClaude
- [ ] Modify `src/orchestrator/mod.rs`:
```rust
pub struct MasterClaude {
    #[cfg(feature = "claude-acp")]
    claude_acp: Option<Arc<Mutex<ClaudeCodeAdapter>>>,
}
```

### 2. Task Delegation Logic
- [ ] Modify `delegate_task()` method
- [ ] Implement Claude Code priority mode
- [ ] Add `delegate_to_claude_acp()` method

### 3. Configuration File Support
- [ ] Load `.ccswarm.yml`
```yaml
claude_acp:
  enabled: true
  url: "ws://localhost:9100"
  auto_connect: true
  prefer_claude: true
```

### 4. Environment Variable Support
- [ ] `CCSWARM_CLAUDE_ACP_URL`
- [ ] `CCSWARM_CLAUDE_ACP_ENABLED`
- [ ] `CCSWARM_CLAUDE_ACP_AUTO_CONNECT`

### 5. Extend Existing Commands
- [ ] Add `--via-acp` flag to `ccswarm task`
- [ ] Add ACP status to `ccswarm status`

## Acceptance Criteria
- [ ] Existing functionality not broken
- [ ] Tasks can execute via Claude Code
- [ ] Configuration file loads correctly
- [ ] Environment variables are reflected

## Estimated Time
8-10 hours

## Labels
- `task`
- `day-6-7`
- `integration`
- `claude-acp`
