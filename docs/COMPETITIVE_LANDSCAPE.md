# Competitive Landscape

Snapshot of where ccswarm sits among coding-agent orchestrators and
general multi-agent frameworks, based on a feature audit against external-workflow
(June 2026). Update when positioning decisions are revisited.

## Lineage: external-workflow

ccswarm is a Rust adaptation of [external-workflow](https://github.com/nrslib/external-workflow)
(TypeScript), which orchestrates AI coding-agent CLIs through declarative
YAML workflows with review loops, faceted prompting, and guardrails.

### Feature parity vs external-workflow (post-v0.7.0)

| external-workflow feature | ccswarm status |
|---|---|
| Flow YAML (stages, initial/max, terminal routing) | Implemented (terminal = empty `rules`; no COMPLETE/ABORT keywords) |
| Stage fields (persona/permission/agent/working_dir/retry) | Implemented (`permission: readonly\|edit\|full` instead of `required_permission_mode`) |
| Rule fields (`interactive_only`, `requires_user_input`) | Implemented |
| Output contracts + `{report:<name>}` | Implemented (multi-report delimiters deferred) |
| Sub-workflow dispatch (`call:`) | Implemented (no `returns` schema) |
| Faceted prompting | Implemented — builtin < repertoire < user < project layers (v0.8.0). 6 personas / 4 policies vs external-workflow's 26 / richer set |
| Providers | claude (deep) / codex (first-class since v0.8.0: `--provider` flag, JSONL telemetry, session resume) / copilot (intentional no-op); external-workflow adds cursor, kiro, opencode, claude-sdk |
| Stream/JSONL telemetry parsing | Implemented for Claude stream-json (`CCSWARM_CLAUDE_STREAM_JSON=1`) and Codex JSONL (`CCSWARM_CODEX_JSON=1`); schema-constrained final responses are deferred |
| Repertoire packages | Implemented — install/list/remove + package facets participate in layer resolution (v0.8.0) |
| Queue + GitHub issue ingestion + worktrees | Implemented (`tracker/` with github + linear adapters) |
| Parallel stages + all()/any() aggregation | Implemented (v0.7.0 wired aggregation into the engine) |
| Loop/cycle detection | Implemented (v0.7.0: `max_stage_visits` runtime bound + `flow check` static analysis) |
| HITL gates that block execution | Implemented (v0.7.0: commit gate on `auto`/`drain --require-approval`) |
| AI judge (`ai("...")` conditions) | Implemented (v0.8.0, opt-in `CCSWARM_LLM_JUDGE=1`; lexical heuristic as offline fallback) |
| Rate-limit fallback chain | Implemented (v0.8.0, flow-level `on_rate_limit`) |
| Step-level provider promotion | Implemented (v0.8.0, stage `promotion`, `at: N` last-match-wins) |
| Team-leader dynamic decomposition | Implemented (v0.8.0, stage `team_leader`, single wave; external-workflow's iterative "more parts?" waves deferred) |
| Command quality gates | Implemented (v0.8.0, stage `gates`, feedback re-runs the stage) |
| OpenTelemetry | Implemented (v0.7.0: opt-in `otel` feature, OTLP spans) |
| Builtin workflows | 5 vs external-workflow's 48 |

ccswarm-only additions not in external-workflow: `lab` research surface (sangha /
extend / evolution / search), run replay/diff/cost, NDJSON event model.

## Coding-agent orchestrators

ccswarm's positioning — declarative flow YAML + automated review/fix
loops + multi-provider CLI + git worktree isolation, in one Rust binary
with NDJSON tracing and run replay/diff — is a combination no single
competitor currently matches. The field splits along those axes:

| Tool | Declarative YAML | Auto review loop | Worktree isolation | Multi-provider | Notes |
|---|---|---|---|---|---|
| ccswarm / external-workflow | yes | yes | yes | yes | this niche |
| claude-flow (ruvnet) | partial | plugins | no | Claude-centric | ~58k★ swarm/hive-mind, shared SQLite memory |
| Claude Code subagents / Agent Teams | .md frontmatter | implicit | yes | Claude only | first-party; ccswarm positions as complement |
| Claude Squad | no (TUI) | human | yes | yes | closest on provider+worktree |
| Goose (Block) | yes (recipes) | recipe-level | no | yes | closest on YAML+CI; single-agent |
| Conductor / Crystal / Vibe Kanban | no (GUI) | human (diff/kanban) | yes | yes | parallel-session GUIs |
| Aider | no | architect→editor | git-native | 100+ models | two-model split pioneer |
| OpenHands | .md microagents | action loop | sandboxes | yes | ICLR 2025, cloud-first |
| Roo Code (Boomerang) | modes config | subtask summaries | context-only | yes | in-IDE orchestrator |

Where competitors lead: default parallel execution, container-level
isolation (container-use, Sketch), GUI diff review, shared cross-agent
memory/consensus (claude-flow), community scale.

## General multi-agent frameworks (pattern standards)

The industry pattern vocabulary (per Anthropic's *Building Effective
Agents*): prompt chaining, routing, parallelization, orchestrator-worker,
evaluator-optimizer, planner-executor, group-chat, on graph/state-machine
substrates. ccswarm's flows map to chaining, routing, parallelization
(`team`), evaluator-optimizer (`review-fix`), planner-executor
(`default`).

Production table stakes across LangGraph / MS Agent Framework / Pydantic
AI / Mastra / LlamaIndex: checkpoint-resume, HITL pause/resume, OTel.
v0.7.0 brings ccswarm onto that bar for HITL (commit gate) and OTel
(opt-in feature); checkpoint-resume exists as session persistence +
`replay`, not mid-run resume.

Declarative-YAML control flow is the minority position but vendor-backed:
CrewAI (agents.yaml/tasks.yaml, recommended path) and Google ADK (Agent
Config, 2025-08). Most frameworks are code-first. The known trade-off:
YAML covers chaining/routing/parallel/review-loop cleanly, while
runtime-dynamic orchestrator-worker decomposition needs a code escape
hatch.

## Roadmap candidates (not committed)

The v0.8.0 cycle implemented all six items from the original audit
roadmap (team_leader, rate-limit fallback, promotion, command gates,
facet layers, LLM judge). Remaining candidates:

1. **team_leader iterative waves** — external-workflow asks the leader "more parts or
   done?" after each wave; v1 is single-wave.
2. **Worker isolation** — worktree (or container) per team_leader part;
   parts currently share the working directory.
3. **Schema-constrained output hardening** — Codex `--output-schema` and
   Claude `--json-schema` can produce JSON Schema-constrained final responses;
   leader decomposition and LLM judge could use them instead of prompt-based
   JSON.
4. **More providers** — external-workflow supports cursor, kiro, opencode,
   claude-sdk; ccswarm's AgentProvider trait makes additions cheap.
5. **Persona library depth** — 6 builtin personas vs external-workflow's 26; the
   review specialists (security/architecture/qa) are the obvious next
   tranche.
6. **Mid-run checkpoint-resume** — session persistence + `replay` exist,
   but resuming a partially completed flow run does not.
