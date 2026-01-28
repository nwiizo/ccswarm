# ccswarm: AI Multi-Agent Orchestration System

> 🚀 **Version 0.4.0** - Rust-Native Multi-Agent Workflow Automation

[![CI](https://github.com/nwiizo/ccswarm/workflows/CI/badge.svg)](https://github.com/nwiizo/ccswarm/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**ccswarm** is a workflow automation system that coordinates specialized AI agents using Claude Code CLI. It provides task delegation, template-based application generation, and Git worktree isolation for parallel development.

## 🎯 What ccswarm Does

- **Task Delegation**: Analyzes tasks and routes to specialized agents (Frontend, Backend, DevOps, QA)
- **Auto-Create**: Generates complete applications from natural language descriptions
- **Git Worktree Isolation**: Each agent works in isolated Git worktrees
- **Template System**: Predefined templates for common development tasks
- **TUI Monitoring**: Real-time terminal UI for task tracking

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+
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
```

## ✅ Working Features

| Feature | Status | Description |
|---------|--------|-------------|
| CLI Commands | ✅ | `init`, `auto-create`, `template`, `doctor`, `task`, `tui` |
| Auto-Create | ✅ | Template-based application generation |
| Git Worktree | ✅ | Agent isolation via worktrees |
| Task Delegation | ✅ | Content-based routing to agents |
| Template System | ✅ | Predefined task templates |
| TUI | ✅ | Terminal UI for monitoring |

## 📁 Project Structure

```
ccswarm/
├── crates/ccswarm/       # Main CLI and orchestration
│   ├── src/
│   │   ├── cli/          # CLI commands
│   │   ├── orchestrator/ # Task delegation
│   │   ├── agent/        # Agent management
│   │   ├── template/     # Template system
│   │   └── providers/    # Claude Code integration
│   └── tests/
└── docs/                 # Documentation
```

## 🛠️ Development

```bash
# Run tests
cargo test --workspace

# Check code quality
cargo fmt && cargo clippy -- -D warnings

# Build release
cargo build --release --workspace
```

## 📚 Documentation

- [Architecture](docs/ARCHITECTURE.md) - System design
- [Application Spec](docs/APPLICATION_SPEC.md) - Feature details

## ⚠️ Limitations

- **Simulation Mode**: By default, auto-create uses template-based generation, not real AI
- **Claude Code Required**: Real AI execution requires Claude Code CLI to be installed
- **macOS/Linux Only**: Windows is not supported

## 📜 License

MIT License - see [LICENSE](LICENSE) for details.
