# Auto-Accept Mode Implementation

This document outlines the auto-accept mode functionality implemented for ccswarm, inspired by claude-squad's auto-accept features.

## Overview

The auto-accept mode allows ccswarm agents to automatically evaluate and execute operations without manual intervention, while maintaining strict safety guardrails and limits.

## Key Components

### 1. AutoAcceptConfig (`src/auto_accept/mod.rs`)

Configuration structure that controls auto-accept behavior:

```rust
pub struct AutoAcceptConfig {
    pub enabled: bool,                    // Master switch for auto-accept
    pub trusted_operations: Vec<OperationType>, // Operations that can be auto-accepted
    pub max_file_changes: usize,          // Limit on files that can be modified
    pub require_tests_pass: bool,         // Whether tests must pass before execution
    pub max_execution_time: u32,          // Maximum execution time in seconds
    pub restricted_files: Vec<String>,    // File patterns requiring manual approval
    pub require_clean_git: bool,          // Whether git status must be clean
    pub emergency_stop: bool,             // Emergency stop flag
}
```

**Default Configuration (Conservative):**
- `enabled: false` - Must be explicitly enabled
- Only safe operations trusted by default: ReadFile, FormatCode, RunTests, LintCode
- `max_file_changes: 5` - Limit concurrent file modifications
- `require_tests_pass: true` - Ensure quality gates
- Restricted files include: `Cargo.toml`, `*.sql`, `*.env`, migration files

### 2. OperationType Enum

Categorizes different types of operations for risk assessment:

```rust
pub enum OperationType {
    ReadFile,           // Risk: 1 (Very Low)
    WriteFile,          // Risk: 4 (Medium)
    EditFile,           // Risk: 3 (Low-Medium)
    DeleteFile,         // Risk: 8 (High)
    RunTests,           // Risk: 2 (Low)
    FormatCode,         // Risk: 1 (Very Low)
    LintCode,           // Risk: 1 (Very Low)
    GitOperation,       // Risk: 3-7 (Variable)
    InstallDependencies,// Risk: 5 (Medium)
    Build,              // Risk: 2 (Low)
    DatabaseOperation,  // Risk: 9 (Very High)
    NetworkRequest,     // Risk: 4 (Medium)
    SystemCommand,      // Risk: 6 (Medium-High)
    CreateDirectory,    // Risk: 2 (Low)
    Other,              // Risk: 5 (Default Medium)
}
```

### 3. AutoAcceptEngine

Core engine that evaluates operations for auto-acceptance:

**Key Methods:**
- `analyze_operation()` - Analyzes commands and determines operation type/risk
- `should_auto_accept()` - Evaluates whether operation should be auto-accepted
- `validate_changes()` - Post-execution validation
- `emergency_stop()` - Immediately disables auto-accept mode

**Decision Process:**
1. Check if auto-accept is enabled and emergency stop is not active
2. Verify operation type is in trusted operations list
3. Ensure risk level is acceptable (≤ 5)
4. Check file change limits
5. Validate against restricted file patterns
6. Apply conditional requirements (tests pass, clean git)

### 4. Agent Integration (`src/agent/mod.rs`)

Extended `ClaudeCodeAgent` with auto-accept capabilities:

**New Fields:**
- `auto_accept_engine: Option<AutoAcceptEngine>` - Auto-accept engine instance

**New Methods:**
- `enable_auto_accept()` - Enable auto-accept with configuration
- `disable_auto_accept()` - Disable auto-accept mode
- `emergency_stop_auto_accept()` - Emergency stop
- `execute_task_with_auto_accept()` - Execute tasks with auto-accept evaluation

**Safety Features:**
- Pre-execution condition validation (tests pass, clean git)
- Identity monitoring during execution
- Post-execution change validation
- Automatic emergency stop on validation failures

### 5. Session Management (`src/session/mod.rs`)

Extended session management to include auto-accept configuration:

**New Fields:**
- `auto_accept_config: Option<AutoAcceptConfig>` - Per-session auto-accept config

