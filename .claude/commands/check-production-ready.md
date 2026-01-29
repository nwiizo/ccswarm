# Production Readiness Check

Verifies ccswarm's production quality criteria.

## Check Items

7 quality criteria based on CLAUDE.md:

| # | Item | Criteria | Check Command |
|---|------|----------|---------------|
| 1 | unwrap() elimination | Forbidden in production code | `grep -r "\.unwrap()" crates/ccswarm/src/` |
| 2 | Result/Error handling | Custom error types with thiserror | `grep -r "thiserror" crates/` |
| 3 | Async patterns | tokio runtime, async-trait | `cargo check` |
| 4 | Documentation | rustdoc for public APIs | `cargo doc --workspace` |
| 5 | Clippy clean | No warnings | `cargo clippy --workspace -- -D warnings` |
| 6 | Channel-Based | Prefer Channel over Arc<Mutex> | `grep -r "Arc<Mutex" crates/` |
| 7 | Minimal tests | Around 8-10 tests | `cargo test --workspace 2>&1 | grep "test result"` |

## Execution Method

```bash
# 1. unwrap() check
echo "=== unwrap() count ==="
grep -r "\.unwrap()" crates/ccswarm/src/ --include="*.rs" | grep -v "test" | wc -l

# 2. Error handling verification
echo "=== thiserror usage ==="
grep -r "use thiserror" crates/

# 3. Clippy check
echo "=== Clippy check ==="
cargo clippy --workspace -- -D warnings

# 4. Documentation generation
echo "=== Documentation ==="
cargo doc --workspace --no-deps

# 5. Arc<Mutex> count
echo "=== Arc<Mutex> count ==="
grep -r "Arc<Mutex" crates/ccswarm/src/ | wc -l

# 6. Test count verification
echo "=== Test count ==="
cargo test --workspace 2>&1 | grep "test result"
```

## Output Format

```json
{
  "production_ready": {
    "unwrap_elimination": {
      "status": "OK|NG",
      "count": N,
      "locations": ["problem locations"]
    },
    "error_handling": {
      "status": "OK|NG",
      "thiserror_used": true|false,
      "custom_errors": ["ErrorType1", "ErrorType2"]
    },
    "async_patterns": {
      "status": "OK|NG",
      "tokio_version": "1.x",
      "async_trait_used": true|false
    },
    "documentation": {
      "status": "OK|NG",
      "public_api_coverage": "N%"
    },
    "clippy_clean": {
      "status": "OK|NG",
      "warnings": N,
      "errors": N
    },
    "channel_based": {
      "status": "OK|NG",
      "arc_mutex_count": N,
      "channel_usage": N,
      "ratio": "channels/arc_mutex"
    },
    "minimal_testing": {
      "status": "OK|NG",
      "test_count": N,
      "target_range": "8-10"
    }
  },
  "score": "N/7",
  "overall_status": "PRODUCTION_READY|NEEDS_WORK|CRITICAL"
}
```

## Evaluation Criteria

| Score | Verdict | Description |
|-------|---------|-------------|
| 7/7 | PRODUCTION_READY | Ready to deploy to production |
| 5-6/7 | NEEDS_WORK | Minor improvements needed |
| 0-4/7 | CRITICAL | Significant improvements needed |

## Usage Example

```
subagent_type: "rust-fix-agent"
prompt: "Run production readiness check on ccswarm.
Verify each of the 7 criteria and fix any issues found.
Report results in JSON format."
```

## Related

- `/check-impl` - Basic checks
- `/review-all` - Full review
- `.claude/agents/rust-fix-agent.md` - Rust fix agent
