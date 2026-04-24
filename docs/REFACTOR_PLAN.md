# Refactor Plan

## Purpose

This document defines an integrated refactoring plan for the `ccswarm`
workspace using multiple local signals:

- duplicate detection from `similarity-rs crates/ --threshold 0.80`
- modularity and coupling analysis from `cargo coupling . --exclude-tests`
- existing project architecture rules from `AGENTS.md`, `CLAUDE.md`, and
  `docs/ARCHITECTURE.md`

The goal is not to "remove duplication everywhere". The goal is to reduce
maintenance cost and change risk while preserving the architecture boundary:

- `ai-session` = terminal/session primitives
- `ccswarm` = workflow + governance

## Analysis Inputs

### Duplicate Analysis

The latest duplicate scan reported **1,962 duplicate pairs**. Most are tests,
small accessors, or intentionally repetitive builders. The high-signal
production duplicates are concentrated in:

- `crates/ccswarm/src/cli/handlers/session.rs`
- `crates/ccswarm/src/hooks/registry.rs`
- `crates/ccswarm/src/cli/handlers/task.rs`
- `crates/ccswarm/src/workflow/repertoire.rs`
- `crates/ccswarm/src/workflow/facets.rs`
- `crates/ccswarm/src/identity/mod.rs`
- `crates/ccswarm/src/cli/handlers/sangha.rs`
- `crates/ai-session/src/bin/ai-session.rs`

### Coupling Analysis

Fresh `cargo coupling . --summary --exclude-tests` output on April 18, 2026:

- Grade: **D**
- Score: **0.44 / 1.00**
- Modules: **114**
- Needs Refactoring: **115 couplings (6%)**
- High issues: **34**
- Medium issues: **14**
- Circular dependencies: **2 cycles**

Top hotspot modules from `cargo coupling . --hotspots=12 --exclude-tests`:

1. `ccswarm::agent`
2. `ccswarm::cli`
3. `ccswarm::cli::quickstart_simple`
4. `ccswarm::workflow::flow`
5. `ccswarm::workflow::execution`
6. `ccswarm::session`
7. `ccswarm::session::bridge`
8. `ccswarm::workflow::cycle`
9. `ccswarm::agent::orchestrator::agent_orchestrator`
10. `ccswarm::cli::handlers::queue`
11. `ccswarm::agent::task_builder_typestate`
12. `ccswarm::providers::tests`

### Impact Highlights

From targeted `cargo coupling --impact` runs:

- `ccswarm::agent`: **HIGH** risk, 6 affected modules
- `ccswarm::session`: **MEDIUM** risk, 29 outgoing couplings, strongly tied to
  `ai-session::ai_session`
- `ccswarm::cli`: **LOW** downstream impact, but intrusive dependence on
  `ccswarm::config`
- `ccswarm::session::bridge`: **LOW** downstream impact, but many direct
  provider and `ai-session` couplings
- `ai-session::core`: **HIGH** risk and part of a circular dependency cycle

## Planning Principles

- Prefer **boundary cleanup before abstraction cleanup**.
- Prefer **small helpers before traits**, and **traits before macros**.
- Refactor **high-change / high-coupling modules** before low-value test
  cleanup.
- Keep user-visible CLI strings and JSON shapes stable unless the work item is
  explicitly about UX.
- Split work by behavior boundary so that each PR is independently reviewable.

## Strategic Diagnosis

The workspace currently has two separate but related problems:

1. **Local duplication**
   - repeated message rendering
   - repeated file loading / YAML parsing
   - repeated role/facet data construction
   - repeated HTTP request / output plumbing

2. **Boundary stress**
   - `ccswarm::agent` couples directly to volatile `identity` and `hooks`
   - `ccswarm::session` is tightly coupled to `ai-session::ai_session`
   - `ccswarm::cli` touches configuration details too directly
   - `ai-session::core` participates in circular dependencies with
     `context` and `persistence`

If we only remove duplicate lines, coupling stays high. If we only chase
coupling metrics, we risk large abstract refactors with weak local payback.
The plan below combines both.

## Refactor Workstreams

