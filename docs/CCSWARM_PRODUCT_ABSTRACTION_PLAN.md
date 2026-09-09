# ccswarm Product Abstraction Plan

This document turns an end-to-end operator workflow audit and the ccswarm
dogfood run into a ccswarm-native product plan. It preserves useful mechanics
while naming and shaping them around ccswarm's own model: flows, stages,
Sangha governance, orders, runs, and A2A/local execution.

The canonical v0.10.0 release scope and sequencing live in
[ROADMAP.md](ROADMAP.md). The milestones here describe the longer product
direction and do not add release requirements.

[Sangha Product Core](SANGHA_PRODUCT_CORE.md) selects the core capabilities
from this broader journey: evidence-based assessment, objection resolution,
bounded revision, decision reports, and recovery. Broader order/desk surfaces
remain supporting design, not prerequisites for that core.

## Decision And Work Contract

Recommendation: optimize the complete operator journey around one primary job,
then adopt workflow mechanics only where they improve a measurable outcome for
that job. Do not pursue command-by-command parity.

- Purpose: let a developer turn uncertain work into a trusted, reviewable
  change without continuously supervising AI agents.
- Scope: first run, intake, isolation, execution, progress, intervention,
  review, recovery, integration, and automation entrypoints.
- Success: less time to a usable brief, fewer manual interventions per accepted
  change, no ambiguous running state, bounded recovery cost, and an auditable
  merge decision.
- Constraints: keep ccswarm's flow/stage/Sangha vocabulary, preserve
  provider independence, default to safe local behavior, and make every
  autonomous action observable and reversible.

The implementation milestones below are ordered by this work contract. A
candidate feature is not a priority merely because another tool implements it.

## Dogfood Finding

The scaffold dogfood run on a static habit-tracker app exposed a product gap
that must be fixed before deeper workflow sophistication:

- `ccswarm scaffold --flow quick --provider claude` created partial artifacts
  but did not complete the stage.
- The event log stopped at `movement_start`, so the user could not tell whether
  the provider was still running, blocked, or orphaned.
- The partial artifact was useful, but ccswarm did not classify it as a
  recoverable partial run.

This makes observability and recovery the first abstraction layer. Advanced
queue, review, and escalation features become brittle if a live stage can hang
without a clear run state.

## Operator Journey Inputs

The operator journey audit covered the complete coding-agent lifecycle:

- conversational intake and instant `exec` turn an ambiguous request into a
  generated workflow;
- editable task directories accept reference files and image attachments;
- queued work runs in isolated shared clones, sequentially, concurrently, or
  through a resident watcher;
- one task-management surface supports diff, instruct, retry, requeue, merge,
  and delete;
- run checkpoints, per-phase usage, session keys, progress feedback, and
  structured outputs make long runs more inspectable;
- SDK and CLI providers can be routed per step, with rate-limit fallback and
  promotion;
- ACP and MCP entrypoints let other clients enqueue or execute work without
  shell-command coupling.

## Job Theory

### Primary Job

> When I have important development work whose path is not fully known, help me
> turn it into a verified, reviewable change while I stay in control without
> babysitting every agent turn.

The functional job is to move work from intent to an integration decision. The
emotional job is to reduce anxiety about silent stalls, uncontrolled edits,
cost, and lost context. The social job is to hand teammates a change whose
process and evidence are credible.

### Forces Of Progress

| Force | What the user experiences | Product response |
|---|---|---|
| Push | Repeating instructions, watching terminals, lost context, and skipped reviews | Durable orders, explicit stages, reports, and supervised runs |
| Pull | A repeatable path from rough request to PR-ready evidence | Assisted intake, safe defaults, desk actions, and structured verdicts |
| Anxiety | Wrong-tree edits, runaway cost, provider lock-in, and opaque automation | Isolation, budgets, capability-aware routing, heartbeats, and recovery points |
| Habit | Calling Codex or Claude directly and reviewing changes manually | Keep a one-shot path and make ccswarm add value without requiring YAML first |

### End-To-End Job Map

