# ai-session

Native AI session management with 93% token savings and cross-platform PTY support.

## Usage
```bash
ai-session <SUBCOMMAND>
```

## Subcommands
- `create` - Create a new AI-optimized session
- `list` - List all active sessions
- `exec` - Execute command in session
- `attach` - Attach to existing session
- `context` - Show AI context for session
- `migrate` - Migrate from tmux sessions
- `stats` - Show performance statistics
- `cleanup` - Clean up inactive sessions

## Description
AI-Session is the core session management library that powers ccswarm's 93% token savings. It replaces traditional terminal multiplexers like tmux with intelligent, AI-optimized session management designed specifically for AI agents.

## Examples

### Create AI-Optimized Session
```bash
$ ai-session create --name dev --ai-context
✅ Created session: dev
   AI Features: Enabled ✨
   Max Tokens: 4096
   Context Compression: Active
   Ready for AI interactions!
```

### List Active Sessions
```bash
$ ai-session list --detailed
📍 Active AI Sessions
════════════════════════════════════════════

ID: dev
Status: Active ✅
AI Features: Enabled ✨
Messages: 15
Token Usage: 1,240 / 4,096 (30%)
Context Compression: 2 compressions performed
Created: 2024-06-24 10:30:00
Last Activity: 2024-06-24 11:45:23

ID: backend-session
Status: Idle 💤
AI Features: Enabled ✨
Messages: 8
Token Usage: 890 / 4,096 (22%)
Created: 2024-06-24 10:35:00
```

### Execute Command with Output Capture
```bash
$ ai-session exec dev "cargo test" --capture
Running tests in session 'dev'...

running 12 tests
test api::test_endpoint ... ok
test models::test_validation ... ok
test utils::test_helpers ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

✅ Command completed successfully
   Output captured for AI context
   Tokens used: +45 (1,285 total)
```

### Show AI Context
```bash
$ ai-session context dev --lines 50
📖 AI Context for session 'dev'
════════════════════════════════════════════

Messages: 15 | Tokens: 1,285 / 4,096 | Compressions: 2

[10:30:00] USER: Let's start working on the API
[10:30:15] ASSISTANT: I'll help you build the API. What functionality do you need?
[10:32:45] USER: cargo test
[10:32:46] SYSTEM: Command executed: cargo test
[10:32:48] SYSTEM: Output: running 12 tests... [output compressed]
[10:33:00] ASSISTANT: Great! All tests are passing. The API endpoints are working correctly.
...

Context Health: ✅ Optimal
Token Efficiency: 93.2% savings vs raw conversation
```

### Migrate from tmux
```bash
$ ai-session migrate --session tmux-dev --ai-features
🔄 Migrating tmux session 'tmux-dev'...
   
Analyzing tmux session...
  ✅ Found 3 windows
  ✅ Captured command history (127 commands)
  ✅ Preserved working directories
  
Creating AI-Session equivalent...
  ✅ Session created: tmux-dev-migrated
  ✅ AI features enabled
  ✅ Context initialized from history
  
Migration complete! New session: tmux-dev-migrated
Original tmux session preserved (use --replace to remove)
```

### Performance Statistics
```bash
$ ai-session stats --global
📊 AI-Session Performance Statistics
════════════════════════════════════════════

💰 Token Efficiency:
  Total Sessions: 6
  Messages Processed: 342
  Raw Token Usage: 45,670
  Compressed Usage: 3,120
  Token Savings: 42,550 (93.2%)
  Estimated Cost Savings: $8.51

⚡ Performance Metrics:
  Average Session Creation: 89ms
  Command Execution Overhead: <5ms
  Context Retrieval: <1ms (4K tokens)
  Multi-Agent Message Rate: 1,247 msg/sec

💾 Resource Usage:
  Memory per Session: ~3.6MB
  Disk Usage: 45MB (all sessions)
  Compression Ratio: 73% (zstd)
  Session Recovery Rate: 100%

🔄 Activity Summary:
  Active Sessions: 3
  Idle Sessions: 2
  Completed Sessions: 1
  Total Commands Executed: 1,456
```

## Features

### AI-Optimized Session Management
- **93% token reduction** through intelligent compression
- **Conversation persistence** across restarts
- **Context-aware operations** with smart recommendations
- **Native PTY support** for cross-platform compatibility

### Integration with ccswarm
- **Seamless integration** with ccswarm orchestrator
- **Multi-agent coordination** via message bus
- **Automatic session management** for specialized agents
- **Quality review integration** for code analysis

### Advanced Capabilities
- **Semantic output parsing** for build results and errors
- **Decision tracking** for AI agent behavior analysis
- **Performance profiling** with resource monitoring
- **Migration tools** from existing tmux sessions

## Configuration

### Session Configuration
```bash
# Create with custom token limit
ai-session create --name dev --max-tokens 8192

# Enable specific AI features
ai-session create --name prod --ai-context --compression

# Configure for specific use case
ai-session create --name frontend --template react-dev
```

### Environment Variables
- `AI_SESSION_HOME` - Configuration directory (default: ~/.ai-session)
- `AI_SESSION_MAX_TOKENS` - Default token limit (default: 4096)
- `AI_SESSION_COMPRESSION` - Enable compression (default: true)
- `RUST_LOG` - Logging level for debugging

## Integration Points

### ccswarm Integration
AI-Session is automatically used by ccswarm for:
- Agent session management ([session.md](session.md))
- Task execution with context preservation
- Multi-agent coordination
- Token savings across the orchestrator

### Direct Library Usage
```rust
use ai_session::{SessionManager, SessionConfig};

// Create AI-optimized session
let manager = SessionManager::new();
let config = SessionConfig::default().with_ai_features(true);
let session = manager.create_session_with_config(config).await?;
```

## Related Commands
- `ccswarm session` - ccswarm's session management interface
- `ccswarm task` - Task execution using AI sessions
- `ccswarm tui` - Monitor AI sessions in real-time

## Documentation
- **[API Guide](../crates/ai-session/docs/API_GUIDE.md)** - Complete API reference
- **[Architecture](../crates/ai-session/docs/ARCHITECTURE.md)** - System design details
- **[Examples](../crates/ai-session/examples/)** - Usage examples and demos