### Workstream A: Safe Duplicate Cleanup In Low-Risk Handlers

Purpose:

- reduce repeated code with minimal architectural movement
- create momentum and smaller diffs before deeper boundary changes

Targets:

- `crates/ccswarm/src/cli/handlers/session.rs`
- `crates/ccswarm/src/cli/handlers/task.rs`
- `crates/ccswarm/src/workflow/repertoire.rs`

Actions:

1. Extract helper functions for repeated JSON/text output in session and task
   handlers.
2. Extract common directory scan / YAML enumeration logic in `repertoire.rs`.
3. Preserve existing output fields and command behavior exactly.

Why first:

- These modules show real duplication.
- `ccswarm::cli` has low downstream impact in coupling analysis, so internal
  cleanup is relatively safe.

Expected result:

- lower code repetition
- smaller handler methods
- easier follow-up decomposition of `ccswarm::cli`

### Workstream B: Hook And Agent Boundary Stabilization

Purpose:

- reduce cascading change risk around `ccswarm::agent`
- centralize repetitive hook registry behavior

Targets:

- `crates/ccswarm/src/hooks/registry.rs`
- `crates/ccswarm/src/agent/*`

Actions:

1. Extract generic helper routines in `hooks::registry` for:
   - sorted registration
   - unregister-by-name
   - hook-name listing
   - "run until blocked" iteration
2. Audit `ccswarm::agent` dependencies on `identity` and `hooks`.
3. Introduce narrow internal interfaces only where they reduce direct knowledge
   of hook or identity internals.

Why here:

- `ccswarm::agent` is the top coupling hotspot.
- duplicate cleanup in hooks is directly aligned with coupling reduction.

Constraints:

- Avoid broad trait hierarchies.
- Keep type-state and ownership clarity intact.

Expected result:

- lower churn propagation from hooks/identity into agent logic
- less repeated orchestration code

### Workstream C: CLI Surface Decomposition

Purpose:

- reduce the size and volatility concentration around `ccswarm::cli`
- separate command wiring from command behavior

Targets:

- `crates/ccswarm/src/cli/mod.rs`
- `crates/ccswarm/src/cli/command_registry.rs`
- `crates/ccswarm/src/cli/handlers/mod.rs`
- `crates/ccswarm/src/cli/quickstart_simple.rs`

Actions:

1. Move shared configuration access and command metadata behind small helper
   functions or focused structs.
2. Tighten command-registration responsibilities so handlers do not need broad
   CLI context.
3. Keep command parsing and execution paths separate.

Why here:

- `ccswarm::cli` and `ccswarm::cli::quickstart_simple` are top hotspots.
- temporal coupling already shows `cli/mod.rs`, `command_registry.rs`, and
  `handlers/mod.rs` changing together frequently.

Expected result:

- lower co-change pressure
- smaller blast radius for future CLI feature changes

### Workstream D: Session Boundary Cleanup

Purpose:

- reduce coupling between `ccswarm` workflow logic and `ai-session`
- clarify what belongs in `session`, `session::bridge`, and providers

Targets:

- `crates/ccswarm/src/session/*`
- `crates/ccswarm/src/providers/*`
- callers in `workflow::flow` and CLI workflow handlers

Actions:

1. Split `ccswarm::session` responsibilities into smaller internal units where
   coupling data shows too many outgoing dependencies.
2. Make `session::bridge` the narrowest possible adapter over provider and
   `ai-session` interactions.
3. Prefer provider-facing abstractions in `ccswarm`, not direct `ai-session`
   knowledge in workflow code.

Why here:

- `ccswarm::session` has medium impact but very high outgoing dependency count.
- `workflow::flow` and `session::bridge` are hotspots adjacent to this boundary.

Expected result:

- clearer `ccswarm -> provider -> ai-session` layering
- lower efferent coupling in `ccswarm::session`

### Workstream E: Facet And Role Definition Consolidation

Purpose:

- remove structural duplication in prompt/facet composition and role defaults
- keep growth in these registries manageable

Targets:

- `crates/ccswarm/src/workflow/facets.rs`
- `crates/ccswarm/src/identity/mod.rs`

