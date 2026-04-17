# Architecture Patterns

## Rust-Native Patterns (REQUIRED)

### What Works
- **Type-State Pattern**: Compile-time state validation with zero runtime cost
- **Channel-Based Orchestration**: Message-passing without Arc<Mutex> or shared state
- **Iterator Pipelines**: Zero-cost abstractions for efficient task processing
- **Actor Model**: Replace locks with message-passing actors
- **Minimal Testing**: Only 8-10 essential tests - focus on core functionality

### What Doesn't Work
- **Layered Architecture**: Unnecessary abstraction in Rust
- **Excessive Arc<Mutex>**: Causes contention, use channels instead
- **Over-testing**: 300+ tests create maintenance burden without value
- **Complex Abstractions**: Direct patterns are clearer and more efficient

## Implementation Patterns

### Command Registry Pattern
- **Purpose**: Eliminates massive match statements in CLI handling
- **Implementation**: HashMap of command handlers with async closures
- **Location**: `crates/ccswarm/src/cli/command_registry.rs`
- **Usage**: Register commands once, dispatch dynamically

### Error Template System
- **Purpose**: Standardizes error diagrams and visualizations
- **Implementation**: Template engine with reusable diagram patterns
- **Templates**: Box diagrams, flow diagrams, network diagrams
- **Location**: `crates/ccswarm/src/utils/error_template.rs`

## Concurrency Rules

- Use `tokio::sync::mpsc` channels for agent communication
- Prefer `RwLock` over `Mutex` when reads dominate
- Never hold locks across `.await` points
- Use channel-based coordination, not shared state

## Claude Code Integration

ccswarm uses **Claude Code via ACP** with efficient patterns:
- **Auto-Connect**: WebSocket connection to ws://localhost:9100
- **Channel-Based Communication**: No shared state between agents
- **Type-Safe Messages**: Compile-time validation of message types
- **Actor Pattern**: Each agent as an independent actor
