# CCSwarm Architecture Optimization Plan

## Executive Summary
This document outlines the architectural optimization decisions for ccswarm to improve maintainability, performance, and scalability.

## Current State Analysis

### Identified Issues
1. **Code Organization**: Large monolithic modules (cli/mod.rs with 5,861 lines)
2. **Concurrency**: 220+ instances of Arc<Mutex>/RwLock causing potential contention
3. **Dependencies**: 247 trait implementations scattered across 85 files
4. **Coupling**: Tight coupling between layers making testing difficult
5. **Performance**: Synchronous blocking in async contexts

## Optimization Decisions

### 1. Module Decomposition Strategy

#### Decision: Split Large Modules
Transform monolithic modules into focused, single-responsibility components.

**Implementation Plan:**
```
Before:
cli/mod.rs (5,861 lines) →

After:
cli/
├── commands/           # Individual command modules
│   ├── agent.rs       # Agent management commands
│   ├── init.rs        # Project initialization
│   ├── session.rs     # Session management
│   ├── task.rs        # Task operations
│   └── sangha.rs      # Sangha voting system
├── handlers/          # Command execution logic
├── parser.rs          # Argument parsing
├── router.rs          # Command routing
└── mod.rs            # Thin facade (< 100 lines)
```

**Benefits:**
- Improved testability (unit test per command)
- Faster compilation (parallel module compilation)
- Better code navigation
- Reduced merge conflicts

### 2. Concurrency Model Optimization

#### Decision: Actor Model Migration
Replace shared state concurrency with message-passing actors.

**Current Problem:**
```rust
// Current: Shared state with locks
struct TaskQueue {
    tasks: Arc<Mutex<Vec<Task>>>,
    agents: Arc<RwLock<HashMap<String, Agent>>>,
}
```

**Optimized Solution:**
```rust
// Actor-based architecture
use tokio::sync::mpsc;

pub struct TaskQueueActor {
    receiver: mpsc::Receiver<TaskCommand>,
    state: TaskQueueState,
}

pub enum TaskCommand {
    AddTask(Task, oneshot::Sender<Result<()>>),
    AssignAgent { task_id: String, agent_id: String },
    GetStatus(oneshot::Sender<QueueStatus>),
}

impl TaskQueueActor {
    pub async fn run(mut self) {
        while let Some(cmd) = self.receiver.recv().await {
            match cmd {
                TaskCommand::AddTask(task, reply) => {
                    let result = self.state.add_task(task);
                    let _ = reply.send(result);
                }
                // ... other commands
            }
        }
    }
}
```

**Benefits:**
- No lock contention
- Predictable performance
- Better error isolation
- Easier to reason about state changes

### 3. Dependency Injection Architecture

#### Decision: Implement Service Container Pattern
Centralize dependency management with a DI container.

**Implementation:**
```rust
// Service container for dependency injection
pub struct ServiceContainer {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServiceContainer {
    pub fn register<T: 'static + Send + Sync>(&mut self, service: T) {
        self.services.insert(TypeId::of::<T>(), Box::new(service));
    }

    pub fn resolve<T: 'static>(&self) -> Option<&T> {
        self.services
            .get(&TypeId::of::<T>())
            .and_then(|s| s.downcast_ref())
    }
}

// Application context with all dependencies
pub struct AppContext {
    container: ServiceContainer,
}

impl AppContext {
    pub fn new() -> Self {
        let mut container = ServiceContainer::new();

        // Register services
        container.register(OrchestratorService::new());
        container.register(AgentFactory::new());
        container.register(SessionManager::new());
        container.register(EventBus::new());

        Self { container }
    }
}
```

### 4. Rust Best Practices Architecture

#### Decision: Embrace Rust-Native Patterns (No Layered Architecture)

Based on user feedback and Rust philosophy, we've eliminated the layered architecture in favor of Rust-native patterns that provide better performance and maintainability.

**Implemented Patterns:**

1. **Type-State Pattern for Agents**
   - Compile-time state machine validation
   - Zero runtime cost using PhantomData
   - Prevents invalid state transitions

