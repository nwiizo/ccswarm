---
name: review-all
description: Full integrated review covering design compliance, code quality, duplicate detection, and architecture patterns. Run after significant changes.
user-invocable: true
context: fork
agent: all-reviewer
---

When subagents are enabled in Codex, prefer delegating this workflow to the `all-reviewer` custom agent.

Run a full review on the ccswarm codebase:

1. **Design Compliance** - Verify consistency with CLAUDE.md and docs/ARCHITECTURE.md
2. **Code Quality** - Check Rust best practices (clippy, unwrap elimination, error handling)
3. **Duplicate Detection** - Run `similarity-rs crates/` for refactoring candidates
4. **Architecture Patterns** - Evaluate Type-State, Channel-Based, Iterator, Actor Model, Minimal Testing

Output a JSON report with `compliance`, `code_quality`, `architecture_patterns`, and `summary` sections.
