# Full Review

Runs all reviews at once on the ccswarm codebase.

## Execution Content

The following reviews are executed:

1. **Design Compliance Check** - Consistency with CLAUDE.md, docs/ARCHITECTURE.md
2. **Code Quality** - Rust best practices compliance
3. **Duplicate Code Detection** - Execute `/review-duplicates` command (similarity-rs)
4. **Architecture Patterns** - Type-State, Channel-Based pattern compliance

## Execution Method

Execute with Task tool using `all-reviewer` agent:

```
subagent_type: "Explore"
prompt: "Execute a full review on ccswarm.
1. CLAUDE.md design compliance check
2. docs/ARCHITECTURE.md architecture pattern compliance check
3. Rust best practices compliance check
4. /review-duplicates - Duplicate code detection
Summarize the results for each category in JSON format."
```

## Architecture Pattern Check Items

Patterns based on CLAUDE.md:

| # | Pattern | Criteria |
|---|---------|----------|
| 1 | Type-State Pattern | Compile-time state verification, zero runtime cost |
| 2 | Channel-Based Orchestration | Prefer Channel over Arc<Mutex> |
| 3 | Iterator Pipelines | Zero-cost abstractions |
| 4 | Actor Model | Message passing over locks |
| 5 | Minimal Testing | Around 8-10 tests, focus on core functionality |

## Rust Best Practices Check Items

| # | Item | Criteria |
|---|------|----------|
| 1 | unwrap() elimination | Forbidden in production code |
| 2 | Result<T, E> | Custom error types with thiserror |
| 3 | async/await | Use tokio runtime |
| 4 | Clippy clean | No warnings |
| 5 | Documentation | rustdoc for public APIs |

## Output Format

```json
{
  "compliance": {
    "claude_md": {"compliant": N, "partial": N, "non_compliant": N},
    "architecture": {"compliant": N, "partial": N, "non_compliant": N}
  },
  "code_quality": {
    "rust": {
      "clippy_warnings": N,
      "unsafe_count": N,
      "unwrap_count": N
    },
    "similarity": {
      "duplicate_patterns": N,
      "refactoring_candidates": ["candidate1", "candidate2"]
    }
  },
  "architecture_patterns": {
    "type_state": "OK|NG",
    "channel_based": "OK|NG",
    "iterator_pipelines": "OK|NG",
    "actor_model": "OK|NG",
    "minimal_testing": "OK|NG",
    "score": "N/5"
  },
  "summary": {
    "overall_status": "OK|WARNING|CRITICAL",
    "priority_actions": []
  }
}
```

## Related

- `/check-impl` - Basic checks (format, lint, test)
- `/review-duplicates` - Duplicate code detection (similarity-rs)
- `/review-architecture` - Architecture pattern detailed review