**New Methods:**
- `enable_auto_accept()` - Enable auto-accept for session
- `disable_auto_accept()` - Disable auto-accept for session
- `update_auto_accept_config()` - Update configuration
- `emergency_stop_all_auto_accept()` - Emergency stop for all sessions
- `get_auto_accept_sessions()` - Get sessions with auto-accept enabled

## Safety Mechanisms

### 1. Multiple Validation Layers
- **Pre-execution:** Validate conditions before running operations
- **During execution:** Monitor agent identity and boundaries
- **Post-execution:** Validate changes and execution time

### 2. Conservative Defaults
- Auto-accept disabled by default
- Only safe operations trusted initially
- Restrictive file change limits
- Required quality gates (tests, clean git)

### 3. Emergency Stop System
- Immediate shutdown capability
- Triggered by validation failures
- System-wide or per-session
- Requires manual intervention to reset

### 4. Risk Assessment
- Each operation type has assigned risk level
- High-risk operations (>5) automatically rejected
- Dynamic risk calculation based on commands

### 5. Pattern-Based File Protection
- Configurable restricted file patterns
- Protects critical files (configs, migrations, etc.)
- Glob pattern matching support

## Command Analysis

The system analyzes commands to determine operation types and risks:

```rust
// Examples:
"cargo test"           -> OperationType::RunTests (risk: 2)
"cargo fmt"            -> OperationType::FormatCode (risk: 1)
"rm important_file"    -> OperationType::DeleteFile (risk: 8)
"git push"             -> OperationType::GitOperation (risk: 7)
"cat src/main.rs"      -> OperationType::ReadFile (risk: 1)
```

## Usage Examples

### Enable Auto-Accept for Agent
```rust
let config = AutoAcceptConfig {
    enabled: true,
    trusted_operations: vec![
        OperationType::ReadFile,
        OperationType::FormatCode,
        OperationType::RunTests,
        OperationType::LintCode,
    ],
    max_file_changes: 10,
    require_tests_pass: true,
    ..AutoAcceptConfig::default()
};

agent.enable_auto_accept(config);
```

### Enable Auto-Accept for Session
```rust
session_manager.enable_auto_accept("session-id", config)?;
```

### Emergency Stop
```rust
// Single agent
agent.emergency_stop_auto_accept();

// All sessions
session_manager.emergency_stop_all_auto_accept();
```

## Testing

Comprehensive test coverage includes:

- **Configuration Tests** - Default values, validation, updates
- **Operation Analysis** - Command parsing, risk assessment
- **Decision Making** - Auto-accept logic, edge cases
- **Pattern Matching** - File restriction patterns
- **Safety Mechanisms** - Emergency stops, validation failures
- **Session Integration** - Session-level configuration
- **Edge Cases** - Error conditions, boundary cases

## Files Modified/Created

### New Files:
- `/src/auto_accept/mod.rs` - Core auto-accept implementation
- `/src/tests/auto_accept_tests.rs` - Comprehensive test suite
- `/AUTO_ACCEPT_IMPLEMENTATION.md` - This documentation

### Modified Files:
- `/src/lib.rs` - Added auto_accept module
- `/src/agent/mod.rs` - Integrated auto-accept into agent execution
- `/src/session/mod.rs` - Added session-level auto-accept configuration
- `/src/tests/mod.rs` - Added auto_accept_tests module

## Future Enhancements

1. **Machine Learning Integration** - Learn from operation outcomes to improve decisions
2. **Advanced Pattern Matching** - Use proper glob library for file patterns
3. **Operation Whitelisting** - User-defined safe operation patterns
4. **Rollback Mechanisms** - Automatic rollback on validation failures
5. **Metrics and Monitoring** - Track auto-accept effectiveness and safety
6. **Custom Risk Calculators** - Project-specific risk assessment rules

## Security Considerations

- **Conservative by Design** - Defaults to manual approval
- **Principle of Least Privilege** - Minimal trusted operations
- **Fail-Safe Mechanisms** - Emergency stops and validation failures
- **Audit Trail** - Operation history tracking
- **Boundary Enforcement** - Agent identity monitoring
- **Quality Gates** - Required test passes and clean state

This implementation provides a robust foundation for automated agent operations while maintaining the safety and security standards required for production use.