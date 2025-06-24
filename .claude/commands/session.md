# ccswarm session

AI-powered session management with 93% token savings.

## Usage
```bash
ccswarm session <SUBCOMMAND>
```

## Subcommands
- `list` - List all active sessions
- `create` - Create a new session
- `attach` - Attach to existing session
- `stats` - View session statistics
- `cleanup` - Clean up stuck sessions
- `pause` - Pause a session
- `resume` - Resume a paused session

## Description
Manages AI-powered terminal sessions that persist conversation history across tasks, providing massive token savings through intelligent context compression and reuse.

## Examples

### List Sessions
```bash
$ ccswarm session list
📍 Active AI Sessions
════════════════════════════════════════════

ID: frontend-a1b2c3
Agent: Frontend Specialist
Status: Active ✅
Tasks Completed: 12
Token Savings: 94.2% (saved 45,320 tokens)
Created: 2024-06-24 10:30:00

ID: backend-d4e5f6
Agent: Backend Specialist
Status: Idle 💤
Tasks Completed: 8
Token Savings: 92.8% (saved 38,150 tokens)
Created: 2024-06-24 10:30:15
```

### View Statistics
```bash
$ ccswarm session stats --show-savings

📊 Session Statistics
════════════════════════════════════════════

Total Sessions: 4
Active: 2 | Idle: 1 | Paused: 1

💰 Token Savings Summary:
  Total Tokens Used: 5,420
  Tokens Without Sessions: 83,470
  Total Savings: 78,050 tokens (93.5%)
  Estimated Cost Savings: $15.61

📈 Performance Metrics:
  Average Response Time: 1.2s
  Context Reuse Rate: 94%
  Session Recovery Success: 100%
```

### Create New Session
```bash
$ ccswarm session create --agent frontend --enable-ai-features
✅ Created session: frontend-g7h8i9
   Agent: Frontend Specialist
   Features: AI-enabled ✨
   Ready for tasks!
```

### Attach to Session
```bash
$ ccswarm session attach frontend-a1b2c3
Attaching to session frontend-a1b2c3...
[Session active - 50 messages in history]
```

### Clean Up Stuck Sessions
```bash
$ ccswarm session cleanup
🧹 Cleaning up stuck sessions...
  ❌ Removing crashed session: qa-x9y8z7 (inactive 48h)
  ❌ Removing orphaned session: temp-123456 (no agent)
✅ Cleaned up 2 sessions
```

## Features

### Token Savings
- **93% average reduction** in API token usage
- Intelligent conversation history compression
- Context reuse across related tasks
- Automatic pruning of old messages

### Session Persistence
- Survives ccswarm restarts
- Automatic recovery from crashes
- 50-message history window
- JSON-based state storage

### Performance Benefits
- Faster task completion (no context rebuilding)
- Reduced API costs
- Better continuity between tasks
- Improved agent memory

## Session Lifecycle
1. **Creation** - New session with agent assignment
2. **Active** - Processing tasks
3. **Idle** - Waiting for tasks
4. **Paused** - Temporarily suspended
5. **Cleanup** - Automatic removal after inactivity

## Related Commands
- `ccswarm agent list` - View agents and their sessions
- `ccswarm task` - Create tasks for sessions
- `ccswarm tui` - Monitor sessions in real-time