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

Local provider runs require [Herdr](https://herdr.dev) 0.9.5 or later. Each
stage runs in its own background Herdr tab, reports its state to Herdr, and
closes when it finishes; [Herdr Integration](docs/HERDR_INTEGRATION.md)
describes the contract.

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
herdr                                 # open Herdr, then run ccswarm in one of its panes
ccswarm doctor                        # probe Herdr and Claude / Codex / gh copilot CLIs
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

Run commands accept the full run ID or any unique prefix, such as the
8-character ID ccswarm prints after a run.

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

`readonly` expresses the requested permission. Codex enforces it with
`codex exec --sandbox read-only`, even when `tools:` lists write or exec
tools, and a rate-limit fallback from a read-only Codex stage to Claude fails
instead of running writable. Claude does not yet enforce it: Claude runs with
`--dangerously-skip-permissions` and an allowed-tools list. Stage `gates:`
run once approvals reach quorum, and a failed gate rejects the decision even
when every member approved; gate results are not yet bound to the reviewed
revision. These are prerequisites for the stricter acceptance policy
described in the product design.

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
| `claude` | Full support | Uses `--allowed-tools`, `--agent`, `--append-system-prompt`, `--max-budget-usd`, and `--worktree`. `CCSWARM_CLAUDE_STREAM_JSON=1` enables stream telemetry |
| `codex` | Non-interactive `codex exec` | System prompt is prepended to the user prompt (Codex has no dedicated flag). Read-only stages use `--sandbox read-only`; others use `workspace-write`. `CCSWARM_CODEX_JSON=1` enables token telemetry |
| `copilot` | **Unsupported for code generation** | `gh copilot suggest` is interactive and returns shell-command strings, not file edits. The provider fails fast with a friendly error — see `providers/copilot.rs` for rationale |

## Herdr

ccswarm runs local providers in background tabs of the Herdr workspace that
started it, so stages, Sangha members, and parallel workers stay visible.
Outside a Herdr pane, local provider runs stop with setup guidance; remote A2A
runs (`CCSWARM_A2A_ENDPOINT`) and commands that do not run providers still work.

```yaml
stages:
  - id: review
    provider: claude
    execution: live   # headless (default) | live
```

- `headless` runs `claude -p` / `codex exec` as before, with the same output,
  exit-code, and token telemetry.
- `live` runs an interactive Claude Code session that you can watch, answer,
  or steer in its tab. ccswarm reads the result from the session transcript and
  waits (with a Herdr notification) when Claude asks a question, including
  folder trust the first time a repository is opened. Live stages run one turn;
  Codex stages and provider fallbacks run headless.

The [Herdr plugin](herdr-plugin/) adds a run board, recent runs, and doctor:

```bash
herdr plugin install nwiizo/ccswarm/herdr-plugin --ref v0.11.0
```

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

## Harness and approvals

```bash
ccswarm harness run                     # scenario-driven regression tests of flows
ccswarm approve plan --id <run>         # HITL gate for risky actions
```

## Architecture at a glance

```
ccswarm (workflow + Sangha + A2A/Herdr provider execution)
```

- **ccswarm/cli** — command parsing and dispatch (35+ subcommands)
- **ccswarm/workflow** — FlowEngine, faceted prompting, stage reports
- **ccswarm/providers** — AgentProvider trait + Claude/Codex/Copilot implementations
- **ccswarm/session/bridge** — A2A/local execution routing, retry, rate-limit fallback
- **ccswarm/session/herdr** — Herdr tab execution (headless and live Claude)
- **ccswarm/events** — NDJSON recorder, run summaries
- **ccswarm/session/a2a** — A2A REST `message:send` client. Set `CCSWARM_A2A_ENDPOINT` to route live turns to a remote
  A2A server.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for module responsibilities and API
boundaries. See [docs/ROADMAP.md](docs/ROADMAP.md) for the current release scope,
[docs/SANGHA_PRODUCT_CORE.md](docs/SANGHA_PRODUCT_CORE.md) for the core decision
process and feature selection, and
[docs/HERDR_INTEGRATION.md](docs/HERDR_INTEGRATION.md) for Herdr execution.

## Development

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p ccswarm -- --help
```

Try the working [Order Desk app](examples/e2e-playwright/generated/index.html)
by opening that file locally. `examples/e2e-playwright/run.sh` tests the included
app and preview without calling a provider. Add `--live` to generate a fresh
app through ccswarm and test that exact output in Chromium. See the
[example guide](examples/e2e-playwright/README.md) for setup and retained run evidence.

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
