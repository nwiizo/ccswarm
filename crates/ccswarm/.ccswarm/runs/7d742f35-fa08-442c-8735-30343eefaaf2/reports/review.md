I now have a thorough understanding of all the changed files. Here is my complete review:

---

# Code Review: Session Management + Events Redesign

## Summary

This changeset adds a `SessionInfo` type to the events module for parsing NDJSON event logs, introduces a new `ccswarm session` CLI subcommand tree (`list`, `view`, `create`, `pause`, `resume`, `attach`, `detach`, `kill`), renames the legacy `SessionAction` → `SessionCommandAction` for backward compat, and applies `cargo fmt` reformatting to several files.

**Verdict: Fixes needed** — two substantive issues and several quality improvements.

---

## Critical Issues

### 1. **O(n²) agent deduplication in `from_events`** — `events/mod.rs:439-442`

```rust
if !agents.contains(&agent.to_owned()) {
    agents.push(agent.to_owned());
}
```

`Vec::contains` with `to_owned()` allocates a new `String` on every check, and the linear scan makes this O(n·a) where `a` is the number of distinct agents. While `a` is typically small (4 agents), this allocates a throwaway `String` per event line even when the agent is already present. Use a `HashSet<String>` for O(1) lookups and zero spurious allocations:

```rust
let mut agents_set: HashSet<String> = HashSet::new();
// ...
if let Some(agent) = event.get("agent").and_then(|v| v.as_str()) {
    agents_set.insert(agent.to_owned());
}
// then: agents_used: agents_set.into_iter().collect()
```

**Severity**: Medium (correctness is fine, but wasteful allocation per NDJSON line in event logs that could grow large).

### 2. **`from_summary` ignores `task`, `last_movement`, `movements_completed`** — `events/mod.rs:341-353`

```rust
Self {
    // ...
    task: None,           // ← always None
    last_movement: None,  // ← always None
    movements_completed: 0,  // ← always 0
    // ...
}
```

`from_summary()` unconditionally sets these to `None`/`0`, even though the summary JSON may contain them. The caller in `handlers/session.rs:62-78` supplements these from events, but if `events.ndjson` doesn't exist or fails to read, the data is silently lost. The summary JSON should be checked for these fields too.

**Severity**: Medium (data loss in edge case — summary exists but events.ndjson is missing or unreadable).

---

## Significant Issues

### 3. **Silently discarded errors in `run_direct_task`** — `main.rs:126-149`

Six `let _ = tokio::process::Command::new("git")...` calls silently ignore failures. If `git init` fails (e.g., permission denied), the subsequent `git add -A` and `git commit` will also fail, and the user gets no feedback. The function continues to run the pipeline against a non-git directory.

At minimum, the critical `git init` result should be checked:
```rust
let output = tokio::process::Command::new("git").arg("init").output().await?;
if !output.status.success() {
    anyhow::bail!("git init failed: {}", String::from_utf8_lossy(&output.stderr));
}
```

**Severity**: Medium-High (user-facing scaffolding silently swallows critical failures).

### 4. **`run_direct_task` writes `package.json` unconditionally** — `main.rs:139`

```rust
tokio::fs::write("package.json", "{}\n").await?;
```

When `is_new_project` is true, the function writes an empty `package.json` to the *current working directory*, not necessarily the `repo` path. Also, every "new project" is assumed to be JavaScript (creates `package.json`, `public/`, `e2e/`). If the task is "Build a Rust CLI tool", this scaffolding is wrong. Consider making the scaffold language-agnostic or at least not creating framework-specific directories.

**Severity**: Medium (incorrect scaffolding for non-JS projects).

### 5. **Duplicate `SessionAction` vs `SessionCommandAction` types** — `cli/mod.rs:1033` vs `cli/commands/session.rs:24`

Two parallel enum hierarchies exist:
- `cli/mod.rs` → `SessionAction` (8 variants: List, View, Create, Pause, Resume, Attach, Detach, Kill) — **actively used**
- `cli/commands/session.rs` → `SessionCommandAction` (3 variants: List, Stats, Create) — **legacy stub, dead code**

