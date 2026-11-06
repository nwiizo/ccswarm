# ccswarm Architecture

## Overview

ccswarm is a single Rust crate (`crates/ccswarm`) that runs YAML flows over AI
coding provider CLIs and A2A-compatible remote agents. It provides flow/stage
orchestration, Sangha consensus, NDJSON event recording, and the `A2ABridge`
that executes each stage.

[Sangha Product Core](SANGHA_PRODUCT_CORE.md) defines the product direction:
the workflow owns acceptance decisions based on evidence and resolved
objections, while providers execute work. The existing quorum implementation
is the starting point; the complete decision protocol is planned behavior.

## Execution Flow

```text
CLI
  -> Workflow Engine (flows, stages, facets, Sangha)
  -> A2ABridge (retry, rate-limit fallback)
       -> A2A REST /message:send when CCSWARM_A2A_ENDPOINT is set
       -> otherwise a local provider CLI (Claude/Codex) in a background Herdr
          tab: headless, or a live Claude session
  -> Output parser
  -> Event recorder + summaries
```

## Components

### Workflow Engine (`src/workflow/`)

- Loads YAML flow definitions with stages.
- Composes prompts from persona, policy, knowledge, and instruction facets.
- Executes sequential, parallel, team-leader, and Sangha stages.
- Emits NDJSON events through `EventRecorder`.
- Will own Sangha report validation, objection resolution, decision policy,
  and recovery routing; these rules do not belong in session helpers.

### A2ABridge (`src/session/bridge.rs`)

- Sends a stage to an A2A server when `CCSWARM_A2A_ENDPOINT` or
  `MovementExecOptions::a2a_endpoint` is set; otherwise runs the selected
  provider through Herdr.
- Retries with backoff and switches providers on rate limits
  (`on_rate_limit`).
- Projects Claude stream-json and Codex JSONL output to text, parses it, and
  reports token counts (real when available, otherwise estimated).

### Herdr Execution (`src/session/herdr.rs`)

- Requires a Herdr-managed pane (`HERDR_ENV=1`, `HERDR_WORKSPACE_ID`) and
  uses `HERDR_BIN_PATH` when Herdr provides it.
- Headless calls submit `/bin/sh <script>` to a new tab and read stdout,
  stderr, and the exit code from a private run directory.
- Live Claude calls use `herdr agent start` / `agent prompt` and read the
  result and token usage from the Claude session transcript.
- Reports stage state and sidebar tokens, sends notifications, and closes the
  tab on completion, failure, or cancellation. See
  [Herdr Integration](HERDR_INTEGRATION.md).

### A2A Client (`src/session/a2a.rs`)

- Posts a user `Message` with a text `Part` to `POST /message:send`.
- Extracts text parts from direct message or task/artifact responses.

### Providers (`src/providers/`)

- Build provider-specific commands for Claude and Codex.
- Copilot remains registered for configuration compatibility but fails fast
  because its CLI is interactive and cannot produce code edits.
- Do not own workflow policy, parsing, or retry behavior.

### Experimental Commands

`lab sangha` stores proposal files that do not determine workflow verdicts.
Action authorization (`ccswarm approve`) remains distinct from acceptance of
a plan or change.

## Verification

```bash
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p ccswarm -- --help
```

See the [roadmap](ROADMAP.md) for the release scope and gates.
