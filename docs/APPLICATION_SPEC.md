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
  Claude or Codex CLI subprocess execution. The registered Copilot provider
  fails fast because `gh copilot suggest` is interactive.
- **NDJSON Event Recording**: `.ccswarm/runs/{run-id}/events.ndjson` and
  summaries for observability.
- **Faceted Prompting**: composable persona, policy, knowledge, and instruction
  facets.
- **Session Persistence**: native ccswarm context and result persistence.

Ordinary stage gates are available, but Sangha dispatch currently bypasses
them. Session persistence is not workflow checkpoint resume. The Codex adapter
currently requests a writable workspace even for read-only stages. These
limitations must be fixed before the target Sangha acceptance policy ships.

`lab sangha` proposal storage and self-extension commands remain experimental
and separate from workflow decisions.

## Requirements

- Rust toolchain compatible with the workspace.
- Git 2.20+.
- At least one live backend:
  - `CCSWARM_A2A_ENDPOINT=https://.../a2a` for a remote A2A server, or
  - Claude Code CLI or Codex CLI installed locally.

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
│       │   ├── governance/
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
3. Local provider CLI selected by stage YAML, CLI flag, environment, or default.

The bridge then parses output into structured build/test/log/plain-text results,
updates per-agent context, persists session state, and returns token/cost
metadata when the selected provider exposes it.

## A2A Objects

ccswarm models the A2A Agent Card fields needed for discovery:

- `name`
- `description`
- `supportedInterfaces`
- `capabilities`
- `version`
- `skills`

For execution, ccswarm posts a user `Message` containing a text `Part` to
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

The canonical release roadmap is [v0.10.0 Roadmap](ROADMAP.md). The longer
product vocabulary and implementation direction remain in
[ccswarm Product Abstraction Plan](CCSWARM_PRODUCT_ABSTRACTION_PLAN.md).
