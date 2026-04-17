# Development Standards

## Language Convention

Ensure all content is in English per international open-source conventions:
- Source code comments and rustdoc
- Commit messages and PR descriptions
- Markdown documentation
- Agent and command definitions (`.claude/`)
- Issue templates and GitHub content

## Code Quality Requirements

- Run before commits: `cargo fmt && cargo clippy -- -D warnings && cargo test`
- **Minimal tests only**: ~10 tests maximum covering core functionality
- Document public APIs with rustdoc
- Keep cyclomatic complexity <10

## Rust Coding Rules

- **Error Handling**: `Result<T, E>` with `thiserror`, no `.unwrap()` in production
- **Type-State Pattern**: Agent state transitions validated at compile time
- **Channel-Based Concurrency**: No `Arc<Mutex>`, use tokio channels or DashMap
- **Iterator Chains**: Use iterator methods for collection processing
- **Zero-Cost Abstractions**: Compile-time optimizations, no runtime overhead
- **Sensitive Data**: Use `SensitiveString` for API keys (masks in Debug/Display)

## Error Handling Patterns

Errors should provide retry guidance:
```rust
impl Error {
    fn should_retry(&self) -> bool { /* Network, Resource errors */ }
    fn suggested_retry_delay(&self) -> Duration { /* 1-2 seconds */ }
    fn max_retries(&self) -> u32 { /* 0-5 based on error type */ }
}
```

## DashMap Usage (NOT Mutex)

DashMap has a different API than `Arc<Mutex<HashMap>>`:
```rust
// Wrong (Mutex style)
self.map.lock().unwrap().get(&key)

// Correct (DashMap style)
self.map.get(&key)                    // Returns Option<Ref<K, V>>
self.map.get_mut(&key)                // Returns Option<RefMut<K, V>>
self.map.iter()                       // Iterate all entries
self.map.iter_mut()                   // Iterate mutably
entry.value()                         // Get value from entry
entry.value_mut()                     // Get mutable value
```

## Testing Strategy

- Unit tests colocated with implementation in `#[cfg(test)]` modules
- Integration tests in `crates/ccswarm/tests/` directory
- Use `#[tokio::test]` for async tests
- Mock external dependencies with `mockall` or similar
- Run workspace-wide tests with `cargo test --workspace`

### CLI Testing Patterns

**E2E Tests** (`tests/e2e_cli_test.rs`):
```rust
fn run_ccswarm(args: &[&str]) -> std::process::Output {
    Command::new(get_binary_path()).args(args).output().unwrap()
}
```

**Unit Tests** (`tests/cli_unit_tests.rs`):
```rust
use clap::Parser;

let cli = Cli::try_parse_from(["ccswarm", "init", "--name", "Test"]).unwrap();
match cli.command {
    Commands::Init { name, .. } => assert_eq!(name, "Test"),
    _ => panic!("Wrong command"),
}
```

**Pattern Matching for Enum Variants with Fields**:
```rust
// Use { .. } for variants with fields
Commands::Start { daemon, port, .. } => { /* ... */ }
Commands::Doctor { .. } => { /* ... */ }  // Even if ignoring all fields
```

### Mockall Best Practices

**Basic Mock Setup**:
```rust
use mockall::mock;
use mockall::predicate::*;

mock! {
    pub MyService {
        fn execute(&self, input: &str) -> Result<String>;
    }
}

#[test]
fn test_with_mock() {
    let mut mock = MockMyService::new();
    mock.expect_execute()
        .times(1)                              // Verify call count
        .withf(|input| input.contains("test")) // Argument matching
        .returning(|_| Ok("result".into()));   // Return value

    assert!(mock.execute("test input").is_ok());
}
```

**Advanced Patterns**:
```rust
// Ordered calls with Sequence
let mut seq = mockall::Sequence::new();
mock.expect_step1().in_sequence(&mut seq).returning(|| Ok(()));
mock.expect_step2().in_sequence(&mut seq).returning(|| Ok(()));

// Verify method NOT called
mock.expect_dangerous_op().times(0);

// Multiple return values
let counter = AtomicUsize::new(0);
mock.expect_execute().returning(move |_| {
    let n = counter.fetch_add(1, Ordering::SeqCst);
    if n < 2 { Err(anyhow!("retry")) } else { Ok("success".into()) }
});
```

Reference: `tests/mockall_tests.rs` for comprehensive examples.

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
