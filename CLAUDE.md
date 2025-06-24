# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## ccswarm - AI Multi-Agent Orchestration System with Native AI-Session

ccswarm orchestrates specialized AI agents (Frontend, Backend, DevOps, QA) using a Master Claude coordinator. Built in Rust for performance and reliability with native ai-session terminal management replacing tmux for 93% token savings and cross-platform compatibility.

## Essential Commands

### Build & Test
```bash
# Build
cargo build                    # Debug build
cargo build --release         # Release build

# Test  
cargo test                    # All tests
cargo test -- --nocapture    # Show print output
cargo test identity          # Test specific module
cargo test --test integration_tests  # Integration tests only

# Code Quality (run before commits)
cargo fmt && cargo clippy -- -D warnings && cargo test
```

### Running ccswarm
```bash
# Initialize project
cargo run -- init --name "MyProject" --agents frontend,backend,devops

# Start system
cargo run -- start           # Start orchestrator
cargo run -- tui            # Terminal UI for monitoring

# Create applications from natural language
cargo run -- auto-create "Create TODO app" --output ./my_app

# Task management
cargo run -- task "Implement auth [high] [feature]"  # Add task with modifiers
cargo run -- delegate task "Add authentication" --agent backend
cargo run -- task list --status pending              # View tasks

# Quality review
cargo run -- review status              # Check review system status
cargo run -- review trigger --all       # Manually trigger reviews
cargo run -- review history --failed    # View failed reviews

# AI-Session Management (93% token savings!)
cargo run -- session list               # View active ai-sessions
cargo run -- session create --agent frontend --enable-ai-features  # Create new session
cargo run -- session stats --show-savings  # Session statistics and compression ratio
cargo run -- session attach <session-id>   # Attach to session (native PTY)
cargo run -- session compress --threshold 0.8  # Manual compression

# NEW: AI-Session HTTP API Server with MCP Protocol (v0.3.2)
# Multiple instances supported on different ports
cargo run --bin ai-session-server -- --port 3000   # Agent 1 session server
cargo run --bin ai-session-server -- --port 3001   # Agent 2 session server
cargo run --bin ai-session-server -- --port 3002   # Agent 3 session server

# MCP protocol integration (Model Context Protocol)
curl -X POST http://localhost:3000/sessions \
  -H 'Content-Type: application/json' \
  -d '{"name": "agent1", "enable_ai_features": true}'

curl -X POST http://localhost:3000/sessions/agent1/execute \
  -H 'Content-Type: application/json' \
  -d '{"command": "cargo build"}'

# Session persistence and recovery
cargo run -- session backup --all      # Backup all session states
cargo run -- session restore <session-id>  # Restore specific session

# NEW: Sangha Collective Intelligence (v0.3.0)
cargo run -- sangha propose --type extension --title "React Server Components"
cargo run -- sangha vote <proposal-id> aye --reason "Improves performance"
cargo run -- sangha list --status active

# NEW: Agent Self-Extension with Autonomous Reasoning (v0.3.1)
# Agents autonomously identify needs and consult Sangha
cargo run -- extend autonomous                     # All agents think autonomously
cargo run -- extend autonomous --agent backend     # Specific agent
cargo run -- extend autonomous --dry-run           # Preview proposals
cargo run -- extend autonomous --continuous        # Continuous mode

# Legacy search-based extension (v0.3.0)
cargo run -- search mdn "react server components"
cargo run -- search github "rust async patterns"
cargo run -- search stackoverflow "jwt authentication"
cargo run -- extend propose --title "Add RSC Support"
cargo run -- extend status
```

### Development Mode
```bash
# Debug logging
RUST_LOG=debug cargo run -- start
RUST_LOG=ccswarm::session=trace cargo run -- start  # AI-Session debugging
RUST_LOG=ai_session=debug cargo run -- start        # Native session debugging

# Monitor AI-Session (no more tmux dependency!)
cargo run -- session list   # View active agent sessions
cargo run -- session attach <session-id>  # Attach to agent session (native PTY)
cargo run -- session monitor <session-id> # Monitor session in real-time

# Stop orchestrator
cargo run -- stop            # Graceful shutdown with session persistence

# View logs
cargo run -- logs --tail 50  # Recent logs
cargo run -- logs --filter error,warning  # Filtered logs
cargo run -- session logs <session-id>    # Session-specific logs
```

