# Claude ACP Integration Guide

> ⚠️ **STATUS: STUB IMPLEMENTATION - NOT FUNCTIONAL**
>
> This document describes the **planned** ACP integration architecture.
>
> **Current reality:**
> - ✅ SimplifiedClaudeAdapter exists (CLI wrapper in `src/acp_claude/`)
> - ❌ WebSocket implementation missing
> - ❌ CLI commands don't exist (`ccswarm claude-acp test/start/send/status/diagnose`)
> - ❌ Not integrated with orchestrator
> - ❌ Provider system not wired up
>
> See [UPCOMING_FEATURES.md](UPCOMING_FEATURES.md) for v0.4.0 integration roadmap.

---

ccswarm integrates with Claude Code via the Agent Client Protocol (ACP) over WebSocket.

## Overview

- **Connection**: WebSocket on `ws://localhost:9100`
- **Authentication**: Uses existing Claude Code CLI session (`~/.claude/config.json`)
- **No API Key Required**: Works with Pro/Max subscription

## Bridge Setup

Claude Code ACP adapters use stdio, so a WebSocket bridge is needed:

```bash
# Install the WebSocket bridge
npm install -g servep

# Install Claude Code ACP adapter
npm install -g acp-claude-code

# Start the bridge (Terminal 1)
servep -p 9100 --ws "/::npx acp-claude-code"

# Test connection (Terminal 2)
ccswarm claude-acp test
```

### Authentication

The `acp-claude-code` adapter uses your existing Claude Code CLI session. If not logged in:

```bash
claude login
```

## CLI Commands

### Basic Operations

```bash
# Test Claude Code connection
ccswarm claude-acp test

# Start ACP adapter
ccswarm claude-acp start

# Send task to Claude Code
ccswarm claude-acp send --task "Analyze code for improvements"

# Check connection status
ccswarm claude-acp status

# Run diagnostics
ccswarm claude-acp diagnose
```

### Development Usage

```bash
# From workspace root (cargo run)
cargo run -p ccswarm -- claude-acp test
cargo run -p ccswarm -- claude-acp send --task "Review this codebase"

# After installation
ccswarm claude-acp test
ccswarm claude-acp send --task "Review this codebase"
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CCSWARM_CLAUDE_ACP_URL` | `ws://localhost:9100` | WebSocket endpoint |
| `CCSWARM_CLAUDE_ACP_AUTO_CONNECT` | `true` | Auto-connect on startup |
| `CCSWARM_CLAUDE_ACP_TIMEOUT` | `30` | Connection timeout (seconds) |
| `CCSWARM_CLAUDE_ACP_MAX_RETRIES` | `3` | Max reconnection attempts |
| `CCSWARM_CLAUDE_ACP_PREFER_CLAUDE` | `true` | Prefer Claude Code provider |
| `CCSWARM_CLAUDE_ACP_DEBUG` | `false` | Enable debug logging |

## Architecture

```
┌─────────────────┐     WebSocket      ┌─────────────────┐     stdio      ┌─────────────────┐
│    ccswarm      │ ◄─────────────────► │     servep      │ ◄────────────► │ acp-claude-code │
│   (client)      │    ws://9100       │    (bridge)     │                │   (adapter)     │
└─────────────────┘                    └─────────────────┘                └─────────────────┘
                                                                                  │
                                                                                  ▼
                                                                         ┌─────────────────┐
                                                                         │  Claude Code    │
                                                                         │    CLI/API      │
                                                                         └─────────────────┘
```

## Module Structure

```
crates/ccswarm/src/acp_claude/
├── mod.rs        # Module exports
├── adapter.rs    # WebSocket adapter implementation
├── config.rs     # Configuration management
└── error.rs      # Error handling
```

## Troubleshooting

### Connection Failed

```bash
# Check if bridge is running
curl -I http://localhost:9100

# Restart bridge
pkill -f servep
servep -p 9100 --ws "/::npx acp-claude-code"

# Run diagnostics
ccswarm claude-acp diagnose
```

### Authentication Issues

```bash
# Re-authenticate Claude Code
claude logout
claude login

# Verify session
cat ~/.claude/config.json | jq .sessionKey
```

### Debug Mode

```bash
# Enable verbose logging
CCSWARM_CLAUDE_ACP_DEBUG=true RUST_LOG=ccswarm::acp_claude=debug ccswarm claude-acp test
```

## Sample Demo

```bash
cd sample/
./setup.sh
./claude_acp_demo.sh
```

See also: [sample/ccswarm.yaml](../sample/ccswarm.yaml) for configuration examples.
