# CLAUDE.md

## Project Overview

ccswarm - AI Multi-Agent Orchestration System that coordinates specialized AI agents (Frontend, Backend, DevOps, QA) using Claude Code via Agent Client Protocol (ACP) as the **default integration method**. Built in Rust for performance and reliability with zero external dependencies.

## Claude Code Integration (Default)

ccswarm now uses **Claude Code via ACP** as the primary communication method:
- **Auto-Connect**: Automatically connects to Claude Code on startup (ws://localhost:9100)
- **WebSocket Protocol**: Real-time bidirectional communication
- **Session Management**: Persistent sessions with UUID tracking
- **Task Delegation**: Direct task routing to Claude Code

## Workspace Structure

This project uses a Cargo workspace with the following structure:
- `crates/ccswarm/` - Main ccswarm application with Claude ACP integration
- `sample/` - Sample scripts demonstrating Claude Code integration

## Development Standards

### Code Quality Requirements
- Run `cargo fmt && cargo clippy -- -D warnings && cargo test` before all commits (from workspace root)
- Maintain test coverage >85% (check with `cargo tarpaulin`)
- Document all public APIs with rustdoc comments
- Cyclomatic complexity must be <10 per function

### Architecture Patterns
- **Claude ACP Integration**: Primary communication via WebSocket protocol
- **Agent Boundaries**: Strictly enforce role isolation (Frontend/Backend/DevOps/QA)
- **Async First**: Use tokio async/await, never block the runtime
- **Error Handling**: Use Result<T, E> with custom error types, no .unwrap() in production

#### Command Registry Pattern
- **Purpose**: Eliminates massive match statements in CLI handling
- **Implementation**: Uses HashMap of command handlers with async closures
- **Benefits**: Reduces code duplication, improves maintainability
- **Location**: `crates/ccswarm/src/cli/command_registry.rs`
- **Usage**: Register commands once, dispatch dynamically

#### Error Template System
- **Purpose**: Standardizes error diagrams and visualizations
- **Implementation**: Template engine with reusable diagram patterns
- **Templates**: Box diagrams, flow diagrams, network diagrams
- **Location**: `crates/ccswarm/src/utils/error_template.rs`
- **Benefits**: Consistent error presentation, reduced duplication

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

### Claude ACP Commands (Default Integration)
```bash
# Test Claude Code connection
cargo run -p ccswarm -- claude-acp test

# Start ACP adapter
cargo run -p ccswarm -- claude-acp start

# Send task to Claude Code
cargo run -p ccswarm -- claude-acp send --task "Analyze code for improvements"

# Check connection status
cargo run -p ccswarm -- claude-acp status

# Run diagnostics
cargo run -p ccswarm -- claude-acp diagnose
```

### Development Workflow
```bash
# Initial setup (from workspace root)
cargo run -p ccswarm -- init --name "MyProject" --agents frontend,backend

# Start system (auto-connects to Claude Code)
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

### New User Experience
```bash
# Interactive setup wizard for first-time users
cargo run -p ccswarm -- setup

# Interactive tutorial to learn ccswarm
cargo run -p ccswarm -- tutorial
cargo run -p ccswarm -- tutorial --chapter 2  # Start from specific chapter

# Enhanced help system with examples
cargo run -p ccswarm -- help-topic "agent management"
cargo run -p ccswarm -- help-topic --search "error"
```

### System Health and Diagnostics
```bash
# System health checks
cargo run -p ccswarm -- health --check-agents --check-sessions
cargo run -p ccswarm -- health --diagnose --detailed
cargo run -p ccswarm -- health --resources --format json

# Doctor command for diagnosing and fixing issues
cargo run -p ccswarm -- doctor
cargo run -p ccswarm -- doctor --fix
cargo run -p ccswarm -- doctor --error "E001"
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
├── README.md                    # Main project documentation
├── docs/
│   ├── ARCHITECTURE.md          # System architecture
│   ├── APPLICATION_SPEC.md      # Application specifications
│   ├── CLAUDE_ACP.md           # Claude ACP integration guide
│   └── commands/
│       ├── README.md            # Commands documentation index
│       └── workspace-commands.md # Workspace development guide
├── crates/
│   └── ccswarm/                 # Main application crate
│       ├── src/                 # Source code
│       │   ├── acp_claude/      # Claude ACP integration module
│       │   │   ├── adapter.rs   # WebSocket adapter
│       │   │   ├── config.rs    # Configuration management
│       │   │   └── error.rs     # Error handling
│       │   ├── cli/             # CLI module with command registry
│       │   │   ├── command_registry.rs  # Command dispatch system
│       │   │   ├── command_handler.rs   # Command execution logic
│       │   │   └── commands/            # Individual command modules
│       │   └── utils/           # Utility modules
│       │       └── error_template.rs    # Error diagram templates
│       ├── tests/               # Integration tests
│       └── Cargo.toml           # Crate configuration
├── sample/                      # Sample scripts and demos
│   ├── claude_acp_demo.sh      # Claude ACP demonstration
│   ├── task_management_demo.sh # Task management demo
│   ├── multi_agent_demo.sh     # Multi-agent collaboration
│   ├── setup.sh                # Setup script
│   └── ccswarm.yaml            # Sample configuration
└── .claude/
    ├── settings.json            # Claude Code settings
    └── commands/
        └── project-rules.md     # Development rules
```