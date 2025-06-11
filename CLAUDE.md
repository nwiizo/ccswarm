# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🚀 Essential Commands

### Building and Testing
```bash
# Build the project
cargo build --release

# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test identity

# Format code
cargo fmt

# Lint code
cargo clippy

# Check without building
cargo check
```

### Running ccswarm
```bash
# Initialize new project
cargo run -- init --name "MyProject" --agents frontend,backend,devops

# Initialize with specific providers
cargo run -- init --name "AiderProject" --template aider-focused
cargo run -- init --name "MixedProject" --template mixed-providers

# Start orchestrator
cargo run -- start

# Start TUI (Terminal User Interface) - ENHANCED!
cargo run -- tui

# Session management
cargo run -- session create --agent frontend --workspace ./agents/frontend
cargo run -- session list
cargo run -- session pause <session-id>
cargo run -- session resume <session-id>

# Show system status
cargo run -- status --detailed

# Add a task
cargo run -- task "Create login component" --priority high --type development

# List agents
cargo run -- agents

# Real-time monitoring
cargo run --example monitoring_demo

# Manage worktrees
cargo run -- worktree list
cargo run -- worktree create agents/test-agent feature/test

# Generate configuration
cargo run -- config generate --template full-stack
cargo run -- config generate --template aider-focused
cargo run -- config generate --template mixed-providers
```

### Development Examples
```bash
# Run the TODO app builder example
cargo run --example todo_app_builder

# Run simple TODO test
cargo run --example todo_test_simple

# Run monitoring system demo
cargo run --example monitoring_demo

# Test with verbose logging
RUST_LOG=debug cargo run -- start --verbose

# Test specific modules
cargo test session
cargo test auto_accept
cargo test monitoring
cargo test provider
```

## 🚀 Session-Persistent Agent Architecture - Revolutionary Token Efficiency

### 93% Token Reduction Implementation

The ccswarm system now features the **Session-Persistent Agent Architecture**, delivering revolutionary 93% token reduction through intelligent session management and context preservation.

#### Key Technical Innovations

**1. Persistent Claude Code Sessions (`src/agent/persistent.rs`)**
- Maintains conversation history across multiple task executions
- One-time identity establishment per session lifecycle
- 50-message rolling buffer for optimal context preservation
- Lightweight identity reminders (200 tokens vs 2000+ tokens)

**2. Git Worktree Integration (`src/session/worktree_session.rs`)**
- Combines git worktree isolation with persistent sessions
- Automatic worktree lifecycle management
- Compact CLAUDE.md generation per worktree
- Session reuse across compatible tasks

**3. Advanced Session Pooling (`src/session/session_pool.rs`)**
- Load balancing strategies (LeastLoaded, RoundRobin, Adaptive)
- Auto-scaling based on workload
- Health monitoring and performance metrics
- Session creation throttling and resource management

**4. Intelligent Batch Processing**
- Execute multiple tasks in single sessions
- Amortize identity overhead across task batches
- Context-aware task grouping and routing

#### Performance Metrics

```
Traditional Approach (50 tasks):
├── CLAUDE.md reading: 50 × 2,000 = 100,000 tokens
├── Identity establishment: 50 × 500 = 25,000 tokens  
├── Task prompts: 50 × 800 = 40,000 tokens
├── Boundary verification: 50 × 300 = 15,000 tokens
└── Total: 180,000 tokens

Session-Persistent Approach (50 tasks):
├── Initial identity: 1 × 3,600 = 3,600 tokens
├── Additional tasks: 49 × 200 = 9,800 tokens
└── Total: 13,400 tokens (93% reduction)
```

#### Usage Examples

**Single Task Execution with Session Persistence:**
```rust
use ccswarm::session::worktree_session::{WorktreeSessionManager, WorktreeSessionConfig};
use ccswarm::identity::default_frontend_role;

let mut manager = WorktreeSessionManager::new(config)?;
manager.start().await?;

// Execute task - session created and reused automatically
let result = manager.execute_task(
    default_frontend_role(),
    task,
    claude_config,
).await?;
```

**Batch Task Execution for Maximum Efficiency:**
```rust
// Execute multiple tasks in single session - maximum token efficiency
let results = manager.execute_task_batch(
    default_frontend_role(),
    vec![task1, task2, task3, task4, task5], // All executed in one session
    claude_config,
).await?;
```

**Advanced Session Pooling:**
```rust
use ccswarm::session::session_pool::{SessionPool, SessionPoolConfig, LoadBalancingStrategy};

let pool_config = SessionPoolConfig {
    min_sessions_per_role: 2,
    max_sessions_per_role: 5,
    load_balancing: LoadBalancingStrategy::LeastLoaded,
    auto_scaling: AutoScalingConfig {
        enabled: true,
        scale_up_threshold: 0.8,
        target_load: 0.6,
        ..Default::default()
    },
    ..Default::default()
};

let mut pool = SessionPool::new(worktree_config, pool_config).await?;
pool.start().await?;

// Pool automatically manages session lifecycle and load balancing
let result = pool.execute_task(role, task, claude_config).await?;
```

