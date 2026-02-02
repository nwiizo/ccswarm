# Workspace Commands Reference

This document provides a comprehensive reference for working with the ccswarm workspace structure.

## Workspace Structure

The ccswarm project uses a Cargo workspace:
- `crates/ccswarm/` - Main application and orchestration system
- `crates/ai-session/` - Native AI session management library (v0.4.0)

## Common Workspace Commands

### Building

```bash
# Build entire workspace
cargo build --workspace

# Build in release mode
cargo build --workspace --release

# Build specific crate
cargo build -p ccswarm
cargo build -p ai-session  # v0.4.0

# Clean and rebuild
cargo clean
cargo build --workspace
```

### Testing

```bash
# Test entire workspace
cargo test --workspace

# Test with output displayed
cargo test --workspace -- --nocapture

# Test specific crate
cargo test -p ccswarm
cargo test -p ai-session  # v0.4.0

# Run integration tests only
cargo test --workspace --test '*'

# Run with coverage (requires cargo-tarpaulin)
cargo tarpaulin --workspace
```

### Code Quality

```bash
# Format all code
cargo fmt --all

# Check formatting without applying
cargo fmt --all -- --check

# Run clippy on entire workspace
cargo clippy --workspace -- -D warnings

# Fix clippy suggestions
cargo clippy --workspace --fix

# Check for outdated dependencies
cargo outdated -w
```

### Documentation

```bash
# Generate docs for entire workspace
cargo doc --workspace --no-deps

# Generate and open docs
cargo doc --workspace --no-deps --open

# Generate docs for specific crate
cargo doc -p ccswarm --no-deps --open
cargo doc -p ai-session --no-deps --open  # v0.4.0
```

### Running ccswarm

```bash
# Run from workspace root
cargo run -p ccswarm -- --help

# Run with arguments
cargo run -p ccswarm -- init --name "MyProject"
cargo run -p ccswarm -- start
cargo run -p ccswarm -- tui

# Run in release mode
cargo run -p ccswarm --release -- start
```

### Development Workflow

```bash
# 1. Make changes in appropriate crate
cd crates/ccswarm/src
# ... edit files ...

# 2. Test locally
cd ../..  # back to crate root
cargo test

# 3. Test workspace-wide impact
cd ../..  # back to workspace root
cargo test --workspace

# 4. Check code quality
cargo fmt --all
cargo clippy --workspace -- -D warnings

# 5. Build and verify
cargo build --workspace
```

### Dependency Management

```bash
# Add dependency to specific crate
cd crates/ccswarm
cargo add tokio --features full

# Update dependencies
cargo update

# Check dependency tree
cargo tree

# Check for security vulnerabilities
cargo audit
```

### Publishing (for maintainers)

```bash
# Publish ai-session first (dependency) - v0.4.0
cd crates/ai-session
cargo publish --dry-run
cargo publish

# Then publish ccswarm
cd ../ccswarm
cargo publish --dry-run
cargo publish
```

## Workspace Configuration

The workspace is configured in the root `Cargo.toml`:

```toml
[workspace]
members = ["crates/ccswarm", "crates/ai-session"]  # ai-session: v0.4.0
resolver = "2"

[workspace.package]
edition = "2021"
# Shared metadata
```

## Best Practices

1. **Always run tests from workspace root** to ensure integration
2. **Use `-p` flag** to target specific crates when needed
3. **Format and lint** before committing
4. **Document changes** in both crate-specific and workspace READMEs
5. **Keep dependencies synchronized** across crates when possible

## Troubleshooting

### Build Cache Issues
```bash
# Clear build cache
cargo clean
rm -rf target/
```

### Dependency Conflicts
```bash
# Check for duplicate dependencies
cargo tree --duplicates

# Update lock file
rm Cargo.lock
cargo build --workspace
```

### Test Failures
```bash
# Run tests with more output
RUST_BACKTRACE=1 cargo test --workspace -- --nocapture

# Run specific test
cargo test -p ccswarm test_name -- --exact
```