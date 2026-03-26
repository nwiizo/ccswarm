# Claude Code Commands (DEPRECATED)

> **Migrated to Skills**: All commands have been migrated to `.claude/skills/`.
> The `commands/` directory is kept for backward compatibility but will be removed in a future version.

## Migration Map

| Old Command | New Skill | Location |
|-------------|-----------|----------|
| `/review-all` | `/review-all` | `.claude/skills/review-all/SKILL.md` |
| `/review-duplicates` | `/review-duplicates` | `.claude/skills/review-duplicates/SKILL.md` |
| `/review-architecture` | `/review-architecture` | `.claude/skills/review-architecture/SKILL.md` |
| `/check-impl` | `/check-impl` | `.claude/skills/check-impl/SKILL.md` |
| `/check-production-ready` | `/check-production-ready` | `.claude/skills/check-production-ready/SKILL.md` |
| `/mutation-test` | `/mutation-test` | `.claude/skills/mutation-test/SKILL.md` |

## Why Skills?

Skills are the modern Claude Code standard, replacing commands. Benefits:
- YAML frontmatter for metadata (name, description, agent, context)
- `context: fork` runs in a subagent to protect main context
- `agent:` binds to specific subagent types
- `argument-hint:` for autocomplete
- Directory structure supports reference docs and scripts
