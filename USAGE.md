# 🚀 ccswarm Usage Guide

This guide explains how to use ccswarm, a multi-agent orchestration system for collaborative development.

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [Project Initialization](#project-initialization)
- [Agent Management](#agent-management)
- [Task Management](#task-management)
- [TUI (Terminal User Interface)](#tui-terminal-user-interface)
- [Git Worktree Management](#git-worktree-management)
- [Configuration](#configuration)
- [Common Workflows](#common-workflows)
- [Examples](#examples)

## 🏁 Quick Start

### 1. Initialize a new project
```bash
# Create a full-stack project with all agent types
cargo run -- init --name "MyProject" --agents frontend,backend,devops

# Minimal setup
cargo run -- init --name "SimpleApp" --agents frontend
```

### 2. Start the orchestrator
```bash
# Start in interactive mode
cargo run -- start

# Start with TUI for real-time monitoring
cargo run -- tui
```

### 3. Add tasks
```bash
# Add a development task
cargo run -- task "Create user login component" --priority high --type development

# Add a testing task
cargo run -- task "Write unit tests for auth" --priority medium --type testing
```

## 🎯 Project Initialization

### Basic Initialization
```bash
# Initialize with default configuration
cargo run -- init --name "TodoApp"

# Initialize with specific agents
cargo run -- init --name "ECommerceApp" --agents frontend,backend,devops,qa

# Initialize with repository URL
cargo run -- init --name "MyAPI" --repo-url "https://github.com/user/myapi.git" --agents backend
```

### Project Templates
```bash
# Generate different configuration templates
cargo run -- config generate --template minimal
cargo run -- config generate --template frontend-only
cargo run -- config generate --template full-stack
```

## 🤖 Agent Management

### List Agents
```bash
# Show configured agents
cargo run -- agents

# Show all agents (including inactive)
cargo run -- agents --all

# JSON output for programmatic use
cargo run -- agents --json
```

### Agent Configuration
Agents are configured in `ccswarm.json`:
```json
{
  "agents": {
    "frontend": {
      "specialization": "react_typescript",
      "worktree": "agents/frontend-agent",
      "branch": "feature/frontend-ui",
      "claude_config": {
        "think_mode": "Think",
        "permission_level": "supervised"
      }
    }
  }
}
```

## 📝 Task Management

### Adding Tasks
```bash
# Basic task
cargo run -- task "Fix login bug"

# Task with priority and type
cargo run -- task "Implement user registration" --priority high --type feature

# Task with details and duration estimate
cargo run -- task "Add API documentation" \
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

### Task Priorities
- `low` - Low priority tasks
- `medium` - Medium priority (default)
- `high` - High priority tasks
- `critical` - Critical/urgent tasks

## 🖥️ TUI (Terminal User Interface)

### Starting TUI
```bash
cargo run -- tui
```

### TUI Navigation
- **Tab/Shift+Tab**: Switch between tabs (Overview, Agents, Tasks, Logs)
- **↑/↓ or j/k**: Navigate through lists
- **Enter**: View details of selected item
- **q**: Quit TUI
- **r**: Refresh data

### TUI Commands
Press **'c'** to open command prompt:

```bash
# Task management
task Create login form [high] [feature]
task Fix API bug [critical] [bugfix]
task Write tests [medium] [test]

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

## 🌳 Git Worktree Management

### List Worktrees
```bash
cargo run -- worktree list
```

### Create Worktrees
```bash
# Create worktree for existing branch
cargo run -- worktree create agents/feature-agent feature/new-feature

# Create worktree with new branch
cargo run -- worktree create agents/test-agent feature/testing --new-branch
```

### Remove and Clean
```bash
# Remove specific worktree
cargo run -- worktree remove agents/old-agent

# Force removal
cargo run -- worktree remove agents/broken-agent --force

# Clean up stale worktrees
cargo run -- worktree prune
```

## ⚙️ Configuration

### Generate Configuration
```bash
# Full-stack configuration
cargo run -- config generate --template full-stack --output ccswarm.json

# Frontend-only configuration
cargo run -- config generate --template frontend-only --output frontend.json

# Minimal configuration
cargo run -- config generate --template minimal --output minimal.json
```

### Validate Configuration
```bash
# Validate current configuration
cargo run -- config validate

# Validate specific file
cargo run -- config validate --file my-config.json
```

### Show Configuration
```bash
# Show current configuration
cargo run -- config show

# Show specific configuration file
cargo run -- config show --file ccswarm.json
```

## 🔄 Common Workflows

### 1. Full-Stack Development Workflow
```bash
# 1. Initialize project
cargo run -- init --name "WebApp" --agents frontend,backend,devops

# 2. Start TUI for monitoring
cargo run -- tui

# 3. Add initial tasks (in TUI command mode)
task Create React app structure [high] [feature]
task Setup Express.js API [high] [feature]
task Configure CI/CD pipeline [medium] [infra]

# 4. Start orchestrator (in another terminal)
cargo run -- start
```

### 2. Bug Fix Workflow
```bash
# 1. Add bug fix task
cargo run -- task "Fix user authentication issue" --priority critical --type bugfix

# 2. Check agent status
cargo run -- status --detailed

# 3. Monitor progress
cargo run -- tui
```

### 3. Feature Development Workflow
```bash
# 1. Create feature branch worktree
cargo run -- worktree create agents/feature-agent feature/user-profiles --new-branch

# 2. Add feature tasks
cargo run -- task "Design user profile UI" --priority high --type development
cargo run -- task "Implement profile API" --priority high --type development
cargo run -- task "Add profile tests" --priority medium --type testing

# 3. Start development
cargo run -- start
```

## 📋 Examples

### Example 1: E-commerce Project
```bash
# Initialize e-commerce project
cargo run -- init --name "ECommerceShop" --agents frontend,backend,devops

# Add initial tasks
cargo run -- task "Create product catalog UI" --priority high --type feature
cargo run -- task "Implement shopping cart API" --priority high --type feature
cargo run -- task "Setup payment integration" --priority medium --type feature
cargo run -- task "Configure production deployment" --priority medium --type infra

# Start with TUI monitoring
cargo run -- tui
```

### Example 2: API-Only Project
```bash
# Initialize API project
cargo run -- init --name "UserAPI" --agents backend,qa

# Configure for API development
cargo run -- config generate --template minimal --output api-config.json

# Add API development tasks
cargo run -- task "Design REST API endpoints" --priority high --type development
cargo run -- task "Implement user CRUD operations" --priority high --type development
cargo run -- task "Add API documentation" --priority medium --type documentation
cargo run -- task "Write integration tests" --priority high --type testing

# Start development
cargo run -- start
```

### Example 3: Frontend-Only Project
```bash
# Initialize frontend project
cargo run -- init --name "ReactDashboard" --agents frontend

# Add frontend tasks
cargo run -- task "Create dashboard layout" --priority high --type development
cargo run -- task "Implement data visualization" --priority medium --type feature
cargo run -- task "Add responsive design" --priority medium --type development
cargo run -- task "Write component tests" --priority medium --type testing

# Monitor with TUI
cargo run -- tui
```

## 🔍 Monitoring and Debugging

### System Status
```bash
# Quick status check
cargo run -- status

# Detailed status with agent information
cargo run -- status --detailed

# Check specific agent
cargo run -- status --agent frontend
```

### Logs
```bash
# View logs
cargo run -- logs

# Follow logs in real-time
cargo run -- logs --follow

# View logs for specific agent
cargo run -- logs --agent backend --lines 50
```

### JSON Output
Add `--json` flag to any command for machine-readable output:
```bash
cargo run -- agents --json
cargo run -- status --json
cargo run -- worktree list --json
```

## ⚡ Pro Tips

1. **Use TUI for real-time monitoring**: `cargo run -- tui` provides the best overview
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
cargo run -- config generate
```

**Worktree conflicts:**
```bash
# Clean up stale worktrees
cargo run -- worktree prune
```

**Agent communication issues:**
```bash
# Check system status
cargo run -- status --detailed

# Review logs
cargo run -- logs --follow
```

For more help, check the [main documentation](README.md) or [architecture guide](CLAUDE.md).