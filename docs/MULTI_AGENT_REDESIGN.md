# ccswarm Multi-Agent Redesign

Status: design note, not implemented behavior.
Last verified: 2026-07-26.

This document reorganizes the multi-agent roadmap around current evidence from
the codebase and current provider documentation. It is a companion to
[CCSWARM_PRODUCT_ABSTRACTION_PLAN.md](CCSWARM_PRODUCT_ABSTRACTION_PLAN.md),
which defines the broader order/run/report/gate/verdict vocabulary and M0-M6
milestones.

The canonical v0.10.0 release scope and sequencing live in
[ROADMAP.md](ROADMAP.md). The milestones here are supporting design direction,
not additional release requirements.

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
| Parallel subagents | Available by explicit request or applicable repository instructions; built-in `default`, `worker`, `explorer`; custom agents in `~/.codex/agents/` or `.codex/agents/` | Custom subagents with separate contexts, tool access, and permissions | Not integrated; ccswarm fans out stages itself |
| Subagent config | `[agents]` settings include `enabled`, `max_concurrent_threads_per_session`, default model/reasoning effort, and interrupt-message control; `max_threads` remains a legacy alias | Subagents are configured as Claude Code custom subagents | Not modeled |
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

## 8. Ecosystem Findings Through Job Theory

The primary job is not "run more agents." It is:

> When work is too broad or uncertain for one execution path, help me divide it
> into independently useful contributions, see whether the team is converging,
> intervene at the right level, and recover without paying for completed work
> again.

This job has four outcome dimensions:

| Outcome | User measure |
| --- | --- |
| Decomposition confidence | Every child has a bounded objective, owner, dependencies, and expected evidence. |
| Coordination clarity | The user can see who is active, blocked, idle, failed, or waiting and why. |
| Intervention precision | The user can steer one child, change one task, or approve one boundary without restarting the team. |
| Recovery economy | Completed child work and team state survive interruption, failure, and process restart. |

Current multi-agent systems solve different parts of that job. The roadmap
should absorb the mechanisms below without copying their product vocabulary or
making a framework dependency part of ccswarm's domain model.

| System | Job users hire it for | Transferable mechanism | ccswarm decision |
| --- | --- | --- | --- |
| Claude Code agent teams | Parallel coding sessions that can coordinate and be steered directly | Shared claimable tasks with dependencies, teammate mailboxes, plan approval, completion hooks, and visible per-agent state | Add a provider-neutral task graph and mailbox events to Layer A; keep team creation opt-in and isolate write ownership because the provider feature remains experimental |
| OpenAI Agents SDK | Compose specialists while choosing who owns the user interaction | Explicit manager-as-tools versus handoff control, filtered handoff context, guardrails, sessions, and resumable human approval | Model control ownership and context projection explicitly; do not treat every delegation as a full transfer |
| Microsoft AutoGen | Select among group-chat, selector, and decentralized handoff teams | Composable termination conditions, streamed team events, team save/load, and human handoff | Add typed stop policies and portable team checkpoints before adding more scheduling strategies |
| LangGraph | Run long-lived stateful agent graphs that pause and resume safely | Durable execution, checkpointers, streamed state, and interrupt/resume commands | Make child checkpoints and interrupts part of the run model instead of an implementation detail |
| CrewAI | Combine autonomous role-based collaboration with deterministic business flows | Expected-output task contracts, sequential or hierarchical crews, typed flow state, human input, and a clear autonomy-versus-control choice | Reuse reports and verdicts as task contracts; expose an explicit deterministic/native/hybrid policy |
| Google ADK | Compose deterministic sequential, parallel, and loop workflows, then evolve to graph or dynamic routing | Separate workflow agents from LLM agents, shared workflow state, graph routes, and A2A integration | Keep deterministic stage algebra as the default and make model-selected routing an explicit capability |

### Cross-System Product Decisions

The comparison changes the roadmap in six concrete ways:

1. A team needs a durable task graph, not only a vector of child prompts.
2. Control ownership must distinguish manager calls, full handoffs, and
   provider-native delegation.
3. Termination is a typed policy combining success, budget, time, turns,
   failure, cancellation, and human handoff.
4. Team state, mailboxes, and per-child evidence must be checkpointed together.
5. Direct steering and approval should target a child or task without
   invalidating unrelated completed work.
6. Team value should be measured by accepted outcomes and wall-clock reduction
   after coordination cost, not by agent count.

## 9. Roadmap

### M0: Clean and supervise

- Forward Claude/Codex stream events as `MovementHeartbeat`.
- Add timeout cleanup and stale-run reconciliation.
- Record a terminal reason for every child, including completed, failed,
  canceled, timed out, budget exhausted, and handed off.

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

