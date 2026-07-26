# ccswarm Multi-Agent Redesign

Status: design note, not implemented behavior.
Last verified: 2026-07-06.

This document reorganizes the multi-agent roadmap around current evidence from
the codebase and current provider documentation. It is a companion to
[CCSWARM_PRODUCT_ABSTRACTION_PLAN.md](CCSWARM_PRODUCT_ABSTRACTION_PLAN.md),
which defines the broader order/run/report/gate/verdict vocabulary and M0-M6
milestones.

The central change is architectural: keep ccswarm's deterministic workflow
engine as the outer control plane, but add explicit capability checks for
provider-native subagents, structured output, resumable sessions, and
long-lived programmatic interfaces.

## 0. Current Reality

The repository does not yet have `StageKind`, `ProviderCapabilities`, native
delegation, a `Grade` stage, or structured `StageVerdict` routing. Those are
roadmap items.

What exists today:

| Area | Current implementation | Evidence |
|---|---|---|
| Provider boundary | `AgentProvider` builds provider CLI commands; no capability API yet | `crates/ccswarm/src/providers/mod.rs` |
| Claude execution | `claude -p`, optional stream-json, `--agent`, `--session-id`, `--continue`, `--worktree` | `crates/ccswarm/src/providers/claude.rs` |
| Codex execution | `codex exec`, optional `--json`, `codex exec resume <thread-id>` | `crates/ccswarm/src/providers/codex.rs` |
| Stream parsing | Claude stream-json and Codex JSONL are projected back to text + metadata | `crates/ccswarm/src/session/bridge.rs` |
| Team leader | One decomposition wave, then parallel synthesized worker stages | `crates/ccswarm/src/workflow/flow.rs` |
| Sangha | Parallel members vote with `SANGHA_DECISION=*` text lines | `crates/ccswarm/src/workflow/flow.rs` |
| Routing | Aggregate/string/tag/LLM-judge routing; no first-class verdict object | `crates/ccswarm/src/workflow/flow.rs` |

## 1. Verified Provider Capabilities

Provider capabilities move quickly. Treat this table as a dated snapshot and
probe the local CLI in `ccswarm doctor` before relying on a capability.

| Capability | Codex | Claude Code | ccswarm status |
|---|---|---|---|
| Parallel subagents | Available by explicit user request; built-in `default`, `worker`, `explorer`; custom agents in `~/.codex/agents/` or `.codex/agents/` | Custom subagents with separate contexts, tool access, and permissions | Not integrated; ccswarm fans out stages itself |
| Subagent config | `[agents]` global settings include `max_threads`, `max_depth`, `job_max_runtime_seconds`; CSV fan-out helpers are probe-gated and should be treated as experimental until `doctor` confirms local support | Subagents are configured as Claude Code custom subagents | Not modeled |
| Non-interactive execution | `codex exec`; `--json`; `--output-schema`; `resume --last` or `resume <SESSION_ID>` | `claude -p`; `--output-format text/json/stream-json`; `--json-schema` | Partially integrated |
| Long-lived interface | `codex app-server` JSON-RPC; `codex mcp-server` for Agents SDK integration | Agent SDK for Python/TypeScript | Not integrated |
| MCP | Codex CLI and IDE support stdio and streamable HTTP MCP servers | Claude Code supports MCP | Outside ccswarm provider capability model |
| Streaming telemetry | `codex exec --json` emits thread/turn/item events and usage | `--output-format stream-json --verbose` emits stream events | Partially parsed; not yet heartbeat/supervision |

Avoid undocumented names in roadmap text. In particular, do not claim a stable
Codex `multi_agent` mode or an `exec-server` unless a current CLI probe or
official page proves it. The documented surfaces are subagent workflows,
`codex exec`, `codex app-server`, and `codex mcp-server`.

## 2. Design Principle

ccswarm should distinguish two layers:

