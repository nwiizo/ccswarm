# ccswarm Codex Guide

## Goal

`ccswarm` is the workflow and governance layer built on top of `ai-session`.
Keep terminal/session primitives in `crates/ai-session` and workflow logic in
`crates/ccswarm`.

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
- Prefer `Result<T, E>` with `thiserror`; avoid `.unwrap()` in production code.
- Prefer Rust-native patterns already used in this repo: type-state, channels,
  iterator pipelines, and actor-style isolation where they improve clarity.
- Avoid adding workflow logic to `crates/ai-session`.
- Keep tests focused on core behavior instead of growing broad fragile suites.
- Never hardcode secrets. Use environment variables and `SensitiveString` when
  handling API keys.

## Codex Assets

- Repo skills live in `.agents/skills/`.
- Custom subagents are defined in `.agents/agents/*.toml` and registered from
  `.codex/config.toml`.
- The original Claude configuration is mirrored under `.codex/claude/` for
  reference.
- Detailed project rules remain available in `.codex/claude/rules/*.md`.

## Available Subagents

- `frontend-specialist`
- `backend-specialist`
- `devops-specialist`
- `qa-specialist`
- `rust-fix-agent`
- `code-refactor-agent`
- `architecture-reviewer`
- `all-reviewer`
- `review-fix-agent`

## Recommended Codex Skills

- `review-fix`: review the current diff, fix clear issues, then re-run
  `cargo clippy --workspace -- -D warnings` and `cargo test --workspace --no-run`.
  This is the repo-local Codex counterpart to the personal `home-self-review`
  workflow.
