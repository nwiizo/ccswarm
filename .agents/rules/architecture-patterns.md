# Architecture Patterns

## Rust-Native Patterns

### What Works
- **Type-State Pattern**: Enforce required state transitions at compile time.
- **Channel-Based Orchestration**: Use messages to isolate agent state and make ownership clear.
- **Iterator Pipelines**: Use iterators when they make data flow easier to follow.
- **Actor Model**: Use actors where independent lifecycles and message handling clarify concurrency.
- **Focused Testing**: Cover core behavior and affected failure paths without a fixed suite-size limit.

### What Doesn't Work
- **Layered Architecture**: Unnecessary abstraction in Rust
- **Unnecessary shared state**: Prefer ownership or messages when multiple owners are not required.
- **Redundant tests**: Avoid checks that only repeat implementation details; preserve meaningful regression coverage.
- **Complex Abstractions**: Prefer direct code when it expresses the required behavior clearly.

## Refactoring Decisions

- Use `similarity-rs` and `cargo-coupling` through `nwiizo-coding-style` for structural changes. Confirm candidate pairs in the code; counts and scores are diagnostic evidence, not refactoring targets.
- Preserve intentional differences between role configuration, hook events, and public entrypoints. Reuse existing state transitions and builders before adding shared abstractions.
- Keep prompt formatting and precedence stable when consolidating facet rendering; validate the composed text, including empty sections and raw-content overrides.

## Module Boundaries

- `workflow/` owns orchestration, routing, Sangha decisions, and policy.
- `session/` owns execution primitives, output parsing, context, and persistence.
- `providers/` builds provider-specific commands without owning workflow policy.

## Concurrency Rules

- Use `tokio::sync::mpsc` channels for agent communication
- Prefer `RwLock` over `Mutex` when reads dominate
- Never hold locks across `.await` points
- Use channel-based coordination, not shared state

## Provider Integration

`A2ABridge` uses the configured A2A REST endpoint when present and otherwise
executes local Claude or Codex CLI commands. Keep provider capabilities and
current limitations aligned with `docs/APPLICATION_SPEC.md`; do not treat
planned Sangha or Astra capabilities as implemented behavior.
