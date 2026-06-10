# ccswarm Application Specification

## Overview

ccswarm is an AI Agent Workflow DevOps toolchain that complements AI coding
provider CLIs. It provides flow-based workflow pipelines, NDJSON event
recording, provider-agnostic AISessionBridge execution, and agent definition
generation. Built in Rust for performance and reliability with native
ai-session terminal management.

## Key Features

### Core Capabilities
- **Flow-Based Workflows**: YAML-driven multi-step pipelines with stage context passing
- **AISessionBridge**: provider CLI execution with continuation, Claude
  --agent routing, retry with exponential backoff, and stream telemetry
- **NDJSON Event Recording**: Observability via `.ccswarm/runs/{run-id}/events.ndjson` with duration tracking
- **Agent Definition Generation**: `agent-gen` command generates/validates `.claude/agents/*.md` from facets
- **Cross-Platform Support**: Native PTY implementation for Linux, macOS (Windows not supported)
- **Git Worktree Isolation**: Each agent works in isolated git worktrees for safety
- **Harness Testing**: Scenario-driven validation of workflow pipelines

### Agent Specializations
1. **Frontend Agent**: React, Vue, UI/UX, CSS, client-side development
2. **Backend Agent**: APIs, databases, server logic, authentication
3. **DevOps Agent**: Docker, CI/CD, infrastructure, deployment
4. **QA Agent**: Testing, quality assurance, test coverage

### Advanced Features
- **Sangha Collective Intelligence**: Democratic decision-making for agent swarms
- **Self-Extension Framework**: Agents propose new capabilities via `extend` command
- **HITL Approvals**: Human-in-the-loop gates for plan/deploy/merge operations
- **Faceted Prompting**: Composable persona/policy/knowledge/instruction facets
- **Session Persistence**: Automatic recovery from crashes and restarts

## System Requirements

### Supported Platforms
- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)
- Windows is NOT supported due to Unix-specific dependencies

### Dependencies
- Rust 1.70+
- Git 2.20+
- At least one provider CLI for live execution: Claude Code CLI or Codex CLI.
  `gh copilot` can be probed but is intentionally unsupported for code
  generation.

### Build Requirements
The project uses a Cargo workspace structure:
- Root workspace defined in `/Cargo.toml`
- Main ccswarm crate: `/crates/ccswarm/`
- AI-Session crate: `/crates/ai-session/`

## Performance Metrics

### AI-Session Integration (v0.3.2)
- **93% API cost reduction** through intelligent session reuse and context compression
- **~70% memory reduction** with native context compression using zstd algorithm
- **Zero external dependencies** - no tmux server management overhead or external processes
- **Cross-platform performance** - native PTY implementation optimized per OS (Linux, macOS, Windows)
- **Standalone capability** - ai-session crate can be used independently of ccswarm
- **MCP protocol support** - Model Context Protocol HTTP API server for external integrations
- **Semantic output parsing** - intelligent analysis of build results, test outputs, and error messages

### Resource Usage
- Git worktrees require ~100MB disk space per agent
- JSON coordination adds <100ms latency
- Session persistence adds <5ms per command
- NDJSON event recording adds negligible overhead

## Project Structure

The ccswarm project uses a Cargo workspace structure for better modularity and build management:

```
ccswarm/
├── Cargo.toml                 # Workspace root configuration
├── crates/
│   ├── ccswarm/              # Main ccswarm crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs       # CLI entry point
│   │   │   └── lib.rs        # Library root
│   │   └── tests/            # Integration tests
│   └── ai-session/           # AI-Session management crate
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs        # Library exports
│       │   └── bin/
│       │       ├── ai-session.rs    # AI-Session CLI
│       │       └── server.rs        # MCP HTTP server
│       └── tests/
├── docs/                     # Documentation
└── examples_disabled/        # Example configurations
```

### Binary Outputs
- `ccswarm`: Main workflow DevOps CLI (from `crates/ccswarm/src/main.rs`)
- `ai-session`: Session management CLI (from `crates/ai-session/src/bin/ai-session.rs`)
- `ai-session-server`: MCP HTTP API server (from `crates/ai-session/src/bin/server.rs`)

## AI-Session Crate Integration

### Overview

The `ai-session` crate provides the core terminal session management capabilities for ccswarm. It can be used as:

