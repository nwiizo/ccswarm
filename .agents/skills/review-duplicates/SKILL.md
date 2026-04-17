---
name: review-duplicates
description: Duplicate code detection using similarity-rs. Identifies refactoring candidates based on semantic similarity.
user-invocable: true
context: fork
agent: code-refactor-agent
---

When subagents are enabled in Codex, prefer delegating this workflow to the `code-refactor-agent` custom agent.

Detect duplicate code in the ccswarm codebase using similarity-rs.

```bash
similarity-rs crates/ --threshold 0.80
```

For Rust code, also consult `cargo coupling` so duplicate cleanup is prioritized
by architectural impact, not only by textual similarity.

```bash
cargo coupling . --summary --exclude-tests
cargo coupling . --hotspots=12 --exclude-tests
```

If a duplicate cluster is located in a coupling hotspot, high-risk boundary, or
cycle-related module, raise its refactoring priority accordingly.

Classify results by priority:

| Priority | Condition | Action |
|----------|-----------|--------|
| High | 95%+ similarity, 10+ lines | Extract to common function |
| Medium | 90-95% similarity, 5+ lines | Consider trait or macro |
| Low | 85-90% similarity | Evaluate structural similarity |
| Skip | Test code, config structs | Allow intentional duplication |

Output a JSON report with `high_priority`, `medium_priority`, `low_priority`, and `ignored` sections.
