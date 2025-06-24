# ccswarm Architecture Documentation

## System Architecture Overview

ccswarm follows a microkernel architecture with pluggable providers and a central orchestration layer.

## Workspace Structure

ccswarm uses a Cargo workspace to organize the codebase into multiple crates:

```
ccswarm/
├── Cargo.toml                    # Workspace root configuration
├── crates/
│   ├── ccswarm/                  # Main ccswarm application
│   │   ├── Cargo.toml
│   │   └── src/                  # Core ccswarm modules
│   └── ai-session/               # Native AI session management
│       ├── Cargo.toml
│       └── src/                  # Session management implementation
```

The workspace configuration enables:
- Shared dependencies across crates
- Parallel development of ccswarm and ai-session
- Cleaner separation of concerns
- Easier testing and benchmarking

```
┌─────────────────────────────────────────────────────────────┐
│                    Master Claude (Orchestrator)              │
│  - Task analysis and delegation                              │
│  - Quality review coordination                               │
│  - Sangha consensus management                               │
└─────────────────────┬───────────────────────────────────────┘
                      │
        ┌─────────────┴─────────────┬─────────────┬────────────┐
        │                           │             │            │
┌───────▼────────┐  ┌──────────────▼──┐  ┌──────▼─────┐  ┌───▼────┐
│Frontend Agent  │  │Backend Agent    │  │DevOps Agent│  │QA Agent│
│- React/Vue     │  │- APIs/Database  │  │- Docker    │  │- Tests │
│- UI/UX         │  │- Business Logic │  │- CI/CD     │  │- QA    │
└────────────────┘  └─────────────────┘  └────────────┘  └────────┘
        │                           │             │            │
        └─────────────┬─────────────┴─────────────┴────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│                    AI-Session Management Layer               │
│  - Native PTY implementation                                 │
│  - Context compression (93% token savings)                   │
│  - Session persistence and recovery                          │
│  - Multi-agent coordination bus                              │
│  📖 See: ../crates/ai-session/docs/ARCHITECTURE.md          │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Orchestrator (`crates/ccswarm/src/orchestrator/`)
The brain of the system that coordinates all agent activities.

#### Master Claude (`master_claude.rs`)
- Analyzes incoming tasks
- Determines optimal agent assignment
- Monitors task progress
- Handles cross-agent dependencies

#### Quality Judge (`llm_quality_judge.rs`)
- Evaluates code on 8 dimensions
- Generates remediation tasks for failures
- Runs every 30 seconds on completed tasks
- Confidence scoring 0.0-1.0

### 2. Agent System (`crates/ccswarm/src/agent/`)
Specialized AI agents with strict role boundaries.

#### Agent Types
- **Simple Agent**: Basic task execution
- **Persistent Agent**: Maintains context across tasks
- **Pool Agent**: Load-balanced agent groups

#### Identity System (`crates/ccswarm/src/identity/`)
- Enforces strict role boundaries
- Prevents scope creep
- Monitors agent responses for violations

### 3. AI-Session Integration (`crates/ccswarm/src/session/`)
Revolutionary session management powered by the ai-session crate, replacing tmux entirely.

📖 **For complete AI-Session documentation**: [../crates/ai-session/docs/README.md](../crates/ai-session/docs/README.md)

#### AI-Session Adapter (`ai_session_adapter.rs`)
- **Bridge Layer**: Connects ccswarm orchestrator with ai-session crate (located in `crates/ai-session/`)
- **Session Lifecycle**: Creates, manages, and terminates ai-session instances for each agent
- **Context Compression**: Leverages ai-session's 93% token reduction capabilities
- **Cross-Platform PTY**: Uses ai-session's native PTY implementation
- **Integration API**: See [../crates/ai-session/docs/ccswarm-integration-api.md](../crates/ai-session/docs/ccswarm-integration-api.md)
- **Message Bus Integration**: Coordinates multi-agent communication via ai-session's coordination layer

#### AI-Session Features Used by ccswarm
- **Token-Efficient Context**: Automatic conversation history compression using zstd
- **Semantic Output Parsing**: Intelligent analysis of build results, test outputs, and error messages
- **Multi-Agent Coordination**: Message bus architecture for agent-to-agent communication
- **Session Persistence**: Automatic crash recovery and state restoration
- **MCP Protocol Support**: HTTP API server for external tool integration
- **Performance Monitoring**: Real-time metrics and token usage tracking

#### Session Types in ccswarm
- **Agent Session**: Specialized ai-session instance per agent (frontend, backend, devops, qa)
- **Worktree Session**: Git worktree integration with ai-session persistence
- **Persistent Session**: Long-running contexts with ai-session state management
- **Pool Session**: Load-balanced ai-session instances for high-throughput scenarios

#### Integration Architecture
```rust
// ccswarm creates specialized ai-session configurations
let session_config = SessionConfig {
    enable_ai_features: true,
    agent_role: AgentRole::Frontend,
    context_config: ContextConfig {
        max_tokens: 4096,
        compression_threshold: 0.8,
    },
    coordination_config: CoordinationConfig {
        enable_message_bus: true,
        agent_id: "frontend-specialist".to_string(),
    },
};