1. **Integrated with ccswarm**: Provides session management for AI agents
2. **Standalone library**: Independent terminal session management for any AI application
3. **CLI tool**: Direct command-line usage for manual session management
4. **HTTP API server**: MCP protocol server for external integrations

### Key Capabilities

#### Token Efficiency
```rust
use ai_session::{SessionManager, SessionConfig};

// Create AI-optimized session with token compression
let mut config = SessionConfig::default();
config.enable_ai_features = true;
config.context_config.max_tokens = 4096;
config.context_config.compression_threshold = 0.8;

let session = manager.create_session_with_config(config).await?;
let context = session.get_ai_context().await?;
// Automatic 93% token reduction through intelligent compression
println!("Token savings: {}%", context.compression_ratio * 100.0);
```

#### Multi-Agent Coordination
```rust
use ai_session::coordination::{CoordinationBus, Message, MessageType};

// ccswarm uses this for agent communication
let bus = Arc::new(RwLock::new(CoordinationBus::new()));

// Agents communicate through the bus
bus.write().await.broadcast(Message {
    sender: "frontend-agent".to_string(),
    receiver: Some("backend-agent".to_string()),
    msg_type: MessageType::TaskAssignment,
    content: json!({"task": "create API endpoint", "priority": "high"}),
    timestamp: Utc::now(),
}).await?;
```

#### Semantic Output Analysis
```rust
// ai-session automatically parses command outputs
let output = session.execute_command("cargo test").await?;
let analysis = session.analyze_output(&output).await?;

match analysis.parsed_output {
    ParsedOutput::TestResults { passed, failed, details } => {
        println!("Tests: {} passed, {} failed", passed, failed);
        // ccswarm uses this for intelligent task updates
    },
    ParsedOutput::BuildOutput { status, artifacts } => {
        println!("Build status: {:?}", status);
    },
    _ => {}
}
```

### Standalone Usage

#### As a Library
```toml
# Add to your Cargo.toml
[dependencies]
ai-session = { path = "path/to/ccswarm/crates/ai-session" }
# or when published:
# ai-session = "0.1"
```

```rust
use ai_session::{SessionManager, SessionConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = SessionManager::new();
    let session = manager.create_session_with_ai_features().await?;
    
    // Use for any AI terminal workflow
    session.send_input("python -m pytest\n").await?;
    let output = session.read_output().await?;
    
    // Get AI-optimized context
    let context = session.get_ai_context().await?;
    println!("Ready for AI processing with {} tokens", context.token_count);
    
    Ok(())
}
```

#### As a CLI Tool
```bash
# Install ai-session CLI
cargo install --path crates/ai-session

# Create and manage sessions independently
ai-session create --name myproject --ai-context
ai-session exec myproject "npm test" --capture
ai-session context myproject --export json
ai-session migrate --from-tmux session-name
```

### API Specifications

#### MCP Protocol HTTP Server
The ai-session crate includes an MCP (Model Context Protocol) HTTP API server for external integrations:

```bash
# Start the MCP server (runs independently)
ai-session-server --port 3000 --host 0.0.0.0
```

#### Core Endpoints
- `POST /sessions` - Create new AI session
- `POST /sessions/{id}/execute` - Execute command with semantic analysis
- `GET /sessions/{id}/output` - Get parsed output with AI context
- `GET /sessions/{id}/context` - Get AI-optimized conversation context
- `PUT /sessions/{id}/compress` - Trigger context compression
- `DELETE /sessions/{id}` - Terminate session

#### Advanced Endpoints
- `GET /sessions/{id}/analysis` - Get semantic output analysis
- `POST /sessions/{id}/coordinate` - Send message to coordination bus
- `GET /sessions/{id}/metrics` - Get performance and token metrics
- `POST /sessions/migrate` - Migrate from tmux sessions

#### Example Usage
```bash
# Create AI-optimized session
curl -X POST http://localhost:3000/sessions \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "ai-agent-1",
    "enable_ai_features": true,
    "context_config": {
      "max_tokens": 4096,
      "compression_threshold": 0.8
    }
  }'

# Execute command with semantic analysis
curl -X POST http://localhost:3000/sessions/ai-agent-1/execute \
  -H 'Content-Type: application/json' \
  -d '{
    "command": "cargo test",
    "capture_output": true,
    "analyze_output": true
  }'

# Get AI context (with 93% token savings)
curl http://localhost:3000/sessions/ai-agent-1/context
```

