# Module Reorganization Plan

## Overview

This plan addresses the large modules identified in the ccswarm codebase that exceed 1000 lines and need to be split into smaller, more focused modules.

## Modules to Reorganize

### 1. CLI Module (5559 lines) - CRITICAL

**Current State:**
- `/crates/ccswarm/src/cli/mod.rs` contains all command definitions, handlers, and CLI logic
- Already has a `commands/` subdirectory but underutilized
- Has command_registry.rs for the registry pattern

**Proposed Structure:**
```
cli/
├── mod.rs (200 lines) - Main CLI struct and entry point
├── command_registry.rs - Registry pattern (existing)
├── commands/
│   ├── mod.rs - Command enum definitions
│   ├── agent.rs - Agent-related commands
│   ├── config.rs - Config management commands
│   ├── delegate.rs - Task delegation commands
│   ├── evolution.rs - Evolution commands
│   ├── extend.rs - Extension commands
│   ├── init.rs (existing)
│   ├── quality.rs - Quality review commands
│   ├── resource.rs - Resource management commands
│   ├── sangha.rs - Sangha voting commands
│   ├── search.rs - Search functionality
│   ├── session.rs - Session management commands
│   ├── status.rs (existing)
│   ├── task.rs (existing)
│   ├── template.rs - Template commands
│   └── worktree.rs - Worktree management
├── handlers/
│   ├── mod.rs - Handler trait definitions
│   ├── auto_create.rs - Auto-create handler
│   ├── doctor.rs - Doctor command handler
│   ├── help.rs - Help system handler
│   ├── quickstart.rs - Quickstart handler
│   ├── setup.rs - Setup wizard handler
│   └── tutorial.rs - Tutorial handler
└── utils/
    ├── mod.rs
    ├── error_help.rs (existing)
    ├── output.rs (existing)
    └── progress.rs (existing)
```

### 2. Orchestrator Module (1609 lines)

**Current State:**
- `/crates/ccswarm/src/orchestrator/mod.rs` contains core orchestration logic
- Already has some split files (proactive_master.rs, auto_create.rs)

**Proposed Structure:**
```
orchestrator/
├── mod.rs (300 lines) - Core orchestrator trait and main struct
├── task_distribution.rs - Task analysis and distribution logic
├── agent_coordination.rs - Agent selection and coordination
├── quality_control.rs - Quality review integration
├── session_management.rs - Session lifecycle management
├── proactive_master.rs (existing)
├── auto_create.rs (existing)
└── utils/
    ├── mod.rs
    └── metrics.rs - Orchestration metrics
```

### 3. TUI Module (ui.rs: 1579 lines, app.rs: 1327 lines)

**Current State:**
- Large UI rendering logic in single files
- Complex state management

**Proposed Structure:**
```
tui/
├── mod.rs (100 lines) - Main TUI entry point
├── app.rs (400 lines) - Core app state only
├── ui/
│   ├── mod.rs (200 lines) - Main UI composition
│   ├── dashboard.rs - Dashboard view
│   ├── agent_list.rs - Agent list view
│   ├── task_view.rs - Task details view
│   ├── logs.rs - Log viewer
│   ├── metrics.rs - Metrics display
│   └── help.rs - Help overlay
├── widgets/
│   ├── mod.rs
│   ├── status_bar.rs
│   ├── command_bar.rs
│   ├── progress_indicators.rs
│   └── charts.rs
└── handlers/
    ├── mod.rs
    ├── keyboard.rs - Keyboard event handling
    ├── mouse.rs - Mouse event handling
    └── commands.rs - Command processing
```

### 4. Agent Module (1362 lines)

**Current State:**
- Core agent functionality mixed with various agent types
- Already has some separation (persistent.rs, pool.rs)

**Proposed Structure:**
```
agent/
├── mod.rs (300 lines) - Core agent traits and base implementation
├── types/
│   ├── mod.rs
│   ├── simple.rs - Simple agent implementation
│   ├── persistent.rs (move existing)
│   ├── pool.rs (move existing)
│   └── search.rs - Search agent implementation
├── identity/
│   ├── mod.rs
│   ├── roles.rs - Agent role definitions
│   └── constraints.rs - Role constraints
├── execution/
│   ├── mod.rs
│   ├── task_runner.rs - Task execution logic
│   ├── context.rs - Execution context
│   └── results.rs - Result handling
└── communication/
    ├── mod.rs
    ├── messages.rs - Inter-agent messages
    └── protocols.rs - Communication protocols
```

## Implementation Strategy

### Phase 1: CLI Module (Day 1-2)
1. Move command enums to `commands/mod.rs`
2. Extract handlers to individual files in `commands/` and `handlers/`
3. Update command_registry.rs to import from new locations
4. Ensure all tests pass

### Phase 2: Orchestrator Module (Day 2)
1. Extract task distribution logic
2. Move agent coordination to separate file
3. Consolidate quality control logic
4. Update imports and tests

### Phase 3: TUI Module (Day 3)
1. Split UI components into widgets
2. Extract event handlers
3. Modularize views
4. Test interactive functionality

### Phase 4: Agent Module (Day 3-4)
1. Move agent types to dedicated files
2. Extract identity management
3. Separate execution logic
4. Update all agent references

### Phase 5: Verification (Day 4-5)
1. Run full test suite
2. Check for circular dependencies
3. Verify no functionality regression
4. Update documentation

## Success Metrics

1. **No file exceeds 1000 lines**
2. **Clear separation of concerns**
3. **Improved code navigation**
4. **All tests pass**
5. **No performance regression**
6. **Easier to add new features**

## Risk Mitigation

1. **Incremental approach** - One module at a time
2. **Comprehensive testing** - Run tests after each change
3. **Git commits** - Commit after each successful reorganization
4. **Preserve public API** - No breaking changes to external interfaces
