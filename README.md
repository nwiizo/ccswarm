# ccswarm: AI Multi-Agent Orchestration System

> **Version 0.4.3** - Rust-Native Multi-Agent Orchestration with ai-session Integration

[![CI](https://github.com/nwiizo/ccswarm/workflows/CI/badge.svg)](https://github.com/nwiizo/ccswarm/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-edition_2024-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**ccswarm** is a workflow automation framework for coordinating specialized AI agents using Claude Code CLI. It provides task delegation infrastructure, template-based scaffolding, and Git worktree isolation for parallel development.

> **AI Integration**: ccswarm uses native PTY sessions via **ai-session** crate. Multi-provider system exists but orchestrator coordination loop is not fully implemented. See [docs/UPCOMING_FEATURES.md](docs/UPCOMING_FEATURES.md) for roadmap.

## Implementation Status

| Category | Status | Description |
|----------|--------|-------------|
| CLI Infrastructure | ✅ Working | All commands parse and route correctly |
| Session Management | ✅ Working | Native PTY-based sessions |
| TUI Monitoring | ✅ Working | Real-time terminal UI with ratatui |
| Configuration | ✅ Working | Project configs, agent settings |
| Git Worktrees | ✅ Working | Create, list, remove, prune |
| Template System | ✅ Working | Predefined templates for app scaffolding |
| Task Queue | ✅ Working | Task queuing and tracking |
| AI Execution | ⚠️ Simulated | Returns keyword-based responses |
| Provider System | 🚧 Planned | Code exists, not integrated |
| ACP Integration | 🚧 Stub | CLI wrapper only, no WebSocket |
| Parallel Executor | ⚠️ Partial | Structure exists, not integrated with orchestrator |
| `start` Command | ⚠️ Partial | Initializes but coordination loop not implemented |
| Auto-Create | ⚠️ Partial | Templates work, full AI generation incomplete |
| Sangha (Voting) | 🚧 Planned | Data structures only |
| Extensions | 🚧 Planned | Stub implementation |

> **Note**: See [docs/analysis/](docs/analysis/) for detailed capability gap analysis.

## Documentation

| Document | Description |
|----------|-------------|
| **[Getting Started](docs/GETTING_STARTED.md)** | Installation, tutorial, first project |
| **[Architecture](docs/ARCHITECTURE.md)** | System design, components, data flow |
| **[Configuration](docs/CONFIGURATION.md)** | All configuration options with examples |
| **[Commands Reference](docs/COMMANDS.md)** | Complete CLI command reference |
| **[Claude ACP Guide](docs/CLAUDE_ACP.md)** | Claude Code integration setup |
| **[Troubleshooting](docs/TROUBLESHOOTING.md)** | Common issues and solutions |
| **[Upcoming Features](docs/UPCOMING_FEATURES.md)** | v0.4.0 implementation patterns |
| **[Application Spec](docs/APPLICATION_SPEC.md)** | Detailed feature specifications |
| **[Contributing](CONTRIBUTING.md)** | How to contribute |

## Key Features

> **Status Legend**: ✅ Working | ⚠️ Partial | 🚧 Planned

### Multi-Provider AI Integration (🚧 Planned)
- **Provider Architecture**: 5 providers implemented (ClaudeCode, Aider, ClaudeAPI, Codex, Custom)
- **Current Execution**: Simulated responses (keyword-based matching)
- **ACP Module**: Stub implementation (CLI wrapper, no WebSocket)
- **Status**: Code exists but not integrated with orchestrator
- See [docs/UPCOMING_FEATURES.md](docs/UPCOMING_FEATURES.md) for integration roadmap

### Developer Experience (✅ Working)
- **Interactive Setup Wizard**: Guided configuration for new users
- **Built-in Tutorial**: Learn by doing with hands-on chapters
- **Smart Error Messages**: Helpful suggestions and solutions
- **Progress Indicators**: Real-time feedback for operations
- **Contextual Help**: `ccswarm help <topic>` for detailed guidance
- **System Doctor**: `ccswarm doctor --fix` diagnoses and fixes issues

### Session Management (✅ Working)
- **Native PTY Sessions**: Cross-platform terminal sessions (no tmux)
- **Session Lifecycle**: Create, list, attach, detach, pause, resume
- **Auto-Recovery**: Automatic reconnection and state persistence
- **Resource Monitoring**: Track session usage and statistics

### Template System (✅ Working)
- **Predefined Templates**: Rust CLI, security review, performance optimization
- **Variable Substitution**: Dynamic content generation with context
- **Category Organization**: Application, utility, review, optimization
- **Custom Templates**: Create and store project-specific templates
- **Validation System**: Type-safe template validation

### Git Worktree Isolation (✅ Working)
- **Parallel Development**: Multiple branches simultaneously without conflicts
- **Per-Agent Workspaces**: Isolated environments for each agent role
- **Automatic Management**: Create, list, remove, prune operations
- **Safe Isolation**: No cross-agent file access or conflicts

### Task Queue & Tracking (✅ Working)
- **Task Management**: Create, queue, and track tasks
- **Priority Levels**: High, medium, low task prioritization
- **Status Tracking**: Pending, in-progress, completed states
- **Task Metadata**: Tags, descriptions, dependencies

### Orchestrator & Delegation (⚠️ Partial)
- **ProactiveMaster**: Intelligent task analysis (structure exists)
- **Agent Assignment**: Optimal agent selection (basic routing works)
- **Coordination Loop**: Continuous monitoring (not implemented)
- **Dependency Resolution**: Auto task ordering (planned)

### Parallel Execution (⚠️ Partial)
- **ParallelExecutor**: Multi-agent concurrency structure exists
- **Message Bus**: Inter-agent communication framework ready
- **Resource Management**: File locking and conflict prevention designed
- **Integration**: Not wired to orchestrator yet

### Auto-Create System (⚠️ Partial)
- **Template Scaffolding**: Generate project structure (working)
- **Variable Substitution**: Dynamic content generation (working)
- **AI-Driven Generation**: Natural language to app (incomplete)
- **Custom Templates**: User-defined project types (working)

### Observability & Tracing (✅ v0.4.3)
- **Span Tracking**: Trace agent execution across workflows
- **Token Usage Metrics**: Monitor and optimize LLM API costs
- **Trace Visualization**: Hierarchical span trees for debugging
- **OpenTelemetry Export**: Jaeger, Zipkin integration (file-export ready)
- **Langfuse Integration**: LLM-specific observability (file-export ready)

### Human-in-the-Loop (✅ v0.4.3)
- **Approval Workflows**: Gate critical actions with human oversight
- **Policy-Based Rules**: Define approval requirements by risk level
- **CLI Notifications**: Interactive approval prompts
- **Timeout Handling**: Automatic escalation on timeout
- **Audit Trail**: Complete history of approval decisions
- **Multi-Channel**: Slack/Email integration (planned)

### Long-term Memory & RAG (✅ v0.4.3)
- **Short/Long-term Memory**: Session-aware memory consolidation
- **Retrieval Augmented Generation**: Context-aware responses
- **Importance Retention**: Smart memory decay and prioritization
- **In-Memory Backend**: Fast access for current session
- **Vector Embeddings**: Semantic search (planned)
- **Persistent Storage**: File/vector DB backends (planned)

### Graph Workflow Engine (✅ v0.4.3)
- **DAG Workflows**: Define task dependencies as graphs
- **Conditional Execution**: Dynamic workflow paths
- **Parallel Tasks**: Run independent tasks concurrently (structure ready)
- **Approval Gates**: HITL integration at checkpoints (planned)
- **Sub-workflows**: Compose complex workflows (planned)

### Benchmark Integration (✅ v0.4.3)
- **SWE-Bench Style**: Standardized agent performance testing
- **Predefined Suites**: Basic coding, bug fixes, refactoring
- **Metrics Collection**: Track pass rates, scores, improvements
- **Leaderboard System**: Compare agent performance over time
- **Custom Benchmarks**: Create project-specific evaluation suites

### Collective Intelligence (🚧 Planned)
- **Sangha System**: Democratic decision-making (data structures only)
- **Self-Extension**: Agents propose improvements (stub implementation)
- **Experience Learning**: Continuous introspective analysis (planned)
- **Consensus Algorithms**: Smart proposal voting (planned)
- **Safe Implementation**: Risk assessment and rollback (planned)

## In Development (v0.4.0)

See **[docs/UPCOMING_FEATURES.md](docs/UPCOMING_FEATURES.md)** for detailed implementation patterns and integration guides.

Features with code already in codebase but not fully integrated:
- 🔧 **Hook System Integration** - Extensible execution hooks with priority registry
- 🔧 **Verification Agent** - Auto-verify applications with 6-check workflow
- 🔧 **DynamicSpawner** - Intelligent workload balancing for agent selection
- 🔧 **Parallel Execution** - True multi-agent parallelism (command & PTY-based)
- 🔧 **ai-session MessageBus** - Inter-agent coordination with 93% token savings
- 🔧 **Session Persistence** - Resume/fork/checkpoint with crash recovery

**Roadmap:**
1. Fix `start` command coordination loop (Critical)
2. Wire ParallelExecutor to orchestrator (Critical)
3. Implement inter-process communication (Critical)
4. Integrate ai-session features (High Value)

## Quick Start

> **New to ccswarm?** See the [Getting Started Guide](docs/GETTING_STARTED.md) for a complete walkthrough.

### Installation

```bash
git clone https://github.com/nwiizo/ccswarm.git
cd ccswarm
cargo build --release
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
│     Multi-Provider Integration          │ ← Planned (v0.4.0)
│     ├─ Provider System (5 impl.)       │   Code exists, not wired
│     └─ Current: Simulated Execution    │   Keyword-based responses
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

## Project Structure

```
ccswarm/
├── crates/
│   └── ccswarm/           # Main orchestration crate
│       ├── src/
│       │   ├── acp_claude/  # Claude ACP integration
│       │   ├── cli/         # CLI commands
│       │   ├── orchestrator/ # ProactiveMaster
│       │   ├── agent/       # Agent types
│       │   ├── session/     # Session management
│       │   ├── template/    # Template system
│       │   ├── subagent/    # Parallel execution
│       │   ├── tui/         # Terminal UI
│       │   └── resource/    # Resource monitoring
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

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

**Experience Claude Code integration with autonomous AI orchestration in ccswarm v0.4.3**
