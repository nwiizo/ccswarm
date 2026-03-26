# CLAUDE.md

## Project Overview

ccswarm v0.6.0 - AI Agent Workflow DevOps toolchain complementing Claude Code Agent Teams.

Cargo workspace with 2 crates:
- **ccswarm** (`crates/ccswarm/`) - Agent workflow engine, CLI, bridge to Claude Code, event recording
- **ai-session** (`crates/ai-session/`) - Token-efficient terminal session management (standalone-capable)

## Quick Commands

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test  # Before commit
cargo run -p ccswarm -- --help                          # Run ccswarm
cargo run -p ai-session -- --help                       # Run ai-session CLI
```

## Workspace Architecture

```
ccswarm (workflow DevOps) ──depends on──> ai-session (terminal management)
                                              ──depends on──> portable-pty, tokio, zstd
```

### ccswarm crate — Workflow DevOps Layer

| Module | Purpose |
|--------|---------|
| `agent/` | ClaudeCodeAgent, AgentRole (Frontend/Backend/DevOps/QA/Master), Type-State TaskBuilder |
| `cli/` | ~23 commands via CommandRegistry pattern, interactive help, setup wizard |
| `workflow/` | Piece engine (YAML movements), Pipeline runner, Faceted prompting, DAG workflows |
| `coordination/` | AgentMailbox, MessageBus bridge, conversion layer to ai-session |
| `events/` | NDJSON EventRecorder to `.ccswarm/runs/{run-id}/events.ndjson`, duration tracking |
| `identity/` | AgentIdentity, role boundaries |
| `session/` | SessionManager, AISessionBridge (Claude Code CLI execution with --resume, --agent routing, retry with exponential backoff) |
| `hooks/` | HookRegistry, SecurityHook, LoggingHook |
| `config/` | CcswarmConfig loader |
| `git/` | Git worktree operations |
| `resource/` | ResourceMonitor, session resource integration |
| `utils/` | Error recovery, error templates, diagnostics |
| `workspace/` | Workspace management |

### ai-session crate — Session Management Layer

| Module | Purpose |
|--------|---------|
| `core/` | AISession, SessionManager, PTY/headless terminal, lifecycle |
| `context/` | TokenEfficientHistory with zstd compression, sliding window (20 recent) |
| `coordination/` | MessageBus (crossbeam channels), TaskDistributor, ResourceManager, RateLimit |
| `output/` | OutputParser (regex heuristics), SemanticCompressor, highlight extraction |
| `persistence/` | JSON + zstd session snapshots, restore by ID |
| `mcp/` | JSON-RPC server, tool registry, stdio transport |
| `integration/` | Tmux migration compat layer |

Binaries: `ai-session` (CLI), `ai-session-server` (MCP HTTP API on port 3000)

## Agent Teams

The 4 domain agents are designed for parallel execution via Claude Code Agent Teams:

```bash
claude --agent-team                          # Interactive team setup
claude --team "frontend-specialist" "backend-specialist" "qa-specialist"
```

Each domain agent has `isolation: worktree` — they work in independent git worktrees and communicate via direct messaging (`@agent-name`).

| Agent | Isolation | Model | Coordinates With |
|-------|-----------|-------|------------------|
| frontend-specialist | worktree | sonnet | backend (API contracts), qa (test readiness) |
| backend-specialist | worktree | sonnet | frontend (contracts), devops (CI/CD) |
| devops-specialist | worktree | sonnet | backend/frontend (build reqs), qa (deploy verify) |
| qa-specialist | worktree | sonnet | all (quality gates, test coverage) |

Review agents run as subagents (not teams):

| Agent | Model | Invoked By |
|-------|-------|------------|
| rust-fix-agent | opus | Proactively on build/clippy errors |
| code-refactor-agent | opus | Proactively on suspected duplication |
| architecture-reviewer | sonnet | `/review-architecture` skill |
| all-reviewer | sonnet | `/review-all` skill |

## Rules

- [development-standards](.claude/rules/development-standards.md) - Code quality, testing, language convention
- [architecture-patterns](.claude/rules/architecture-patterns.md) - Rust patterns, Claude ACP integration
- [security-guidelines](.claude/rules/security-guidelines.md) - Security, agent roles, environment
- [performance](.claude/rules/performance.md) - Optimization guidelines

## Skills

| Skill | Description |
|-------|-------------|
| `/review-all` | Full review (design compliance, quality, architecture) |
| `/review-architecture` | Architecture pattern compliance check |
| `/review-duplicates` | Duplicate code detection via similarity-rs |
| `/check-impl` | Build, lint, test verification |
| `/check-production-ready` | Production readiness audit |
| `/mutation-test` | Mutation testing execution |
| `/git-worktree` | Parallel development with git worktrees |
| `/rust-agent-specialist` | Rust-native pattern guidance |
| `/deploy-workflow` | Release deployment process |
| `/benchmark-runner` | Performance benchmark execution |
| `/hitl-approval` | Human-in-the-loop approval workflows |

## Hooks

| Event | Script | Purpose |
|-------|--------|---------|
| PreToolUse (Edit/Write) | `validate-agent-scope.sh` | Enforce agent role boundaries on file edits |
| PostToolUse (Edit/Write) | `format-code.sh` | Auto-format Rust/Go/TS/Python after edits |
| SubagentStop | `audit-trail.sh` | Log agent team lifecycle to NDJSON audit trail |
| Stop | `audit-trail.sh` | Log session completion |

## Development Learnings

### Pipeline Usage (from real-world testing)
- **Task description size**: Large task descriptions (>500 words) cause implement movement to exceed 600s timeout. Split complex tasks into multiple pipeline runs or use the `develop-and-verify` piece with smaller steps.
- **Complete movement**: Use empty instruction (`""`) for terminal movements to skip Claude CLI call. Local summary is instant vs ~12s for Claude.
- **Template variables**: Use `{task}`, `{plan_output}`, `{verify_output}` in movement instructions for inter-step context. Variables auto-expand from `state.variables`.
- **Custom pieces**: Place YAML files in `.ccswarm/pieces/` for auto-loading. Builtin pieces: `default`, `research`, `review-fix`.
- **data-testid**: Include data-testid requirements in the task description for Playwright-testable output. Claude generates them if explicitly asked.
- **Timeout tuning**: Default 600s is often not enough for complex implement movements. Consider `--timeout 900` or splitting tasks.
- **AI-Session context**: The `--resume` flag is sent to Claude Code CLI but may not work as expected in all versions. Context passing between movements uses the prompt injection approach (state.variables).

### Error Handling
- `CCSwarmError` uses struct variants: `Agent { agent_id, message, source }`, `Session { session_id, message, source }`, `Configuration { field, message }`
- Helper constructors: `CCSwarmError::config()`, `agent()`, `session()`, `task()`, `git()`, `user_error()`

### Module Patterns
- Splitting `foo.rs` → `foo/mod.rs`: keep all `pub use` re-exports in `mod.rs`
- The `format-code.sh` hook runs formatters after edits; re-read files if content changes

### CI/CD
- Publish order: `ai-session` first, then `ccswarm` (dependency order)
- Release workflow needs `permissions: contents: write`
- `CARGO_REGISTRY_TOKEN` required in GitHub Secrets

## Documentation

@docs/ARCHITECTURE.md
@docs/APPLICATION_SPEC.md
