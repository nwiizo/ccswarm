# Repository Guidelines

## Project Structure & Module Organization
The workspace root only carries workspace manifests (`Cargo.toml`, `ccswarm.json`, lint configs). Core orchestration and CLI glue live in `crates/ccswarm/src/{cli,orchestrator,agent,providers}`, while `crates/ai-session/src` mirrors the layout for the shared session runtime. Each crate owns a sibling `tests/` directory for integration coverage; reusable fixtures and prompt snippets live under `sample/`, and long-form docs sit in `docs/`.

## Build, Test, and Development Commands
- `cargo build --workspace`: compile the full workspace so shared APIs stay ABI-compatible.
- `cargo run -p ccswarm -- tui|doctor|template list`: smoke-check CLI modes locally.
- `cargo test --workspace -- --nocapture`: run fast+slow suites; add `-p <crate>` for targeted iteration.
- `cargo fmt && cargo clippy --workspace -- -D warnings`: enforce formatting plus a zero-lint bar.
- `./scripts/dev/regen-templates.sh`: rerender template artifacts whenever template sources change.

## ai-session & ccswarm Interop
- `crates/ai-session` exports `AISessionManager` plus terminal handles consumed inside `crates/ccswarm/src/session`. Set `SessionConfig.force_headless = true` (and leave `allow_headless_fallback` on) when running in CI sandboxes that block `openpty`.
- `crates/ccswarm/tests/ai_session_bridge.rs` is the health check for this contract; run `cargo test -p ccswarm --test ai_session_bridge` whenever editing either crate’s session glue.
- Reserve raw PTY usage (for instance `parallel_executor` calling `PtyHandle::spawn_claude`) for developer machines or trusted runners; gate risky flows with feature flags or capability checks.

## Coding Style & Naming Conventions
Adhere to `rustfmt` defaults (4-space indent, grouped imports) and keep functions near 100 lines by extracting helpers. Use `snake_case` for functions/modules, `PascalCase` for types/traits, and `SCREAMING_SNAKE_CASE` for constants. Document any public item with rustdoc that states behavior plus failure modes, prefer `anyhow::Result` for CLI/application layers, and register every new file inside its parent `mod.rs`. Shared defaults belong in `ccswarm.json`.

## Testing Guidelines
Keep unit tests next to the modules they cover and reserve `<crate>/tests/` for multi-module flows. Name regressions `<feature>_<condition>` (`task_routing_handles_pool_overflow`) and source fixtures from `sample/` to ensure deterministic IO. Run `cargo test -p <crate>` before pushing focused changes, and guard slow orchestration suites behind feature flags or environment toggles so CI remains <5 minutes.

## Commit & Pull Request Guidelines
Follow the existing `type(scope): summary` pattern (`refactor(session): ...`, `test(cli): ...`) and reference issues with `Fixes #123` when relevant. PRs should summarize intent, enumerate notable changes, attach test evidence (logs or screenshots for UX work), and call out config/template migrations. Loop in the maintainer for each touched crate and aim for concise diffs (<400 lines) to keep review flow quick.
