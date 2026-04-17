---
name: review-fix
description: Review uncommitted changes, fix actionable correctness issues, and verify the result.
user-invocable: true
context: fork
agent: review-fix-agent
---

When subagents are enabled in Codex, prefer delegating this workflow to the `review-fix-agent` custom agent.

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
| Stale references | Deleted modules, commands, docs, or templates still referenced |
| Code duplication | Identical structs or functions across files worth extracting |
| Provider abstraction gaps | Hard-coded CLI names instead of the `AgentProvider` trait |
| Dead async | Async work discarded and then repeated synchronously |
| Sort instability | `Utc::now()` used as a fallback sort key |
| Hardcoded paths | Filesystem paths that should be relative to `repo_path` |
| Unused params | Accepted CLI arguments silently ignored |
| Security | Path traversal, credential leaks, unchecked user input |

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