## 🏗️ Architecture Overview

### Core System Design
ccswarm is an **advanced multi-agent orchestration system** featuring the **Session-Persistent Agent Architecture** for **93% token reduction**. Master Claude Code orchestrates specialized AI agents with persistent sessions, auto-accept mode, real-time monitoring, and multi-provider support, enabling scalable distributed development using Git worktrees, intelligent session pooling, and CLAUDE.md configuration files.

### Key Architectural Components

#### 1. Agent Identity System (`src/identity/`)
- **Multi-layered identity establishment** to solve Claude's "forgetfulness problem"
- **Environment-level identification** via `AgentIdentity` struct with workspace paths and session IDs
- **CLAUDE.md reinforcement system** with role-specific instructions and boundary definitions
- **Continuous identity monitoring** via `IdentityMonitor` that tracks responses for drift detection
- **Boundary enforcement** through `TaskBoundaryChecker` that evaluates task appropriateness

#### 2. Agent Specializations (`src/identity/mod.rs`)
```rust
pub enum AgentRole {
    Frontend { technologies, responsibilities, boundaries },
    Backend { technologies, responsibilities, boundaries },
    DevOps { technologies, responsibilities, boundaries },
    QA { technologies, responsibilities, boundaries },
    Master { oversight_roles, quality_standards },
}
```

#### 3. Session-Persistent Management System (`src/session/`)
- **Persistent Claude Code sessions** with conversation history preservation
- **Session pooling and reuse** for maximum token efficiency
- **Worktree-integrated sessions** combining git isolation with session persistence
- **Advanced load balancing** with multiple strategies (LeastLoaded, RoundRobin, Adaptive)
- **Auto-scaling** based on workload and performance metrics
- **Batch task processing** for amortized overhead costs
- **Health monitoring** and performance tracking
- **Session lifecycle management** including creation, monitoring, and cleanup

**Session Management Components:**
- `persistent_session.rs` - Core session persistence and lifecycle management
- `worktree_session.rs` - Git worktree integration with persistent sessions
- `session_pool.rs` - Advanced pooling, load balancing, and auto-scaling
- `coordinator.rs` - Integration with existing coordination systems

#### 4. Auto-Accept Engine (`src/auto_accept/`)
- **Safety-first automation** with configurable trust levels
- **Operation risk assessment** (1-10 scale) for different task types
- **Multi-layered validation** with pre/post execution checks
- **Emergency stop system** with manual reset requirements
- **File protection patterns** for critical system files

#### 5. Real-time Monitoring (`src/monitoring/` & `src/streaming/`)
- **Live output streaming** from all agents with structured logging
- **Performance metrics** and statistics tracking
- **Output filtering** by agent, type, content patterns
- **Subscription-based broadcasting** for TUI and external consumers
- **Buffer management** with automatic cleanup and size limits

#### 6. Multi-Provider Support (`src/providers/`)
- **Claude Code Provider** - Enhanced original functionality
- **Aider Provider** - AI coding assistant with auto-commit
- **OpenAI Codex Provider** - HTTP API integration
- **Custom Provider** - Flexible command-line tool integration
- **Unified interface** for consistent agent behavior across providers

#### 7. Git Worktree Integration (`src/git/`)
- **Shell-based Git operations** to avoid libgit2 dependencies
- **Isolated agent workspaces** where each agent operates in its own worktree
- **Branch management** with automatic creation and cleanup
- **Worktree lifecycle management** including creation, removal, and pruning

#### 8. Task Coordination (`src/coordination/`)
- **JSON-based communication bus** for inter-agent messaging
- **Task queue management** with priority-based scheduling
- **Status tracking** for real-time agent monitoring
- **Coordination protocols** for task delegation and boundary checking

#### 9. Agent Integration (`src/agent/`)
- **Multi-provider execution** with environment variables and workspace isolation
- **Think mode utilization** (think, think_hard, ultrathink, etc.)
- **Permission management** with `dangerous_skip` for automated agents
- **Auto-accept integration** with safety validation
- **Session-aware execution** with tmux integration
- **JSON output handling** for programmatic control

#### 10. Terminal User Interface (`src/tui/`)
- **Real-time dashboard** built with ratatui for monitoring multi-agent activities
- **Interactive tabs** for Overview, Agents, Tasks, and Logs with live streaming
- **Command mode** with intelligent task creation and agent management
- **Live output filtering** by agent, type, and content patterns
- **Provider information display** with icons and status indicators
- **Session management UI** for tmux session control
- **Auto-scroll control** and search capabilities
- **Event-driven architecture** using tokio for async event handling