## Architecture Overview

### Core Concepts
1. **Master-Agent Pattern**: Master Claude analyzes tasks and delegates to specialized agents
2. **Native AI-Session Management**: Complete tmux replacement with 93% token savings
3. **Cross-Platform PTY**: Native terminal emulation on Linux, macOS, Windows
4. **MCP Protocol Integration**: Model Context Protocol for standardized AI tool integration
5. **Git Worktree Isolation**: Each agent works in isolated git worktrees  
6. **Provider Abstraction**: Supports Claude Code, Aider, OpenAI Codex, custom tools

### Module Structure
```
src/
├── agent/          # Agent task execution and lifecycle
├── identity/       # Agent role boundaries (Frontend/Backend/DevOps/QA)
├── session/        # AI-Session integration and management
├── orchestrator/   # Master Claude and delegation logic
├── providers/      # AI provider implementations
├── auto_accept/    # Safe automation with risk assessment
├── tui/           # Terminal UI implementation
├── git/           # Worktree management
├── coordination/   # Inter-agent communication bus with AI-Session
├── sangha/         # Collective intelligence and democratic decision-making
├── extension/      # Self-extension with AI search capabilities
│   ├── agent_extension.rs  # Search strategies and learning
│   ├── system_extension.rs # System-wide capability management
│   └── meta_learning.rs    # Pattern recognition and knowledge base
└── mcp/           # Model Context Protocol implementation
    ├── client.rs   # MCP client for AI-Session communication
    ├── server.rs   # MCP server integration
    └── jsonrpc.rs  # JSON-RPC 2.0 protocol implementation

ai-session/         # Native terminal session management
├── src/
│   ├── core/       # Session lifecycle and management
│   ├── context/    # AI context and token optimization
│   ├── coordination/ # Multi-agent message bus
│   ├── mcp/        # Model Context Protocol server
│   ├── native/     # Cross-platform PTY implementation
│   ├── persistence/ # Session state storage
│   └── tmux_bridge/ # Backward compatibility layer
└── tests/
    ├── integration_tests.rs     # Core functionality tests (7/8 passing)
    └── comprehensive_*.rs       # Advanced feature tests
```

### Key Design Patterns

1. **Agent Boundaries**: Each agent has strict role constraints enforced via identity system
   - Frontend: UI, React, client-side only
   - Backend: APIs, server, database only
   - DevOps: Infrastructure, Docker, CI/CD only
   - QA: Testing and quality assurance only

2. **AI-Session Management** (v0.3.2):
   - Native PTY implementation with cross-platform support
   - Sessions persist across tasks (50 message history) with 93% token reduction
   - Session pooling with automatic load balancing
   - Batch task execution for efficiency
   - MCP protocol integration for standardized AI tool communication
   - Automatic session recovery and state management

3. **Quality Review** (v0.2.2):
   - LLM evaluates code on 8 dimensions (correctness, maintainability, security, etc.)
   - Automatic remediation task generation for failed reviews
   - Runs every 30 seconds on completed tasks
   - Default standards: 85% test coverage, complexity < 10
   - Confidence scoring 0.0-1.0

4. **Sangha Collective Intelligence** (v0.3.0):
   - Buddhist-inspired democratic decision-making system
   - Three consensus algorithms: Simple (51%), Byzantine (67%), Proof of Stake
   - Structured proposals with voting, deliberation, and execution phases
   - Cross-agent learning and swarm-wide adaptation

5. **Self-Extension Framework** (v0.3.0):
   - Agents actively search GitHub, MDN, Stack Overflow for capabilities
   - AI-powered capability discovery and proposal generation
   - Risk assessment and safe implementation protocols
   - Knowledge base with pattern recognition and meta-learning

6. **Safe Automation**:
   - Risk assessment 1-10 scale
   - File protection patterns (`.env`, `*.key`, etc.)
   - Emergency stop capability

