# ccswarm Demos

This directory contains demonstration projects, examples, and sample applications showcasing ccswarm's capabilities.

## Demo Categories

### Application Demos
- **todo-app/** - Complete TODO application with React frontend and Express backend
  - Full CRUD operations
  - Example builder code (`todo_app_builder.rs`)
  - Ready-to-run application
  
- **blog-app/** - Blog platform with authentication (template available)
- **ecommerce-app/** - E-commerce site with shopping cart (template available)
- **chat-app/** - Real-time chat with WebSockets (template available)

### Feature Demos
- **multi-agent/** - Multi-agent orchestration examples
  - Configuration examples for different setups
  - Claude MD templates for agent identities
  - Provider integration demos
  - Sample projects (DemoProject, TestProject, etc.)
  
- **session-persistence/** - Session management demonstrations
  - 93% token reduction examples
  - Session pooling and batch processing
  - Real-world session demos
  
- **auto-create/** - Natural language to application generation
  - Auto-create demonstration code
  - Shows how to generate apps from descriptions

## Running Demos

Each demo directory can contain its own README with specific instructions. General pattern:

```bash
# Initialize a demo project
cargo run -- init --name "DemoProject" --agents frontend,backend,devops

# Auto-create from template
cargo run -- auto-create "Create TODO app" --output ./demos/todo-app

# Start with specific configuration
cargo run -- start --config ./demos/multi-agent/ccswarm.json
```

## Note

All demo content is git-ignored to keep the repository clean. Demos are meant to be generated locally for testing and experimentation.
EOF < /dev/null