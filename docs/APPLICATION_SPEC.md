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

## Performance Metrics

### AI-Session Integration (v0.3.2)
- **93% API cost reduction** through intelligent session reuse
- **~70% memory reduction** with native context compression (zstd)
- **Zero external dependencies** - no tmux server management overhead
- **Cross-platform performance** - native PTY implementation optimized per OS

### Resource Usage
- Git worktrees require ~100MB disk space per agent
- JSON coordination adds <100ms latency
- TUI monitoring adds <3% overhead
- Quality review runs async, minimal impact
- Session persistence adds <5ms per command

## API Specifications

### MCP Protocol Server
The AI-Session HTTP API Server implements the Model Context Protocol for standardized AI tool communication.

#### Endpoints
- `POST /sessions` - Create new session
- `POST /sessions/{id}/execute` - Execute command in session
- `GET /sessions/{id}/output` - Get session output
- `DELETE /sessions/{id}` - Terminate session

#### Example Usage
```bash
# Create session
curl -X POST http://localhost:3000/sessions \
  -H 'Content-Type: application/json' \
  -d '{"name": "agent1", "enable_ai_features": true}'

# Execute command
curl -X POST http://localhost:3000/sessions/agent1/execute \
  -H 'Content-Type: application/json' \
  -d '{"command": "cargo build"}'
```

## Configuration

### Project Configuration (ccswarm.json)
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

### Environment Variables
- `ANTHROPIC_API_KEY`: Required for Claude-based providers
- `OPENAI_API_KEY`: Required for OpenAI-based providers
- `RUST_LOG`: Control logging verbosity
- `CCSWARM_HOME`: Configuration directory (default: ~/.ccswarm)

## Usage Examples

### Basic Workflow
```bash
# Initialize project
ccswarm init --name "TodoApp" --agents frontend,backend

# Start system
ccswarm start

# Create task
ccswarm task "Create user authentication system [high] [feature]"

# Monitor progress
ccswarm tui
```

### Advanced Usage
```bash
# Auto-create complete application
ccswarm auto-create "Create a real-time chat application with React and WebSockets"

# Autonomous agent extension
ccswarm extend autonomous --continuous

# Sangha proposal and voting
ccswarm sangha propose --type feature --title "Add GraphQL support"
ccswarm sangha vote <proposal-id> aye --reason "Improves API flexibility"
```

## Version History

### v0.3.5 (Current)
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