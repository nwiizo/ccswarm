# ccswarm Architecture Documentation

## System Architecture Overview

ccswarm is an AI Agent Workflow DevOps toolchain that complements Claude Code Agent Teams. It provides workflow orchestration via flow/stage pipelines, NDJSON event recording, and an AISessionBridge for Claude Code CLI execution with session resumption and retry.

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
│                  CLI (~23 commands)                          │
│  init, task, pipeline, flow, harness, agent-gen, approve,  │
│  sangha, extend, search, evolution, doctor, ...             │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│              Workflow Engine (Flows + Stages)            │
│  - FlowEngine: YAML-driven multi-step pipelines            │
│  - FacetRegistry: persona/policy/knowledge composition      │
│  - Stage fields: agent, working_dir, retry_delay_ms      │
│  - Context passing between steps                            │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│              AISessionBridge (session/bridge.rs)             │
│  - Claude Code CLI subprocess execution                     │
│  - --resume flag for session continuation                   │
│  - --agent routing to .claude/agents/*.md                   │
│  - Retry with exponential backoff                           │
│  - Semantic output parsing via ai-session                   │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│              Events + Hooks                                  │
│  - NDJSON EventRecorder (.ccswarm/runs/{run-id}/)           │
│  - Duration tracking per stage                           │
│  - HookRegistry for pre/post actions                        │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────▼───────────────────────────────────────┐
│              AI-Session Management Layer                     │
│  - Context compression, output parsing, persistence         │
│  - Message bus for inter-agent communication                │
│  - Session persistence and recovery                         │
│  See: crates/ai-session/README.md                           │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Workflow Engine (`crates/ccswarm/src/workflow/`)
The core of ccswarm, driving flow-based workflow pipelines.

#### FlowEngine
- Loads YAML flow definitions with stages (steps)
- Executes stages sequentially with context passing between steps
- Supports stage-level fields: `agent`, `working_dir`, `retry_delay_ms`
- Integrates with EventRecorder for NDJSON audit trail

#### FacetRegistry
- Composes prompts from independent facets (persona, policy, knowledge, instruction)
- Builtin personas with enriched system prompts
- Resolution order: builtin < project < user

### 2. Agent System (`crates/ccswarm/src/agent/`)
Agent definitions and task modeling for Claude Code Agent Teams.

#### Agent Roles
- **Frontend**: React, Vue, UI/UX development
- **Backend**: APIs, databases, server logic
- **DevOps**: Docker, CI/CD, infrastructure
- **QA**: Testing, quality assurance
- **Master**: Coordination role

#### agent-gen command
- Generates `.claude/agents/*.md` from facet definitions
- Validates existing agent definitions against facet registry

#### Identity System (`crates/ccswarm/src/identity/`)
- AgentIdentity with role boundaries
- Enforces agent scope constraints

### 3. AISessionBridge (`crates/ccswarm/src/session/bridge.rs`)
Bridge between ccswarm workflows and Claude Code CLI execution.

#### Key Features
- **Claude Code CLI Execution**: Spawns `claude` subprocess with task prompts
- **Session Resumption**: Uses `--resume` flag to continue existing sessions
- **Agent Routing**: Routes to `.claude/agents/*.md` via `--agent` flag
- **Retry with Exponential Backoff**: Automatic retry on transient failures
- **Semantic Output Parsing**: Uses ai-session's OutputParser for structured results
- **Context Compression**: Leverages ai-session for token-efficient context management
- **Message Bus Integration**: Inter-agent communication via ai-session coordination layer

### 4. Events (`crates/ccswarm/src/events/`)
NDJSON event recording for observability.

#### EventRecorder
- Writes events to `.ccswarm/runs/{run-id}/events.ndjson`
- Duration tracking per stage
- Produces `summary.json` at run completion
- Event types: stage.start/end, task.enqueue/start/end, hitl.request/decision

### 5. Governance Layer (`crates/ccswarm/src/governance/`)
Renamed from `coordination/` to disambiguate from `ai-session::coordination` (which is a
technical message bus, while this is governance / HITL state).

#### Components
- **Proposals**: `.ccswarm/coordination/proposals/*.json` (sangha votes — still on-disk
  name kept for back-compat with existing data)
- **Extensions**: `.ccswarm/coordination/extensions/*.json` (agent self-extension)
- **Approvals**: `.ccswarm/approvals/*.json` (HITL gate state)
- **Agent Messages**: in-process channels for orchestration

### 6. Providers Layer (`crates/ccswarm/src/providers/`)
Multi-provider subprocess command builders, added in the multi-provider refactor.

- `AgentProvider` trait: `build_command(prompt, working_dir, options) -> Command`
- `ClaudeProvider` (default), `CodexProvider`, `CopilotProvider` (currently unsupported
  for code generation — see `providers/copilot.rs` for rationale)
- Selection precedence: stage YAML `provider:` > `CCSWARM_PROVIDER` env > Claude

## Data Flow

### Pipeline Execution Flow
```
User Input → CLI Parser → FlowEngine loads YAML
    ↓
Stage Iteration → FacetRegistry composes prompt
    ↓
AISessionBridge → Claude Code CLI (--resume, --agent)
    ↓
Output Parsing → EventRecorder (NDJSON) → Next Stage or COMPLETE
    ↓
RunSummary written → .ccswarm/runs/{run-id}/summary.json
```

### Harness Execution Flow
```
Scenario YAML → Harness Runner → FlowEngine
    ↓
Assertions verified → Consolidated report (JSON)
    ↓
Baseline diff → Approve/reject
```

## Module Dependencies

### Core Dependencies
- `tokio`: Async runtime
- `serde`: Serialization
- `clap`: CLI framework
- `ai-session`: Session management (workspace crate at `crates/ai-session/`)
  - Context compression and output parsing
  - Multi-agent coordination and message bus
  - Can be used independently as a standalone library or CLI tool

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
- Protected file patterns
- Emergency stop capability

## Known Limitations and Practical Constraints

### Pipeline Timeout
- Complex tasks (>500 word descriptions) can cause the implement stage to exceed 600s
- Workaround: split tasks, use shorter descriptions, or increase `--timeout`
- The `complete` stage now uses local summary (no Claude call) to avoid wasted time

### Stage Context Passing
- Context between stages is injected via prompt text, not Claude Code session state
- The `--resume` flag is sent but Claude Code CLI session continuity is version-dependent
- Template variables `{key}` in instructions expand from state.variables

### Output Parsing
- ai-session's OutputParser uses regex heuristics (not structured parsing)
- Works well for cargo test/build, basic for Playwright/npm test output
- Complex multi-line outputs may fall through to PlainText

## Performance Optimizations

### Session Reuse
- Token reduction through context compression via ai-session
- Session resumption via --resume flag

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
- Minimal focused tests (~10 core tests)
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

### Adding New Agent Roles
1. Define role in `AgentRole` enum in `crates/ccswarm/src/agent/`
2. Create identity constraints in `crates/ccswarm/src/identity/`
3. Add facet definitions for the new role
4. Use `ccswarm agent-gen generate` to create `.claude/agents/*.md`

### Adding New Workflow Flows
1. Create YAML flow definition with stages
2. Define facets (persona, policy, knowledge, instruction)
3. Place in `.ccswarm/flows/` (project) or `~/.ccswarm/flows/` (user)
4. Or install via `ccswarm repertoire add <git-url>`

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

#### Documentation
- **[AI-Session README](../crates/ai-session/README.md)** - Overview and API reference