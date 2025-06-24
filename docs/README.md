# ccswarm Documentation

Welcome to the ccswarm documentation. This directory contains comprehensive documentation about the ccswarm AI Multi-Agent Orchestration System.

## Documentation Structure

### Core Documentation
- **[APPLICATION_SPEC.md](APPLICATION_SPEC.md)** - Complete application specifications, features, and usage
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - System architecture, components, and data flow
- **[CHANGELOG.md](../CHANGELOG.md)** - Version history and release notes

### Technical Specifications
- **[ai-session-migration-plan.md](ai-session-migration-plan.md)** - Native session management implementation
- **[MCP_IMPLEMENTATION_PLAN.md](MCP_IMPLEMENTATION_PLAN.md)** - Model Context Protocol integration
- **[sangha-extension.md](sangha-extension.md)** - Collective intelligence system
- **[llm-quality-judge.md](llm-quality-judge.md)** - Quality review system

### Command Documentation
All command-specific documentation is in `.claude/commands/`:
- Usage examples
- Command options
- Best practices

### Development Guidelines
- **[../CLAUDE.md](../CLAUDE.md)** - Claude Code integration guidelines
- **[../.claude/commands/project-rules.md](../.claude/commands/project-rules.md)** - Critical project rules and patterns

## Quick Navigation

### For Users
1. Start with [APPLICATION_SPEC.md](APPLICATION_SPEC.md) for overview
2. Check `.claude/commands/` for specific command help
3. See [../README.md](../README.md) for quick start guide

### For Developers
1. Read [ARCHITECTURE.md](ARCHITECTURE.md) for system design
2. Review [../CLAUDE.md](../CLAUDE.md) for development guidelines
3. Check [../.claude/commands/project-rules.md](../.claude/commands/project-rules.md) for coding standards

### For Contributors
1. Follow guidelines in [../CONTRIBUTING.md](../CONTRIBUTING.md)
2. Check [../.claude/commands/git-workflow.md](../.claude/commands/git-workflow.md) for Git practices
3. Review [../.claude/commands/release-procedure.md](../.claude/commands/release-procedure.md) for releases

## Documentation Standards

### File Naming
- Use UPPERCASE for primary docs (APPLICATION_SPEC.md)
- Use lowercase with hyphens for technical docs (ai-session-migration-plan.md)
- Keep names descriptive and searchable

### Content Structure
1. **Title and Overview** - What the document covers
2. **Table of Contents** - For documents >500 lines
3. **Main Content** - Organized with clear headers
4. **Examples** - Code samples and usage examples
5. **References** - Links to related documentation

### Maintenance
- Update documentation with code changes
- Review quarterly for accuracy
- Archive outdated docs in `archive/` directory

## Archive
The `archive/` directory contains historical documentation that may be useful for reference but is no longer current.