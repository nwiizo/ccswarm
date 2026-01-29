# Contributing to ccswarm

Thank you for your interest in contributing to ccswarm! This guide will help you get started with contributing to our AI-powered multi-agent orchestration system.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)
- [Submitting Changes](#submitting-changes)
- [Community](#community)

## Code of Conduct

ccswarm is committed to providing a welcoming and inclusive environment for all contributors. Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: Version 2.20 or higher
- **API Keys**: For testing (Anthropic, OpenAI)
- **System Dependencies**: Build tools for your platform

```bash
# Ubuntu/Debian
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
xcode-select --install
brew install openssl pkg-config

# Fedora/RHEL
sudo dnf install gcc openssl-devel pkg-config
```

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:

```bash
git clone https://github.com/yourusername/ccswarm.git
cd ccswarm
```

3. Add the upstream repository:

```bash
git remote add upstream https://github.com/nwiizo/ccswarm.git
```

## Development Setup

### Workspace Structure

ccswarm uses a Rust workspace with two main crates:

```
ccswarm/
├── Cargo.toml              # Workspace definition
├── crates/
│   ├── ccswarm/           # Main orchestration crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   └── tests/
│   └── ai-session/        # Terminal session management (v0.4.0)
│       ├── Cargo.toml
│       ├── src/
│       └── examples/
├── docs/                  # Documentation
├── demos/                 # Example applications
└── .claude/              # Claude-specific configuration
```

### Build and Test

```bash
# Build entire workspace
cargo build --workspace

# Build specific crate
cargo build -p ccswarm
cargo build -p ai-session  # v0.4.0

# Run all tests
cargo test --workspace

# Run specific tests
cargo test -p ccswarm
cargo test -p ai-session  # v0.4.0

# Run with verbose output
cargo test --workspace -- --nocapture
```

### Environment Setup

Create a `.env` file for development:

```bash
cat > .env << 'EOF'
ANTHROPIC_API_KEY=your-key-here
OPENAI_API_KEY=your-key-here
RUST_LOG=debug
CCSWARM_TEST_MODE=true
EOF
```

## Project Structure

### ccswarm Crate (`crates/ccswarm/`)

```
src/
├── main.rs                # CLI entry point
├── lib.rs                # Library root
├── cli/                  # Command-line interface
│   ├── mod.rs
│   ├── commands/
│   └── args.rs
├── orchestrator/         # Master Claude orchestration
│   ├── mod.rs
│   ├── master_claude.rs
│   └── llm_quality_judge.rs
├── agent/               # Agent system
│   ├── mod.rs
│   ├── simple_agent.rs
│   ├── persistent_agent.rs
│   └── pool_agent.rs
├── providers/           # AI provider integrations
│   ├── mod.rs
│   ├── claude_code.rs
│   ├── aider.rs
│   └── custom.rs
├── session/            # Session management
│   ├── mod.rs
│   ├── ai_session_adapter.rs
│   └── session_pool.rs
├── coordination/       # Inter-agent coordination
│   ├── mod.rs
│   ├── task_queue.rs
│   └── message_bus.rs
├── extension/          # Self-extension framework
│   ├── mod.rs
│   └── autonomous_agent_extension.rs
├── sangha/            # Collective intelligence
│   ├── mod.rs
│   ├── consensus.rs
│   └── proposals.rs
├── identity/          # Agent identity management
│   ├── mod.rs
│   └── role_enforcement.rs
├── security/          # Security monitoring
│   ├── mod.rs
│   └── owasp_scanner.rs
├── config/            # Configuration management
│   ├── mod.rs
│   └── validation.rs
├── tui/              # Terminal user interface
│   ├── mod.rs
│   └── components/
└── utils/            # Shared utilities
    ├── mod.rs
    └── error.rs
```

### ai-session Crate (`crates/ai-session/`) - v0.4.0

```
src/
├── lib.rs              # Library root
├── session/           # Core session management
│   ├── mod.rs
│   ├── manager.rs
│   └── persistence.rs
├── pty/              # Platform-specific PTY
│   ├── mod.rs
│   ├── unix.rs
│   └── windows.rs
├── compression/      # Context compression
│   ├── mod.rs
│   └── zstd_compression.rs
├── mcp/             # Model Context Protocol
│   ├── mod.rs
│   └── server.rs
└── utils/           # Utilities
    ├── mod.rs
    └── error.rs
```

## Development Workflow

### Branch Strategy

We use a feature branch workflow:

1. **main**: Stable release branch
2. **develop**: Integration branch for new features
3. **feature/**: Individual feature branches
4. **hotfix/**: Critical bug fixes

### Creating a Feature Branch

```bash
# Update your fork
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name

# Make your changes
# ... code, test, commit ...

# Push to your fork
git push origin feature/your-feature-name

# Create pull request on GitHub
```

### Commit Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Format: <type>(<scope>): <subject>

# Examples:
git commit -m "feat(agent): add role boundary enforcement"
git commit -m "fix(session): resolve memory leak in compression"
git commit -m "docs(api): add session management examples"
git commit -m "test(orchestrator): add delegation strategy tests"
git commit -m "refactor(core): extract common error types"
```

#### Commit Types

- **feat**: New features
- **fix**: Bug fixes
- **docs**: Documentation changes
- **test**: Adding or updating tests
- **refactor**: Code refactoring without functional changes
- **perf**: Performance improvements
- **style**: Code style changes (formatting, etc.)
- **chore**: Maintenance tasks

## Coding Standards

### Language Convention

To ensure accessibility for the broader international open-source community, please use English for:
- Source code comments and rustdoc
- Commit messages and PR descriptions
- Issue and discussion content
- Markdown documentation (README, CLAUDE.md, etc.)
- Agent and command definitions (`.claude/` directory)

This helps contributors across the globe collaborate effectively.

### Rust Guidelines

Follow the [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/) and these additional rules:

#### Code Formatting

```bash
# Format all code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

#### Linting

```bash
# Run clippy on all crates
cargo clippy --workspace -- -D warnings

# Fix common issues
cargo clippy --workspace --fix
```

#### Error Handling

Always use `Result<T, E>` for fallible operations:

```rust
// Good: Proper error handling
pub fn create_session(name: &str) -> Result<Session, SessionError> {
    let session = Session::new(name)
        .map_err(SessionError::CreationFailed)?;
    Ok(session)
}

// Bad: Using unwrap
pub fn create_session(name: &str) -> Session {
    Session::new(name).unwrap() // ❌ Don't do this
}
```

#### Documentation

All public APIs must have documentation:

```rust
/// Creates a new AI session with the specified configuration.
///
/// # Arguments
///
/// * `name` - A unique identifier for the session
/// * `config` - Session configuration options
///
/// # Returns
///
/// Returns a `Result` containing the created session or an error.
///
/// # Errors
///
/// Returns `SessionError::InvalidName` if the name is empty or contains
/// invalid characters.
///
/// # Examples
///
/// ```
/// use ccswarm::session::{Session, SessionConfig};
///
/// let config = SessionConfig::default();
/// let session = Session::create("test-session", config)?;
/// ```
pub fn create(name: &str, config: SessionConfig) -> Result<Session, SessionError> {
    // Implementation
}
```

#### Async Code

Use `tokio` for async operations:

```rust
// Good: Proper async function
#[tokio::test]
async fn test_session_creation() -> Result<(), SessionError> {
    let session = Session::create("test").await?;
    assert_eq!(session.name(), "test");
    Ok(())
}

// Good: Async trait implementation
#[async_trait]
impl Provider for ClaudeCodeProvider {
    async fn execute(&self, task: &Task) -> Result<String, ProviderError> {
        // Implementation
    }
}
```

#### Memory Management

Prefer borrowing over cloning:

```rust
// Good: Use references when possible
pub fn process_task(task: &Task, agent: &Agent) -> Result<(), ProcessError> {
    // Implementation
}

// Good: Use Arc for shared ownership
pub struct SessionPool {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}
```

### Code Organization

#### Module Structure

```rust
// Good: Clear module organization
pub mod session {
    pub mod manager;
    pub mod persistence;
    pub mod compression;

    pub use manager::SessionManager;
    pub use persistence::SessionStore;
}

// Good: Re-export important types
pub use session::{SessionManager, Session, SessionConfig};
pub use agent::{Agent, AgentRole, AgentConfig};
```

#### Error Types

Use `thiserror` for error definitions:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Session not found: {name}")]
    NotFound { name: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {message}")]
    Configuration { message: String },
}
```

## Testing

### Test Organization

Tests are organized into three categories:

1. **Unit Tests**: Test individual components in isolation
2. **Integration Tests**: Test component interactions
3. **End-to-End Tests**: Test complete workflows

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_session_creation() {
        let config = SessionConfig::default();
        let session = Session::create("test", config).await.unwrap();
        assert_eq!(session.name(), "test");
    }

    #[test]
    fn test_role_validation() {
        let role = AgentRole::Frontend;
        assert!(role.can_access_file("src/components/Button.tsx"));
        assert!(!role.can_access_file("server/api.py"));
    }
}
```

### Integration Tests

```rust
// tests/integration_tests.rs
use ccswarm::{Session, Agent, Task};

#[tokio::test]
async fn test_agent_task_execution() {
    let session = Session::create("test").await.unwrap();
    let agent = Agent::new(AgentRole::Frontend, session).await.unwrap();

    let task = Task::new("Create a button component");
    let result = agent.execute(task).await.unwrap();

    assert!(result.contains("button"));
}
```

### Test Utilities

Use test utilities for common setup:

```rust
// tests/common/mod.rs
pub async fn create_test_session() -> Session {
    let config = SessionConfig {
        compression: false,
        persistence: false,
        ..Default::default()
    };
    Session::create("test", config).await.unwrap()
}

pub fn create_test_task(description: &str) -> Task {
    Task::builder()
        .description(description)
        .priority(Priority::Medium)
        .build()
}
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run specific test file
cargo test --test integration_tests

# Run with output
cargo test --workspace -- --nocapture

# Run ignored tests
cargo test --workspace -- --ignored

# Run tests with coverage
cargo tarpaulin --workspace --out Html
```

### Mocking

Use `mockall` for mocking in tests:

```rust
use mockall::{automock, predicate::*};

#[automock]
pub trait Provider {
    async fn execute(&self, task: &Task) -> Result<String, ProviderError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_with_mock_provider() {
        let mut mock_provider = MockProvider::new();
        mock_provider
            .expect_execute()
            .with(eq(task))
            .times(1)
            .returning(|_| Ok("result".to_string()));

        let agent = Agent::new(mock_provider);
        let result = agent.execute(task).await.unwrap();
        assert_eq!(result, "result");
    }
}
```

## Documentation

### Types of Documentation

1. **Code Documentation**: Rustdoc comments for public APIs
2. **User Guides**: Markdown files in `docs/`
3. **Examples**: Working examples in `demos/`
4. **Architecture Documentation**: High-level design documents

### Writing Rustdoc

```rust
/// A session manager that handles AI session lifecycle.
///
/// The `SessionManager` provides a high-level interface for creating,
/// managing, and destroying AI sessions. It supports session pooling,
/// persistence, and automatic cleanup.
///
/// # Examples
///
/// ```
/// use ccswarm::SessionManager;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let manager = SessionManager::new().await?;
///     let session = manager.create_session("my-session").await?;
///
///     // Use the session...
///
///     manager.destroy_session(session.id()).await?;
///     Ok(())
/// }
/// ```
pub struct SessionManager {
    // Fields...
}
```

### Updating Documentation

```bash
# Generate documentation
cargo doc --workspace --no-deps --open

# Check documentation links
cargo doc --workspace --no-deps 2>&1 | grep -E "(warning|error)"

# Build docs with all features
cargo doc --workspace --all-features --no-deps
```

### User Guide Updates

When adding new features, update relevant documentation:

- `README.md`: Overview and quick start
- `docs/GETTING_STARTED.md`: Beginner tutorials
- `docs/CONFIGURATION.md`: Configuration options
- `docs/TROUBLESHOOTING.md`: Common issues and solutions

## Submitting Changes

### Pull Request Process

1. **Create a clear PR title** following conventional commit format
2. **Write a detailed description** explaining the changes
3. **Link related issues** using GitHub keywords
4. **Add tests** for new functionality
5. **Update documentation** as needed
6. **Ensure CI passes** before requesting review

### PR Template

```markdown
## Description
Brief description of changes and motivation.

## Type of Change
- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix or feature causing existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review of code completed
- [ ] Code is commented, particularly in hard-to-understand areas
- [ ] Corresponding documentation updated
- [ ] No new warnings introduced

## Related Issues
Fixes #123
Relates to #456
```

### Review Process

1. **Automated Checks**: CI must pass
2. **Code Review**: At least one maintainer approval required
3. **Testing**: Verify changes work as expected
4. **Documentation**: Ensure docs are updated
5. **Merge**: Squash and merge preferred

## Community

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and general discussion
- **Pull Requests**: Code contributions and reviews

### Getting Help

- Check existing [documentation](docs/)
- Search [GitHub Issues](https://github.com/nwiizo/ccswarm/issues)
- Ask in [GitHub Discussions](https://github.com/nwiizo/ccswarm/discussions)
- Read the [troubleshooting guide](docs/TROUBLESHOOTING.md)

### Reporting Issues

When reporting bugs, include:

1. **ccswarm version**: `ccswarm --version`
2. **System information**: OS, Rust version
3. **Configuration**: Relevant parts of `ccswarm.json`
4. **Steps to reproduce**: Clear reproduction steps
5. **Expected vs actual behavior**: What went wrong
6. **Logs**: Relevant error messages

### Feature Requests

For new features:

1. **Check existing issues** to avoid duplication
2. **Describe the use case** and motivation
3. **Propose a solution** if you have ideas
4. **Consider implementation complexity**
5. **Be open to feedback** and alternative approaches

## Development Tips

### Debugging

```bash
# Enable debug logging
export RUST_LOG=debug
cargo run -- start

# Component-specific logging
export RUST_LOG=ccswarm::session=trace
cargo run -- start

# Use debugger
cargo build
rust-gdb target/debug/ccswarm
```

### Performance Profiling

```bash
# Install profiling tools
cargo install cargo-profiling

# Profile specific operations
cargo profile --features profiling -- session create

# Memory profiling
valgrind --tool=massif target/debug/ccswarm
```

### IDE Setup

For VS Code, install these extensions:

- **rust-analyzer**: Rust language server
- **CodeLLDB**: Debugging support
- **Better TOML**: TOML syntax highlighting
- **Markdown All in One**: Markdown editing

### Common Pitfalls

1. **Forgetting to run tests**: Always run `cargo test --workspace`
2. **Not updating documentation**: Keep docs in sync with code
3. **Ignoring clippy warnings**: Address all linter warnings
4. **Large commits**: Keep commits focused and atomic
5. **Breaking changes**: Consider backward compatibility

---

Thank you for contributing to ccswarm! Your help makes this project better for everyone. If you have questions about contributing, don't hesitate to ask in GitHub Discussions or create an issue.
