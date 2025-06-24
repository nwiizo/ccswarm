# ccswarm Documentation

Welcome to the ccswarm documentation. This directory contains comprehensive documentation following Claude Code best practices.

## Workspace Structure

ccswarm uses a Cargo workspace with the following organization:
- `crates/ccswarm/` - Main orchestration application
- `crates/ai-session/` - Native AI session management library

See [commands/workspace-commands.md](commands/workspace-commands.md) for workspace-specific commands.

## Documentation Structure

### Core Documentation
- **[APPLICATION_SPEC.md](APPLICATION_SPEC.md)** - Complete application specifications and features
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture and design patterns
- **[commands/](commands/)** - Command reference documentation
- **[../.claude/commands/](../.claude/commands/)** - Additional CLI command references

### Technical Specifications
- **[ai-session-migration-plan.md](ai-session-migration-plan.md)** - Native session management details
- **[MCP_IMPLEMENTATION_PLAN.md](MCP_IMPLEMENTATION_PLAN.md)** - Model Context Protocol integration
- **[sangha-extension.md](sangha-extension.md)** - Collective intelligence system
- **[llm-quality-judge.md](llm-quality-judge.md)** - Quality review system

### Claude Code Integration
- **[../CLAUDE.md](../CLAUDE.md)** - Project-specific Claude instructions (root directory)
- **[../.claude/settings.json](../.claude/settings.json)** - Claude Code configuration
- **[../.claude/commands/project-rules.md](../.claude/commands/project-rules.md)** - Development rules

## Quick Start Guide

### For New Users
1. Read [APPLICATION_SPEC.md](APPLICATION_SPEC.md) for system overview
2. Follow installation in the main [README.md](../README.md)
3. Check [../.claude/commands/init.md](../.claude/commands/init.md) to create your first project
4. Use [../.claude/commands/task.md](../.claude/commands/task.md) to understand task management

### For Developers
1. Study [ARCHITECTURE.md](ARCHITECTURE.md) for system design
2. Read [../CLAUDE.md](../CLAUDE.md) for development guidelines
3. Review [commands/workspace-commands.md](commands/workspace-commands.md) for workspace development
4. Follow [../.claude/commands/project-rules.md](../.claude/commands/project-rules.md) for coding standards
5. Check [../.claude/commands/](../.claude/commands/) for all available commands

### For Contributors
1. Read [../CONTRIBUTING.md](../CONTRIBUTING.md) for contribution process
2. Study [../.claude/commands/git-workflow.md](../.claude/commands/git-workflow.md) for Git practices
3. Follow [../.claude/commands/release-procedure.md](../.claude/commands/release-procedure.md) for releases

## Command Documentation

All CLI commands are documented in the [../.claude/commands/](../.claude/commands/) directory:

### Essential Commands
- [init.md](../.claude/commands/init.md) - Initialize new projects
- [start.md](../.claude/commands/start.md) - Start the orchestrator
- [task.md](../.claude/commands/task.md) - Create and manage tasks
- [session.md](../.claude/commands/session.md) - Session management
- [tui.md](../.claude/commands/tui.md) - Terminal UI monitoring

### Advanced Commands
- [auto-create.md](../.claude/commands/auto-create.md) - Generate apps from descriptions
- [sangha.md](../.claude/commands/sangha.md) - Collective intelligence
- [extend.md](../.claude/commands/extend.md) - Agent self-extension
- [quality.md](../.claude/commands/quality.md) - Code quality reviews

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