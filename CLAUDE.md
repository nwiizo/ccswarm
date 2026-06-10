# CLAUDE.md

## What ccswarm does

Hire ccswarm when you have a coding task and want a PR-ready diff with quality gates
already run, reproducibly. OK/NG-driven: you only press y or n.

- **Provider-agnostic**: Claude Code (default), Codex, GitHub Copilot CLI.
- **Reproducible**: declarative flow YAML → same quality every time.
- **Traceable**: NDJSON events + summaries in `.ccswarm/runs/<id>/`.

## Daily usage

```bash
ccswarm                              # interactive — ccswarm asks what to build
ccswarm pipeline --task "..."        # single-shot: plan → implement → review → commit → PR
ccswarm queue add "..."              # accumulate tasks during the day
ccswarm queue add --from-issue 42    # ingest a GitHub issue as a task
ccswarm queue drain                  # run all pending, y/n at commit+PR time
ccswarm auto --require-approval      # unattended; pauses before commit until
                                     #   `ccswarm approve commit --id <run-id>`
ccswarm doctor                       # probe all providers (claude/codex/gh copilot)
```

## After a run

```bash
ccswarm run list                    # recent runs
ccswarm tail                         # follow events of the current run, tail-like
ccswarm cost <run-id>                # duration + token breakdown
ccswarm run view <run-id>            # full event log
ccswarm run diff <a> <b>             # compare two runs' timelines
ccswarm replay <run-id>              # re-execute the recorded task
ccswarm undo <run-id>                # advisory: show commits since run started
```

## Authoring flows

```bash
ccswarm flow list                   # builtin + custom flows
ccswarm flow new <name>             # scaffold a new flow (minimal|faceted)
ccswarm flow render <name>          # preview composed prompts per stage
ccswarm flow check <name>           # validate flow YAML
ccswarm flow eject <name>           # copy builtin to .ccswarm/flows/
ccswarm facets [personas|policies|knowledge]  # list facet library
```

## Post-Pipeline Flow (automatic)

```
Pipeline完了 → テスト自動実行 → 失敗なら自動修復(最大3回)
→ "Commit? [Y/n]" → "Create PR? [Y/n]"
```

## Development commands

```bash
cargo fmt && cargo clippy --workspace -- -D warnings && cargo test --workspace
cargo run -p ccswarm -- --help
```

## Workspace architecture

```
ccswarm (workflow + governance) ──depends on──> ai-session (terminal primitives)
```

### ccswarm crate

| Module | Purpose |
|--------|---------|
| `cli/` | Command parsing + dispatch (3 entry modes: interactive / direct task / subcommand) |
| `workflow/` | FlowEngine, Pipeline, faceted prompting, stage reports |
| `providers/` | AgentProvider trait + ClaudeProvider / CodexProvider / CopilotProvider |
| `session/` | AISessionBridge — delegates command construction to providers, owns context/retry/persistence |
| `events/` | NDJSON EventRecorder, duration tracking, run summaries |
| `governance/` | Proposals, extensions, approvals, coordination bus (renamed from `coordination/`) |
| `agent/` | AgentRole, type-state TaskBuilder |
| `identity/` | AgentIdentity, role boundaries |
| `hooks/` | HookRegistry |

### ai-session crate

| Module | Purpose |
|--------|---------|
| `core/` | AISession, SessionManager, PTY/headless |
| `context/` | TokenEfficientHistory (zstd, ~93% token reduction) |
| `output/` | OutputParser (cargo/Playwright/npm/Jest patterns) |
| `persistence/` | Session snapshots |
| `coordination/` | Inter-session message bus (distinct from ccswarm's `governance/`) |

Role boundary: ai-session = terminal/session primitives. ccswarm = workflow + governance.
Don't add workflow logic to ai-session.

## Providers

```yaml
# flow YAML (per-stage override)
provider: claude   # claude | codex | copilot (default: claude)
model: sonnet
```

- Selection precedence: stage YAML > `--provider` flag (global) > `CCSWARM_PROVIDER` env > claude
- **claude**: full flag coverage (--allowed-tools, --agent, --resume, --system-prompt, --max-budget-usd, --worktree). `CCSWARM_CLAUDE_STREAM_JSON=1` for real token telemetry.
- **codex**: `codex exec`, multi-turn via `codex exec resume <thread-id>`. `CCSWARM_CODEX_JSON=1` for real token telemetry (forced on for multi-turn). No worktree/budget/allowed-tools.
- **copilot**: unsupported for code generation (gh copilot suggest is interactive); falls back to friendly error

`ccswarm doctor` probes all three CLIs.

Reliability (flow/stage YAML):

```yaml
on_rate_limit:                  # flow-level: switch provider on rate limit
  - { provider: codex }
stages:
  - id: fix
    promotion:                  # escalate from the Nth visit (last match wins)
      - { at: 2, model: opus }
    gates:                      # machine gates; failure re-runs the stage
      - { name: build, command: "cargo build" }
    team_leader:                # orchestrator-worker: leader decomposes at runtime
      max_parts: 3
```

`CCSWARM_LLM_JUDGE=1` makes `ai("...")` rule conditions ask a real model (YES/NO) instead of the lexical heuristic.

## Builtin flows

| Flow | Description | Agents |
|-------|-------------|--------|
| `default` | plan → implement → review → fix → complete | planner, coder, reviewer |
| `team` | plan → parallel(frontend + backend) → supervisor review | planner, frontend-specialist, backend-specialist, supervisor |
| `team-dynamic` | plan → team_leader decomposes at runtime → parallel workers → review | planner, coder (leader + workers), reviewer |
| `quick` | single-shot (1 stage) | coder |
| `review-fix` | review → fix loop | reviewer, coder |
| `research` | investigate → report | researcher |

Custom: `.ccswarm/flows/*.yaml`. Installable packages: `ccswarm repertoire add <git-url>`.

## Builtin facets

Personas: planner, coder, reviewer, researcher, supervisor, ai-antipattern-reviewer.
Policies: coding, review, security, testing.
Knowledge: (user-provided under `.ccswarm/facets/knowledge/*.yaml`).

## Pipeline learnings

- タスク記述は簡潔に(500語以下)。長いとimplementがタイムアウト。
- `{task}`, `{plan_output}` テンプレート変数で stage 間コンテキスト受け渡し。
- `pass_previous_response: false` で fix stage のコンテキストをリセット。
- Empty `agents_used` in a summary means no provider CLI was invoked for that stage.
- review→fix ループは flow の `max_stage_visits`(default 3)で打ち切られる。超過は Aborted。

## Rules

- [development-standards](.claude/rules/development-standards.md)
- [architecture-patterns](.claude/rules/architecture-patterns.md)
- [security-guidelines](.claude/rules/security-guidelines.md)
- [performance](.claude/rules/performance.md)

## Documentation

@docs/ARCHITECTURE.md
@docs/APPLICATION_SPEC.md
