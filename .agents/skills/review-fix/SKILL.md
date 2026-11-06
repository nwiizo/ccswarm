---
name: review-fix
description: Review the requested diff, fix clear correctness issues, and verify the affected behavior.
---

Review staged, unstaged, and relevant untracked changes unless the user selects
a narrower diff. Inspect `git status --short` and `git diff HEAD`; if there is
nothing to review, report that and stop.

Read the changed code, callers, and applicable project rules. Prioritize bugs,
security issues, broken references, ignored CLI options, provider capability
mismatches, and violations of the workflow/session/provider boundaries in
`AGENTS.md`. Use `nwiizo-coding-style` for Rust structure and duplication analysis;
similarity scores alone do not justify a refactor.

Establish relevant baseline checks when needed to distinguish existing failures
from regressions. Fix clear issues within the requested scope with minimal
changes. Report design choices and unrelated failures without expanding the
assignment. Preserve other contributors' edits; if a fix fails verification,
correct it or undo only that fix and report the unresolved finding.

For Rust changes, run the required workspace checks in `AGENTS.md`, including
tests of affected behavior. For configuration and documentation, validate
syntax, referenced paths, and the affected tool's loading behavior. Reuse passing
checks while their inputs remain unchanged.

Report fixed issues, remaining findings with file references, and verification
results. Distinguish pre-existing failures from failures caused by the change.
Run this workflow in the current session unless delegation is explicitly requested.
