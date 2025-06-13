# ccswarm session

Manage persistent AI agent sessions for token efficiency and context preservation.

## Description

The `session` command manages persistent conversation sessions that dramatically reduce token usage (up to 93%) by preserving conversation history and context between tasks. Sessions enable agents to maintain context across multiple tasks and restarts.

## Usage

```bash
ccswarm session [SUBCOMMAND] [OPTIONS]
```

## Subcommands

- `list` - List all sessions (default)
- `show <ID>` - Show session details
- `create` - Create a new session
- `attach <ID>` - Attach to existing session
- `pause <ID>` - Pause a session
- `resume <ID>` - Resume a paused session
- `clear` - Clear idle sessions
- `stats` - Show session statistics
- `export <ID>` - Export session history
- `import` - Import session from file

## Options

### For `list`
- `--active` - Show only active sessions
- `--idle` - Show only idle sessions
- `--agent <NAME>` - Filter by agent
- `--format <FORMAT>` - Output format (table, json)

### For `create`
- `--agent <NAME>` - Assign to agent
- `--role <ROLE>` - Session role
- `--pool` - Add to session pool

### For `clear`
- `--older-than <TIME>` - Clear sessions older than
- `--idle-only` - Only clear idle sessions
- `--force` - Skip confirmation

### For `stats`
- `--period <TIME>` - Time period for stats
- `--by-agent` - Group by agent
- `--show-savings` - Show token savings

## Examples

### List all sessions
```bash
ccswarm session list
```

Output:
```
ID              Agent               Role      Status   Messages  Token Savings
sess_abc123     frontend-specialist Frontend  Active   127       89%
sess_def456     backend-api        Backend   Active   203       92%
sess_ghi789     devops-expert      DevOps    Idle     45        85%
sess_jkl012     -                  QA        Pool     0         -
```

### Show session details
```bash
ccswarm session show sess_abc123
```

### Create new session
```bash
ccswarm session create --agent frontend-specialist --role Frontend
```

### Attach to session
```bash
ccswarm session attach sess_abc123
```

### View session statistics
```bash
ccswarm session stats --show-savings
```

## Session Lifecycle

### 1. Creation
Sessions are created automatically when agents start tasks:
```bash
# Automatic creation
ccswarm task "Create login form"  # Creates session if needed

# Manual creation
ccswarm session create --role Frontend --pool
```

### 2. Active Use
During task execution:
- Conversation history preserved
- Context maintained between tasks
- Token usage optimized

### 3. Idle State
After task completion:
- Session enters idle state
- Available for reuse
- History preserved

### 4. Reuse
When new tasks arrive:
- Matching idle session selected
- Context restored
- Continues from previous state

### 5. Cleanup
Periodic maintenance:
```bash
# Clear old sessions
ccswarm session clear --older-than 7d

# Clear all idle sessions
ccswarm session clear --idle-only
```

## Session Pooling

### Pool Management
```bash
# View pool status
ccswarm session list --pool

# Add sessions to pool
ccswarm session create --role Frontend --pool --count 3

# Pool statistics
ccswarm session stats --pool
```

### Pool Configuration
```json
{
  "session_management": {
    "persistent_sessions": true,
    "max_sessions_per_role": 3,
    "pool_config": {
      "min_idle": 1,
      "max_idle": 5,
      "idle_timeout": "30m"
    }
  }
}
```

## Token Savings Analysis

### View savings
```bash
ccswarm session stats --show-savings --period 30d
```

Output:
```
Session Token Savings (Last 30 days)
===================================

Total Sessions: 47
Total Messages: 12,847
Total Tokens Without Sessions: 2,456,000
Total Tokens With Sessions: 184,200
Total Savings: 2,271,800 tokens (92.5%)

Cost Savings: $68.15

By Role:
- Frontend: 89% reduction (saved 567,000 tokens)
- Backend: 93% reduction (saved 823,000 tokens)
- DevOps: 91% reduction (saved 445,000 tokens)
- QA: 87% reduction (saved 436,800 tokens)
```

### Export detailed metrics
```bash
ccswarm session stats --export metrics.csv
```

## Session History

### Export session
```bash
ccswarm session export sess_abc123 --output session-backup.json
```

### Import session
```bash
ccswarm session import --file session-backup.json --agent frontend-specialist
```

### View conversation history
```bash
ccswarm session show sess_abc123 --history --limit 50
```

## Advanced Features

### Session Templates
Create reusable session templates:
```bash
# Create template
ccswarm session create --template react-expert \
  --preload "You are a React specialist with expertise in hooks and performance"

# Use template
ccswarm session create --from-template react-expert --agent frontend-specialist
```

### Session Merging
Combine multiple sessions:
```bash
ccswarm session merge sess_abc123 sess_def456 --output sess_merged
```

### Batch Operations
```bash
# Pause all idle sessions
ccswarm session list --idle --format json | \
  jq -r '.[] | .id' | \
  xargs -I {} ccswarm session pause {}

# Resume sessions for specific role
ccswarm session resume --role Frontend --all
```

## Session Configuration

### Message Retention
```json
{
  "session_management": {
    "message_retention": {
      "max_messages": 200,
      "max_age": "7d",
      "compression": true
    }
  }
}
```

### Context Window Management
```json
{
  "session_management": {
    "context_window": {
      "size": 100000,
      "strategy": "sliding",
      "preserve_important": true
    }
  }
}
```

## Troubleshooting

### Session not found
```bash
# List all sessions
ccswarm session list --all

# Check session file exists
ls .ccswarm/sessions/
```

### Session corruption
```bash
# Validate session
ccswarm session validate sess_abc123

# Repair if possible
ccswarm session repair sess_abc123
```

### High token usage
```bash
# Check session efficiency
ccswarm session stats --agent backend-api --show-efficiency

# Optimize session
ccswarm session optimize sess_abc123
```

## Best Practices

1. **Regular Cleanup** - Clear old sessions weekly
2. **Pool Sizing** - Maintain 1-2 idle sessions per role
3. **History Limits** - Set reasonable message retention
4. **Export Important Sessions** - Backup critical context
5. **Monitor Savings** - Track token reduction metrics

## Integration

### With Tasks
```bash
# Task automatically uses best session
ccswarm task "Implement feature" --prefer-session sess_abc123
```

### With Agents
```bash
# Agent session assignment
ccswarm agents update frontend-specialist --session sess_abc123
```

## Related Commands

- [`agents`](agents.md) - View agent-session mappings
- [`task`](task.md) - Tasks use sessions automatically
- [`stats`](status.md) - Overall system statistics
- [`config`](config.md) - Configure session settings

## Notes

- Sessions persist across orchestrator restarts
- Each session maintains full conversation history
- Token savings increase with session reuse
- Sessions are compressed for storage efficiency
- Critical for cost-effective AI agent operation