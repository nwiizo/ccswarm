# CLAUDE.md

## Project Overview

ccswarm - High-performance AI Multi-Agent Orchestration System built with **Rust-native patterns**. No layered architecture - uses direct, efficient patterns for maximum performance and compile-time safety.

## Claude Code Integration

- **Auto-Connect**: WebSocket to `ws://localhost:9100` via ACP
- **Channel-Based Communication**: No shared state between agents
- **Actor Pattern**: Each agent as an independent actor

## Quick Commands

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test  # Before commit
cargo run -p ccswarm -- --help                          # Run ccswarm
```

## Rules

- [architecture-patterns](.claude/rules/architecture-patterns.md) - Rust patterns (Type-State, Channels, Actor)
- [development-standards](.claude/rules/development-standards.md) - Code quality, testing
- [security-guidelines](.claude/rules/security-guidelines.md) - Security, agent roles
- [performance](.claude/rules/performance.md) - Optimization guidelines

## Hooks

Automated validation and formatting via Claude Code hooks:
- `validate-agent-scope.sh` - Pre-edit agent scope validation
- `format-code.sh` - Post-edit auto-formatting
- `audit-trail.sh` - Session activity logging

## Agents (Subagents)

Specialized agents for Task tool delegation:
- [frontend-specialist](.claude/agents/frontend-specialist.md) - React, Vue, UI/UX
- [backend-specialist](.claude/agents/backend-specialist.md) - APIs, databases
- [devops-specialist](.claude/agents/devops-specialist.md) - Docker, CI/CD
- [qa-specialist](.claude/agents/qa-specialist.md) - Testing, quality
- [rust-fix-agent](.claude/agents/rust-fix-agent.md) - Rust build/clippy fixes
- [code-refactor-agent](.claude/agents/code-refactor-agent.md) - Code refactoring
- [architecture-reviewer](.claude/agents/architecture-reviewer.md) - Architecture review

## Reference (Load On-Demand)

- [commands](.claude/reference/commands.md) - All CLI commands
- [file-structure](.claude/reference/file-structure.md) - Project structure
- [version-notes](.claude/reference/version-notes.md) - v0.3.8 module details

## Skills

- [git-worktree](.claude/skills/git-worktree/SKILL.md) - Parallel development workflow
- [rust-agent-specialist](.claude/skills/rust-agent-specialist/SKILL.md) - Rust-native patterns
- [deploy-workflow](.claude/skills/deploy-workflow/SKILL.md) - Release deployment
- [benchmark-runner](.claude/skills/benchmark-runner/SKILL.md) - Performance benchmarks
- [hitl-approval](.claude/skills/hitl-approval/SKILL.md) - Human-in-the-loop approval

## Detailed Documentation

@docs/ARCHITECTURE.md
@docs/APPLICATION_SPEC.md
@.claude/settings.json

## Implementation Patterns (Learnings)

### SensitiveString Pattern
API keys and secrets should use `SensitiveString` wrapper type:
```rust
use ccswarm::providers::SensitiveString;

let api_key = SensitiveString::new("sk-secret");
println!("{:?}", api_key);  // Output: SensitiveString(****)
let actual = api_key.expose();  // Get actual value when needed
```
Benefits:
- Debug/Display masks values (prevents accidental logging)
- Supports Clone/Serialize/Deserialize
- Uses `secrecy` crate for memory safety

### Error Retry Hints Pattern
Errors should provide retry guidance:
```rust
impl CCSwarmError {
    fn should_retry(&self) -> bool { /* ... */ }
    fn suggested_retry_delay(&self) -> Duration { /* ... */ }
    fn max_retries(&self) -> u32 { /* ... */ }
}
```

### CLI Testing Patterns
- **E2E tests**: Execute actual binary (`tests/e2e_cli_test.rs`)
- **Unit tests**: Test argument parsing with `Cli::try_parse_from()` (`tests/cli_unit_tests.rs`)
- Use `{ .. }` pattern for enum variants with fields: `Commands::Start { .. } => {}`

### Type-State Pattern Notes
- `.expect()` is acceptable in type-state builders for invariants enforced by the type system
- Document why `.expect()` is safe in comments
- Type-state guarantees make these "impossible" to fail at runtime