// ai-session handles the low-level terminal management
let ai_session = ai_session::SessionManager::new()
    .create_session_with_config(session_config).await?;
```

### 4. Coordination Layer (`crates/ccswarm/src/coordination/`)
Inter-agent communication and task management.

#### Components
- **Task Queue**: Async task distribution
- **Message Bus**: Agent communication
- **Dialogue System**: Structured conversations

### 5. Extension Framework (`crates/ccswarm/src/extension/`)
Self-improvement and capability expansion.

#### Autonomous Extension (`autonomous_agent_extension.rs`)
- Analyzes agent experiences
- Identifies capability gaps
- Proposes improvements via Sangha

#### Search Integration (`agent_extension.rs`)
- GitHub API for code patterns
- MDN for web standards
- Stack Overflow for solutions

### 6. Sangha System (`crates/ccswarm/src/sangha/`)
Democratic decision-making inspired by Buddhist principles.

#### Consensus Algorithms
- **Simple Majority**: 51% approval
- **Byzantine Fault Tolerant**: 67% approval
- **Proof of Stake**: Weighted by contribution

## Data Flow

### Task Execution Flow
```
User Input → CLI Parser → Task Queue → Master Claude Analysis
    ↓
Agent Assignment → Session Creation → Task Execution
    ↓
Output Streaming → Quality Review → Task Completion
    ↓
Session Persistence → Knowledge Base Update
```

### Extension Proposal Flow
```
Agent Experience Analysis → Capability Assessment
    ↓
Need Identification → Proposal Generation
    ↓
Sangha Submission → Democratic Voting
    ↓
Consensus Achievement → Implementation
```

## Module Dependencies

### Core Dependencies
- `tokio`: Async runtime
- `serde`: Serialization
- `portable-pty`: Cross-platform PTY
- `ai-session`: Advanced AI-optimized session management (workspace crate at `crates/ai-session/`)
  - Provides 93% token reduction through intelligent context compression
  - Native cross-platform PTY implementation
  - Multi-agent coordination and message bus architecture
  - Can be used independently as a standalone library or CLI tool

### Provider Dependencies
- `claude_code`: Anthropic integration
- `aider`: Aider tool integration
- `codex`: OpenAI integration
- `custom`: Custom tool support

### Workspace Dependencies and Integration

The workspace structure enables tight integration between ccswarm and ai-session:

```toml
# In crates/ccswarm/Cargo.toml
[dependencies]
ai-session = { path = "../ai-session" }

# ai-session can also be used independently:
# ai-session = "0.1"  # When published to crates.io
```

#### Integration Benefits
1. **Shared Development**: Both crates evolve together in the same repository
2. **Consistent APIs**: Breaking changes are coordinated across both crates
3. **Performance Optimization**: Direct memory sharing without serialization overhead
4. **Unified Documentation**: Single source of truth for the entire system
5. **Simplified Testing**: Integration tests can test both crates together

#### Dependency Flow
```
ccswarm (orchestration)
    ↓ depends on
ai-session (terminal management)
    ↓ depends on
portable-pty, tokio, serde, etc.
```

#### Standalone Usage
Developers can use ai-session independently without ccswarm:

```bash
# Install just ai-session
cargo install --path crates/ai-session

