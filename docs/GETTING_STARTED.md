# Getting Started with ccswarm

Welcome to **ccswarm** - an AI-powered multi-agent orchestration system that manages specialized AI agents for software development. This guide will walk you through everything you need to know to get started quickly and effectively.

## What is ccswarm?

ccswarm is a Rust-based system that coordinates specialized AI agents (Frontend, Backend, DevOps, QA) using a Master Claude orchestrator. It features:

- **93% token savings** through intelligent session reuse
- **Native terminal management** with cross-platform PTY support
- **Autonomous orchestration** with proactive task prediction
- **Security monitoring** with OWASP Top 10 scanning
- **Collective intelligence** through democratic decision-making

## Prerequisites

Before you begin, ensure you have:

### System Requirements
- **Operating System**: Linux, macOS (Windows not supported)
- **Rust**: Version 1.70 or higher
- **Git**: Version 2.20 or higher
- **Memory**: At least 4GB RAM recommended
- **Disk Space**: 1GB for installation + 100MB per agent worktree

### API Keys (Optional)
While you can run ccswarm in standalone mode, for full AI functionality you'll need:
- **Anthropic API Key** (recommended): For Claude-based agents
- **OpenAI API Key** (optional): For OpenAI-based agents

```bash
# Set your API keys
export ANTHROPIC_API_KEY="your-key-here"
export OPENAI_API_KEY="your-key-here"  # Optional
```

## Installation

### Option 1: Install from Crates.io (Recommended)

```bash
# Install the latest stable version
cargo install ccswarm

# Verify installation
ccswarm --version
```

### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/nwiizo/ccswarm.git
cd ccswarm

# Build and install
cargo build --release
cargo install --path crates/ccswarm

# Optional: Install ai-session CLI separately (v0.4.0)
cargo install --path crates/ai-session
```

## First-Time Setup

### Interactive Setup Wizard (Recommended)

The easiest way to get started is with the interactive setup wizard:

```bash
ccswarm setup
```

This will guide you through:
1. Project configuration
2. Agent selection and roles
3. Provider configuration
4. API key setup
5. Initial project structure

### Quick Manual Setup

If you prefer manual configuration:

```bash
# Initialize a new project
ccswarm init --name "MyFirstProject" --agents frontend,backend,devops

# This creates:
# - ccswarm.json configuration file
# - Git worktrees for each agent
# - Project-specific settings
```

## Your First Project

Let's create a simple TODO application to demonstrate ccswarm's capabilities.

### Step 1: Initialize Project

```bash
# Create a new TODO app project
ccswarm init --name "TodoApp" --agents frontend,backend

# Check the created structure
ls -la
```

You should see:
- `ccswarm.json` - Main configuration file
- `frontend-agent/` - Frontend agent worktree
- `backend-agent/` - Backend agent worktree

### Step 2: Start the System

Open two terminals:

**Terminal 1: Start the orchestrator**
```bash
ccswarm start
```

You'll see output like:
```
🚀 ccswarm v0.3.8 starting...
✅ Master Claude initialized (proactive mode enabled)
✅ Frontend agent ready
✅ Backend agent ready
🔒 Security agent monitoring enabled
📊 Proactive analysis running every 30s
```

**Terminal 2: Launch the monitoring UI**
```bash
ccswarm tui
```

This opens an interactive terminal interface showing:
- Real-time agent status
- Task queue and completion
- Session statistics
- System health metrics

### Step 3: Create Your First Task

```bash
# Create a high-priority feature task
ccswarm task "Create a responsive login form [high] [feature]"

# The system will:
# 1. Analyze the task with Master Claude
# 2. Determine it's frontend work
# 3. Delegate to the frontend agent
# 4. Execute the task automatically
```

### Step 4: Monitor Progress

In the TUI (Terminal 2), you can:
- Press `Tab` to switch between tabs
- Use `↑↓` or `jk` to navigate
- Press `c` for command mode
- Press `t` to add new tasks
- Press `q` to quit

### Step 5: Auto-Create Complete Applications

For faster development, use the auto-create feature:

```bash
# Generate a complete TODO application
ccswarm auto-create "Create a TODO app with React frontend and Node.js backend" --output ./my-todo-app

# The system will:
# 1. Analyze the requirements
# 2. Generate file structure
# 3. Create frontend components
# 4. Build backend API
# 5. Add Docker configuration
# 6. Include documentation
```

## Understanding Agent Roles

ccswarm enforces strict role boundaries to ensure quality and prevent scope creep:

### Frontend Agent
- **Responsibilities**: React, Vue, UI/UX, CSS, client-side code
- **Cannot do**: Backend APIs, database queries, server configuration
- **Tools**: Modern frontend frameworks, styling libraries, build tools

### Backend Agent  
- **Responsibilities**: APIs, databases, server logic, authentication
- **Cannot do**: UI components, styling, frontend routing
- **Tools**: Express, FastAPI, database ORMs, authentication systems

### DevOps Agent
- **Responsibilities**: Docker, CI/CD, infrastructure, deployment
- **Cannot do**: Application logic, UI components, business features
- **Tools**: Docker, Kubernetes, GitHub Actions, cloud platforms

### QA Agent
- **Responsibilities**: Testing, quality assurance, test coverage
- **Cannot do**: Feature implementation, infrastructure setup
- **Tools**: Jest, Pytest, Selenium, load testing tools

## Key Commands Reference

### Project Management
```bash
# Initialize new project
ccswarm init --name "ProjectName" --agents frontend,backend,devops

