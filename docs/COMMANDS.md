# Commands Reference

Complete CLI command reference for ccswarm.

> See also: [Claude ACP Guide](CLAUDE_ACP.md) for Claude Code commands

## Basic Operations

```bash
# Initialize project
ccswarm init --name "Project" --agents frontend,backend

# Start system
ccswarm start

# Launch TUI
ccswarm tui

# Check status
ccswarm status --detailed

# Stop orchestrator
ccswarm stop
```

## Task Management

```bash
# Add task
ccswarm task "Create login form" --priority high --type feature

# Delegate task with analysis
ccswarm delegate analyze "Add authentication" --verbose
ccswarm delegate task "Add auth" --agent backend --priority high

# View delegation statistics
ccswarm delegate stats --period 24h
```

## System Management

```bash
# Check system health
ccswarm health
ccswarm health --detailed
ccswarm health agents
ccswarm health sessions

# Template management
ccswarm template list
ccswarm template create "MyTemplate" --from-project ./my-app
ccswarm template apply "MyTemplate" --output ./new-app

# Evolution tracking
ccswarm evolution status
ccswarm evolution history --agent frontend
ccswarm evolution metrics --period 7d
```

## Auto-Create Applications

```bash
# TODO app
ccswarm auto-create "Create TODO app" --output ./todo

# Blog with features
ccswarm auto-create "Blog with auth and comments" --output ./blog

# E-commerce
ccswarm auto-create "Online shop with cart" --output ./shop

# Custom template
ccswarm auto-create "Project description" --template custom --output ./app
```

## Session Commands

```bash
ccswarm session list
ccswarm session stats --show-savings

# Create and manage sessions
ccswarm session create --agent frontend --enable-ai-features
ccswarm session attach <session-id>
ccswarm session pause <session-id>
ccswarm session resume <session-id>

# MCP server
ccswarm session start-mcp-server --port 3000
ccswarm session mcp-status

# Session optimization
ccswarm session compress --threshold 0.8
ccswarm session optimize --all
```

## Proactive & Security

```bash
# Proactive analysis
ccswarm proactive analyze --all-agents
ccswarm proactive analyze --agent frontend

# Security scanning
ccswarm security scan --directory ./src
ccswarm security report --show-history
ccswarm security check --owasp-top-10

# Goals and milestones
ccswarm goal set "Build MVP" --deadline 30d
ccswarm milestone add "Frontend Complete" --deadline 14d
ccswarm progress show --detailed

# Dependencies
ccswarm deps analyze --show-blockers
ccswarm deps resolve --auto-order
```

## Sangha (Collective Intelligence)

```bash
# Submit proposals
ccswarm sangha propose --type doctrine --title "Code Quality Standards"
ccswarm sangha propose --type extension --title "React Server Components"

# Vote on proposals
ccswarm sangha vote <proposal-id> aye --reason "Improves performance"
ccswarm sangha vote <proposal-id> nay --reason "Too complex"

# View proposals
ccswarm sangha list --status active
ccswarm sangha show <proposal-id>
```

## Self-Extension

```bash
# Autonomous self-extension
ccswarm extend autonomous
ccswarm extend autonomous --agent backend
ccswarm extend autonomous --dry-run
ccswarm extend autonomous --continuous

# Search and propose
ccswarm search mdn "react server components"
ccswarm search github "rust async patterns"
ccswarm extend propose --title "Add RSC Support"

# View status
ccswarm extend status
ccswarm extend stats
```

## Diagnostics

```bash
# System doctor
ccswarm doctor
ccswarm doctor --fix
ccswarm doctor --check sessions

# Debug logging
RUST_LOG=debug ccswarm start
RUST_LOG=ccswarm::session=trace ccswarm start
```

## See Also

- [Getting Started](GETTING_STARTED.md)
- [Configuration](CONFIGURATION.md)
- [Claude ACP Guide](CLAUDE_ACP.md)
- [Troubleshooting](TROUBLESHOOTING.md)
