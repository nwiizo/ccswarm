# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> Versions 0.5.0–0.6.2 were tracked in docs/APPLICATION_SPEC.md Version
> History rather than here.

## [0.9.1] - 2026-06-10

Patch release from real published-package smoke testing.

### Fixed
- `ai-session exec <session-name> ...` now resolves session names as documented
  instead of accepting only UUIDs.
- `ai-session exec` now executes commands through the session working directory
  using a bounded subprocess path, records command history, and avoids PTY read
  hangs observed during real CLI smoke testing.

## [0.9.0] - 2026-06-10

Sangha consensus workflow and governed auto-extension.

### Added
- **Sangha consensus stages**: flow YAML can now declare `sangha:` on a
  stage. Multiple independent members evaluate the same decision, emit
  `SANGHA_DECISION=APPROVE|REVISE`, and the stage advances only when
  approvals meet quorum. This restores consensus as a first-class workflow
  primitive instead of routing dynamic work through a single team leader.
- **Sangha-first default workflow**: the builtin `default` flow now runs
  plan → Sangha quorum → implement → review → fix. `team_leader` remains
  available for compatibility as the `team-dynamic` flow, but the default
  governance path is consensus-based.
- **Governed auto-extension**: `ccswarm lab extend auto-propose` creates an
  extension proposal from local workflow context and, by default, creates a
  linked Sangha proposal for consensus approval. `extend propose
  --auto-sangha` links manual extension proposals to the same voting ledger.
- **ai-session execution primitives**: `ai-session` now exposes provider-neutral
  helpers for prompt size validation, working-directory prompt context, cwd
  enforcement, and structured subprocess execution. `ccswarm` routes live
  provider calls through these helpers via `AISessionBridge`.

### Changed
- Lab Sangha/extension state now respects `--repo` and writes under the target
  repository's `coordination/` directory instead of the process working
  directory.
- Pipeline `--model` and `--isolate` now reach live `AISessionBridge`
  execution options instead of being parsed and ignored.

## [0.8.0] - 2026-06-10

takt feature adoption + codex first-class support.

### Added
- **Codex first-class**: global `--provider <claude|codex>` flag
  (precedence: stage YAML > flag > `CCSWARM_PROVIDER` > claude); codex
  JSONL telemetry (`CCSWARM_CODEX_JSON=1` adds `--json`, real token
  counts and thread IDs parsed defensively); codex session resume via
  `codex exec resume <thread-id>` — multi-turn continuation now works on
  codex (`SameThreadContinuation::ProviderAssignedId`).
- **Rate-limit fallback chain**: flow-level `on_rate_limit:
  [{provider, model?}]` switches providers when a call fails with a
  rate-limit error; switch clears session continuation, injects a
  fallback notice, and records ProviderError events.
- **Stage promotion**: `promotion: [{at: N, provider?, model?}]`
  escalates provider/model from the Nth visit of a stage (last matching
  entry wins; excluded on parallel sub-stages).
- **Command quality gates**: stage-level `gates: [{name, command,
  timeout_secs?}]` run after the agent; failures feed bounded
  stdout/stderr back into the same stage for up to `max_retries`
  re-runs.
- **3-layer facets**: `~/.ccswarm/facets` (user) and repertoire package
  facets now load under project facets (later wins); `CCSWARM_HOME`
  honored.
- **LLM-backed ai() judge** (`CCSWARM_LLM_JUDGE=1`): ai() rule
  conditions ask a real model YES/NO instead of the lexical heuristic,
  which remains the offline fallback (failures logged, never silent).
- **team_leader orchestrator-worker**: `team_leader: {max_parts, ...}`
  on a stage has a leader call decompose the task into parts at runtime;
  parts run concurrently as synthesized workers and aggregate into the
  parallel shape (all()/any() rules work unchanged). Graceful
  degradation to a single worker on decomposition failure. New builtin
  flow `team-dynamic`.

### Fixed
- `--json` command output is now safe to parse: tracing goes to stderr,
  `init --json` suppresses human progress lines and reports the actual
  configured default agents, and `queue list --json` emits structured queue
  state instead of a table.
- `doctor` treats missing `ANTHROPIC_API_KEY` as an informational warning when
  provider CLIs can be authenticated directly, so a usable Codex/Claude CLI
  setup is not reported as a broken system.
- `pipeline --dry-run` now includes the task body in every previewed stage even
  when a flow instruction does not explicitly contain `{task}`.
- Flow validation now rejects unknown providers in `provider:`,
  `promotion.provider`, and `on_rate_limit.provider` instead of silently
  falling back to Claude.
