# Troubleshooting Guide

Solutions for common ccswarm issues.

## Quick Diagnostics

```bash
# Run comprehensive diagnostics
ccswarm doctor

# Auto-fix common issues
ccswarm doctor --fix

# Check specific components
ccswarm doctor --check sessions
ccswarm doctor --check providers
ccswarm doctor --check worktrees
ccswarm doctor --check environment
```

## Common Issues

### Session Issues

#### "Session not found"

```bash
# List all sessions
ccswarm session list

# Create a new session
ccswarm session create --agent frontend

# Check if session was terminated
ccswarm session stats --show-all
```

**Error Code:** `SES001`

#### Session Won't Start

```bash
# Check for resource issues
ccswarm health --resources

# Clear stale sessions
ccswarm session cleanup

# Force restart
ccswarm session reset --agent frontend
```

### Provider Issues

#### "Provider error" / Connection Failed

```bash
# Check if ACP bridge is running
ss -tlnp | grep 9100
# or
curl -I http://localhost:9100

# Start the bridge if not running
servep -p 9100 --ws "/::npx acp-claude-code"

# Run ACP diagnostics
ccswarm claude-acp diagnose
```

#### Claude Code Authentication

```bash
# Check Claude Code login status
claude /login

# Re-authenticate if needed
claude logout
claude login

# Verify session exists
cat ~/.claude/config.json | jq .sessionKey
```

#### API Key Issues

```bash
# Verify API key is set
echo $ANTHROPIC_API_KEY

# Check configuration
ccswarm config show

# Test provider connections
ccswarm config test-providers
```

**Error Code:** `PRV001`

### Git Worktree Issues

#### "Worktree conflicts"

```bash
# List all worktrees
ccswarm worktree list

# Clean up unused worktrees
ccswarm worktree clean

# Reset specific agent worktree
ccswarm worktree reset --agent frontend

# Force cleanup (use with caution)
ccswarm worktree clean --force
```

#### Worktree Locked

```bash
# Check for lock files
ls -la .git/worktrees/*/locked

# Remove stale locks
ccswarm worktree unlock --agent frontend

# Or manually
rm .git/worktrees/frontend-agent/locked
```

**Error Code:** `GIT001`

### Task Issues

#### Task Not Delegating

```bash
# Check agent availability
ccswarm agent list --status

# Verify task format
ccswarm task "Description [high] [feature]"

# Force delegation
ccswarm delegate task "Description" --agent backend --force
```

#### Tasks Stuck in Queue

```bash
# View task queue
ccswarm task list --status pending

# Check for blockers
ccswarm deps analyze --show-blockers

# Clear stuck tasks
ccswarm task clear --status stuck
```

### Configuration Issues

#### "Configuration Error"

```bash
# Validate configuration
ccswarm config validate

# Show current config
ccswarm config show

# Reinitialize if corrupt
ccswarm init --name "ProjectName" --force
```

**Error Code:** `CFG001`

#### Missing Required Fields

The error message will indicate which field is missing:

```
Missing required field: project.name

Quick Fix:
  ccswarm init --name "YourProject"
```

### Performance Issues

#### High Memory Usage

```bash
# Check resource usage
ccswarm health --resources --format json

# Compress sessions
ccswarm session compress --all --threshold 0.8

# Clear old sessions
ccswarm session cleanup --older-than 24h
```

#### Slow Response Times

```bash
# Enable debug logging
RUST_LOG=debug ccswarm start

# Check network latency
ccswarm claude-acp test --verbose

# Optimize sessions
ccswarm session optimize --all
```

### TUI Issues

#### TUI Not Rendering Correctly

```bash
# Reset terminal
reset

# Check terminal capabilities
echo $TERM

# Try different terminal
TERM=xterm-256color ccswarm tui
```

#### TUI Crashes

```bash
# Run with debug logging
RUST_LOG=ccswarm::tui=debug ccswarm tui

# Check for terminal size issues (min 80x24)
stty size
```

## Error Codes Reference

| Code | Category | Description |
|------|----------|-------------|
| `SES001` | Session | Session not found |
| `SES002` | Session | Session creation failed |
| `SES003` | Session | Session timeout |
| `PRV001` | Provider | Provider connection failed |
| `PRV002` | Provider | Authentication failed |
| `PRV003` | Provider | Provider not configured |
| `GIT001` | Git | Worktree conflict |
| `GIT002` | Git | Worktree locked |
| `GIT003` | Git | Branch not found |
| `CFG001` | Config | Invalid configuration |
| `CFG002` | Config | Missing required field |
| `TSK001` | Task | Task creation failed |
| `TSK002` | Task | Delegation failed |
| `ACP001` | ACP | Bridge not running |
| `ACP002` | ACP | WebSocket connection failed |

## Debug Mode

### Enable Verbose Logging

```bash
# All components
RUST_LOG=debug ccswarm start

# Specific module
RUST_LOG=ccswarm::session=trace ccswarm start

# Multiple modules
RUST_LOG=ccswarm::session=trace,ccswarm::acp_claude=debug ccswarm start
```

### ACP Debug

```bash
CCSWARM_CLAUDE_ACP_DEBUG=true RUST_LOG=ccswarm::acp_claude=debug ccswarm claude-acp test
```

## Recovery Procedures

### Full System Reset

```bash
# Stop all processes
ccswarm stop --force

# Clean up sessions
ccswarm session cleanup --all

# Clean up worktrees
ccswarm worktree clean --all

# Reinitialize
ccswarm init --name "ProjectName" --force
```

### Recover from Crash

```bash
# Check system state
ccswarm doctor

# Auto-fix issues
ccswarm doctor --fix

# Restart with recovery
ccswarm start --recover
```

## Getting More Help

### Built-in Help

```bash
# Topic-specific help
ccswarm help sessions
ccswarm help configuration
ccswarm help --search "error"
```

### Contextual Error Messages

ccswarm provides smart error messages:

```
Cannot connect to Claude Code

Possible Causes:
1. ACP bridge not running
2. Claude Code not authenticated
3. Network issues

Try This:
1. servep -p 9100 --ws "/::npx acp-claude-code"
2. claude login
3. ccswarm claude-acp diagnose

Error Code: ACP001
```

### Report Issues

If you can't resolve the issue:

1. Run diagnostics: `ccswarm doctor --export diagnostics.json`
2. Collect logs: `RUST_LOG=debug ccswarm start 2>&1 | tee ccswarm.log`
3. Open issue: https://github.com/nwiizo/ccswarm/issues

Include:
- Error message and code
- ccswarm version (`ccswarm --version`)
- OS and Rust version
- Diagnostic output
- Steps to reproduce

## See Also

- [Getting Started](GETTING_STARTED.md) - Setup guide
- [Configuration](CONFIGURATION.md) - Config reference
- [Commands Reference](COMMANDS.md) - CLI commands
- [Claude ACP Guide](CLAUDE_ACP.md) - ACP troubleshooting