# Start system
ccswarm start

# Check status
ccswarm status --detailed

# Stop system
ccswarm stop
```

### Task Management
```bash
# Add tasks with priority and type
ccswarm task "Task description [high|medium|low] [feature|bug|test|docs]"

# List all tasks
ccswarm task list

# Get task details
ccswarm task show <task-id>

# Manual delegation
ccswarm delegate task "Task description" --agent frontend
```

### Session Management
```bash
# List active sessions (shows token savings)
ccswarm session list

# View session statistics
ccswarm session stats --show-savings

# Create new session
ccswarm session create --agent frontend

# Attach to session
ccswarm session attach <session-id>
```

### Auto-Create Applications
```bash
# Generate complete applications
ccswarm auto-create "App description" --output ./directory

# Examples:
ccswarm auto-create "Blog with authentication" --output ./blog
ccswarm auto-create "E-commerce shop" --output ./shop
ccswarm auto-create "Real-time chat app" --output ./chat
```

## Interactive Tutorial

ccswarm includes a comprehensive interactive tutorial:

```bash
# Start the tutorial
ccswarm tutorial

# Jump to specific chapters
ccswarm tutorial --chapter 3

# Available chapters:
# 1. Basic setup and configuration
# 2. Creating and managing tasks
# 3. Working with agents
# 4. Session management
# 5. Auto-create applications
# 6. Advanced features (Sangha, extensions)
```

## Health Check and Troubleshooting

### System Health Check
```bash
# Run comprehensive system diagnostics
ccswarm doctor

# Fix common issues automatically
ccswarm doctor --fix

# Check specific components
ccswarm doctor --check sessions
ccswarm doctor --check providers
ccswarm doctor --check worktrees
```

### Common Issues and Solutions

**Issue: "Session not found"**
```bash
# List all sessions
ccswarm session list

# Create a new session if needed
ccswarm session create --agent frontend
```

**Issue: "Provider error" or API key issues**
```bash
# Check your API keys
echo $ANTHROPIC_API_KEY

# Verify configuration
ccswarm config show

# Test provider connection
ccswarm config test-providers
```

**Issue: Git worktree conflicts**
```bash
# List worktrees
ccswarm worktree list

# Clean up unused worktrees
ccswarm worktree clean

# Reset if needed
ccswarm worktree reset --agent frontend
```

## Understanding the Interface

### Terminal UI (TUI) Navigation

The TUI has several tabs:

1. **Overview**: System status and metrics
2. **Tasks**: Active and completed tasks
3. **Agents**: Agent status and workload
4. **Sessions**: Session management and statistics
5. **Logs**: Real-time system logs

### Key Bindings
- `Tab/Shift+Tab`: Switch between tabs
- `↑↓` or `jk`: Navigate lists
- `Enter`: Select/activate items
- `c`: Command mode
- `t`: Quick task creation
- `s`: Session management
- `h` or `?`: Help
- `q`: Quit

### Command Mode
Press `c` to enter command mode, then use:
```
task <description> [priority] [type]    # Create task
agent <role>                           # Focus on agent
session list|attach|pause|resume       # Session commands
filter <pattern>                       # Filter content
monitor <agent>                        # Monitor specific agent
help                                   # Show help
```

## Next Steps

Now that you have ccswarm running:

1. **Explore the Tutorial**: Run `ccswarm tutorial` for hands-on learning
2. **Read the Configuration Guide**: See [CONFIGURATION.md](CONFIGURATION.md) for advanced setup
3. **Try Auto-Create**: Generate a few applications to see the system's capabilities
4. **Join the Community**: Contribute to the project or report issues on GitHub
5. **Advanced Features**: Learn about Sangha collective intelligence and self-extension

## Performance Tips

### Maximizing Token Savings
- Keep sessions active for related tasks (93% savings through reuse)
- Use similar tasks in sequence for better context compression
- Enable session pooling for high-throughput workflows

### Optimal Agent Usage
- Assign tasks to the most specialized agent
- Keep tasks focused within role boundaries
- Use batch operations for multiple related tasks

### System Performance
- Monitor memory usage with long-running sessions
- Clean up unused worktrees periodically
- Use the TUI for real-time performance monitoring

## Getting Help

### Built-in Help System
```bash
# General help
ccswarm help

# Topic-specific help
ccswarm help tasks
ccswarm help sessions
ccswarm help configuration

# Search help content
ccswarm help --search "delegation"
```

### Contextual Assistance
ccswarm provides smart error messages with suggested solutions:

```
❌ Configuration Error
   Missing required field: project.name

   💡 Quick Fix:
   ccswarm init --name "YourProject"

   📚 Learn More:
   ccswarm help configuration
   
   Error Code: CFG001
```

### Community Resources
- **GitHub Issues**: Report problems or request features
- **Documentation**: Comprehensive guides in the `docs/` directory
- **Examples**: Sample applications in the `demos/` directory

---

You're now ready to harness the power of AI-driven multi-agent development with ccswarm! The system will learn and adapt to your workflow, providing increasingly intelligent assistance as you use it.

For advanced configuration options, see [CONFIGURATION.md](CONFIGURATION.md).
For troubleshooting help, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).