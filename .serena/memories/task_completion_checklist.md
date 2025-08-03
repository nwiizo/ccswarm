# Task Completion Checklist

When completing any coding task in ccswarm, follow these steps:

## 1. Code Quality Checks
Run the following commands in order:

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --workspace -- -D warnings

# Run tests
cargo test --workspace
```

## 2. Combined Quality Check
For convenience, run all checks at once:
```bash
cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace
```

## 3. Verify Changes
- Check git diff to review changes: `git diff`
- Ensure no unintended files are modified
- Verify all tests pass
- Check that formatting is correct

## 4. Documentation
- Update rustdoc comments if APIs changed
- Update README if features added
- Add examples for new functionality

## 5. Before Committing
- All tests must pass
- No clippy warnings
- Code is properly formatted
- Documentation is updated
- No hardcoded secrets or API keys

## 6. Commit Message Format
Use conventional commits:
- `feat(module): add new feature`
- `fix(module): fix bug description`
- `refactor(module): improve code structure`
- `docs(module): update documentation`
- `test(module): add/update tests`