| Job step | Desired outcome | Current ccswarm gap | Recommended capability |
|---|---|---|---|
| Define | Turn rough intent into a testable brief quickly | Interactive entry does not create a durable, editable contract | Assisted intake that writes an order brief and acceptance criteria |
| Prepare | Attach the right code, issue, image, and policy context once | Queue items remain thinner than full work packages | Order directories with references and validated attachments |
| Choose | Select a safe workflow/provider without understanding every option | Provider and flow choices expose implementation detail early | Goal-oriented presets plus a fully rendered preview |
| Isolate | Know edits cannot damage unrelated work | Isolation exists but is not the default mental model everywhere | One isolation policy with current/worktree/shared-clone/container backends |
| Execute | Start work and leave it alone | A stage can become ambiguous after its start event | Supervised calls, heartbeats, timeouts, and cleanup |
| Monitor | Answer “what is happening and why?” in seconds | Events exist but phase/progress state is incomplete | Operator-oriented tail, phase state, ETA-free elapsed time, and cost |
| Intervene | Add direction without restarting from zero | Retry/replay surfaces are lower-level | Desk instruct/retry/resume with prior evidence preloaded |
| Judge | Trust that completion means requirements and gates passed | Text routing remains brittle | Reports, structured verdicts, command/HITL gates, and no-match failure |
| Integrate | Inspect, merge, archive, or reject safely | Queue, run, and git actions are fragmented | A single desk view with explicit reversible actions |
| Learn | Improve flow/provider choices from actual outcomes | Cost data is not tied to accepted outcomes | Outcome, intervention, recovery, latency, and cost analytics |

### Outcome Metrics

Use these measures to decide whether a borrowed interaction is actually useful:

| Outcome | Initial target |
|---|---|
| Time from rough request to editable brief | Under 5 minutes for the common path |
| Ambiguous run state | Zero runs left indefinitely at a start-only event |
| Recovery work | Resume from the last safe stage without repeating completed stages |
| Operator interventions | At most one required intervention on the default successful path |
| Decision evidence | Every completion links requirements, reports, gates, and changed files |
| Cost clarity | Token/time/cost visible by order, stage, phase, provider, and outcome |
| Isolation confidence | Queued work never edits the operator's current tree by default |

## Experience Adoption

### Absorb Directly

- Enqueue-first conversational intake with an explicit “run now” escape hatch.
- Editable task/order directories with reference files and image attachments.
- Immediate progress feedback for thinking, prompt preparation, provider calls,
  gates, and routing.
- A single operator surface for diff, instruct, retry, requeue, merge, archive,
  and delete.
- Safe isolated execution by default for queued work.
- Resident queue watching and bounded task concurrency after recovery is solid.
- Prompt/workflow preview before execution.
- Checkpointed resume and failure context carried into retry.
- Stdio automation tools for enqueue, status, and run-next.

### Adapt To ccswarm

| Operator experience | ccswarm adaptation |
|---|---|
| Conversational execution generates a workflow | `ccswarm intake --assist` chooses or parameterizes a validated flow first; generated flows remain an advanced opt-in |
| A unified task list manages task branches | `ccswarm desk` joins order, run, reports, gates, diff, and integration actions |
| Shared-clone implementation behind a `worktree` field | A truthful `isolation.mode` abstraction with backend-specific diagnostics |
| Provider auto-routing | Outcome-aware routing constrained by capability, budget, privacy, and reproducibility policies |
| ACP and MCP entrypoints | Start with a minimal MCP-compatible enqueue/status/run-next server; keep A2A for agent execution and add ACP only when a real client job requires it |
| Many builtin workflow variants | A small goal-oriented catalog with visible cost/quality tradeoffs and composable facets |
| Session keys and compaction | Explicit context lanes owned by `session`, with provider capability fallback recorded in run metadata |
| Phase usage telemetry | Tie phase cost to accepted/rejected/partial outcomes and operator interventions |

### Defer Or Reject

- Do not copy external command names or rename `flow` and `stage`.
- Do not generate arbitrary workflows by default before validation,
  supervision, and recovery are trustworthy.
- Do not add provider breadth ahead of reliable execution and evidence.
- Do not make automatic merge, push, or PR creation the hidden default.
- Do not build both ACP and MCP surfaces before the minimal integration job is
  validated.
- Do not multiply builtin workflows when facets and parameters can express the
  difference.
- Do not adopt a complex findings ledger until structured reports and verdicts
  demonstrate where simpler contracts fail.

## Product Vocabulary

ccswarm should keep its current language and deepen it:

| ccswarm term | Meaning |
|---|---|
| Flow | A reusable workflow state machine. |
| Stage | One unit in a flow. Stages may execute, judge, branch, fork, call subflows, or govern. |
| Order | A durable work request, richer than a queued string. |
| Run | A concrete execution attempt of an order or direct task. |
| Report | A named output artifact promised by a stage. |
| Gate | A machine or human condition that must pass before a stage or run advances. |
| Verdict | The normalized decision from a stage, judge, Sangha member, or gate. |
| Desk | The operator surface for completed, failed, partial, and pending work. |
| Escalation | A controlled provider/model/permission upgrade when progress stalls. |
| Recovery Point | The exact flow/stage/order state from which a run can resume or be retried. |

