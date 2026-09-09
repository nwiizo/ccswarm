# ccswarm Architecture Documentation

## System Architecture Overview

ccswarm is an AI Agent Workflow DevOps toolchain that complements AI coding
provider CLIs and A2A-compatible remote agents. It provides flow/stage
orchestration, Sangha consensus, governance, NDJSON event recording, and a
provider-agnostic `A2ABridge` for live execution.

[Sangha Product Core](SANGHA_PRODUCT_CORE.md) defines the product direction:
the workflow owns acceptance decisions based on evidence and resolved
objections, while providers execute work. The existing quorum implementation
is the starting point; the complete decision protocol is planned behavior.

## Workspace Structure

The Cargo workspace now contains a single runtime crate:

```text
ccswarm/
├── Cargo.toml
└── crates/
    └── ccswarm/
        ├── Cargo.toml
        └── src/
```

`crates/ccswarm/src/session` owns the execution primitives that used to live in
a separate session crate: context history, output parsing, prompt preparation,
provider subprocess execution, persistence, and A2A client types.

## Execution Flow

```text
CLI
  -> Workflow Engine (flows, stages, facets, Sangha)
  -> A2ABridge
       -> A2A REST /message:send when CCSWARM_A2A_ENDPOINT is set
       -> otherwise a supported local provider CLI (Claude/Codex)
  -> Output parser + context persistence
  -> Event recorder + summaries
```

## Core Components

### Workflow Engine (`crates/ccswarm/src/workflow/`)

- Loads YAML flow definitions with stages.
- Composes prompts from persona, policy, knowledge, and instruction facets.
- Executes sequential, parallel, team-leader, and Sangha stages.
- Emits NDJSON events through `EventRecorder`.
- Will own Sangha report validation, objection resolution, decision policy,
  and recovery routing; these rules do not belong in session helpers.

### A2ABridge (`crates/ccswarm/src/session/bridge.rs`)

- Sends tasks to an A2A server when `CCSWARM_A2A_ENDPOINT` or
  `MovementExecOptions::a2a_endpoint` is set.
- Falls back to local provider CLI execution using the provider abstraction.
- Supports provider-specific continuation for Claude and Codex.
- Handles retry, rate-limit fallback, stream telemetry projection, context
  history, output parsing, token estimates, and session persistence.

### A2A Types (`crates/ccswarm/src/session/a2a.rs`)

- Models Agent Card discovery metadata.
- Provides a small REST client for `POST /message:send`.
- Extracts text parts from direct message or task/artifact responses.

### Providers (`crates/ccswarm/src/providers/`)

- Builds provider-specific subprocess commands for Claude, Codex, and GitHub
  Copilot.
- Copilot remains registered for configuration compatibility but fails fast
  because its CLI is interactive and cannot produce non-interactive code edits.
- Does not own workflow policy, persistence, parsing, or retry behavior.

### Governance (`crates/ccswarm/src/governance/`)

- Stores proposals, extensions, approvals, and in-process agent messages.
- Sangha workflow consensus is implemented as a workflow primitive and uses the
  governance vocabulary for decisions.
- Experimental `lab sangha` proposal files do not determine workflow verdicts.
  Action authorization remains distinct from acceptance of a plan or change.

## A2A Integration

A2A support follows the official Agent2Agent data model: ccswarm models Agent
Card metadata, exchanges `Message` objects containing `Part`s, and submits work
through task/message operations. Live remote execution currently requires an
explicit endpoint (`CCSWARM_A2A_ENDPOINT` or `MovementExecOptions::a2a_endpoint`)
and uses the HTTP+JSON `POST /message:send` path; local provider CLIs remain the
default.

## Verification

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p ccswarm -- --help
```

## Product Roadmap

See [v0.10.0 Roadmap](ROADMAP.md) for the release scope and gates. The
[ccswarm Product Abstraction Plan](CCSWARM_PRODUCT_ABSTRACTION_PLAN.md) retains
the longer job-theory plan for orders, desk, reports, verdicts, gates,
escalation, recovery points, and phase-level observability.
