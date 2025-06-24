# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ccswarm is an AI Multi-Agent Orchestration System. For detailed specifications, see:
- Application specifications: `docs/APPLICATION_SPEC.md`
- Architecture details: `docs/ARCHITECTURE.md`
- Command documentation: `.claude/commands/`

## Development Guidelines

### Code Quality Standards
```bash
# Always run before commits
cargo fmt && cargo clippy -- -D warnings && cargo test
```

### Critical Implementation Rules

1. **Agent Boundaries are Sacred**
   - Frontend agents ONLY handle UI/React/client-side code
   - Backend agents ONLY handle APIs/server/database code
   - DevOps agents ONLY handle infrastructure/Docker/CI/CD
   - QA agents ONLY handle testing and quality assurance
   - NEVER allow agents to work outside their role

2. **Session Management is Core**
   - Always use ai-session for agent terminals (93% token savings)
   - Native PTY implementation - no tmux dependency
   - Sessions persist across tasks and crashes
   - MCP protocol compliance required

3. **Quality Standards**
   - Test coverage must be >85%
   - Complexity score must be <10
   - All public APIs must have documentation
   - Security vulnerabilities must be fixed immediately

4. **Extension Philosophy**
   - Agents should self-reflect and propose improvements
   - All extensions go through Sangha consensus
   - Risk assessment is mandatory
   - Knowledge base grows over time

## Project-Specific Patterns

### Error Handling
```rust
// Always use custom error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Never use .unwrap() in production code
```

### Async Patterns
```rust
// Always use tokio for async runtime
#[tokio::main]
async fn main() -> Result<()> {
    // ...
}

// Prefer channels over shared state
let (tx, rx) = async_channel::unbounded();
```

### Testing Patterns
```rust
// Test modules next to implementation
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_feature() {
        // Test implementation
    }
}
```

## Common Pitfalls to Avoid

1. **Cross-Contamination**: Never let agents access other agents' worktrees
2. **Token Waste**: Always reuse sessions, never create new ones unnecessarily
3. **Blocking Operations**: Use async/await, never block the runtime
4. **Security Risks**: Never log secrets, always validate inputs
5. **Memory Leaks**: Clean up sessions and channels properly

## Debugging Tips

### Session Issues
```bash
# Check active sessions
cargo run -- session list

# View session efficiency
cargo run -- session stats --show-savings

# Debug session issues
RUST_LOG=ai_session=debug cargo run
```

### Agent Problems
```bash
# Check agent status
cargo run -- agent list

# View agent logs
cargo run -- logs --agent frontend --tail 50

# Debug identity violations
RUST_LOG=ccswarm::identity=trace cargo run
```

### Task Failures
```bash
# View failed tasks
cargo run -- task list --status failed

# Check quality review failures
cargo run -- review history --failed

# Debug task delegation
RUST_LOG=ccswarm::orchestrator=debug cargo run
```

## Performance Optimization

1. **Session Pooling**: Reuse sessions across similar tasks
2. **Batch Operations**: Group related tasks for efficiency
3. **Context Compression**: Enable AI features for 93% token savings
4. **Lazy Loading**: Don't load agent contexts until needed
5. **Concurrent Execution**: Run independent tasks in parallel

## Security Considerations

1. **API Keys**: Never commit keys, use environment variables
2. **File Access**: Respect .gitignore and protected patterns
3. **Command Injection**: Always sanitize user inputs
4. **Session Isolation**: Each agent runs in isolated environment
5. **Audit Trail**: Log all agent actions for accountability

## Quick Reference

### Essential Commands
- `cargo run -- init`: Initialize new project
- `cargo run -- start`: Start orchestrator
- `cargo run -- task "description"`: Create task
- `cargo run -- tui`: Monitor system
- `cargo run -- stop`: Graceful shutdown

### Important Files
- `ccswarm.json`: Project configuration
- `.ccswarm/`: Runtime data and logs
- `agents/*/CLAUDE.md`: Agent-specific instructions
- `.claude/commands/`: Command documentation

### Environment Variables
- `ANTHROPIC_API_KEY`: Required for Claude
- `RUST_LOG`: Debug logging control
- `CCSWARM_HOME`: Config directory

Remember: This is a complex distributed system. When in doubt, check the documentation in `docs/` and `.claude/commands/`.