These names map proven operator mechanics to a coherent ccswarm surface:

| Operator mechanic | ccswarm abstraction |
|---|---|
| task directory + order.md | Order directory + `brief.md` |
| task list/list actions | Desk |
| output contracts | Reports |
| quality gates | Gates |
| provider promotion | Escalation |
| watch | Queue watch |
| workflow doctor | Flow check |
| prompt preview | Flow render |
| repertoire | Repertoire, retained |
| arpeggio | Batch stage |
| workflow_call | Subflow call, retained |
| rate_limit_fallback | Fallback chain, retained |

## Core Domain Model

### Order

An order is the persistent intent. It is not the run itself.

```yaml
id: 20260619-001-habit-current
title: Build habit tracker
status: pending # pending | claimed | running | completed | failed | partial | archived
flow: default
brief: .ccswarm/orders/20260619-001-habit-current/brief.md
references:
  - .ccswarm/orders/20260619-001-habit-current/wireframe.png
issue: 42
branch: ccswarm/20260619-001-habit-current
isolation:
  mode: worktree # current | worktree | shared-clone | container
created_at: "2026-06-19T00:00:00+09:00"
attempts:
  - run_id: 4a8d...
    status: partial
```

Order directories should live under `.ccswarm/orders/{slug}/`. The queue file
should index orders, not duplicate large task bodies.

### Run

A run is one execution attempt. It owns telemetry, stage outputs, reports, and
recovery metadata.

```yaml
run_id: 4a8d...
order_id: 20260619-001-habit-current
flow: default
status: running # running | completed | failed | partial | canceled | timed_out
current_stage: implement
recovery_point:
  stage: implement
  stage_visit: 1
  prompt_path: .ccswarm/runs/4a8d/context/implement.prompt.md
  partial_artifacts: true
provider:
  kind: claude
  model: sonnet
```

Every run must end in an explicit terminal state. If the process exits without a
stage end event, the next ccswarm invocation reconciles it to `partial` or
`failed` based on artifacts and process status.

### Stage

Stages should be treated as a small algebra of execution forms:

| Stage kind | Existing ccswarm surface | Target behavior |
|---|---|---|
| Execute | default stage | Provider/A2A call with context, retry, telemetry. |
| Judge | rules / AI judge | Produces a verdict without editing. |
| Gate | `gates` / approval | Blocks, reruns, or fails based on command/HITL result. |
| Fork | `parallel` | Runs children concurrently and aggregates verdicts. |
| Lead | `team_leader` | Decomposes and dispatches work in waves. |
| Govern | `sangha` | Collects independent votes with quorum. |
| Call | `call` | Invokes a subflow and maps its outcome. |
| Batch | new | Applies a stage template to many data items. |

This lets the engine dispatch by kind instead of accumulating special-case
fields inside one large execution path.

## CLI Surface

The target CLI should be understandable as four operator loops.

### 1. Intake Loop

Purpose: turn rough intent into an order.

```bash
ccswarm intake
ccswarm intake "Build billing export"
ccswarm intake --from-issue 42
ccswarm intake --file docs/request.md
```

Implementation path:

- Keep `queue add` as a compatibility alias.
- Store `brief.md` plus references in `.ccswarm/orders/{slug}/`.
- Support `--flow`, `--branch`, `--isolation`, and `--create-pr`.
- Interactive mode should produce a clear brief before enqueueing.

### 2. Execution Loop

Purpose: execute pending orders or a one-off task.

```bash
ccswarm queue drain
ccswarm queue watch
ccswarm pipeline --task "..."
ccswarm auto --watch
```

Implementation path:

- Add `queue watch` as a resident worker.
- Add concurrency with a bounded worker pool.
- Add startup reconciliation: stale `running` orders become `partial` or
  `failed`.
- Make `scaffold` use the same run-state reconciliation as pipeline.

### 3. Desk Loop

Purpose: inspect and decide what to do with work.

```bash
ccswarm desk
ccswarm desk --json
ccswarm desk diff <order-id>
ccswarm desk retry <order-id>
ccswarm desk instruct <order-id>
ccswarm desk merge <order-id>
ccswarm desk archive <order-id>
```

Implementation path:

- Keep `queue list` and `run list` as lower-level views.
- `desk` becomes the human operator view across orders, branches, and runs.
- Retry mode should preload failure context, run reports, changed files, and the
  original brief.

### 4. Authoring Loop

Purpose: create and validate reusable process definitions.

```bash
ccswarm flow init my-flow
ccswarm flow check my-flow
ccswarm flow render my-flow --phase all
ccswarm facets catalog
ccswarm repertoire add github:owner/repo
```