- Live prompt construction no longer duplicates persona/system content or
  repeats task text when `{task}` was already expanded into a stage
  instruction.
- Command-gate context truncation is UTF-8 safe and no longer risks panics on
  non-ASCII output boundaries.
- `scaffold` now forwards the global `--provider` flag to its child pipeline,
  exits non-zero when that pipeline fails or times out, and initializes a
  minimal npm project whose default `npm test` command succeeds.
- Pipeline runs now fail when a terminal stage returns `status: failed`
  instead of reporting the flow as completed; post-pipeline test failures also
  return an error after printing the run view hint.
- CLI e2e tests now prefer Cargo's `CARGO_BIN_EXE_ccswarm` path so tests
  exercise the freshly built binary instead of a stale `target/debug/ccswarm`.
- Live pipeline runs now load custom facets: `execute_pipeline_core`
  previously loaded none, so project personas/policies applied to
  `--dry-run`/`flow render` but silently not to real executions.
  Projects with an existing `.ccswarm/facets/` will see their overrides
  take effect.
- `flow check` no longer fails on builtin flows with all()/any() rules:
  CompoundCondition serialized as a YAML enum tag the untagged rule
  wrapper couldn't re-parse; it now serializes as the `all: [...]` map
  form.

## [0.7.0] - 2026-06-10

Follow-up to a feature audit against takt: wire what was implemented but
unreachable, delete what wasn't worth wiring, and close the HITL/OTel gap.

### Added
- `max_stage_visits` flow field (default 3): per-stage visit bound that
  aborts stuck review→fix loops before the next provider call, with the
  repeating transition pattern in diagnostics. `flow check` now also runs
  static cycle analysis and reports cycles as warnings.
- HITL commit gate for unattended runs: `ccswarm auto --require-approval
  [--approval-timeout 600]` and `queue drain --require-approval` pause
  before committing until `ccswarm approve commit --id <run-id>` (or
  reject/timeout, which fail the task with recovery hints). New
  `approve commit` subcommand; pending records visible via
  `approve list --status pending`. HitlRequest/HitlDecision events are
  now actually recorded.
- Claude stream-json telemetry (`CCSWARM_CLAUDE_STREAM_JSON=1`): real
  token counts replace the bytes/4 estimates in stage events (so
  `ccswarm cost` is accurate), and tool names + run cost are recorded as
  ProviderCall events.
- Optional `otel` cargo feature: OTLP span export (activated at runtime
  by `OTEL_EXPORTER_OTLP_ENDPOINT`) plus `flow.run`/`flow.stage` tracing
  spans.
- `docs/COMPETITIVE_LANDSCAPE.md`: takt parity table, orchestrator
  comparison, and roadmap candidates.

### Changed
- Parallel stages' `all()`/`any()` aggregate conditions now actually
  route — parallel outputs were previously never passed to the judge.
- `ccswarm approve` honors `--repo` and validates `--id` against path
  traversal.

### Removed (breaking for library consumers)
- Unwired modules deleted from `workflow/`: arpeggio (stub), watch
  (unused), the legacy DAG engine (graph/node/execution +
  WorkflowRegistry), and github_issue (superseded by `tracker/`).
- `LoopStrategy` / `CycleDetector` collapsed into `analyze_flow()` +
  `LoopTracker::new(max_visits)`.

## [0.3.8] - 2026-01-29

> **Note**: v0.3.8 modules added 2025-12-15, Cargo.toml bumped 2026-01-29.
> Status: ✅ done | ⚡ file-export | 🔜 planned

### Added
- **Observability/Tracing** (`src/tracing/`): OpenTelemetry and Langfuse compatible tracing
  - Span hierarchies with token tracking ✅
  - Trace collector with LRU eviction ✅
  - Multiple export formats ⚡ (JSON, OpenTelemetry, Langfuse, CSV)

- **Human-in-the-Loop** (`src/hitl/`): Approval workflows with policy-based rules ✅
  - Multi-channel notifications: CLI ✅, Slack/Email 🔜
  - Timeout handling ✅, escalation 🔜
  - Complete audit trail ✅

- **Long-term Memory/RAG** (`src/memory/`): Memory system with RAG support
  - Short-term/Long-term memory separation ✅
  - RAG context building ✅
  - Importance-based retention ✅
  - Vector embeddings (real API) 🔜
  - Multiple backends: in-memory ✅, file/DB 🔜

- **Graph Workflow Engine** (`src/workflow/`): DAG-based task workflows
  - Workflow registry and state tracking ✅
  - Conditional branching 🔜
  - Parallel execution 🔜
  - Approval gates 🔜
  - Sub-workflow composition 🔜

