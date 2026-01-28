# AI-Session CLI Guide

## Overview

The `ai-session` CLI provides a command-line interface for managing AI-optimized terminal sessions. It serves as a drop-in replacement for tmux with enhanced features for AI agents and modern development workflows.

## Installation

```bash
# Build from source
git clone https://github.com/nwiizo/ccswarm.git
cd ccswarm/ai-session
cargo build --release

# The binary will be available at:
# target/release/ai-session
```

## Quick Start

```bash
# Create a new session
ai-session create --name "my-dev-session" --ai-context

# List active sessions
ai-session list

# Attach to a session
ai-session attach <session-id>

# Execute command in session
ai-session exec <session-id> "npm run dev"

# Kill a session
ai-session kill <session-id>
```

## Command Reference

### Global Options

```bash
ai-session [OPTIONS] <COMMAND>

OPTIONS:
    -h, --help      Print help information
    -V, --version   Print version information
```

### Commands

#### `create` - Create a new session

Create a new AI-optimized terminal session.

```bash
ai-session create [OPTIONS]

OPTIONS:
    -n, --name <NAME>                Session name (optional)
    -d, --dir <DIRECTORY>           Working directory [default: current dir]
    -s, --shell <SHELL>             Shell to use [default: $SHELL]
    -c, --ai-context                Enable AI context features
    -t, --token-limit <TOKENS>      AI context token limit [default: 4096]
    -r, --role <ROLE>               Agent role (frontend, backend, devops, qa)
    -e, --env <KEY=VALUE>...        Environment variables

EXAMPLES:
    # Basic session
    ai-session create
    
    # Named session with AI features
    ai-session create --name "frontend-dev" --ai-context --role frontend
    
    # Session in specific directory
    ai-session create --dir "/path/to/project" --shell "/bin/zsh"
    
    # Session with environment variables
    ai-session create --env "NODE_ENV=development" --env "PORT=3000"
    
    # High-capacity AI session
    ai-session create --ai-context --token-limit 8192
```

#### `list` - List active sessions

Display all active sessions with their status and information.

```bash
ai-session list [OPTIONS]

OPTIONS:
    -d, --detailed      Show detailed session information
    -f, --format <FMT>  Output format [default: table] [possible: table, json, csv]
    -s, --status <STATUS>  Filter by status [possible: running, paused, error]

EXAMPLES:
    # Simple list
    ai-session list
    
    # Detailed information
    ai-session list --detailed
    
    # JSON output
    ai-session list --format json
    
    # Only running sessions
    ai-session list --status running
```

**Sample Output:**
```
Active sessions:
ID       Name          Status    Created    Directory         AI Features
a1b2c3d  frontend-dev  Running   10:23:45  /home/user/app   ✓ (2048 tokens)
e4f5g6h  backend-api   Running   10:25:12  /home/user/api   ✓ (4096 tokens)
i7j8k9l  testing       Paused    09:45:33  /home/user/test  ✗
```

#### `attach` - Attach to a session

Connect to an existing session for interactive use.

```bash
ai-session attach <SESSION_ID> [OPTIONS]

OPTIONS:
    -r, --read-only     Attach in read-only mode
    -f, --force         Force attach even if session is busy

ARGUMENTS:
    <SESSION_ID>        Session ID or name to attach to

EXAMPLES:
    # Attach to session by ID
    ai-session attach a1b2c3d
    
    # Attach to session by name
    ai-session attach frontend-dev
    
    # Read-only attachment
    ai-session attach a1b2c3d --read-only
```

#### `exec` - Execute command in session

Execute a command in the specified session and optionally capture output.

```bash
ai-session exec <SESSION_ID> [OPTIONS] <COMMAND>...

OPTIONS:
    -c, --capture       Capture output for AI analysis
    -t, --timeout <SEC> Command timeout in seconds [default: 300]
    -w, --wait          Wait for command completion
    -s, --silent        Don't display output

ARGUMENTS:
    <SESSION_ID>        Session ID or name
    <COMMAND>...        Command to execute

EXAMPLES:
    # Execute simple command
    ai-session exec a1b2c3d "ls -la"
    
    # Execute with output capture
    ai-session exec frontend-dev "npm test" --capture
    
    # Long-running command with timeout
    ai-session exec backend-api "cargo build --release" --timeout 600
    
    # Silent execution
    ai-session exec a1b2c3d "git pull" --silent
```