2. **Channel-Based Orchestration**
   - Message-passing concurrency without locks
   - No shared state (follows Rust's ownership model)
   - Predictable performance without contention

3. **Iterator-Based Task Processing**
   - Zero-cost abstractions with iterator chains
   - Lazy evaluation for efficiency
   - Composable transformations

4. **Command Pattern with Traits**
   - Type-safe command execution
   - Compile-time validation
   - No runtime dispatch overhead

### 5. Event-Driven Architecture

#### Decision: Implement Event Bus System
Decouple components through event-driven communication.

**Implementation:**
```rust
// Core event system
#[derive(Debug, Clone)]
pub enum DomainEvent {
    TaskCreated { id: String, description: String },
    AgentAssigned { task_id: String, agent_id: String },
    TaskCompleted { id: String, result: TaskResult },
    QualityReviewRequested { task_id: String },
    SanghaVoteInitiated { proposal_id: String },
}

pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<TypeId, Vec<EventHandler>>>>,
    event_queue: mpsc::Sender<DomainEvent>,
}

impl EventBus {
    pub async fn publish(&self, event: DomainEvent) {
        self.event_queue.send(event).await.ok();
    }

    pub fn subscribe<E: 'static>(&self, handler: impl Fn(E) + Send + Sync + 'static) {
        // Register handler for event type
    }
}
```

### 6. Performance Optimizations

#### Decision: Implement Resource Pooling
Reduce allocation overhead with object pooling.

**Implementation:**
```rust
// Session pool for reuse
pub struct SessionPool {
    available: Vec<Session>,
    in_use: HashMap<String, Session>,
    max_size: usize,
}

// Connection pool for AI providers
pub struct ConnectionPool {
    connections: deadpool::managed::Pool<AIConnection>,
}
```

### 7. Testing Strategy Optimization

#### Decision: Implement Test Pyramid
- **Unit Tests**: 70% (fast, isolated)
- **Integration Tests**: 20% (component interaction)
- **E2E Tests**: 10% (full workflow validation)

**Test Infrastructure:**
```rust
// Test fixtures and builders
pub struct TestContext {
    container: ServiceContainer,
    event_bus: EventBus,
}

impl TestContext {
    pub fn builder() -> TestContextBuilder {
        TestContextBuilder::new()
    }
}

// Property-based testing for critical paths
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn task_queue_maintains_order(tasks in prop::collection::vec(task_strategy(), 0..100)) {
            // Test invariants
        }
    }
}
```

## Implementation Timeline

### Phase 1: Foundation (Week 1-2)
- [ ] Set up module decomposition structure
- [ ] Implement service container
- [ ] Create event bus system

### Phase 2: Core Refactoring (Week 3-4)
- [ ] Migrate CLI to modular structure
- [ ] Convert TaskQueue to actor model
- [ ] Implement dependency injection

### Phase 3: Infrastructure (Week 5-6)
- [ ] Implement session pooling
- [ ] Add connection pooling
- [ ] Create test infrastructure

### Phase 4: Optimization (Week 7-8)
- [ ] Performance profiling
- [ ] Fine-tuning and benchmarking
- [ ] Documentation updates

## Metrics for Success

### Performance Metrics
- **Response Time**: < 100ms for command execution
- **Throughput**: Handle 1000+ concurrent agents
- **Memory Usage**: < 500MB for typical workload
- **CPU Usage**: < 50% average utilization

### Code Quality Metrics
- **Module Size**: No module > 500 lines
- **Cyclomatic Complexity**: < 10 per function
- **Test Coverage**: > 85% for critical paths
- **Compilation Time**: < 30 seconds for incremental builds

### Maintainability Metrics
- **Code Duplication**: < 5% (measured by similarity-rs)
- **Coupling**: Low coupling between modules
- **Documentation**: 100% public API documented
- **PR Review Time**: < 2 hours average

## Risk Mitigation

### Identified Risks
1. **Breaking Changes**: Mitigate with comprehensive tests
2. **Performance Regression**: Continuous benchmarking
3. **Migration Complexity**: Incremental refactoring
4. **Team Adoption**: Clear documentation and examples

### Rollback Strategy
- Git worktree for experimental changes
- Feature flags for gradual rollout
- Parallel implementation during transition

## Conclusion

These optimization decisions will transform ccswarm into a more maintainable, performant, and scalable system. The actor model will eliminate concurrency issues, dependency injection will improve testability, and event-driven architecture will enable better extensibility.

The implementation should be done incrementally, with continuous validation through testing and benchmarking to ensure no regression in functionality or performance.