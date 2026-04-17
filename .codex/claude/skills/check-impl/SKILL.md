---
name: check-impl
description: Implementation verification - runs format, lint, test, and build checks on the workspace.
user-invocable: true
---

Run implementation checks on the ccswarm workspace:

```bash
# 1. Format check
cargo fmt --all --check

# 2. Lint check
cargo clippy --workspace -- -D warnings

# 3. Tests
cargo test --workspace

# 4. Build verification
cargo build --workspace
```

Note: ccswarm uses Rust 2024 Edition. `std::env::set_var` requires `unsafe`, pattern matching has implicit borrowing.

Report results as JSON: `{ format, clippy: { warnings, errors }, test: { passed, failed, ignored }, build, overall }`.
