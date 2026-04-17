---
name: review-architecture
description: Architecture pattern compliance review. Evaluates Type-State, Channel-Based, Iterator Pipelines, Actor Model, and Minimal Testing patterns.
user-invocable: true
context: fork
agent: architecture-reviewer
---

When subagents are enabled in Codex, prefer delegating this workflow to the `architecture-reviewer` custom agent.

Review ccswarm architecture pattern compliance based on CLAUDE.md and docs/ARCHITECTURE.md.

Evaluate these 5 patterns:

| Pattern | Search For | Avoid |
|---------|-----------|-------|
| Type-State | `PhantomData`, `impl<S: State>` | Runtime state enums |
| Channel-Based | `mpsc::channel`, `broadcast::channel` | `Arc<Mutex<...>>` |
| Iterator Pipelines | `.iter().filter().map().collect()` | Manual for loops |
| Actor Model | `Receiver<Message>`, `recv().await` | Shared mutable state |
| Minimal Testing | ~8-10 focused tests | 300+ fragile tests |

Output a JSON report with per-pattern `{ status, usage_count, score }`, `overall_score`, and `recommendations`.
