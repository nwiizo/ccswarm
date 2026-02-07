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

## Documentation

@docs/ARCHITECTURE.md
@docs/APPLICATION_SPEC.md
@.claude/settings.json
