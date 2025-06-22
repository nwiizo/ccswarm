# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## ccswarm - AI Multi-Agent Orchestration System

ccswarm orchestrates specialized AI agents (Frontend, Backend, DevOps, QA) using a Master Claude coordinator. Built in Rust for performance and reliability.

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

# Session management (93% token savings!)
cargo run -- session list               # View active sessions
cargo run -- session stats --show-savings  # Session statistics

# NEW: Sangha Collective Intelligence (v0.3.0)
cargo run -- sangha propose --type extension --title "React Server Components"
cargo run -- sangha vote <proposal-id> aye --reason "Improves performance"
cargo run -- sangha list --status active

# NEW: Agent Self-Extension with AI Search (v0.3.0)
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
RUST_LOG=ccswarm::session=trace cargo run -- start  # Session debugging

# Monitor tmux sessions
tmux ls                      # View active agent sessions
tmux attach -t <session>     # Attach to agent session

# Stop orchestrator
cargo run -- stop            # Graceful shutdown

# View logs
cargo run -- logs --tail 50  # Recent logs
cargo run -- logs --filter error,warning  # Filtered logs
```

## Architecture Overview

### Core Concepts
1. **Master-Agent Pattern**: Master Claude analyzes tasks and delegates to specialized agents
2. **Session Persistence**: Maintains conversation history, reducing API tokens by 93%
3. **Git Worktree Isolation**: Each agent works in isolated git worktrees
4. **Provider Abstraction**: Supports Claude Code, Aider, OpenAI Codex, custom tools

### Module Structure
```
src/
├── agent/          # Agent task execution and lifecycle
├── identity/       # Agent role boundaries (Frontend/Backend/DevOps/QA)
├── session/        # Session persistence and pooling
├── orchestrator/   # Master Claude and delegation logic
├── providers/      # AI provider implementations
├── auto_accept/    # Safe automation with risk assessment
├── tui/           # Terminal UI implementation
├── git/           # Worktree management
├── coordination/   # Inter-agent communication bus
├── sangha/         # Collective intelligence and democratic decision-making
└── extension/      # Self-extension with AI search capabilities
    ├── agent_extension.rs  # Search strategies and learning
    ├── system_extension.rs # System-wide capability management
    └── meta_learning.rs    # Pattern recognition and knowledge base
```

### Key Design Patterns

1. **Agent Boundaries**: Each agent has strict role constraints enforced via identity system
   - Frontend: UI, React, client-side only
   - Backend: APIs, server, database only
   - DevOps: Infrastructure, Docker, CI/CD only
   - QA: Testing and quality assurance only

2. **Session Management**: 
   - Sessions persist across tasks (50 message history)
   - Session pooling with automatic load balancing
   - Batch task execution for efficiency

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
- **Session errors**: Check `ccswarm session list`
- **Provider failures**: Verify API keys in environment
- **Worktree conflicts**: Run `ccswarm worktree clean`
- **TUI issues**: Try `ccswarm tui --reset`
- **Review failures**: Check `ccswarm review history --failed`
- **Remediation tasks**: Use `ccswarm task list --type remediation`
- **Sangha voting issues**: Check proposal status with `ccswarm sangha show <id>`
- **Extension search failures**: Verify API keys and rate limits
- **Knowledge base corruption**: Clear with `ccswarm extend reset --knowledge-base`

## Performance Considerations

- Session reuse reduces API costs by ~93%
- Git worktrees require ~100MB disk space per agent
- JSON coordination adds <100ms latency
- TUI monitoring adds <3% overhead
- Quality review runs async, minimal impact

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
2. **Session persistence is core** - never bypass for efficiency
3. **tmux isolation is required** for agent safety
4. **Auto-accept patterns must be conservative**
5. **Quality review runs every 30 seconds** on completed tasks
6. **Task modifiers**: `[high/medium/low]`, `[bug/feature/test/docs]`, `[auto]`, `[review]`
7. **Sangha proposals require consensus** - use appropriate algorithm for change scope
8. **Extension searches are rate-limited** - respect API limits for GitHub/MDN/SO
9. **Knowledge base grows over time** - monitor storage and prune old patterns
10. **Risk assessment is mandatory** for all self-extension proposals

## New v0.3.0 Features in Detail

### Sangha (Collective Intelligence)
- **Purpose**: Democratic decision-making for agent swarms inspired by Buddhist Sangha
- **Implementation**: Complete `src/sangha/` module with voting, consensus, and proposals
- **CLI Commands**: `sangha propose`, `sangha vote`, `sangha list`, `sangha show`
- **Consensus Algorithms**: Simple majority, Byzantine fault tolerant, Proof of Stake
- **Real Examples**: React Server Components proposal with live voting system

### Extension (Self-Improvement)
- **Purpose**: Agents autonomously discover and propose new capabilities
- **Implementation**: `src/extension/` with search strategies and learning frameworks
- **API Integration**: Real connections to MDN, GitHub, Stack Overflow APIs
- **CLI Commands**: `search <source> <query>`, `extend propose`, `extend status`
- **Search Capabilities**: 
  - MDN: `https://developer.mozilla.org/api/v1/search`
  - GitHub: Uses `gh` CLI for repository and code search
  - Stack Overflow: `https://api.stackexchange.com/2.3/search`
- **Real Examples**: Live search results with relevance scoring and metadata

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

The codebase follows Rust best practices with comprehensive error handling, async/await patterns, and strong typing throughout.