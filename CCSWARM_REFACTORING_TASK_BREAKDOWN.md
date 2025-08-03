# CCSwarm Refactoring Task Breakdown

## Overview

This document provides a comprehensive breakdown of the ccswarm refactoring work, organized by priority and complexity. The refactoring aims to improve code maintainability, reduce duplication, and establish better architectural patterns.

---

## 1. CLI Command Handler Refactoring - Replace massive switch statement with registry pattern

### Current State and Problems

- **Location**: `crates/ccswarm/src/cli/mod.rs`
- **Problem**: Large match statement in execute method handling all CLI commands
- **Issues**:
  - Single file contains 1000+ lines of command handling logic
  - Difficult to add new commands without modifying central file
  - Testing individual commands requires setting up entire CLI context
  - No clear separation of concerns between command parsing and execution

### Target Architecture

```rust
// Command registry pattern
pub trait Command: Send + Sync {
    async fn execute(&self, context: CommandContext) -> Result<()>;
    fn validate(&self) -> Result<()>;
    fn name(&self) -> &'static str;
}

// Registry to hold all commands
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

// Each command in its own file
pub struct InitCommand {
    name: String,
    repo_url: Option<String>,
    agents: Vec<String>,
}

impl Command for InitCommand { ... }
```

### Implementation Steps

1. **Create command trait and registry** (2 hours)
   - Define `Command` trait in `crates/ccswarm/src/cli/command.rs`
   - Implement `CommandRegistry` with registration and lookup methods
   - Add `CommandContext` struct to pass shared state

2. **Extract individual commands** (8 hours)
   - Create separate modules for each command:
     - `cli/commands/init.rs`
     - `cli/commands/start.rs`
     - `cli/commands/task.rs`
     - `cli/commands/session.rs`
     - `cli/commands/sangha.rs`
     - etc.
   - Move command logic from main match statement to individual implementations

3. **Implement command registration** (2 hours)
   - Create registry initialization in `cli/mod.rs`
   - Register all commands during CLI initialization
   - Replace match statement with registry lookup

4. **Add command validation** (3 hours)
   - Implement validation logic in each command
   - Add pre-execution validation phase
   - Improve error messages for invalid arguments

5. **Testing infrastructure** (3 hours)
   - Create test utilities for command testing
   - Add unit tests for each command
   - Integration tests for command registry

### Dependencies

- Must maintain backward compatibility with existing CLI interface
- Requires coordination with error template engine for consistent error display

### Estimated Complexity: **HIGH** (18 hours)

---

## 2. Error Diagram Template Engine - Consolidate duplicate error display logic

### Current State and Problems

- **Location**: `crates/ccswarm/src/utils/error_template.rs`
- **Problem**: Multiple places in codebase creating similar error diagrams
- **Issues**:
  - Duplicate ASCII art generation code
  - Inconsistent error formatting across modules
  - Hard to maintain visual consistency
  - Template logic mixed with business logic

### Target Architecture

```rust
// Unified error template system
pub struct ErrorTemplateEngine {
    templates: HashMap<String, Template>,
}

pub enum ErrorTemplate {
    NetworkError { url: String, error: String },
    SessionError { session_id: String, state: String },
    ConfigError { path: PathBuf, reason: String },
}

impl ErrorTemplate {
    pub fn render(&self) -> String {
        // Use template engine to render consistent error diagrams
    }
}
```

### Implementation Steps

1. **Enhance template engine** (3 hours)
   - Extend existing `ErrorTemplateEngine` with more template types
   - Add support for nested templates and conditionals
   - Implement template variable validation

2. **Create error template catalog** (2 hours)
   - Define all error templates in central location
   - Categorize by error type (network, session, config, etc.)
   - Add metadata for each template (severity, help links)

3. **Replace duplicate error displays** (4 hours)
   - Search for all manual error formatting
   - Replace with template engine calls
   - Ensure consistent styling across all errors

4. **Add error context system** (2 hours)
   - Implement error context tracking
   - Add breadcrumb trail for errors
   - Include suggested fixes in templates

5. **Documentation and examples** (1 hour)
   - Document all available templates
   - Create examples for common error scenarios
   - Add guidelines for creating new templates

### Dependencies