### Enhanced Data Flow Architecture
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Master Claude │───▶│ Task Distribution│───▶│ Multi-Provider  │
│   Orchestrator  │    │  & Auto-Accept   │    │ Agent Pool      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         │              ┌────────▼────────┐             │
         │              │ Session Manager │             │
┌────────▼────────┐     │ + tmux Sessions │     ┌───────▼───────┐
│ Real-time       │     │ + Auto-Accept   │     │ Git Worktrees │
│ Monitoring      │     │ + Safety Checks │     │ + CLAUDE.md   │
│ & Streaming     │     └─────────────────┘     │ Configuration │
└─────────────────┘              │              └───────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                    JSON Coordination Bus + Output Streams
```

## 🔧 Key Implementation Patterns

### Enhanced Agent Initialization Sequence
1. **Identity Creation**: Generate unique agent ID and session
2. **Provider Selection**: Configure AI provider (Claude Code, Aider, Codex, Custom)
3. **Session Setup**: Create tmux session with workspace isolation
4. **Worktree Setup**: Create isolated Git environment
5. **CLAUDE.md Generation**: Write role-specific instructions
6. **Auto-Accept Configuration**: Set up safety parameters and trusted operations
7. **Monitoring Registration**: Connect to real-time output streaming
8. **Boundary Verification**: Test task acceptance/rejection
9. **Status Registration**: Join coordination system

### Enhanced Task Execution Flow
1. **Task Reception**: Receive task from coordination bus
2. **Provider Readiness**: Check provider availability and session status
3. **Auto-Accept Evaluation**: Assess if task can be auto-accepted safely
4. **Boundary Evaluation**: Check if task matches specialization
5. **Safety Validation**: Pre-execution safety checks and file pattern validation
6. **Decision Making**: Accept, Delegate, Clarify, or Reject
7. **Session Execution**: Run in tmux session with provider-specific execution
8. **Real-time Monitoring**: Stream output with identity tracking and metrics
9. **Post-Execution Validation**: Verify changes and run quality checks
10. **Result Reporting**: Update coordination system with outcomes and metrics

### Identity Drift Prevention
- **Mandatory identity headers** in all agent responses
- **Response pattern analysis** to detect boundary violations
- **Automatic correction prompts** when drift is detected
- **Critical failure handling** for severe identity loss

## 📁 Module Organization

### Primary Source Structure
- `src/agent/` - Core agent implementations and task management
- `src/identity/` - Agent identity, roles, and boundary checking
- `src/session/` - **NEW** tmux-based session management and lifecycle
- `src/auto_accept/` - **NEW** Safety-first automation with risk assessment
- `src/monitoring/` - **NEW** Real-time output streaming and metrics
- `src/streaming/` - **NEW** Subscription-based output broadcasting
- `src/providers/` - **NEW** Multi-provider support (Claude Code, Aider, Codex, Custom)
- `src/tmux/` - **NEW** tmux client integration and session operations
- `src/coordination/` - Inter-agent communication and task routing
- `src/git/` - Git worktree management and shell operations
- `src/orchestrator/` - Master Claude and system coordination
- `src/config/` - Configuration management and templates
- `src/cli/` - Command-line interface and user interactions
- `src/tui/` - Terminal User Interface with real-time monitoring
- `src/workspace/` - Simple workspace management without Git

### TUI Key Bindings (inspired by claude-squad)
- **Tab/Shift+Tab**: Switch between tabs (Overview, Agents, Tasks, Logs)
- **↑/↓ or j/k**: Navigate through lists
- **Enter**: Activate/view details of selected item
- **n**: Create new agent session
- **d**: Delete current agent session
- **t**: Add new task
- **c**: Command prompt (NEW!)
- **s**: Show system status
- **r**: Refresh data
- **q**: Quit TUI
- **Esc**: Cancel current action

### TUI Command System
The TUI includes a powerful command system accessible via **'c'** key:

**Available Commands:**
- `help` - Show all available commands
- `status` - Show detailed system status
- `agents` - List all agents with details
- `tasks` - List all pending tasks
- `task <description>` - Add new task (supports [high]/[low] for priority, [test]/[docs] for type)
- `agent <type>` - Create new agent (frontend/backend/devops/qa)
- `session <create|list|pause|resume|attach|detach> [args]` - **NEW** Session management
- `monitor [agent] [filter]` - **NEW** Real-time output monitoring
- `filter <pattern>` - **NEW** Filter logs by pattern
- `nofilter` - **NEW** Clear all filters
- `autoscroll [on|off]` - **NEW** Control auto-scroll behavior
- `start` - Start orchestrator
- `stop` - Stop orchestrator
- `refresh` - Refresh all data
- `clear` - Clear logs
- `worktree [list|prune]` - Manage git worktrees

**Smart Task Creation:**
- `task Fix login bug [high] [bugfix]` - Creates high priority bugfix task
- `task Write API docs [docs]` - Creates documentation task
- `task Add tests for auth [test]` - Creates testing task

### Configuration Templates
- `examples/ccswarm-full-stack.json` - Complete multi-agent setup
- `examples/ccswarm-frontend-only.json` - Frontend-focused configuration
- `examples/ccswarm-minimal.json` - Minimal agent setup
- `examples/ccswarm-aider-focused.json` - **NEW** Aider-based development
- `examples/ccswarm-mixed-providers.json` - **NEW** Mixed provider configuration
- `examples/ccswarm-openai-codex.json` - **NEW** OpenAI Codex integration
- `examples/ccswarm-custom-tools.json` - **NEW** Custom tool integration
- `examples/claude-md-templates/` - Role-specific CLAUDE.md templates
- `examples/PROVIDER_EXAMPLES.md` - **NEW** Provider configuration guide

## 🎯 Working with Agent Boundaries

### Understanding Agent Roles
Each agent has **strict specialization boundaries**:
- **Frontend**: UI components, styling, client-side logic only
- **Backend**: APIs, databases, server-side logic only  
- **DevOps**: Infrastructure, deployment, CI/CD only
- **QA**: Testing, quality assurance, validation only
- **Master**: Orchestration, quality review, no direct code

### Task Delegation Protocol
When tasks cross boundaries, agents must:
1. **Recognize scope mismatch** through boundary checking
2. **Generate delegation recommendation** with target agent
3. **Provide context and rationale** for delegation
4. **Update coordination system** with delegation status

### Identity Verification Requirements
All agent responses must include:
```
🤖 AGENT: [AgentType]
📁 WORKSPACE: [WorkspacePath]
🎯 SCOPE: [TaskAssessment]
```

## 🔍 Testing Strategy

### Unit Tests Focus Areas
- Agent identity establishment and monitoring
- **Session management and tmux integration**
- **Auto-accept safety validation and risk assessment**
- **Real-time monitoring and output streaming**
- **Multi-provider execution and configuration**
- Task boundary checking and delegation
- Git worktree operations and cleanup
- Configuration parsing and validation

### Integration Tests
- Multi-agent task coordination workflows
- **Session lifecycle management with auto-accept**
- **Provider-agnostic task execution**
- **Real-time monitoring with multiple agents**
- Identity drift detection and correction
- Worktree isolation and communication
- End-to-end task completion scenarios

### Test Execution
```bash
# All tests
cargo test