### Integration with ccswarm

ccswarm integrates ai-session through the `AISessionBridge`:

```rust
// In ccswarm codebase
use crate::session::bridge::AISessionBridge;

// ccswarm creates bridge instances for workflow execution
let bridge = AISessionBridge::new(config);

// Execute via the selected provider CLI with session continuation and routing
let result = bridge.execute(task_prompt, agent_identity).await?;
// Result includes parsed output, success status, and duration
```

### Documentation References

- **[AI-Session README](../crates/ai-session/README.md)** - Overview and API reference

## Multi-Agent Fundamentals (Operating Model)

This section formalizes multi-agent behavior in ccswarm, aligning with widely used agent‑orchestration patterns and preparing for future provider integrations. It complements, not replaces, existing architecture.

### Agent Lifecycle
- Perceive: Ingest task/context, repository state, and recent coordination messages.
- Deliberate: Produce a plan (steps, risks, dependencies, stop conditions).
- Act: Execute atomic steps via session (idempotent where possible).
- Observe: Parse outputs (tests/build/logs) and update internal state.
- Report: Emit status/metrics and propose next actions.
- Reflect: Summarize learnings, refine prompt/tool usage, adjust strategy.

### Core Roles (Composable)
- Planner: Decomposes goals into DAG tasks, sets acceptance criteria.
- Executor: Performs edits, builds, tests, and environment operations.
- Researcher: Gathers external facts, patterns, and examples.
- Critic/Verifier: Reviews artifacts against checklists and policies.
- Coordinator: Arbitrates assignments, resolves conflicts, escalates for HITL.

ccswarm can instantiate specialized agents (Frontend/Backend/DevOps/QA) and also overlay these roles per agent.

### Interaction Protocols
- Flow Pipeline (current): Workflow engine executes stages, routing to agents via AISessionBridge.
- Sangha Consensus (current): `sangha:` stages collect independent member
  decisions and require quorum before implementation or policy changes proceed.
- Contract Net/Auction (planned): Agents bid on tasks using capability/cost/latency scores.
- Blackboard Bus: Shared topics for proposals, status, and decisions, with retention and replay.

### Task Contract (Canonical Schema)
Each task must include fields that enable automated assignment and robust verification.
- id, description, priority, type, details
- expected_outcomes: list of measurable acceptance items
- constraints: safety, performance, compatibility
- deps: prerequisite task ids, barrier policy (all/any)
- skills_required: tags mapped to provider capabilities
- estimated_cost/time, risk_level

### Capability Model
- Agent capability descriptor: { skills: string[], tools: string[], domains: string[], quality: {lint, test, perf}, limits: {tokens, time} }
- Provider adapter exposes affordances (edit, build, test, run, search, browse) and cost profile.
- Matching policy: score(task, agent) = f(skill match, historical success, queue length, SLA).

### Coordination Patterns
- Barriers for DAG nodes: don’t start downstream until upstream completes.
- Speculative execution (optional): run multiple approaches with budget guardrail; pick best.
- Rollback hooks: revert failing edits; keep failing artifacts for diagnosis.
- Checkpointing: persist intermediate artifacts and prompts for reproducibility.

### Safety and HITL
- Guardrails: file/path allow/deny lists, command policies, secret redaction.
- HITL gates: “plan”, “risky edit”, “deploy” checkpoints with ack/deny/comment.
- Rate/Cost limits: per agent and per project; graceful degradation when exceeded.

### Metrics and Evaluation
- Latency (queue, exec), success rate, revert rate, test pass %, lint errors, token/CPU usage.
- Rubrics per task type (feature, bugfix, refactor, docs) with thresholds.
- Leaderboard across agents and provider configurations.

### Failure Taxonomy and Recovery
- Categories: spec_mismatch, env_issue, flaky_test, merge_conflict, provider_error.
- Policy: retry with backoff (idempotent steps), escalate to Critic/Researcher, or HITL.

## Interfaces (Multi-Agent Specific)

