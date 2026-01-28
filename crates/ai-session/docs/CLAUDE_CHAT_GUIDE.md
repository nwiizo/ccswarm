# Claude Chat Guide

AI-Session provides seamless integration with Claude Code for interactive AI-powered coding assistance.

## Quick Start

### 1. Start the Server

```bash
ai-session-server --port 4000 &
```

### 2. Start Chatting

```bash
claude-chat
```

That's it! You're now in an interactive chat session with Claude Code.

## Features

### 🤖 Auto-Setup
- Automatically creates sessions if they don't exist
- Starts Claude Code in the session
- Manages connection and initialization

### 💬 Interactive Commands
- `/help` - Show available commands
- `/status` - Check session status
- `/output` - Get latest output
- `/clear` - Clear screen
- `/exit` or `/quit` - Exit chat mode

### 🎯 Natural Conversations
Simply type your questions or requests:

```
💬 > Create a Python function to sort a list using quicksort

💬 > Now add type hints and docstrings

💬 > Can you optimize it for nearly sorted arrays?

💬 > Write comprehensive unit tests
```

## Advanced Usage

### Custom Server
```bash
claude-chat --server http://localhost:5000
```

### Custom Session Name
```bash
claude-chat --session my-project
```

### Raw Output Mode
```bash
claude-chat --raw
```

### Manual Session Management
```bash
# Create session manually
ai-session remote create claude-code --ai-features --server http://localhost:4000

# Then connect
claude-chat --no-auto-create
```

## ccswarm Integration

### Multi-Agent Workflow

1. **Frontend Agent**
   ```bash
   ai-session-server --port 4001 &
   claude-chat --server http://localhost:4001 --session frontend-agent
   ```

2. **Backend Agent**
   ```bash
   ai-session-server --port 4002 &
   claude-chat --server http://localhost:4002 --session backend-agent
   ```

3. **DevOps Agent**
   ```bash
   ai-session-server --port 4003 &
   claude-chat --server http://localhost:4003 --session devops-agent
   ```

### Task Coordination

```bash
# Submit task to specific agent
ai-session remote exec backend-agent "implement user authentication API" \
  --server http://localhost:4002

# Check agent status
ai-session remote status backend-agent --server http://localhost:4002

# Get task output
ai-session remote output backend-agent --server http://localhost:4002
```

## Best Practices

### 1. Context Management
- Keep conversations focused on specific tasks
- Use clear, specific prompts
- Reference file names and functions explicitly

### 2. Multi-Line Input
For complex prompts, use triple quotes:
```
💬 > """
Create a REST API with the following endpoints:
- GET /users
- POST /users
- PUT /users/:id
- DELETE /users/:id

Include authentication and validation.
"""
```

### 3. Code Review Workflow
```
💬 > Review the code in src/main.rs for security issues

💬 > Suggest performance improvements

💬 > Generate unit tests for the identified edge cases
```

## Troubleshooting

### Session Not Found
```bash
# Check existing sessions
ai-session remote list --server http://localhost:4000

# Create manually if needed
ai-session remote create claude-code --ai-features
```

### Connection Issues
```bash
# Check server health
ai-session remote health --server http://localhost:4000

# Restart server
pkill ai-session-server
ai-session-server --port 4000 &
```

### Claude Not Responding
```bash
# Check session status
ai-session remote status claude-code

# Restart Claude in session
ai-session remote exec claude-code "claude" --server http://localhost:4000
```

## Environment Variables

```bash
# Default server URL
export CLAUDE_SERVER="http://localhost:4000"

# Default session name
export CLAUDE_SESSION="my-claude"

# AI-Session binary path
export AI_SESSION_BIN="/usr/local/bin/ai-session"
```

## Tips & Tricks

### 1. Project Templates
```bash
💬 > Create a new Rust web service with axum, including:
     - Database migrations
     - JWT authentication  
     - Docker configuration
     - GitHub Actions CI/CD
```

### 2. Code Analysis
```bash
💬 > Analyze the codebase and identify:
     - Performance bottlenecks
     - Security vulnerabilities
     - Code smells
     - Missing test coverage
```

### 3. Learning Mode
```bash
💬 > Explain how async/await works in Rust with examples

💬 > Show me best practices for error handling

💬 > Compare different web frameworks pros and cons
```

## Integration with IDEs

### VS Code
```json
{
  "terminal.integrated.defaultProfile.linux": "claude-chat",
  "terminal.integrated.profiles.linux": {
    "claude-chat": {
      "path": "claude-chat",
      "args": ["--server", "http://localhost:4000"]
    }
  }
}
```

### Vim/Neovim
```vim
" Quick Claude chat
nnoremap <leader>cc :terminal claude-chat<CR>

" Send visual selection to Claude
vnoremap <leader>cs :w !ai-session remote exec claude-code -<CR>
```

## Security Considerations

1. **Local Only by Default**: Server binds to localhost
2. **Session Isolation**: Each session runs in its own context
3. **No Credential Storage**: API keys managed externally
4. **Audit Trail**: All commands logged with timestamps

## Performance Tips

1. **Token Efficiency**: AI features reduce API costs by 93%
2. **Response Caching**: Frequently used patterns cached
3. **Streaming Output**: Real-time response display
4. **Connection Pooling**: Reuses HTTP connections

Happy coding with Claude! 🤖✨