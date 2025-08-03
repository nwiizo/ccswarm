# CI Troubleshooting Guide for ccswarm

## Common CI Issues and Solutions

### 1. Docker Dependencies
- **Problem**: `bollard` crate not found
- **Solution**: Use feature flags for optional dependencies
  ```toml
  [features]
  container = ["bollard", "tempfile"]
  ```

### 2. Type Name Mismatches
- **Problem**: Old API names causing errors
- **Solution**: Update type names:
  - `SkillCategory` → `WisdomCategory`
  - `Personality` → `AgentPersonality`
  - `Capabilities` → `PersonalityTraits`

### 3. Test Failures
- **Problem**: Integration tests timeout or flaky tests
- **Solution**: Add `#[ignore]` attribute to problematic tests temporarily

### 4. Clippy Warnings
- **Problem**: Various linting warnings
- **Solution**: 
  - Add `#[allow(clippy::...)]` for non-critical warnings
  - Prefix unused variables with underscore

### 5. Example Files Compilation
- **Problem**: Outdated examples fail to compile
- **Solution**: Rename to `.disabled` extension and add to .gitignore

## CI Best Practices

### Local CI Reproduction
```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

### GitHub Actions Configuration
- Use `Swatinem/rust-cache@v2` for caching
- Set `RUST_BACKTRACE=1` for debugging
- Configure appropriate timeouts
- Run tests sequentially if needed: `cargo test -- --test-threads=1`

### Platform-Specific Code
Use `#[cfg(target_os = "...")]` for platform-specific implementations

### Recommended CI Pipeline
1. Format check
2. Clippy linting
3. Unit tests
4. Integration tests
5. Release build