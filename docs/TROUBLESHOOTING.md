# Troubleshooting Guide

This guide helps you diagnose and resolve common issues with ccswarm. The problems are organized by category with step-by-step solutions.

## Quick Diagnostics

Start with the built-in diagnostic tools:

```bash
# Run comprehensive system check
ccswarm doctor

# Fix common issues automatically
ccswarm doctor --fix

# Check specific components
ccswarm doctor --check sessions
ccswarm doctor --check providers
ccswarm doctor --check worktrees
ccswarm doctor --check configuration
```

## Installation Issues

### "cargo install ccswarm" fails

**Symptoms:**
- Compilation errors during installation
- Missing dependencies
- Version conflicts

**Solutions:**

1. **Update Rust toolchain:**
```bash
rustup update
rustc --version  # Should be 1.70+
```

2. **Clear cargo cache:**
```bash
cargo clean
rm -rf ~/.cargo/registry/cache
cargo install ccswarm
```

3. **Install system dependencies:**
```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
xcode-select --install
brew install openssl pkg-config

# Fedora/RHEL
sudo dnf install gcc openssl-devel pkg-config
```

4. **Build from source if needed:**
```bash
git clone https://github.com/nwiizo/ccswarm.git
cd ccswarm
cargo build --release
cargo install --path crates/ccswarm
```

### Binary not found after installation

**Symptoms:**
- `ccswarm: command not found`
- Installation appears successful but command doesn't work

**Solutions:**

1. **Check cargo bin directory:**
```bash
echo $PATH | grep -o "[^:]*\.cargo/bin[^:]*"
# Should show ~/.cargo/bin or similar
```

2. **Add to PATH if missing:**
```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
# or ~/.zshrc depending on your shell
source ~/.bashrc
```

3. **Verify installation location:**
```bash
which ccswarm
ls -la ~/.cargo/bin/ccswarm
```

## Configuration Issues

### Invalid configuration file

**Symptoms:**
- "Failed to parse ccswarm.json"
- "Missing required field" errors
- Configuration validation failures

**Solutions:**

1. **Validate JSON syntax:**
```bash
# Check for syntax errors
ccswarm config validate

# Use JSON validator
python -m json.tool ccswarm.json
```

2. **Generate valid configuration:**
```bash
# Create default configuration
ccswarm config init

# Use template for complex setups
ccswarm config init --template enterprise
```

3. **Fix common JSON issues:**
```bash
# Remove trailing commas
sed -i 's/,\s*}/}/g' ccswarm.json
sed -i 's/,\s*]/]/g' ccswarm.json

# Fix quote issues
sed -i 's/'/"/g' ccswarm.json
```

### Missing required fields

**Error:** `Missing required field: project.name`

**Solution:**
```bash
# Quick fix - add minimum required fields
cat > ccswarm.json << 'EOF'
{
  "project": {
    "name": "MyProject"
  },
  "agents": [
    {
      "name": "default-agent",
      "role": "Frontend",
      "provider": "claude_code"
    }
  ]
}
EOF
```

### API key issues

**Symptoms:**
- "Authentication failed" 
- "Invalid API key"
- Provider connection errors

**Solutions:**

1. **Check API key format:**
```bash
# Anthropic keys start with 'sk-ant-'
echo $ANTHROPIC_API_KEY | head -c 10

# OpenAI keys start with 'sk-'
echo $OPENAI_API_KEY | head -c 5
```

2. **Set API keys correctly:**
```bash
# Add to shell profile
echo 'export ANTHROPIC_API_KEY="your-key-here"' >> ~/.bashrc
source ~/.bashrc

# Or use .env file
cat > .env << 'EOF'
ANTHROPIC_API_KEY=your-key-here
OPENAI_API_KEY=your-key-here
EOF
```

3. **Test API key validity:**
```bash
# Test provider connection
ccswarm config test-providers --verbose

# Manual test with curl
curl -H "Authorization: Bearer $ANTHROPIC_API_KEY" \
     -H "Content-Type: application/json" \
     https://api.anthropic.com/v1/messages
```

## Session Management Issues

### "Session not found" errors

**Symptoms:**
- "No active session with ID: xyz"
- Sessions disappear unexpectedly
- Unable to attach to sessions

**Solutions:**

1. **List all sessions:**
```bash
# Show all sessions with details
ccswarm session list --detailed

# Show session statistics
ccswarm session stats
```

2. **Create new session if needed:**
```bash
# Create session for specific agent
ccswarm session create --agent frontend --enable-ai-features

# Create with custom name
ccswarm session create --name "my-session" --agent backend
```