Implementation path:

- Extend existing `flow` and `facets` commands instead of adding a separate
  authoring namespace.
- `flow render` should show system prompt, user prompt, reports, gates, and
  provider routing per stage.

## Flow Schema Deepening

### Reports

Current `output_contract` support should become first-class `reports`:

```yaml
reports:
  - name: 00-plan.md
    contract: plan
  - name: findings.json
    schema: finding-ledger
```

Rules:

- Report names must be path-safe.
- Report contracts may be markdown templates or JSON schemas.
- `{report:name}` remains the prompt reference syntax.
- Missing required reports fail the stage before rule evaluation.

### Verdicts

Every stage should normalize its output to:

```json
{
  "status": "success",
  "verdict": "approved",
  "confidence": 0.82,
  "needs_human": false,
  "reports": ["00-plan.md"],
  "recovery": null
}
```

This gives rules, Sangha, gates, desk actions, and run summaries a common
decision language.

### Rules

Rule evaluation should be explicit and ordered:

1. Structured verdict.
2. Tag verdict, for legacy prompt-compatible flows.
3. Aggregate verdict from fork/Sangha/lead children.
4. Local heuristic.
5. AI judge, when enabled.

If a stage has rules and no rule matches, the run fails fast. Silent default
routing should be treated as a bug.

### Gates

Machine gates and human gates should share one model:

```yaml
gates:
  - name: unit-tests
    kind: command
    command: cargo test --workspace
    retry:
      feed_back_to_stage: true
      max_attempts: 2
  - name: commit-approval
    kind: approval
    timeout_seconds: 600
```

Gate output must be bounded, stored on disk, and referenced in the next prompt
by path and summary. Raw unbounded stdout should never be injected directly.

### Escalation

Escalation should combine visit-count promotion with ccswarm's provider
fallback:

```yaml
escalation:
  - when:
      stage_visits_at_least: 3
    provider: claude
    model: opus
  - when:
      judge: "progress has stalled"
    provider: codex
    model: gpt-5
fallback:
  on_rate_limit:
    - provider: codex
      model: gpt-5
```

The engine should record why an escalation happened and reset escalation state
after a successful stage.

## Runtime Reliability

### Process Supervision

Each provider/A2A call should have a supervised lifecycle:

- Spawn record with PID or request ID.
- Heartbeat/progress event every N seconds while waiting.
- Timeout event with provider, stage, prompt path, and elapsed time.
- Child-process cleanup on timeout/cancel.
- Partial run reconciliation on next startup.

Dogfood acceptance test:

1. Start a scaffold run with a fake provider that writes one file then hangs.
2. Timeout the stage.
3. Assert the run is marked `partial`.
4. Assert the partial file is listed in summary metadata.
5. Assert no provider child process remains.

### Recovery Points

After every stage boundary and before every provider call, write:

- flow state,
- stage visit counts,
- prompt path,
- provider options,
- report index,
- changed-file snapshot.

This enables `desk retry`, `run resume`, and failed-run inspection without
guesswork.

## Observability

ccswarm already has NDJSON events and opt-in OpenTelemetry. The next layer is
phase-level observability:

| Phase | Description |
|---|---|
| `prepare` | Prompt build, context expansion, report lookup. |
| `execute` | Provider/A2A call. |
| `parse` | Output projection and verdict extraction. |
| `report` | Report writing and schema validation. |
| `gate` | Command/HITL gate execution. |
| `route` | Rule evaluation and next-stage decision. |

`ccswarm cost` should aggregate by flow, stage, phase, provider, and model.
`ccswarm tail` should display long-running stage heartbeats, not just start/end.

## Implementation Milestones

### M0: Trust The Run

Job outcome: start work and safely stop watching it.

Deliverables:

- Provider/A2A heartbeat and phase events.
- Timeout/cancel cleanup for provider child processes and requests.
- Startup reconciliation for stale running runs.
- Partial run summary, changed-file snapshot, and recovery point.
- Fake-provider tests for hang, timeout, partial artifacts, and cleanup.

Acceptance:

- A hung provider cannot leave `movement_start` as the last meaningful state.
- `run list` and `tail` explain the active or terminal state without raw-log
  archaeology.
- A retry can restart from the recorded safe boundary.

### M1: Define And Control The Work

Job outcome: turn rough intent into a durable work package and retain control
through integration.

Deliverables:

- `.ccswarm/orders/{slug}/brief.md` plus validated reference attachments.
- `ccswarm intake --assist` with enqueue-first and explicit run-now choices.
- Queue metadata points to orders instead of duplicating briefs.
- Read-only `ccswarm desk` joining order, run, reports, gates, and diff.
- Safe isolation is the default for queued work.