### Coordination Bus Topics (planned consolidation)
- proposals.*: Planner/Researcher proposals (title, rationale, expected impact)
- assignments.*: Coordinator→Agent task assignments
- status.*: Agent periodic status and metrics
- reviews.*: Critic verdicts, checklists, and suggested fixes
- votes.*: Sangha votes and tally records

Message envelope fields (common): { id, ts, sender, topic, payload, correlation_id? }

### Agent Registration
- announce(capabilities, version, limits)
- heartbeat(interval, load)
- deregister(reason)

## Execution Policies (Refinements)
- Idempotency first: prefer commands that can be retried safely.
- Small safe steps: narrow diffs with tests between steps.
- Evidence‑driven: require build/test evidence for state transitions to completed.
- Budget‑aware: cap parallelism by priority and remaining budget.

## Acceptance Criteria (Multi-Agent Extensions)
- Agents can register capabilities and receive assignments filtered by required skills.
- Workflow engine can route stages to specific agents via the `agent` field (planned: auction/vote).
- Critic can block completion until all acceptance items pass.
- Bus retains messages long enough for late joiners to catch up and reconcile.

## Flows and Stages (Workflow Model)

ccswarm adopts a declarative workflow model inspired by established multi‑agent tools. A flow describes a sequence of stages (steps). Each stage binds a persona, permissions, and rules that determine the next step.

### Flow (YAML schema – planned alignment)
- name: string
- initial_movement: string
- max_movements: number (guard against infinite loops)
- stages: Stage[]

