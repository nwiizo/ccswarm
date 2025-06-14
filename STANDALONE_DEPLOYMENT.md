# 🚀 ccswarm Standalone Deployment Guide

This guide explains how to deploy and use ccswarm without Claude Code or other AI provider dependencies. ccswarm includes built-in templates and simulation modes that allow you to explore its multi-agent orchestration capabilities without requiring API keys or external services.

## 📋 Table of Contents

- [Installation](#installation)
- [Configuration for Standalone Operation](#configuration-for-standalone-operation)
- [Using Simulation Mode](#using-simulation-mode)
- [Auto-Create Functionality](#auto-create-functionality)
- [Docker Deployment](#docker-deployment)
- [Practical Examples](#practical-examples)
- [Architecture Overview](#architecture-overview)

## 🛠️ Installation

### Option 1: Install from Crates.io
```bash
cargo install ccswarm
```

### Option 2: Build from Source
```bash
# Clone the repository
git clone https://github.com/nwiizo/ccswarm.git
cd ccswarm

# Build in release mode
cargo build --release

# Install to PATH
cargo install --path .
```

### Option 3: Direct Binary Download
```bash
# Download latest release (adjust for your platform)
curl -L https://github.com/nwiizo/ccswarm/releases/latest/download/ccswarm-linux-amd64 -o ccswarm
chmod +x ccswarm
sudo mv ccswarm /usr/local/bin/
```

### System Requirements
- Rust 1.70+ (for building from source)
- Git (for worktree management)
- Node.js (optional, for running generated applications)
- tmux (optional, for session management)

## 🔧 Configuration for Standalone Operation

### 1. Initialize a Standalone Project
```bash
# Create a new ccswarm project without AI dependencies
ccswarm init --name "MyStandaloneProject" --agents frontend,backend,devops

# This creates a ccswarm.json configuration file
```

### 2. Minimal Configuration (ccswarm.json)
```json
{
  "project": {
    "name": "MyStandaloneProject",
    "repository": {
      "url": "./",
      "main_branch": "main"
    },
    "master_claude": {
      "role": "technical_lead",
      "quality_threshold": 0.9,
      "think_mode": "standard",
      "permission_level": "supervised",
      "claude_config": {
        "model": "simulation",
        "dangerous_skip": true,
        "think_mode": null,
        "json_output": true
      }
    }
  },
  "agents": {
    "frontend": {
      "specialization": "react_typescript",
      "worktree": "agents/frontend-agent",
      "branch": "feature/frontend",
      "claude_config": {
        "model": "simulation",
        "dangerous_skip": true
      },
      "provider": {
        "type": "simulation",
        "auto_complete": true
      }
    },
    "backend": {
      "specialization": "node_microservices",
      "worktree": "agents/backend-agent",
      "branch": "feature/backend",
      "claude_config": {
        "model": "simulation",
        "dangerous_skip": true
      },
      "provider": {
        "type": "simulation",
        "auto_complete": true
      }
    },
    "devops": {
      "specialization": "docker_kubernetes",
      "worktree": "agents/devops-agent",
      "branch": "feature/devops",
      "claude_config": {
        "model": "simulation",
        "dangerous_skip": true
      },
      "provider": {
        "type": "simulation",
        "auto_complete": true
      }
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

## 🎭 Using Simulation Mode

ccswarm includes a simulation mode that mimics agent behavior without requiring actual AI providers:

### 1. Start in Simulation Mode
```bash
# Start the orchestrator with simulation agents
ccswarm start --simulation

# Or use environment variable
CCSWARM_SIMULATION=true ccswarm start
```

### 2. Monitor Simulated Agents
```bash
# Launch the TUI to see agents in action
ccswarm tui

# Check status
ccswarm status --detailed
```

### 3. Add Tasks for Simulation
```bash
# Add tasks that will be processed by simulated agents
ccswarm task "Create user authentication system" --priority high --type feature
ccswarm task "Add API endpoints for user management" --priority medium --type development
ccswarm task "Write tests for authentication" --priority medium --type testing
ccswarm task "Deploy to staging environment" --priority low --type infrastructure
```

## 🚀 Auto-Create Functionality

The most powerful standalone feature is auto-create, which generates complete applications using built-in templates:

### 1. Create a TODO Application
```bash
# Generate a complete TODO app with React frontend and Express backend
ccswarm auto-create "Create a TODO application with task management" --output ./my-todo-app

# The system will:
# - Analyze your request
# - Delegate tasks to appropriate agents
# - Generate all necessary files
# - Create a fully functional application
```

### 2. Create a Blog Application
```bash
ccswarm auto-create "Build a blog with article management" --output ./my-blog
```

### 3. Create an E-commerce Site
```bash
ccswarm auto-create "Create an online shop with product catalog" --output ./my-shop
```

### 4. Use Templates
```bash
# List available templates
ccswarm auto-create --list-templates

# Use a specific template
ccswarm auto-create "My App" --template todo --output ./my-app
```

### Generated Application Structure
```
my-todo-app/
├── index.html          # React-based frontend
├── app.js             # Frontend logic
├── styles.css         # Styling
├── server.js          # Express backend
├── package.json       # Node.js dependencies
├── Dockerfile         # Container configuration
├── docker-compose.yml # Multi-container setup
├── app.test.js        # Test structure
├── README.md          # Documentation
└── .gitignore         # Git configuration
```

### Running Generated Applications
```bash
cd my-todo-app

# Install dependencies
npm install

# Start the application
npm start

# Access at http://localhost:3001
```

## 🐳 Docker Deployment

### 1. Dockerize ccswarm
```Dockerfile
# Dockerfile for ccswarm
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    git \
    tmux \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ccswarm /usr/local/bin/
WORKDIR /workspace

CMD ["ccswarm", "start"]
```

### 2. Docker Compose Setup
```yaml
# docker-compose.yml
version: '3.8'

services:
  ccswarm-master:
    build: .
    volumes:
      - ./workspace:/workspace
      - ./ccswarm.json:/workspace/ccswarm.json
    environment:
      - CCSWARM_SIMULATION=true
      - RUST_LOG=info
    command: ["ccswarm", "start", "--daemon"]

  ccswarm-tui:
    build: .
    volumes:
      - ./workspace:/workspace
    environment:
      - TERM=xterm-256color
    tty: true
    stdin_open: true
    command: ["ccswarm", "tui"]
    depends_on:
      - ccswarm-master
```

### 3. Deploy with Docker
```bash
# Build and start ccswarm
docker-compose up -d

# View logs
docker-compose logs -f ccswarm-master

# Access TUI
docker-compose run --rm ccswarm-tui

# Generate an application
docker-compose exec ccswarm-master ccswarm auto-create \
  "Create TODO app" --output /workspace/todo-app
```

## 📚 Practical Examples

### Example 1: Full Development Workflow
```bash
# 1. Initialize project
ccswarm init --name "WebApp" --agents frontend,backend,devops,qa

# 2. Start orchestrator in simulation mode
CCSWARM_SIMULATION=true ccswarm start --daemon

# 3. Auto-create a web application
ccswarm auto-create "Create a task management dashboard with user authentication" \
  --output ./task-dashboard

# 4. Monitor progress
ccswarm tui

# 5. Check generated files
cd task-dashboard
ls -la

# 6. Run the application
npm install
npm start
```

### Example 2: Batch Task Processing
```bash
# Create a task file
cat > tasks.txt << EOF
Create user registration form
Implement JWT authentication
Add password reset functionality
Create user profile page
Set up email notifications
Write API documentation
Create deployment scripts
EOF

# Process tasks in batch
while IFS= read -r task; do
  ccswarm task "$task" --priority medium --type development
done < tasks.txt

# Monitor execution
ccswarm status --detailed
```

### Example 3: Custom Agent Configuration
```bash
# Create custom agent configuration
cat > custom-agents.json << EOF
{
  "ml-engineer": {
    "specialization": "machine_learning",
    "worktree": "agents/ml-agent",
    "branch": "feature/ml",
    "provider": {
      "type": "custom",
      "command": "echo",
      "args": ["Simulating ML task: {prompt}"]
    }
  },
  "security": {
    "specialization": "security_audit",
    "worktree": "agents/security-agent",
    "branch": "feature/security",
    "provider": {
      "type": "custom",
      "command": "echo",
      "args": ["Simulating security scan: {prompt}"]
    }
  }
}
EOF

# Merge with existing configuration
ccswarm config merge custom-agents.json
```

## 🏗️ Architecture Overview

### How Standalone Mode Works

1. **Simulation Engine**: Built-in logic that mimics agent behavior based on task types
2. **Template System**: Pre-built application templates for common use cases
3. **Task Routing**: Intelligent task distribution based on agent specializations
4. **File Generation**: Direct file creation without external API calls
5. **Coordination Bus**: JSON-based inter-agent communication

### Agent Specializations in Standalone Mode

- **Frontend Agent**: Generates React/Vue/Angular components, HTML/CSS/JS
- **Backend Agent**: Creates REST APIs, database schemas, server logic
- **DevOps Agent**: Produces Docker configs, CI/CD pipelines, deployment scripts
- **QA Agent**: Generates test suites, test plans, quality reports

### Benefits of Standalone Mode

1. **No API Keys Required**: Works without external services
2. **Predictable Behavior**: Consistent output from templates
3. **Fast Execution**: No network latency
4. **Learning Tool**: Understand multi-agent orchestration
5. **Prototyping**: Quickly scaffold applications
6. **Offline Development**: Works without internet connection

## 🔍 Monitoring and Debugging

### View Agent Activities
```bash
# Real-time monitoring
ccswarm monitor --agent frontend --filter "info,warning,error"

# View logs
ccswarm logs --follow --lines 100

# Check task queue
ccswarm task list --pending
```

### Debug Configuration
```bash
# Validate configuration
ccswarm config validate

# Show current configuration
ccswarm config show

# Test agent connectivity
ccswarm agent test --all
```

## 🎯 Advanced Usage

### Custom Templates
Create your own templates by adding them to the auto-create engine:

```rust
// In your fork of ccswarm
// src/orchestrator/auto_create.rs

templates.insert(AppType::Custom("my-template".to_string()), vec![
    TaskTemplate {
        id: "custom-task-1".to_string(),
        description: "Create custom component".to_string(),
        target_agent: "frontend".to_string(),
        priority: Priority::High,
        task_type: TaskType::Feature,
        dependencies: vec![],
        estimated_duration: Some(1800),
    },
    // Add more tasks...
]);
```

### Integration with CI/CD
```yaml
# .github/workflows/ccswarm.yml
name: Generate App with ccswarm

on:
  push:
    branches: [main]

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install ccswarm
        run: |
          curl -L https://github.com/nwiizo/ccswarm/releases/latest/download/ccswarm-linux-amd64 -o ccswarm
          chmod +x ccswarm
          sudo mv ccswarm /usr/local/bin/
      
      - name: Generate Application
        run: |
          ccswarm auto-create "${{ github.event.head_commit.message }}" \
            --output ./generated-app
      
      - name: Upload Artifact
        uses: actions/upload-artifact@v3
        with:
          name: generated-app
          path: ./generated-app
```

## 📝 Conclusion

ccswarm's standalone mode provides a powerful way to explore multi-agent orchestration and generate real applications without external dependencies. Whether you're prototyping, learning, or building actual projects, the built-in templates and simulation capabilities offer a complete development experience.

For more advanced features with actual AI providers, refer to the main documentation. The standalone mode serves as both a learning platform and a practical tool for rapid application development.