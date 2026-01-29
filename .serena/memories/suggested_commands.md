# ccswarm Development Commands

## Build Commands
- `cargo build --workspace` - Build entire workspace
- `cargo build --release --workspace` - Build release version
- `cargo build -p ccswarm` - Build specific crate
- `cargo build -p ai-session` - Build ai-session crate (v0.4.0)

## Test Commands
- `cargo test --workspace` - Run all tests
- `cargo test --lib -p ccswarm` - Test ccswarm library
- `cargo test --lib -p ai-session` - Test ai-session library (v0.4.0)
- `cargo test -- --nocapture` - Show test output

## Code Quality
- `cargo fmt --all` - Format all code
- `cargo clippy --workspace -- -D warnings` - Run linter with warnings as errors
- `cargo check` - Fast type checking
- `cargo tarpaulin` - Code coverage (if installed)

## Running ccswarm
- `cargo run -p ccswarm -- --help` - Show help
- `cargo run -p ccswarm -- init --name "ProjectName"` - Initialize project
- `cargo run -p ccswarm -- start` - Start system
- `cargo run -p ccswarm -- tui` - Terminal UI
- `cargo run -p ccswarm -- task "description"` - Create task
- `cargo run -p ccswarm -- auto-create "app description"` - Auto-create app

## Utility Commands (Darwin/macOS)
- `git status` - Check git status
- `git diff` - Show changes
- `ls -la` - List files with details
- `find . -name "*.rs"` - Find Rust files
- `grep -r "pattern" .` - Search for pattern
- `rg "pattern"` - Fast search with ripgrep
- `tree -L 2` - Show directory tree

## Combined Quality Check
- `cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace` - Format, lint, and test