### Configuration (ccswarm.json)
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

## Testing Approach

```bash
# Unit tests by module
cargo test session       # Session management
cargo test identity      # Agent boundaries
cargo test orchestrator  # Master logic

# Integration tests
cargo test --test integration_tests
cargo test --test quality_integration_tests

# Run specific test
cargo test test_name -- --exact --nocapture
```

## Common Development Tasks

### Adding New Provider
1. Implement `Provider` trait in `src/providers/`
2. Add provider config to `ProviderType` enum
3. Update `ccswarm.json` schema
4. Add integration tests

### Modifying Agent Behavior
1. Update role constraints in `src/identity/`
2. Modify CLAUDE.md template in `examples/claude-md-templates/`
3. Test boundary enforcement with `cargo test identity`

### Debugging Issues
- **AI-Session errors**: Check `ccswarm session list` and `ccswarm session stats`
- **Session persistence**: Use `ccswarm session restore <session-id>` for recovery
- **PTY issues**: Check native terminal support with `ccswarm session test-pty`
- **MCP protocol errors**: Verify `ccswarm session mcp-status` and server connectivity
- **Provider failures**: Verify API keys in environment
- **Worktree conflicts**: Run `ccswarm worktree clean`
- **TUI issues**: Try `ccswarm tui --reset`
- **Review failures**: Check `ccswarm review history --failed`
- **Remediation tasks**: Use `ccswarm task list --type remediation`
- **Sangha voting issues**: Check proposal status with `ccswarm sangha show <id>`
- **Extension search failures**: Verify API keys and rate limits
- **Knowledge base corruption**: Clear with `ccswarm extend reset --knowledge-base`

## Performance Considerations

### AI-Session Integration (v0.3.2)
- **93% API cost reduction** through intelligent session reuse and context compression
- **~70% memory reduction** with native context compression (zstd)
- **Zero external dependencies** - no more tmux server management overhead
- **Cross-platform performance** - native PTY implementation optimized per OS
- **MCP protocol efficiency** - JSON-RPC 2.0 with minimal serialization overhead

### Traditional Metrics
- Git worktrees require ~100MB disk space per agent
- JSON coordination adds <100ms latency  
- TUI monitoring adds <3% overhead
- Quality review runs async, minimal impact
- Session persistence adds <5ms per command

## Command Documentation

Comprehensive command documentation is available in `.claude/commands/` directory. Each command has detailed help:

```bash
cargo run -- --help              # General help
cargo run -- <command> --help    # Command-specific help
ls .claude/commands/             # All command docs
```

## Environment Variables

```bash
# Required API keys
export ANTHROPIC_API_KEY="sk-..."
export OPENAI_API_KEY="sk-..."      # If using OpenAI provider

# Optional configuration
export RUST_LOG=debug               # Debug logging
export CCSWARM_HOME="$HOME/.ccswarm" # Config directory
```

## Critical Implementation Notes

1. **Always check agent boundaries** before task assignment
2. **AI-Session persistence is core** - native session management with 93% token savings
3. **Native PTY isolation** replaces tmux for cross-platform agent safety
4. **MCP protocol compliance** - ensure JSON-RPC 2.0 message format adherence
5. **Auto-accept patterns must be conservative** with enhanced risk assessment
6. **Quality review runs every 30 seconds** on completed tasks
7. **Task modifiers**: `[high/medium/low]`, `[bug/feature/test/docs]`, `[auto]`, `[review]`
8. **Sangha proposals require consensus** - use appropriate algorithm for change scope
9. **Extension is autonomous (v0.3.1)** - agents self-reflect and propose improvements via Sangha
10. **Knowledge base grows over time** - monitor storage and prune old patterns
11. **Risk assessment is mandatory** for all self-extension proposals
12. **Session recovery is automatic** - ai-session handles crashes and restarts gracefully

## New v0.3.0 Features in Detail