- CLI output formatter must support template output
- Consider terminal width for responsive error display

### Estimated Complexity: **MEDIUM** (12 hours)

---

## 3. Session Management Abstraction - Create trait for common session operations

### Current State and Problems

- **Location**: `crates/ccswarm/src/session/`
- **Problem**: Multiple session types with duplicated logic
- **Files affected**:
  - `session/claude_session.rs`
  - `session/worktree_session.rs`
  - `session/persistent_session.rs`
  - `session/ai_session_adapter.rs`
- **Issues**:
  - Common operations implemented separately in each session type
  - Inconsistent error handling across session implementations
  - Difficult to add new session types
  - No unified interface for session operations

### Target Architecture

```rust
// Common session trait
#[async_trait]
pub trait Session: Send + Sync {
    async fn create(&mut self) -> Result<()>;
    async fn execute_command(&self, command: &str) -> Result<String>;
    async fn get_output(&self) -> Result<String>;
    async fn terminate(&mut self) -> Result<()>;
    async fn health_check(&self) -> Result<SessionHealth>;
}

// Shared session utilities
pub struct SessionManager<S: Session> {
    sessions: HashMap<String, S>,
    resource_monitor: ResourceMonitor,
}

// Session lifecycle management
pub trait SessionLifecycle {
    async fn pause(&mut self) -> Result<()>;
    async fn resume(&mut self) -> Result<()>;
    async fn checkpoint(&self) -> Result<SessionCheckpoint>;
}
```

### Implementation Steps

1. **Define core session traits** (3 hours)
   - Create `Session` trait with common operations
   - Add `SessionLifecycle` trait for state management
   - Define `SessionMetrics` trait for monitoring

2. **Extract common functionality** (4 hours)
   - Create `session/common.rs` for shared utilities
   - Move duplicate code to trait default implementations
   - Add helper functions for common patterns

3. **Refactor existing sessions** (6 hours)
   - Update `ClaudeSession` to implement traits
   - Update `WorktreeSession` to implement traits
   - Update `PersistentSession` to implement traits
   - Update `AISessionAdapter` to use trait abstractions

4. **Create unified session manager** (3 hours)
   - Implement generic `SessionManager<S: Session>`
   - Add session pooling capabilities
   - Integrate with resource monitoring

5. **Add session middleware** (2 hours)
   - Create middleware pattern for session operations
   - Add logging, metrics, and retry middleware
   - Enable composition of session behaviors

### Dependencies

- Must maintain compatibility with ai-session crate
- Resource monitoring integration required
- Consider impact on existing session pool implementation

### Estimated Complexity: **HIGH** (18 hours)

---

## 4. JSON-RPC Builder Pattern - Eliminate constructor duplication

### Current State and Problems

- **Location**: `crates/ccswarm/src/mcp/jsonrpc.rs`
- **Problem**: Repetitive JSON-RPC object construction
- **Issues**:
  - Multiple constructors doing similar initialization
  - Boilerplate code for creating requests/responses
  - Inconsistent error object creation
  - No validation of JSON-RPC compliance

### Target Architecture

```rust
// Fluent builder pattern
pub struct JsonRpcBuilder {
    version: String,
}

impl JsonRpcBuilder {
    pub fn request(self) -> RequestBuilder { ... }
    pub fn response(self) -> ResponseBuilder { ... }
    pub fn notification(self) -> NotificationBuilder { ... }
    pub fn error(self) -> ErrorBuilder { ... }
}

// Type-safe builders
pub struct RequestBuilder {
    id: Option<RequestId>,
    method: Option<String>,
    params: Option<Value>,
}

impl RequestBuilder {
    pub fn id(mut self, id: impl Into<RequestId>) -> Self { ... }
    pub fn method(mut self, method: impl Into<String>) -> Self { ... }
    pub fn params<T: Serialize>(mut self, params: T) -> Self { ... }
    pub fn build(self) -> Result<JsonRpcRequest> { ... }
}
```

### Implementation Steps

1. **Implement builder pattern** (2 hours)
   - Create `JsonRpcBuilder` as entry point
   - Add specialized builders for each message type
   - Ensure type safety and validation