```text
Layer A: ccswarm orchestration
  Owns flow YAML, stages, gates, reports, event logs, retry, recovery,
  cross-provider independence, and auditability.

Layer B: provider-native subagency
  Lives inside one stage. The provider spawns and manages child agents,
  returns a consolidated result, and may reuse provider-side context/cache.
```

Default rule:

- Use Layer A when results must be independently auditable, cross-provider, or
  recoverable per child.
- Use Layer B when all children share one provider, the task is read-heavy or
  parallelizable, and ccswarm only needs the aggregate result.
- Never infer Layer B support from provider name alone. Probe capability first
  and fall back to Layer A.

## 3. Target Provider Capability Model

Add a capability API before adding new dispatch behavior:

```rust
pub struct ProviderCapabilities {
    pub native_subagents: CapabilityState,
    pub structured_output: CapabilityState,
    pub resumable_session: CapabilityState,
    pub persistent_app_server: CapabilityState,
    pub mcp_server: CapabilityState,
    pub stream_events: CapabilityState,
    pub worktree_isolation: CapabilityState,
}

pub enum CapabilityState {
    Supported,
    Experimental,
    Unsupported,
    Unknown,
}
```

Capabilities should be discovered by `ccswarm doctor` and cached in run
metadata. Hardcoding capability assumptions will make the roadmap stale again.

Initial probes:

| Probe | Expected evidence |
|---|---|
| `codex exec --help` | `--json`, `--output-schema`, `resume` |
| `codex app-server --help` | app-server availability and transport flags |
| `codex mcp-server --help` | MCP server availability |
| `claude -p --help` or `claude --help` | `--output-format`, `--json-schema`, `--include-partial-messages` |
| Provider subagent config files | `.codex/agents/*.toml`, `.claude/agents/*` |

## 4. Stage Algebra

Introduce an internal `StageKind` boundary as a refactor before changing
behavior:

```rust
pub enum StageKind {
    Execute,
    Goal,
    Grade,
    Fork,
    Lead,
    Govern,
    Call,
    Batch,
}
```

`Stage::kind()` should initially derive from existing YAML fields:

| YAML shape | StageKind |
|---|---|
| `call:` | `Call` |
| `team_leader:` | `Lead` |
| `sangha:` | `Govern` |
| `parallel: true` with children | `Fork` |
| `grade:` | `Grade` |
| `batch:` | `Batch` |
| default | `Execute` |

The first implementation slice should preserve behavior. The value is in
isolating dispatch and making every stage return the same contract.

## 5. Verdict Contract

Routing should eventually read structured verdicts before lexical heuristics.

```json
{
  "status": "success",
  "verdict": "approved",
  "confidence": 0.82,
  "needs_human": false,
  "rubric_scores": [
    {
      "criterion": "tests pass",
      "met": true,
      "evidence": "reports/test.log:exit 0"
    }
  ],
  "reports": ["00-plan.md"],
  "recovery": null
}
```

Provider mapping:

- Codex: use `codex exec --output-schema <schema>` when available.
- Claude Code: use `claude -p --json-schema '<schema>'` when available.
- A2A: accept a JSON part from the remote agent if the A2A server supports it.
- Fallback: request fenced JSON, parse strictly, and mark capability as
  `Unknown` or `Unsupported` in metadata.

New routing priority:

1. Structured `StageVerdict`.
2. Aggregated verdict from `Fork`, `Lead`, or `Govern`.
3. Legacy `[STEP:N]` and simple string rules.
4. Opt-in LLM judge.
5. Explicit failure when rules exist but no condition matches.

## 6. Supervised Execution

The execution backend should move from "spawn and await" toward supervised
sessions with heartbeats.

```rust
pub enum ExecutionBackend {
    OneShotCli,
    ResumableCli,
    AppServer,
    McpServer,
}
```

Required lifecycle events:

1. Provider call started with PID, thread ID, session ID, and prompt path when
   available.
2. Heartbeat from provider stream events or a synthetic interval.
3. Timeout event with provider, stage, elapsed time, and cleanup status.
4. Provider call completed with exit status, usage, cost where available, and
   parsed session ID.
