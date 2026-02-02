# CLAUDE.md

## Project Overview

ccswarm v0.4.3 - AI Multi-Agent Orchestration System built with **Rust-native patterns**. Provides workflow automation infrastructure with native PTY session management via ai-session.

> **Implementation Status**: ~50% complete. See [docs/analysis/](docs/analysis/) for detailed gap analysis.

## What Works

- **CLI Infrastructure**: All commands parse and route correctly
- **Session Management**: Native PTY sessions via ai-session (no tmux)
- **TUI Dashboard**: Real-time monitoring with ratatui
- **Git Worktrees**: Isolated workspaces per agent
- **Template System**: Project scaffolding from templates
- **Configuration**: Project and agent config management

## What's Partial/In Progress

- **`start` Command**: Initializes but coordination loop not implemented
- **Parallel Executor**: Structure exists, not wired to orchestrator
- **Auto-Create**: Template generation works; full AI generation incomplete
- **ai-session Integration**: Used for sessions; MessageBus not leveraged

## Quick Commands

```bash
cargo fmt && cargo clippy -- -D warnings && cargo test  # Before commit
cargo run -p ccswarm -- --help                          # Run ccswarm
```

## Rules

- [architecture-patterns](.claude/rules/architecture-patterns.md) - Rust patterns (Type-State, Channels, Actor)
- [development-standards](.claude/rules/development-standards.md) - Code quality, testing
- [security-guidelines](.claude/rules/security-guidelines.md) - Security, agent roles
- [performance](.claude/rules/performance.md) - Optimization guidelines

## Hooks

Automated validation and formatting via Claude Code hooks:
- `validate-agent-scope.sh` - Pre-edit agent scope validation
- `format-code.sh` - Post-edit auto-formatting
- `audit-trail.sh` - Session activity logging

## Agents (Subagents)

Specialized agents for Task tool delegation:
- [frontend-specialist](.claude/agents/frontend-specialist.md) - React, Vue, UI/UX
- [backend-specialist](.claude/agents/backend-specialist.md) - APIs, databases
- [devops-specialist](.claude/agents/devops-specialist.md) - Docker, CI/CD
- [qa-specialist](.claude/agents/qa-specialist.md) - Testing, quality
- [rust-fix-agent](.claude/agents/rust-fix-agent.md) - Rust build/clippy fixes
- [code-refactor-agent](.claude/agents/code-refactor-agent.md) - Code refactoring
- [architecture-reviewer](.claude/agents/architecture-reviewer.md) - Architecture review

## Reference (Load On-Demand)

- [commands](.claude/reference/commands.md) - All CLI commands
- [file-structure](.claude/reference/file-structure.md) - Project structure
- [version-notes](.claude/reference/version-notes.md) - v0.4.0 module details

## Skills

- [git-worktree](.claude/skills/git-worktree/SKILL.md) - Parallel development workflow
- [rust-agent-specialist](.claude/skills/rust-agent-specialist/SKILL.md) - Rust-native patterns
- [deploy-workflow](.claude/skills/deploy-workflow/SKILL.md) - Release deployment
- [benchmark-runner](.claude/skills/benchmark-runner/SKILL.md) - Performance benchmarks
- [hitl-approval](.claude/skills/hitl-approval/SKILL.md) - Human-in-the-loop approval

## Detailed Documentation

@docs/ARCHITECTURE.md
@docs/APPLICATION_SPEC.md
@.claude/settings.json

## Implementation Patterns (Learnings)

### SensitiveString Pattern
API keys and secrets should use `SensitiveString` wrapper type:
```rust
use ccswarm::providers::SensitiveString;

let api_key = SensitiveString::new("sk-secret");
println!("{:?}", api_key);  // Output: SensitiveString(****)
let actual = api_key.expose();  // Get actual value when needed
```
Benefits:
- Debug/Display masks values (prevents accidental logging)
- Supports Clone/Serialize/Deserialize
- Uses `secrecy` crate for memory safety

### Error Retry Hints Pattern
Errors should provide retry guidance:
```rust
impl CCSwarmError {
    fn should_retry(&self) -> bool { /* ... */ }
    fn suggested_retry_delay(&self) -> Duration { /* ... */ }
    fn max_retries(&self) -> u32 { /* ... */ }
}
```

### CLI Testing Patterns
- **E2E tests**: Execute actual binary (`tests/e2e_cli_test.rs`)
- **Unit tests**: Test argument parsing with `Cli::try_parse_from()` (`tests/cli_unit_tests.rs`)
- Use `{ .. }` pattern for enum variants with fields: `Commands::Start { .. } => {}`