The legacy stub's `Command::execute()` implementation just logs via `tracing::info!` and does nothing. It should be removed to avoid confusion, or clearly documented why it must remain.

**Severity**: Low-Medium (dead code, maintenance burden, confusing for contributors).

---

## Minor Issues / Nits

### 6. **`extract_piece_name` doesn't handle mismatched quotes** — `events/mod.rs:483-490`

If message contains a single quote but no closing quote (e.g., `"Starting piece 'broken"`), it returns the full message. This is fine, but consider using `split_once` for clarity:

```rust
fn extract_piece_name(message: &str) -> String {
    message.split('\'').nth(1).unwrap_or(message).to_owned()
}
```

### 7. **`format_duration` negative duration** — `events/mod.rs:496`

`.max(0)` handles negative durations, but if `ended_at < started_at` (clock skew), the duration silently becomes `"0s"`. Consider logging a warning when this occurs.

### 8. **Handler boilerplate duplication** — `handlers/session.rs:412-539`

`session_pause`, `session_resume`, `session_attach`, `session_detach` are nearly identical methods that all say "managed by the pipeline engine." These could be unified into a single method:

```rust
async fn session_lifecycle_info(&self, action: &str, session_id: &str) -> Result<()> { ... }
```

### 9. **`use super::super::*`** — `handlers/session.rs:1`

Wildcard glob import from grandparent module is fragile and makes it unclear what symbols are being used. Prefer explicit imports:

```rust
use crate::cli::{CliRunner, SessionAction};
use crate::error::Result;
use colored::Colorize;
use std::path::PathBuf;
```

### 10. **Hardcoded separator width** — `handlers/session.rs:170`

```rust
println!("{}", "─".repeat(110).bright_black());
```

The 110-char width may not match actual column widths on narrow terminals. Consider computing it from the format string or using a constant.

### 11. **`is_known_subcommand` manual maintenance** — `main.rs:34-67`

This hardcoded list must be manually updated every time a new command is added. It's a source of bugs. Consider deriving it from clap's `Commands` enum or using `Cli::try_parse_from` with a fallback approach.

---

## Test Coverage Assessment

**Good coverage for `SessionInfo`** — 13 new tests in `events/mod.rs` covering:
- ✅ `from_events`: completed, failed, running, empty, with agents, provider_error, error_level, malformed lines
- ✅ `from_summary`: normal, with failures
- ✅ `extract_piece_name`, `format_duration` edge cases

**Good CLI parse tests** — 10 new tests in `cli_unit_tests.rs` covering all `SessionAction` variants.

**Missing test coverage:**
- ❌ `handlers/session.rs` → `collect_sessions()` has no integration test (complex file-system logic with 3 code paths)
- ❌ `session_view()` → no test for the event display rendering
- ❌ `session_list()` JSON output format → no test verifying the JSON structure

---

## Security Review

- ✅ No hardcoded secrets
- ✅ `session_create` validates agent type against allowlist (`handlers/session.rs:364-371`)
- ✅ File paths use `self.repo_path.join()` rather than user-controlled absolute paths
- ⚠️ `session_view` takes a raw `id` string and joins it directly to `.ccswarm/runs/` (line 229). If `id` contains `../`, this could escape the runs directory. While clap prevents shell injection, a path traversal check would be prudent:

```rust
if id.contains("..") || id.contains('/') {
    anyhow::bail!("Invalid session ID: {}", id);
}
```

---

## Architecture Assessment

- **Separation of concerns**: Clean separation — `SessionInfo` data model in `events/`, CLI definitions in `cli/mod.rs`, handler logic in `handlers/session.rs`. Follows existing project patterns.
- **Dependency direction**: Correct — handler depends on events module, not vice versa.
- **Module organization**: The `handlers/session.rs` file uses `super::super::*` which is a weak coupling pattern. Otherwise clean.
- **The events module** is growing (now 897 lines). Consider splitting `SessionInfo` into its own file (`events/session_info.rs`) when it grows further.

---

[STEP:1] - fixes_needed
