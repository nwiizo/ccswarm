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

# Start orchestrator
cargo run -- start

# Start TUI (Terminal User Interface) - NEW!
cargo run -- tui

# Show system status
cargo run -- status --detailed

# Add a task
cargo run -- task "Create login component" --priority high --type development

# List agents
cargo run -- agents

# Manage worktrees
cargo run -- worktree list
cargo run -- worktree create agents/test-agent feature/test

# Generate configuration
cargo run -- config generate --template full-stack
```

### Development Examples
```bash
# Run the TODO app builder example
cargo run --example todo_app_builder

# Run simple TODO test
cargo run --example todo_test_simple

# Test with verbose logging
RUST_LOG=debug cargo run -- start --verbose
```

## 🏗️ Architecture Overview

### Core System Design
ccswarm is a **multi-agent orchestration system** where specialized Claude Code agents collaborate through **Git worktrees** and **JSON-based coordination**. Each agent maintains strict boundaries and identity management to prevent role drift.

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

#### 3. Git Worktree Integration (`src/git/`)
- **Shell-based Git operations** to avoid libgit2 dependencies
- **Isolated agent workspaces** where each agent operates in its own worktree
- **Branch management** with automatic creation and cleanup
- **Worktree lifecycle management** including creation, removal, and pruning

#### 4. Task Coordination (`src/coordination/`)
- **JSON-based communication bus** for inter-agent messaging
- **Task queue management** with priority-based scheduling
- **Status tracking** for real-time agent monitoring
- **Coordination protocols** for task delegation and boundary checking

#### 5. Claude Code Integration (`src/agent/`)
- **Direct Claude Code execution** with environment variables and workspace isolation
- **Think mode utilization** (think, think_hard, ultrathink, etc.)
- **Permission management** with `dangerous_skip` for automated agents
- **JSON output handling** for programmatic control

#### 6. Terminal User Interface (`src/tui/`)
- **Real-time dashboard** built with ratatui for monitoring multi-agent activities
- **Interactive tabs** for Overview, Agents, Tasks, and Logs
- **Live status updates** with agent health, task progress, and system metrics
- **Keyboard navigation** inspired by claude-squad with intuitive controls
- **Event-driven architecture** using tokio for async event handling

### Data Flow Architecture
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Master Claude │───▶│ Task Distribution│───▶│ Specialized     │
│   Orchestrator  │    │    & Routing     │    │ Agent Pool      │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         │              ┌────────▼────────┐             │
         │              │ Git Worktrees   │             │
         │              │ + CLAUDE.md     │             │
         │              │ Configuration   │             │
         │              └─────────────────┘             │
         │                                              │
         └──────────────────────────────────────────────┘
                    JSON Coordination Bus
```

## 🔧 Key Implementation Patterns

### Agent Initialization Sequence
1. **Identity Creation**: Generate unique agent ID and session
2. **Worktree Setup**: Create isolated Git environment
3. **CLAUDE.md Generation**: Write role-specific instructions
4. **Boundary Verification**: Test task acceptance/rejection
5. **Status Registration**: Join coordination system

### Task Execution Flow
1. **Task Reception**: Receive task from coordination bus
2. **Boundary Evaluation**: Check if task matches specialization
3. **Decision Making**: Accept, Delegate, Clarify, or Reject
4. **Execution with Monitoring**: Run Claude Code with identity tracking
5. **Result Reporting**: Update coordination system with outcomes

### Identity Drift Prevention
- **Mandatory identity headers** in all agent responses
- **Response pattern analysis** to detect boundary violations
- **Automatic correction prompts** when drift is detected
- **Critical failure handling** for severe identity loss

## 📁 Module Organization

### Primary Source Structure
- `src/agent/` - Core agent implementations and task management
- `src/identity/` - Agent identity, roles, and boundary checking
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
- `examples/claude-md-templates/` - Role-specific CLAUDE.md templates

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
- Task boundary checking and delegation
- Git worktree operations and cleanup
- Configuration parsing and validation

### Integration Tests
- Multi-agent task coordination workflows
- Identity drift detection and correction
- Worktree isolation and communication
- End-to-end task completion scenarios

### Test Execution
```bash
# All tests
cargo test

# Integration tests only
cargo test --test integration_tests

# Identity-specific tests
cargo test identity

# Boundary checking tests
cargo test boundary
```

## 🚨 Critical Implementation Notes

### Security Considerations
- **Environment isolation** prevents agent cross-contamination
- **Permission management** controls Claude Code execution privileges
- **Boundary enforcement** prevents unauthorized operations
- **Session tracking** enables audit and debugging

### Performance Considerations
- **Shell-based Git operations** avoid dependency issues but may be slower
- **JSON coordination** provides simple but potentially high-latency communication
- **Workspace isolation** requires disk space for multiple worktrees
- **Identity monitoring** adds overhead to all agent responses

### Debugging and Monitoring
- Use `RUST_LOG=debug` for detailed execution tracing
- Check `coordination/` directory for inter-agent communication logs
- Monitor worktree status with `ccswarm worktree list`
- Verify agent boundaries with `ccswarm status --detailed`