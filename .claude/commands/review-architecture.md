# Architecture Review

Reviews ccswarm's architecture pattern compliance in detail.

## Execution Content

Based on CLAUDE.md and docs/ARCHITECTURE.md, verify compliance with the following patterns:

1. **Type-State Pattern** - Compile-time state verification
2. **Channel-Based Orchestration** - Message passing
3. **Iterator Pipelines** - Zero-cost abstractions
4. **Actor Model** - Lock-free design
5. **Minimal Testing** - Minimum necessary tests

## Check Items

### 1. Type-State Pattern

```rust
// Expected: Verify state transitions at compile time
struct Agent<S: State> {
    state: PhantomData<S>,
    // ...
}

impl Agent<Uninitialized> {
    fn initialize(self) -> Agent<Ready> { ... }
}

impl Agent<Ready> {
    fn execute(self, task: Task) -> Agent<Running> { ... }
}
```

Search patterns:
```bash
grep -r "PhantomData" crates/
grep -r "impl.*<.*State>" crates/
```

### 2. Channel-Based Orchestration

```rust
// Expected: Prefer Channel over Arc<Mutex>
let (tx, rx) = tokio::sync::mpsc::channel(100);

// Avoid:
// let shared = Arc::new(Mutex::new(state));
```

Search patterns:
```bash
grep -r "Arc<Mutex" crates/ | wc -l  # Lower is better
grep -r "mpsc::channel\|broadcast::channel" crates/
```

### 3. Iterator Pipelines

```rust
// Expected: Zero-cost abstractions with iterator chains
let results: Vec<_> = items
    .iter()
    .filter(|x| x.active)
    .map(|x| process(x))
    .collect();

// Avoid:
// for item in items { if item.active { ... } }
```

### 4. Actor Model

```rust
// Expected: Each agent as an independent actor
struct AgentActor {
    receiver: mpsc::Receiver<Message>,
}

impl AgentActor {
    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle(msg).await;
        }
    }
}
```

### 5. Minimal Testing

| Criteria | Expected Value |
|----------|----------------|
| Total tests | Around 8-10 |
| Integration tests | Core functionality only |
| Unit tests | Complex logic only |

## Output Format

```json
{
  "patterns": {
    "type_state": {
      "status": "OK|PARTIAL|NG",
      "usage_count": N,
      "examples": ["Agent<Ready>", "Session<Connected>"],
      "missing": ["recommended application areas"]
    },
    "channel_based": {
      "status": "OK|PARTIAL|NG",
      "channel_count": N,
      "arc_mutex_count": N,
      "refactor_candidates": ["candidates for channel conversion"]
    },
    "iterator_pipelines": {
      "status": "OK|PARTIAL|NG",
      "iterator_usage": N,
      "loop_usage": N,
      "refactor_candidates": ["candidates for iterator conversion"]
    },
    "actor_model": {
      "status": "OK|PARTIAL|NG",
      "actor_count": N,
      "message_types": ["TaskMessage", "StatusMessage"]
    },
    "minimal_testing": {
      "status": "OK|PARTIAL|NG",
      "test_count": N,
      "integration_tests": N,
      "unit_tests": N,
      "recommendation": "test addition/removal recommendation"
    }
  },
  "score": "N/5",
  "recommendations": ["priority action items"]
}
```

## Usage Example

```
subagent_type: "Explore"
prompt: "Review ccswarm's architecture pattern compliance.
1. Type-State Pattern usage
2. Channel-Based vs Arc<Mutex> ratio
3. Iterator Pipelines utilization
4. Actor Model implementation
5. Test count and quality
Create a report in JSON format."
```

## Related

- `/review-all` - Full review
- `CLAUDE.md` - Architecture guidelines
- `docs/ARCHITECTURE.md` - Detailed architecture
