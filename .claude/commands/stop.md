# ccswarm stop

Stop the running ccswarm orchestrator gracefully.

## Description

The `stop` command gracefully shuts down the ccswarm orchestrator, ensuring all agents complete their current tasks, sessions are saved, and system state is preserved for the next startup.

## Usage

```bash
ccswarm stop [OPTIONS]
```

## Options

- `--force` - Force immediate shutdown without waiting for tasks
- `--timeout <SECONDS>` - Maximum wait time for graceful shutdown (default: 30)
- `--save-state` - Explicitly save all state before stopping (default: true)
- `--no-save-state` - Skip state saving for faster shutdown
- `--agent <NAME>` - Stop specific agent only
- `--all` - Stop all ccswarm instances on the system

## Examples

### Graceful stop (default)
```bash
ccswarm stop
```

### Force immediate shutdown
```bash
ccswarm stop --force
```

### Stop with extended timeout
```bash
ccswarm stop --timeout 60
```

### Stop without saving state
```bash
ccswarm stop --no-save-state
```

### Stop specific agent
```bash
ccswarm stop --agent frontend-specialist
```

### Stop all instances
```bash
ccswarm stop --all
```

## Shutdown Process

1. **Signal Agents**
   - Sends graceful shutdown signal to all agents
   - Agents complete current operations
   - No new tasks are accepted

2. **Save Sessions**
   - Persists conversation history
   - Saves session state and context
   - Updates session metadata

3. **Complete Tasks**
   - Waits for in-progress tasks
   - Marks interrupted tasks for resume
   - Saves task queue state

4. **Clean Resources**
   - Closes tmux sessions
   - Releases file locks
   - Cleans up temporary files

5. **Final State Save**
   - Writes orchestrator state
   - Updates status files
   - Logs shutdown metrics

## What Gets Preserved

- Active session conversations
- Task queue and priorities
- Agent configurations
- Worktree states
- Performance metrics
- Quality review history

## Verification

### Check if stopped
```bash
ccswarm status
```

### View shutdown logs
```bash
ccswarm logs --tail 50 | grep shutdown
```

### List orphaned processes
```bash
ps aux | grep ccswarm
tmux ls | grep ccswarm
```

## Force Cleanup

If graceful stop fails:

```bash
# Kill all ccswarm processes
pkill -f ccswarm

# Clean up tmux sessions
tmux kill-session -t ccswarm-frontend 2>/dev/null
tmux kill-session -t ccswarm-backend 2>/dev/null

# Remove lock files
rm -f .ccswarm/orchestrator.lock
```

## Restart After Stop

```bash
# Normal restart
ccswarm start

# Clean restart (new sessions)
ccswarm start --clean

# Resume with saved state
ccswarm start --resume
```

## Related Commands

- [`start`](start.md) - Start the orchestrator
- [`status`](status.md) - Check if running
- [`session`](session.md) - Manage saved sessions

## Notes

- Graceful shutdown preserves all work
- Force shutdown may lose in-flight tasks
- Sessions are automatically restored on restart
- Quality reviews are suspended during shutdown
- Audit trail logs all shutdown events