3. **Check session persistence:**
```bash
# Verify session directory exists
ls -la ~/.ccswarm/sessions/

# Check session files
ls -la ~/.ccswarm/sessions/*/
```

4. **Clean up stale sessions:**
```bash
# Remove orphaned sessions
ccswarm session cleanup

# Force cleanup all sessions
ccswarm session cleanup --force
```

### Sessions consuming too much memory

**Symptoms:**
- High memory usage
- System slowdown
- Out of memory errors

**Solutions:**

1. **Enable compression:**
```json
{
  "session_management": {
    "compression": {
      "enabled": true,
      "level": 3,
      "threshold": 0.8
    }
  }
}
```

2. **Limit message history:**
```json
{
  "session_management": {
    "history": {
      "max_messages": 50,
      "sliding_window": true
    }
  }
}
```

3. **Clean up old sessions:**
```bash
# Remove sessions older than 24 hours
ccswarm session cleanup --older-than 24h

# Compress existing sessions
ccswarm session compress --all
```

### AI-Session server won't start

**Symptoms:**
- "Failed to start ai-session server"
- "Port already in use"
- Connection refused errors

**Solutions:**

1. **Check port availability:**
```bash
# Check if port 3000 is in use
lsof -i :3000
netstat -tulpn | grep :3000
```

2. **Kill existing processes:**
```bash
# Find and kill ai-session processes
pkill -f ai-session
killall ai-session-server
```

3. **Use different port:**
```bash
# Start server on different port
ccswarm session start-mcp-server --port 3001

# Update configuration
ccswarm config set session_management.mcp_port 3001
```

4. **Check permissions:**
```bash
# Ensure binary is executable
chmod +x ~/.cargo/bin/ai-session-server

# Check if port requires sudo
sudo lsof -i :3000
```

## Agent Issues

### Agent role violations

**Symptoms:**
- "Agent exceeded role boundaries"
- Tasks assigned to wrong agents
- Identity violation warnings

**Solutions:**

1. **Check agent configuration:**
```bash
# Show agent details
ccswarm agent list --detailed

# Show agent constraints
ccswarm agent show frontend --constraints
```

2. **Fix role boundaries:**
```json
{
  "agents": [
    {
      "name": "frontend-agent",
      "role": "Frontend",
      "constraints": {
        "file_patterns": ["src/**/*.{js,ts,jsx,tsx,css}"],
        "forbidden_patterns": ["server/**", "api/**", "*.py"]
      }
    }
  ]
}
```

3. **Manual task delegation:**
```bash
# Delegate task to correct agent
ccswarm delegate task "Fix CSS styling" --agent frontend

# Check delegation analysis
ccswarm delegate analyze "Add database model" --verbose
```

### Agents not responding

**Symptoms:**
- Tasks stuck in "assigned" state
- No output from agents
- Timeout errors

**Solutions:**

1. **Check agent status:**
```bash
# Show agent health
ccswarm status --agents

# Check agent logs
ccswarm logs --agent frontend --tail 100
```

2. **Restart stuck agents:**
```bash
# Restart specific agent
ccswarm agent restart frontend

# Restart all agents
ccswarm restart --agents-only
```

3. **Check provider connectivity:**
```bash
# Test provider for specific agent
ccswarm agent test frontend --provider

# Show provider status
ccswarm provider status
```

4. **Increase timeouts:**
```json
{
  "agents": [
    {
      "name": "frontend-agent",
      "claude_config": {
        "timeout": 600,
        "retry_attempts": 3
      }
    }
  ]
}
```

## Git Worktree Issues

### Worktree creation failures

**Symptoms:**
- "Failed to create worktree"
- "Directory already exists"
- Permission denied errors

**Solutions:**

1. **Check worktree status:**
```bash
# List all worktrees
git worktree list

# Show ccswarm worktrees
ccswarm worktree list
```

2. **Clean up existing worktrees:**
```bash
# Remove unused worktrees
git worktree prune

# Force cleanup
ccswarm worktree clean --force
```

3. **Fix permissions:**
```bash
# Check directory permissions
ls -la ../

# Fix permissions if needed
chmod 755 ../
```

4. **Manual worktree creation:**
```bash
# Create worktree manually
git worktree add ../frontend-agent main

# Update ccswarm configuration
ccswarm worktree sync
```

### Worktree synchronization issues

**Symptoms:**
- Changes not visible between worktrees
- Merge conflicts
- Inconsistent git state

**Solutions:**

1. **Sync worktrees:**
```bash
# Sync all worktrees
ccswarm worktree sync --all

# Sync specific worktree
ccswarm worktree sync frontend
```

