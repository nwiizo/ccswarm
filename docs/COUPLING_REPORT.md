# Coupling Analysis Report

## Executive Summary

**Health Grade**: 🟠 D (Attention needed)

| Metric | Value |
|--------|-------|
| Files Analyzed | 113 |
| Total Modules | 112 |
| Total Couplings | 1805 |
| Balance Score | 0.45/1.00 |
| Balanced | 50 (3%) |
| Needs Refactoring | 128 |

**⚠️ Action Required**

- 🟠 **31 High** priority issues should be addressed soon
- 🟡 12 Medium priority issues to review

## 🔧 Refactoring Priorities

### Immediate Actions

**1. 🟠 `ccswarm::agent::task_builder_typestate` → `ccswarm::Task`**

- **Issue**: Cascading Change Risk - Intrusive coupling to frequently-changed component ccswarm::Task
- **Why**: Strongly coupling to volatile components means changes will cascade through the system, requiring updates in many places.
- **Action**: Add stable interface `TaskInterface`
- **Balance Score**: 0.00

**2. 🟠 `ccswarm::agent` → `ccswarm::identity`**

- **Issue**: Cascading Change Risk - Intrusive coupling to frequently-changed component ccswarm::identity
- **Why**: Strongly coupling to volatile components means changes will cascade through the system, requiring updates in many places.
- **Action**: Add stable interface `IdentityInterface`
- **Balance Score**: 0.00

**3. 🟠 `ccswarm::agent` → `ccswarm::hooks`**

- **Issue**: Cascading Change Risk - Intrusive coupling to frequently-changed component ccswarm::hooks
- **Why**: Strongly coupling to volatile components means changes will cascade through the system, requiring updates in many places.
- **Action**: Add stable interface `HooksInterface`
- **Balance Score**: 0.00

**4. 🟠 `ccswarm::agent` → `ccswarm::agent`**

- **Issue**: Cascading Change Risk - Intrusive coupling to frequently-changed component ccswarm::agent
- **Why**: Strongly coupling to volatile components means changes will cascade through the system, requiring updates in many places.
- **Action**: Add stable interface `AgentInterface`
- **Balance Score**: 0.00

**5. 🟠 `ccswarm::agent` → `ccswarm::hooks`**

- **Issue**: Cascading Change Risk - Intrusive coupling to frequently-changed component ccswarm::hooks
- **Why**: Strongly coupling to volatile components means changes will cascade through the system, requiring updates in many places.
- **Action**: Add stable interface `HooksInterface`
- **Balance Score**: 0.00

## Issues by Category

### High Afferent Coupling (4 instances)

> A module that many others depend on is hard to change. Any modification risks breaking dependents.

| Severity | Source | Target | Action |
|----------|--------|--------|--------|
| High | `41 dependents` | `ai-session::ai_session` | Introduce trait `Ai_sessionInterface` wi... |
| Medium | `40 dependents` | `ccswarm::*` | Introduce trait `*Interface` with method... |
| Medium | `23 dependents` | `ccswarm::workflow` | Introduce trait `WorkflowInterface` with... |
| Medium | `21 dependents` | `ccswarm::config` | Introduce trait `ConfigInterface` with m... |

### Cascading Change Risk (30 instances)

> Strongly coupling to volatile components means changes will cascade through the system, requiring updates in many places.

| Severity | Source | Target | Action |
|----------|--------|--------|--------|
| High | `...task_builder_typestate` | `ccswarm::Task` | Add stable interface `TaskInterface` |
| High | `ccswarm::agent` | `ccswarm::identity` | Add stable interface `IdentityInterface` |
| High | `ccswarm::agent` | `ccswarm::hooks` | Add stable interface `HooksInterface` |
| High | `ccswarm::agent` | `ccswarm::agent` | Add stable interface `AgentInterface` |
| High | `ccswarm::agent` | `ccswarm::hooks` | Add stable interface `HooksInterface` |

*...and 25 more instances*

### Global Complexity (6 instances)

> Strong coupling to distant components increases cognitive load and makes the system harder to understand and modify.

