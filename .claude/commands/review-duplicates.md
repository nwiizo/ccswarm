# Duplicate Code Detection

Uses similarity-rs to detect semantically similar code in the codebase and identify refactoring candidates.

## Execution Method

```bash
# Basic execution (analyze crates/ directory)
similarity-rs crates/

# Specify similarity threshold (default: 0.85)
similarity-rs crates/ --threshold 0.80

# Include code in output
similarity-rs crates/ --print

# Specify minimum lines (default: 3 lines)
similarity-rs crates/ --min-lines 5
```

## Options

| Option | Description | Default |
|--------|-------------|---------|
| `--threshold` | Similarity threshold (0.0-1.0) | 0.85 |
| `--print` | Include code in output | false |
| `--min-lines` | Minimum lines | 3 |
| `--min-tokens` | Minimum tokens | 30 |
| `--extensions` | Target file extensions | rs |

## Detection Patterns

### Refactoring Priority

| Priority | Condition | Action |
|----------|-----------|--------|
| High | 95%+ similarity, 10+ lines | Extract to common function |
| Medium | 90-95% similarity, 5+ lines | Consider trait or macro conversion |
| Low | 85-90% similarity | Check structural similarity, extract if needed |

### Acceptable Duplication

The following may be detected as duplicates but often don't need refactoring:

1. **Test code** - Prioritize test independence
2. **Configuration structs** - Similar configuration patterns
3. **Error handling** - Context-dependent error processing
4. **ClaudeCodeConfig initialization** - Explicit field specification is preferred

## Output Format

```json
{
  "duplicate_analysis": {
    "total_functions": N,
    "similar_pairs": N,
    "high_priority": [
      {
        "file1": "path1",
        "file2": "path2",
        "lines1": "L10-L30",
        "lines2": "L50-L70",
        "similarity": "97%",
        "action": "Extract to common function"
      }
    ],
    "medium_priority": [],
    "low_priority": [],
    "ignored": [
      {
        "reason": "Test code",
        "count": N
      }
    ]
  }
}
```

## ccswarm-Specific Refactoring Strategies

### 1. Integration with Channel-Based Pattern

```rust
// Before: Shared state using Arc<Mutex>
let shared_state = Arc::new(Mutex::new(State::new()));

// After: Integrate with Channel-Based
let (tx, rx) = tokio::sync::mpsc::channel(100);
```

### 2. Using Type-State Pattern

```rust
// Before: Runtime state check
if self.state == State::Connected { ... }

// After: Compile-time verification
impl<S: ConnectionState> Agent<S> {
    fn send(self: Agent<Connected>) -> Result<Response> { ... }
}
```

### 3. Integration with Iterator Pipelines

```rust
// Before: Same filtering in multiple places
let items: Vec<_> = items.iter().filter(|x| x.active).collect();

// After: Common iterator adapter
trait ActiveFilter {
    fn active_only(self) -> impl Iterator<Item = Self::Item>;
}
```

## Notes

### Cases NOT to Refactor

1. **Reduced readability** - Abstraction makes code harder to understand
2. **Excessive DRY** - Allow duplication less than 3 times (YAGNI principle)
3. **Different change reasons** - May change separately in the future
4. **Test independence** - Allow duplication in test code

### Cases TO Refactor

1. **Bug fix duplication** - Need to apply same fix to multiple places
2. **Business logic** - Same rules scattered across multiple places
3. **Provider implementation** - Common ProviderExecutor pattern

## Usage Example

```
subagent_type: "code-refactor-agent"
prompt: "Run similarity-rs crates/ to detect duplicate code.
Analyze the results and report the following:
1. High priority refactoring candidates (95%+ similarity)
2. Medium priority candidates (90-95% similarity)
3. Duplicates to ignore (test code, etc.)
4. Refactoring proposals based on ccswarm patterns"
```

## Related

- `/review-all` - Full review (includes duplicate detection)
- `.claude/agents/code-refactor-agent.md` - Refactoring agent
