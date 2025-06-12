# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🚀 Essential Commands

### Building and Testing
```bash
# Build
cargo build                      # Debug build
cargo build --release           # Release build

# Test
cargo test                      # All tests
cargo test -- --nocapture      # With output
cargo test identity            # Specific module
cargo test --test integration_tests  # Integration tests only

# Code Quality
cargo fmt                       # Format
cargo clippy -- -D warnings     # Lint (fail on warnings)
cargo check                     # Quick check
cargo doc --no-deps --open      # Generate docs
```

### Running ccswarm
```bash
# Initialize project
cargo run -- init --name "MyProject" --agents frontend,backend,devops
cargo run -- init --name "AiderProject" --template aider-focused

# Core operations
cargo run -- start               # Start orchestrator
cargo run -- tui                 # Terminal UI with real-time monitoring
cargo run -- status --detailed   # System status

# Auto-create applications from natural language
cargo run -- auto-create "Create TODO app" --output ./my_app
cargo run -- auto-create "Create blog with auth" --output ./blog
cargo run -- auto-create "E-commerce site" --template ecommerce

# Task delegation
cargo run -- delegate analyze "Create login form" --verbose
cargo run -- delegate task "Add auth" --agent backend --priority high

# Session management
cargo run -- session list
cargo run -- session attach <session-id>
cargo run -- session pause <session-id>
cargo run -- worktree list

# Monitoring
cargo run -- monitor --agent backend --filter "error,warning"
RUST_LOG=debug cargo run -- start  # Debug mode
```

## 🏗️ Architecture Overview

ccswarm is an AI-powered multi-agent orchestration system with four key features:

1. **Session-Persistent Architecture** - 93% token reduction through conversation history preservation
2. **Master Delegation System** - Intelligent task analysis and agent assignment  
3. **Auto-Create System** - Generate complete applications from natural language
4. **Multi-Provider Support** - Claude Code, Aider, OpenAI Codex, Custom tools

### Core Modules (`src/`)
- `agent/` - Agent implementations with persistent sessions
- `identity/` - Agent roles and boundary checking (200 tokens vs 2000+)
- `session/` - Session persistence, pooling, and worktree integration
- `orchestrator/` - Master Claude, delegation, and auto-create logic
- `providers/` - Multi-provider support (Claude Code, Aider, Codex)
- `auto_accept/` - Safe automation with risk assessment (1-10 scale)
- `monitoring/` & `streaming/` - Real-time output tracking
- `tui/` - Terminal UI with live monitoring
- `git/` - Worktree-based agent isolation
- `coordination/` - Inter-agent communication bus
- `tmux/` - tmux session management for agents

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
- Conversation history preservation (93% token reduction)
- Session pooling with load balancing
- Batch task execution for efficiency
- Auto-scaling based on workload

### Key Components
- `src/session/persistent.rs` - Conversation history preservation
- `src/session/worktree_session.rs` - Git worktree integration
- `src/session/session_pool.rs` - Load balancing and auto-scaling
- `src/session/manager.rs` - High-level session orchestration

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

// Session reuse
let session = manager.get_or_create_session(role).await?;
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
Task → Master Analysis → Agent Assignment → Execution → Quality Review
         ↓                    ↓
    Confidence Score     Provider Selection
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
├── docker-compose.yml # Multi-container
└── README.md       # Documentation
```

### Auto-Create Templates
Located in `src/auto_create/templates/`:
- `todo.rs` - Full CRUD TODO application
- `blog.rs` - Blog with user authentication
- `ecommerce.rs` - Shopping cart and catalog
- `chat.rs` - Real-time WebSocket chat

## 🔧 Key Implementation Patterns

### Agent Task Flow
1. Receive task from coordination bus
2. Check provider readiness and auto-accept safety
3. Validate task boundaries using identity system
4. Execute in isolated tmux session
5. Stream output in real-time
6. Report results with metrics

### Provider Configuration
```json
{
  "providers": {
    "claude_code": {
      "command": "claude-code-agent",
      "api_key_env": "ANTHROPIC_API_KEY",
      "think_mode": "ultrathink"
    },
    "aider": {
      "command": "aider",
      "args": ["--model", "claude-3-5-sonnet"],
      "auto_commit": true
    }
  }
}
```

### Identity Management
- Each agent maintains strict role boundaries
- CLAUDE.md files reinforce agent identity
- Continuous monitoring prevents drift
- Automatic correction for boundary violations
- Located in `examples/claude-md-templates/`

### Safety Features
- Auto-accept with risk assessment (1-10 scale)
- File protection patterns (`.env`, `*.key`, etc.)
- Emergency stop system
- Pre/post execution validation
- Audit trails via session tracking

## 📊 TUI Commands

Access command mode with 'c':
- `task <description>` - Add task with `[high]`, `[test]`, etc.
- `agent <type>` - Create agent (frontend/backend/devops/qa)
- `filter <pattern>` - Filter output
- `session <cmd>` - Session management (list/attach/pause/resume)
- `worktree <cmd>` - Worktree operations
- `monitor <agent>` - Focus on specific agent
- `help` - Show all commands

### Task Modifiers
- `[high]`, `[medium]`, `[low]` - Priority
- `[bug]`, `[feature]`, `[test]` - Type
- `[auto]` - Enable auto-accept if safe

## 🧪 Testing

```bash
# Module tests
cargo test session        # Session management
cargo test auto_accept    # Safety validation
cargo test monitoring     # Real-time streaming
cargo test provider       # Multi-provider
cargo test identity       # Agent boundaries

# Integration tests
cargo test --test integration_tests

# Examples
cargo run --example todo_app_builder
cargo run --example monitoring_demo
cargo run --example session_persistent_demo
cargo run --example multi_provider_demo
```

## ⚙️ Configuration

### ccswarm.json Structure
```json
{
  "project": {
    "name": "MyProject",
    "master_claude_instructions": "Orchestrate agents..."
  },
  "agents": [
    {
      "name": "frontend-specialist",
      "role": "Frontend",
      "provider": "claude_code",
      "auto_accept": { "enabled": true, "risk_threshold": 5 }
    }
  ],
  "coordination": {
    "method": "JSON_FILES",
    "delegation_strategy": "Hybrid"
  },
  "session_management": {
    "persistent_sessions": true,
    "max_sessions_per_role": 3
  }
}
```

### Migration from Legacy Config
```bash
# Automatic migration
cargo run -- config migrate --input old-config.json --output ccswarm.json
```

## ⚠️ Critical Notes

### Performance
- Session reuse reduces API costs by ~90%
- Batch processing amortizes identity overhead
- Git worktree isolation requires disk space
- JSON coordination may have slight latency
- Real-time monitoring adds <5% overhead

### Security
- tmux session isolation per agent
- API key sandboxing per provider
- Pattern-based file protection
- Audit trails via session tracking
- Emergency stop capability

### Debugging
```bash
RUST_LOG=debug cargo run -- start
RUST_LOG=ccswarm::session=trace cargo run -- start  # Session debugging
cargo run -- tui                                    # Real-time monitoring
tmux ls                                            # View active sessions
tail -f logs/ccswarm.log                           # System logs
```

### Common Issues
- **Session not found**: Check `ccswarm session list`
- **Provider errors**: Verify API keys and commands
- **Worktree conflicts**: Use `ccswarm worktree clean`
- **Auto-accept blocked**: Check risk assessment logs