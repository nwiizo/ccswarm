# ccswarm Code Style and Conventions

## Rust Code Style
- **Edition**: 2021
- **Formatting**: Use `cargo fmt` (rustfmt)
- **Linting**: Use `cargo clippy` with warnings as errors
- **Naming**: Snake_case for functions/variables, CamelCase for types
- **Module Organization**: One module per directory with mod.rs

## Error Handling
- Use `Result<T, E>` for fallible operations
- Create custom error types with `thiserror`
- Never use `.unwrap()` in production code
- Always propagate errors with `?` operator

## Documentation
- Document all public APIs with rustdoc comments
- Use `///` for item documentation
- Include examples in documentation
- Document errors with `# Errors` section

## Testing
- Unit tests in `#[cfg(test)]` modules
- Integration tests in `tests/` directory
- Use `#[tokio::test]` for async tests
- Maintain >85% test coverage

## Async Patterns
- Use `tokio` for async runtime
- Prefer `async/await` over futures
- Don't block the runtime with synchronous operations
- Use `tokio::spawn` for concurrent tasks

## Security Requirements
- Never hardcode API keys or secrets
- Validate all user inputs
- Check protected file patterns
- Use environment variables for sensitive data

## Git Workflow
- Never push to main/master directly
- Use feature branches
- Conventional commits: `feat:`, `fix:`, `docs:`, etc.
- All PRs require review and passing CI