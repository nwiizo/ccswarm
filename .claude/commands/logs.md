# ccswarm logs

Show logs from the ccswarm orchestrator and agents.

## Description

The `logs` command provides access to various log streams from the ccswarm system, including orchestrator logs, agent outputs, session histories, and debug information. It supports real-time streaming, filtering, and search capabilities.

## Usage

```bash
ccswarm logs [OPTIONS]
```

## Options

- `--follow` or `-f` - Stream logs in real-time
- `--tail <NUM>` or `-n` - Show last N lines (default: 50)
- `--since <TIME>` - Show logs since timestamp
- `--until <TIME>` - Show logs until timestamp
- `--agent <NAME>` - Show logs from specific agent
- `--level <LEVEL>` - Filter by log level
- `--filter <PATTERN>` - Filter by regex pattern
- `--format <FORMAT>` - Output format (text, json)
- `--source <SOURCE>` - Log source (orchestrator, agent, session)
- `--no-color` - Disable colored output

## Log Levels

- `error` - Error messages only
- `warn` - Warnings and above
- `info` - Informational messages and above (default)
- `debug` - Debug messages and above
- `trace` - All messages including trace

## Examples

### View recent logs
```bash
ccswarm logs
```

### Stream logs in real-time
```bash
ccswarm logs --follow
```

### Show last 100 lines
```bash
ccswarm logs --tail 100
```

### Filter by agent
```bash
ccswarm logs --agent frontend-specialist
```

### Filter by log level
```bash
ccswarm logs --level error
```

### Complex filtering
```bash
ccswarm logs --filter "error|warning" --agent backend-api --follow
```

### Time-based queries
```bash
# Last hour
ccswarm logs --since "1 hour ago"

# Specific time range
ccswarm logs --since "2024-01-15T10:00:00" --until "2024-01-15T11:00:00"

# Today's logs
ccswarm logs --since "today"
```

## Log Sources

### Orchestrator Logs
```bash
ccswarm logs --source orchestrator
```
Includes:
- Task delegation decisions
- Quality review results
- Session management
- System events

### Agent Logs
```bash
ccswarm logs --source agent --agent frontend-specialist
```
Includes:
- Task execution output
- Provider interactions
- Error messages
- Performance metrics

### Session Logs
```bash
ccswarm logs --source session
```
Includes:
- Conversation history
- Token usage
- Session lifecycle events

## Output Formats

### Default Text Format
```
2024-01-15 10:23:45 [INFO] orchestrator: Task delegation started
2024-01-15 10:23:46 [DEBUG] orchestrator: Analyzing task complexity
2024-01-15 10:23:47 [INFO] frontend-specialist: Received task: Create navbar
2024-01-15 10:23:48 [WARN] backend-api: High memory usage detected
```

### JSON Format
```bash
ccswarm logs --format json --tail 5
```
```json
{
  "timestamp": "2024-01-15T10:23:45Z",
  "level": "INFO",
  "source": "orchestrator",
  "message": "Task delegation started",
  "metadata": {
    "task_id": "task_123",
    "agent": "frontend-specialist"
  }
}
```

## Advanced Filtering

### Multiple filters
```bash
# Errors from specific agent
ccswarm logs --agent backend-api --level error

# Pattern matching
ccswarm logs --filter "authentication|authorization"

# Exclude patterns
ccswarm logs --filter "^(?!.*test).*$"  # Exclude test logs
```

### Log aggregation
```bash
# Count errors by agent
ccswarm logs --format json --level error | \
  jq -r '.agent' | sort | uniq -c

# Extract task IDs
ccswarm logs --filter "task_[0-9]+" -o
```

## Log Files

### Default locations
```
.ccswarm/
├── logs/
│   ├── orchestrator.log      # Main orchestrator log
│   ├── agents/               # Agent-specific logs
│   │   ├── frontend.log
│   │   └── backend.log
│   ├── sessions/             # Session logs
│   ├── quality-review.log    # Review system logs
│   └── debug.log            # Debug information
```

### Direct file access
```bash
# Tail orchestrator log file
tail -f .ccswarm/logs/orchestrator.log

# Search in all logs
grep -r "error" .ccswarm/logs/
```

## Log Rotation

### Configuration
```json
{
  "logging": {
    "max_size": "100MB",
    "max_files": 10,
    "compress": true,
    "rotate_daily": true
  }
}
```

### Manual rotation
```bash
ccswarm logs rotate
```

## Integration

### Export logs
```bash
# Export last hour to file
ccswarm logs --since "1 hour ago" > ccswarm-logs.txt

# Export as JSON for analysis
ccswarm logs --format json --since "today" > logs.json
```

### Stream to external service
```bash
# Stream to logging service
ccswarm logs --follow --format json | \
  curl -X POST https://logs.example.com/ingest \
    -H "Content-Type: application/json" \
    --data-binary @-
```

## Debugging

### Enable debug logging
```bash
# Via environment variable
RUST_LOG=debug ccswarm start

# For specific module
RUST_LOG=ccswarm::orchestrator=debug ccswarm start
```

### Trace logging
```bash
# Maximum verbosity
RUST_LOG=trace ccswarm start

# Trace specific agent
ccswarm logs --agent frontend-specialist --level trace
```

## Performance Considerations

### Log verbosity impact
- `info` - Minimal impact (<1% overhead)
- `debug` - Moderate impact (2-5% overhead)
- `trace` - Significant impact (5-10% overhead)

### Optimize log queries
```bash
# Use time ranges
ccswarm logs --since "10 minutes ago" --until "5 minutes ago"

# Limit output
ccswarm logs --tail 100 --no-follow

# Filter at source
ccswarm logs --level error --agent backend-api
```

## Related Commands

- [`status`](status.md) - Quick system status
- [`monitor`](tui.md) - Real-time monitoring
- [`agents`](agents.md) - Agent-specific information
- [`review`](review.md) - Quality review logs

## Notes

- Logs are automatically rotated at 100MB
- Sensitive information is redacted by default
- Log retention is 30 days by default
- Real-time streaming uses minimal resources
- JSON format enables advanced analysis