# Integration tests only
cargo test --test integration_tests

# Module-specific tests
cargo test identity
cargo test session
cargo test auto_accept
cargo test monitoring
cargo test provider
cargo test boundary

# Test with verbose output
cargo test -- --nocapture

# Run monitoring demo
cargo run --example monitoring_demo
```

## 🚨 Critical Implementation Notes

### Security Considerations
- **Environment isolation** prevents agent cross-contamination
- **Session isolation** with tmux sessions for secure execution
- **Auto-accept safety** with multi-layered validation and emergency stops
- **Provider security** with API key management and sandboxing
- **Permission management** controls execution privileges per provider
- **Boundary enforcement** prevents unauthorized operations
- **Session tracking** enables audit and debugging
- **File protection** with pattern-based restrictions for critical files

### Performance Considerations
- **Real-time streaming** with efficient buffering and subscriber management
- **Session reuse** minimizes tmux session creation overhead
- **Provider optimization** with connection pooling and caching
- **Auto-accept efficiency** reduces manual intervention for safe operations
- **Shell-based Git operations** avoid dependency issues but may be slower
- **JSON coordination** provides simple but potentially high-latency communication
- **Workspace isolation** requires disk space for multiple worktrees
- **Identity monitoring** adds overhead to all agent responses
- **Memory management** with automatic buffer cleanup and size limits

### Enhanced Debugging and Monitoring
- **Real-time TUI monitoring** with live output streaming and filtering
- **Session debugging** with tmux session inspection and attachment
- **Auto-accept metrics** for safety validation and risk assessment
- **Provider-specific logging** with structured output and error handling
- Use `RUST_LOG=debug` for detailed execution tracing
- Use `cargo run -- tui` for real-time system monitoring
- Use `cargo run --example monitoring_demo` for output streaming demo
- Check `coordination/` directory for inter-agent communication logs
- Monitor worktree status with `ccswarm worktree list`
- Verify agent boundaries with `ccswarm status --detailed`
- Monitor session health with `ccswarm session list`
- Filter logs by agent, type, or content in TUI