Acceptance:

- A common request reaches an editable brief in under five minutes.
- The user can add references, edit the brief, and run later without losing
  context.
- `desk --json` supports scripts while the human view answers what happened and
  what actions are safe.

### M2: Trust The Decision

Job outcome: know why a run is complete, blocked, or needs changes.

Deliverables:

- Structured `StageVerdict` and required report validation.
- Explicit rule evaluation order and no-match failure.
- Unified command/HITL gate result shape.
- `flow render --phase all` for prompt, report, permission, gate, and routing
  preview.
- Desk instruct/retry/resume with prior failure evidence preloaded.

Acceptance:

- Review/fix loops route from validated verdicts rather than brittle prose.
- Every completion links requirements, reports, gate evidence, and changed
  files.
- Missing evidence fails early with an actionable message.

### M3: Capture Work Where It Appears

Job outcome: enqueue or inspect work from the user's existing client without
coupling that client to shell syntax.

Deliverables:

- Minimal stdio MCP-compatible tools: enqueue order, get status, and run next.
- Project-root confinement, input limits, path validation, and structured
  errors.
- A2A remains the remote agent execution protocol; MCP is the operator/client
  integration surface.
- ACP is a separately validated follow-on, not part of the first slice.

Acceptance:

- Codex, Claude, or another MCP client can enqueue a durable order and inspect
  its status.
- Tool stdout remains protocol-clean.
- Client integration cannot escape the allowed project root.

### M4: Spend The Right Capability

Job outcome: meet the quality target without unnecessary latency, cost, or
provider dependence.

Deliverables:

- `ProviderCapabilities` discovered by `doctor` and snapshotted per run.
- Routing policy considers stage job, required tools, structured output,
  privacy, budget, and reliability.
- Existing promotion/fallback becomes auditable escalation with explicit
  reasons.
- Cost and routing decisions are visible before and during execution.

Acceptance:

- Unsupported capabilities fail or fall back explicitly.
- The operator can explain why a provider/model was selected.
- Budget or privacy constraints cannot be silently bypassed.

### M5: Scale Repeatable Work

Job outcome: process multiple independent work packages without creating a new
supervision burden.

Deliverables:

- Resident queue watcher and bounded worker pool.
- Internal `StageKind` dispatch boundary.
- Durable team-task graph with owners, dependencies, expected reports, claim
  locking, and file-ownership hints.
- Typed stop policies for success, time, turns, cost, failure, cancellation,
  quorum, and human handoff.
- Iterative team-leader waves and batch stage over JSON/CSV/file lists.
- Sangha/fork/lead/batch/subflow results share the verdict contract.
- Child-targeted instruct, interrupt, approve, retry, and checkpointed resume.
- Per-order output prefixes and graceful shutdown.

Acceptance:

- Concurrent orders remain isolated, individually recoverable, and readable.
- A failure in one order does not corrupt another order's state or output.
- A process restart retains completed child reports and resumes only incomplete
  tasks.
- The operator can explain why each child exists, who owns it, what blocks it,
  and what will stop it.

### M6: Learn From Outcomes

Job outcome: improve future flow and provider choices from evidence rather than
intuition.

Deliverables:

- Phase-level NDJSON and optional OTLP records.
- Cost grouping by order, stage, phase, provider, model, and outcome.
- Intervention, retry, recovery, acceptance, and time-to-decision metrics.
- Recommendations remain advisory until their outcome advantage is proven.

Acceptance:

- A user can answer where time and tokens went, which interventions were
  required, and whether the change was accepted.
- Flow/provider comparisons use accepted outcomes as the denominator, not call
  count alone.

## Non-Goals

- Do not rename `flow` to `workflow` or `stage` to `step`; ccswarm already has
  a coherent vocabulary.
- Do not broaden provider support before the execution lifecycle is reliable.
- Do not let untrusted flow YAML run arbitrary commands unless explicitly
  allowed by config.
- Do not introduce a GUI before `desk --json` and terminal workflows are solid.

## First Concrete Slice

The first PR should be small and recovery-focused:

1. Add a `partial` run status and stale-run reconciliation.
2. Add heartbeat events while a provider call is in flight.
3. Add timeout child cleanup.
4. Add a fake hanging provider test.
5. Update `run list` / `tail` to show partial and heartbeat states.

This directly addresses the dogfood failure and creates the foundation for
orders, desk, and richer stage abstractions. Do not start assisted intake,
auto-routing, MCP, or concurrent queue work before this slice is verified.
