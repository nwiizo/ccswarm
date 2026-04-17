---
name: review-fix
description: Code review with automatic fix. Detects bugs, stale references, code duplication, provider abstraction gaps, and sort instability — then fixes and verifies.
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

If the baseline already fails, report the failures and stop — do not mask pre-existing breakage.

### Phase 3: Review

Inspect the diff and new untracked files. Check for:

| Category | What to look for |
|----------|-----------------|
| **Stale references** | Deleted modules/commands still referenced in routing, docs, templates |
| **Code duplication** | Identical structs/functions across multiple files (> 10 lines, > 90% similarity) |
| **Provider abstraction gaps** | Hard-coded CLI names (`claude`, `codex`, `gh copilot`) instead of `AgentProvider` trait |
| **Dead async** | Async operations whose results are discarded (`let _ = tokio::...await.ok()` then re-done synchronously) |
| **Sort instability** | `Utc::now()` used as fallback sort key (non-deterministic) |
| **Hardcoded paths** | Filesystem paths that should be relative to `repo_path` |
| **Unused TODO params** | CLI arguments accepted but silently ignored (`_param: T`) |
| **Security** | Path traversal in user-supplied IDs, credential leaks |

Classify each finding:

- 🔴 **Fix**: Clear bug or correctness issue — fix it now.
- 🟡 **Warn**: Design concern or missing feature — report but do not fix unless trivial.
- 🟢 **Note**: Style or informational — report only.

### Phase 4: Fix

For each 🔴 finding:

1. Make the minimal, surgical edit.
2. Do **not** change unrelated code.
3. After all fixes, run:

```bash
cargo clippy --workspace -- -D warnings
cargo test --workspace --no-run
```

If clippy or test compilation fails, revert the offending edit and report it as 🟡 instead.

### Phase 5: Report

Output a structured summary:

```
## Review + Fix Report

### Fixed (🔴)
- <file>:<line> — <description>

### Warnings (🟡)
- <file>:<line> — <description>

### Notes (🟢)
- <description>

### Verification
- clippy: PASS / FAIL
- test compile: PASS / FAIL
- files modified: N
```
