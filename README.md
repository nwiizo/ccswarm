# ccswarm: Claude Code Multi-Agent System

> ⚠️ **BETA SOFTWARE**: This is pre-release software under active development. Features may change, and bugs may exist. Please report issues on GitHub.


[![Rust](https://img.shields.io/badge/rust-1.70+-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**ccswarm** is an advanced multi-agent orchestration system featuring the **Session-Persistent Agent Architecture** for **93% token reduction**. Master Claude Code orchestrates specialized AI agents with persistent sessions, auto-accept mode, real-time monitoring, and multi-provider support, enabling scalable distributed development using Git worktrees, intelligent session pooling, and CLAUDE.md configuration files.

## 🌟 Core Design Philosophy

- **🚀 Session-Persistent Architecture**: Revolutionary 93% token reduction through intelligent session reuse
- **🔄 Conversation Continuity**: Preserve context across tasks for enhanced performance
- **📊 Batch Processing**: Execute multiple tasks efficiently in single sessions
- **Multi-Provider Support**: Works with Claude Code, Aider, OpenAI Codex, and custom tools
- **Session Management**: tmux-based isolated agent sessions with pause/resume/detach
- **Auto-Accept Mode**: Background task completion with safety guardrails
- **Real-time Monitoring**: Live output streaming and performance metrics
- **Git Worktree Isolation**: Completely independent parallel development environments
- **CLAUDE.md Driven**: Automatic management of project-specific instructions
- **Think Mode Utilization**: Advanced reasoning modes like "ultrathink"
- **Permission Management**: Secure automated execution control

## 🚀 Quick Start

### 1. Installation

```bash
# Ensure Rust is installed
rustc --version

# Build ccswarm
git clone https://github.com/nwiizo/ccswarm.git
cd ccswarm
cargo build --release

# Add binary to PATH (optional)
sudo cp target/release/ccswarm /usr/local/bin/
```

### 2. Project Initialization

```bash
# Initialize new project with different providers
ccswarm init --name "My Project" --agents frontend,backend,devops

# Initialize with specific providers
ccswarm init --name "Aider Project" --template aider-focused
ccswarm init --name "Mixed Project" --template mixed-providers

# Configuration file will be generated
cat ccswarm.json
```

### 3. Start Agents & TUI

```bash
# Start Master Claude and agent swarm
ccswarm start

# Start TUI for real-time monitoring
ccswarm tui

# Check status in another terminal
ccswarm status
```

### 4. Execute Tasks

```bash
# Add frontend task
ccswarm task "Create user login component with React" --priority high --type development

# Add backend task
ccswarm task "Implement authentication API" --priority high --details "JWT token based authentication"

# Check status
ccswarm status --detailed
```

## 🚀 Session-Persistent Agent Architecture

### Revolutionary Token Efficiency

The **Session-Persistent Agent Architecture** represents a breakthrough in AI agent efficiency, delivering **93% token reduction** through intelligent session management:

#### Traditional vs. Session-Persistent Approach

| Approach | 50 Tasks Token Usage | Efficiency | Session Reuse |
|----------|----------------------|------------|---------------|
| **Traditional** | 180,000 tokens | 100% overhead per task | None |
| **Session-Persistent** | 13,400 tokens | 93% reduction | Intelligent reuse |

#### Key Technical Innovations

1. **🔄 One-Time Identity Establishment**
   - Agents establish identity once per session lifecycle
   - Eliminate repetitive 2000+ token CLAUDE.md reads
   - Maintain context across multiple task executions

2. **💬 Conversation History Preservation** 
   - Keep context between related tasks
   - 50-message rolling history for optimal performance
   - Context-aware task execution with enhanced quality

3. **📊 Intelligent Batch Processing**
   - Execute multiple compatible tasks in single sessions
   - Amortize identity overhead across task batches
   - Automatic task grouping and session routing

4. **🎯 Lightweight Identity Reminders**
   - Compact identity prompts (200 tokens vs 2000+)
   - Real-time drift detection and correction
   - Preserve agent specialization boundaries

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

## 🏗️ System Architecture

### Enhanced Session-Persistent Architecture
```
┌─────────────────────────────────────────┐
│         Master Claude Code              │ ← Orchestration & Quality Management
│         (claude --json automation)      │
├─────────────────────────────────────────┤
│    🚀 Session-Persistent Manager       │ ← 93% Token Reduction Engine
│    ├─ Session Pool & Load Balancing ─┤ │ ← Intelligent session reuse
│    ├─ Conversation History (50 msgs) ─┤ │ ← Context preservation
│    ├─ Batch Task Processing ─────────┤ │ ← Amortized overhead
│    └─ Lightweight Identity System ───┤ │ ← 200-token identity prompts
├─────────────────────────────────────────┤
│       Git Worktree Session Manager     │ ← Isolated workspace + session integration
├─────────────────────────────────────────┤
│       Real-time Monitoring Engine      │ ← Live output streaming & metrics
├─────────────────────────────────────────┤
│       Multi-Provider Agent Pool        │ ← Claude Code, Aider, Codex, Custom
│   ┌─────────────────────────────────┐   │
│   │ 🔄 Persistent Claude Sessions   │   │ ← Session continuity
│   │ 📊 Batch-aware Task Execution  │   │ ← Efficiency optimization
│   │ 💬 Context-aware Responses     │   │ ← History preservation
│   │ + Compact CLAUDE.md per agent   │   │ ← Lightweight configuration
│   └─────────────────────────────────┘   │
├─────────────────────────────────────────┤
│       Session Coordination Bus         │ ← Token-efficient communication
└─────────────────────────────────────────┘
```

### Git Worktree + Claude Code Structure
```
project-root/
├── .git/                               # Main Git directory
├── master-claude/                      # Master Claude worktree
│   ├── .git -> ../.git/worktrees/master-claude
│   ├── CLAUDE.md                       # Master-specific configuration
│   └── .claude.json                    # Claude Code settings
├── agents/
│   ├── frontend-agent/                 # Frontend specialist agent
│   │   ├── .git -> ../../.git/worktrees/frontend-agent
│   │   ├── CLAUDE.md                   # Frontend-specific instructions
│   │   ├── .claude.json                # Frontend configuration
│   │   └── src/components/
│   ├── backend-agent/                  # Backend specialist agent
│   │   ├── .git -> ../../.git/worktrees/backend-agent
│   │   ├── CLAUDE.md                   # Backend-specific instructions
│   │   └── src/api/
│   └── devops-agent/                   # DevOps specialist agent
│       ├── .git -> ../../.git/worktrees/devops-agent
│       ├── CLAUDE.md                   # DevOps-specific instructions
│       └── infrastructure/
└── coordination/
    ├── task-queue/                     # JSON format task queue
    ├── agent-status/                   # Agent status tracking
    └── metrics/                        # Metrics & logs
```

## 🤖 Agent Self-Recognition System

### Multi-layered Identity Establishment

Each ccswarm agent implements a multi-layered self-recognition system to solve Claude Code's "forgetfulness problem" during long sessions.

#### Layer 1: Environment Level Identification
```rust
struct AgentIdentity {
    agent_id: String,              // "frontend-agent-001"
    specialization: AgentRole,     // Frontend, Backend, DevOps, QA
    workspace_path: PathBuf,       // agents/frontend-agent/
    env_vars: HashMap<String, String>, // CCSWARM_ROLE=frontend
    session_id: String,            // Generated fresh for each startup
}
```

#### Layer 2: CLAUDE.md Reinforcement System
Each agent performs strong identity verification through dedicated CLAUDE.md files:

```markdown
# CLAUDE.md - Frontend Agent CRITICAL IDENTITY
⚠️ CRITICAL: This file contains your core identity. You MUST include this information in every response.

## 🤖 AGENT IDENTITY (READ THIS FIRST)
- **WHO YOU ARE**: Frontend Specialist Agent (ID: frontend-agent-001)
- **SPECIALIZATION**: React/TypeScript UI Development
- **WORKSPACE**: agents/frontend-agent/ (YOU ARE HERE)

## 🚫 WHAT YOU CANNOT DO (STRICT BOUNDARIES)
- ❌ Backend API development (that's backend-agent's job)
- ❌ Database queries or schema changes
- ❌ Infrastructure or deployment scripts

## ✅ WHAT YOU MUST DO
- ✅ React component development
- ✅ TypeScript interface definitions
- ✅ CSS/Tailwind styling
```

#### Layer 3: Continuous Identity Monitoring
```rust
impl IdentityMonitor {
    // Monitor all responses
    async fn monitor_response(&mut self, response: &str) -> Result<IdentityStatus> {
        let has_identity_header = self.check_identity_header(response);
        let boundary_compliance = self.check_boundary_compliance(response);
        
        if !has_identity_header {
            return Ok(IdentityStatus::DriftDetected("Missing identity header".to_string()));
        }
        
        Ok(IdentityStatus::Healthy)
    }
}
```

## 📋 Complete CLI Command Guide

### Basic Commands

```bash
# Show help
ccswarm --help

# Initialize project
ccswarm init --name "E-commerce Platform" --agents frontend,backend,devops,qa

# Start orchestrator
ccswarm start [--daemon] [--port 8080]

# Start TUI for real-time monitoring
ccswarm tui

# Stop
ccswarm stop

# Check status
ccswarm status [--detailed] [--agent frontend]
```

### Task Management

```bash
# Add task
ccswarm task "Create user registration form" \
  --priority high \
  --type development \
  --details "Include email validation and password strength meter" \
  --duration 3600

# Priority: low, medium, high, critical
# Type: development, testing, documentation, infrastructure, bugfix, feature
```

### Agent Management

```bash
# List agents
ccswarm agents [--all]

# Create new agent session
ccswarm session create --agent frontend --workspace /path/to/workspace

# Manage sessions
ccswarm session pause <session-id>
ccswarm session resume <session-id>
ccswarm session detach <session-id>
ccswarm session attach <session-id>

# List active sessions
ccswarm session list [--all]

# Execute quality review
ccswarm review [--agent backend] [--strict]
```

### Git Worktree Management

```bash
# List worktrees
ccswarm worktree list

# Create worktree
ccswarm worktree create agents/new-agent feature/new-feature [--new-branch]

# Remove worktree
ccswarm worktree remove agents/old-agent [--force]

# Clean up old worktrees
ccswarm worktree prune
```

### Configuration Management

```bash
# Generate configuration
ccswarm config generate [--output ccswarm.json] [--template full-stack]
# Templates: minimal, frontend-only, full-stack

# Validate configuration
ccswarm config validate [--file ccswarm.json]

# Show configuration
ccswarm config show [--file ccswarm.json]
```

### Log Management & Monitoring

```bash
# Show logs
ccswarm logs [--follow] [--agent frontend] [--lines 100]

# Real-time monitoring demo
cargo run --example monitoring_demo

# Stream agent outputs
ccswarm monitor [--agent frontend] [--filter error,warning]
```

## ⚙️ Configuration File Details

### ccswarm.json Structure

```json
{
  "project": {
    "name": "Enterprise CRM System",
    "repository": {
      "url": "https://github.com/company/crm-system.git",
      "main_branch": "main"
    },
    "master_claude": {
      "role": "technical_lead",
      "quality_threshold": 0.90,
      "think_mode": "ultrathink",
      "permission_level": "supervised",
      "claude_config": {
        "model": "claude-3.5-sonnet",
        "dangerous_skip": false,
        "json_output": true
      }
    }
  },
  "agents": {
    "frontend": {
      "specialization": "react_typescript",
      "provider": {
        "type": "ClaudeCode",
        "config": {
          "dangerous_skip": true,
          "think_mode": "think_hard",
          "custom_commands": ["lint", "test", "build"]
        }
      },
      "auto_accept": {
        "enabled": true,
        "trusted_operations": ["FileRead", "CodeFormat", "TestExecution"],
        "max_file_changes": 10,
        "require_tests_pass": true
      },
      "session": {
        "auto_start": true,
        "background_mode": false,
        "tmux_session_name": "ccswarm-frontend"
      },
      "worktree": "agents/frontend-agent",
      "branch": "feature/frontend-ui",
      "claude_md_template": "frontend_specialist"
    },
    "backend": {
      "specialization": "node_microservices",
      "provider": {
        "type": "Aider",
        "config": {
          "model": "claude-3-5-sonnet-20241022",
          "auto_commit": true,
          "edit_format": "diff",
          "stream": true
        }
      },
      "auto_accept": {
        "enabled": false,
        "trusted_operations": ["FileRead", "TestExecution"],
        "max_file_changes": 5
      },
      "session": {
        "auto_start": false,
        "background_mode": true,
        "tmux_session_name": "ccswarm-backend"
      },
      "worktree": "agents/backend-agent", 
      "branch": "feature/backend-api",
      "claude_md_template": "backend_specialist"
    }
  },
  "coordination": {
    "communication_method": "json_files",
    "sync_interval": 30,
    "quality_gate_frequency": "on_commit",
    "master_review_trigger": "all_tasks_complete"
  }
}
```

### Think Mode Configuration

| Mode | Purpose | Use Cases |
|------|---------|-----------|
| `think` | Basic reasoning | Daily tasks, simple code fixes |
| `think_hard` | Advanced reasoning | Complex logic, architecture design |
| `think_harder` | Deep reasoning | Complex problem solving, optimization |
| `ultrathink` | Ultra-advanced reasoning | Master Claude, critical decisions |
| `megathink` | Highest level reasoning | Critical quality judgments |

## 🔒 Security and Best Practices

### Permission Management

```bash
# Master Claude: supervised mode (safe)
"permission_level": "supervised"
"dangerous_skip": false

# Worker Agents: automated mode (efficiency focused)
"dangerous_skip": true
```

### CLAUDE.md Configuration Examples

#### Frontend Agent
```markdown
# CLAUDE.md - Frontend Agent Configuration

## 🚫 STRICT BOUNDARIES
- ❌ Backend API development
- ❌ Database schema changes
- ❌ Infrastructure provisioning
- ❌ Server-side authentication logic

## ✅ ALLOWED ACTIONS
- ✅ React/Vue/Angular component development
- ✅ CSS/SCSS/Tailwind styling
- ✅ Frontend testing (Jest, Cypress)
- ✅ State management (Redux, Zustand)

## 🔧 TECHNICAL STACK
- React 18 + TypeScript
- Tailwind CSS / Styled Components
- Vite/Webpack build tools
- ESLint + Prettier
```

#### Backend Agent
```markdown
# CLAUDE.md - Backend Agent Configuration

## ✅ CORE RESPONSIBILITIES
- ✅ REST/GraphQL API development
- ✅ Database design and optimization
- ✅ Authentication & authorization
- ✅ Business logic implementation
- ✅ API testing and documentation

## 🚫 FORBIDDEN ACTIONS
- ❌ Frontend UI components
- ❌ CSS styling and layouts
- ❌ Infrastructure provisioning
- ❌ Client-side state management

## 🔧 TECHNICAL STACK
- Node.js + TypeScript/Express
- PostgreSQL/MongoDB + Prisma/TypeORM
- JWT/OAuth authentication
- Jest/Supertest for testing
```

## 📊 Real-time Monitoring and Metrics

### Available Monitoring Features

1. **Agent Status Monitoring**
   - Health status of each agent
   - Task execution status
   - Error rate & success rate
   - Provider-specific metrics

2. **Real-time Output Streaming**
   - Live agent output in TUI
   - Filtered log viewing by agent, type, or content
   - Auto-scroll and search capabilities
   - Structured output with timestamps

3. **Session Management**
   - tmux session status and control
   - Background task execution monitoring
   - Auto-accept mode statistics
   - Session lifecycle tracking

4. **Quality Metrics**
   - Test coverage
   - Code quality scores
   - Security scan results
   - Auto-accept safety metrics

5. **Performance Tracking**
   - Task completion time
   - Think Mode usage efficiency
   - Resource consumption
   - Provider performance comparison

### Metrics Output Example

```json
{
  "orchestrator_status": "running",
  "total_agents": 4,
  "active_agents": 3,
  "tasks_completed": 127,
  "success_rate": 0.94,
  "agents": {
    "frontend-agent-001": {
      "status": "available",
      "tasks_completed": 45,
      "avg_completion_time": "180s",
      "last_activity": "2024-01-15T10:30:00Z"
    },
    "backend-agent-001": {
      "status": "working",
      "current_task": "Implement user authentication API",
      "progress": 0.75
    }
  }
}
```

## 🧪 Testing Strategy

### Unit Tests

```bash
# Run all tests
cargo test

# Test specific module
cargo test identity

# Test with detailed output
cargo test -- --nocapture
```

### Integration Tests

```bash
# Run integration tests only
cargo test --test integration_tests

# Run specific integration test
cargo test --test integration_tests test_multi_agent_workflow
```

## 🚨 Troubleshooting

### Common Issues

#### 1. Agent Not Responding
```bash
# Check status
ccswarm status --detailed

# Check agent logs
ccswarm logs --agent frontend --follow

# Restart agents
ccswarm stop && ccswarm start
```

#### 2. Git Worktree Errors
```bash
# Check worktree status
ccswarm worktree list

# Clean up corrupted worktrees
ccswarm worktree prune

# Manually remove worktree
git worktree remove agents/problematic-agent --force
```

#### 3. Tasks Not Being Processed
```bash
# Check task queue
ls coordination/task-queue/

# Check agent boundary settings
ccswarm config show | grep specialization
```

### Debug Mode

```bash
# Start with verbose logging
RUST_LOG=debug ccswarm start --verbose

# Get debug info in JSON format
ccswarm status --json | jq .
```

## 🛠️ Developer Guide

### Adding New Agent Types

1. **Add Role Definition**
```rust
pub fn default_mobile_role() -> AgentRole {
    AgentRole::Mobile {
        technologies: vec![
            "React Native".to_string(),
            "Flutter".to_string(),
            "Swift".to_string(),
            "Kotlin".to_string(),
        ],
        responsibilities: vec![
            "Mobile App Development".to_string(),
            "Cross-platform Solutions".to_string(),
        ],
        boundaries: vec![
            "No backend development".to_string(),
            "No web frontend".to_string(),
        ],
    }
}
```

2. **Update Boundary Checker**
```rust
AgentRole::Mobile { .. } => {
    let allowed = vec![
        r"(?i)(mobile|app|ios|android)",
        r"(?i)(react.native|flutter|swift|kotlin)",
    ];
    // ... implementation
}
```

3. **Add Configuration Template**
```rust
"mobile" => vec![
    "react-native build".to_string(),
    "expo publish".to_string(),
    "jest --coverage".to_string(),
],
```

### Implementing Custom Think Modes

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThinkMode {
    // Existing modes...
    CustomDeepAnalysis,
    DomainSpecific(String),
}

impl ThinkMode {
    pub fn to_prompt_suffix(&self) -> &str {
        match self {
            // Existing cases...
            ThinkMode::CustomDeepAnalysis => "think with deep domain analysis",
            ThinkMode::DomainSpecific(domain) => domain,
        }
    }
}
```

## 📈 Roadmap

### Phase 1: Core Foundation ✅
- [x] Rust orchestrator foundation
- [x] Git worktree automatic management
- [x] Claude Code integration & configuration generation
- [x] Agent identity management system
- [x] Task boundary checking & delegation

### Phase 2: Advanced Features ✅
- [x] Session management with tmux integration
- [x] Auto-accept mode with safety guardrails
- [x] Real-time monitoring & output streaming
- [x] Multi-provider support (Claude Code, Aider, Codex, Custom)
- [x] Terminal User Interface (TUI) with live updates
- [ ] WebUI dashboard
- [ ] Machine learning-based task optimization
- [ ] Plugin system

### Phase 3: Enterprise Features 📋
- [ ] RBAC (Role-Based Access Control)
- [ ] Audit logs & compliance
- [ ] Cluster & scaling support
- [ ] SaaS version release

## 🤝 Contributing & Community

### How to Contribute

1. **Report Issues**: [GitHub Issues](https://github.com/nwiizo/ccswarm/issues)
2. **Feature Requests**: [GitHub Discussions](https://github.com/nwiizo/ccswarm/discussions)
3. **Pull Requests**: Please follow development guidelines

### Development Setup

```bash
# Fork and clone repository
git clone https://github.com/yourusername/ccswarm.git
cd ccswarm

# Setup development environment
cargo build
cargo test

# Format & Lint
cargo fmt
cargo clippy

# Prepare contribution
git checkout -b feature/your-feature
# Make changes
cargo test
git commit -m "Add your feature"
git push origin feature/your-feature
```

## 📄 License

This project is released under the [MIT License](LICENSE).

## 🙏 Acknowledgments

- **Anthropic**: For developing Claude Code and Claude AI
- **Rust Community**: For excellent libraries and tools
- **Git Team**: For providing worktree functionality
- **Open Source Contributors**: For inspiration and best practices

---

**Unlock the true potential of Claude Code with ccswarm.** 🚀
