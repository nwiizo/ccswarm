# Command Reference

## Workspace Management

```bash
cargo build --workspace              # Build entire workspace
cargo test --workspace               # Test entire workspace
cargo build -p ccswarm               # Build specific crate
cargo build -p ai-session
cargo run -p ccswarm -- --help       # Run ccswarm
cargo fmt --all                      # Format workspace
cargo clippy --workspace -- -D warnings  # Lint workspace
cargo doc --workspace --no-deps --open   # Generate docs
```

## Claude ACP Commands

```bash
cargo run -p ccswarm -- claude-acp test      # Test connection
cargo run -p ccswarm -- claude-acp start     # Start adapter
cargo run -p ccswarm -- claude-acp send --task "..."  # Send task
cargo run -p ccswarm -- claude-acp status    # Check status
cargo run -p ccswarm -- claude-acp diagnose  # Run diagnostics
```

## Development Workflow

```bash
# Initialize
cargo run -p ccswarm -- init --name "MyProject" --agents frontend,backend

# Start system
cargo run -p ccswarm -- start
cargo run -p ccswarm -- tui

# Task management
cargo run -p ccswarm -- task "Task description [high] [feature]"
cargo run -p ccswarm -- task list --status pending
cargo run -p ccswarm -- delegate task "Task" --agent backend

# Session management
cargo run -p ccswarm -- session list
cargo run -p ccswarm -- session stats
cargo run -p ccswarm -- session attach <session-id>
```

## Debugging

```bash
RUST_LOG=debug cargo run -p ccswarm -- start
RUST_LOG=ccswarm::session=trace cargo run -p ccswarm -- start
cargo run -p ccswarm -- agent list
cargo run -p ccswarm -- logs --agent frontend --tail 50
cargo run -p ccswarm -- review status
```

## Advanced Features

```bash
# Auto-create
cargo run -p ccswarm -- auto-create "Description..."

# Sangha
cargo run -p ccswarm -- sangha propose --type feature --title "..."
cargo run -p ccswarm -- sangha vote <id> aye --reason "..."

# Extension
cargo run -p ccswarm -- extend autonomous --continuous
```

## System Health

```bash
cargo run -p ccswarm -- health --check-agents --check-sessions
cargo run -p ccswarm -- health --diagnose --detailed
cargo run -p ccswarm -- doctor
cargo run -p ccswarm -- doctor --fix
```

## New User Experience

```bash
cargo run -p ccswarm -- setup        # Interactive wizard
cargo run -p ccswarm -- tutorial     # Interactive tutorial
cargo run -p ccswarm -- help-topic "agent management"
```
