# Advanced Refactoring Plan - Maximum Code Reduction

## Overview

This advanced refactoring plan aims for 60-70% code reduction through aggressive use of Rust's macro system, trait abstractions, and generic programming patterns.

## 1. Macro-Based Message Generation (unified_bus.rs)

### Current Problem
- 3 nearly identical message creation methods
- Each method ~14 lines, total ~42 lines
- Manual implementation for each message type

### Advanced Solution: Declarative Macro System

```rust
// Define all message types in a single macro
macro_rules! define_messages {
    ($(
        $variant:ident {
            $($field:ident: $type:ty),* $(,)?
        }
    )*) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub enum UnifiedMessage {
            $($variant($variant),)*
        }

        $(
            #[derive(Debug, Clone, Serialize, Deserialize)]
            pub struct $variant {
                pub id: String,
                $(pub $field: $type,)*
                pub timestamp: chrono::DateTime<chrono::Utc>,
            }

            impl $variant {
                pub fn new($($field: $type),*) -> UnifiedMessage {
                    UnifiedMessage::$variant(Self {
                        id: Uuid::new_v4().to_string(),
                        $($field,)*
                        timestamp: chrono::Utc::now(),
                    })
                }
            }
        )*
    };
}

// Single declaration replaces all message structs and creation methods
define_messages! {
    SessionMessage {
        session_id: String,
        msg_type: SessionMessageType,
        payload: serde_json::Value,
    }
    TaskMessage {
        task_id: String,
        msg_type: TaskMessageType,
        payload: serde_json::Value,
    }
    EventMessage {
        source: String,
        event_type: EventType,
        payload: serde_json::Value,
    }
    CoordinationMessage {
        sender: String,
        receiver: Option<String>,
        msg_type: MessageType,
        content: serde_json::Value,
    }
    DirectMessage {
        from: String,
        to: String,
        content: String,
        metadata: HashMap<String, String>,
    }
}
```

**Reduction**: 300+ lines → ~50 lines (83% reduction)

## 2. Generic Pattern Engine (proactive_master.rs)

### Current Problem
- Multiple pattern initialization methods with similar structure
- Each method 60-80 lines
- Hardcoded pattern definitions

### Advanced Solution: DSL-Based Pattern Definition

```rust
// Pattern DSL using builder pattern + macros
macro_rules! pattern_dsl {
    ($($name:ident => {
        triggers: [$($trigger:expr),*],
        tasks: [$({
            desc: $desc:expr,
            type: $task_type:ident,
            priority: $priority:ident,
            duration: $duration:expr,
            agent: $agent:expr,
        }),*]
    })*) => {{
        let mut patterns = HashMap::new();
        $(
            patterns.insert(
                stringify!($name).to_string(),
                TaskPattern::builder()
                    .pattern_id(stringify!($name))
                    .triggers(vec![$($trigger.to_string()),*])
                    .tasks(vec![
                        $(TaskTemplate::new()
                            .description($desc)
                            .task_type(TaskType::$task_type)
                            .priority(Priority::$priority)
                            .duration($duration)
                            .agent($agent)
                            .build()),*
                    ])
                    .build()
            );
        )*
        patterns
    }};
}

// Replace all initialization methods with single DSL
impl ProactiveMaster {
    fn initialize_all_patterns() -> HashMap<String, TaskPattern> {
        pattern_dsl! {
            frontend_component => {
                triggers: ["component created"],
                tasks: [{
                    desc: "Write unit tests for {component_name}",
                    type: Testing,
                    priority: High,
                    duration: 30,
                    agent: "QA",
                }, {
                    desc: "Add {component_name} to docs",
                    type: Documentation,
                    priority: Medium,
                    duration: 15,
                    agent: "Frontend",
                }]
            }
            api_endpoint => {
                triggers: ["API endpoint created"],
                tasks: [{
                    desc: "Write integration tests for {endpoint_name}",
                    type: Testing,
                    priority: High,
                    duration: 45,
                    agent: "QA",
                }, {
                    desc: "Update API documentation",
                    type: Documentation,
                    priority: Medium,
                    duration: 20,
                    agent: "Backend",
                }]
            }
            // Add more patterns here
        }
    }
}
```