Actions:

1. Extract shared YAML-loading utilities for personas, policies, and knowledge.
2. Extract text assembly helpers for rendering structured facet content.
3. Convert default role definitions to a more data-driven internal format.

Why here:

- strong duplicate signal
- improves maintainability without forcing a public API rewrite

Constraints:

- keep persona/policy/knowledge semantics explicit
- keep role boundaries readable in code review

Expected result:

- less repetition in registry growth areas
- easier addition of new roles and facets

### Workstream F: File-Backed Governance Handler Cleanup

Purpose:

- unify repeated file-backed command flows
- reduce inconsistent JSON persistence behavior

Targets:

- `crates/ccswarm/src/cli/handlers/sangha.rs`
- related proposal/extension file helpers

Actions:

1. Extract JSON file lifecycle helpers:
   - directory creation
   - id generation
   - read/write helpers
   - common output rendering
2. Refactor `handle_sangha` and `handle_extend` around those helpers.

Why later:

- high duplicate payoff, but touches stateful command behavior
- safer after CLI and helper patterns are already cleaned up

Expected result:

- simpler governance handlers
- lower chance of drift between proposal and extension workflows

### Workstream G: Break `ai-session` Cycles And Remote CLI Duplication

Purpose:

- address the highest architectural risk in the workspace
- reduce circular dependencies and transport boilerplate

Targets:

- `crates/ai-session/src/core/*`
- `crates/ai-session/src/context/*`
- `crates/ai-session/src/persistence/*`
- `crates/ai-session/src/bin/ai-session.rs`

Actions:

1. Break the `core <-> context` cycle.
2. Break the `core <-> persistence` cycle.
3. Move shared remote session HTTP logic behind a thin client/helper layer.
4. Keep core/session primitives independent from CLI presentation concerns.

Why last:

- highest impact and highest regression risk
- should be attempted only after `ccswarm` boundary cleanup is clearer

Expected result:

- improved modularity grade potential
- safer long-term evolution of `ai-session`

## Recommended Order Of Execution

1. Workstream A: safe duplicate cleanup in low-risk handlers
2. Workstream B: hook and agent boundary stabilization
3. Workstream C: CLI surface decomposition
4. Workstream D: session boundary cleanup
5. Workstream E: facet and role definition consolidation
6. Workstream F: file-backed governance handler cleanup
7. Workstream G: break `ai-session` cycles and remote CLI duplication

## PR Breakdown

Recommended PRs:

1. Session/task/repertoire duplicate cleanup
2. Hook registry helper extraction
3. Agent boundary reduction against hooks/identity
4. CLI registration and config access cleanup
5. Session and session-bridge boundary cleanup
6. Facet loader/renderer consolidation
7. Identity role definition consolidation
8. Sangha/extend file workflow consolidation
9. `ai-session` cycle break: core/context
10. `ai-session` cycle break: core/persistence
11. `ai-session` remote CLI request/response consolidation

## Explicitly Deferred

These items should remain out of scope for the first wave:

- most test-only duplication
- `cargo coupling` findings that point only to test modules
- broad interface extraction driven solely by tool naming suggestions
- macro-based deduplication for small repeated bodies

## Verification Gates

Every PR in this plan should pass:

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`

For any workstream that changes CLI behavior or persisted JSON:

- add targeted tests for output stability
- add targeted tests for schema/file compatibility
- compare before/after behavior with representative commands

For coupling-focused workstreams:

- re-run `cargo coupling . --summary --exclude-tests`
- re-run `cargo coupling . --hotspots=12 --exclude-tests`
- re-check the specific `--impact` target being refactored

## Success Criteria

The first refactor wave is successful when:

- Priority production duplicates are reduced,
- `ccswarm::agent`, `ccswarm::cli`, and `ccswarm::session` have clearer
  internal boundaries,
- `ai-session::core` no longer participates in the current dependency cycles,
- the architecture boundary between `ccswarm` and `ai-session` is easier to
  explain and enforce,
- and the workspace can show measurable improvement in `cargo coupling`
  hotspots, not just fewer duplicate lines.
