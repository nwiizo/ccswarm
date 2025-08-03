# Semantic Code Duplication Refactoring Plan

## Executive Summary

Similarity analysis detected significant code duplication across the ccswarm codebase, with similarities ranging from 85% to 100%. This document outlines a comprehensive refactoring strategy to eliminate duplication through design patterns and architectural improvements.

## Analysis Results

### Critical Duplication Areas (>95% similarity)

1. **ai-session/unified_bus.rs**: Message creation methods with 100% similarity
2. **ccswarm/orchestrator/proactive_master.rs**: Pattern initialization methods with ~98% similarity  
3. **ccswarm/agent/search_agent.rs**: Agent lifecycle methods with ~97-99% similarity

### Statistics
- Total files analyzed: 208
- High similarity duplicates (>95%): 147 instances
- Estimated code reduction potential: 35-40%
- Test coverage impact: Minimal (test functions excluded)

## Refactoring Strategies

### 1. Factory Pattern for Message Creation (unified_bus.rs)

**Problem**: Three methods (`create_session_message`, `create_task_message`, `create_event_message`) have 100% identical structure.

**Solution**: Generic message factory

```rust
// Before: Three nearly identical methods
pub fn create_session_message(...) -> UnifiedMessage { ... }
pub fn create_task_message(...) -> UnifiedMessage { ... }
pub fn create_event_message(...) -> UnifiedMessage { ... }

// After: Single generic factory
pub fn create_message<T: MessageTrait>(
    id: &str,
    msg_type: T::MessageType,
    payload: serde_json::Value,
) -> UnifiedMessage {
    T::create(id, msg_type, payload)
}

// Trait for message types
trait MessageTrait {
    type MessageType;
    fn create(id: &str, msg_type: Self::MessageType, payload: Value) -> UnifiedMessage;
}
```

**Benefits**:
- Reduces 45 lines to ~15 lines
- Single point of maintenance
- Type-safe message creation

### 2. Template Method Pattern for ProactiveMaster

**Problem**: Pattern and template initialization methods share ~98% similar structure.

**Solution**: Abstract initialization template

```rust
// Base trait for initialization
trait PatternInitializer {
    type Item;
    fn create_patterns() -> Vec<(String, Self::Item)>;
}

// Generic initialization method
impl ProactiveMaster {
    fn initialize<T: PatternInitializer>() -> HashMap<String, T::Item> {
        T::create_patterns()
            .into_iter()
            .collect()
    }
}

// Concrete implementations
struct TaskPatternInitializer;
impl PatternInitializer for TaskPatternInitializer {
    type Item = TaskPattern;
    fn create_patterns() -> Vec<(String, TaskPattern)> {
        vec![
            ("frontend_component".to_string(), /* pattern */),
            ("api_endpoint".to_string(), /* pattern */),
        ]
    }
}
```

**Benefits**:
- Eliminates ~200 lines of duplicate initialization logic
- Easier to add new pattern types
- Consistent initialization across all pattern types

### 3. State Machine Pattern for SearchAgent

**Problem**: Multiple agent methods have 97-99% similar lifecycle management code.

**Solution**: State machine with shared transitions

```rust
// State machine for agent lifecycle
#[derive(Debug)]
enum AgentState {
    Initializing,
    Registering,
    Monitoring,
    Working,
    Available,
}

struct AgentStateMachine {
    state: AgentState,
    transitions: HashMap<(AgentState, Event), AgentState>,
}

impl SearchAgent {
    // Single method handles all state transitions
    async fn transition(&mut self, event: Event) -> Result<()> {
        let new_state = self.state_machine.transition(self.state, event)?;
        
        // Execute state-specific logic
        match new_state {
            AgentState::Monitoring => self.execute_monitoring().await?,
            AgentState::Working => self.execute_work().await?,
            // ...
        }
        
        self.state = new_state;
        Ok(())
    }
}
```

**Benefits**:
- Reduces ~300 lines of similar state management code
- Clear state transitions
- Easier to test and maintain

## Implementation Plan

### Phase 1: Foundation (Week 1)
- [ ] Create base traits and interfaces
- [ ] Set up generic factory utilities
- [ ] Implement error handling patterns

### Phase 2: Refactor Core Components (Week 2-3)
- [ ] Refactor unified_bus.rs message creation
- [ ] Implement ProactiveMaster template pattern
- [ ] Convert SearchAgent to state machine

### Phase 3: Testing & Validation (Week 4)
- [ ] Update unit tests for refactored components
- [ ] Add integration tests for new patterns
- [ ] Performance benchmarking

### Phase 4: Documentation & Rollout (Week 5)
- [ ] Update API documentation
- [ ] Create migration guide
- [ ] Code review and merge

## Risk Mitigation

1. **Backward Compatibility**: Maintain existing public APIs with deprecation warnings
2. **Performance**: Benchmark critical paths to ensure no regression
3. **Testing**: Increase test coverage before refactoring
4. **Rollback Plan**: Feature flags for gradual rollout

## Success Metrics

- **Code Reduction**: Target 35% reduction in duplicated code
- **Test Coverage**: Maintain or improve current 85% coverage
- **Performance**: No more than 5% performance degradation
- **Maintainability**: Reduce cyclomatic complexity by 30%

## Additional Opportunities

### Cross-Module Patterns

1. **Error Handling**: Standardize error creation and propagation
2. **Logging**: Create structured logging utilities
3. **Async Patterns**: Shared retry and timeout logic
4. **Configuration**: Unified configuration management

### Architectural Improvements

1. **Message Bus Abstraction**: Generic pub/sub interface
2. **Agent Trait Hierarchy**: Common agent behaviors
3. **Task Pipeline**: Reusable task processing framework

## Conclusion

This refactoring plan addresses the most critical code duplication issues while maintaining system stability. By implementing these design patterns, we can achieve significant code reduction, improved maintainability, and better architectural clarity.

Estimated effort: 5 weeks
Expected ROI: 40% reduction in maintenance time, 35% less code to maintain