### M3: Task and verdict contracts

- Add a durable `TeamTask` graph with owner, dependencies, status, expected
  reports, and file-ownership hints.
- Add `StageVerdict`.
- Add `Grade` as a read-only fresh-context stage.
- Route on structured verdicts first, with legacy rules as fallback.
- Reject duplicate claims and block tasks whose dependencies are incomplete.

### M4: Team checkpoint and control

- Checkpoint the task graph, child session identifiers, mailbox cursor, reports,
  and aggregate verdict at every task transition.
- Add typed, composable stop policies for time, turns, cost, failure, quorum,
  success, cancellation, and human handoff.
- Stream child lifecycle and mailbox events into the parent run.
- Support child-targeted instruct, interrupt, approve, retry, and resume.

### M5: Multi-agent selection

- Add explicit control ownership: `manager`, `handoff`, or `native`.
- Add `delegate: ccswarm | native | auto`.
- Support native delegation for read-heavy `Fork` first.
- Keep `Govern` on Layer A unless the user explicitly accepts weaker
  independence.
- Render the selected layer, control owner, context projection, budget, and
  fallback before execution.

### M6: Long-lived and distributed backends

- Add Codex app-server integration for rich clients or long-lived local
  orchestration.
- Add Codex MCP-server integration when using Agents SDK as the outer
  orchestrator.
- Consider Claude Agent SDK integration separately from the CLI backend.
- Map remote A2A agents onto the same task, stop-policy, checkpoint, and event
  contracts instead of creating a second team model.

### M7: Outcome-aware team sizing

- Estimate whether parallelism can reduce wall-clock time after coordination
  overhead and dependency depth.
- Default to one executor for sequential or same-file work.
- Bound active children by budget, provider limits, file ownership, and
  available independent tasks.
- Compare accepted outcome, elapsed time, intervention count, retries, and cost
  against the single-agent baseline.

Acceptance:

- The operator can explain why a team was chosen, why each child exists, and
  what would stop it.
- A process restart preserves completed tasks and resumes only incomplete work.
- One failed or redirected child does not discard unrelated accepted reports.
- Increasing team size without measurable outcome benefit is visible and never
  becomes the automatic default.

## 10. Non-goals

- Do not replace ccswarm's flow engine with provider subagents.
- Do not auto-select native delegation for write-heavy code edits.
- Do not call a provider feature "stable" unless `doctor` proves it locally or
  current official documentation says so.
- Do not treat same-provider subagents as independent reviewers for governance
  decisions.
- Do not add scheduling variants before task contracts, stop policies, and
  checkpoint recovery are reliable.
- Do not build a GUI before `desk --json` and the event model are stable.

## 11. Source Snapshot

Local evidence:

- `crates/ccswarm/src/providers/mod.rs`
- `crates/ccswarm/src/providers/claude.rs`
- `crates/ccswarm/src/providers/codex.rs`
- `crates/ccswarm/src/session/bridge.rs`
- `crates/ccswarm/src/workflow/flow.rs`

External source pages checked on 2026-07-26:

- [Codex Subagents](https://developers.openai.com/codex/subagents)
- [Codex subagent concepts](https://developers.openai.com/codex/concepts/subagents)
- [Codex non-interactive mode](https://developers.openai.com/codex/noninteractive)
- [Codex app-server](https://developers.openai.com/codex/app-server)
- [Codex MCP](https://developers.openai.com/codex/mcp)
- [Claude Code subagents](https://docs.anthropic.com/en/docs/claude-code/sub-agents)
- [Claude Code agent teams](https://code.claude.com/docs/en/agent-teams)
- [Claude Code CLI reference](https://docs.anthropic.com/en/docs/claude-code/cli-reference)
- [Claude Agent SDK overview](https://docs.anthropic.com/en/docs/claude-code/sdk)
- [OpenAI Agents SDK orchestration](https://openai.github.io/openai-agents-python/multi_agent/)
- [OpenAI Agents SDK handoffs](https://openai.github.io/openai-agents-python/handoffs/)
- [Microsoft AutoGen teams](https://microsoft.github.io/autogen/stable/reference/python/autogen_agentchat.teams.html)
- [Microsoft AutoGen termination](https://microsoft.github.io/autogen/stable/user-guide/agentchat-user-guide/tutorial/termination.html)
- [LangGraph overview](https://docs.langchain.com/oss/python/langgraph/overview)
- [LangGraph interrupts](https://docs.langchain.com/oss/python/langgraph/interrupts)
- [CrewAI documentation](https://docs.crewai.com/)
- [Google ADK collaborative workflows](https://adk.dev/workflows/collaboration/)
- [Google ADK graph workflows](https://adk.dev/graphs/)