#### `kill` - Terminate a session

Stop and remove a session.

```bash
ai-session kill <SESSION_ID> [OPTIONS]

OPTIONS:
    -f, --force         Force kill without graceful shutdown
    -a, --all           Kill all sessions
    -s, --status <STATUS>  Kill sessions with specific status

ARGUMENTS:
    <SESSION_ID>        Session ID or name to kill

EXAMPLES:
    # Graceful shutdown
    ai-session kill a1b2c3d
    
    # Force kill
    ai-session kill a1b2c3d --force
    
    # Kill all sessions
    ai-session kill --all
    
    # Kill all error sessions
    ai-session kill --all --status error
```

#### `context` - Show session context and history

Display AI context, command history, and session analytics.

```bash
ai-session context <SESSION_ID> [OPTIONS]

OPTIONS:
    -h, --history <N>       Show last N commands [default: 10]
    -t, --tokens            Show token usage statistics
    -p, --patterns          Show detected output patterns
    -s, --summary           Show context summary
    -e, --export <FILE>     Export context to file

ARGUMENTS:
    <SESSION_ID>            Session ID or name

EXAMPLES:
    # Show context summary
    ai-session context a1b2c3d --summary
    
    # Show command history
    ai-session context frontend-dev --history 20
    
    # Show token usage
    ai-session context a1b2c3d --tokens
    
    # Export full context
    ai-session context a1b2c3d --export context.json
```

**Sample Output:**
```bash
Session Context: a1b2c3d (frontend-dev)
═══════════════════════════════════════

📊 Summary:
   Status: Running
   Uptime: 2h 15m
   Total Commands: 47
   
💰 Token Usage:
   Total Tokens: 3,247 / 4,096 (79%)
   Compression Ratio: 2.3x
   Efficiency: 93% token savings
   
🎯 Recent Commands:
   [10:45:23] npm test
   [10:42:15] git status
   [10:40:33] code src/components/Header.tsx
   
🔍 Detected Patterns:
   - React development workflow
   - TypeScript compilation
   - Jest testing framework
   - Git version control
```

#### `migrate` - Migrate from tmux

Migrate existing tmux sessions to ai-session.

```bash
ai-session migrate [OPTIONS]

OPTIONS:
    -f, --from-tmux         Migrate from tmux
    -s, --session <NAME>    Specific tmux session to migrate
    -a, --all               Migrate all tmux sessions
    -p, --preserve          Keep original tmux sessions
    -d, --dry-run           Show what would be migrated

EXAMPLES:
    # Migrate all tmux sessions
    ai-session migrate --from-tmux --all
    
    # Migrate specific session
    ai-session migrate --from-tmux --session "my-tmux-session"
    
    # Dry run migration
    ai-session migrate --from-tmux --all --dry-run
    
    # Migrate and preserve originals
    ai-session migrate --from-tmux --all --preserve
```

## Configuration

### Environment Variables

```bash
# AI Session configuration
export AI_SESSION_HOME="$HOME/.ai-session"
export AI_SESSION_DEFAULT_SHELL="/bin/zsh"
export AI_SESSION_TOKEN_LIMIT=4096

# Provider configuration
export ANTHROPIC_API_KEY="your-api-key"
export OPENAI_API_KEY="your-api-key"

# Logging
export RUST_LOG="ai_session=info"
export AI_SESSION_LOG_LEVEL="info"
```

### Configuration File

Create `~/.ai-session/config.toml`:

```toml
[defaults]
shell = "/bin/zsh"
ai_context = true
token_limit = 4096
working_directory = "~"

[ui]
color = true
format = "table"
show_detailed = false

[security]
allowed_paths = ["/home/user", "/tmp"]
denied_paths = ["/etc", "/usr/bin"]
rate_limit = 60  # requests per minute

[providers]
default = "anthropic"

[providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-3-sonnet"

[providers.openai]
api_key_env = "OPENAI_API_KEY"
model = "gpt-4"

[logging]
level = "info"
file = "~/.ai-session/logs/ai-session.log"
```

## Advanced Usage

### Session Templates

Create reusable session configurations:

