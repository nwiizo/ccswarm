# ccswarm

> Turn tasks into PR-ready diffs with quality gates already run. Reproducibly.

[![Rust](https://img.shields.io/badge/rust-edition_2024-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`ccswarm` is a workflow engine for AI coding agents. You describe a task, pick a flow
(declarative YAML workflow), and ccswarm drives the provider CLI through plan →
implement → review → fix → commit → PR, with full NDJSON audit trails you can replay,
diff, and roll back.

**OK/NG driven**: the only keys you press during a run are `y` and `n`.

## Hire ccswarm when

- You want a quality-gated change (plan → implement → review → fix) without rebuilding
  the workflow each task.
- You need reproducibility: the same flow YAML yields the same quality process,
  whether Alice or Bob runs it.
- You want to replay, diff, or undo what the agent did yesterday.
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

## Multi-provider

```yaml
# ccswarm.json or flow YAML (per-stage)
provider: claude          # claude | codex | copilot
model: sonnet
```

Precedence: stage YAML `provider:` > global `--provider` flag >
`CCSWARM_PROVIDER` env > Claude default. Unknown providers in `provider:`,
`promotion.provider`, or `on_rate_limit.provider` fail validation instead of
falling back silently.

| Provider | Status | Notes |
|----------|--------|-------|
| `claude` | Full support | All flags: --allowed-tools, --agent, --resume, --system-prompt, --max-budget-usd, --worktree |
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
| `default` | plan → implement → review → fix → complete | planner, coder, reviewer |
| `team` | plan → parallel(frontend + backend) → supervisor review | planner, frontend-specialist, backend-specialist, supervisor |
| `quick` | single-shot | coder |
| `review-fix` | review → fix loop | reviewer, coder |
| `research` | investigate → report | researcher |

Custom: drop YAML files into `.ccswarm/flows/`. See `ccswarm flow eject default` for
a starting template.

## Lab (experimental)

Group for features that sit beside the core JTBD but are not part of the primary flow.
May change without notice.

```bash
ccswarm lab sangha propose ...          # collective voting on proposals
ccswarm lab extend propose ...          # agent self-extension tracking
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
ccswarm (workflow + governance) ──depends on──> ai-session (terminal primitives)
```

- **ccswarm/cli** — command parsing and dispatch (35+ subcommands)
- **ccswarm/workflow** — FlowEngine, faceted prompting, stage reports
- **ccswarm/providers** — AgentProvider trait + Claude/Codex/Copilot implementations
- **ccswarm/session/bridge** — retry, context, persistence (provider-agnostic)
- **ccswarm/events** — NDJSON recorder, run summaries
- **ccswarm/governance** — proposals, extensions, approvals (renamed from coordination/)
- **ai-session** — PTY, zstd context compression (~93% token reduction), output
  parsing, persistence. Usable standalone.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for module responsibilities and API
boundaries, and [docs/COUPLING_REPORT.md](docs/COUPLING_REPORT.md) for the most recent
cargo-coupling modularity analysis.

## Development

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p ccswarm -- --help
```

End-to-end: `examples/e2e-playwright/run.sh` exercises pipeline → generated app →
Playwright browser test (requires a logged-in `claude` CLI or `ANTHROPIC_API_KEY`).

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
