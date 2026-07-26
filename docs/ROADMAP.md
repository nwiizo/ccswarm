# ccswarm v0.10.0 Roadmap

Status: planned, not released.
Last verified: 2026-07-26.

This is the release source of truth for v0.10.0. The
[Product Abstraction Plan](CCSWARM_PRODUCT_ABSTRACTION_PLAN.md) defines the
longer operator journey, and the
[Multi-Agent Redesign](MULTI_AGENT_REDESIGN.md) contains the provider and
framework research behind the multi-agent choices below.

## Release Decision

v0.10.0 should be the **trustworthy team execution** release.

The release should make an existing ccswarm run observable, bounded, and
recoverable before adding broader intake, UI, provider-native team, or
automation surfaces. A version number, feature count, or locally passing test
suite is not sufficient to release it.

The primary job is:

> When I delegate uncertain coding work to several execution paths, help me
> leave the run unattended, return to an unambiguous state, intervene at the
> smallest useful boundary, and resume without discarding accepted work.

This scope combines the first three layers needed for that job:

1. a release process that cannot publish a known-incomplete distribution;
2. a supervised run lifecycle with explicit terminal reasons;
3. durable task, verdict, and recovery contracts for multi-agent work.

## Evidence Baseline

The roadmap is based on the current repository and public release state, not
only the desired architecture.

### Existing strengths

- One Rust crate with 353 passing tests and 2 ignored tests at this snapshot.
- Declarative flows, bounded stage visits, parallel stages, dynamic team-leader
  decomposition, Sangha quorum, command gates, and HITL commit approval.
- Claude Code and Codex CLI execution, provider fallback and promotion, session
  continuation, NDJSON events, run summaries, replay, diff, cost, and tail.
- Native A2A data types and one-shot HTTP+JSON message execution.
- Cross-platform release targets for Linux and macOS on AMD64 and ARM64.

### Gaps that block v0.10.0

- No tag-triggered Release workflow in the public history has completed
  successfully. The v0.9.2 run published only three of four binary targets and
  its original crate job referenced a removed workspace crate.
- The release workflow creates a public release before the build matrix is
  known to pass, and crate publishing is not gated on all binary artifacts.
- `EventType` has start/end and provider events but no heartbeat, checkpoint,
  cancellation, timeout, or reconciliation events.
- Timeout output can be described as partial, but there is no durable run-state
  contract that reconciles an interrupted process on the next invocation.
- `StageKind`, `StageVerdict`, `ProviderCapabilities`, `TeamTask`, and typed
  stop policies remain design concepts rather than production types.
- The A2A client sends one REST message and recursively extracts text. It does
  not yet negotiate A2A 1.0, model the complete task lifecycle, stream updates,
  cancel tasks, or resume task observation.

## Outcome Metrics

Every metric is a release gate, not an aspirational dashboard item.

| Outcome | v0.10.0 target |
| --- | --- |
| Ambiguous state | Zero test runs may end with only a start event |
| Terminal reason | 100% of root and child executions record one typed reason |
| Recovery economy | Completed stages and accepted child reports are not rerun after resume |
| Cleanup | No provider child remains after timeout or cancellation in fault-injection tests |
| Intervention precision | Retry, cancel, or resume can target one failed boundary |
| Release completeness | Four archives, four checksums, provenance, and smoke results exist before stable publication |
| Release reliability | One release candidate and the stable tag both complete the same pipeline successfully |
| Protocol honesty | A2A features are negotiated and unsupported operations fail explicitly |
| Cost clarity | Duration and usage are attributable to run, stage, child, provider, and outcome |

## Scope And Delivery Order

Milestones are ordered by risk, but independent pull requests may proceed in
parallel as shown in the dependency table below. A dependent slice does not
start until its prerequisites meet their acceptance criteria. This prevents
team features from hiding an unreliable execution or distribution layer.

### R0: Make Release Failure Safe

Job outcome: maintainers can test the complete distribution path without
creating a misleading stable release.

Deliverables:

- Validate that the tag, `Cargo.toml`, `Cargo.lock`, and changelog version agree.
- Run format, Clippy, tests, audit, docs, package, and publish dry-run before any
  public release is created.
- Build all target archives into workflow artifacts before uploading release
  assets.
- Gate crates.io publishing and GitHub publication on the complete build matrix.
- Test the release plan and cross-platform builds on pull requests or a manual
  release-candidate workflow.
- Pin the cross-compilation tool and GitHub Actions to reviewed versions.
- Generate SHA-256 checksums and GitHub artifact attestations.
- Create the GitHub release as a draft, attach every asset, smoke-test the
  downloaded archives, then publish it.
- Mark semantic prereleases as prereleases and never publish their crate unless
  explicitly enabled.
- Evaluate `dist` as an alternative to the handwritten workflow. Adopt it only
  if its generated plan/build/host/publish/announce pipeline remains reviewable
  and supports the required crate publishing policy.

