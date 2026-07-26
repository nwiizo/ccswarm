# CLAUDE.md

## What ccswarm does

Hire ccswarm when you have a coding task and want a PR-ready diff with quality gates
already run, reproducibly. OK/NG-driven: you only press y or n.

- **Provider-neutral workflow**: Claude Code (default) and Codex execute code;
  GitHub Copilot CLI is probed only for migration diagnostics.
- **Reproducible**: declarative flow YAML → same quality every time.
- **Traceable**: NDJSON events + summaries in `.ccswarm/runs/<id>/`.

## Daily usage

```bash
ccswarm                              # interactive — ccswarm asks what to build
ccswarm pipeline --task "..."        # single-shot: plan → implement → review
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

## Optional post-pipeline flow

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
ccswarm (workflow + governance + A2A/local provider execution)
```

### ccswarm crate

| Module | Purpose |
|--------|---------|
| `cli/` | Command parsing + dispatch (3 entry modes: interactive / direct task / subcommand) |
| `workflow/` | FlowEngine, Pipeline, faceted prompting, stage reports |
| `providers/` | AgentProvider trait + ClaudeProvider / CodexProvider / CopilotProvider |
| `session/` | A2ABridge, native context/retry/persistence, output parsing, A2A client types |
| `events/` | NDJSON EventRecorder, duration tracking, run summaries |
| `governance/` | Proposals, extensions, approvals, coordination bus (renamed from `coordination/`) |
| `agent/` | AgentRole, type-state TaskBuilder |
| `identity/` | AgentIdentity, role boundaries |
| `hooks/` | HookRegistry |

Role boundary: `workflow/` owns orchestration and policy; `session/` owns
A2A/local execution primitives, context history, parsing, and persistence.

## Providers

```yaml
# flow YAML (per-stage override)
provider: claude   # claude | codex | copilot (default: claude)
model: sonnet
```

- Selection precedence: stage YAML > `--provider` flag (global) > `CCSWARM_PROVIDER` env > claude
- **claude**: `--allowed-tools`, `--agent`, `--session-id` / `--continue`, `--append-system-prompt`, `--max-budget-usd`, and `--worktree`. `CCSWARM_CLAUDE_STREAM_JSON=1` enables stream telemetry.
- **codex**: `codex exec`, multi-turn via `codex exec resume <thread-id>`. `CCSWARM_CODEX_JSON=1` for real token telemetry (forced on for multi-turn). No worktree/budget/allowed-tools.
- **copilot**: unsupported for code generation (gh copilot suggest is interactive); falls back to friendly error
- **A2A**: set `CCSWARM_A2A_ENDPOINT=https://.../a2a` to send live turns via
  REST `POST /message:send` instead of spawning a local provider CLI.

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
| `default` | plan → Sangha quorum → implement → review → fix → complete | planner, reviewer, qa, coder |
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
- Playwright-targeted tasks should ask for stable `data-testid` attributes explicitly.
- Complex implement stages often need `--timeout 900` or smaller task slices.
- Provider resume is best-effort: Claude uses `--session-id` / `--continue`;
  Codex uses `codex exec resume <thread-id>`.
- review→fix ループは flow の `max_stage_visits`(default 3)で打ち切られる。超過は Aborted。

## Rules

- [development-standards](.claude/rules/development-standards.md)
- [architecture-patterns](.claude/rules/architecture-patterns.md)
- [security-guidelines](.claude/rules/security-guidelines.md)
- [performance](.claude/rules/performance.md)

## Documentation

@docs/ROADMAP.md
@docs/ARCHITECTURE.md
@docs/APPLICATION_SPEC.md
