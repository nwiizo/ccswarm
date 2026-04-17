---
name: review-fix
description: Code review with automatic fix. Detects bugs, stale references, duplication, provider abstraction gaps, and sort instability, then fixes and verifies.
user-invocable: true
context: fork
agent: review-fix-agent
---

Review uncommitted changes, fix actionable issues, and verify the result.

## Workflow

### Phase 1: Scope

1. `git status --short` and `git diff --stat HEAD` to determine what changed.
2. If no changes exist, abort with "nothing to review".

### Phase 2: Baseline

Run build and lint to establish a clean starting point:

```bash
cargo clippy --workspace -- -D warnings 2>&1 | tail -20
cargo test --workspace --no-run 2>&1 | tail -5
```

If the baseline already fails, report the failures and stop. Do not mask pre-existing breakage.

### Phase 3: Review

Inspect the diff and new untracked files. Check for:

| Category | What to look for |
|----------|-----------------|
| Stale references | Deleted modules or commands still referenced in routing, docs, or templates |
| Code duplication | Identical structs/functions across multiple files |
| Provider abstraction gaps | Hard-coded CLI names instead of `AgentProvider` |
| Dead async | Async operations discarded and then repeated |
| Sort instability | `Utc::now()` used as a fallback sort key |
| Hardcoded paths | Filesystem paths that should be relative to `repo_path` |
| Unused params | CLI arguments accepted but silently ignored |
| Security | Path traversal, credential leaks |

Classify each finding:

- Fix: clear bug or correctness issue. Fix it now.
- Warn: design concern or missing feature. Report it, but do not fix unless trivial.
- Note: style or informational only.

### Phase 4: Fix

For each fix-level finding:

1. Make the minimal, surgical edit.
2. Do not change unrelated code.
3. After all fixes, run:

```bash
cargo clippy --workspace -- -D warnings
cargo test --workspace --no-run
```

If clippy or test compilation fails, revert the offending edit and downgrade it to a warning.

### Phase 5: Report

Output a structured summary:

```text
## Review + Fix Report

### Fixed
- <file>:<line> - <description>

### Warnings
- <file>:<line> - <description>

### Notes
- <description>

### Verification
- clippy: PASS / FAIL
- test compile: PASS / FAIL
- files modified: N
```