2. **Check git status:**
```bash
# Check each worktree
for dir in ../*/; do
  echo "=== $dir ==="
  (cd "$dir" && git status --short)
done
```

3. **Resolve conflicts:**
```bash
# Show conflicted files
ccswarm worktree conflicts

# Auto-resolve simple conflicts
ccswarm worktree resolve --auto

# Manual resolution
ccswarm worktree resolve --interactive
```

## Provider Issues

### Claude Code provider errors

**Symptoms:**
- "claude_code command not found"
- Claude Code authentication failures
- Timeout errors

**Solutions:**

1. **Install Claude Code:**
```bash
# Install Claude Code CLI
curl -fsSL https://claude.ai/install.sh | bash

# Verify installation
claude_code --version
```

2. **Check authentication:**
```bash
# Login to Claude Code
claude_code auth login

# Test authentication
claude_code auth status
```

3. **Update provider configuration:**
```json
{
  "providers": {
    "claude_code": {
      "config": {
        "timeout": 600,
        "retry_attempts": 3,
        "dangerous_skip": true
      }
    }
  }
}
```

### Aider provider issues

**Symptoms:**
- "aider command not found"
- Git configuration errors
- Model availability issues

**Solutions:**

1. **Install Aider:**
```bash
# Install via pip
pip install aider-chat

# Or via pipx
pipx install aider-chat

# Verify installation
aider --version
```

2. **Configure git:**
```bash
# Set git user (required by Aider)
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

3. **Test Aider configuration:**
```bash
# Test Aider with specific model
aider --model claude-3-5-sonnet --test

# Check available models
aider --models
```

### Custom provider issues

**Symptoms:**
- Custom tool not executing
- Invalid command format
- Permission denied

**Solutions:**

1. **Check tool permissions:**
```bash
# Make tool executable
chmod +x /path/to/custom/tool

# Test tool directly
/path/to/custom/tool --help
```

2. **Verify configuration:**
```json
{
  "providers": {
    "custom": {
      "type": "custom",
      "command": "/usr/local/bin/my-tool",
      "args": ["--interactive", "--format", "json"],
      "config": {
        "timeout": 300,
        "working_directory": "/tmp"
      }
    }
  }
}
```

3. **Add to PATH:**
```bash
# Add tool directory to PATH
export PATH="/path/to/tools:$PATH"

# Update ccswarm configuration
ccswarm config set providers.custom.command "my-tool"
```

## Performance Issues

### High CPU usage

**Symptoms:**
- ccswarm consuming excessive CPU
- System becoming unresponsive
- Fan spinning up constantly

**Solutions:**

1. **Reduce analysis frequency:**
```json
{
  "project": {
    "master_claude": {
      "proactive_frequency": 60,
      "high_frequency": 30
    }
  }
}
```

2. **Limit concurrent operations:**
```json
{
  "coordination": {
    "dependencies": {
      "max_parallel": 2
    }
  },
  "session_management": {
    "max_sessions_per_role": 2
  }
}
```

3. **Enable CPU monitoring:**
```bash
# Monitor ccswarm processes
top -p $(pgrep ccswarm)

# Check system load
uptime
```

### High memory usage

**Solutions:**

1. **Enable compression:**
```json
{
  "session_management": {
    "compression": {
      "enabled": true,
      "level": 6
    }
  }
}
```

2. **Limit history:**
```json
{
  "session_management": {
    "history": {
      "max_messages": 25,
      "sliding_window": true
    }
  }
}
```

3. **Clean up regularly:**
```bash
# Clean up old sessions
ccswarm session cleanup --older-than 1h

# Compress existing sessions
ccswarm session compress --all
```

### Slow task execution

**Solutions:**

1. **Check provider response times:**
```bash
# Test provider performance
ccswarm provider benchmark --provider claude_code

# Show detailed timing
ccswarm logs --timing --tail 50
```

2. **Optimize session reuse:**
```json
{
  "session_management": {
    "pooling": {
      "enabled": true,
      "max_pool_size": 10
    }
  }
}
```

3. **Enable parallel execution:**
```json
{
  "coordination": {
    "dependencies": {
      "parallel_execution": true,
      "max_parallel": 5
    }
  }
}
```

## Security Issues

### Auto-accept safety concerns

**Symptoms:**
- Dangerous operations being auto-accepted
- Unwanted file changes
- Security warnings

**Solutions:**

1. **Lower risk threshold:**
```json
{
  "agents": [
    {
      "auto_accept": {
        "enabled": true,
        "risk_threshold": 3
      }
    }
  ]
}
```

2. **Add protection patterns:**
```json
{
  "auto_accept": {
    "protected_files": [
      ".env*",
      "*.key",
      "*.pem",
      "secrets/*",
      "production/*"
    ],
    "forbidden_operations": [
      "rm -rf",
      "sudo",
      "chmod 777"
    ]
  }
}
```

3. **Enable confirmation prompts:**
```json
{
  "auto_accept": {
    "require_confirmation": [
      "delete",
      "execute",
      "network"
    ]
  }
}
```

### Security scanning failures

**Solutions:**

1. **Check security agent status:**
```bash
# Show security agent status
ccswarm security status

