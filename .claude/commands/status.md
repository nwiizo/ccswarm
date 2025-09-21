# ccswarm status

Show the current status of the ccswarm orchestrator and agents.

## Description

The `status` command provides a comprehensive view of the ccswarm system state, including orchestrator health, agent status, task queues, and session information. It's useful for quick health checks and debugging.

## Usage

```bash
ccswarm status [OPTIONS]
```

## Options

- `--detailed` - Show detailed status information
- `--json` - Output in JSON format
- `--agents` - Show only agent status
- `--tasks` - Show only task status
- `--sessions` - Show only session status
- `--metrics` - Include performance metrics
- `--health` - Perform health checks
- `--watch` - Continuously update status

## Examples

### Basic status
```bash
ccswarm status
```

Output:
```
ccswarm Status: Running
Uptime: 2h 15m 30s
Agents: 4 active, 0 paused
Tasks: 3 queued, 2 in progress, 15 completed
Sessions: 4 active, 12 total
```

### Detailed status
```bash
ccswarm status --detailed
```

### JSON output for scripting
```bash
ccswarm status --json | jq '.agents'
```

### Watch mode (updates every 2 seconds)
```bash
ccswarm status --watch
```

### Agent-specific status
```bash
ccswarm status --agents
```

### Health check
```bash
ccswarm status --health
```

## Status Information

### Orchestrator Status
- **State**: Running, Paused, Stopped, Error
- **Uptime**: Time since start
- **PID**: Process ID
- **Port**: API port
- **Version**: ccswarm version

### Agent Information
- **Name**: Agent identifier
- **Role**: Frontend, Backend, DevOps, QA
- **Status**: Active, Paused, Error, Offline
- **Provider**: claude_code, aider, etc.
- **Current Task**: Active task if any
- **Session**: Session ID

### Task Metrics
- **Queued**: Tasks waiting for agents
- **In Progress**: Currently executing
- **Completed**: Successfully finished
- **Failed**: Tasks with errors
- **Average Time**: Task completion time

### Session Stats
- **Active**: Currently in use
- **Idle**: Available for reuse
- **Total**: All sessions created
- **Reuse Rate**: Session efficiency

### Performance Metrics
- **CPU Usage**: Orchestrator CPU
- **Memory**: RAM usage
- **API Calls**: Provider API usage
- **Token Usage**: LLM token consumption
- **Cache Hits**: Session cache efficiency

## Output Formats

### Default Format
```
ccswarm Status: Running
Uptime: 2h 15m 30s

Agents (4 active):
  ✓ frontend-specialist [Frontend] - Active (Task: Create navbar)
  ✓ backend-api [Backend] - Active (Task: Add auth endpoint)
  ✓ devops-expert [DevOps] - Idle
  ⏸ qa-tester [QA] - Paused

Tasks:
  Queued: 3 | In Progress: 2 | Completed: 15 | Failed: 0

Sessions:
  Active: 4 | Idle: 8 | Reuse Rate: 87%
```

### JSON Format
```json
{
  "orchestrator": {
    "status": "running",
    "uptime_seconds": 8130,
    "pid": 12345,
    "version": "0.2.0"
  },
  "agents": [
    {
      "name": "frontend-specialist",
      "role": "Frontend",
      "status": "active",
      "current_task": "Create navbar",
      "session_id": "sess_123"
    }
  ],
  "tasks": {
    "queued": 3,
    "in_progress": 2,
    "completed": 15,
    "failed": 0
  }
}
```

## Health Checks

The `--health` option performs:

1. **API Connectivity** - Can reach orchestrator API
2. **Agent Health** - All agents responding
3. **Provider Status** - API keys valid
4. **Session Health** - Sessions accessible
5. **Disk Space** - Adequate space for worktrees
6. **Memory Usage** - Within acceptable limits

## Exit Codes

- `0` - System healthy and running
- `1` - System not running
- `2` - System degraded (some components failing)
- `3` - System error (critical failure)

## Monitoring Scripts

### Check if running
```bash
if ccswarm status >/dev/null 2>&1; then
    echo "ccswarm is running"
else
    echo "ccswarm is not running"
fi
```

### Monitor task completion
```bash
watch -n 5 'ccswarm status --tasks'
```

### Alert on errors
```bash
ccswarm status --json | jq -e '.tasks.failed > 0' && \
    echo "Failed tasks detected!"
```

## Related Commands

- [`start`](start.md) - Start the orchestrator
- [`stop`](stop.md) - Stop the orchestrator
- [`tui`](tui.md) - Interactive monitoring
- [`agents`](agents.md) - Detailed agent info
- [`session`](session.md) - Session management

## Notes

- Status checks are non-blocking
- Cached for 2 seconds to reduce overhead
- Health checks may take longer
- JSON output includes all available data