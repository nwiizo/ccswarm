# Development Standards

## Code Quality Requirements

- Run before commits: `cargo fmt && cargo clippy -- -D warnings && cargo test`
- **Minimal tests only**: ~10 tests maximum covering core functionality
- Document public APIs with rustdoc
- Keep cyclomatic complexity <10

## Rust Coding Rules

- **Error Handling**: `Result<T, E>` with `thiserror`, no `.unwrap()` in production
- **Type-State Pattern**: Agent state transitions validated at compile time
- **Channel-Based Concurrency**: No `Arc<Mutex>`, use tokio channels
- **Iterator Chains**: Use iterator methods for collection processing
- **Zero-Cost Abstractions**: Compile-time optimizations, no runtime overhead

## Testing Strategy

- Unit tests colocated with implementation in `#[cfg(test)]` modules
- Integration tests in `crates/ccswarm/tests/` directory
- Use `#[tokio::test]` for async tests
- Mock external dependencies with `mockall` or similar
- Run workspace-wide tests with `cargo test --workspace`

## Workspace Commands

```bash
# Essential checks
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace

# Build
cargo build --workspace
cargo build --release --workspace
```