| Severity | Source | Target | Action |
|----------|--------|--------|--------|
| Medium | `...task_builder_typestate` | `ccswarm::Task` | Introduce trait `TaskTrait` with methods... |
| Medium | `...m::agent::task_builder` | `ccswarm::Task` | Introduce trait `TaskTrait` with methods... |
| Medium | `...or::agent_orchestrator` | `ccswarm::task_plan` | Introduce trait `Task_planTrait` with me... |
| Medium | `...m::workflow::execution` | `...rm::NodeExecutionState` | Introduce trait `NodeExecutionStateTrait... |
| Medium | `...m::workflow::execution` | `...arm::WorkflowExecution` | Introduce trait `WorkflowExecutionTrait`... |

*...and 1 more instances*

### God Module (2 instances)

> Module has too many responsibilities - too many functions, types, or implementations. Consider splitting into focused, cohesive modules. (SRP violation)

| Severity | Source | Target | Action |
|----------|--------|--------|--------|
| Medium | `coordination` | `...ns, 16 types, 16 impls` | Split into modules: coordination_core, c... |
| Medium | `cli` | `...ons, 22 types, 1 impls` | Split into modules: cli_core, cli_helper... |

### High Efferent Coupling (1 instances)

> A module depending on too many others is fragile and hard to test. Changes anywhere affect this module.

| Severity | Source | Target | Action |
|----------|--------|--------|--------|
| Medium | `ccswarm::session` | `29 dependencies` | Split into modules: ccswarm::session_cor... |

## Coupling Distribution

### By Integration Strength

| Strength | Count | % | Description |
|----------|-------|---|-------------|
| Contract | 60 | 3% | Depends on traits/interfaces only |
| Model | 552 | 31% | Uses data types/structs |
| Functional | 1065 | 59% | Calls specific functions |
| Intrusive | 128 | 7% | Accesses internal details |

### By Distance

| Distance | Count | % |
|----------|-------|---|
| Same Module (close) | 153 | 8% |
| Different Module | 128 | 7% |
| External Crate (far) | 1524 | 84% |

### By Volatility (Internal Couplings)

| Volatility | Count | % | Impact on Balance |
|------------|-------|---|-------------------|
| Low (rarely changes) | 11 | 4% | No penalty |
| Medium (sometimes changes) | 73 | 26% | Moderate penalty |
| High (frequently changes) | 197 | 70% | Significant penalty |

### Worst Balanced Couplings

| Source | Target | Strength | Distance | Volatility | Score | Status |
|--------|--------|----------|----------|------------|-------|--------|
| `ai-session::context` | `...versation_history` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `ai-session::context` | `...y::ContextSummary` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `ai-session::context` | `Message::Message` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `ai-session::context` | `...CompressedHistory` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `ai-session::context` | `...essed::compressed` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `ai-session::context` | `...:CompressionStats` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `...ession::core::pty` | `...pty::portable_pty` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `...ion::coordination` | `...t::MessageContent` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `...ion::coordination` | `...t::MessageContent` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `...ion::coordination` | `...t::MessageContent` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `...ion::coordination` | `...t::MessageContent` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `...ion::coordination` | `...t::MessageContent` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `...ion::coordination` | `...t::MessageContent` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `...ion::coordination` | `...t::MessageContent` | Intrusive | External | Med | 0.00 | 🔴 Critical |
| `...ion::coordination` | `...t::MessageContent` | Intrusive | External | Med | 0.00 | 🔴 Critical |

*Showing 15 of 1805 couplings*

## Module Statistics

| Module | Trait Impl | Inherent Impl | Internal Deps | External Deps |
|--------|------------|---------------|---------------|---------------|
| `agent` | 1 | 2 | 9 | 13 |
| `workflow` | 1 | 1 | 2 | 17 |
| `mcp::server` | 0 | 1 | 13 | 5 |
| `workflow::execution` | 3 | 3 | 11 | 4 |
| `...strator::agent_orchestrator` | 1 | 1 | 8 | 5 |
| `cli::handlers::workflow` | 0 | 1 | 11 | 1 |
| `persistence` | 1 | 2 | 6 | 6 |
| `workflow::cycle` | 2 | 2 | 7 | 4 |
| `core` | 5 | 3 | 4 | 7 |
| `cli` | 0 | 1 | 4 | 7 |
| `coordination` | 8 | 8 | 3 | 7 |
| `workflow::judge` | 2 | 1 | 5 | 5 |
| `workflow::pipeline` | 1 | 5 | 5 | 5 |
| `session_persistence` | 0 | 1 | 8 | 2 |
| `cli::handlers::harness` | 0 | 1 | 3 | 7 |
| `session::bridge` | 0 | 1 | 5 | 5 |
| `governance` | 0 | 3 | 2 | 7 |
| `core::lifecycle` | 0 | 0 | 6 | 3 |
| `utils::common` | 1 | 0 | 5 | 4 |
| `session` | 2 | 2 | 3 | 6 |

*Showing top 20 of 112 modules*

## Volatility Analysis

### High Volatility Files

⚠️ Strong coupling to these files increases cascading change risk.

| File | Changes |
|------|---------|
| `crates/ccswarm/src/cli/mod.rs` | 19 |
| `crates/ccswarm/src/workflow/flow.rs` | 16 |
| `crates/ccswarm/src/cli/command_registry.rs` | 14 |
| `crates/ccswarm/src/session/mod.rs` | 13 |
| `crates/ccswarm/src/cli/handlers/workflow.rs` | 12 |

## Temporal Coupling (Co-Change Analysis)

Files that frequently change together in git commits, indicating implicit coupling
beyond what code structure reveals.

### Strong Temporal Coupling (>50% co-change ratio)

⚠️ These pairs may share implicit knowledge (business logic, assumptions, data formats).

| File A | File B | Co-changes | Ratio |
|--------|--------|------------|-------|
| `crates/ccswarm/src/cli/command_registry.rs` | `crates/ccswarm/src/cli/mod.rs` | 12 | 86% |
| `crates/ccswarm/src/session/mod.rs` | `crates/ccswarm/src/subagent/parallel_executor.rs` | 8 | 89% |
| `crates/ccswarm/src/cli/command_registry.rs` | `crates/ccswarm/src/cli/handlers/mod.rs` | 6 | 86% |
| `crates/ccswarm/src/cli/handlers/mod.rs` | `crates/ccswarm/src/cli/mod.rs` | 6 | 86% |
| `crates/ccswarm/src/cli/mod.rs` | `crates/ccswarm/src/lib.rs` | 6 | 75% |
| `crates/ccswarm/src/agent/mod.rs` | `crates/ccswarm/src/cli/mod.rs` | 6 | 67% |
| `crates/ccswarm/src/cli/command_registry.rs` | `crates/ccswarm/src/cli/handlers/workflow.rs` | 6 | 50% |
| `crates/ccswarm/src/cli/handlers/workflow.rs` | `crates/ccswarm/src/cli/mod.rs` | 6 | 50% |
| `crates/ccswarm/src/cli/handlers/diagnostics.rs` | `crates/ccswarm/src/cli/handlers/task.rs` | 5 | 100% |
| `crates/ccswarm/src/cli/command_registry.rs` | `crates/ccswarm/src/cli/handlers/task.rs` | 5 | 100% |

### Moderate Temporal Coupling

| File A | File B | Co-changes | Ratio |
|--------|--------|------------|-------|
| `crates/ccswarm/src/cli/mod.rs` | `crates/ccswarm/src/session/mod.rs` | 6 | 46% |
| `crates/ccswarm/src/cli/command_registry.rs` | `crates/ccswarm/src/session/mod.rs` | 5 | 38% |
| `crates/ccswarm/src/cli/command_registry.rs` | `crates/ccswarm/src/workflow/flow.rs` | 5 | 36% |
| `crates/ccswarm/src/agent/mod.rs` | `crates/ccswarm/src/cli/command_registry.rs` | 4 | 44% |
| `crates/ccswarm/src/cli/mod.rs` | `crates/ccswarm/src/subagent/parallel_executor.rs` | 4 | 44% |
| `crates/ccswarm/src/cli/handlers/workflow.rs` | `crates/ccswarm/src/workflow/flow.rs` | 4 | 33% |
| `crates/ccswarm/src/cli/mod.rs` | `crates/ccswarm/src/workflow/flow.rs` | 4 | 25% |
| `crates/ccswarm/src/cli/mod.rs` | `crates/ccswarm/src/orchestrator/auto_create.rs` | 3 | 43% |
| `crates/ccswarm/src/orchestrator/auto_create.rs` | `crates/ccswarm/src/subagent/parallel_executor.rs` | 3 | 43% |
| `crates/ccswarm/src/lib.rs` | `crates/ccswarm/src/subagent/parallel_executor.rs` | 3 | 38% |

## ⚠️ Circular Dependencies

Found **2 circular dependency cycle(s)** involving **3 modules**.

Circular dependencies make code harder to understand, test, and maintain.
Consider breaking cycles by:

1. Extracting shared types into a separate module
2. Inverting dependencies using traits/interfaces
3. Moving functionality to reduce coupling

### Detected Cycles

1. `ai-session::core → ai-session::context` → `ai-session::core`
2. `ai-session::core → ai-session::persistence` → `ai-session::core`

## Balance Guidelines

The goal is **balanced coupling**, not zero coupling.

### Ideal Patterns ✅

| Pattern | Example | Why It Works |
|---------|---------|--------------|
| Strong + Close | `impl` blocks in same module | Cohesion within boundaries |
| Weak + Far | Trait impl for external crate | Loose coupling across boundaries |

### Problematic Patterns ❌

| Pattern | Problem | Solution |
|---------|---------|----------|
| Strong + Far | Global complexity | Introduce adapter or move closer |
| Strong + Volatile | Cascading changes | Add stable interface |
| Intrusive + Cross-boundary | Encapsulation violation | Extract trait API |

