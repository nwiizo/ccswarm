# Version Notes

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
