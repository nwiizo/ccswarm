# Commands Documentation

This directory contains command reference documentation for the ccswarm project.

## Available Documentation

- **[workspace-commands.md](workspace-commands.md)** - Comprehensive guide to working with the Cargo workspace structure

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

For the full list of workspace commands and best practices, see [workspace-commands.md](workspace-commands.md).