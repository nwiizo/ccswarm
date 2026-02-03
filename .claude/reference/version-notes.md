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
| `start` Command | Partial | Initializes but coordination loop incomplete |
| Parallel Executor | Partial | Structure exists, not wired to orchestrator |
| Auto-Create | Partial | Template generation works, AI generation incomplete |
| Sangha (Voting) | Planned | Data structures only |
| Extensions | Planned | Stub implementation |

**Key Limitation**: Orchestrator coordination loop not fully implemented. `ccswarm start` initializes but doesn't run continuous agent coordination.

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
