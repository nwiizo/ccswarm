# ccswarm Documentation Hub

Welcome to the comprehensive ccswarm documentation. This serves as the master index for both ccswarm orchestration and ai-session management documentation.

## 🏗️ Workspace Structure

ccswarm is a Cargo workspace with two primary crates:
- **`crates/ccswarm/`** - Main AI multi-agent orchestration application
- **`crates/ai-session/`** - Native AI session management library (93% token savings)

See [commands/workspace-commands.md](commands/workspace-commands.md) for workspace-specific commands.

## 📚 Documentation Structure

### 🎯 Core ccswarm Documentation
- **[APPLICATION_SPEC.md](APPLICATION_SPEC.md)** - Complete application specifications and features
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture and design patterns
- **[GETTING_STARTED.md](GETTING_STARTED.md)** - Quick start guide for new users
- **[CONFIGURATION.md](CONFIGURATION.md)** - Configuration options and examples
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Common issues and solutions

### 🧠 AI-Session Documentation
- **[../crates/ai-session/README.md](../crates/ai-session/README.md)** - AI-Session overview and features
- **[../crates/ai-session/docs/README.md](../crates/ai-session/docs/README.md)** - AI-Session documentation index
- **[../crates/ai-session/docs/API_GUIDE.md](../crates/ai-session/docs/API_GUIDE.md)** - Complete API reference
- **[../crates/ai-session/docs/CLI_GUIDE.md](../crates/ai-session/docs/CLI_GUIDE.md)** - Command-line interface
- **[../crates/ai-session/docs/ARCHITECTURE.md](../crates/ai-session/docs/ARCHITECTURE.md)** - AI-Session architecture
- **[../crates/ai-session/docs/ccswarm-integration-api.md](../crates/ai-session/docs/ccswarm-integration-api.md)** - ccswarm integration details

### 🔧 Technical Specifications
- **[MCP_IMPLEMENTATION_PLAN.md](MCP_IMPLEMENTATION_PLAN.md)** - Model Context Protocol integration
- **[sangha-extension.md](sangha-extension.md)** - Collective intelligence system
- **[llm-quality-judge.md](llm-quality-judge.md)** - Quality review system

### 📖 Command References
- **[commands/](commands/)** - Command reference documentation
- **[../.claude/commands/](../.claude/commands/)** - Complete CLI command references
- **[../.claude/commands/session.md](../.claude/commands/session.md)** - AI-Session management commands

### 🔧 Claude Code Integration
- **[../CLAUDE.md](../CLAUDE.md)** - Project-specific Claude instructions (root directory)
- **[../.claude/settings.json](../.claude/settings.json)** - Claude Code configuration
- **[../.claude/commands/project-rules.md](../.claude/commands/project-rules.md)** - Development rules

## 🚀 Quick Start Guide

### 🆕 For New Users
1. **System Overview**: Read [APPLICATION_SPEC.md](APPLICATION_SPEC.md) for complete feature list
2. **Installation**: Follow setup in the main [README.md](../README.md)
3. **First Project**: Use [../.claude/commands/init.md](../.claude/commands/init.md) to create your first project
4. **Task Management**: Learn [../.claude/commands/task.md](../.claude/commands/task.md) for creating and delegating tasks
5. **AI Sessions**: Understand [../.claude/commands/session.md](../.claude/commands/session.md) for 93% token savings

### 👨‍💻 For Developers
1. **System Architecture**: Study [ARCHITECTURE.md](ARCHITECTURE.md) for overall design
2. **AI-Session Architecture**: Review [../crates/ai-session/docs/ARCHITECTURE.md](../crates/ai-session/docs/ARCHITECTURE.md) for session management
3. **Development Rules**: Read [../CLAUDE.md](../CLAUDE.md) for coding guidelines
4. **Workspace Commands**: Use [commands/workspace-commands.md](commands/workspace-commands.md) for multi-crate development
5. **Project Standards**: Follow [../.claude/commands/project-rules.md](../.claude/commands/project-rules.md)
6. **API Integration**: Check [../crates/ai-session/docs/ccswarm-integration-api.md](../crates/ai-session/docs/ccswarm-integration-api.md)

### 🤝 For Contributors
1. **Contribution Process**: Read [../CONTRIBUTING.md](../CONTRIBUTING.md)
2. **Git Workflow**: Study [../.claude/commands/git-workflow.md](../.claude/commands/git-workflow.md)
3. **Release Process**: Follow [../.claude/commands/release-procedure.md](../.claude/commands/release-procedure.md)
4. **AI-Session Development**: See [../crates/ai-session/docs/README.md](../crates/ai-session/docs/README.md)

## 📖 Command Documentation

