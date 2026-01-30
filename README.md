# ccswarm: AI Multi-Agent Orchestration System

<<<<<<< HEAD
> **Version 0.4.3** - Rust Edition 2024
=======
> **Version 0.3.8** - Rust-Native Multi-Agent Orchestration with Advanced Features
>>>>>>> e29fbdc (docs: restructure README with hub-and-spoke model)

[![CI](https://github.com/nwiizo/ccswarm/workflows/CI/badge.svg)](https://github.com/nwiizo/ccswarm/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-edition_2024-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**ccswarm** is a workflow automation framework for coordinating specialized AI agents using Claude Code CLI. It provides task delegation infrastructure, template-based scaffolding, and Git worktree isolation for parallel development.

<<<<<<< HEAD
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
=======
> **Default Integration**: ccswarm uses **Claude Code via ACP (Agent Client Protocol)** as the default communication method. Start the [ACP bridge](docs/CLAUDE_ACP.md) and ccswarm automatically connects!

## Documentation

| Document | Description |
|----------|-------------|
| **[Getting Started](docs/GETTING_STARTED.md)** | Installation, tutorial, first project |
| **[Architecture](docs/ARCHITECTURE.md)** | System design, components, data flow |
| **[Configuration](docs/CONFIGURATION.md)** | All configuration options with examples |
| **[Commands Reference](docs/COMMANDS.md)** | Complete CLI command reference |
| **[Claude ACP Guide](docs/CLAUDE_ACP.md)** | Claude Code integration setup |
| **[Troubleshooting](docs/TROUBLESHOOTING.md)** | Common issues and solutions |
| **[Application Spec](docs/APPLICATION_SPEC.md)** | Detailed feature specifications |
| **[Contributing](CONTRIBUTING.md)** | How to contribute |

## Key Features

> **Status**: ✅ implemented | ⚡ file-export | 🔜 planned

### Claude Code Integration via ACP (Default)
- **Native Claude Code Support**: Direct integration with Claude Code through Agent Client Protocol (ACP)
- **WebSocket Communication**: Real-time bidirectional communication with Claude Code
- **Auto-Connect**: Automatically connects to Claude Code on startup (default: ws://localhost:9100)
- **Task Delegation**: Send tasks directly to Claude Code for execution
- **Session Management**: Persistent session IDs for continuous interaction
- **Diagnostics**: Built-in connection testing and troubleshooting tools

### Developer Experience First
- **Interactive Setup Wizard**: Guided configuration for new users
- **Built-in Tutorial**: Learn by doing with hands-on chapters
- **Smart Error Messages**: Helpful suggestions and solutions
- **Progress Indicators**: Real-time feedback for all operations
- **Contextual Help**: `ccswarm help <topic>` for detailed guidance
- **System Doctor**: `ccswarm doctor --fix` diagnoses and fixes issues
- **CLI Performance**: Ongoing refactoring for improved performance and maintainability

### Proactive ProactiveMaster
- **Autonomous Orchestration**: Intelligent task prediction and generation (enabled by default)
- **Real-time Progress Analysis**: Continuous monitoring with bottleneck detection
- **Dependency Resolution**: Automatic task ordering and dependency management
- **Goal-Driven Planning**: OKR integration with milestone tracking
- **Pattern Recognition**: Learn from task completion patterns for better predictions
- **Velocity Tracking**: Team performance analysis and optimization suggestions

### Security Agent
- **OWASP Top 10 Scanning**: Comprehensive vulnerability detection
- **Risk Assessment**: Automated security scoring with CI/CD integration
- **Real-time Monitoring**: Continuous vulnerability scanning during development
- **Dependency Security**: Scan npm, cargo, pip, and composer packages
- **Security Reporting**: Detailed reports with remediation suggestions

### Session Management
- **WebSocket Sessions**: Persistent connections via Claude ACP
- **Cross-Platform Support**: Works on Linux, macOS, and Windows
- **Multi-Agent Coordination**: Enhanced message bus architecture
- **Auto-Recovery**: Automatic reconnection and state persistence

### Template System
- **Predefined Templates**: Rust CLI, security review, performance optimization
- **Variable Substitution**: Dynamic content generation with context
- **Category-Based Organization**: Application, utility, review, optimization
- **Custom Templates**: Create and store project-specific templates
- **Validation System**: Type-safe template validation before application

### Collective Intelligence
- **Sangha System**: Buddhist-inspired democratic decision-making
- **Autonomous Self-Extension**: Agents independently analyze and propose improvements
- **Experience-Based Learning**: Continuous introspective analysis drives growth
- **Smart Proposal System**: Structured proposals with consensus algorithms
- **Safe Implementation**: Risk assessment and rollback mechanisms

### Observability & Tracing (NEW in v0.3.8)
- **OpenTelemetry Compatible**: Export traces to Jaeger, Zipkin, or custom backends ⚡
- **Langfuse Integration**: LLM-specific observability with token tracking ⚡
- **Span Tracking**: Trace agent execution across the entire workflow ✅
- **Token Usage Metrics**: Monitor and optimize LLM API costs ✅
- **Trace Visualization**: Hierarchical span trees for debugging ✅

### Human-in-the-Loop (HITL) (NEW in v0.3.8)
- **Approval Workflows**: Gate critical actions with human oversight ✅
- **Policy-Based Rules**: Define approval requirements by risk level ✅
- **Multi-Channel Notifications**: CLI ✅, Slack/Email 🔜
- **Escalation Support**: Timeout ✅, escalation 🔜
- **Audit Trail**: Complete history of all approval decisions ✅

### Long-term Memory & RAG (NEW in v0.3.8)
- **Vector Embeddings**: Semantic search over past experiences 🔜
- **Short-term/Long-term Memory**: Session-aware memory consolidation ✅
- **Retrieval Augmented Generation**: Context-aware agent responses ✅
- **Importance-based Retention**: Smart memory decay and prioritization ✅
- **Multiple Backends**: In-memory ✅, file-based/vector DB 🔜

### Graph-based Workflow Engine (NEW in v0.3.8)
- **DAG Workflows**: Define complex task dependencies as graphs ✅
- **Conditional Branching**: Dynamic workflow paths based on conditions 🔜
- **Parallel Execution**: Run independent tasks concurrently 🔜
- **Approval Gates**: Integrate HITL at workflow checkpoints 🔜
- **Sub-workflows**: Compose complex workflows from simpler ones 🔜

### Benchmark Integration (NEW in v0.3.8)
- **SWE-Bench Style Evaluation**: Standardized agent performance testing ✅
- **Predefined Suites**: Basic coding, bug fixes, refactoring benchmarks ✅
- **Metrics Collection**: Track pass rates, scores, and improvements ✅
- **Leaderboard System**: Compare agent performance over time ✅
- **Custom Benchmarks**: Create project-specific evaluation suites ✅

### Core Capabilities
- **Multi-Provider Support**: Claude Code, Aider, OpenAI Codex, Custom tools
- **Intelligent Delegation**: ProactiveMaster analyzes and assigns tasks optimally
- **Auto-Create System**: Generate complete applications from natural language
- **Enhanced TUI**: Real-time monitoring with task management and filtering
- **Git Worktree Isolation**: Parallel development without conflicts
- **Auto-Accept Mode**: Safe automated execution with risk assessment
- **LLM Quality Judge**: Advanced code evaluation with multi-dimensional scoring
- **Search Agent**: Web search integration via Gemini CLI for research tasks

## Quick Start

> **New to ccswarm?** See the [Getting Started Guide](docs/GETTING_STARTED.md) for a complete walkthrough.
>>>>>>> e29fbdc (docs: restructure README with hub-and-spoke model)

### Installation

```bash
git clone https://github.com/nwiizo/ccswarm.git
cd ccswarm
cargo build --release
<<<<<<< HEAD
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
=======
cargo install --path crates/ccswarm
```

### First Project

```bash
# Interactive setup
ccswarm setup

# Or quick init
ccswarm init --name "MyProject" --agents frontend,backend

# Start system
ccswarm start

# Launch TUI (in another terminal)
ccswarm tui

# Create a task
ccswarm task "Create login form [high] [feature]"

# Auto-create complete app
ccswarm auto-create "Create TODO app with React" --output ./todo-app
```

### Sample Demos

```bash
cd sample/
./setup.sh
./claude_acp_demo.sh      # Claude Code integration
./task_management_demo.sh  # Task management
./multi_agent_demo.sh     # Multi-agent collaboration
```

## Architecture

```
┌─────────────────────────────────────────┐
│         ProactiveMaster                 │ ← Type-State Pattern
│     ├─ Channel-Based Orchestration     │   Zero shared state
│     ├─ Task Analysis & Delegation      │   Pattern matching
│     └─ Quality Review Integration      │   Async/await
├─────────────────────────────────────────┤
│     Claude ACP Integration              │ ← WebSocket (ws://localhost:9100)
│     ├─ Agent Client Protocol           │   JSON-RPC 2.0
│     └─ Real-time Task Delegation       │   Auto-reconnect
├─────────────────────────────────────────┤
│     Specialized Agent Pool              │ ← Actor Model
│     ├─ Frontend Agent (React/Vue/UI)   │
│     ├─ Backend Agent (APIs/Database)   │
│     ├─ DevOps Agent (Docker/CI/CD)     │
│     └─ QA Agent (Testing/Quality)      │
├─────────────────────────────────────────┤
│     Template System                     │ ← Variable substitution
├─────────────────────────────────────────┤
│     Git Worktree Manager                │ ← Isolated development
├─────────────────────────────────────────┤
│     Real-time Monitoring (TUI)          │ ← Crossterm-based
└─────────────────────────────────────────┘
```

### Rust-Native Patterns

- **Type-State Pattern**: Compile-time state validation with zero runtime cost
- **Channel-Based Orchestration**: Message-passing without shared state or locks
- **Iterator Pipelines**: Zero-cost abstractions for task processing
- **Minimal Testing**: Only 8 essential tests covering core functionality
- **No Arc<Mutex>**: Replaced with actor model and channels
>>>>>>> e29fbdc (docs: restructure README with hub-and-spoke model)

## Project Structure

```
ccswarm/
<<<<<<< HEAD
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
=======
├── crates/
│   └── ccswarm/           # Main orchestration crate
│       ├── src/
│       │   ├── acp_claude/  # Claude ACP integration
│       │   ├── cli/         # CLI commands
│       │   ├── orchestrator/ # ProactiveMaster
│       │   └── agent/       # Agent types
│       └── tests/
├── docs/                  # Documentation
└── sample/                # Demo scripts
```

## Contributing

```bash
cargo test --workspace
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Standalone Deployment

ccswarm can run without Claude Code! See [STANDALONE_DEPLOYMENT.md](STANDALONE_DEPLOYMENT.md) for:
- Simulation mode for testing
- Built-in templates without AI
- Docker deployment
- Custom providers

## License
>>>>>>> e29fbdc (docs: restructure README with hub-and-spoke model)

```rust
use ai_session::{SessionManager, SessionConfig};

<<<<<<< HEAD
// Create session manager
let manager = SessionManager::new();
=======
## Acknowledgments
>>>>>>> e29fbdc (docs: restructure README with hub-and-spoke model)

// Create a session
let mut config = SessionConfig::default();
config.name = "dev-session".to_string();
let session = manager.create_session(config).await?;

// Session lifecycle
manager.pause_session(&session.id).await?;
manager.resume_session(&session.id).await?;
```

<<<<<<< HEAD
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
=======
**Experience Claude Code integration with autonomous AI orchestration in ccswarm v0.3.8**
>>>>>>> e29fbdc (docs: restructure README with hub-and-spoke model)
