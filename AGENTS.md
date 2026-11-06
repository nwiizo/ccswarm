# ccswarm Agent Guide

## Goal

`ccswarm` is the workflow, Sangha, A2A, and Herdr provider execution layer.
`workflow/` owns orchestration, routing, Sangha decisions, and policy;
`session/` owns A2A and Herdr execution and output parsing; `providers/`
builds provider-specific commands. These modules live in `crates/ccswarm/src/`.

## Read First

- `CLAUDE.md`
- `docs/ARCHITECTURE.md`
- `docs/APPLICATION_SPEC.md`

## Build And Verify

- `cargo fmt --all`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `cargo run -p ccswarm -- --help`

## Project Rules

- Write comments, rustdoc, commit messages, and markdown docs in English.
- Document public APIs with rustdoc. Preserve prompt formatting and precedence
  when consolidating facets, including empty sections and raw-content overrides.
- Use `nwiizo-coding-style` for implementation and simplification. Preserve the
  language and resources of that bundled upstream skill when refreshing it.
- Use `anyhow::Result` with context; avoid `.unwrap()` in production code.
- Prefer the repo's type-state, channel, iterator, and actor patterns where they
  improve clarity. Prefer ownership or messages for coordination; use locks when
  shared state requires them and never hold a lock guard across `.await`.
- Keep tests focused on core behavior and affected failure paths, without a fixed
  test-count limit. Isolate configuration and login tests from user state.
- For lint failures in macro-generated code, inspect the generating dependency
  and compatible upstream fixes before adding a local lint allowance.
- Never hardcode secrets. Use environment variables and `SensitiveString` for
  API keys; keep credentials and runtime state outside Git. Validate inputs at
  trust boundaries and protect `.env`, `*.key`, and `.git/` contents.
- Treat `docs/APPLICATION_SPEC.md` as the current capability reference;
  planned Sangha behavior is not implemented behavior.
- Delegate only when the user or applicable instructions explicitly request it.
  Pass scope, ownership, and verification requirements; preserve others' edits
  and reconcile the complete result.
- Commit or push only when requested. Use conventional commits and verify the
  branch and remote before publication.

## Agent Assets

- Shared skills live in `.agents/skills/`. `.claude/skills/` links to the same
  directories so Claude Code and Codex use one maintained copy.
- The bundled `nwiizo-coding-style` skill is copied from `nwiizo/dotfiles`,
  including its references, metadata, and license. Preserve those resources
  together when refreshing the copy.
- Project rules are maintained here; `CLAUDE.md` imports this guide.
- Codex subagents are defined in `.agents/agents/*.toml` and registered from
  `.codex/config.toml`. Claude Code definitions live in `.claude/agents/`.

## Available Subagents

- `frontend-specialist`: client-side work in target projects and examples;
  also referenced by the builtin `team` flow.
- `backend-specialist`: Rust, workflow, provider, and persistence work;
  also referenced by the builtin `team` flow.
- `review-fix-agent`: scoped review and correction using the shared
  `review-fix` skill.

## Available Skills

- `nwiizo-coding-style`: implementation, simplification, and Rust structural
  analysis with `similarity-rs` and `cargo-coupling`.
- `check-impl`: workspace format, lint, test, build, and CLI checks. Keep its
  first shell block executable: the pipeline reads it through
  `.claude/skills/check-impl/SKILL.md`.
- `review-fix`: inspect the requested diff, fix clear correctness issues, and
  verify the affected behavior.
- `deploy-workflow`: prepare and verify releases; publish only when authorized.
