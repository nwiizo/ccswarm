# 🧠 AI-Session Documentation Hub

Welcome to the AI-Session documentation. AI-Session is the core session management library that powers ccswarm's 93% token savings and multi-agent coordination capabilities.

## 🎯 Overview

AI-Session replaces traditional terminal multiplexers like tmux with intelligent, AI-optimized session management designed specifically for AI agents and modern development workflows.

> **Key Benefits**: 93% token reduction • Native PTY • Multi-agent coordination • MCP protocol • Session persistence

## 📚 Documentation Structure

### 🚀 Getting Started
- **[../README.md](../README.md)** - AI-Session overview and quick start
- **[CLI_GUIDE.md](CLI_GUIDE.md)** - Command-line interface reference
- **[API_GUIDE.md](API_GUIDE.md)** - Complete API documentation with examples
- **[../examples/](../examples/)** - Practical usage examples and demos

### 🏗️ Architecture & Design
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture and design patterns
- **[ccswarm-integration-api.md](ccswarm-integration-api.md)** - ccswarm integration details
- **[message_bus_enhancements.md](message_bus_enhancements.md)** - Multi-agent communication

### 🔧 Integration Guides
- **[CLAUDE_CHAT_GUIDE.md](CLAUDE_CHAT_GUIDE.md)** - Claude integration patterns
- **[../../../docs/README.md](../../../docs/README.md)** - ccswarm master documentation
- **[../../../.claude/commands/session.md](../../../.claude/commands/session.md)** - Session commands in ccswarm

## 🔗 ccswarm Integration

AI-Session is deeply integrated with ccswarm to provide:

### 🧠 Token Efficiency
- **93% token reduction** through intelligent context compression
- **Conversation persistence** across task executions
- **Context reuse** for related agent operations
- **Automatic pruning** of old conversation history

### 🤝 Multi-Agent Features
- **Message bus architecture** for inter-agent communication
- **Role-based isolation** (Frontend, Backend, DevOps, QA agents)
- **Shared context management** across specialized agents
- **Coordination primitives** for task distribution

### 📊 Observability & Monitoring
- **Real-time metrics** via ccswarm TUI ([../../../.claude/commands/tui.md](../../../.claude/commands/tui.md))
- **Session statistics** and token usage tracking
- **Performance profiling** for optimization opportunities
- **Decision tracking** for AI agent behavior analysis

## 🚀 Quick Start Navigation

### 🆕 New to AI-Session?
1. **Overview**: Read [../README.md](../README.md) for features and benefits
2. **Installation**: Follow setup in [../../../README.md](../../../README.md)
3. **First Session**: Try examples in [../examples/](../examples/)
4. **CLI Usage**: Learn commands in [CLI_GUIDE.md](CLI_GUIDE.md)

### 👨‍💻 Developing with AI-Session?
1. **API Reference**: Study [API_GUIDE.md](API_GUIDE.md)
2. **Architecture**: Understand [ARCHITECTURE.md](ARCHITECTURE.md)
3. **Integration**: Check [ccswarm-integration-api.md](ccswarm-integration-api.md)
4. **Examples**: Explore [../examples/](../examples/) for patterns

### 🔧 ccswarm Integration?
1. **Session Commands**: Learn [../../../.claude/commands/session.md](../../../.claude/commands/session.md)
2. **Master Documentation**: See [../../../docs/README.md](../../../docs/README.md)
3. **Architecture**: Review [../../../docs/ARCHITECTURE.md](../../../docs/ARCHITECTURE.md)
4. **Task Management**: Understand [../../../.claude/commands/task.md](../../../.claude/commands/task.md)

## 📖 Core Features Documentation

### 🧠 AI-Optimized Session Management
- **Context Compression**: Automatic conversation summarization
- **Token Tracking**: Real-time usage monitoring
- **Smart Persistence**: Selective state preservation
- **Native PTY**: Cross-platform terminal emulation

### 🤝 Multi-Agent Coordination
- **Message Bus**: [message_bus_enhancements.md](message_bus_enhancements.md)
- **Agent Isolation**: Role-based boundaries
- **Task Distribution**: Intelligent workload balancing
- **Shared Context**: Cross-agent knowledge sharing

### 📡 Protocol Integration
- **MCP Support**: Model Context Protocol for standardized AI tool integration
- **HTTP API**: RESTful endpoints for external systems
- **JSON-RPC 2.0**: Standard communication protocol
- **Tool Discovery**: Automatic capability registration

