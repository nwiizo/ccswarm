# 🚀 ccswarm Enhanced Usage Guide

This guide explains how to use ccswarm, an advanced multi-agent orchestration system with session management, auto-accept mode, real-time monitoring, and multi-provider support.

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Project Initialization](#project-initialization)
- [Agent Management](#agent-management)
- [Session Management](#session-management) **NEW**
- [Task Management](#task-management)
- [Auto-Accept Mode](#auto-accept-mode) **NEW**
- [Real-time Monitoring](#real-time-monitoring) **NEW**
- [Multi-Provider Support](#multi-provider-support) **NEW**
- [TUI (Terminal User Interface)](#tui-terminal-user-interface)
- [Git Worktree Management](#git-worktree-management)
- [Configuration](#configuration)
- [Common Workflows](#common-workflows)
- [Examples](#examples)

## 🏁 Quick Start

### 1. Initialize a new project
```bash
# Create a full-stack project with all agent types
ccswarm -- init --name "MyProject" --agents frontend,backend,devops

# Initialize with specific providers
ccswarm -- init --name "AiderProject" --template aider-focused
ccswarm -- init --name "MixedProject" --template mixed-providers

# Minimal setup
ccswarm -- init --name "SimpleApp" --agents frontend
```

### 2. Start the orchestrator
```bash
# Start in interactive mode
ccswarm -- start

# Start with enhanced TUI for real-time monitoring
ccswarm -- tui

# Start with monitoring demo
ccswarm --example monitoring_demo
```

### 3. Add tasks and manage sessions
```bash
# Add a development task
ccswarm -- task "Create user login component" --priority high --type development

# Create a session for frontend work
ccswarm -- session create --agent frontend --workspace ./frontend

# Add a testing task
ccswarm -- task "Write unit tests for auth" --priority medium --type testing
```

## 🎯 Project Initialization

### Basic Initialization
```bash
# Initialize with default configuration
ccswarm -- init --name "TodoApp"

# Initialize with specific agents
ccswarm -- init --name "ECommerceApp" --agents frontend,backend,devops,qa

# Initialize with repository URL
ccswarm -- init --name "MyAPI" --repo-url "https://github.com/user/myapi.git" --agents backend
```

### Project Templates
```bash
# Generate different configuration templates
ccswarm -- config generate --template minimal
ccswarm -- config generate --template frontend-only
ccswarm -- config generate --template full-stack
```

## 🤖 Agent Management

### List Agents
```bash
# Show configured agents
ccswarm -- agents

# Show all agents (including inactive)
ccswarm -- agents --all

# JSON output for programmatic use
ccswarm -- agents --json
```

### Agent Configuration with Providers
Agents are configured in `ccswarm.json` with provider support:
```json
{
  "agents": {
    "frontend": {
      "specialization": "react_typescript",
      "provider": {
        "type": "ClaudeCode",
        "config": {
          "think_mode": "Think",
          "permission_level": "supervised"
        }
      },
      "session": {
        "auto_start": true,
        "background_mode": false,
        "tmux_session_name": "ccswarm-frontend"
      },
      "auto_accept": {
        "enabled": true,
        "trusted_operations": ["FileRead", "CodeFormat"],
        "max_file_changes": 10
      },
      "worktree": "agents/frontend-agent",
      "branch": "feature/frontend-ui"
    }
  }
}
```

## 🔄 Session Management

### Basic Session Commands
```bash
# Create a new session
ccswarm -- session create --agent frontend --workspace /path/to/workspace

# List all sessions
ccswarm -- session list

# List only active sessions
ccswarm -- session list --active

# Show session details
ccswarm -- session show <session-id>
```

### Session Control
```bash
# Pause a session
ccswarm -- session pause <session-id>

# Resume a paused session
ccswarm -- session resume <session-id>

# Detach from a session (keeps running in background)
ccswarm -- session detach <session-id>

# Attach to a detached session
ccswarm -- session attach <session-id>

# Terminate a session
ccswarm -- session terminate <session-id>
```

### Background Mode
```bash
# Enable background mode for a session
ccswarm -- session background <session-id> --enable

# Disable background mode
ccswarm -- session background <session-id> --disable

# Check background sessions
ccswarm -- session list --background
```

## 📝 Task Management

### Adding Tasks
```bash
# Basic task
ccswarm -- task "Fix login bug"

# Task with priority and type
ccswarm -- task "Implement user registration" --priority high --type feature

# Task with details and duration estimate
ccswarm -- task "Add API documentation" \
  --priority medium \
  --type documentation \
  --details "Document all REST endpoints" \
  --duration 3600
```

### Task Types
- `development` / `dev` - General development work
- `testing` / `test` - Testing and QA tasks
- `documentation` / `docs` - Documentation tasks
- `infrastructure` / `infra` - DevOps and infrastructure
- `bugfix` / `bug` - Bug fixes
- `feature` - New feature development
- `review` - Code review tasks
- `coordination` - Inter-agent coordination

### Auto-Accept Integration
```bash
# Add task with auto-accept hint
ccswarm -- task "Format code" --auto-accept-safe

# Add task that requires manual approval
ccswarm -- task "Delete database" --no-auto-accept

# Check auto-accept status for tasks
ccswarm -- task list --show-auto-accept
```

### Task Priorities
- `low` - Low priority tasks
- `medium` - Medium priority (default)
- `high` - High priority tasks
- `critical` - Critical/urgent tasks

## 🖥️ TUI (Terminal User Interface)

### Starting TUI
```bash
ccswarm -- tui
```

### TUI Navigation
- **Tab/Shift+Tab**: Switch between tabs (Overview, Agents, Tasks, Logs)
- **↑/↓ or j/k**: Navigate through lists
- **Enter**: View details of selected item
- **q**: Quit TUI
- **r**: Refresh data

### Enhanced TUI Commands
Press **'c'** to open command prompt:

```bash
# Task management
task Create login form [high] [feature]
task Fix API bug [critical] [bugfix]
task Write tests [medium] [test]

# Session management (NEW)
session create frontend /path/to/workspace
session list
session pause <session-id>
session resume <session-id>
session attach <session-id>

# Monitoring and filtering (NEW)
monitor frontend
filter error
nofilter
autoscroll on

# Agent management
agent frontend
agent backend
agent devops

# System commands
status
start
stop
refresh
clear
help

# Worktree management
worktree list
worktree prune
```

### Real-time Logs Tab
- **Live output streaming** from all agents
- **Filter by agent**: Click agent name or use `filter <agent>`
- **Filter by type**: `filter error`, `filter warning`, `filter success`
- **Search content**: `filter "search term"`
- **Auto-scroll control**: Toggle with `autoscroll on/off`
- **Clear logs**: Use `clear` command

## 🤖 Auto-Accept Mode

### Configuration
```bash
# Enable auto-accept for safe operations
ccswarm -- config set auto-accept.enabled true

# Set trusted operations
ccswarm -- config set auto-accept.trusted-operations "FileRead,CodeFormat,TestExecution"

# Set safety limits
ccswarm -- config set auto-accept.max-file-changes 10
```

### Safety Features
```bash
# Check auto-accept status
ccswarm -- auto-accept status

# Emergency stop (disables all auto-accept)
ccswarm -- auto-accept emergency-stop

# Reset emergency stop
ccswarm -- auto-accept reset

# View auto-accept history
ccswarm -- auto-accept history --limit 50
```

### Operation Types and Risk Levels
- **FileRead** (Risk: 1) - Reading files
- **CodeFormat** (Risk: 2) - Code formatting
- **TestExecution** (Risk: 3) - Running tests
- **FileWrite** (Risk: 5) - Writing files
- **GitCommit** (Risk: 6) - Git operations
- **FileDelete** (Risk: 9) - Deleting files
- **DatabaseOperation** (Risk: 10) - Database changes

## 📊 Real-time Monitoring

### Monitoring Commands
```bash
# Start real-time monitoring
ccswarm -- monitor

# Monitor specific agent
ccswarm -- monitor --agent frontend

# Monitor with filters
ccswarm -- monitor --filter "error,warning"

# Monitor with output to file
ccswarm -- monitor --output monitoring.log
```

### Monitoring Demo
```bash
# Run comprehensive monitoring demonstration
ccswarm --example monitoring_demo

# Monitor with multiple agents
ccswarm --example monitoring_demo --agents 4

# High-load monitoring test
ccswarm --example monitoring_demo --messages 15000
```

### Output Types
- **Info** - General information
- **Success** - Successful operations
- **Warning** - Warning messages
- **Error** - Error messages
- **Debug** - Debug information

## 🔄 Multi-Provider Support

### Available Providers
```bash
# Claude Code (default)
ccswarm -- provider set claude-code --think-mode "think_hard"

# Aider integration
ccswarm -- provider set aider --model "claude-3-5-sonnet" --auto-commit

# OpenAI Codex
ccswarm -- provider set codex --api-key "your-key" --model "gpt-4"

# Custom tool
ccswarm -- provider set custom --command "my-ai-tool" --args "--interactive"
```

### Provider Configuration
```bash
# List available providers
ccswarm -- provider list

# Show provider capabilities
ccswarm -- provider info aider

# Test provider health
ccswarm -- provider health-check --all

# Switch provider for specific agent
ccswarm -- agent configure frontend --provider aider
```

### Provider Examples
Check the examples directory for provider-specific configurations:
- `examples/ccswarm-aider-focused.json`
- `examples/ccswarm-mixed-providers.json`
- `examples/ccswarm-openai-codex.json`
- `examples/ccswarm-custom-tools.json`

## 🌳 Git Worktree Management

### List Worktrees
```bash
ccswarm -- worktree list

# Show worktree details
ccswarm -- worktree list --detailed

# JSON output
ccswarm -- worktree list --json
```

### Create Worktrees
```bash
# Create worktree for existing branch
ccswarm -- worktree create agents/feature-agent feature/new-feature

# Create worktree with new branch
ccswarm -- worktree create agents/test-agent feature/testing --new-branch

# Create with session integration
ccswarm -- worktree create agents/session-agent feature/session --with-session
```

### Remove and Clean
```bash
# Remove specific worktree
ccswarm -- worktree remove agents/old-agent

# Force removal
ccswarm -- worktree remove agents/broken-agent --force

# Clean up stale worktrees
ccswarm -- worktree prune

# Clean with session cleanup
ccswarm -- worktree prune --cleanup-sessions
```

## ⚙️ Enhanced Configuration

### Generate Configuration
```bash
# Full-stack configuration with session management
ccswarm -- config generate --template full-stack --output ccswarm.json

# Aider-focused configuration
ccswarm -- config generate --template aider-focused --output aider.json

# Mixed providers configuration
ccswarm -- config generate --template mixed-providers --output mixed.json

# OpenAI Codex configuration
ccswarm -- config generate --template openai-codex --output codex.json

# Custom tools configuration
ccswarm -- config generate --template custom-tools --output custom.json

# Frontend-only configuration
ccswarm -- config generate --template frontend-only --output frontend.json

# Minimal configuration
ccswarm -- config generate --template minimal --output minimal.json
```

### Configuration with Session Management
```json
{
  "project": {
    "name": "Enhanced Project",
    "session_persistent": true,
    "token_optimization": {
      "enabled": true,
      "target_reduction": 0.93,
      "conversation_history_limit": 50
    }
  },
  "agents": {
    "frontend": {
      "provider": {
        "type": "ClaudeCode",
        "config": {
          "think_mode": "think_hard",
          "session_persistent": true
        }
      },
      "session": {
        "auto_start": true,
        "background_mode": false,
        "reuse_threshold": 0.8
      },
      "auto_accept": {
        "enabled": true,
        "trusted_operations": ["FileRead", "CodeFormat"],
        "max_file_changes": 10
      }
    }
  }
}
```

### Validate Configuration
```bash
# Validate current configuration
ccswarm -- config validate

# Validate specific file
ccswarm -- config validate --file my-config.json

# Validate with provider checks
ccswarm -- config validate --check-providers

# Validate session configuration
ccswarm -- config validate --check-sessions
```

### Show Configuration
```bash
# Show current configuration
ccswarm -- config show

# Show specific configuration file
ccswarm -- config show --file ccswarm.json

# Show with provider details
ccswarm -- config show --include-providers

# Show session configuration
ccswarm -- config show --sessions
```

## 🔄 Enhanced Common Workflows

### 1. Session-Persistent Full-Stack Development
```bash
# 1. Initialize project with session management
ccswarm -- init --name "WebApp" --agents frontend,backend,devops --session-persistent

# 2. Start enhanced TUI for real-time monitoring
ccswarm -- tui

# 3. Create sessions for each agent
session create frontend ./frontend --auto-accept
session create backend ./backend --background
session create devops ./devops

# 4. Add initial tasks (in TUI command mode)
task Create React app structure [high] [feature]
task Setup Express.js API [high] [feature]
task Configure CI/CD pipeline [medium] [infra]

# 5. Start orchestrator with monitoring
ccswarm -- start --with-monitoring
```

### 2. Enhanced Bug Fix Workflow with Auto-Accept
```bash
# 1. Add bug fix task with auto-accept for safe operations
ccswarm -- task "Fix user authentication issue" --priority critical --type bugfix --auto-accept-safe

# 2. Check agent and session status
ccswarm -- status --detailed --include-sessions

# 3. Monitor progress with real-time output
ccswarm -- tui
# OR
ccswarm -- monitor --agent backend --filter "error,warning"

# 4. Use session for quick fixes
ccswarm -- session attach backend-session-001
```

### 3. Enhanced Feature Development with Multi-Provider
```bash
# 1. Create feature branch worktree with session integration
ccswarm -- worktree create agents/feature-agent feature/user-profiles --new-branch --with-session

# 2. Configure mixed providers for different tasks
ccswarm -- agent configure frontend --provider claude-code
ccswarm -- agent configure backend --provider aider --auto-commit
ccswarm -- agent configure qa --provider custom --command "custom-test-tool"

# 3. Add feature tasks with appropriate auto-accept settings
ccswarm -- task "Design user profile UI" --priority high --type development --auto-accept-safe
ccswarm -- task "Implement profile API" --priority high --type development
ccswarm -- task "Add profile tests" --priority medium --type testing --auto-accept-safe

# 4. Start development with monitoring
ccswarm -- start --with-monitoring
ccswarm --example monitoring_demo  # In separate terminal
```

## 📋 Examples

### Example 1: E-commerce Project
```bash
# Initialize e-commerce project
ccswarm -- init --name "ECommerceShop" --agents frontend,backend,devops

# Add initial tasks
ccswarm -- task "Create product catalog UI" --priority high --type feature
ccswarm -- task "Implement shopping cart API" --priority high --type feature
ccswarm -- task "Setup payment integration" --priority medium --type feature
ccswarm -- task "Configure production deployment" --priority medium --type infra

# Start with TUI monitoring
ccswarm -- tui
```

### Example 2: API-Only Project
```bash
# Initialize API project
ccswarm -- init --name "UserAPI" --agents backend,qa

# Configure for API development
ccswarm -- config generate --template minimal --output api-config.json

# Add API development tasks
ccswarm -- task "Design REST API endpoints" --priority high --type development
ccswarm -- task "Implement user CRUD operations" --priority high --type development
ccswarm -- task "Add API documentation" --priority medium --type documentation
ccswarm -- task "Write integration tests" --priority high --type testing

# Start development
ccswarm -- start
```

### Example 3: Frontend-Only Project
```bash
# Initialize frontend project
ccswarm -- init --name "ReactDashboard" --agents frontend

# Add frontend tasks
ccswarm -- task "Create dashboard layout" --priority high --type development
ccswarm -- task "Implement data visualization" --priority medium --type feature
ccswarm -- task "Add responsive design" --priority medium --type development
ccswarm -- task "Write component tests" --priority medium --type testing

# Monitor with TUI
ccswarm -- tui
```

## 🔍 Monitoring and Debugging

### System Status
```bash
# Quick status check
ccswarm -- status

# Detailed status with agent information
ccswarm -- status --detailed

# Check specific agent
ccswarm -- status --agent frontend
```

### Logs
```bash
# View logs
ccswarm -- logs

# Follow logs in real-time
ccswarm -- logs --follow

# View logs for specific agent
ccswarm -- logs --agent backend --lines 50
```

### JSON Output
Add `--json` flag to any command for machine-readable output:
```bash
ccswarm -- agents --json
ccswarm -- status --json
ccswarm -- worktree list --json
```

## ⚡ Pro Tips

1. **Use TUI for real-time monitoring**: `ccswarm -- tui` provides the best overview
2. **Smart task creation in TUI**: Use `[high]`, `[test]`, `[docs]` shortcuts in task descriptions
3. **Worktree isolation**: Each agent works in its own Git worktree for parallel development
4. **Configuration templates**: Start with templates and customize as needed
5. **JSON output**: Use `--json` flag for integration with other tools
6. **Batch operations**: Use TUI command mode for quick task creation

## 🆘 Troubleshooting

### Common Issues

**Configuration not found:**
```bash
# Generate default configuration
ccswarm -- config generate
```

**Worktree conflicts:**
```bash
# Clean up stale worktrees
ccswarm -- worktree prune
```

**Agent communication issues:**
```bash
# Check system status
ccswarm -- status --detailed

# Review logs
ccswarm -- logs --follow
```

## 🔍 Enhanced Monitoring and Debugging

### System Status with Sessions
```bash
# Quick status check with session information
ccswarm -- status --include-sessions

# Detailed status with session metrics
ccswarm -- status --detailed --session-metrics

# Check specific agent and its sessions
ccswarm -- status --agent frontend --show-sessions

# Check session efficiency metrics
ccswarm -- session efficiency --all
```

### Real-time Monitoring
```bash
# Start comprehensive monitoring
ccswarm -- monitor --all-agents

# Monitor with session-specific filtering
ccswarm -- monitor --session frontend-session-001

# Monitor auto-accept operations
ccswarm -- monitor --filter "auto-accept"

# Monitor token efficiency
ccswarm -- monitor --show-tokens
```

### Enhanced Logs
```bash
# View logs with session context
ccswarm -- logs --include-sessions

# Follow logs with real-time filtering
ccswarm -- logs --follow --filter "error,warning,success"

# View session-specific logs
ccswarm -- logs --session backend-session-001 --lines 100

# View auto-accept operation logs
ccswarm -- logs --auto-accept --follow
```

### JSON Output with Enhanced Data
Add `--json` flag to any command for machine-readable output:
```bash
ccswarm -- agents --json --include-sessions
ccswarm -- status --json --session-metrics
ccswarm -- session list --json --detailed
ccswarm -- monitor --json --agent frontend
```

## ⚡ Enhanced Pro Tips

1. **Maximize token efficiency**: Use session-persistent mode for 93% token reduction
2. **Smart session management**: Create sessions early and reuse them for related tasks
3. **Auto-accept optimization**: Enable auto-accept for safe operations like formatting and testing
4. **Real-time monitoring**: Use `ccswarm -- tui` with enhanced logs tab for live insights
5. **Provider selection**: Choose the right provider for each task type (Claude Code for complex logic, Aider for quick edits)
6. **Batch task execution**: Group related tasks for maximum session reuse efficiency
7. **Background sessions**: Use background mode for long-running tasks like DevOps operations
8. **Emergency controls**: Know the auto-accept emergency stop commands for safety
9. **Session metrics**: Regularly check `ccswarm -- session efficiency` to monitor performance
10. **Configuration templates**: Start with provider-specific templates for optimal setup
11. **Monitoring demos**: Run `ccswarm --example monitoring_demo` to understand system behavior
12. **JSON integration**: Use enhanced JSON output with session data for external tools

## 🎆 Performance Optimization

### Token Efficiency Tips
```bash
# Check current token usage
ccswarm -- session stats --show-tokens

# Optimize session reuse
ccswarm -- config set session.reuse_threshold 0.8

# Enable conversation history
ccswarm -- config set session.conversation_history 50

# Monitor token reduction
ccswarm -- monitor --show-token-efficiency
```

### Session Pool Optimization
```bash
# Configure session pooling
ccswarm -- config set session.pool.min_sessions 2
ccswarm -- config set session.pool.max_sessions 5
ccswarm -- config set session.pool.load_balancing "LeastLoaded"

# Enable auto-scaling
ccswarm -- config set session.pool.auto_scaling true
```

## 🎉 Quick Start Examples

### 5-Minute Token-Efficient Setup
```bash
# 1. Create project with session persistence
ccswarm -- init --name "QuickStart" --template mixed-providers

# 2. Start TUI
ccswarm -- tui

# 3. In TUI command mode ('c' key):
session create frontend ./frontend --auto-accept
task Create homepage [high] [feature]
task Add styling [medium] [development]

# 4. Watch 93% token reduction in action!
```

### Advanced Multi-Provider Setup
```bash
# 1. Initialize with custom configuration
ccswarm -- init --name "Advanced" --template custom-tools

# 2. Configure different providers
ccswarm -- agent configure frontend --provider claude-code --think-mode "think_hard"
ccswarm -- agent configure backend --provider aider --auto-commit
ccswarm -- agent configure qa --provider custom --command "test-suite"

# 3. Start with full monitoring
ccswarm -- start --with-monitoring
ccswarm --example monitoring_demo
```

For more help, check the [main documentation](README.md) or [architecture guide](CLAUDE.md).