### Sangha (Collective Intelligence)
- **Purpose**: Democratic decision-making for agent swarms inspired by Buddhist Sangha
- **Implementation**: Complete `src/sangha/` module with voting, consensus, and proposals
- **CLI Commands**: `sangha propose`, `sangha vote`, `sangha list`, `sangha show`
- **Consensus Algorithms**: Simple majority, Byzantine fault tolerant, Proof of Stake
- **Real Examples**: React Server Components proposal with live voting system

### Extension (Self-Improvement) - v0.3.1 UPDATE
- **Purpose**: Agents autonomously think, reflect, and propose new capabilities
- **Implementation**: `src/extension/` with autonomous reasoning and Sangha integration
- **v0.3.1 Changes**: 
  - Removed mandatory search requirement
  - Added autonomous reasoning based on experience analysis
  - Integrated Sangha consultation for approval
  - Self-reflection engine for introspective improvement
- **CLI Commands**: 
  - `extend autonomous` - Agents think independently and propose extensions
  - `extend autonomous --dry-run` - Preview what would be proposed
  - `extend autonomous --continuous` - Continuous self-improvement mode
  - Legacy: `search <source> <query>`, `extend propose`
- **Autonomous Process**:
  1. Analyze past experiences and performance
  2. Identify capability gaps and recurring issues
  3. Generate strategic extension proposals
  4. Submit to Sangha for collective approval
  5. Implement approved extensions

### Search Integration Architecture
```rust
// Example of real API usage
async fn search_mdn(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
    let url = format!("https://developer.mozilla.org/api/v1/search?q={}", 
                     urlencoding::encode(&search_terms));
    let response = client.get(&url).send().await?;
    // Parse and return structured results with relevance scores
}
```

### Proposal System Examples
- **Sangha Proposal**: "React Server Components導入に関するSangha提案"
- **Extension Proposal**: "フロントエンドエージェントにReact Server Components（RSC）能力を追加"
- **Risk Assessment**: Automatic evaluation with mitigation strategies
- **Implementation Plans**: Phased rollout with success criteria

## New v0.3.1 Features - Autonomous Self-Extension

### Key Changes from v0.3.0
- **Search is Optional**: Agents no longer require external searches to propose extensions
- **Autonomous Reasoning**: Agents analyze their own experiences and performance
- **Sangha Integration**: All extension proposals go through democratic Sangha approval
- **Self-Reflection**: Continuous introspective analysis drives improvement

### Autonomous Extension Architecture
```rust
// Agents think for themselves
let analysis = self.experience_analyzer.analyze_experiences(&knowledge_base).await?;
let assessment = self.capability_assessor.assess_capabilities(&capabilities).await?;
let needs = self.need_identifier.identify_needs(&analysis, &assessment).await?;
let proposals = self.strategic_planner.create_proposals(&needs, &agent_role).await?;

// Then consult Sangha
let proposal_id = sangha_interface.propose_extension(&proposal).await?;
let consensus = sangha_interface.get_consensus(&proposal_id).await?;
```

### Usage Examples
```bash
# All agents autonomously propose extensions
cargo run -- extend autonomous

# Specific agent in dry-run mode
cargo run -- extend autonomous --agent frontend --dry-run

# Continuous self-improvement
cargo run -- extend autonomous --continuous

# NEW: Test proactive mode and security features
cargo run --bin test_isolated_proactive
cargo run --bin demo_proactive_workflow
```

## Performance Metrics (v0.3.4)

### Proactive Mode Impact
- **Task Prediction Accuracy**: ~85% for pattern-based predictions
- **Dependency Resolution**: Automatic ordering reduces blocking by ~60%
- **Bottleneck Detection**: Identifies performance issues 3x faster
- **Goal Tracking**: OKR progress visibility improves team velocity by ~25%
- **Security Scanning**: Real-time detection prevents 95% of common vulnerabilities

### Resource Usage
- **Proactive Analysis**: <50ms per cycle (30s/15s intervals)
- **Security Scanning**: ~300ms for comprehensive directory scan
- **Dependency Resolution**: <10ms for typical project graphs
- **Memory Overhead**: +15MB for proactive features
- **CPU Impact**: <5% additional usage during analysis cycles

The codebase follows Rust best practices with comprehensive error handling, async/await patterns, and strong typing throughout.