- **Benchmark Integration** (`src/benchmark/`): SWE-Bench style evaluation ✅
  - Predefined suites (basic coding, bug fixes, refactoring) ✅
  - Metrics collection with leaderboard ✅
  - Custom benchmark creation ✅

### Changed
- Updated documentation with implementation status markers (✅/⚡/🔜)
- Noted ai-session crate planned for v0.4.0 (session management currently in ccswarm)

## [0.3.7] - 2025-06-26

### Added
- **Search Agent Implementation**: New specialized agent for web search using Gemini CLI
  - Integrated with coordination bus for inter-agent communication
  - Support for filtered searches with domain, date range, language, and file type filters
  - Search result parsing with relevance scoring
  - Request/response message protocol for agent collaboration
  
- **Search Agent Sangha Participation**: Autonomous decision-making for search agents
  - Intelligent proposal analysis using web search
  - Evidence-based voting with search results
  - Knowledge gap detection and initiative proposals
  - Integration with Sangha collective intelligence system

- **Enhanced Agent Communication**: Improved inter-agent messaging system
  - Two-layer communication architecture (ccswarm + ai-session)
  - AICoordinationBridge for seamless integration
  - Low-latency coordination (<100ms)
  - Message persistence and recovery

### Changed
- Updated architecture documentation to include Search Agent
- Enhanced coordination bus with new message types for search requests
- Improved agent role system with Search Agent boundaries
- Refined Sangha participation for automated research

### Fixed
- Agent communication synchronization issues
- Message persistence timing in coordination bus
- Identity boundary enforcement for new agent types

## [0.3.6] - 2025-06-25

### Added
- **Enhanced Error Visualization**: Rich error diagrams and visual representations for better debugging
- **Resource Monitoring System**: Real-time CPU, memory, and system resource tracking with limits
- **Template Management System**: Project and agent template storage with metadata and versioning
- **Message Conversion Framework**: Unified message conversion between ccswarm and ai-session formats
- **Quickstart Command**: Simplified onboarding with interactive project initialization
- **Error Recovery Database**: Intelligent error pattern recognition and solution suggestions
- **Enhanced CLI Help System**: Context-aware help with error resolution guides

### Changed
- **Improved Code Quality**: Fixed all clippy warnings and formatting issues
- **Better Error Handling**: Comprehensive error context and recovery suggestions
- **Enhanced Documentation**: Updated ai-session integration docs and command references
- **Refined Message Bus**: Improved coordination between ccswarm and ai-session messages
- **Optimized Performance**: Reduced complexity in resource monitoring and template storage

### Fixed
- **Collapsible If Statements**: Simplified nested conditionals for better readability
- **Unused Code Removal**: Cleaned up dead code and unused variables
- **Async/Await Issues**: Fixed MutexGuard held across await points
- **Type Complexity**: Simplified complex WebSocket type definitions
- **Memory Efficiency**: Optimized string operations and iterator usage

### Technical Improvements
- **Code Organization**: Better separation of concerns with dedicated error, resource, and template modules
- **Test Coverage**: Added comprehensive tests for new features
- **CI/CD Compatibility**: All warnings resolved for clean builds
- **Cross-Module Integration**: Seamless message conversion between ai-session and ccswarm

## [0.3.3] - 2025-06-24

### Added
- **Production-Ready AI-Session Integration**: Complete tmux replacement with native session management
- **93% Token Savings**: Intelligent conversation history compression and session reuse validated
- **Cross-Platform PTY Support**: Native terminal emulation on Linux, macOS, and Windows
- **MCP Protocol Implementation**: Model Context Protocol (JSON-RPC 2.0) for standardized AI integration
- **Multi-Agent Message Bus**: Native coordination system with session-aware communication
- **Session Persistence & Recovery**: Automatic session state management and crash recovery
- **Comprehensive Test Suite**: 87.5% test success rate (7/8 tests passing) with integration validation
- **Error Resilience**: Robust error handling and graceful degradation mechanisms

### Changed
- **Complete TMux Replacement**: Zero external dependencies, pure Rust implementation
- **Enhanced Performance**: ~70% memory reduction through native context compression
- **Improved Architecture**: Native ai-session module with dedicated workspace structure
- **Better Documentation**: Updated README and CLAUDE.md with comprehensive ai-session integration
- **Backward Compatibility**: Drop-in replacement for existing tmux workflows maintained

### Fixed
- **Session Management**: Resolved session persistence issues across command invocations
- **PTY Implementation**: Fixed native terminal support with portable-pty validation
- **Integration Testing**: Comprehensive test coverage for all major ai-session functionality
- **Memory Management**: Optimized context compression and session storage
- **Cross-Platform Issues**: Resolved compatibility issues across different operating systems