### Type-State Pattern Notes
- `.expect()` is acceptable in type-state builders for invariants enforced by the type system
- Document why `.expect()` is safe in comments
- Type-state guarantees make these "impossible" to fail at runtime

### Parallel Claude Execution (v0.4.0)
Two approaches for parallel execution:

**Command-Based (Simple)**:
```rust
use ccswarm::subagent::{ParallelExecutor, ParallelConfig, SpawnTask};

let config = ParallelConfig {
    max_concurrent: 5,           // Up to 5 parallel Claude processes
    default_timeout_ms: 600_000, // 10 minutes per task
    fail_fast: false,            // Continue on failures
    ..Default::default()
};

let executor = ParallelExecutor::new(config);
let tasks = vec![
    SpawnTask::new("Create frontend components"),
    SpawnTask::new("Create backend API"),
];

let result = executor.execute_with_claude(tasks, Some(work_dir)).await?;
```

**PTY-Based (Interactive)**:
```rust
// For session-aware, interactive execution
let result = executor.execute_with_claude_pty(
    tasks,
    Some(work_dir),
    Some(3),  // max_turns
).await?;
```

### ai-session Integration Pattern
ccswarm uses ai-session for terminal management:
```rust
use ai_session::PtyHandle;

// Create PTY for Claude session
let pty = PtyHandle::new(24, 80)?;
pty.spawn_claude(&prompt, &working_dir, Some(3)).await?;

// Read output with timeout
let output = pty.read_with_timeout(timeout_ms).await?;
```

Benefits:
- Native PTY (no tmux dependency)
- Cross-platform (Linux, macOS)
- Interactive terminal support
- 93% token savings via session reuse

### SpawnTask Builder Pattern
Tasks for parallel execution use builder pattern:
```rust
let task = SpawnTask::new("Create a REST API")
    .with_id("backend-task-1")
    .with_agent_hint("backend")
    .with_priority(5);
```

### Hook System Integration (v0.4.0)
Integrated execution hooks from Claude Agent SDK pattern:
```rust
use ccswarm::hooks::{HookContext, HookRegistry, PreExecutionInput, HookResult};

// Create hook registry
let mut registry = HookRegistry::new();
registry.register_execution_hook(MyExecutionHook::new());

// Hooks run automatically during task execution
// - pre_execution: Before task starts
// - post_execution: After task completes
// - on_error: When errors occur
```

Hook results control flow:
- `HookResult::Continue` - Normal execution
- `HookResult::Skip { reason }` - Skip operation
- `HookResult::Deny { reason }` - Block operation
- `HookResult::Abort { reason }` - Abort entire task

### Verification Agent Pattern (v0.4.0)
Auto-created applications are automatically verified:
```rust
use ccswarm::orchestrator::{VerificationAgent, VerificationConfig};

let config = VerificationConfig::default();
let agent = VerificationAgent::new(config);
let result = agent.verify_app(app_path).await?;

// Check results
if result.success {
    println!("All {} checks passed", result.checks.len());
} else {
    // Get remediation suggestions
    let suggestions = VerificationAgent::get_remediation_suggestions(&result);
    for s in suggestions {
        println!("{}: {}", s.check_name, s.suggestion);
    }
}
```

Verification checks:
1. Required files exist (package.json, server.js, index.html)
2. Dependencies installed (npm install)
3. Backend server health check
4. Frontend HTML validation
5. API endpoints working
6. Tests pass (if present)

### DynamicSpawner Pattern (v0.4.0)
Dynamic agent spawning with workload balancing:
```rust
use ccswarm::subagent::{SubagentManager, DynamicSpawner, WorkloadBalancer};

// Create spawner from manager
let manager = Arc::new(RwLock::new(SubagentManager::new()));
let spawner = SubagentManager::create_spawner_from(manager.clone());

// Select agent based on capabilities
let agent_id = manager.read().await
    .select_agent_for_task(&["frontend", "react"]).await?;

// Use workload balancer for optimal selection
let balancer = manager.read().await.create_balancer();
let selected = balancer.select_agent(
    &required_capabilities,
    &available_agents,
    &current_workloads
);
```

### App Type Detection Pattern
Verification agent detects app type for customized checks:
```rust
use ccswarm::orchestrator::verification::AppType;

let app_type = VerificationAgent::detect_app_type(app_path);
match app_type {
    AppType::NodeJs => { /* Node.js checks */ }
    AppType::Python => { /* Python checks */ }
    AppType::Rust => { /* Rust checks */ }
    AppType::Go => { /* Go checks */ }
    AppType::Static => { /* Static site checks */ }
    AppType::Unknown => { /* Generic checks */ }
}
```