# Run manual security scan
ccswarm security scan --directory ./src
```

2. **Update scanning tools:**
```bash
# Update npm audit database
npm audit fix

# Update cargo audit database
cargo install cargo-audit
cargo audit
```

3. **Configure scanning frequency:**
```json
{
  "security": {
    "security_agent": {
      "scan_frequency": "1h",
      "owasp_top_10": true
    }
  }
}
```

## Network Issues

### Connection timeouts

**Symptoms:**
- API requests timing out
- Intermittent connection failures
- "Connection refused" errors

**Solutions:**

1. **Increase timeouts:**
```json
{
  "providers": {
    "claude_code": {
      "config": {
        "timeout": 900,
        "retry_attempts": 5
      }
    }
  }
}
```

2. **Check network connectivity:**
```bash
# Test API endpoints
curl -v https://api.anthropic.com/v1/messages
curl -v https://api.openai.com/v1/models

# Check DNS resolution
nslookup api.anthropic.com
```

3. **Configure proxy if needed:**
```bash
# Set proxy environment variables
export HTTP_PROXY=http://proxy.company.com:8080
export HTTPS_PROXY=http://proxy.company.com:8080

# Or in configuration
export CCSWARM_PROXY=http://proxy.company.com:8080
```

### Rate limiting

**Symptoms:**
- "Rate limit exceeded" errors
- Requests being throttled
- 429 HTTP status codes

**Solutions:**

1. **Configure rate limits:**
```json
{
  "providers": {
    "claude_code": {
      "rate_limits": {
        "requests_per_minute": 30,
        "tokens_per_minute": 50000
      }
    }
  }
}
```

2. **Add delays between requests:**
```json
{
  "coordination": {
    "task_queue": {
      "delay_between_tasks": 5000
    }
  }
}
```

3. **Use session pooling:**
```json
{
  "session_management": {
    "pooling": {
      "enabled": true,
      "intelligent_reuse": true
    }
  }
}
```

## Logging and Debugging

### Enable detailed logging

```bash
# Full debug logging
export RUST_LOG=debug
ccswarm start

# Component-specific logging
export RUST_LOG=ccswarm::session=trace,ccswarm::agent=debug
ccswarm start

# Log to file
ccswarm start 2>&1 | tee ccswarm.log
```

### Analyze logs

```bash
# View recent logs
ccswarm logs --tail 100

# Filter logs by component
ccswarm logs --component session --tail 50

# Search logs
ccswarm logs --search "error" --tail 200

# Export logs
ccswarm logs --export /tmp/ccswarm-logs.txt
```

### Debug specific components

```bash
# Debug session management
ccswarm session debug --session-id abc123

# Debug agent behavior
ccswarm agent debug frontend --verbose

# Debug task delegation
ccswarm delegate debug "Create login form" --verbose
```

## Getting Help

### Built-in help system

```bash
# General help
ccswarm help

# Topic-specific help
ccswarm help troubleshooting
ccswarm help configuration
ccswarm help sessions

# Search help
ccswarm help --search "worktree"
```

### Error codes reference

ccswarm uses structured error codes:

- **CFG001-099**: Configuration errors
- **SES001-099**: Session management errors  
- **AGT001-099**: Agent errors
- **PRV001-099**: Provider errors
- **NET001-099**: Network errors
- **SEC001-099**: Security errors

### Community support

1. **GitHub Issues**: Report bugs and request features
2. **Discussions**: Ask questions and share solutions
3. **Documentation**: Check docs/ directory for guides
4. **Examples**: See demos/ directory for working examples

### Creating bug reports

When reporting issues, include:

```bash
# System information
ccswarm --version
rustc --version
uname -a

# Configuration (remove sensitive data)
ccswarm config show --redact

# Logs (recent errors)
ccswarm logs --error --tail 50

# System health
ccswarm doctor --verbose
```

---

If you're still experiencing issues after trying these solutions, please create a GitHub issue with the relevant error messages, configuration, and system information.