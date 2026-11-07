# ccswarm Application Specification

## Overview

ccswarm is a CLI for repeatable coding workflows centered on Sangha decisions.
The product direction is to connect shared criteria, separate assessments,
verification, objection resolution, and recovery. The current runtime supplies
flow pipelines, quorum voting, NDJSON events, and human approval mechanisms,
with execution through remote A2A agents or local provider CLIs.

[Sangha Product Core](SANGHA_PRODUCT_CORE.md) defines the target decision
process and explicitly separates it from current implementation behavior.

## Key Features

- **Sangha Assessment**: parallel member stages use final-line recommendations;
  only successfully completed members can approve, and unreachable quorum
  settings fail validation. Required
  evidence and blocking-objection handling are planned core work.
- **Flow-Based Workflows**: YAML-driven multi-step pipelines with context
  passing, retries, gates, parallel stages, and workflow calls.
- **A2ABridge**: A2A REST `message:send` execution when configured, otherwise
  Claude or Codex CLI execution in a background Herdr tab. The registered
  Copilot provider fails fast because `gh copilot suggest` is interactive.
- **Herdr Execution**: headless stages run `claude -p` / `codex exec` with
  file-based results; `execution: live` stages run an interactive Claude Code
  session read back from its transcript. Stages report their state to Herdr,
  and runs notify on completion and approval waits. See
  [Herdr Integration](HERDR_INTEGRATION.md).
- **NDJSON Event Recording**: `.ccswarm/runs/{run-id}/events.ndjson` and
  summaries for observability.
- **Faceted Prompting**: composable persona, policy, knowledge, and instruction
  facets.

Ordinary stage gates are available, but Sangha dispatch currently bypasses
them. Run events do not provide workflow checkpoint resume. Claude stages do
not yet enforce read-only review (see [Stage Permissions](#stage-permissions)).
These limitations must be fixed before the target Sangha acceptance policy
ships.

`lab sangha` proposal storage and self-extension commands remain experimental
and separate from workflow decisions.

## Stage Permissions

A stage's `permission` (`readonly`, `edit`, or `full`) is the single source of
its workspace boundary. `tools:` only names the tools a provider may offer.

| Setting | Derived from | Claude | Codex |
| --- | --- | --- | --- |
| Allowed tools | `tools:`, or the permission's default list | `--allowed-tools` | Not supported |
| Workspace access | `permission` only | Not enforced (`--dangerously-skip-permissions`) | `--sandbox read-only` for `readonly`, otherwise `workspace-write` |

- Explicit `tools:` never widen the workspace boundary: a `readonly` Codex
  stage that lists `bash` can run read-only commands but cannot write files.
- Commands that write, such as builds or test runs that create artifacts, fail
  in a `readonly` Codex stage. Run them as stage `gates:` or in an `edit`
  stage; gates are executed by ccswarm, not by the provider.
- Sangha members default to `readonly`; the AI rule judge
  (`CCSWARM_LLM_JUDGE=1`) always requests read-only access.
- A rate-limit fallback from a read-only Codex stage to a provider that cannot
  enforce read-only access fails with an error instead of continuing with
  write access.
- A2A endpoints receive the prompt only; no workspace boundary is sent.

## Requirements

- Rust toolchain compatible with the workspace.
- Git 2.20+.
- At least one live backend:
  - `CCSWARM_A2A_ENDPOINT=https://.../a2a` for a remote A2A server, or
  - Claude Code CLI or Codex CLI installed locally, with ccswarm running in a
    Herdr 0.9.5+ pane (`HERDR_ENV=1`).

## Project Structure

```text
ccswarm/
├── Cargo.toml
├── crates/
│   └── ccswarm/
│       ├── Cargo.toml
│       ├── src/
│       │   ├── cli/
│       │   ├── workflow/
│       │   ├── session/
│       │   ├── providers/
│       │   └── events/
│       └── tests/
├── docs/
└── examples/
```

## Live Execution

`A2ABridge` prepares the task prompt with working-directory context, then
chooses execution in this order:

1. `MovementExecOptions::a2a_endpoint`, if supplied.
2. `CCSWARM_A2A_ENDPOINT`, if set.
3. Local provider CLI selected by stage YAML, CLI flag, environment, or default,
   run in a background tab of the calling Herdr workspace. Outside Herdr this
   step fails before any retry.

The bridge then parses output into structured build/test/log/plain-text results
and returns token/cost metadata when the selected provider exposes it.

## A2A Requests

ccswarm posts a user `Message` containing a text `Part` to
`POST /message:send` and extracts text parts from the returned message, task,
or artifacts.

## Verification

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p ccswarm -- --help
```

## Forward Plan

The release scope and future milestones are tracked in the [roadmap](ROADMAP.md).
