---
name: rust-agent-specialist
description: Apply Rust-native patterns (Type-State, Channel-Based, Actor Model) to ccswarm development.
user-invocable: true
---

Guidance for implementing Rust-idiomatic patterns in ccswarm.

## Core Patterns

### Type-State (compile-time state validation)
```rust
pub struct Agent<State> { inner: AgentInner, _state: PhantomData<State> }
impl Agent<Uninitialized> { fn initialize(self) -> Agent<Ready> { ... } }
impl Agent<Ready> { fn execute(&self, task: Task) -> Result<Output> { ... } }
```

### Channel-Based (replace Arc<Mutex>)
```rust
let (tx, rx) = tokio::sync::mpsc::channel(100);
// No shared mutable state between agents
```

### Actor Model (independent actors)
```rust
struct AgentActor { mailbox: mpsc::Receiver<Message>, state: AgentState }
impl AgentActor {
    async fn run(mut self) {
        while let Some(msg) = self.mailbox.recv().await { self.handle(msg).await; }
    }
}
```

## Identify & Apply

```bash
grep -r "Arc<Mutex" crates/ccswarm/src/ --include="*.rs"    # Convert to channels
grep -r "enum.*State\|State::" crates/ccswarm/src/          # Convert to type-state
```

Verify: `cargo fmt && cargo clippy -- -D warnings && cargo test`