### 🔒 Security & Isolation
- **Capability-based security**: Fine-grained access control
- **Session isolation**: Secure separation between agents
- **Rate limiting**: Resource abuse prevention
- **Audit trail**: Complete command history

## 📋 Common Use Cases

### 🎯 ccswarm Agent Management
```bash
# Create AI-optimized session for frontend agent
ccswarm session create --agent frontend --enable-ai-features

# View token savings across all sessions
ccswarm session stats --show-savings

# Attach to existing session with full context
ccswarm session attach frontend-abc123
```

See [../../../.claude/commands/session.md](../../../.claude/commands/session.md) for complete command reference.

### 🔧 Direct AI-Session Usage
```bash
# Create standalone session
ai-session create --name dev --ai-context

# Execute with output capture
ai-session exec dev "cargo test" --capture

# View AI-optimized context
ai-session context dev --lines 50
```

See [CLI_GUIDE.md](CLI_GUIDE.md) for complete CLI documentation.

### 💻 Library Integration
```rust
use ai_session::{SessionManager, SessionConfig};

// Create AI-optimized session
let manager = SessionManager::new();
let config = SessionConfig::default().with_ai_features(true);
let session = manager.create_session_with_config(config).await?;

// Execute commands with context preservation
session.send_input("cargo build\n").await?;
let context = session.get_ai_context().await?;
```

See [API_GUIDE.md](API_GUIDE.md) for complete API documentation.

## 🔗 Related Documentation

### ccswarm Core Documentation
- **[../../../docs/APPLICATION_SPEC.md](../../../docs/APPLICATION_SPEC.md)** - Complete ccswarm features
- **[../../../docs/ARCHITECTURE.md](../../../docs/ARCHITECTURE.md)** - Overall system design
- **[../../../CLAUDE.md](../../../CLAUDE.md)** - Development guidelines

### Command References
- **[../../../.claude/commands/](../../../.claude/commands/)** - All ccswarm commands
- **[../../../.claude/commands/init.md](../../../.claude/commands/init.md)** - Project initialization
- **[../../../.claude/commands/task.md](../../../.claude/commands/task.md)** - Task management
- **[../../../.claude/commands/tui.md](../../../.claude/commands/tui.md)** - Terminal monitoring

### Development Resources
- **[../../../docs/DEVELOPER_GUIDE.md](../../../docs/DEVELOPER_GUIDE.md)** - Development workflows
- **[../../../.claude/commands/project-rules.md](../../../.claude/commands/project-rules.md)** - Coding standards
- **[../../../.claude/commands/workspace-commands.md](../../../docs/commands/workspace-commands.md)** - Multi-crate development

## 🎯 Performance & Benefits

### Token Efficiency Metrics
- **93% average token reduction** compared to traditional approaches
- **~70% memory reduction** with zstd compression
- **<100ms session creation** for typical workloads
- **>1000 messages/sec** multi-agent coordination

### Resource Usage
- **~3.6MB per active session** memory footprint
- **<5ms overhead** per command execution
- **<1ms context retrieval** for 4K token contexts
- **Zero external dependencies** for PTY operations

## 🔧 Development & Contributing

### Building AI-Session
```bash
# Build the library
cd crates/ai-session
cargo build --release

# Run tests
cargo test

# Build with CLI features
cargo build --features cli
```

### Integration Testing
```bash
# Run integration tests
cargo test --test integration_tests

# Test ccswarm integration
cd ../.. && cargo test -p ccswarm --features ai-session
```

### Documentation
```bash
# Generate API docs
cargo doc --no-deps --open

# Check documentation links
cargo doc --document-private-items
```

## 📞 Getting Help

### Common Issues
- **Session Creation Fails**: Check [../../../docs/TROUBLESHOOTING.md](../../../docs/TROUBLESHOOTING.md)
- **Token Usage High**: Review context configuration in [API_GUIDE.md](API_GUIDE.md)
- **Integration Problems**: See [ccswarm-integration-api.md](ccswarm-integration-api.md)

### Support Channels
- **Issues**: [GitHub Issues](https://github.com/nwiizo/ccswarm/issues)
- **Discussions**: [GitHub Discussions](https://github.com/nwiizo/ccswarm/discussions)
- **Documentation**: This directory and [../../../docs/README.md](../../../docs/README.md)

---

**Note**: This documentation is part of the ccswarm workspace. For the complete system documentation, see [../../../docs/README.md](../../../docs/README.md).