---
name: check-impl
description: Run ccswarm workspace format, lint, test, build, and CLI checks.
---

Run implementation checks on the ccswarm workspace:

```bash
cargo fmt --all --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo build --workspace
cargo run -p ccswarm -- --help
```

The pipeline executes this first shell block line by line. Keep it limited to
verification commands; put explanations outside it.

Report which checks passed or failed and include actionable failure details.
Do not claim checks passed when they were skipped or could not run.
