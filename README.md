# ccswarm: AI Multi-Agent Orchestration System

> **Version 0.4.3** - Rust Edition 2024

[![CI](https://github.com/nwiizo/ccswarm/workflows/CI/badge.svg)](https://github.com/nwiizo/ccswarm/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-edition_2024-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**ccswarm** is a workflow automation system that coordinates specialized AI agents using Claude Code CLI. It provides task delegation, template-based application generation, and Git worktree isolation for parallel development.

## What ccswarm Does

- **Task Delegation**: Analyzes tasks and routes to specialized agents (Frontend, Backend, DevOps, QA)
- **Auto-Create**: Generates complete applications from natural language descriptions
- **Git Worktree Isolation**: Each agent works in isolated Git worktrees
- **Template System**: Predefined templates for common development tasks
- **TUI Monitoring**: Real-time terminal UI for task tracking
- **Native Session Management**: Native PTY-based session management (no tmux required)
- **Resource Monitoring**: Track agent resource usage

## Quick Start

### Prerequisites

- Rust (Edition 2024 compatible toolchain)
- Git 2.20+
- Claude Code CLI (`claude` command available)

### Installation

```bash
git clone https://github.com/nwiizo/ccswarm.git
cd ccswarm
cargo build --release
```

### Basic Usage

```bash
# Initialize a project
./target/release/ccswarm init --name "MyProject"

# Create a TODO app
./target/release/ccswarm auto-create "Create a simple TODO app" --output /tmp/todo-app

# List templates
./target/release/ccswarm template list

# Check system health
./target/release/ccswarm doctor

# Start TUI monitoring
./target/release/ccswarm tui
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `init` | Initialize a new ccswarm project |
| `start` | Start the orchestrator |
| `stop` | Stop the running orchestrator |
| `status` | Show status of orchestrator and agents |
| `task` | Task management |
| `agents` | List agents and configurations |
| `tui` | Start Terminal User Interface |
| `auto-create` | Generate application with AI agents |
| `template` | Template management |
| `session` | Session management |
| `worktree` | Git worktree management |
| `resource` | Resource monitoring |
| `doctor` | System health checks |
| `review` | Run quality review |
| `delegate` | Master delegation commands |
| `sangha` | Collective decision making |
| `extend` | Extension management |
| `search` | Search external resources |
| `subagent` | Manage Claude Code subagents |
| `tutorial` | Interactive learning tutorial |

## Project Structure

```
ccswarm/
├── Cargo.toml              # Workspace configuration (edition = "2024")
├── crates/
│   ├── ccswarm/            # Main CLI and orchestration
│   │   ├── src/
│   │   │   ├── cli/        # CLI commands
│   │   │   ├── orchestrator/ # Task delegation
│   │   │   ├── agent/      # Agent management
│   │   │   ├── session/    # Session management
│   │   │   ├── template/   # Template system
│   │   │   ├── providers/  # Claude Code integration
│   │   │   ├── subagent/   # Parallel execution
│   │   │   ├── tui/        # Terminal UI
│   │   │   ├── workflow/   # Workflow engine
│   │   │   └── resource/   # Resource monitoring
│   │   └── tests/          # Integration & mockall tests
│   └── ai-session/         # Session management library
│       ├── src/
│       │   ├── core/       # Core session management
│       │   ├── context/    # Context compression
│       │   ├── coordination/ # Multi-agent coordination
│       │   └── security/   # Capability-based security
│       └── tests/
└── docs/                   # Documentation
```

## Development

```bash
# Run all tests (242 lib tests + 37 mockall tests)
cargo test --workspace

# Run only library tests
cargo test --lib -p ccswarm

# Run mockall tests
cargo test --test mockall_tests -p ccswarm
cargo test --test mockall_tests -p ai-session

# Check code quality
cargo fmt && cargo clippy -- -D warnings

# Build release
cargo build --release --workspace
```

### Testing with Mockall

The project uses mockall for mock-based testing. See `tests/mockall_tests.rs` for examples:

```rust
use mockall::mock;
use mockall::predicate::*;

mock! {
    pub MyService {
        fn execute(&self, input: &str) -> Result<String>;
    }
}

#[test]
fn test_with_mock() {
    let mut mock = MockMyService::new();
    mock.expect_execute()
        .times(1)
        .withf(|input| input.contains("test"))
        .returning(|_| Ok("result".into()));

    assert!(mock.execute("test input").is_ok());
}
```

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - System design
- [Application Spec](docs/APPLICATION_SPEC.md) - Feature details
- [Development Standards](.claude/rules/development-standards.md) - Coding guidelines

## v0.4.3 Changes

- **Native Session Management**: Removed tmux dependency - uses native PTY via ai-session
- **Session Search API**: Added `find_session_by_name()` and `list_sessions_by_prefix()` methods
- **Detailed Session Info**: New `list_sessions_detailed()` for comprehensive session information

## Limitations

- **Claude Code Required**: Auto-create requires Claude Code CLI (`claude` command)
- **macOS/Linux Only**: Windows is not supported due to Unix-specific dependencies

## License

MIT License - see [LICENSE](LICENSE) for details.
