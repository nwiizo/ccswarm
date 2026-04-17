---
name: check-production-ready
description: Production readiness audit - checks unwrap elimination, error handling, clippy, docs, channel usage, and test count.
user-invocable: true
context: fork
agent: rust-fix-agent
---

When subagents are enabled in Codex, prefer delegating this workflow to the `rust-fix-agent` custom agent.

Verify 7 production quality criteria:

| # | Item | Check |
|---|------|-------|
| 1 | unwrap() elimination | `grep -r "\.unwrap()" crates/ccswarm/src/ --include="*.rs"` (exclude tests) |
| 2 | Error handling | thiserror usage, custom error types |
| 3 | Async patterns | tokio runtime, proper async usage |
| 4 | Documentation | `cargo doc --workspace --no-deps` |
| 5 | Clippy clean | `cargo clippy --workspace -- -D warnings` |
| 6 | Channel-Based | Channel vs Arc<Mutex> ratio |
| 7 | Minimal testing | ~8-10 focused tests |

Score: 7/7 = PRODUCTION_READY, 5-6/7 = NEEDS_WORK, 0-4/7 = CRITICAL.

Fix any issues found, then report results as JSON.
