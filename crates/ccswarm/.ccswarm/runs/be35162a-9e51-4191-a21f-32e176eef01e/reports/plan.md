## Analysis Complete

Here is my analysis and implementation plan for the **invalid command handling** behavior in ccswarm.

---

# Task Analysis: Invalid Command Handling in ccswarm

## Problem Statement

When a user types `ccswarm invalid-command-that-does-not-exist`, the current behavior has a **critical UX bug**: the input is silently treated as a "direct task" (pipeline execution) instead of showing a helpful error message with command suggestions.

## Current Behavior Flow

```
User: ccswarm invalid-command-that-does-not-exist
  ↓
main.rs:10-11 → is_known_subcommand("invalid-command-that-does-not-exist") → false
  ↓
args[1] doesn't start with '-' → is_direct_task = true
  ↓
run_direct_task("invalid-command-that-does-not-exist")
  ↓
Attempts to run a pipeline with nonsensical task → fails (or worse, succeeds at wasting resources)
```

**Expected behavior**: Show an error with "did you mean X?" suggestions.

## Root Cause

The `is_known_subcommand()` function in `main.rs:34-67` acts as a negative filter: anything NOT in the hardcoded list is assumed to be a "direct task" (natural language input). This creates two problems:

1. **Typos become tasks**: `ccswarm initt` → treated as task "initt" instead of suggesting "init"
2. **Maintenance burden**: The hardcoded 28-entry list in `is_known_subcommand()` must stay in sync with the `Commands` enum in `cli/mod.rs` — any new command added to `Commands` must also be added here

## Impact Scope

| File | Role | Change Required |
|------|------|-----------------|
| `crates/ccswarm/src/main.rs` | Entry point, direct-task detection | **Primary** — improve heuristic |
| `crates/ccswarm/src/cli/mod.rs` | `Commands` enum definition | **Read-only** — source of truth for valid subcommands |
| `crates/ccswarm/src/cli/command_registry.rs` | Command dispatch (unreachable for unknown) | No change |
| `crates/ccswarm/tests/e2e_cli_test.rs:393` | Test for invalid command | **Update** — verify improved UX |
| `crates/ccswarm/tests/cli_unit_tests.rs:164` | Unit test for parse failure | **Update** — verify improved UX |
| `Cargo.toml` | Dependencies | **Add** `strsim` crate (if fuzzy matching) |

## Design Options

### Option A: Heuristic Enhancement (Minimal Change)
Add a simple check: if the input looks like a subcommand (single hyphenated word, no spaces, short), validate it against known commands before falling through to direct-task mode.

**Pros**: No new dependencies, simple  
**Cons**: Fragile heuristic, still needs hardcoded list

### Option B: Fuzzy Match + Smart Detection (Recommended)
Use `strsim` crate for Levenshtein distance. If input is a single word with edit-distance ≤ 3 from any known subcommand, show a suggestion instead of treating as direct task.

**Pros**: Great UX, catches typos  
**Cons**: New dependency (~10KB, widely used)

### Option C: Derive Subcommand List from Clap
Use `Cli::command().get_subcommands()` to extract the list at runtime, eliminating the hardcoded `is_known_subcommand()` function entirely.

**Pros**: Always in sync, no maintenance  
**Cons**: Requires clap `Command` construction before parsing

## Recommended Implementation Plan

**Combine Options B + C** for maximum reliability and UX quality.

### Step 1: Eliminate hardcoded subcommand list
Replace `is_known_subcommand()` with a function that derives valid subcommands from clap's `Command` metadata:

```rust
fn get_known_subcommands() -> Vec<String> {
    Cli::command()
        .get_subcommands()
        .map(|cmd| cmd.get_name().to_string())
        .collect()
}

fn is_known_subcommand(arg: &str) -> bool {
    get_known_subcommands().iter().any(|cmd| cmd == arg)
}
```

**File**: `crates/ccswarm/src/main.rs` — replace lines 34-67

### Step 2: Add fuzzy command suggestion
Add `strsim` dependency and implement typo detection:

```rust
fn suggest_subcommand(input: &str) -> Option<String> {
    let commands = get_known_subcommands();
    commands
        .iter()
        .filter_map(|cmd| {
            let dist = strsim::levenshtein(input, cmd);
            if dist <= 3 { Some((cmd.clone(), dist)) } else { None }
        })
        .min_by_key(|(_, dist)| *dist)
        .map(|(cmd, _)| cmd)
}
```

**File**: `crates/ccswarm/src/main.rs`  
**Dependency**: Add `strsim = "0.11"` to `crates/ccswarm/Cargo.toml`

### Step 3: Improve direct-task heuristic
Before entering direct-task mode, check if the input might be a mistyped command:

```rust
if is_direct_task {
    // Check for potential command typo (single word, no spaces, looks like a command)
    let first_arg = &args[1];
    if !first_arg.contains(' ') && first_arg.len() < 30 {
        if let Some(suggestion) = suggest_subcommand(first_arg) {
            eprintln!(
                "{} Unknown command '{}'. Did you mean '{}'?\n",
                "Error:".red().bold(), first_arg, suggestion.green()
            );
            eprintln!("  {} ccswarm {}", "Try:".cyan().bold(), suggestion);
            eprintln!("  {} ccswarm --help", "Or:".cyan().bold());
            std::process::exit(2);
        }
    }
    // ... proceed with direct task mode
}
```

**File**: `crates/ccswarm/src/main.rs` — modify lines 13-22

### Step 4: Update tests

**`tests/e2e_cli_test.rs`**: The existing test `test_invalid_command` already expects failure — verify it still passes. Add a new test for typo suggestion:

```rust
#[test]
fn test_typo_command_suggests() {
    let output = run_ccswarm(&["initt"], None);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Did you mean") || stderr.contains("init"));
}
```

**`tests/cli_unit_tests.rs`**: No changes needed (tests clap parsing directly).

### Step 5: Verify quality

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test
```

## Files to Modify (Summary)

| # | File | Action |
|---|------|--------|
| 1 | `crates/ccswarm/Cargo.toml` | Add `strsim = "0.11"` |
| 2 | `crates/ccswarm/src/main.rs` | Replace `is_known_subcommand()`, add `suggest_subcommand()`, modify direct-task guard |
| 3 | `crates/ccswarm/tests/e2e_cli_test.rs` | Add typo suggestion test |

## Risk Assessment

- **Low risk**: Changes are isolated to the entry point (`main.rs`), no core logic affected
- **Backward compatible**: Direct task mode still works for multi-word inputs and genuine natural language
- **No breaking changes**: Existing subcommands and flags remain identical
- **Dependency risk**: `strsim` is a well-maintained, zero-dependency crate (~700k downloads/month)

## Acceptance Criteria

1. `ccswarm initt` → shows "Did you mean 'init'?" and exits with code 2
2. `ccswarm "Create a Snake game"` → still enters direct-task mode (multi-word)
3. `ccswarm init --name Foo` → works unchanged
4. `ccswarm totally-random-gibberish` → enters direct-task mode (no close match)
5. `is_known_subcommand()` no longer hardcoded — automatically syncs with `Commands` enum
6. All existing tests pass; new typo-suggestion test added

---

[STEP:0]
