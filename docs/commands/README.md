# Commands Documentation

This directory contains command reference documentation for the ccswarm project.

## Available Documentation

- **[quickstart.md](quickstart.md)** - One-command setup guide for new ccswarm projects
- **[workspace-commands.md](workspace-commands.md)** - Comprehensive guide to working with the Cargo workspace structure
- **[../RESOURCE_MONITORING.md](../RESOURCE_MONITORING.md)** - Resource monitoring and efficiency features

## Additional Command References

For CLI-specific commands, see:
- **[../../.claude/commands/](../../.claude/commands/)** - Detailed ccswarm CLI command documentation
- **[../../CLAUDE.md](../../CLAUDE.md)** - Frequently used commands section

## Quick Reference

### Most Common Workspace Commands

```bash
# Build everything
cargo build --workspace

# Test everything  
cargo test --workspace

# Run ccswarm
cargo run -p ccswarm -- --help

# Check code quality
cargo fmt --all
cargo clippy --workspace -- -D warnings
```

### Resource Monitoring Commands

```bash
# Show resource usage for all agents
cargo run -p ccswarm -- resource status

# Show efficiency statistics
cargo run -p ccswarm -- resource stats

# Configure resource limits
cargo run -p ccswarm -- resource limits --max-cpu 60 --idle-timeout-min 30

# Check and suspend idle agents
cargo run -p ccswarm -- resource check-idle

# Resume a suspended agent
cargo run -p ccswarm -- resource resume <agent-id>
```

For the full list of workspace commands and best practices, see [workspace-commands.md](workspace-commands.md).