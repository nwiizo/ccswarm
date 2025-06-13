# ccswarm start

Start the ccswarm orchestrator to manage AI agents and coordinate tasks.

## Description

The `start` command launches the ccswarm orchestrator, which manages all AI agents, handles task delegation, maintains sessions, and coordinates inter-agent communication. It runs as a daemon process by default.

## Usage

```bash
ccswarm start [OPTIONS]
```

## Options

- `--config <FILE>` - Path to configuration file (default: ./ccswarm.json)
- `--foreground` - Run in foreground instead of daemon mode
- `--port <PORT>` - API port for orchestrator (default: 8080)
- `--no-auto-accept` - Disable auto-accept mode globally
- `--quality-review-interval <SECONDS>` - Quality review interval (default: 30)
- `--max-agents <NUMBER>` - Maximum concurrent agents (default: 10)
- `--verbose` - Enable verbose logging
- `--debug` - Enable debug mode with detailed output

## Environment Variables

- `RUST_LOG` - Control logging level (debug, info, warn, error)
- `ANTHROPIC_API_KEY` - API key for Claude providers
- `OPENAI_API_KEY` - API key for OpenAI providers
- `CCSWARM_HOME` - Override default ccswarm directory

## Examples

### Basic start
```bash
ccswarm start
```

### Run in foreground with verbose output
```bash
ccswarm start --foreground --verbose
```

### Custom configuration file
```bash
ccswarm start --config ./custom-config.json
```

### Debug mode with detailed logging
```bash
RUST_LOG=debug ccswarm start --debug
```

### Custom quality review interval (60 seconds)
```bash
ccswarm start --quality-review-interval 60
```

### Limited agent pool
```bash
ccswarm start --max-agents 5
```

## What Happens on Start

1. **Configuration Loading**
   - Loads ccswarm.json configuration
   - Validates agent definitions
   - Checks provider availability

2. **Session Initialization**
   - Restores persistent sessions
   - Creates session pools per role
   - Initializes worktree management

3. **Agent Startup**
   - Spawns tmux sessions for agents
   - Validates provider connections
   - Sets up monitoring streams

4. **Orchestrator Services**
   - Starts task delegation service
   - Initializes quality review timer
   - Opens coordination bus
   - Starts API server

5. **Monitoring Setup**
   - Begins real-time output streaming
   - Starts metrics collection
   - Initializes audit logging

## Process Management

### Check if running
```bash
ccswarm status
```

### View logs
```bash
ccswarm logs --follow
```

### Stop orchestrator
```bash
ccswarm stop
```

## Troubleshooting

### Port already in use
```bash
# Use different port
ccswarm start --port 8081
```

### Session restoration fails
```bash
# Clear sessions and restart
ccswarm session clear
ccswarm start
```

### Provider connection issues
```bash
# Verify API keys
echo $ANTHROPIC_API_KEY
# Start with debug logging
RUST_LOG=ccswarm::provider=debug ccswarm start
```

## Related Commands

- [`stop`](stop.md) - Stop the running orchestrator
- [`status`](status.md) - Check orchestrator status
- [`tui`](tui.md) - Monitor with Terminal UI
- [`logs`](logs.md) - View orchestrator logs

## Notes

- The orchestrator runs with `--dangerously-skip-permissions` by default
- Quality reviews happen automatically every 30 seconds
- Sessions are persisted between restarts
- Graceful shutdown preserves agent states