Acceptance:

- A failed target leaves no stable GitHub release and does not publish the crate.
- Pull requests exercise at least the release plan; the release-candidate run
  exercises every target.
- A maintainer can verify each downloaded archive with its checksum and
  provenance attestation.

### R1: Supervise Every Execution

Job outcome: the operator can leave a run and still know whether work is active,
blocked, failed, or safely stopped.

Deliverables:

- Add typed run and child states:
  `pending`, `running`, `waiting_for_input`, `completed`, `partial`, `failed`,
  `canceled`, and `timed_out`.
- Add typed terminal reasons separate from status.
- Emit prepare, execute, parse, report, gate, and route phase events.
- Emit provider heartbeats from stream activity or a synthetic interval.
- Record provider PID, remote request/task ID, session ID, stage, prompt path,
  start time, last activity, and cleanup outcome when available.
- On timeout, cancellation, signal, or parent failure, terminate and reap local
  children and cancel remote work when the negotiated protocol supports it.
- Reconcile stale `running` records on startup using process, session, event,
  and artifact evidence.

Acceptance:

- Fake providers cover hang-before-output, partial-write-then-hang, ignored
  termination, malformed stream, and parent interruption.
- `run list`, `run view`, and `tail` explain the state without raw-log
  archaeology.
- No fault-injection scenario leaves a start-only lifecycle.

### R2: Persist Recovery Points

Job outcome: recovery repeats only work whose result is not already accepted.

Deliverables:

- Write an atomic checkpoint before each provider call and after every stage,
  report, gate, verdict, and task transition.
- Store flow position, visit counts, provider options, session/task IDs,
  report index, changed-file snapshot, and terminal reason.
- Add resume validation that rejects mismatched flow definitions, repository
  identity, or incompatible checkpoint schema.
- Add targeted retry and resume for the failed stage or child.
- Preserve completed reports as immutable inputs to resumed work.
- Introduce a checkpoint schema version and migration policy.

Acceptance:

- Killing the process at every checkpoint boundary and restarting produces the
  same accepted result as an uninterrupted control run.
- Resume never silently falls back to a full replay.
- Corrupt or incompatible state fails with a recovery path and does not mutate
  the worktree.

### R3: Normalize Stage, Task, And Verdict Contracts

Job outcome: the operator can see why each child exists, what it owns, what
blocks it, and what evidence completes it.

Deliverables:

- Refactor existing dispatch behind internal `StageKind` variants without
  changing supported flow YAML.
- Add `StageVerdict` with status, verdict, confidence, reports, gate evidence,
  human-input need, and recovery metadata.
- Add a durable `TeamTask` graph with owner, dependencies, expected reports,
  status, attempt, and file-ownership hints.
- Add atomic task claiming and dependency unblocking.
- Add typed, composable stop policies for success, failure, time, turns, cost,
  cancellation, quorum, and human handoff.
- Route from structured verdicts first while preserving documented legacy
  routing as an explicit compatibility fallback.

Acceptance:

- Existing call, parallel, team-leader, Sangha, and review/fix fixtures behave
  identically after the internal dispatch refactor.
- Duplicate task claims and unresolved dependency execution are impossible.
- Missing required reports or unmatched routing fail explicitly.

### R4: Make Provider And A2A Capabilities Honest

Job outcome: ccswarm selects only behavior the current provider or remote agent
can actually perform.

Deliverables:

- Add `ProviderCapabilities` with `Supported`, `Experimental`, `Unsupported`,
  and `Unknown` states.
- Probe local CLI capabilities in `doctor` and snapshot them into each run.
- Render required capabilities, selected backend, context projection, budget,
  and fallback before execution.
- Align the A2A model and client with released A2A 1.0 concepts, bindings, and
  version negotiation.
- Support the minimum useful A2A task lifecycle: discovery, send, get,
  cancellation, and streaming or explicit polling fallback.
- Validate remote data, bound response sizes, sanitize errors, require HTTPS
  outside explicit local development, and keep credentials out of events.

Acceptance:

- Capability fixtures prove supported, unsupported, experimental, and unknown
  behavior for each provider backend.
- A2A conformance tests cover version mismatch, input-required, auth-required,
  completed, failed, canceled, malformed, oversized, and disconnected cases.
- Unsupported streaming or cancellation is visible before execution where it
  can be discovered.

### R5: Observe Outcomes Without Leaking Content

Job outcome: the operator can compare reliability, time, and cost while prompts
and secrets remain private by default.

Deliverables:

- Correlate run, stage, team task, child, provider session, and A2A task IDs.
- Record duration, token usage, retry, intervention, recovery, and accepted
  outcome by phase.
- Map optional OTLP spans onto a pinned OpenTelemetry GenAI semantic-convention
  version instead of tracking an unversioned moving schema.
- Keep prompt, response, tool arguments, tool results, and credentials
  disabled by default; document explicit opt-in and redaction behavior.
