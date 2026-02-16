# CLAUDE.md

## Project Overview

ccswarm v0.4.5 - AI Multi-Agent Orchestration System with **ai-session** integration.

> **Implementation Status**: ~75% complete. See `.claude/reference/version-notes.md` for details.

## Quick Commands

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test  # Before commit
cargo run -p ccswarm -- --help                          # Run ccswarm
```

## Rules

- [development-standards](.claude/rules/development-standards.md) - Code quality, testing, language convention
- [architecture-patterns](.claude/rules/architecture-patterns.md) - Rust patterns, Claude ACP integration
- [security-guidelines](.claude/rules/security-guidelines.md) - Security, agent roles, environment
- [performance](.claude/rules/performance.md) - Optimization guidelines

## Hooks

Automated validation via Claude Code hooks:
- `validate-agent-scope.sh` - Pre-edit agent scope validation
- `format-code.sh` - Post-edit auto-formatting
- `audit-trail.sh` - Session activity logging

## Agents (Subagents)

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
- [version-notes](.claude/reference/version-notes.md) - Implementation status, v0.3.8/v0.4.0 features

## Skills

- [git-worktree](.claude/skills/git-worktree/SKILL.md) - Parallel development workflow
- [rust-agent-specialist](.claude/skills/rust-agent-specialist/SKILL.md) - Rust-native patterns
- [deploy-workflow](.claude/skills/deploy-workflow/SKILL.md) - Release deployment
- [benchmark-runner](.claude/skills/benchmark-runner/SKILL.md) - Performance benchmarks
- [hitl-approval](.claude/skills/hitl-approval/SKILL.md) - Human-in-the-loop approval

## Development Learnings

### Error Handling
- `CCSwarmError` variants are **struct variants** (not tuple): `CCSwarmError::Agent { agent_id, message, source }`
- Other variants: `Session { session_id, message, source }`, `Configuration { field, message }`, `Provider { provider, message, source }`
- Never use `.unwrap()` in production; use `Result<T, E>` with `thiserror`

### Module Patterns
- When splitting `foo.rs` into `foo/mod.rs`: keep all `pub use` re-exports in `mod.rs` for API compatibility
- New submodules in `orchestrator/` and `coordination/` must add re-exports in their `mod.rs`
- The `format-code.sh` hook runs `cargo fmt` after edits; re-read files if content changes unexpectedly

### Clippy Fixes
- `.or_insert_with(Vec::new)` -> `.or_default()`
- Boolean simplification: `(A && B) || (C && B)` -> `B && (A || C)`

### CI/CD
- Release workflow needs `permissions: contents: write` for GitHub release creation
- Use `softprops/action-gh-release@v2` (not deprecated `actions/create-release@v1`)
- Publish order: `ai-session` first, then `ccswarm` (dependency order)
- `CARGO_REGISTRY_TOKEN` must be set in GitHub Secrets for CI publish

### Crate Publishing
- Workspace crates with path dependencies need version field for crates.io
- `cargo publish -p <crate>` publishes from workspace root
- Wait for dependency crate to be indexed before publishing dependent crate

## Documentation

@docs/ARCHITECTURE.md
@docs/APPLICATION_SPEC.md
@.claude/settings.json
