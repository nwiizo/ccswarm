# CLAUDE.md - ccswarm v0.2.0 Development Guide

This file provides guidance to Claude Code (claude.ai/code) when working with the ccswarm codebase. Updated for v0.2.0 with enhanced quality review, improved session management, and comprehensive command documentation.

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

## 🏗️ Architecture Overview (v0.2.0)

ccswarm is an AI-powered multi-agent orchestration system with enhanced features:

1. **Session-Persistent Architecture** - 93% token reduction with improved pooling and load balancing
2. **Master Delegation System** - Intelligent task analysis with enhanced routing algorithms
3. **Auto-Create System** - Generate complete applications with expanded template support
4. **Multi-Provider Support** - Claude Code, Aider, OpenAI Codex, Custom tools with better configuration
5. **Quality Review System** - Automated quality checks with iterative remediation tracking
6. **Enhanced TUI** - Real-time monitoring with improved task management and filtering

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

### Identity Management (v0.2.0 Enhanced)
- Each agent maintains strict role boundaries
- CLAUDE.md files reinforce agent identity
- Continuous monitoring prevents drift
- Automatic correction for boundary violations
- Located in `examples/claude-md-templates/`
- Improved boundary checking algorithms
- Better handling of cross-domain tasks

### Safety Features (v0.2.0 Enhanced)
- Auto-accept with improved risk assessment (1-10 scale)
- Extended file protection patterns (`.env`, `*.key`, `*.pem`, etc.)
- Emergency stop system with graceful shutdown
- Enhanced pre/post execution validation
- Comprehensive audit trails via session tracking
- Better handling of sensitive operations
- Improved error recovery mechanisms

## 📊 TUI Commands (v0.2.0 Enhanced)

Access command mode with 'c':
- `task <description>` - Add task with enhanced modifiers
- `agent <type>` - Create agent (frontend/backend/devops/qa)
- `filter <pattern>` - Advanced output filtering
- `session <cmd>` - Session management (list/attach/pause/resume/stats)
- `worktree <cmd>` - Worktree operations (list/clean/status)
- `monitor <agent>` - Focus on specific agent with metrics
- `review <cmd>` - Quality review commands (status/history/trigger)
- `delegate <cmd>` - Delegation commands (analyze/task/stats)
- `help` - Show all commands with descriptions

### Task Modifiers (Enhanced)
- `[high]`, `[medium]`, `[low]` - Priority
- `[bug]`, `[feature]`, `[test]`, `[docs]`, `[refactor]` - Type
- `[auto]` - Enable auto-accept if safe
- `[review]` - Force quality review after completion
- `[urgent]` - Bypass queue for critical tasks

## 🧪 Testing (v0.2.0)

```bash
# Module tests
cargo test session        # Session management
cargo test auto_accept    # Safety validation
cargo test monitoring     # Real-time streaming
cargo test provider       # Multi-provider
cargo test identity       # Agent boundaries
cargo test quality_review # Quality review system
cargo test delegation     # Task delegation
cargo test tui           # Terminal UI

# Integration tests
cargo test --test integration_tests
cargo test --test quality_integration_tests  # New in v0.2.0

# Examples (now in demos/)
cargo run --example todo_app_builder         # See demos/todo-app/
cargo run --example monitoring_demo          # See demos/multi-agent/
cargo run --example session_persistent_demo  # See demos/session-persistence/
cargo run --example auto_create_demo         # See demos/auto-create/
cargo run --example quality_review_demo      # See demos/quality-review/
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

# Validate configuration
cargo run -- config validate --file ccswarm.json

# Generate template with all options
cargo run -- config generate --template full --output ccswarm-full.json
```

## ⚠️ Critical Notes (v0.2.0)

### Performance
- Session reuse reduces API costs by ~93% with improved pooling
- Batch processing amortizes identity overhead
- Git worktree isolation requires disk space (~100MB per agent)
- JSON coordination optimized for <100ms latency
- Real-time monitoring adds <3% overhead with v0.2.0 optimizations
- Quality review adds minimal overhead with async processing

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

### Common Issues (v0.2.0)
- **Session not found**: Check `ccswarm session list` or `ccswarm session stats`
- **Provider errors**: Verify API keys with `ccswarm config validate`
- **Worktree conflicts**: Use `ccswarm worktree clean` or `ccswarm worktree status`
- **Auto-accept blocked**: Check risk assessment logs in `logs/safety.log`
- **Quality review failures**: Check `ccswarm review history` for details
- **TUI rendering issues**: Try `ccswarm tui --reset` to clear state

## 🔍 Quality Review System

### Overview
Master Claude performs automated quality reviews on completed tasks, creating remediation tasks when issues are found.

### Review Process
- **Interval**: Every 30 seconds
- **Scope**: All completed tasks in agent history
- **Metrics**: Test coverage, code complexity, security, documentation

### Quality Standards (src/identity/mod.rs)
```rust
pub struct QualityStandards {
    pub min_test_coverage: f64,      // Default: 0.85 (85%)
    pub max_complexity: u32,         // Cyclomatic complexity limit
    pub security_scan_required: bool,
    pub performance_threshold: Duration,
}
```

### Review Message Flow
```rust
// When quality issues are detected:
AgentMessage::QualityIssue {
    agent_id: String,
    task_id: String,
    issues: Vec<String>,  // e.g., ["Low test coverage", "High complexity"]
}
```

### Remediation Task Creation
When issues are found, a remediation task is automatically created:
- **Task Type**: `TaskType::Remediation`
- **Priority**: Always `High`
- **Assignment**: Same agent that completed original task
- **Parent Task**: Links to original task for tracking

### Fix Instructions Mapping
```rust
"Low test coverage" → "Add unit tests to achieve 85% coverage"
"High complexity" → "Refactor to reduce cyclomatic complexity"
"Security vulnerability" → "Fix security issues and validate inputs"
"Missing documentation" → "Add comprehensive documentation"
```

### Review History Tracking
```rust
pub struct ReviewHistoryEntry {
    pub task_id: String,
    pub agent_id: String,
    pub review_date: DateTime<Utc>,
    pub issues_found: Vec<String>,
    pub remediation_task_id: Option<String>,
    pub review_passed: bool,
    pub iteration: u32,  // Tracks review attempts
}
```

### Implementation Files (v0.2.0)
- **Quality Review**: `src/orchestrator/mod.rs::perform_quality_review()`
- **Message Handler**: `src/orchestrator/mod.rs::handle_agent_message()`
- **Task Types**: `src/agent/task.rs` (added `Remediation` variant)
- **Review History**: `src/orchestrator/review_history.rs`
- **Tests**: `src/orchestrator/review_test.rs`
- **TUI Enhancements**: `src/tui/enhanced_commands.rs`
- **Session Pool**: `src/session/pool_v2.rs`

## 📚 Command Documentation

Comprehensive command documentation is now available in `.claude/commands/` directory:
- Each command has its own markdown file
- Includes usage, options, examples, and related commands
- Auto-generated help from these docs

Access with:
```bash
ls .claude/commands/
cat .claude/commands/init.md  # Example
```