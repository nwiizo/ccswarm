# CLAUDE.md

## Project Overview

ccswarm - AI Multi-Agent Orchestration System that coordinates specialized AI agents (Frontend, Backend, DevOps, QA) using a Master Claude coordinator. Built in Rust for performance and reliability with native ai-session management.

## Workspace Structure

This project uses a Cargo workspace with the following structure:
- `crates/ccswarm/` - Main ccswarm application and orchestration system
- `crates/ai-session/` - Native AI session management library

## Development Standards

### Code Quality Requirements
- Run `cargo fmt && cargo clippy -- -D warnings && cargo test` before all commits (from workspace root)
- Maintain test coverage >85% (check with `cargo tarpaulin`)
- Document all public APIs with rustdoc comments
- Cyclomatic complexity must be <10 per function

### Architecture Patterns
- **Session Management**: Always use ai-session adapter for agent terminals
- **Agent Boundaries**: Strictly enforce role isolation (Frontend/Backend/DevOps/QA)
- **Async First**: Use tokio async/await, never block the runtime
- **Error Handling**: Use Result<T, E> with custom error types, no .unwrap() in production

### Testing Strategy
- Unit tests colocated with implementation in `#[cfg(test)]` modules
- Integration tests in `crates/ccswarm/tests/` directory
- Use `#[tokio::test]` for async tests
- Mock external dependencies with `mockall` or similar
- Run workspace-wide tests with `cargo test --workspace`

## Frequently Used Commands

### Workspace Management
```bash
# Build entire workspace
cargo build --workspace

# Test entire workspace
cargo test --workspace

# Build specific crate
cargo build -p ccswarm
cargo build -p ai-session

# Run ccswarm from workspace root
cargo run -p ccswarm -- --help

# Format and check entire workspace
cargo fmt --all
cargo clippy --workspace -- -D warnings

# Generate documentation for all crates
cargo doc --workspace --no-deps --open
```

### Crate-Specific Development
```bash
# Work on ai-session crate
cd crates/ai-session
cargo test
cargo doc --open

# Work on ccswarm crate
cd crates/ccswarm
cargo test
cargo run -- --help

# Return to workspace root for workspace commands
cd ../..
cargo test --workspace
```

### Development Workflow
```bash
# Initial setup (from workspace root)
cargo run -p ccswarm -- init --name "MyProject" --agents frontend,backend

# Start system
cargo run -p ccswarm -- start
cargo run -p ccswarm -- tui  # Monitor in terminal UI

# Create and manage tasks
cargo run -p ccswarm -- task "Implement user authentication [high] [feature]"
cargo run -p ccswarm -- task list --status pending
cargo run -p ccswarm -- delegate task "Add login API" --agent backend

# Session management
cargo run -p ccswarm -- session list
cargo run -p ccswarm -- session stats
cargo run -p ccswarm -- session attach <session-id>
```

### Debugging Commands
```bash
# Logging levels
RUST_LOG=debug cargo run -p ccswarm -- start
RUST_LOG=ccswarm::session=trace cargo run -p ccswarm -- start
RUST_LOG=ai_session=debug cargo run -p ccswarm -- start

# Check agent status
cargo run -p ccswarm -- agent list
cargo run -p ccswarm -- logs --agent frontend --tail 50

# Quality review
cargo run -p ccswarm -- review status
cargo run -p ccswarm -- review history --failed
```

### Advanced Features
```bash
# Auto-create applications
cargo run -p ccswarm -- auto-create "Create a real-time chat app with React and WebSockets"

# Sangha collective intelligence
cargo run -p ccswarm -- sangha propose --type feature --title "Add GraphQL support"
cargo run -p ccswarm -- sangha vote <proposal-id> aye --reason "Improves API flexibility"

# Autonomous agent extension
cargo run -p ccswarm -- extend autonomous --continuous
```

## Project-Specific Guidelines

### Agent Role Enforcement
- Frontend agents: React, Vue, UI/UX, CSS only
- Backend agents: APIs, databases, server logic only
- DevOps agents: Docker, CI/CD, infrastructure only
- QA agents: Testing and quality assurance only

### Security Requirements
- Never hardcode API keys or secrets
- Validate all user inputs
- Respect protected file patterns (.env, *.key, .git/)
- Use environment variables for sensitive data

### Performance Optimizations
- Reuse sessions whenever possible for efficiency
- Run independent tasks concurrently
- Use session pooling for similar operations
- Enable context compression for long-running sessions

## Import Additional Documentation
@docs/ARCHITECTURE.md
@docs/APPLICATION_SPEC.md
@docs/commands/workspace-commands.md
@.claude/settings.json
@.claude/commands/project-rules.md

## Workspace File Structure
```
ccswarm/
├── Cargo.toml                   # Workspace configuration
├── CLAUDE.md                    # This file
├── docs/
│   ├── ARCHITECTURE.md          # System architecture
│   ├── APPLICATION_SPEC.md      # Application specifications
│   └── commands/
│       ├── README.md            # Commands documentation index
│       └── workspace-commands.md # Workspace development guide
├── crates/
│   ├── ccswarm/                 # Main application crate
│   │   ├── src/                 # Source code
│   │   ├── tests/               # Integration tests
│   │   └── Cargo.toml           # Crate configuration
│   └── ai-session/              # AI session library crate
│       ├── src/                 # Library source code
│       └── Cargo.toml           # Crate configuration
└── .claude/
    ├── settings.json            # Claude Code settings
    └── commands/
        └── project-rules.md     # Development rules
```