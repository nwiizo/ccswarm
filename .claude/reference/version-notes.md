# Version Notes

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| CLI Infrastructure | Working | All commands parse and route correctly |
| Session Management | Working | Native PTY sessions via ai-session (no tmux) |
| TUI Dashboard | Working | Real-time monitoring with ratatui |
| Git Worktrees | Working | Isolated workspaces per agent |
| Template System | Working | Project scaffolding from templates |
| Configuration | Working | Project and agent config management |
| Task Queue | Partial | Queuing works, execution not connected |
| `start` Command | Working | Coordination loop with agent spawning, delegate mode, ACP support |
| Parallel Executor | Working | ParallelExecutor with WorkloadBalancing strategies |
| Auto-Create | Partial | Template generation works, AI generation incomplete |
| Sangha (Voting) | Working | Full consensus (SimpleMajority/BFT/ProofOfStake), proposal lifecycle, persistence |
| IPC Server | Working | Axum HTTP server for inter-agent communication |
| Team Management | Working | Lead + teammates model, shared task list, task claiming |
| Mailbox Messaging | Working | Direct/broadcast messaging with priority support |
| Delegate Mode | Working | Lead-only orchestration, no direct code execution |
| Plan Approval | Working | Read-only plan mode with lead approval workflow |
| Task Converter | Working | Converts agent tasks to coordination bus messages |
| Extensions | Planned | Stub implementation |

**Key Limitation**: Orchestrator coordination loop runs but uses simulated AI execution. Real provider integration pending.

## v0.4.5 Features

### Agent Teams (`orchestrator/team.rs`, `orchestrator/delegate.rs`)
- Team creation/management (lead + teammates model)
- Shared task list with status tracking and dependencies
- File-lock-based task claiming
- Delegate mode: lead orchestrates only, no direct code execution
- `--delegate` CLI flag

### Mailbox Messaging (`coordination/mailbox.rs`)
- Direct message (1-to-1) and broadcast messaging
- Priority levels (Low/Normal/High/Critical)
- Integration with CoordinationBus
- Per-agent mailbox with message history

### Sangha Consensus (`sangha/`)
- Refactored from single file to module directory
- `SanghaConsensus` trait fully implemented on `Sangha`
- Three algorithms: SimpleMajority (51%), BFT (67%), ProofOfStake (weighted)
- `ProposalManager` with full lifecycle (Pending→Voting→Approved/Rejected)
- JSON persistence via `SanghaPersistence`
- Algorithm-integrated proposal finalization

### IPC Server (`ipc/`)
- Axum-based HTTP API for inter-agent communication
- Endpoints: health, task submission, status, agent list, messages
- Configurable port with graceful shutdown
- Task state management with atomic status updates

### Plan Approval (`orchestrator/plan_approval.rs`)
- Read-only plan mode for review
- Lead approval/rejection with feedback
- HITL integration for human review

### Task Converter (`orchestrator/task_converter.rs`)
- Converts AgentTask to CoordinationBus messages
- Priority mapping and metadata preservation
- Bidirectional conversion support

### CLI Enhancements
- `ccswarm verify <path>` command for verification agent
- `--delegate` flag for delegate mode
- `--enable-acp` flag for ACP protocol

## v0.4.0 Features

See [docs/UPCOMING_FEATURES.md](../../docs/UPCOMING_FEATURES.md) for:
- Hook System Integration
- Verification Agent Pattern
- DynamicSpawner with workload balancing
- Parallel execution (command-based and PTY-based)
- ai-session MessageBus coordination
- Session persistence and resume/fork

## v0.3.8 - New Modules

### Observability/Tracing (`src/tracing/`)
- OpenTelemetry and Langfuse compatible export
- Span hierarchies with token tracking
- Trace collector with LRU eviction
- Multiple export formats (JSON, OpenTelemetry, Langfuse, CSV)

### Human-in-the-Loop (`src/hitl/`)
- Approval workflows with policy-based rules
- Multi-channel notifications (CLI, Slack, Email)
- Escalation support with timeout handling
- Complete audit trail for all decisions

### Long-term Memory/RAG (`src/memory/`)
- Vector embeddings with cosine similarity
- Short-term/Long-term memory separation
- Retrieval-augmented generation support
- Importance-based retention with decay

### Graph Workflow Engine (`src/workflow/`)
- DAG-based task workflows
- Conditional branching and parallel execution
- Approval gates at workflow checkpoints
- Sub-workflow composition

### Benchmark Integration (`src/benchmark/`)
- SWE-Bench style evaluation framework
- Predefined suites (basic coding, bug fixes, refactoring)
- Metrics collection with leaderboard
- Custom benchmark creation
