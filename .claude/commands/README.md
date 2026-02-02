# Claude Code Commands

Slash commands for the ccswarm project.

## Command List

| Command | Description |
|---------|-------------|
| `/review-all` | Full review (design compliance, quality, duplicate detection) |
| `/review-duplicates` | Duplicate code detection (refactoring candidate identification via similarity-rs) |
| `/review-architecture` | Architecture review (Type-State, Channel-Based pattern compliance) |
| `/check-impl` | Implementation check (fmt, clippy, test) |
| `/check-production-ready` | Production readiness check (Rust best practices) |

## Usage

Execute in Claude Code as follows:

```
/review-all
```

## Flow

### Development Flow

1. Development work
2. `/check-impl` - Basic checks
3. `/review-duplicates` - Duplicate code detection
4. `/review-all` - Full review

### Review Flow

`/review-all` executes the following:
- CLAUDE.md design compliance check
- Rust best practices check
- Duplicate code detection (`/review-duplicates`)
- Architecture pattern compliance check

## Related

- `.claude/agents/` - Agent definitions
- `CLAUDE.md` - Project guidelines
- `docs/ARCHITECTURE.md` - Architecture design