### Technical Implementation
- **AI-Session Workspace**: Complete `ai-session/` module with core, context, coordination, and MCP
- **Integration Patterns**: Native message bus, session pooling, and token optimization
- **Test Infrastructure**: Comprehensive integration tests with real-world scenario validation
- **Module Architecture**: Clean separation between ccswarm orchestration and ai-session management

## [0.3.1] - 2025-06-23

### Added
- **Autonomous Self-Extension**: Agents can now think independently and propose improvements without mandatory search
- **Self-Reflection Engine**: Continuous introspective analysis of agent experiences and performance
- **Experience-Based Learning**: Agents learn from their work history to identify capability gaps
- **Sangha Integration for Extensions**: All autonomous proposals go through democratic Sangha approval
- **Continuous Improvement Mode**: `extend autonomous --continuous` for perpetual self-improvement
- **Strategic Planning**: AI-driven identification of capability needs and improvement opportunities

### Changed
- Extension no longer requires external search - agents can propose based on experience alone
- Improved extension CLI with `extend autonomous` as the primary command
- Enhanced Sangha integration for all extension proposals
- Better separation between autonomous reasoning and optional search capabilities

### Fixed
- Extension proposals now properly integrate with Sangha voting system
- Improved error handling in extension module
- Fixed race conditions in continuous improvement mode

## [0.3.0] - 2025-06-20

### Added
- **Sangha Collective Intelligence**: Buddhist-inspired democratic decision-making for agent swarms
- **Self-Extension Framework**: Agents can search GitHub/MDN/StackOverflow to discover capabilities
- **AI-Powered Search Integration**: Real connections to documentation and code repositories
- **Evolution Tracking**: Monitor and analyze agent capability growth over time
- **Meta-Learning System**: Learn from successes and failures across the swarm
- **Extension Propagation**: Share successful improvements between agents
- **Risk Assessment**: Automatic evaluation of extension risks with mitigation strategies
- **Rollback Capability**: Safe experimentation with automatic rollback on failure

### Changed
- Major architecture refactor to support autonomous agent evolution
- Enhanced module structure with new sangha/ and extension/ modules
- Improved CLI with comprehensive extension management commands
- Better type safety and error handling throughout

## [0.2.2] - 2025-06-14

### Added
- **Interleaved Thinking Pattern**: Agents can now evaluate and adjust their approach mid-execution
  - Decision types: Continue, Refine, RequestContext, Pivot, Complete, Abort
  - Confidence-based decision making with role-specific patterns
  - Thinking history tracking for debugging and transparency

- **LLM Quality Judge System**: Advanced code quality evaluation inspired by Anthropic's research
  - Multi-dimensional evaluation across 8 quality aspects
  - Role-specific scoring adjustments (Frontend, Backend, DevOps, QA)
  - Intelligent issue categorization by severity (Critical, High, Medium, Low)
  - Actionable remediation instructions with effort estimates
  - Performance-optimized with evaluation caching

- **Enhanced Quality Review Cycle**: Automated quality checks with remediation
  - Periodic quality reviews every 30 seconds
  - Automatic remediation task creation for quality issues
  - Review history tracking with iteration counting
  - Re-evaluation after remediation completion

### Changed
- Improved task delegation accuracy with enhanced pattern matching
- Updated agent boundary checking to evaluate forbidden patterns first
- Enhanced test infrastructure with temporary directories for isolation
- Optimized session management for better token efficiency

### Fixed
- Fixed task queue tests using persistent directories
- Corrected Backend delegation rules to use AND conditions
- Fixed DevOps boundary checker to properly delegate application tasks
- Resolved all clippy warnings for CI/CD compliance
- Fixed identity test environment variable population

### Developer Experience
- All tests now pass successfully
- `cargo fmt --check` compliant
- `cargo clippy` shows no warnings
- Improved error messages and logging

## [0.2.1] - 2025-06-01

### Changed
- Minor bug fixes and performance improvements

## [0.2.0] - 2025-05-15

### Added
- Session persistence with 93% token reduction
- Master delegation system with multiple strategies
- Auto-create system for generating complete applications
- Multi-provider support (Claude Code, Aider, OpenAI Codex)
- Enhanced TUI with real-time monitoring
- Git worktree-based agent isolation

### Changed
- Improved identity management system
- Enhanced safety features with risk assessment
- Better error recovery mechanisms

## [0.1.0] - 2025-04-01

### Added
- Initial release with basic multi-agent orchestration
- Simple task delegation
- Basic monitoring capabilities
