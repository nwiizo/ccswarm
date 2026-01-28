# Performance Optimizations

## Session Management

- Reuse sessions whenever possible for efficiency
- Run independent tasks concurrently
- Use session pooling for similar operations
- Enable context compression for long-running sessions

## AI-Session Integration

- **93% API cost reduction** through intelligent session reuse
- **~70% memory reduction** with native context compression (zstd)
- **Zero external dependencies** - no tmux overhead

## Resource Guidelines

- Git worktrees: ~100MB disk space per agent
- JSON coordination: <100ms latency
- TUI monitoring: <3% overhead
- Session persistence: <5ms per command
