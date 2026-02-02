# ccswarm Application Specification

## Overview

ccswarm is an AI Multi-Agent Orchestration System that coordinates specialized AI agents (Frontend, Backend, DevOps, QA) using a Master Claude coordinator. Built in Rust for performance and reliability with native ai-session terminal management.

## Key Features

### Core Capabilities
- **Multi-Agent Orchestration**: Master Claude analyzes tasks and delegates to specialized agents
- **Native AI-Session Management**: 93% token savings through intelligent session reuse
- **Cross-Platform Support**: Native PTY implementation for Linux, macOS (Windows not supported)
- **Model Context Protocol (MCP)**: Standardized AI tool integration via JSON-RPC 2.0
- **Git Worktree Isolation**: Each agent works in isolated git worktrees for safety

### Agent Specializations
1. **Frontend Agent**: React, Vue, UI/UX, CSS, client-side development
2. **Backend Agent**: APIs, databases, server logic, authentication
3. **DevOps Agent**: Docker, CI/CD, infrastructure, deployment
4. **QA Agent**: Testing, quality assurance, test coverage

### Advanced Features
- **Sangha Collective Intelligence**: Democratic decision-making for agent swarms
- **Self-Extension Framework**: Agents autonomously propose new capabilities
- **Quality Review System**: Automatic code quality evaluation with remediation
- **Auto-Create**: Generate complete applications from natural language descriptions
- **Session Persistence**: Automatic recovery from crashes and restarts

## System Requirements

### Supported Platforms
- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)
- Windows is NOT supported due to Unix-specific dependencies

### Dependencies
- Rust 1.70+
- Git 2.20+
- API keys for AI providers (Anthropic, OpenAI, etc.)

### Build Requirements
The project uses a Cargo workspace structure:
- Root workspace defined in `/Cargo.toml`
- Main ccswarm crate: `/crates/ccswarm/`
- AI-Session crate: `/crates/ai-session/`

## Performance Metrics

### AI-Session Integration (v0.3.2)
- **93% API cost reduction** through intelligent session reuse and context compression
- **~70% memory reduction** with native context compression using zstd algorithm
- **Zero external dependencies** - no tmux server management overhead or external processes
- **Cross-platform performance** - native PTY implementation optimized per OS (Linux, macOS, Windows)
- **Standalone capability** - ai-session crate can be used independently of ccswarm
- **MCP protocol support** - Model Context Protocol HTTP API server for external integrations
- **Semantic output parsing** - intelligent analysis of build results, test outputs, and error messages

### Resource Usage
- Git worktrees require ~100MB disk space per agent
- JSON coordination adds <100ms latency
- TUI monitoring adds <3% overhead
- Quality review runs async, minimal impact
- Session persistence adds <5ms per command

## Project Structure

The ccswarm project uses a Cargo workspace structure for better modularity and build management:

```
ccswarm/
├── Cargo.toml                 # Workspace root configuration
├── crates/
│   ├── ccswarm/              # Main ccswarm crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs       # CLI entry point
│   │   │   └── lib.rs        # Library root
│   │   └── tests/            # Integration tests
│   └── ai-session/           # AI-Session management crate
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs        # Library exports
│       │   └── bin/
│       │       ├── ai-session.rs    # AI-Session CLI
│       │       └── server.rs        # MCP HTTP server
│       └── tests/
├── docs/                     # Documentation
└── examples_disabled/        # Example configurations
```

### Binary Outputs
- `ccswarm`: Main orchestration CLI (from `crates/ccswarm/src/main.rs`)
- `ai-session`: Session management CLI (from `crates/ai-session/src/bin/ai-session.rs`)
- `ai-session-server`: MCP HTTP API server (from `crates/ai-session/src/bin/server.rs`)

## AI-Session Crate Integration

### Overview

The `ai-session` crate provides the core terminal session management capabilities for ccswarm. It can be used as:

1. **Integrated with ccswarm**: Provides session management for AI agents
2. **Standalone library**: Independent terminal session management for any AI application
3. **CLI tool**: Direct command-line usage for manual session management
4. **HTTP API server**: MCP protocol server for external integrations

### Key Capabilities