# Use for any AI terminal workflow
ai-session create --name myproject --ai-context
ai-session exec myproject "python train.py" --capture
```

## Security Architecture

### Agent Isolation
- Git worktree isolation per agent
- No cross-agent file access
- Sandboxed execution environment

### Risk Assessment
- Auto-accept patterns with risk scoring
- Protected file patterns
- Emergency stop capability

### Security Agent Integration
- OWASP vulnerability scanning
- Real-time security monitoring
- Automated remediation

## Performance Optimizations

### Session Reuse
- 93% token reduction through context caching
- Intelligent session pooling
- Automatic garbage collection

### Memory Management
- Context compression with zstd
- Sliding window for history
- Lazy loading of agent contexts

### Concurrency
- Async/await throughout
- Lock-free data structures where possible
- Work-stealing task scheduler

## Testing Strategy

### Unit Tests
- 267 core tests
- Module-level isolation
- Mocked dependencies

### Integration Tests
- End-to-end workflows
- Multi-agent scenarios
- Session persistence verification

### Ignored Tests
Tests that may fail in CI due to timing or environment:
- Concurrent agent operations
- Message persistence timing
- Complex integration workflows

## Extension Points

### Adding New Providers
1. Implement `Provider` trait in `crates/ccswarm/src/providers/`
2. Add to `ProviderType` enum
3. Update configuration schema
4. Add integration tests in `crates/ccswarm/tests/`

### Adding New Agent Roles
1. Define role in `AgentRole` enum in `crates/ccswarm/src/agent/`
2. Create identity constraints in `crates/ccswarm/src/identity/`
3. Add CLAUDE.md template in `crates/ccswarm/templates/`
4. Update delegation logic in `crates/ccswarm/src/orchestrator/`

### Custom Extensions
1. Implement extension trait in `crates/ccswarm/src/extension/`
2. Register with Sangha in `crates/ccswarm/src/sangha/`
3. Define consensus requirements
4. Add migration logic

### AI-Session Crate Structure and Integration

The ai-session crate (`crates/ai-session/`) provides the foundational terminal management capabilities:

#### Core Modules Used by ccswarm
- **Core Session Management** (`crates/ai-session/src/core/`)
  - Native PTY implementation with cross-platform support
  - Session lifecycle management and resource cleanup
  - Process management and terminal emulation

- **Context Management** (`crates/ai-session/src/context/`)
  - Token-efficient conversation history (93% savings)
  - Intelligent compression using zstd algorithm
  - AI-optimized context preservation and retrieval

- **Multi-Agent Coordination** (`crates/ai-session/src/coordination/`)
  - Message bus architecture for agent communication
  - Task distribution and result aggregation
  - Cross-agent knowledge sharing

- **Output Processing** (`crates/ai-session/src/output/`)
  - Semantic parsing of command outputs
  - Structured analysis of build results, tests, and logs
  - Pattern recognition for error detection

- **Session Persistence** (`crates/ai-session/src/persistence/`)
  - Automatic state snapshots and recovery
  - Compressed storage of session history
  - Cross-platform file system integration

- **MCP Protocol Server** (`crates/ai-session/src/mcp/`)
  - HTTP API server for external integrations
  - JSON-RPC 2.0 implementation
  - Tool discovery and capability registration

#### Standalone vs Integrated Usage

**Standalone ai-session usage:**
```rust
// Independent of ccswarm
use ai_session::{SessionManager, SessionConfig};

let manager = SessionManager::new();
let session = manager.create_session_with_ai_features().await?;
```

**ccswarm integration:**
```rust
// ccswarm wraps ai-session with orchestration logic
use crate::session::ai_session_adapter::AISessionAdapter;

let adapter = AISessionAdapter::new(agent_config);
let session = adapter.create_specialized_session().await?;
```

#### Documentation Cross-References
- **[AI-Session Architecture](../crates/ai-session/docs/ARCHITECTURE.md)** - Detailed ai-session system design
- **[AI-Session API Guide](../crates/ai-session/docs/API_GUIDE.md)** - Complete API reference
- **[Integration Examples](../crates/ai-session/examples/)** - Practical usage patterns