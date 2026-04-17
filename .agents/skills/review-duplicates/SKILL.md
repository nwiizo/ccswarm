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

Classify results by priority:

| Priority | Condition | Action |
|----------|-----------|--------|
| High | 95%+ similarity, 10+ lines | Extract to common function |
| Medium | 90-95% similarity, 5+ lines | Consider trait or macro |
| Low | 85-90% similarity | Evaluate structural similarity |
| Skip | Test code, config structs | Allow intentional duplication |

Output a JSON report with `high_priority`, `medium_priority`, `low_priority`, and `ignored` sections.
