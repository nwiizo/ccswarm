# ccswarm: AI Multi-Agent Orchestration System

> **Version 0.4.3** - Rust Edition 2024

[![CI](https://github.com/nwiizo/ccswarm/workflows/CI/badge.svg)](https://github.com/nwiizo/ccswarm/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-edition_2024-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**ccswarm** is a workflow automation framework for coordinating specialized AI agents using Claude Code CLI. It provides task delegation infrastructure, template-based scaffolding, and Git worktree isolation for parallel development.

## Implementation Status

| Category | Status | Description |
|----------|--------|-------------|
| CLI Infrastructure | Working | All commands parse and route correctly |
| Session Management | Working | Native PTY-based sessions via ai-session |
| TUI Monitoring | Working | Real-time terminal UI with ratatui |
| Configuration | Working | Project configs, agent settings |
| Git Worktrees | Working | Create, list, remove, prune |
| Template System | Working | Predefined templates for app scaffolding |
| Task Queue | Working | Task queuing and tracking |
| Parallel Executor | Partial | Structure exists, not integrated with orchestrator |
| `start` Command | Partial | Initializes but coordination loop not implemented |
| Auto-Create | Partial | Templates work, full AI generation incomplete |
| Sangha (Voting) | Planned | Data structures only |
| Extensions | Planned | Stub implementation |

> **Note**: See [docs/analysis/](docs/analysis/) for detailed capability gap analysis.

## What Works Today

- **CLI Commands**: Initialize projects, manage sessions, run TUI, check health
- **Session Management**: Create, list, attach, detach sessions (native PTY, no tmux)
- **Template Scaffolding**: Generate project structure from templates
- **Git Worktrees**: Isolated workspaces per agent role
- **TUI Dashboard**: Monitor tasks and sessions in terminal
- **Resource Monitoring**: Track agent usage statistics
- **Configuration**: Load/save project and agent configs

## What's In Progress

- **Orchestrator Coordination**: `start` command initializes but doesn't run coordination loop
- **Parallel Agent Execution**: ParallelExecutor implemented but not wired to orchestrator
- **Auto-Create**: Template generation works; full AI-driven creation incomplete
- **ai-session Integration**: Available for session management; multi-agent coordination not leveraged

## Quick Start

### Prerequisites

- Rust (Edition 2024 compatible toolchain)
- Git 2.20+
- Claude Code CLI (`claude` command) - optional for template-only usage

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

# List available templates
./target/release/ccswarm template list

# Check system health
./target/release/ccswarm doctor

# Start TUI monitoring
./target/release/ccswarm tui

# Session management
./target/release/ccswarm session list
./target/release/ccswarm session create --name "dev-session"

# Git worktree management
./target/release/ccswarm worktree list
./target/release/ccswarm worktree create feature/auth
```

## CLI Commands

### Working Commands

| Command | Description |
|---------|-------------|
| `init` | Initialize a new ccswarm project |
| `tui` | Start Terminal User Interface |
| `session` | Session management (list, create, attach, detach) |
| `worktree` | Git worktree management |
| `template` | Template listing and management |
| `config` | Configuration management |
| `doctor` | System health checks and diagnostics |
| `health` | Resource and connectivity checks |
| `tutorial` | Interactive learning tutorial |
| `setup` | Interactive setup wizard |
| `status` | Show current status |
| `agents` | List agent configurations |

### Partial/Stub Commands

| Command | Status |
|---------|--------|
| `start` | Initializes but exits immediately |
| `stop` | Stub - prints message only |
| `auto-create` | Template scaffolding works; AI generation incomplete |
| `task` | Queue structures work; execution not connected |
| `delegate` | Routing exists; actual delegation incomplete |
| `review` | Structure exists; quality evaluation incomplete |
| `sangha` | Data structures only |
| `extend` | Stub implementation |
| `search` | Types defined; no implementation |
| `subagent` | Management exists; parallel execution not wired |

## Project Structure

```
ccswarm/
├── Cargo.toml              # Workspace configuration
├── crates/
│   ├── ccswarm/            # Main CLI and orchestration
│   │   ├── src/
│   │   │   ├── cli/        # CLI commands (working)
│   │   │   ├── orchestrator/ # Task delegation (partial)
│   │   │   ├── agent/      # Agent management (working)
│   │   │   ├── session/    # Session management (working)
│   │   │   ├── template/   # Template system (working)
│   │   │   ├── subagent/   # Parallel execution (partial)
│   │   │   ├── tui/        # Terminal UI (working)
│   │   │   └── resource/   # Resource monitoring (working)
│   │   └── tests/
│   └── ai-session/         # Session management library
│       ├── src/
│       │   ├── core/       # PTY, lifecycle, process
│       │   ├── context/    # Context compression
│       │   ├── coordination/ # Message bus (available)
│       │   └── mcp/        # MCP server
│       └── tests/
├── docs/
│   ├── ARCHITECTURE.md
│   ├── APPLICATION_SPEC.md
│   └── analysis/           # Implementation gap analysis
└── .claude/                # Claude Code configuration
```

## Development

```bash
# Run all tests
cargo test --workspace

# Run only library tests
cargo test --lib -p ccswarm

# Run ai-session tests
cargo test --lib -p ai-session

# Check code quality
cargo fmt && cargo clippy -- -D warnings

# Build release
cargo build --release --workspace
```

## ai-session Crate

The `ai-session` crate provides native PTY-based session management:

```rust
use ai_session::{SessionManager, SessionConfig};

// Create session manager
let manager = SessionManager::new();

// Create a session
let mut config = SessionConfig::default();
config.name = "dev-session".to_string();
let session = manager.create_session(config).await?;

// Session lifecycle
manager.pause_session(&session.id).await?;
manager.resume_session(&session.id).await?;
```

Features:
- Cross-platform PTY implementation (Linux, macOS)
- Session lifecycle management
- Context compression (available)
- MCP server (available)
- Message bus for coordination (available, not utilized by orchestrator)

## Known Limitations

- **Orchestrator Not Running**: `ccswarm start` initializes but doesn't run coordination loop
- **Claude Code Required**: Auto-create features require Claude Code CLI
- **macOS/Linux Only**: Windows not supported due to Unix-specific PTY dependencies
- **No True Parallel Execution**: ParallelExecutor exists but not connected to orchestrator

## Roadmap

See [Issue #67](https://github.com/nwiizo/ccswarm/issues/67) for redesign discussion.

**Phase 1 (Current Focus):**
- Fix `start` command to run coordination loop
- Wire ParallelExecutor to orchestrator
- Implement actual agent process spawning

**Phase 2:**
- Leverage ai-session's MessageBus for agent coordination
- Implement context compression for token savings
- Add ACP (Agent Client Protocol) support for multi-vendor agents

## Documentation

- [Architecture](docs/ARCHITECTURE.md) - System design
- [Application Spec](docs/APPLICATION_SPEC.md) - Feature details
- [Capability Gap Analysis](docs/analysis/00-capability-gap-analysis.md) - Implementation status
- [Architecture Comparison](docs/analysis/01-multi-agent-architecture-comparison.md) - ACP vs SDK vs PTY
- [Development Standards](.claude/rules/development-standards.md) - Coding guidelines

## License

MIT License - see [LICENSE](LICENSE) for details.