Stage fields:
- name: string
- persona: string (who acts; maps to agent prompt/facets)
- edit: boolean (whether file edits are permitted)
- agent: string (optional; routes to specific .claude/agents/*.md via --agent flag)
- working_dir: string (optional; working directory for the stage)
- retry_delay_ms: number (optional; base delay for exponential backoff on retry)
- required_permission_mode: enum (e.g., view|edit|exec; planned)
- rules: [{ condition: string, next: string|COMPLETE|ABORT }]

Rules encode fix loops (e.g., review → implement → review) and allow parallel review stages in future. COMPLETE finalizes the workflow; ABORT exits with failure and retains artifacts/logs.

### Faceted Prompting (Personas/Policies/Knowledge)
Prompts are composed from independent facets to improve reuse and control:
- persona: role definition (planner, coder, reviewer, security, performance)
- policy: approval criteria, risk thresholds, allowed operations
- knowledge: domain notes, codebase guidance
- instruction: step‑specific instructions and guardrails

Facet resolution order (planned): builtin < project (`.ccswarm/facets/`) < user (`~/.ccswarm/facets/`). Provide `ccswarm eject <flow|facet>` to simplify customization.

### Repertoire Packages (shared workflows)
Install workflow/facet sets from external Git repositories with `ccswarm repertoire add <git-url>`. Installed to `~/.ccswarm/repertoire/@{owner}/{repo}/`. Resolve across three layers (builtin → project → user) with nearest override wins.

### Directory Layout (recommended)
- `~/.ccswarm/`
  - `config.yaml` — default provider/model/language
  - `flows/` — user-defined flows
  - `facets/` — user-defined facets (persona/policy/knowledge/instruction)
  - `repertoire/` — installed packages
- `./.ccswarm/` (project)
  - `config.yaml`
  - `facets/` (project-local facets)
  - `tasks.yaml` queue and `tasks/` specs
  - `runs/` execution reports and NDJSON logs
  - `logs/` runtime logs

### Provider Sandbox & Permissions (enhancement)
- permission modes: view|edit|exec per stage; enforce guardrails (write scopes/allowed commands/network policy).
- trusted directories declared in global/project config.
- audit trail: record all stage I/O to NDJSON for reproducibility.

### Quality Gates and Review Loops
- Parallel reviewers (architecture/security/performance) supported in future.
- Aggregation rules (all‑approve/weighted/threshold) declared in flow rules.
- REJECT/NEEDS_FIX routes back to implement to form fix loop.

## Acceptance Criteria (Flows/Facets/Repertoire)
- AC‑P1: `ccswarm flow list/show/eject` works across builtin + project + user layers.
- AC‑P2: stage `edit`/permission reflects in runtime guards; forbidden ops are blocked.
- AC‑P3: `.ccswarm/tasks.yaml` queued tasks can be processed sequentially/parallel by a `ccswarm run` path (future).
- AC‑P4: Execution traces (NDJSON) are saved under `.ccswarm/runs/*`.
- AC‑P5: `ccswarm repertoire add <git-url>` installs packages resolvable by flow loader.

## Harness Engineering

See docs/HARNESS_ENGINEERING.md for scenario format and CLI. Harness executes flow pipelines against predefined tasks and verifies assertions, writing a consolidated report suitable for CI.

## Scheduling & Assignment (Advanced)

- Matching policy computes suitability score per agent: skills match, past success, queue length, SLA.
- Policies: greedy, Sangha consensus (default workflow governance), auction (future bid by cost/latency/confidence).
- DAG barriers respected: downstream tasks activate only when prerequisites complete.
- Acceptance criteria: selected policy and decision rationale recorded in events.ndjson.

## Budgets & Cost Controls

- Per‑project and per‑agent budgets: tokens, CPU time, wall time.
- Budget‑aware parallelism: cap concurrency based on remaining budget and task priority.
- Stop conditions: no progress, budget exhausted, excessive revert rate.

## HITL Approvals

- Gates: plan / risky‑edit / deploy / merge.
- CLI: `ccswarm approve <gate> --id <run|task>` or `--reject --reason`.
- Events: hitl.request, hitl.decision with actor and comment.

## Event Schema (NDJSON)

- Common fields: { ts, level, run_id, agent, stage?, task_id?, message }.
- Types: stage.start|end, task.enqueue|start|end, review.request|result, hitl.request|decision, provider.call|error.
- Storage: `.ccswarm/runs/<run-id>/{events.ndjson, summary.json}`.

## Sangha/Extend/Search/Evolution (CLI Spec)

This section re‑introduces four CLI surfaces with takt‑aligned patterns. They are designed as thin handlers that delegate business logic to coordination modules, store state as JSON files, and support dual output (text/JSON).

### takt Patterns Applied
- Verb‑first positional args: primary input is positional (e.g., `search docs "query"`).
- Minimal, action‑focused descriptions: short help texts with context hints.
- Feature layer separation: CLI handlers dispatch; logic lives in coordination/analytics modules.
- Filesystem as state store: JSON files under `coordination/{proposals,extensions,agent-status,task-queue}/` and `/.ccswarm/runs/`.
- Format‑aware output: `self.json_output` returns `{ status, message, data }`.
- Grouped handlers: Sangha+Extend (write ops), Search+Evolution (read‑only analytics).

### Command Overview
- sangha
  - propose: `--title <str> --description <str> --proposal-type <feature|refactor|policy|tooling=feature>`
  - vote: `<id> [--approve] [--reason <str>]` (absence of `--approve` implies reject)
  - list: `[--status <open|accepted|rejected>]`
  - status: `<id>`
- extend
  - propose: `--title <str> --description <str> [--agent <frontend|backend|devops|qa|all=all>] [--auto-sangha]`
  - auto-propose: `[--agent <frontend|backend|devops|qa|all=all>] [--reason <str>] [--no-auto-sangha]`
  - list: `[--status <proposed|approved|active|deprecated>]`
  - status: `<id>`
  - history: `[<limit=20>]`
- search
  - docs: `<query> [--limit <n=10>]` (search in `docs/` tree)
  - code: `<query> [--glob <pattern>] [--limit <n=10>]` (search in repo; glob like `*.rs`)
- evolution
  - metrics: `[--agent <name>] [--format <text|json=text>]`
  - patterns: `[--agent <name>] [--limit <n=50>]`
  - report: `[--format <text|json|markdown=text>]`

### Storage Layout (Filesystem as Source of Truth)
- `coordination/proposals/{id}.json`
  - `{ id, title, description, type, created_at, status, votes: [{agent, approve, reason, ts}] }`
- `coordination/extensions/{id}.json`
  - `{ id, title, description, agent, status, sangha_proposal_id?, source?, history?: [{event, detail, ts}] }`
- `coordination/agent-status/{agent}.json`
  - `{ agent_id, status, metrics, timestamp }`
- `coordination/task-queue/{id}.json`
  - `{ id, description, priority, type, state, timestamps }`
- `./.ccswarm/runs/*/*.ndjson` — stage/task traces for audit

JSON writes use pretty‑printed `serde_json`, filenames are UUID‑based, and directories are auto‑created with `tokio::fs::create_dir_all`.

### Output Contracts
- Text mode: concise colored lines for human consumption.
- JSON mode (`--json` or config): `{ "status": "success|error", "message": "...", "data": { ... } }`.

### Acceptance Criteria (CLI)
- AC‑C1: `ccswarm sangha --help` 等で上記サブコマンド/引数が表示される。
- AC‑C2: `sangha propose` 実行で `coordination/proposals/` に JSON が作成され、`sangha list/status` で参照できる。
- AC‑C3: `sangha vote <id> [--approve]` 実行で既存 proposal に投票が追記され、集計が反映される。
- AC‑C4: `extend propose` 実行で `coordination/extensions/` に JSON が作成され、`extend list/status/history` で参照できる。
- AC‑C4b: `extend auto-propose` は extension JSON と linked Sangha proposal JSON を作成し、extension に `sangha_proposal_id` を記録する。
- AC‑C5: `search docs/code` はローカル検索（docs/ とリポジトリ）で結果を返し、外部 API を不要とする。
- AC‑C6: `evolution metrics/patterns/report` は `coordination/*` の履歴から集計/分析を生成し、`--format json` で構造化出力を返す。

### Notes
- Search は ripgrep 互換の `--glob` を採用。
- Vote は単一 `--approve` フラグで衝突を排除（未指定は reject）。
- 4 コマンドはファイル I/O のみで実行可能（実行エンジン不要）。

## Configuration

### Project Configuration (ccswarm.json)
Project configurations are stored in the project directory, not within the ccswarm codebase:

```json
{
  "project": {
    "name": "MyProject"
  },
  "agents": [
    {
      "name": "frontend-specialist",
      "role": "Frontend"
    }
  ]
}
```

Example configurations can be found in `crates/ccswarm/examples_disabled/`.

### Environment Variables
- `ANTHROPIC_API_KEY`: Required for Claude API-backed flows when Claude Code
  CLI is not already authenticated
- `RUST_LOG`: Control logging verbosity
- `CCSWARM_HOME`: Configuration directory (default: ~/.ccswarm)

## Usage Examples

### Installation and Building
```bash
# Clone the repository
git clone https://github.com/nwiizo/ccswarm
cd ccswarm

# Build the workspace (builds all crates)
cargo build --release

# Install ccswarm globally
cargo install --path crates/ccswarm

# Or run directly from workspace
cargo run --package ccswarm -- init --name "TodoApp"
```

### Basic Workflow
```bash
# Initialize project
ccswarm init --name "TodoApp" --agents frontend,backend

# Execute a task through a flow pipeline
ccswarm pipeline --task "Add user authentication" --flow default

# Accumulate tasks and drain in batch
ccswarm queue add "Add login"
ccswarm queue add --from-issue 42
ccswarm queue drain

# Follow a running pipeline / inspect a past run
ccswarm tail
ccswarm cost <run-id>
ccswarm run diff <run-a> <run-b>
ccswarm replay <run-id>

# Generate agent definitions from facets
ccswarm agent-gen generate

# Run harness scenarios (regression tests of flows)
ccswarm harness run

# Environment + provider CLI probe
ccswarm doctor
```

### Advanced Usage

Sangha / extend / evolution / search live under the `lab` subcommand — they are
research features that sit beside the core JTBD flow.

```bash
# Sangha proposal and voting
ccswarm lab sangha propose --title "Add GraphQL support" --description "..."
ccswarm lab sangha vote <proposal-id> --approve --reason "Improves API flexibility"

# Agent extension proposals
ccswarm lab extend propose --title "GraphQL resolver" --description "..." --agent backend

# HITL approval workflow (core, not lab)
ccswarm approve plan --id run-abc123
ccswarm approve deploy --id task-456 --reject --reason "needs more tests"

# Evolution metrics and analysis
ccswarm lab evolution report --format json

# Autonomous mode (drain queue, auto-commit, auto-PR)
ccswarm auto --watch --stop-on-error
```

### Development Commands
```bash
# Run tests for all workspace crates
cargo test --workspace

# Run tests for specific crate
cargo test --package ccswarm
cargo test --package ai-session

# Build release binaries for all crates
cargo build --release --workspace

# Run ccswarm directly from source
cargo run --package ccswarm -- <command>

# Run ai-session-server from source
cargo run --package ai-session --bin server
```

## Version History

### v0.9.0 (Current)
- Sangha consensus is a workflow primitive (`sangha:` stage) with quorum
  approval and independent member decisions.
- Builtin `default` flow is Sangha-first: plan → quorum → implement → review
  → fix, while `team-dynamic` remains available for leader/worker
  compatibility.
- Governed auto-extension: `extend auto-propose` and `extend propose
  --auto-sangha` create linked Sangha proposals before extension adoption.
- ccswarm live provider calls use ai-session execution primitives for prompt
  sizing, working-directory context, cwd enforcement, structured subprocess
  execution, output parsing, and persistence.
- Pipeline `--model` and `--isolate` are forwarded into the live
  `AISessionBridge` execution options.

### v0.8.0
- takt feature adoption + codex first-class support
- Global `--provider <claude|codex>` flag; codex JSONL telemetry (`CCSWARM_CODEX_JSON=1`) and session resume (`codex exec resume`) — multi-turn works on codex
- Rate-limit fallback chain (flow-level `on_rate_limit`) and stage `promotion` rules (provider/model escalation from the Nth visit)
- Command quality gates (stage `gates:` run commands post-agent; failures feed back into the same stage)
- 3-layer facet resolution (repertoire < user `~/.ccswarm/facets` < project); fixed live runs not loading custom facets at all
- LLM-backed ai() judge (`CCSWARM_LLM_JUDGE=1`) with the lexical heuristic as offline fallback
- team_leader orchestrator-worker: runtime task decomposition into parallel workers; builtin `team-dynamic` flow
- Fixed CompoundCondition YAML round-trip (`flow check` on flows with all()/any() rules)
- Final readiness fixes: parseable `--json` stdout for init/queue workflows,
  provider validation for flow-level fallback/promotion fields, non-duplicated
  live prompt construction, UTF-8-safe gate-output truncation, and scaffold
  failure/provider propagation fixes informed by an actual static app creation
  smoke run

### v0.7.0
- Audit follow-up: wired implemented-but-unreachable features, deleted the rest
- Deleted unwired workflow modules (~4.2k LOC): arpeggio, watch, legacy DAG engine (graph/node/execution), github_issue
- Loop guard: `max_stage_visits` flow field bounds per-stage revisits; `flow check` reports structural cycles
- Parallel `all()`/`any()` aggregation wired into rule evaluation (was judge-only)
- HITL commit gate: `auto --require-approval` / `queue drain --require-approval` block before commit until `ccswarm approve commit --id <run-id>`; HitlRequest/HitlDecision events recorded
- Stream-json telemetry: tool names, real token counts, and cost recorded as ProviderCall events; `ccswarm cost` uses real counts when stream-json is on
- Optional `otel` feature: OTLP span export + flow.run/flow.stage spans
- docs/COMPETITIVE_LANDSCAPE.md added

### v0.6.0
- Major restructuring: project identity shifted from "AI multi-agent orchestrator" to "AI Agent Workflow DevOps toolchain"
- Deleted ~22.6k LOC: orchestrator/, providers/, acp_claude/, subagent/, execution/, tui/, template/, mcp/, ipc/, auto_accept/
- Wired EventRecorder into FlowEngine with NDJSON events at .ccswarm/runs/
- Added AISessionBridge with --resume, --agent routing, retry with exponential backoff
- Added agent-gen command for generating/validating .claude/agents/*.md from facets
- Enriched builtin persona system prompts
- Added Stage fields: agent, working_dir, retry_delay_ms
- Stage context passing between steps
- Duration tracking in events
- Harness scenarios added
- Removed CLI commands: Start, Stop, Status, Tui, Verify, Review, Delegate, Session, Resource, AutoCreate, Quality, Template

### v0.5.0
- Large-scale codebase reduction (~22.8k LOC deleted in 6 phases)
- Workspace restructuring

### v0.3.8
- Observability/Tracing with OpenTelemetry and Langfuse support
- Human-in-the-Loop approval workflows
- Graph Workflow Engine with DAG-based execution

### v0.3.0
- Sangha collective intelligence
- Democratic decision-making
- Extension framework

See CHANGELOG.md for complete version history.
