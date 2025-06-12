# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🚀 Essential Commands

### Building and Testing
```bash
# Build
cargo build --release

# Test
cargo test                        # All tests
cargo test -- --nocapture        # With output
cargo test identity              # Specific module

# Code Quality
cargo fmt                        # Format
cargo clippy                     # Lint
cargo check                      # Quick check
```

### Running ccswarm
```bash
# Initialize project
cargo run -- init --name "MyProject" --agents frontend,backend,devops

# Core operations
cargo run -- start               # Start orchestrator
cargo run -- tui                 # Terminal UI with real-time monitoring
cargo run -- status --detailed   # System status

# Auto-create applications from natural language
cargo run -- auto-create "Create TODO app" --output ./my_app
cargo run -- auto-create "Create blog with auth" --output ./blog

# Task delegation
cargo run -- delegate analyze "Create login form" --verbose
cargo run -- delegate task "Add auth" --agent backend --priority high

# Session management
cargo run -- session list
cargo run -- worktree list
```

## 🏗️ Architecture Overview

ccswarm is an AI-powered multi-agent orchestration system with three key features:

1. **Session-Persistent Architecture** - Session management and reuse capabilities
2. **Master Delegation System** - Task analysis and agent assignment  
3. **Auto-Create System** - Generate application templates from predefined structures

### Core Modules (`src/`)
- `agent/` - Agent implementations with persistent sessions
- `identity/` - Agent roles and boundary checking
- `session/` - Session persistence and pooling
- `orchestrator/` - Master Claude, delegation, and auto-create
- `providers/` - Multi-provider support (Claude Code, Aider, Codex)
- `auto_accept/` - Safe automation with risk assessment
- `monitoring/` & `streaming/` - Real-time output tracking
- `tui/` - Terminal UI with live monitoring
- `git/` - Worktree-based agent isolation
- `coordination/` - Inter-agent communication

### Agent Specializations
```rust
pub enum AgentRole {
    Frontend,  // UI, React, styling, client-side only
    Backend,   // APIs, server, database only
    DevOps,    // Infrastructure, Docker, CI/CD only
    QA,        // Testing, quality assurance only
    Master,    // Orchestration, no direct coding
}
```

## 💡 Session-Persistent Architecture

### Session Management Features
- Conversation history preservation
- Session pooling and reuse
- Batch task execution

### Key Components
- `src/session/persistent.rs` - Conversation history preservation
- `src/session/worktree_session.rs` - Git worktree integration
- `src/session/session_pool.rs` - Load balancing and auto-scaling

### Usage Pattern
```rust
// Single task
let result = manager.execute_task(role, task, config).await?;

// Batch execution (maximum efficiency)
let results = manager.execute_task_batch(
    role,
    vec![task1, task2, task3, task4, task5],
    config,
).await?;
```

## 🎯 Master Delegation System

### Delegation Strategies
1. **ContentBased** - Keyword and technology matching
2. **LoadBalanced** - Workload distribution
3. **ExpertiseBased** - Historical performance
4. **WorkflowBased** - Task dependencies
5. **Hybrid** (default) - Combines all strategies

### Delegation Flow
```
Task → Master Analysis → Agent Assignment → Execution
         ↓
    Confidence Score & Reasoning
```

## 🚀 Auto-Create System

### Supported Applications
- TODO apps with CRUD operations
- Blogs with authentication
- E-commerce with shopping cart
- Real-time chat with WebSockets
- Custom applications via AI analysis

### Generated Structure
```
my_app/
├── index.html       # React entry
├── app.js          # Components
├── server.js       # Express API
├── package.json    # Dependencies
├── Dockerfile      # Containerization
└── README.md       # Documentation
```

## 🔧 Key Implementation Patterns

### Agent Task Flow
1. Receive task from coordination bus
2. Check provider readiness and auto-accept safety
3. Validate task boundaries
4. Execute in isolated tmux session
5. Stream output in real-time
6. Report results with metrics

### Identity Management
- Each agent maintains strict role boundaries
- CLAUDE.md files reinforce agent identity
- Continuous monitoring prevents drift
- Automatic correction for boundary violations

### Safety Features
- Auto-accept with risk assessment (1-10 scale)
- File protection patterns
- Emergency stop system
- Pre/post execution validation

## 📊 TUI Commands

Access command mode with 'c':
- `task <description>` - Add task with `[high]`, `[test]`, etc.
- `agent <type>` - Create agent (frontend/backend/devops/qa)
- `filter <pattern>` - Filter output
- `session <cmd>` - Session management
- `help` - Show all commands

## 🧪 Testing

```bash
# Module tests
cargo test session        # Session management
cargo test auto_accept    # Safety validation
cargo test monitoring     # Real-time streaming
cargo test provider       # Multi-provider

# Examples
cargo run --example todo_app_builder
cargo run --example monitoring_demo
```

## ⚠️ Critical Notes

### Performance
- Session reuse can improve efficiency
- Git worktree isolation requires disk space
- JSON coordination may have latency
- Real-time monitoring adds minimal overhead

### Security
- tmux session isolation
- API key sandboxing per provider
- Pattern-based file protection
- Audit trails via session tracking

### Debugging
```bash
RUST_LOG=debug cargo run -- start
cargo run -- tui                    # Real-time monitoring
tmux ls                            # View active sessions
```