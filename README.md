# ccswarm

> Turn uncertain coding work into reviewable changes through Sangha.

[![Rust](https://img.shields.io/badge/rust-edition_2024-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`ccswarm` is a workflow engine for AI coding agents, with **Sangha as its product
core**: shared acceptance criteria, separate assessments, evidence-backed
objections, revision, and a recorded decision. Claude Code and Codex execute
the work; ccswarm governs how the workflow advances.

The current default flow runs plan → Sangha quorum → implement → review → fix.
Sangha counts approval markers only from completed reviews and rejects an
unreachable quorum. Evidence-bound acceptance, objection
resolution, and checkpoint recovery are the next design, not shipped
guarantees. See [Sangha Product Core](docs/SANGHA_PRODUCT_CORE.md) for the
principles, implementation gaps, and feature priorities.
[Astra Integration](docs/ASTRA_INTEGRATION.md) adds the design for live guidance,
pending tool results, reasoning choices, and bounded delegation, distinguishing
model selection from future backend integration.

## Hire ccswarm when

- You want a repeatable plan, assessment, implementation, and review process
  without rebuilding the workflow for each task.
- You want reviewer concerns and verification evidence to inform the decision
  to accept a change; this is the direction of the Sangha core.
- You need recorded runs and timeline comparisons. Replay re-executes the task;
  undo currently reports commits rather than rolling them back.
- You use multiple provider CLIs (Claude Code / Codex, with gh copilot probed for
  diagnostics) and don't want to pick one.

## Quick start

```bash
cargo install --path crates/ccswarm
ccswarm doctor                        # probe Claude / Codex / gh copilot CLIs
ccswarm pipeline --task "Add login"   # one-shot
ccswarm pipeline --task "..." --dry-run --provider codex  # preview prompts
ccswarm scaffold --dir ./myapp --task "Create a todo app" --provider codex
ccswarm                               # interactive: asks what to build
```

`doctor` reports missing `ANTHROPIC_API_KEY` as a warning, not a hard failure,
when you use provider CLIs that are already authenticated locally.

`scaffold` creates a new git repository, writes a minimal npm project with a
passing `npm test` script, then runs the selected flow in that project. The
global `--provider` flag is forwarded into the child pipeline, and scaffold
exits non-zero if the pipeline fails or times out.

## Daily flow

```bash
ccswarm queue add "..."                 # accumulate during the day
ccswarm queue add --from-issue 42       # ingest a GitHub issue
ccswarm --json queue list               # machine-readable queue state
ccswarm queue drain                     # run all pending; y/n at commit + PR
ccswarm auto --watch                    # unattended: no y/n, auto-commit + PR
```

## After a run

```bash
ccswarm run list                       # past runs, newest first
ccswarm tail                            # follow the current run, tail-like
ccswarm cost <run-id>                   # per-stage + per-agent breakdown
ccswarm run view <run-id>               # full event log
ccswarm run diff <a> <b>                # compare two runs' timelines
ccswarm replay <run-id>                 # re-execute the recorded task
ccswarm undo <run-id>                   # advisory: list commits since run started
```

## Authoring flows

```bash
ccswarm flow list                      # builtin + custom
ccswarm flow new my-flow --template faceted
ccswarm flow render my-flow            # preview composed prompts per stage
ccswarm flow check my-flow             # validate YAML
ccswarm facets                          # browse personas / policies / knowledge
ccswarm repertoire add <git-url>        # install shared workflow packages
```

Stages can request Sangha assessment with `sangha:`. Member prompts ask for a
separate assessment ending with `SANGHA_DECISION=APPROVE` or
`SANGHA_DECISION=REVISE`. The current acceptance predicate only compares the
number of successful approvals with quorum; it does not resolve dissent or bind machine
checks to the decision.

```yaml
stages:
  - id: sangha
    instruction: "Review the plan before implementation"
    permission: readonly
    sangha:
      quorum: 2
      members:
        - { id: planner, persona: planner }
        - { id: reviewer, persona: reviewer }
        - { id: qa, persona: qa }
```

`readonly` expresses the requested permission. The current Codex adapter uses
`workspace-write`, so this setting does not enforce read-only review on Codex.
Required checks also need separate execution: Sangha currently bypasses the
ordinary stage-gate path. These are prerequisites for the stricter acceptance
policy described in the product design.

## Multi-provider

```yaml
# ccswarm.json or flow YAML (per-stage)
provider: claude          # claude | codex; copilot compatibility names fail fast
model: sonnet
```

Precedence: stage YAML `provider:` > global `--provider` flag >
`CCSWARM_PROVIDER` env > Claude default. Unknown providers in `provider:`,
`promotion.provider`, or `on_rate_limit.provider` fail validation instead of
falling back silently.

| Provider | Status | Notes |
|----------|--------|-------|
| `claude` | Full support | Uses `--allowed-tools`, `--agent`, `--session-id` / `--continue`, `--append-system-prompt`, `--max-budget-usd`, and `--worktree`. `CCSWARM_CLAUDE_STREAM_JSON=1` enables stream telemetry |
| `codex` | Non-interactive `codex exec` | System prompt is prepended to the user prompt (Codex has no dedicated flag). `codex exec resume <thread-id>` is used for same-thread continuation when Codex JSON telemetry provides a thread ID |
| `copilot` | **Unsupported for code generation** | `gh copilot suggest` is interactive and returns shell-command strings, not file edits. The provider fails fast with a friendly error — see `providers/copilot.rs` for rationale |

## Machine-readable output

Use `--json` when scripting commands. Application data is written to stdout as
JSON; tracing/log output is written to stderr so stdout remains parseable.

```bash
ccswarm --json init --name MyApp
ccswarm --json queue list
ccswarm --json config show
```

## Builtin flows

| Flow | Steps | Agents |
|-------|------|--------|
| `default` | plan → sangha quorum → implement → review → fix → complete | planner, reviewer, qa, coder |
| `team` | plan → parallel(frontend + backend) → supervisor review | planner, frontend-specialist, backend-specialist, supervisor |
| `team-dynamic` | plan → team_leader workers → review | planner, coder, reviewer |
| `quick` | single-shot | coder |
| `review-fix` | review → fix loop | reviewer, coder |
| `research` | investigate → report | researcher |

Custom: drop YAML files into `.ccswarm/flows/`. See `ccswarm flow eject default` for
a starting template.

## Lab (experimental)

Experimental commands may change without notice. `lab sangha` stores proposals
and votes separately from the workflow's `sangha:` stage; it does not enforce
workflow acceptance. Sangha's core product role refers to the workflow
decision process, not these proposal-storage commands.

```bash
ccswarm lab sangha propose ...          # collective voting on proposals
ccswarm lab extend propose ...          # agent self-extension tracking
ccswarm lab extend auto-propose ...     # generate an extension proposal + Sangha vote
ccswarm lab evolution report            # per-agent performance analytics
ccswarm lab search docs "..."           # ripgrep over docs/ and source
```

## Governance

```bash
ccswarm harness run                     # scenario-driven regression tests of flows
ccswarm approve plan --id <run>         # HITL gate for risky actions
```

## Architecture at a glance

```
ccswarm (workflow + governance + A2A/local provider execution)
```

- **ccswarm/cli** — command parsing and dispatch (35+ subcommands)
- **ccswarm/workflow** — FlowEngine, faceted prompting, stage reports
- **ccswarm/providers** — AgentProvider trait + Claude/Codex/Copilot implementations
- **ccswarm/session/bridge** — A2A/local execution, retry, context, persistence
- **ccswarm/events** — NDJSON recorder, run summaries
- **ccswarm/governance** — proposals, extensions, approvals (renamed from coordination/)
- **ccswarm/session/a2a** — A2A Agent Card metadata and REST `message:send`
  client support. Set `CCSWARM_A2A_ENDPOINT` to route live turns to a remote
  A2A server.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for module responsibilities and API
boundaries. See [docs/ROADMAP.md](docs/ROADMAP.md) for the canonical v0.10.0
release plan,
[docs/SANGHA_PRODUCT_CORE.md](docs/SANGHA_PRODUCT_CORE.md) for the core decision
process and feature selection,
[docs/CCSWARM_PRODUCT_ABSTRACTION_PLAN.md](docs/CCSWARM_PRODUCT_ABSTRACTION_PLAN.md)
for the longer job-theory product plan, and
[docs/MULTI_AGENT_REDESIGN.md](docs/MULTI_AGENT_REDESIGN.md) for the
provider-neutral multi-agent design.

## Development

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p ccswarm -- --help
```

End-to-end: `examples/e2e-playwright/run.sh` exercises queue intake, JSON
output, pipeline dry-run planning, and the tracked static app through a
Playwright browser test.

## Safety

- Every auto-commit (`ccswarm auto`) is gated by a sensitive-path deny-list
  (`.env`, `*.pem`, `*.key`, `id_rsa`, `credentials*`, `secrets.y*ml`, ...) before
  files are staged.
- Run IDs accepted by `tail` / `cost` / `replay` / `undo` are validated against a
  strict `[A-Za-z0-9_-]` allow-list to prevent path traversal.
- `ccswarm undo` is intentionally advisory: it prints the `git log` since the run
  started, it never rewrites history on its own.

## License

MIT. See [LICENSE](LICENSE).
