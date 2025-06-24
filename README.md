# ccswarm: AI Multi-Agent Orchestration System

> 🚀 **Version 0.3.5** - Autonomous orchestration with proactive intelligence and security monitoring

[![Crates.io](https://img.shields.io/crates/v/ccswarm.svg)](https://crates.io/crates/ccswarm)
[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/crates/d/ccswarm.svg)](https://crates.io/crates/ccswarm)

**ccswarm** is an AI-powered multi-agent orchestration system that manages specialized AI agents using Claude Code, Aider, and other providers. Built in Rust for performance and reliability, it features autonomous task prediction, real-time security monitoring, and intelligent delegation.

## 📚 Documentation

| Document | Description |
|----------|-------------|
| **[Getting Started Guide](docs/GETTING_STARTED.md)** | Complete tutorial for new users with step-by-step instructions |
| **[Configuration Reference](docs/CONFIGURATION.md)** | Comprehensive guide to all configuration options |
| **[Troubleshooting Guide](docs/TROUBLESHOOTING.md)** | Solutions for common issues and debugging tips |
| **[Contributing Guide](CONTRIBUTING.md)** | How to contribute to the project |
| **[Architecture Overview](docs/ARCHITECTURE.md)** | Technical architecture and design decisions |
| **[Application Specification](docs/APPLICATION_SPEC.md)** | Detailed feature specifications and API reference |

## 🎯 Quick Navigation

- [Installation](#-quick-start) • [Features](#-key-features) • [Architecture](#-architecture) • [Commands](#-core-commands)
- [Tutorial](#25-learn-with-interactive-tutorial) • [Configuration](#-configuration) • [Auto-Create](#-auto-create-system) • [Monitoring](#-terminal-ui-tui)
- [Troubleshooting](docs/TROUBLESHOOTING.md) • [Contributing](CONTRIBUTING.md) • [Documentation](docs/)

## 📦 Workspace Structure

ccswarm is organized as a Rust workspace with two main crates:

### Crates Overview

- **`crates/ccswarm`**: The main orchestration system and CLI
  - Multi-agent orchestration with Master Claude
  - Provider integrations (Claude Code, Aider, etc.)
  - Task management and delegation
  - Sangha collective intelligence
  - Auto-create application generator
  - **Integrates with**: ai-session for terminal management

- **`crates/ai-session`**: Advanced AI-optimized terminal session management (standalone crate)
  - **93% token savings** through intelligent context compression
  - Native cross-platform PTY implementation (no tmux dependency)
  - Multi-agent coordination with message bus architecture
  - Session persistence and crash recovery
  - MCP (Model Context Protocol) HTTP API server
  - Semantic output parsing and analysis
  - **Can be used independently** of ccswarm or integrated with it
  - **Documentation**: [AI-Session README](crates/ai-session/README.md) | [Architecture](crates/ai-session/docs/ARCHITECTURE.md)

### Directory Structure
```
ccswarm/
├── Cargo.toml              # Workspace definition
├── crates/
│   ├── ccswarm/           # Main orchestration crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   └── tests/
│   └── ai-session/        # Terminal session crate
│       ├── Cargo.toml
│       ├── src/
│       └── examples/
├── docs/                  # Shared documentation
└── demos/                 # Example applications

## 🌟 Key Features

### 🎯 Developer Experience First
- **Interactive Setup Wizard**: Guided configuration for new users
- **Built-in Tutorial**: Learn by doing with hands-on chapters  
- **Smart Error Messages**: Helpful suggestions and solutions
- **Progress Indicators**: Real-time feedback for all operations
- **Contextual Help**: `ccswarm help <topic>` for detailed guidance
- **System Doctor**: `ccswarm doctor --fix` diagnoses and fixes issues

### 🧠 Proactive Master Claude
- **Autonomous Orchestration**: Intelligent task prediction and generation (enabled by default)
- **Real-time Progress Analysis**: Continuous monitoring with bottleneck detection
- **Dependency Resolution**: Automatic task ordering and dependency management
- **Goal-Driven Planning**: OKR integration with milestone tracking
- **Pattern Recognition**: Learn from task completion patterns for better predictions
- **Velocity Tracking**: Team performance analysis and optimization suggestions

### 🔒 Security Agent
- **OWASP Top 10 Scanning**: Comprehensive vulnerability detection
- **Risk Assessment**: Automated security scoring with CI/CD integration
- **Real-time Monitoring**: Continuous vulnerability scanning during development
- **Dependency Security**: Scan npm, cargo, pip, and composer packages
- **Security Reporting**: Detailed reports with remediation suggestions

### 🖥️ AI-Session Integration: Revolutionary Terminal Management
- **Native Session Management**: Powered by ai-session crate with zero external dependencies (no tmux required)
- **93% Token Savings**: Intelligent conversation history compression and context reuse
- **Cross-Platform PTY**: Native terminal emulation on Linux, macOS (Windows support in ai-session)
- **MCP Protocol Support**: Model Context Protocol HTTP API server for seamless AI integration
- **Multi-Agent Coordination**: Enhanced message bus architecture with agent-specific sessions
- **Semantic Output Analysis**: Intelligent parsing of build results, test outputs, and error messages
- **Session Persistence**: Automatic crash recovery and state restoration
- **Standalone Capability**: ai-session can be used independently for any AI terminal workflows

### 🏛️ Collective Intelligence
- **Sangha System**: Buddhist-inspired democratic decision-making
- **Autonomous Self-Extension**: Agents independently analyze and propose improvements
- **Experience-Based Learning**: Continuous introspective analysis drives growth
- **Smart Proposal System**: Structured proposals with consensus algorithms
- **Safe Implementation**: Risk assessment and rollback mechanisms

### 🎯 Core Capabilities
- **Multi-Provider Support**: Claude Code, Aider, OpenAI Codex, Custom tools
- **Intelligent Delegation**: Master Claude analyzes and assigns tasks optimally
- **Auto-Create System**: Generate complete applications from natural language
- **Enhanced TUI**: Real-time monitoring with task management and filtering
- **Git Worktree Isolation**: Parallel development without conflicts
- **Auto-Accept Mode**: Safe automated execution with risk assessment
- **LLM Quality Judge**: Advanced code evaluation with multi-dimensional scoring

## 🚀 Quick Start

> **New to ccswarm?** Start with our [📖 Getting Started Guide](docs/GETTING_STARTED.md) for a comprehensive walkthrough with examples and best practices!

> **Note**: Don't have Claude Code or API keys? Check out our [Standalone Deployment Guide](STANDALONE_DEPLOYMENT.md) to run ccswarm without any AI dependencies!

### 1. Installation

```bash
# Install from crates.io
cargo install ccswarm

# Or build from source (workspace-aware)
git clone https://github.com/nwiizo/ccswarm.git
cd ccswarm
cargo build --release
cargo install --path crates/ccswarm

# Optional: Install ai-session CLI separately
cargo install --path crates/ai-session
```

### 2. Initialize Project

```bash
# Interactive setup wizard (recommended for first-time users)
ccswarm setup

# Or use quick initialization
ccswarm init --name "MyProject" --agents frontend,backend,devops

# With specific template
ccswarm init --name "AiderProject" --template aider-focused
```

### 2.5. Learn with Interactive Tutorial

```bash
# Start the interactive tutorial
ccswarm tutorial

# Jump to specific chapter
ccswarm tutorial --chapter 3
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

ccswarm v0.3.5 features a comprehensive multi-layer architecture designed for autonomous operation:

```
┌─────────────────────────────────────────┐
│         Proactive Master Claude         │ ← Autonomous Orchestration
│     ├─ Intelligent Task Prediction     │   (Enabled by Default)
│     ├─ Real-time Progress Analysis      │   30s standard / 15s high-freq
│     ├─ Dependency Resolution Engine     │
│     ├─ Goal & Milestone Tracking        │
│     └─ Bottleneck Detection & Resolution│
├─────────────────────────────────────────┤
│         Security Agent                  │ ← OWASP Top 10 Scanning
│     ├─ Vulnerability Detection          │
│     ├─ Dependency Security Scanning     │
│     ├─ Real-time Risk Assessment        │
│     └─ Security Score Calculation       │
├─────────────────────────────────────────┤
│        AI-Session Manager               │ ← Native Terminal Management
│     ├─ Cross-Platform PTY Support      │
│     ├─ MCP Protocol Integration         │
│     ├─ Session Persistence (93% saves)  │
│     ├─ Multi-Agent Message Bus          │
│     └─ Conversation History (50 msgs)   │
├─────────────────────────────────────────┤
│     Sangha Collective Intelligence      │ ← Democratic Decision Making
│     ├─ Proposal System                 │
│     ├─ Consensus Algorithms             │
│     └─ Self-Extension Framework         │
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

### 🎆 Key Benefits of the Two-Crate Architecture

#### 🚀 For ccswarm Users (Full AI Orchestration)
- **Zero Setup Complexity**: ai-session integration is automatic and transparent
- **Intelligent Delegation**: Master Claude uses ai-session's semantic parsing for better decisions
- **93% Cost Savings**: ai-session's token compression reduces API costs dramatically
- **Multi-Agent Coordination**: Seamless agent communication through ai-session's message bus
- **Quality Assurance**: ai-session's output analysis powers the LLM quality judge

#### 🧠 For ai-session Users (Terminal Management)
- **Universal Compatibility**: Works with any AI application, not just ccswarm
- **Zero External Dependencies**: No tmux, screen, or other session managers required
- **Native Performance**: Optimized PTY implementation for each platform
- **Standalone Operation**: Complete functionality without ccswarm
- **Easy Integration**: Simple library API for embedding in other projects

#### 🔗 For Both (Integrated Benefits)
- **Shared Innovation**: Improvements in ai-session automatically benefit ccswarm
- **Consistent Experience**: Same session management across manual and automated workflows
- **Flexible Deployment**: Use ai-session for development, ccswarm for production
- **Learning Path**: Start with ai-session, graduate to full ccswarm orchestration

### Integration Architecture
```rust
// ccswarm uses ai-session for all agent interactions
pub enum AgentRole {
    Frontend,  // UI development (via ai-session)
    Backend,   // API development (via ai-session)
    DevOps,    // Infrastructure (via ai-session)
    QA,        // Testing (via ai-session)
    Master,    // Orchestration only (coordinates ai-sessions)
}

// Each agent gets its own ai-session instance
struct Agent {
    role: AgentRole,
    session: ai_session::AISession,  // Managed by ai-session crate
    config: AgentConfig,
}

// ai-session can also be used independently:
use ai_session::{SessionManager, SessionConfig};
let session = SessionManager::new()
    .create_session_with_ai_features().await?;
```

## 🤔 When to Use Which Crate?

### 🚀 Use ccswarm when you want:
- **Full AI orchestration** with Master Claude making intelligent decisions
- **Multi-agent workflows** with specialized roles (Frontend, Backend, DevOps, QA)
- **Autonomous task generation** and proactive intelligence
- **Quality review systems** with automatic remediation
- **Sangha collective intelligence** for democratic decision-making
- **Auto-create functionality** to generate complete applications
- **Enterprise AI development** with comprehensive governance

### 🧠 Use ai-session when you want:
- **AI-optimized terminal sessions** for any application (not just ccswarm)
- **93% token savings** through intelligent context compression
- **Native PTY management** without tmux dependencies
- **Multi-agent coordination** in your own AI systems
- **Session persistence** and crash recovery
- **MCP protocol server** for AI tool integration
- **Semantic output parsing** for builds, tests, and logs
- **Building your own AI tools** that need terminal management

### 🔗 Use both when you want:
- **The complete AI development experience** (recommended)
- **Gradual adoption**: Start with ai-session, add ccswarm orchestration later
- **Hybrid workflows**: Manual ai-session for experimentation, ccswarm for production
- **Team scenarios**: Different team members using different levels of automation

---

## 📋 Core Commands

> 📖 **ccswarm commands**: Full documentation in `.claude/commands/`  
> 🧠 **ai-session commands**: See [AI-Session CLI Guide](crates/ai-session/docs/CLI_GUIDE.md)

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
    "master_claude": {
      "role": "technical_lead",
      "quality_threshold": 0.85,
      "think_mode": "ultra_think",
      "permission_level": "supervised",
      "enable_proactive_mode": true,
      "proactive_frequency": 30,
      "high_frequency": 15,
      "claude_config": {
        "model": "claude-3.5-sonnet",
        "dangerous_skip": true,
        "think_mode": "ultra_think"
      }
    }
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

## 🔧 AI-Session Integration

### Advanced Session Features (powered by ai-session crate)
- **93% token reduction** through intelligent context compression
- **Persistent conversation history** with crash recovery
- **Session pooling and reuse** for efficient resource utilization
- **Multi-agent message bus** for coordinated AI workflows
- **Semantic output parsing** for build results, tests, and logs
- **MCP protocol HTTP server** for external tool integration
- **Cross-platform PTY** implementation (Linux, macOS, Windows)
- **Standalone operation** - ai-session can be used without ccswarm

### AI-Session Crate Benefits
- **Zero external dependencies** - no tmux server required
- **Native performance** - optimized PTY implementation per OS
- **Memory efficient** - ~70% reduction through zstd compression
- **Developer friendly** - comprehensive API and CLI tools

### Proactive & Security Commands
```bash
# Proactive mode is enabled by default in all new projects
ccswarm start  # Automatically enables proactive analysis

# Manual proactive analysis trigger
ccswarm proactive analyze --all-agents
ccswarm proactive analyze --agent frontend

# Security scanning
ccswarm security scan --directory ./src
ccswarm security report --show-history
ccswarm security check --owasp-top-10

# Goal and milestone management
ccswarm goal set "Build MVP" --deadline 30d
ccswarm milestone add "Frontend Complete" --deadline 14d
ccswarm progress show --detailed

# Dependency analysis
ccswarm deps analyze --show-blockers
ccswarm deps resolve --auto-order
```

### AI-Session Management
```bash
# List ai-sessions with token savings (powered by ai-session crate)
ccswarm session list
ccswarm session stats --show-savings

# Create and manage sessions
ccswarm session create --agent frontend --enable-ai-features
ccswarm session attach <session-id>
ccswarm session pause <session-id>
ccswarm session resume <session-id>

# MCP protocol support (ai-session HTTP API server)
ccswarm session start-mcp-server --port 3000
ccswarm session mcp-status

# Session optimization (93% token reduction)
ccswarm session compress --threshold 0.8
ccswarm session optimize --all

# Direct ai-session CLI usage (independent of ccswarm)
ai-session create --name dev --ai-context
ai-session list --detailed
ai-session exec dev "cargo build" --capture
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

# View ai-sessions
ccswarm session list
```

## 🧪 Testing

```bash
# All tests in workspace
cargo test --workspace

# Tests for specific crate
cargo test -p ccswarm
cargo test -p ai-session

# Specific module in ccswarm
cargo test -p ccswarm session
cargo test -p ccswarm identity
cargo test -p ccswarm quality_review  # New in v0.2.0

# Integration tests
cargo test -p ccswarm --test integration_tests

# Examples (relocated to demos/)
# See the demo applications in demos/ directory:
# - demos/todo-app/          - Complete TODO application
# - demos/multi-agent/       - Multi-agent monitoring demo
# - demos/session-persistence/ - Session recovery demo
# - demos/auto-create/       - Application generation demo

# Run ai-session library examples
cargo run -p ai-session --example basic_session
cargo run -p ai-session --example multi_agent
cargo run -p ai-session --example mcp_server

# Install ai-session CLI separately
cargo install --path crates/ai-session

# Use ai-session independently of ccswarm
ai-session create --name myproject --ai-context
ai-session exec myproject "npm test" --capture
```

## 🚨 Need Help?

### 📖 Comprehensive Documentation Available

We've created extensive documentation to help you succeed with ccswarm:

- **🚀 [Getting Started](docs/GETTING_STARTED.md)**: Complete beginner's guide with hands-on tutorials
- **⚙️ [Configuration](docs/CONFIGURATION.md)**: All configuration options explained with examples  
- **🔧 [Troubleshooting](docs/TROUBLESHOOTING.md)**: Detailed solutions for common issues
- **🤝 [Contributing](CONTRIBUTING.md)**: How to contribute to the project

### Quick Troubleshooting

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

**For more detailed solutions, see our [Troubleshooting Guide](docs/TROUBLESHOOTING.md)**

## 🏛️ Collective Intelligence & Self-Extension

### Sangha Democratic Decision-Making

ccswarm implements Buddhist Sangha principles for democratic agent decision-making:

```bash
# Submit proposals for swarm consideration
ccswarm sangha propose --type doctrine --title "Code Quality Standards"
ccswarm sangha propose --type extension --title "React Server Components"
ccswarm sangha propose --type task --title "Performance Optimization"

# Vote on active proposals
ccswarm sangha vote <proposal-id> aye --reason "Improves performance"
ccswarm sangha vote <proposal-id> nay --reason "Too complex for current sprint"

# View proposals and voting status
ccswarm sangha list --status active
ccswarm sangha show <proposal-id>
```

**Consensus Algorithms:**
- **Simple Majority** (51%+): Fast decisions for routine changes
- **Byzantine Fault Tolerant** (67%+): Critical system changes
- **Proof of Stake**: Weighted voting based on agent expertise

### Autonomous Self-Extension

Agents autonomously analyze their experiences and propose improvements:

```bash
# Autonomous self-extension - agents think independently
ccswarm extend autonomous                     # All agents self-reflect
ccswarm extend autonomous --agent backend     # Specific agent
ccswarm extend autonomous --dry-run           # Preview proposals
ccswarm extend autonomous --continuous        # Continuous improvement

# Optional search-based extension
ccswarm search mdn "react server components"
ccswarm search github "rust async patterns" 
ccswarm extend propose --title "Add RSC Support"

# View extension progress
ccswarm extend status
ccswarm extend stats
```

**Autonomous Process:**
1. **Experience Analysis**: Review past task performance
2. **Capability Assessment**: Identify strengths and gaps
3. **Strategic Planning**: Generate improvement proposals
4. **Sangha Consultation**: Submit for democratic approval
5. **Implementation**: Execute approved extensions

### Example: Live Demo Results

**Search Results (Real Data):**
```
🔍 GitHub Search Results for "react hooks":
• react-use (⭐ 43,170) - Essential React Hooks collection
• rehooks (⭐ 1,800) - Modern React Hooks library
• awesome-react-hooks (⭐ 9,200) - Curated list of hooks

📚 MDN Results for "web components":
• Custom Elements API - Create reusable components
• Shadow DOM API - Encapsulated component styling
• HTML Templates - Declarative component templates
```

**Sangha Proposal Generated:**
```
Proposal ID: e66349a2-d64c-4b68-8e0b-01fbfee4d515
Title: React Server Components Integration
Type: Extension
Status: Active (awaiting votes)
Description: Add RSC support based on community research
```

**Extension Proposal Created:**
```
Proposal ID: c52fe40e-96ae-46a7-8013-8de551f001a7
Agent: Frontend Specialist
Capability: React Server Components
Risk Level: Medium
Expected Impact: 30% faster page loads, 25% smaller bundles
```

## 🛠️ Development

### Working with the Workspace

ccswarm uses a Rust workspace structure for better organization:

```bash
# Build all crates in the workspace
cargo build --workspace

# Run tests for all crates
cargo test --workspace

# Build only ccswarm (main orchestration)
cargo build -p ccswarm

# Build only ai-session (terminal management)
cargo build -p ai-session

# Run ccswarm from workspace root
cargo run -p ccswarm -- init --name "MyProject"

# Run ai-session independently
cargo run -p ai-session -- create --name dev

# Start ai-session MCP server
cargo run -p ai-session --bin server -- --port 3000

# Generate documentation for entire workspace
cargo doc --workspace --no-deps --open
```

### Development Workflow

```bash
# 1. Make changes in the appropriate crate
cd crates/ccswarm  # or crates/ai-session

# 2. Run crate-specific tests
cargo test

# 3. Run workspace-wide tests
cd ../..
cargo test --workspace

# 4. Check formatting and linting
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

### Adding Custom Providers
1. Implement `ProviderExecutor` trait in `crates/ccswarm/src/providers/`
2. Add to `ProviderType` enum
3. Update configuration parsing
4. Add provider-specific configuration options

### v0.3.0 Architecture Improvements
- Sangha collective intelligence system
- Self-extension framework with search capabilities
- Meta-learning and pattern recognition
- Evolution tracking and metrics
- Safe extension propagation mechanisms

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
cd ccswarm

# Run all tests in workspace
cargo test --workspace

# Format all code
cargo fmt --all

# Run clippy on all crates
cargo clippy --workspace -- -D warnings

# Check documentation for entire workspace
cargo doc --workspace --no-deps --open

# Build release version
cargo build --release --workspace
```

## 💡 Enhanced User Experience

### Getting Started
```bash
# First time? Use the setup wizard
ccswarm setup

# Need help? Interactive tutorial  
ccswarm tutorial

# Check system health
ccswarm doctor

# Get contextual help
ccswarm help tasks
ccswarm help --search "delegation"
```

### Smart Error Messages
When things go wrong, ccswarm helps you fix them:

```
❌ Session not found
   No active session with ID: abc123

   💡 Try this:
   1. List all sessions: ccswarm session list
   2. Create a new session: ccswarm session create
   3. Check if the session was terminated

   Error code: SES001
```

### Real-time Progress Feedback
All operations show live progress:

```
⏳ Creating task: Implement user authentication... (2.3s)
✅ Task created successfully!

   Task ID: task-a1b2
   Description: Implement user authentication
   Priority: 🟡 High
   Type: Feature
   
💡 Quick tips:
  • View task progress: ccswarm task status task-a1b2
  • List all tasks: ccswarm task list
```

## 🚀 What's New in v0.3.5

ccswarm v0.3.5 introduces **autonomous orchestration** as the default mode:

- **🧠 Proactive Master Claude**: Enabled by default with 30s analysis intervals
- **🔒 Security Agent**: OWASP Top 10 scanning with real-time monitoring  
- **📊 Dependency Resolution**: Automatic task ordering and bottleneck detection
- **🎯 Goal Tracking**: OKR integration with milestone management
- **⚡ Native Sessions**: 93% token savings with cross-platform PTY support
- **🏛️ Collective Intelligence**: Sangha democratic decision-making
- **🤖 Self-Extension**: Autonomous agent improvement and learning

### Contributing

We welcome contributions! See our [Contributing Guide](CONTRIBUTING.md) for detailed instructions.

**Quick Start for Contributors:**

```bash
# Fork and clone
git clone https://github.com/yourusername/ccswarm.git
cd ccswarm

# Run all tests in workspace
cargo test --workspace

# Format all code
cargo fmt --all

# Run clippy on all crates
cargo clippy --workspace -- -D warnings

# Check documentation for entire workspace
cargo doc --workspace --no-deps --open

# Build release version
cargo build --release --workspace
```

**Documentation Contributions Welcome:** Help us improve our guides by contributing to:
- [Getting Started Guide](docs/GETTING_STARTED.md) - Add examples and tutorials
- [Configuration Reference](docs/CONFIGURATION.md) - Expand configuration examples  
- [Troubleshooting Guide](docs/TROUBLESHOOTING.md) - Add solutions for new issues

## 📄 License

MIT License - see [LICENSE](LICENSE)

## 🙏 Acknowledgments

- Anthropic for Claude and Claude Code
- Rust community for excellent libraries
- Contributors and early adopters

---

**Experience autonomous AI orchestration with proactive intelligence and security monitoring in ccswarm v0.3.5** 🧠🔒🚀