- Add a single-run summary showing critical path, parallel savings, idle time,
  failed work, repeated work, and accepted evidence.

Acceptance:

- The same run can be explained from terminal output, NDJSON, summary JSON, and
  OTLP without contradictory states.
- No default telemetry fixture contains prompt text, response text, secret
  values, or credential-bearing URLs.
- Team-size comparisons use accepted outcomes and total cost, not call count.

### R6: Prove The Release Candidate

Job outcome: users receive a release whose core recovery claim has been
demonstrated on packaged binaries.

Required scenarios:

1. A normal single-provider review/fix run.
2. A parallel run with one slow child and one failed child.
3. A provider that writes a partial artifact and hangs.
4. Process termination followed by checkpoint resume.
5. Cancellation during a local provider call.
6. A2A 1.0 task completion, input request, timeout, and cancellation.
7. A downloaded archive on each supported target running `--version`,
   `--help`, `doctor`, flow validation, and a dry run.
8. A clean install from the packaged crate.

Exit criteria:

- `v0.10.0-rc.1` completes the production-equivalent workflow.
- No release-blocking bug remains open.
- Upgrade and rollback notes are tested from v0.9.2.
- Stable v0.10.0 is cut from the exact reviewed release-candidate commit or a
  documented, reviewed fix-only descendant.

## Planned Pull Request Sequence

Each pull request should be independently reviewable and keep the workspace
green.

| Order | Slice | Depends on |
| --- | --- | --- |
| 1 | Transactional release workflow and release-plan checks | None |
| 2 | Run state, terminal reason, and additive event schema | None |
| 3 | Local process supervision, heartbeat, cancel, and cleanup | 2 |
| 4 | Atomic checkpoints, reconciliation, and targeted resume | 2, 3 |
| 5 | `StageKind` behavior-preserving dispatch refactor | 2 |
| 6 | `StageVerdict` and required report validation | 5 |
| 7 | `TeamTask` graph, claims, dependencies, and stop policies | 4, 6 |
| 8 | Provider capability probes and rendered selection | 5 |
| 9 | A2A 1.0 lifecycle and conformance fixtures | 2, 4, 8 |
| 10 | Correlated summaries, privacy-safe OTLP mapping, and dogfood | 3, 4, 7, 9 |
| 11 | Release candidate, packaged-binary smoke matrix, and release notes | 1-10 |

## Compatibility Policy

- Existing flow YAML remains valid throughout v0.10.0 unless a security issue
  requires rejection.
- New event fields and event kinds are additive. Consumers can use the schema
  version to negotiate future incompatible changes.
- Legacy string and tag routing remains available for v0.10.0 with diagnostics
  when structured verdicts are not used.
- Checkpoint compatibility is explicit. An incompatible checkpoint is never
  guessed or replayed automatically.
- Experimental `lab` commands do not define v0.10.0 stability guarantees.

## Explicitly Deferred

These are useful but do not belong in the v0.10.0 critical path:

- Assisted intake, order directories, and the full Desk command surface.
- Provider-native subagent execution and automatic native/ccswarm selection.
- A general mailbox UI or direct peer-to-peer chat between local children.
- MCP task integration while the task utility remains experimental.
- ACP support.
- A GUI, hosted control plane, distributed scheduler, or automatic merge.
- Long-term memory, vector storage, and autonomous flow optimization.
- Additional provider breadth.
- Arbitrary graph scheduling beyond the current flow model and durable task
  dependencies required by this release.

## Release Checklist

The release owner records evidence for every item:

- [ ] Version, lockfile, changelog, tag, and binary `--version` agree.
- [ ] Format, Clippy, tests, audit, docs, release build, and package checks pass.
- [ ] Release-plan and all four target builds pass from the release commit.
- [ ] Fault-injection and checkpoint-resume scenarios pass.
- [ ] A2A conformance fixtures pass.
- [ ] Packaged archives and the packaged crate pass smoke tests.
- [ ] Four archives, four checksums, and attestations are present.
- [ ] Release notes state compatibility, migration, known limitations, and
      rollback.
- [ ] The GitHub release is published only after its complete draft is verified.
- [ ] crates.io publication succeeds from the same commit.
- [ ] The stable workflow is green and linked from the release.

## Research Snapshot

Official sources checked on 2026-07-26:

- [A2A 1.0 specification](https://a2a-protocol.org/latest/specification/)
- [MCP 2025-11-25 tasks](https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/tasks)
- [OpenTelemetry GenAI semantic conventions](https://github.com/open-telemetry/semantic-conventions/tree/main/docs/gen-ai)
- [GitHub immutable releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases)
- [GitHub artifact attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations)
- [`dist` release pipeline](https://github.com/axodotdev/cargo-dist)
- Multi-agent framework and provider sources:
  [Multi-Agent Redesign, Source Snapshot](MULTI_AGENT_REDESIGN.md#11-source-snapshot)