#### Token Efficiency
```rust
use ai_session::{SessionManager, SessionConfig};

// Create AI-optimized session with token compression
let mut config = SessionConfig::default();
config.enable_ai_features = true;
config.context_config.max_tokens = 4096;
config.context_config.compression_threshold = 0.8;

let session = manager.create_session_with_config(config).await?;
let context = session.get_ai_context().await?;
// Automatic 93% token reduction through intelligent compression
println!("Token savings: {}%", context.compression_ratio * 100.0);
```

#### Multi-Agent Coordination
```rust
use ai_session::coordination::{CoordinationBus, Message, MessageType};

// ccswarm uses this for agent communication
let bus = Arc::new(RwLock::new(CoordinationBus::new()));

// Agents communicate through the bus
bus.write().await.broadcast(Message {
    sender: "frontend-agent".to_string(),
    receiver: Some("backend-agent".to_string()),
    msg_type: MessageType::TaskAssignment,
    content: json!({"task": "create API endpoint", "priority": "high"}),
    timestamp: Utc::now(),
}).await?;
```

#### Semantic Output Analysis
```rust
// ai-session automatically parses command outputs
let output = session.execute_command("cargo test").await?;
let analysis = session.analyze_output(&output).await?;

match analysis.parsed_output {
    ParsedOutput::TestResults { passed, failed, details } => {
        println!("Tests: {} passed, {} failed", passed, failed);
        // ccswarm uses this for intelligent task updates
    },
    ParsedOutput::BuildOutput { status, artifacts } => {
        println!("Build status: {:?}", status);
    },
    _ => {}
}
```

### Standalone Usage

#### As a Library
```toml
# Add to your Cargo.toml
[dependencies]
ai-session = { path = "path/to/ccswarm/crates/ai-session" }
# or when published:
# ai-session = "0.1"
```

```rust
use ai_session::{SessionManager, SessionConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = SessionManager::new();
    let session = manager.create_session_with_ai_features().await?;
    
    // Use for any AI terminal workflow
    session.send_input("python -m pytest\n").await?;
    let output = session.read_output().await?;
    
    // Get AI-optimized context
    let context = session.get_ai_context().await?;
    println!("Ready for AI processing with {} tokens", context.token_count);
    
    Ok(())
}
```

#### As a CLI Tool
```bash
# Install ai-session CLI
cargo install --path crates/ai-session

# Create and manage sessions independently
ai-session create --name myproject --ai-context
ai-session exec myproject "npm test" --capture
ai-session context myproject --export json
ai-session migrate --from-tmux session-name
```

### API Specifications

#### MCP Protocol HTTP Server
The ai-session crate includes an MCP (Model Context Protocol) HTTP API server for external integrations:

```bash
# Start the MCP server (runs independently)
ai-session-server --port 3000 --host 0.0.0.0

# Or from ccswarm
ccswarm session start-mcp-server --port 3000
```

#### Core Endpoints
- `POST /sessions` - Create new AI session
- `POST /sessions/{id}/execute` - Execute command with semantic analysis
- `GET /sessions/{id}/output` - Get parsed output with AI context
- `GET /sessions/{id}/context` - Get AI-optimized conversation context
- `PUT /sessions/{id}/compress` - Trigger context compression
- `DELETE /sessions/{id}` - Terminate session

#### Advanced Endpoints
- `GET /sessions/{id}/analysis` - Get semantic output analysis
- `POST /sessions/{id}/coordinate` - Send message to coordination bus
- `GET /sessions/{id}/metrics` - Get performance and token metrics
- `POST /sessions/migrate` - Migrate from tmux sessions

#### Example Usage
```bash
# Create AI-optimized session
curl -X POST http://localhost:3000/sessions \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "ai-agent-1",
    "enable_ai_features": true,
    "context_config": {
      "max_tokens": 4096,
      "compression_threshold": 0.8
    }
  }'

# Execute command with semantic analysis
curl -X POST http://localhost:3000/sessions/ai-agent-1/execute \
  -H 'Content-Type: application/json' \
  -d '{
    "command": "cargo test",
    "capture_output": true,
    "analyze_output": true
  }'

# Get AI context (with 93% token savings)
curl http://localhost:3000/sessions/ai-agent-1/context
```

### Integration with ccswarm

ccswarm integrates ai-session through the `AISessionAdapter`:

```rust
// In ccswarm codebase
use crate::session::ai_session_adapter::AISessionAdapter;

// ccswarm creates ai-session instances for each agent
let session = AISessionAdapter::create_session(&agent_config).await?;
session.execute_command("cargo build").await?;

// Automatic token optimization and context management
let compressed_context = session.get_compressed_context().await?;
// This achieves the 93% token savings ccswarm advertises
```

