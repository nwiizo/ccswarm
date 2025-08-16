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

## Compilation Error and Warning Resolution

### Quick Fix Commands
```bash
# Automatically fix most warnings (saves ~70% of manual work)
cargo clippy --fix --allow-dirty --allow-staged

# Check for compilation errors
cargo build --lib 2>&1 | grep "error\[E"

# Count warnings
cargo build --lib 2>&1 | grep "warning:" | wc -l

# Show specific error types
cargo build --lib 2>&1 | grep "E0609"  # field access errors
```

### Common Error Patterns and Fixes

#### 1. Struct Field Simplification
```rust
// After refactoring reduces struct complexity
// Old: agent.identity.agent_id
// New: agent.agent.id
```

#### 2. Constructor Signature Changes
```rust
// Maintain backward compatibility
pub fn new_with_id(id: String, description: String, priority: Priority, task_type: TaskType) -> Self
```

#### 3. Unused Items Resolution
```rust
// Variables: prefix with _
let _config = Config::new();

// Fields: prefix and update all usages
struct Handler {
    _analyzer: Arc<SemanticAnalyzer>,  // was: analyzer
}

// Imports: remove or prefix
use crate::module::Type as _Type;  // if might be needed later
```

### Systematic Resolution Workflow
1. **Run clippy auto-fix first** - Fixes Default impls, redundant code, etc.
2. **Fix compilation errors** - Focus on one error type at a time
3. **Fix warnings by category** - Not by file, for consistency
4. **Verify after each step** - `cargo check --lib`
5. **Test coverage** - `cargo test --lib` to ensure fixes don't break tests

### Field Renaming Checklist
When adding `_` prefix to unused fields:
- [ ] Update field declaration
- [ ] Update all struct initializations  
- [ ] Update all field accesses
- [ ] Update pattern matching

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
@.claude/commands/rust-compilation-fixes.md
@.claude/commands/efficient-warning-resolution.md

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
│   │   │   ├── cli/             # CLI module with command registry
│   │   │   │   ├── command_registry.rs  # Command dispatch system
│   │   │   │   ├── command_handler.rs   # Command execution logic
│   │   │   │   ├── commands/            # Individual command modules
│   │   │   │   └── ...                  # Other CLI utilities
│   │   │   └── utils/           # Utility modules
│   │   │       ├── error_template.rs    # Error diagram templates
│   │   │       └── ...                  # Other utilities
│   │   ├── tests/               # Integration tests
│   │   └── Cargo.toml           # Crate configuration
│   └── ai-session/              # AI session library crate
│       ├── src/                 # Library source code
│       └── Cargo.toml           # Crate configuration
└── .claude/
    ├── settings.json                        # Claude Code settings
    └── commands/
        ├── project-rules.md                 # Development rules
        ├── rust-compilation-fixes.md        # Error resolution patterns
        └── efficient-warning-resolution.md  # Warning fix strategies
```