### 🎯 Essential ccswarm Commands
- **[init.md](../.claude/commands/init.md)** - Initialize new projects
- **[start.md](../.claude/commands/start.md)** - Start the orchestrator
- **[task.md](../.claude/commands/task.md)** - Create and manage tasks
- **[tui.md](../.claude/commands/tui.md)** - Terminal UI monitoring
- **[agent.md](../.claude/commands/agents.md)** - Manage specialized agents

### 🧠 AI-Session Commands
- **[session.md](../.claude/commands/session.md)** - AI session management (93% token savings)
- **[../crates/ai-session/docs/CLI_GUIDE.md](../crates/ai-session/docs/CLI_GUIDE.md)** - Complete AI-Session CLI reference

### ⚡ Advanced Features
- **[auto-create.md](../.claude/commands/auto-create.md)** - Generate complete applications from descriptions
- **[sangha.md](../.claude/commands/sangha.md)** - Collective intelligence and democratic decision-making
- **[extend.md](../.claude/commands/extend.md)** - Autonomous agent self-extension
- **[quality.md](../.claude/commands/quality.md)** - Automated code quality reviews

### 🔧 Development Commands
- **[worktree.md](../.claude/commands/worktree.md)** - Git worktree management for agent isolation
- **[config.md](../.claude/commands/config.md)** - Configuration management
- **[logs.md](../.claude/commands/logs.md)** - Logging and debugging
- **[status.md](../.claude/commands/status.md)** - System status monitoring

## 🔗 Key Integration Points

### ccswarm ↔ AI-Session Integration
- **[../crates/ai-session/docs/ccswarm-integration-api.md](../crates/ai-session/docs/ccswarm-integration-api.md)** - Technical integration details
- **Token Savings**: AI-Session provides 93% token reduction for ccswarm agents
- **Session Persistence**: Conversation history survives restarts and crashes
- **Multi-Agent Coordination**: Message bus for inter-agent communication

### Common Workflows
1. **Project Initialization**: [init.md](../.claude/commands/init.md) → [session.md](../.claude/commands/session.md)
2. **Task Execution**: [task.md](../.claude/commands/task.md) → [session.md](../.claude/commands/session.md) → [quality.md](../.claude/commands/quality.md)
3. **Agent Development**: [ARCHITECTURE.md](ARCHITECTURE.md) → [../crates/ai-session/docs/ARCHITECTURE.md](../crates/ai-session/docs/ARCHITECTURE.md)
4. **Troubleshooting**: [TROUBLESHOOTING.md](TROUBLESHOOTING.md) → [logs.md](../.claude/commands/logs.md)

## 📋 Documentation Categories

### 📖 Beginner Documentation
- [APPLICATION_SPEC.md](APPLICATION_SPEC.md) - What ccswarm does
- [GETTING_STARTED.md](GETTING_STARTED.md) - Installation and first steps
- [../crates/ai-session/README.md](../crates/ai-session/README.md) - AI-Session overview
- [../.claude/commands/init.md](../.claude/commands/init.md) - Creating your first project

### 🔧 Technical Documentation
- [ARCHITECTURE.md](ARCHITECTURE.md) - ccswarm system design
- [../crates/ai-session/docs/ARCHITECTURE.md](../crates/ai-session/docs/ARCHITECTURE.md) - AI-Session architecture
- [../crates/ai-session/docs/API_GUIDE.md](../crates/ai-session/docs/API_GUIDE.md) - Complete API reference
- [MCP_IMPLEMENTATION_PLAN.md](MCP_IMPLEMENTATION_PLAN.md) - Model Context Protocol

### 🚀 Advanced Features
- [sangha-extension.md](sangha-extension.md) - Collective intelligence
- [llm-quality-judge.md](llm-quality-judge.md) - Automated quality reviews
- [../.claude/commands/extend.md](../.claude/commands/extend.md) - Agent self-improvement
- [../.claude/commands/auto-create.md](../.claude/commands/auto-create.md) - AI app generation

## Documentation Standards

### Markdown Formatting
- Use clear hierarchical headings (H1 for title, H2 for sections)
- Include code examples with syntax highlighting
- Add tables for complex comparisons
- Use relative links between documents

### Content Guidelines
- Start with a brief overview
- Include practical examples
- Document edge cases and gotchas
- Keep language clear and concise

### Maintenance
- Update docs with code changes
- Review quarterly for accuracy
- Archive outdated content in [archive/](archive/)
- Track major changes in [../CHANGELOG.md](../CHANGELOG.md)

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/nwiizo/ccswarm/issues)
- **Discussions**: [GitHub Discussions](https://github.com/nwiizo/ccswarm/discussions)
- **Documentation**: This directory and [../.claude/commands/help-topic.md](../.claude/commands/help-topic.md)

## Archive

Historical documentation is preserved in the [archive/](archive/) directory for reference.