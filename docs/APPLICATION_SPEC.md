# ccswarm Application Specification

## Overview

ccswarm is an AI Agent Workflow DevOps CLI for reproducible coding workflows.
It runs flow-based pipelines, records NDJSON events, supports Sangha consensus
and human approvals, and executes live stages through either a remote A2A agent
or local provider CLIs.

## Key Features

- **Flow-Based Workflows**: YAML-driven multi-step pipelines with context
  passing, retries, gates, parallel stages, and workflow calls.
- **A2ABridge**: A2A REST `message:send` execution when configured, otherwise
  Claude or Codex CLI subprocess execution. The registered Copilot provider
  fails fast because `gh copilot suggest` is interactive.
- **Sangha Consensus**: independent member stages vote with quorum before a
  workflow advances.
- **Self-Extension Framework**: agents propose and vote on capability
  extensions.
- **NDJSON Event Recording**: `.ccswarm/runs/{run-id}/events.ndjson` and
  summaries for observability.
- **Faceted Prompting**: composable persona, policy, knowledge, and instruction
  facets.
- **Session Persistence**: native ccswarm context and result persistence.

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