5. Startup reconciliation: stale `running` runs become `partial`.

This is the M0 priority because it fixes the class of bugs where a run gets
stuck at `movement_start` with no evidence of whether the provider is still
running.

## 7. Native Delegation Mapping

`delegate: auto` should exist only after provider capabilities are available.

| ccswarm kind | Layer A behavior | Layer B candidate |
|---|---|---|
| `Goal` | One provider call with full brief and report contract | Same provider call; no native subagent needed |
| `Grade` | Fresh independent provider call against rubric | Read-only subagent when independence is not critical |
| `Fork` | `run_stages_parallel` | Provider subagents for read-heavy exploration or independent checks |
| `Lead` | Leader decomposes, workers run, leader may dispatch another wave | Provider subagents if all work stays on one provider |
| `Govern` | Independent members, quorum, preferably cross-provider | Usually keep Layer A; native subagents weaken independence |
| `Batch` | Stage template per item | Provider-native CSV/batch subagent helper when proven available |

Native delegation is not always better. It can reduce CLI cold starts and keep
the main context cleaner, but it also hides per-child recovery and can create
write conflicts. Use it first for read-heavy exploration, test/log analysis,
and independent review slices.

## 8. Roadmap

### M0: Clean and supervise

- Remove generated `target2/` noise from the worktree or ignore it.
- Forward Claude/Codex stream events as `MovementHeartbeat`.
- Add timeout cleanup and stale-run reconciliation.

### M1: Capability discovery

- Add `ProviderCapabilities` and `CapabilityState`.
- Extend `ccswarm doctor` to probe Codex, Claude, A2A, app-server, MCP server,
  and structured output support.
- Store capability snapshots in run metadata.

### M2: Stage boundary

- Add `StageKind` and refactor `execute_movement` dispatch with no behavior
  change.
- Add tests that prove existing `call`, `team_leader`, `sangha`, and
  `parallel` flows still route identically.

### M3: Structured verdicts

- Add `StageVerdict`.
- Add `Grade` as a read-only fresh-context stage.
- Route on structured verdicts first, with legacy rules as fallback.

### M4: Multi-agent selection

- Add `delegate: ccswarm | native | auto`.
- Support native delegation for read-heavy `Fork` first.
- Keep `Govern` on Layer A unless the user explicitly accepts weaker
  independence.

### M5: Long-lived backends

- Add Codex app-server integration for rich clients or long-lived local
  orchestration.
- Add Codex MCP-server integration when using Agents SDK as the outer
  orchestrator.
- Consider Claude Agent SDK integration separately from the CLI backend.

## 9. Non-goals

- Do not replace ccswarm's flow engine with provider subagents.
- Do not auto-select native delegation for write-heavy code edits.
- Do not call a provider feature "stable" unless `doctor` proves it locally or
  current official documentation says so.
- Do not treat same-provider subagents as independent reviewers for governance
  decisions.
- Do not build a GUI before `desk --json` and the event model are stable.

## 10. Source Snapshot

Local evidence:

- `crates/ccswarm/src/providers/mod.rs`
- `crates/ccswarm/src/providers/claude.rs`
- `crates/ccswarm/src/providers/codex.rs`
- `crates/ccswarm/src/session/bridge.rs`
- `crates/ccswarm/src/workflow/flow.rs`

External source pages checked on 2026-07-06:

- Codex Subagents: https://developers.openai.com/codex/subagents
- Codex subagent concepts: https://developers.openai.com/codex/concepts/subagents
- Codex non-interactive mode: https://developers.openai.com/codex/noninteractive
- Codex app-server: https://developers.openai.com/codex/app-server
- Codex MCP: https://developers.openai.com/codex/mcp
- Claude Code subagents: https://docs.anthropic.com/en/docs/claude-code/sub-agents
- Claude Code CLI reference: https://docs.anthropic.com/en/docs/claude-code/cli-reference
- Claude Agent SDK overview: https://docs.anthropic.com/en/docs/claude-code/sdk
