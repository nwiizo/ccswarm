#!/bin/bash
# Quick start script for ccswarm standalone mode

set -e

echo "🚀 ccswarm Standalone Quick Start"
echo "================================"

# Check if ccswarm is installed
if ! command -v ccswarm &> /dev/null; then
    echo "❌ ccswarm not found. Please install it first:"
    echo "   cargo install ccswarm"
    echo "   or"
    echo "   cargo install --path ."
    exit 1
fi

# Create workspace directory
WORKSPACE_DIR="./ccswarm-standalone-demo"
echo "📁 Creating workspace at: $WORKSPACE_DIR"
mkdir -p "$WORKSPACE_DIR"
cd "$WORKSPACE_DIR"

# Initialize Git repository if needed
if [ ! -d .git ]; then
    echo "📝 Initializing Git repository..."
    git init
    git config user.name "ccswarm Demo"
    git config user.email "demo@ccswarm.local"
fi

# Initialize ccswarm project
echo "🔧 Initializing ccswarm project..."
ccswarm init --name "StandaloneDemo" --agents frontend,backend,devops,qa

# Update configuration for standalone mode
echo "⚙️ Configuring for standalone operation..."
cat > ccswarm.json << 'EOF'
{
  "project": {
    "name": "StandaloneDemo",
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
      }
    },
    "backend": {
      "specialization": "node_microservices",
      "worktree": "agents/backend-agent",
      "branch": "feature/backend",
      "claude_config": {
        "model": "simulation",
        "dangerous_skip": true
      }
    },
    "devops": {
      "specialization": "docker_kubernetes",
      "worktree": "agents/devops-agent",
      "branch": "feature/devops",
      "claude_config": {
        "model": "simulation",
        "dangerous_skip": true
      }
    },
    "qa": {
      "specialization": "testing_automation",
      "worktree": "agents/qa-agent",
      "branch": "feature/qa",
      "claude_config": {
        "model": "simulation",
        "dangerous_skip": true
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
EOF

# Start orchestrator in background
echo "🎯 Starting ccswarm orchestrator..."
CCSWARM_SIMULATION=true ccswarm start --daemon --port 8080 &
ORCHESTRATOR_PID=$!

# Wait for orchestrator to start
sleep 3

# Check status
echo "📊 Checking system status..."
ccswarm status --detailed

# Create a TODO application
echo ""
echo "🏗️ Auto-creating a TODO application..."
ccswarm auto-create "Create a modern TODO application with React frontend and Express backend" \
    --output ./todo-app

# Wait for generation to complete
echo "⏳ Waiting for application generation..."
sleep 5

# Show generated files
echo ""
echo "📁 Generated application structure:"
if [ -d ./todo-app ]; then
    tree ./todo-app 2>/dev/null || ls -la ./todo-app
else
    echo "Application directory not found. Checking workspace..."
    ls -la
fi

# Instructions for next steps
echo ""
echo "✅ Quick start complete!"
echo ""
echo "📋 Next steps:"
echo "1. Monitor agents in real-time:"
echo "   ccswarm tui"
echo ""
echo "2. Add more tasks:"
echo "   ccswarm task 'Add user authentication' --priority high"
echo "   ccswarm task 'Create API documentation' --priority medium"
echo ""
echo "3. Run the generated TODO app:"
echo "   cd todo-app"
echo "   npm install"
echo "   npm start"
echo ""
echo "4. Create more applications:"
echo "   ccswarm auto-create 'Create a blog with comments' --output ./blog-app"
echo ""
echo "5. Stop the orchestrator:"
echo "   ccswarm stop"
echo "   # or"
echo "   kill $ORCHESTRATOR_PID"
echo ""
echo "🎉 Enjoy exploring ccswarm in standalone mode!"