**Reduction**: 400+ lines → ~80 lines (80% reduction)

## 3. Async State Machine Framework (search_agent.rs)

### Current Problem
- Repetitive async method implementations
- Similar error handling and logging patterns
- Duplicate state transition logic

### Advanced Solution: Declarative Async State Machine

```rust
// Generic async state machine trait
#[async_trait]
trait AsyncStateMachine: Sized {
    type State: Clone + Eq + Hash;
    type Event;
    type Context;
    type Error;

    fn transitions() -> HashMap<(Self::State, Self::Event), Self::State>;
    
    async fn on_enter_state(
        &mut self,
        state: &Self::State,
        ctx: &mut Self::Context,
    ) -> Result<(), Self::Error>;
    
    async fn on_exit_state(
        &mut self,
        state: &Self::State,
        ctx: &mut Self::Context,
    ) -> Result<(), Self::Error>;
}

// Macro to generate state machine implementation
macro_rules! async_state_machine {
    (
        $machine:ident {
            states: [$($state:ident),*],
            events: [$($event:ident),*],
            transitions: [
                $( ($from:ident, $on:ident) => $to:ident ),*
            ],
            handlers: {
                $(
                    $handler_state:ident => $handler:expr
                ),*
            }
        }
    ) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        enum State {
            $($state),*
        }

        #[derive(Debug, Clone)]
        enum Event {
            $($event),*
        }

        impl $machine {
            async fn handle_event(&mut self, event: Event) -> Result<()> {
                let transitions = Self::transitions();
                let new_state = transitions
                    .get(&(self.state.clone(), event))
                    .ok_or_else(|| anyhow!("Invalid transition"))?;
                
                self.transition_to(new_state.clone()).await
            }
            
            async fn transition_to(&mut self, new_state: State) -> Result<()> {
                // Generic transition logic with logging
                info!("Transitioning from {:?} to {:?}", self.state, new_state);
                
                self.on_exit_state(&self.state, &mut self.context).await?;
                self.state = new_state.clone();
                self.on_enter_state(&new_state, &mut self.context).await?;
                
                Ok(())
            }
        }
    };
}

// Use the macro for SearchAgent
async_state_machine! {
    SearchAgent {
        states: [Initializing, Registering, Available, Working, Monitoring],
        events: [Initialize, Register, StartWork, CompleteWork, StartMonitoring],
        transitions: [
            (Initializing, Initialize) => Registering,
            (Registering, Register) => Available,
            (Available, StartWork) => Working,
            (Working, CompleteWork) => Available,
            (Available, StartMonitoring) => Monitoring
        ],
        handlers: {
            Registering => |agent, ctx| agent.register_internal(ctx),
            Working => |agent, ctx| agent.execute_work(ctx),
            Monitoring => |agent, ctx| agent.monitor_internal(ctx)
        }
    }
}
```

**Reduction**: 500+ lines → ~100 lines (80% reduction)

## 4. Generic Async Patterns Framework

### Problem
- Repeated async patterns: retry logic, timeout handling, error propagation
- Similar coordination bus message handling
- Duplicate logging and metrics collection

### Solution: Composable Async Primitives