```bash
# Create a template
ai-session template create --name "react-dev" \
  --shell "/bin/zsh" \
  --ai-context \
  --token-limit 8192 \
  --env "NODE_ENV=development" \
  --role "frontend"

# Use a template
ai-session create --template "react-dev" --name "my-react-app"

# List templates
ai-session template list

# Delete template
ai-session template delete "react-dev"
```

### Batch Operations

```bash
# Create multiple sessions
ai-session batch create \
  --template "react-dev" \
  --names "frontend,backend,testing" \
  --dirs "/app/frontend,/app/backend,/app/tests"

# Execute command across sessions
ai-session batch exec --pattern "dev-*" "git pull"

# Kill multiple sessions
ai-session batch kill --status "error"
```

### Scripting Support

```bash
#!/bin/bash
# Example script for automated development setup

# Create development environment
FRONTEND_ID=$(ai-session create --name "frontend" --role "frontend" --ai-context --format json | jq -r '.id')
BACKEND_ID=$(ai-session create --name "backend" --role "backend" --ai-context --format json | jq -r '.id')

# Setup frontend
ai-session exec "$FRONTEND_ID" "cd /app/frontend && npm install" --wait
ai-session exec "$FRONTEND_ID" "npm run dev" &

# Setup backend
ai-session exec "$BACKEND_ID" "cd /app/backend && cargo build" --wait
ai-session exec "$BACKEND_ID" "cargo run" &

echo "Development environment ready!"
echo "Frontend: $FRONTEND_ID"
echo "Backend: $BACKEND_ID"
```

### Integration with Other Tools

#### VS Code Integration

```bash
# Create session and open in VS Code
SESSION_ID=$(ai-session create --ai-context --format json | jq -r '.id')
code . && ai-session attach "$SESSION_ID"
```

#### CI/CD Integration

```bash
# In your CI pipeline
ai-session create --name "ci-build-$BUILD_ID" --ai-context
ai-session exec "ci-build-$BUILD_ID" "cargo test --all" --capture
ai-session context "ci-build-$BUILD_ID" --export "test-results.json"
```

## Troubleshooting

### Common Issues

#### Permission Denied
```bash
# Check file permissions
ai-session security check --path "/path/to/file"

# Fix permissions
chmod +x ~/.ai-session/bin/ai-session
```

#### Session Not Found
```bash
# List all sessions to verify ID
ai-session list --detailed

# Check session status
ai-session status <session-id>
```

#### High Memory Usage
```bash
# Check session resource usage
ai-session stats <session-id>

# Reduce token limit
ai-session config set token-limit 2048

# Enable compression
ai-session config set compression true
```

#### API Rate Limits
```bash
# Check rate limit status
ai-session quota status

# Adjust rate limits
ai-session config set rate-limit 30
```

### Debug Mode

Enable detailed logging for troubleshooting:

```bash
# Set debug logging
export RUST_LOG="ai_session=debug"

# Run with debug output
ai-session --verbose create --ai-context

# Check logs
tail -f ~/.ai-session/logs/ai-session.log
```

### Performance Monitoring

```bash
# Session performance stats
ai-session stats <session-id> --detailed

# System resource usage
ai-session system stats

# Token usage analysis
ai-session analytics tokens --session <session-id>
```

## Keyboard Shortcuts

When attached to a session:

```
Ctrl+B, D       Detach from session
Ctrl+B, C       Create new window in session
Ctrl+B, N       Next window
Ctrl+B, P       Previous window
Ctrl+B, [       Enter copy mode (vi-like navigation)
Ctrl+B, ]       Paste from clipboard
Ctrl+B, ?       Show help
```

## Tips and Best Practices

### Performance Optimization

1. **Use appropriate token limits**: Don't allocate more tokens than needed
2. **Enable compression**: For sessions with large output
3. **Clean up unused sessions**: Regularly kill terminated sessions
4. **Use session templates**: For consistent environment setup

### Security Best Practices

1. **Limit file access**: Configure allowed/denied paths
2. **Use role-based permissions**: Assign appropriate agent roles
3. **Monitor session activity**: Regular security audits
4. **Rotate API keys**: Update provider credentials regularly

### AI Context Management

1. **Set meaningful session names**: Helps with context understanding
2. **Use structured commands**: Clear, descriptive command syntax
3. **Export context regularly**: Backup important session history
4. **Monitor token usage**: Prevent context overflow

This concludes the comprehensive CLI guide for ai-session. For more advanced features and customization options, refer to the API documentation and source code.