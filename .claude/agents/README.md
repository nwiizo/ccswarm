# Claude Code Agents

Sub-agents for the ccswarm project.

## Agent List

| Agent | Description | Command |
|-------|-------------|---------|
| `all-reviewer` | Integrated review (design compliance, quality, architecture) | `/review-all` |
| `architecture-reviewer` | Architecture pattern compliance review | `/review-architecture` |
| `rust-fix-agent` | Rust build/clippy error fixing | `/check-impl` |
| `code-refactor-agent` | Duplicate code detection and refactoring | `/review-duplicates` |

## Usage

Invoke via Task tool:

```
subagent_type: "agent-name"
prompt: "[task content]"
```

## Agent Details

### all-reviewer

Integrated agent that runs all reviews at once.

**Check Items:**
- CLAUDE.md design compliance
- docs/ARCHITECTURE.md architecture compliance
- Rust best practices
- Duplicate code detection

**Output**: Integrated report (JSON) covering design compliance, code quality, and architecture

### architecture-reviewer

Specialized architecture pattern review.

**Check Items:**
- Type-State Pattern usage
- Channel-Based vs Arc<Mutex> ratio
- Iterator Pipelines utilization
- Actor Model implementation
- Minimal Testing compliance

**Output**: Evaluation and score for each pattern (JSON)

### rust-fix-agent

Specialized agent for fixing Rust build errors and clippy warnings.

**Features:**
- Compile error fixes
- Clippy warning resolution
- Rust 2024 Edition support

**Principle**: YAGNI (You Aren't Gonna Need It) - minimal necessary fixes

### code-refactor-agent

Agent for duplicate code detection and refactoring.

**Features:**
- Semantic similarity detection using similarity-rs
- Refactoring proposals based on DRY principle
- Conversion to ccswarm patterns

**Detection Categories:**
| Category | Detection Pattern |
|----------|-------------------|
| Common function extraction | 95%+ similarity, 10+ lines |
| Trait conversion | 90-95% similarity, 5+ lines |
| Channel conversion | Arc<Mutex> shared state |

## Model Settings

| Agent | Model | Reason |
|-------|-------|--------|
| all-reviewer | sonnet | Balanced review |
| architecture-reviewer | sonnet | Suitable for pattern analysis |
| rust-fix-agent | opus | High precision for complex fixes |
| code-refactor-agent | opus | High precision for semantic analysis |

## Related

- `.claude/commands/` - Slash command definitions
- `CLAUDE.md` - Project guidelines
- `docs/ARCHITECTURE.md` - Architecture design
