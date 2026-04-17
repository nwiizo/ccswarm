# Implementation Check

Runs basic checks on the ccswarm workspace.

## Execution Content

```bash
# 1. Format/Lint
cargo fmt --all --check && cargo clippy --workspace -- -D warnings

# 2. Tests
cargo test --workspace

# 3. Build verification
cargo build --workspace --release
```

## Check Items

| Item | Command | Criteria |
|------|---------|----------|
| Format | `cargo fmt --all --check` | No errors |
| Lint | `cargo clippy --workspace -- -D warnings` | No warnings |
| Tests | `cargo test --workspace` | All pass |
| Build | `cargo build --workspace` | No errors |

## Rust 2024 Edition Support

ccswarm uses Rust 2024 Edition. Note the following:

- `std::env::set_var` requires `unsafe` block
- Pattern matching has implicit borrowing so `ref`/`ref mut` are unnecessary
- Explicit type annotations are required more often

## Output Format

```json
{
  "format": "OK|NG",
  "clippy": {
    "warnings": N,
    "errors": N
  },
  "test": {
    "passed": N,
    "failed": N,
    "ignored": N
  },
  "build": "OK|NG",
  "overall": "OK|NG"
}
```

## Related

- `/review-all` - Full review (includes design compliance and quality)
- `/review-duplicates` - Duplicate code detection
