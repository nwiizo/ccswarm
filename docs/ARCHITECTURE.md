# ccswarm Architecture Documentation

## System Architecture Overview

ccswarm follows a microkernel architecture with pluggable providers and a central orchestration layer.

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
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Orchestrator (`src/orchestrator/`)
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

### 2. Agent System (`src/agent/`)
Specialized AI agents with strict role boundaries.

#### Agent Types
- **Simple Agent**: Basic task execution
- **Persistent Agent**: Maintains context across tasks
- **Pool Agent**: Load-balanced agent groups

#### Identity System (`src/identity/`)
- Enforces strict role boundaries
- Prevents scope creep
- Monitors agent responses for violations

### 3. Session Management (`src/session/`)
Revolutionary session management replacing tmux.

#### AI-Session Adapter (`ai_session_adapter.rs`)
- Bridges ccswarm with ai-session crate
- Handles session lifecycle
- Manages context compression

#### Session Types
- **Worktree Session**: Git worktree integration
- **Persistent Session**: Long-running contexts
- **Pool Session**: Load-balanced sessions

### 4. Coordination Layer (`src/coordination/`)
Inter-agent communication and task management.

#### Components
- **Task Queue**: Async task distribution
- **Message Bus**: Agent communication
- **Dialogue System**: Structured conversations

### 5. Extension Framework (`src/extension/`)
Self-improvement and capability expansion.

#### Autonomous Extension (`autonomous_agent_extension.rs`)
- Analyzes agent experiences
- Identifies capability gaps
- Proposes improvements via Sangha

#### Search Integration (`agent_extension.rs`)
- GitHub API for code patterns
- MDN for web standards
- Stack Overflow for solutions

### 6. Sangha System (`src/sangha/`)
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
- `ai-session`: Native session management

### Provider Dependencies
- `claude_code`: Anthropic integration
- `aider`: Aider tool integration
- `codex`: OpenAI integration
- `custom`: Custom tool support

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
1. Implement `Provider` trait
2. Add to `ProviderType` enum
3. Update configuration schema
4. Add integration tests

### Adding New Agent Roles
1. Define role in `AgentRole` enum
2. Create identity constraints
3. Add CLAUDE.md template
4. Update delegation logic

### Custom Extensions
1. Implement extension trait
2. Register with Sangha
3. Define consensus requirements
4. Add migration logic