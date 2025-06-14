# ccswarm: AI Multi-Agent Orchestration System

> 🚀 **Version 0.2.2** - Enhanced with Anthropic-inspired interleaved thinking, advanced LLM quality judge, and improved CI/CD compatibility!

[![Crates.io](https://img.shields.io/crates/v/ccswarm.svg)](https://crates.io/crates/ccswarm)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/crates/d/ccswarm.svg)](https://crates.io/crates/ccswarm)

**ccswarm** is an AI-powered multi-agent orchestration system that manages specialized AI agents using Claude Code, Aider, and other providers. It features session persistence, intelligent task delegation, auto-create functionality, and real-time monitoring through a Terminal UI.

## 🌟 Key Features (v0.2.2)

- **🤖 Multi-Provider Support**: Claude Code, Aider, OpenAI Codex, Custom tools
- **💾 Session Persistence**: 93% token reduction through conversation history
- **🎯 Intelligent Delegation**: Master Claude analyzes and assigns tasks optimally
- **🚀 Auto-Create System**: Generate complete applications from natural language
- **📊 Enhanced TUI**: Real-time monitoring with improved task management and filtering
- **🔄 Git Worktree Isolation**: Parallel development without conflicts
- **✅ Auto-Accept Mode**: Safe automated execution with risk assessment
- **🔒 Execution Mode**: Runs with `--dangerously-skip-permissions` by default
- **🔍 LLM Quality Judge**: Advanced code evaluation using Claude with multi-dimensional scoring
- **📚 Command Documentation**: Comprehensive docs in `.claude/commands/`
- **🎛️ Improved Session Pool**: Better load balancing and resource management
- **🔧 Enhanced Config System**: More flexible provider and agent configuration

## 🚀 Quick Start

> **Note**: Don't have Claude Code or API keys? Check out our [Standalone Deployment Guide](STANDALONE_DEPLOYMENT.md) to run ccswarm without any AI dependencies!

### 1. Installation

```bash
# Install from crates.io
cargo install ccswarm

# Or build from source
git clone https://github.com/nwiizo/ccswarm.git
cd ccswarm
cargo build --release
cargo install --path .
```

### 2. Initialize Project

```bash
# Basic initialization
ccswarm init --name "MyProject" --agents frontend,backend,devops

# With specific template
ccswarm init --name "AiderProject" --template aider-focused
```

### 3. Start System

```bash
# Terminal 1: Start orchestrator
ccswarm start

# Terminal 2: Start TUI for monitoring
ccswarm tui
```

### 4. Create Applications

```bash
# Generate TODO app
ccswarm auto-create "Create TODO app" --output ./my_app

# Generate blog
ccswarm auto-create "Create blog with auth" --output ./blog
```

## 🏗️ Architecture

### Session-Persistent Architecture
```
┌─────────────────────────────────────────┐
│         Master Claude                   │ ← Orchestration & Delegation
│     ├─ Task Assignment                  │
│     ├─ Quality Review (30s interval)    │
│     └─ Remediation Task Generation      │
├─────────────────────────────────────────┤
│     Session-Persistent Manager          │ ← 93% Token Reduction
│     ├─ Session Pool & Reuse            │
│     ├─ Conversation History (50 msgs)   │
│     └─ Batch Task Processing           │
├─────────────────────────────────────────┤
│     Git Worktree Manager                │ ← Isolated Development
├─────────────────────────────────────────┤
│     Multi-Provider Agent Pool           │
│     ├─ Claude Code (default)           │
│     ├─ Aider                           │
│     ├─ OpenAI Codex                    │
│     └─ Custom Tools                    │
├─────────────────────────────────────────┤
│     Real-time Monitoring (TUI)          │ ← Live Status Updates
└─────────────────────────────────────────┘
```

### Agent Roles
```rust
pub enum AgentRole {
    Frontend,  // UI development only
    Backend,   // API development only
    DevOps,    // Infrastructure only
    QA,        // Testing only
    Master,    // Orchestration (no coding)
}
```

## 📋 Core Commands

> 📖 **Full command documentation available in `.claude/commands/`**

### Basic Operations
```bash
# Initialize project
ccswarm init --name "Project" --agents frontend,backend

# Start system
ccswarm start

# Launch TUI (enhanced in v0.2.0)
ccswarm tui

# Check status
ccswarm status --detailed

# Stop orchestrator
ccswarm stop
```

### Task Management
```bash
# Add task (enhanced in v0.2.0)
ccswarm task "Create login form" --priority high --type feature

# Delegate task with improved analysis
ccswarm delegate analyze "Add authentication" --verbose
ccswarm delegate task "Add auth" --agent backend --priority high

# View delegation statistics
ccswarm delegate stats --period 24h
```

### Auto-Create Applications (Enhanced)
```bash
# TODO app with modern stack
ccswarm auto-create "Create TODO app" --output ./todo

# Blog with advanced features
ccswarm auto-create "Blog with auth and comments" --output ./blog

# E-commerce with full stack
ccswarm auto-create "Online shop with cart" --output ./shop

# Custom template support (v0.2.0)
ccswarm auto-create "Project description" --template custom --output ./app
```

## 🎮 Terminal UI (TUI)

Start with `ccswarm tui`:

### Key Bindings
- `Tab/Shift+Tab` - Switch tabs
- `↑↓/jk` - Navigate
- `Enter` - Select/Activate
- `c` - Command mode
- `t` - Add task
- `q` - Quit

### Command Mode (`c` key) - Enhanced in v0.2.0
```
task <description> [high|medium|low] [feature|bug|test|docs|refactor]
agent <type>
session list|attach|pause|resume|stats
filter <pattern>
worktree list|clean
monitor <agent>
review status|history
help
```

### Smart Task Parsing
```
task Fix login bug [high] [bug]
task Add docs [docs]
task Create dashboard [medium] [feature]
```

## ⚙️ Configuration

### ccswarm.json Structure
```json
{
  "project": {
    "name": "MyProject",
    "master_claude_instructions": "Orchestrate agents efficiently"
  },
  "agents": [
    {
      "name": "frontend-specialist",
      "role": "Frontend",
      "provider": "claude_code",
      "claude_config": {
        "model": "claude-3.5-sonnet",
        "dangerous_skip": true,
        "think_mode": "think_hard"
      },
      "auto_accept": {
        "enabled": true,
        "risk_threshold": 5
      }
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

### Provider Configuration

#### Claude Code (Default)
```json
{
  "provider": "claude_code",
  "claude_config": {
    "model": "claude-3.5-sonnet",
    "dangerous_skip": true,
    "think_mode": "think_hard"
  }
}
```

#### Aider
```json
{
  "provider": "aider",
  "config": {
    "model": "claude-3-5-sonnet",
    "auto_commit": true,
    "edit_format": "diff"
  }
}
```

## 🎯 Master Delegation System

### Delegation Strategies
- **ContentBased**: Keyword matching
- **LoadBalanced**: Workload distribution
- **ExpertiseBased**: Historical performance
- **WorkflowBased**: Task dependencies
- **Hybrid** (default): Combined approach

### Delegation Commands
```bash
# Analyze task
ccswarm delegate analyze "Create responsive nav" --verbose

# Manual delegation
ccswarm delegate task "Add API endpoint" --agent backend

# View statistics
ccswarm delegate stats --period 24h
```

## 🚀 Auto-Create System

### Supported Applications
- TODO apps with CRUD
- Blogs with authentication
- E-commerce platforms
- Real-time chat apps
- Custom applications

### Generated Structure
```
my_app/
├── index.html       # React app
├── app.js          # Components
├── server.js       # Express API
├── package.json    # Dependencies
├── Dockerfile      # Container
├── README.md       # Documentation
└── .gitignore      # Git config
```

## 🔧 Session Management

### Session Features
- Persistent conversation history
- Session pooling and reuse
- Batch task execution
- Auto-scaling

### Session Commands
```bash
# List sessions
ccswarm session list

# Attach to session
ccswarm session attach <session-id>

# Pause/Resume
ccswarm session pause <session-id>
ccswarm session resume <session-id>
```

## 🛡️ Safety Features (Enhanced)

### Auto-Accept Mode
- Risk assessment (1-10 scale)
- Enhanced file protection patterns
- Emergency stop capability
- Comprehensive audit trails
- Pre/post execution validation

### Execution Mode
By default, ccswarm runs with `dangerous_skip: true`, which adds the `--dangerously-skip-permissions` flag to Claude Code commands for automated execution.

### New in v0.2.0
- Improved risk assessment algorithms
- Better handling of sensitive files
- Enhanced validation patterns

## 🔍 LLM-as-Judge Quality Review (v0.2.2)

### Advanced Code Evaluation
Master Claude now uses sophisticated LLM-based evaluation to assess code quality across 8 dimensions:

1. **Multi-Dimensional Scoring (0.0-1.0)**
   - **Correctness**: Does the code implement requirements correctly?
   - **Maintainability**: Is it well-structured and easy to modify?
   - **Test Quality**: Are tests comprehensive with good coverage?
   - **Security**: Does it follow security best practices?
   - **Performance**: Are there optimization opportunities?
   - **Documentation**: Is the code properly documented?
   - **Architecture**: Does it follow good design patterns?
   - **Error Handling**: Is error handling robust?

2. **Issue Severity Classification**
   - **Critical**: Must fix immediately (e.g., security vulnerabilities)
   - **High**: Should fix before deployment (e.g., missing auth)
   - **Medium**: Should address soon (e.g., low test coverage)
   - **Low**: Nice to fix (e.g., minor documentation gaps)

3. **Intelligent Remediation**
   - LLM generates detailed, context-aware fix instructions
   - Suggestions tailored to agent specialization
   - Tracks confidence levels for each evaluation
   - Provides specific code examples and best practices

### Example Quality Evaluation
```json
{
  "overall_score": 0.78,
  "dimensions": {
    "correctness": 0.90,
    "test_quality": 0.65,
    "security": 0.75,
    "documentation": 0.70
  },
  "issues": [
    {
      "severity": "high",
      "category": "TestCoverage",
      "description": "Test coverage is 65%, below 85% requirement",
      "suggested_fix": "Add unit tests for error cases"
    }
  ],
  "feedback": "Good implementation but needs more comprehensive testing",
  "passes_standards": false,
  "confidence": 0.92
}
```

### Review Workflow
```
Task Completed → LLM Quality Review → Detailed Evaluation → Remediation Task
                     ↓                      ↓                      ↓
                 Score ≥ 0.85          Issues Found          Smart Fix Instructions
                     ↓                      ↓                      ↓
                 Task Approved         Agent Fixes            Re-evaluate
```

## 📊 Monitoring

### Real-time Metrics
- Agent health status
- Task completion rates
- Session utilization
- Performance tracking

### Debug Mode
```bash
# Verbose logging
RUST_LOG=debug ccswarm start

# Session debugging
RUST_LOG=ccswarm::session=trace ccswarm start

# View tmux sessions
tmux ls
```

## 🧪 Testing

```bash
# All tests
cargo test

# Specific module
cargo test session
cargo test identity
cargo test quality_review  # New in v0.2.0

# Integration tests
cargo test --test integration_tests

# Examples (relocated to demos/)
cargo run --example todo_app_builder     # See demos/todo-app/
cargo run --example monitoring_demo      # See demos/multi-agent/
cargo run --example session_demo         # See demos/session-persistence/
cargo run --example auto_create_demo     # See demos/auto-create/
```

## 🚨 Troubleshooting

### Common Issues

**Session not found**
```bash
ccswarm session list
ccswarm session create --agent frontend
```

**Provider errors**
```bash
# Check API keys
echo $ANTHROPIC_API_KEY

# Verify provider config
ccswarm config show
```

**Worktree conflicts**
```bash
ccswarm worktree list
ccswarm worktree clean
```

## 🛠️ Development

### Adding Custom Providers
1. Implement `ProviderExecutor` trait
2. Add to `ProviderType` enum
3. Update configuration parsing
4. Add provider-specific configuration options

### v0.2.0 Architecture Improvements
- Enhanced session pool management
- Improved task routing algorithms
- Better error recovery mechanisms
- Extended provider API support

## 🚀 Standalone Deployment (No AI Dependencies)

ccswarm can run without Claude Code or other AI providers! Check out the [**STANDALONE_DEPLOYMENT.md**](STANDALONE_DEPLOYMENT.md) guide for:

- **Simulation Mode**: Run with simulated agents for testing and learning
- **Built-in Templates**: Generate complete applications without AI providers
- **Docker Deployment**: Containerized setup for easy deployment
- **Custom Providers**: Create your own agent implementations
- **Offline Operation**: Full functionality without internet connection

### Quick Standalone Example
```bash
# Start in simulation mode
CCSWARM_SIMULATION=true ccswarm start

# Generate a complete TODO app without AI
ccswarm auto-create "Create TODO app" --output ./my-app

# Run the generated app
cd my-app && npm install && npm start
```

For detailed instructions, examples, and Docker configurations, see [STANDALONE_DEPLOYMENT.md](STANDALONE_DEPLOYMENT.md).

### Contributing
```bash
# Fork and clone
git clone https://github.com/yourusername/ccswarm.git

# Run tests
cargo test

# Format code
cargo fmt
cargo clippy -- -D warnings

# Check documentation
cargo doc --no-deps --open
```

## 📄 License

MIT License - see [LICENSE](LICENSE)

## 🙏 Acknowledgments

- Anthropic for Claude and Claude Code
- Rust community for excellent libraries
- Contributors and early adopters

## 📝 Release Notes (v0.2.0)

### New Features
- Enhanced quality review system with iteration tracking
- Improved TUI with better task filtering and management
- Comprehensive command documentation in `.claude/commands/`
- Better session pool management and load balancing
- Extended provider configuration options

### Improvements
- More robust error handling and recovery
- Enhanced auto-create templates
- Better git worktree management
- Improved delegation algorithms
- Performance optimizations

### Bug Fixes
- Fixed session persistence edge cases
- Resolved TUI rendering issues
- Corrected task priority handling
- Fixed provider timeout issues

---

**Experience the power of AI agent orchestration with ccswarm v0.2.0** 🚀