```rust
// Generic async operation wrapper
#[async_trait]
trait AsyncOperation<T, E> {
    async fn execute(&self) -> Result<T, E>;
}

// Decorator pattern for cross-cutting concerns
struct AsyncOperationBuilder<T, E, O: AsyncOperation<T, E>> {
    operation: O,
    retry_policy: Option<RetryPolicy>,
    timeout: Option<Duration>,
    circuit_breaker: Option<CircuitBreaker>,
    _phantom: PhantomData<(T, E)>,
}

impl<T, E, O: AsyncOperation<T, E>> AsyncOperationBuilder<T, E, O> {
    pub fn with_retry(mut self, policy: RetryPolicy) -> Self {
        self.retry_policy = Some(policy);
        self
    }
    
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }
    
    pub fn with_circuit_breaker(mut self, cb: CircuitBreaker) -> Self {
        self.circuit_breaker = Some(cb);
        self
    }
    
    pub async fn execute(self) -> Result<T, E> {
        let mut op = self.operation;
        
        // Apply timeout
        if let Some(timeout) = self.timeout {
            op = TimeoutDecorator::new(op, timeout);
        }
        
        // Apply retry
        if let Some(retry) = self.retry_policy {
            op = RetryDecorator::new(op, retry);
        }
        
        // Apply circuit breaker
        if let Some(cb) = self.circuit_breaker {
            op = CircuitBreakerDecorator::new(op, cb);
        }
        
        op.execute().await
    }
}

// Macro for common async patterns
macro_rules! async_handler {
    ($name:ident($($arg:ident: $type:ty),*) -> $ret:ty $body:block) => {
        pub async fn $name(&mut self, $($arg: $type),*) -> $ret {
            let op = AsyncOperationBuilder::new(|| async move $body)
                .with_retry(RetryPolicy::exponential_backoff(3, Duration::from_millis(100)))
                .with_timeout(Duration::from_secs(30))
                .with_circuit_breaker(self.circuit_breaker.clone());
            
            match op.execute().await {
                Ok(result) => {
                    metrics::counter!("operation.success", 1, "op" => stringify!($name));
                    result
                },
                Err(e) => {
                    metrics::counter!("operation.failure", 1, "op" => stringify!($name));
                    error!("Operation {} failed: {:?}", stringify!($name), e);
                    Err(e)
                }
            }
        }
    };
}
```

## 5. Unified Error Handling Framework

```rust
// Generic error type with context
#[derive(Debug, thiserror::Error)]
pub enum AppError<T: std::error::Error + 'static> {
    #[error("{context}: {source}")]
    WithContext {
        context: String,
        source: T,
        backtrace: Backtrace,
    },
    
    #[error("Operation failed: {0}")]
    OperationFailed(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Extension trait for easy error context
trait ErrorContext<T> {
    fn context(self, ctx: &str) -> Result<T, AppError<Box<dyn std::error::Error>>>;
}

impl<T, E: std::error::Error + 'static> ErrorContext<T> for Result<T, E> {
    fn context(self, ctx: &str) -> Result<T, AppError<Box<dyn std::error::Error>>> {
        self.map_err(|e| AppError::WithContext {
            context: ctx.to_string(),
            source: Box::new(e),
            backtrace: Backtrace::capture(),
        })
    }
}
```

## Implementation Strategy

### Phase 1: Macro Infrastructure (Week 1)
1. Implement core macro utilities
2. Create code generation templates
3. Set up testing framework for macros

### Phase 2: Pattern Migration (Week 2-3)
1. Migrate message types to macro system
2. Convert pattern initialization to DSL
3. Implement async state machine framework

### Phase 3: Cross-Cutting Concerns (Week 4)
1. Implement async operation decorators
2. Create unified error handling
3. Add instrumentation and metrics

### Phase 4: Full Integration (Week 5-6)
1. Migrate all components to new patterns
2. Remove legacy code
3. Update documentation

## Expected Results

### Code Reduction
- **unified_bus.rs**: 85% reduction (300 → 45 lines)
- **proactive_master.rs**: 80% reduction (400 → 80 lines)
- **search_agent.rs**: 80% reduction (500 → 100 lines)
- **Overall**: 60-70% reduction in repetitive code

### Quality Improvements
- **Type Safety**: Compile-time guarantees through macros
- **Maintainability**: Single source of truth for patterns
- **Performance**: Zero-cost abstractions
- **Testing**: Easier to test generic components

### Architecture Benefits
- **Consistency**: Uniform patterns across codebase
- **Extensibility**: Easy to add new message types/patterns
- **Debugging**: Better error messages with context
- **Monitoring**: Built-in metrics and tracing