2. **Add validation layer** (2 hours)
   - Validate JSON-RPC 2.0 compliance
   - Check required fields before building
   - Add custom validation rules

3. **Replace existing constructors** (3 hours)
   - Update all JSON-RPC object creation
   - Remove redundant constructor methods
   - Maintain backward compatibility where needed

4. **Add convenience methods** (1 hour)
   - Common error responses (parse error, method not found)
   - Batch request/response handling
   - Type conversions for common patterns

5. **Testing and documentation** (2 hours)
   - Unit tests for all builder paths
   - Integration tests with MCP client
   - Usage examples in documentation

### Dependencies

- Must maintain JSON-RPC 2.0 specification compliance
- Consider impact on MCP client implementation

### Estimated Complexity: **MEDIUM** (10 hours)

---

## 5. Module Reorganization - Split large modules into smaller, focused ones

### Current State and Problems

- **Problem**: Several modules exceed 1000 lines and mix concerns
- **Large modules**:
  - `cli/mod.rs` (2000+ lines)
  - `session/mod.rs` (900+ lines)
  - `orchestrator/master_claude.rs` (1500+ lines)
- **Issues**:
  - Difficult to navigate and understand
  - Multiple responsibilities in single files
  - Hard to test individual components
  - Increased merge conflicts

### Target Architecture

```
crates/ccswarm/src/
├── cli/
│   ├── mod.rs (100 lines - just exports)
│   ├── runner.rs (CLI runner core)
│   ├── parser.rs (argument parsing)
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── init.rs
│   │   ├── start.rs
│   │   └── ...
│   └── utils/
│       ├── output.rs
│       └── progress.rs
├── session/
│   ├── mod.rs (exports and traits)
│   ├── traits.rs (Session trait definitions)
│   ├── manager.rs (SessionManager)
│   ├── types/
│   │   ├── claude.rs
│   │   ├── worktree.rs
│   │   └── persistent.rs
│   └── utils/
│       ├── pool.rs
│       └── metrics.rs
```

### Implementation Steps

1. **Analyze and plan module splits** (2 hours)
   - Map current module dependencies
   - Identify logical boundaries
   - Plan new module structure

2. **Extract CLI commands** (4 hours)
   - Move each command to separate module
   - Update imports and exports
   - Maintain public API compatibility

3. **Split session module** (3 hours)
   - Extract traits to `session/traits.rs`
   - Move implementations to `session/types/`
   - Separate utilities to `session/utils/`

4. **Refactor orchestrator module** (4 hours)
   - Split master claude into smaller components
   - Extract delegation logic
   - Separate quality review components

5. **Update documentation** (1 hour)
   - Update module documentation
   - Add module-level README files
   - Update architecture documentation

### Dependencies

- Must maintain all public APIs
- Coordinate with other refactoring tasks
- Update all import statements across codebase

### Estimated Complexity: **LOW** (14 hours)

---

## Execution Plan

### Phase 1: Foundation (Week 1)
1. **Module Reorganization** - Establish clean structure
2. **Error Template Engine** - Create consistent error handling

### Phase 2: Core Refactoring (Week 2)
3. **Session Management Abstraction** - Unify session handling
4. **JSON-RPC Builder Pattern** - Simplify message construction

### Phase 3: CLI Enhancement (Week 3)
5. **CLI Command Handler Refactoring** - Implement registry pattern

### Total Estimated Time: 72 hours (2-3 weeks of focused development)

## Success Metrics

- **Code Reduction**: 20-30% fewer lines of code
- **Test Coverage**: Increase from current to 90%+
- **Module Size**: No module exceeds 500 lines
- **Duplication**: Reduce code duplication by 50%
- **Performance**: No regression in CLI response time

## Risk Mitigation

1. **Backward Compatibility**: All refactoring must maintain existing CLI interface
2. **Testing**: Each phase must include comprehensive tests before moving forward
3. **Documentation**: Update docs as part of each task, not after
4. **Incremental Delivery**: Each task should be mergeable independently
5. **Performance Monitoring**: Benchmark before and after each major change

## Notes

- Consider using feature flags for gradual rollout
- Each task should have its own PR for easier review
- Regular sync with team on architecture decisions
- Keep CHANGELOG.md updated with all changes