### Documentation References

For comprehensive ai-session documentation:

- **[AI-Session README](../crates/ai-session/README.md)** - Overview and quick start
- **[AI-Session Architecture](../crates/ai-session/docs/ARCHITECTURE.md)** - Detailed system design
- **[AI-Session API Guide](../crates/ai-session/docs/API_GUIDE.md)** - Complete API reference
- **[AI-Session CLI Guide](../crates/ai-session/docs/CLI_GUIDE.md)** - Command-line interface
- **[Integration Examples](../crates/ai-session/examples/)** - Practical usage examples

## Configuration

### Project Configuration (ccswarm.json)
Project configurations are stored in the project directory, not within the ccswarm codebase:

```json
{
  "project": {
    "name": "MyProject",
    "master_claude_instructions": "Custom orchestration instructions"
  },
  "agents": [
    {
      "name": "frontend-specialist",
      "role": "Frontend",
      "provider": "claude_code",
      "auto_accept": { "enabled": true, "risk_threshold": 5 }
    }
  ]
}
```

Example configurations can be found in `crates/ccswarm/examples_disabled/`.

### Environment Variables
- `ANTHROPIC_API_KEY`: Required for Claude-based providers
- `OPENAI_API_KEY`: Required for OpenAI-based providers
- `RUST_LOG`: Control logging verbosity
- `CCSWARM_HOME`: Configuration directory (default: ~/.ccswarm)

## Usage Examples

### Installation and Building
```bash
# Clone the repository
git clone https://github.com/nwiizo/ccswarm
cd ccswarm

# Build the workspace (builds all crates)
cargo build --release

# Install ccswarm globally
cargo install --path crates/ccswarm

# Or run directly from workspace
cargo run --package ccswarm -- init --name "TodoApp"
```

### Basic Workflow
```bash
# Initialize project (creates ai-session configurations)
ccswarm init --name "TodoApp" --agents frontend,backend

# Start system (spawns ai-session instances for each agent)
ccswarm start

# Create task (delegated to agents via ai-session)
ccswarm task "Create user authentication system [high] [feature]"

# Monitor progress (shows ai-session statistics)
ccswarm tui

# View ai-session details
ccswarm session list --show-savings
ccswarm session stats --detailed
```

### Advanced Usage
```bash
# Auto-create complete application (uses ai-session for all agent interactions)
ccswarm auto-create "Create a real-time chat application with React and WebSockets"

# Autonomous agent extension (agents use ai-session for coordination)
ccswarm extend autonomous --continuous

# Sangha proposal and voting (coordination via ai-session message bus)
ccswarm sangha propose --type feature --title "Add GraphQL support"
ccswarm sangha vote <proposal-id> aye --reason "Improves API flexibility"

# Direct ai-session usage for advanced workflows
ai-session create --name experimental --multi-agent
ai-session coordinate --from frontend --to backend --message "api-ready"
ai-session compress --session experimental --threshold 0.9
```

### Development Commands
```bash
# Run tests for all workspace crates
cargo test --workspace

# Run tests for specific crate
cargo test --package ccswarm
cargo test --package ai-session

# Build release binaries for all crates
cargo build --release --workspace

# Run ccswarm directly from source
cargo run --package ccswarm -- <command>

# Run ai-session-server from source
cargo run --package ai-session --bin server
```

## Version History

### v0.3.8 (Current)
- Observability/Tracing with OpenTelemetry and Langfuse support
- Human-in-the-Loop approval workflows
- Long-term Memory/RAG with vector embeddings
- Graph Workflow Engine with DAG-based execution
- Benchmark Integration with SWE-Bench style evaluation

### v0.3.7
- Search Agent with Gemini CLI integration
- Enhanced Sangha participation for agents
- Improved inter-agent communication

### v0.3.5
- Proactive Master Claude with goal tracking
- Security agent integration
- Enhanced auto-create capabilities

### v0.3.2
- MCP protocol integration
- Session HTTP API server
- Improved error recovery

### v0.3.1
- Autonomous agent reasoning
- Self-reflection engine
- Continuous improvement mode

### v0.3.0
- Sangha collective intelligence
- Democratic decision-making
- Extension framework

See CHANGELOG.md for complete version history.