# Upcoming Features (v0.4.0)

> ⚠️ **Status:** Code exists in codebase but NOT fully integrated with orchestrator.
> This document guides Claude on completing v0.4.0 features documented in ARCHITECTURE.md.

## Implementation Roadmap

### Phase 1: Core Orchestration (Critical - blocks all usage)
- [ ] Fix `start` command to run coordination loop instead of exiting
- [ ] Wire ParallelExecutor to orchestrator for actual agent spawning
- [ ] Implement inter-process communication (Unix socket or SQLite)
- [ ] Enable true daemon mode (`--daemon` flag currently ignored)

### Phase 2: ai-session Integration (Major - documented but not implemented)
- [ ] Integrate ai-session MessageBus for agent coordination
- [ ] Enable context compression (93% token savings via zstd)
- [ ] Implement session persistence (move from in-memory to disk)
- [ ] Add session resume/fork/checkpoint capabilities

### Phase 3: Implementation Patterns (High Value - code exists, not integrated)
- [ ] Wire HookSystem into orchestrator task execution
- [ ] Call VerificationAgent after auto-create completes
- [ ] Implement DynamicSpawner workload balancing
- [ ] Connect ParallelExecutor to task queue

### Phase 4: Polish (Improves UX)
- [ ] Fix TUI async key handlers
- [ ] Add configurable verification checks
- [ ] Implement workload metrics collection

---

## v0.4.0 Implementation Patterns

### 1. Hook System Integration
**Status:** ✅ Code exists | 🔧 Not integrated

**Location:** `crates/ccswarm/src/hooks/`

**Files:**
- `mod.rs` - HookResult, HookContext, ExecutionHooks, ToolHooks traits
- `registry.rs` - HookRegistry with priority-based execution
- `builtin.rs` - LoggingHook, SecurityHook implementations

**What it does:**
Provides extension points for custom behavior before/after/on-error for task execution and tool usage.

**Usage Example:**
```rust
use ccswarm::hooks::{HookRegistry, ExecutionHooks};

// Create and register hooks
let mut registry = HookRegistry::new();
registry.register_execution_hook(MyCustomHook::new());

// Hooks run automatically during task execution:
// - pre_execution: Before task starts (can skip, deny, or abort)
// - post_execution: After task completes (can modify results)
// - on_error: When errors occur (can retry or abort)
```

**HookResult Controls Flow:**
- `HookResult::Continue` - Normal execution proceeds
- `HookResult::ContinueWith(data)` - Modify execution context
- `HookResult::Skip { reason }` - Skip this operation
- `HookResult::Deny { reason }` - Block operation entirely
- `HookResult::Abort { reason }` - Abort entire task

**Built-in Hooks:**
```rust
// LoggingHook: Event logging with configurable levels
use ccswarm::hooks::builtin::LoggingHook;
registry.register_execution_hook(LoggingHook::new(LogLevel::Info));

// SecurityHook: File/command protection
use ccswarm::hooks::builtin::SecurityHook;
let security = SecurityHook::new()
    .protect_files(&[".env", "*.key", ".git/config"])
    .protect_commands(&["rm -rf", "sudo"]);
registry.register_tool_hook(security);
```

**Integration TODO:**
- [ ] Wire hooks into `crates/ccswarm/src/orchestrator/proactive_master.rs` task execution
- [ ] Add hook configuration to `ccswarm.yaml` schema
- [ ] Document `.claude/hooks/` shell scripts integration with HookRegistry
- [ ] Add hook execution tracing to TUI

**References:**
- Gap Analysis: `docs/analysis/00-capability-gap-analysis.md` (missing extensibility)
- Code: `crates/ccswarm/src/hooks/` (complete implementation)

---

### 2. Verification Agent Pattern
**Status:** ✅ Code exists | 🔧 Not integrated

**Location:** `crates/ccswarm/src/orchestrator/verification.rs`

**Purpose:** Auto-verify created applications with 6-check workflow

**Verification Checks:**
1. **Required files exist** - package.json, server.js, index.html (app-type specific)
2. **Dependencies installed** - npm install, cargo build, pip install
3. **Backend health check** - GET /health endpoint responds
4. **Frontend HTML validation** - Valid structure, no broken links
5. **API endpoints working** - Test documented endpoints
6. **Tests pass** - Run test suite if present

**Usage Example:**
```rust
use ccswarm::orchestrator::{VerificationAgent, VerificationConfig};

// Configure verification
let config = VerificationConfig {
    port: 3000,
    timeout_secs: 30,
    auto_install_deps: true,
    strict_mode: false,
};

// Create agent and verify
let agent = VerificationAgent::new(config);
let result = agent.verify_app(&app_path).await?;

// Check results
if result.success {
    println!("✅ All {} checks passed in {:?}",
        result.checks.len(), result.duration);
} else {
    // Get remediation suggestions
    let suggestions = VerificationAgent::get_remediation_suggestions(&result);
    for suggestion in suggestions {
        println!("🔧 {}: {}", suggestion.check_name, suggestion.suggestion);
    }
}
```

**App Type Detection:**
```rust
use ccswarm::orchestrator::verification::AppType;

// Automatically detect app type from files
let app_type = VerificationAgent::detect_app_type(&app_path);

match app_type {
    AppType::NodeJs => {
        // Check for package.json, node_modules, server.js
        // Run: npm install && npm test
    }
    AppType::Python => {
        // Check for requirements.txt, venv, app.py
        // Run: pip install && pytest
    }
    AppType::Rust => {
        // Check for Cargo.toml, target/
        // Run: cargo build && cargo test
    }
    AppType::Go => {
        // Check for go.mod, main.go
        // Run: go build && go test
    }
    AppType::Static => {
        // Check for index.html, CSS, JS
        // Validate HTML structure only
    }
    AppType::Unknown => {
        // Run generic file existence checks
    }
}
```

**Integration TODO:**
- [ ] Call `VerificationAgent::verify_app()` after `auto-create` completes
- [ ] Add verification results to TUI dashboard
- [ ] Generate remediation tasks from failed checks
- [ ] Add `ccswarm verify <path>` CLI command

**References:**
- Gap Analysis: `docs/analysis/00-capability-gap-analysis.md` (auto-create validation missing)
- Code: `crates/ccswarm/src/orchestrator/verification.rs` (complete implementation)

---

### 3. DynamicSpawner Pattern
**Status:** ✅ Code exists | 🔧 Not integrated

**Location:** `crates/ccswarm/src/subagent/spawner.rs`

**Purpose:** Task-time dynamic agent generation with intelligent workload balancing

**Core Types:**
- `SpawnerConfig` - Concurrency limits, cleanup settings, timeout configuration
- `SpawnContext` - Builder for agent spawning with capabilities and resources
- `SpawnTask` - Builder pattern for task definition
- `WorkloadBalancer` - Agent selection strategies

**Workload Balancing Strategies:**
```rust
pub enum BalancingStrategy {
    RoundRobin,          // Sequential agent selection (fairness)
    LeastLoaded,         // Select agent with minimum current load
    CapabilityMatching,  // Match required capabilities to agent skills
    PriorityBased,       // Select based on task priority levels
    Custom(Box<dyn Fn(&[Agent], &Task) -> AgentId>),  // User-defined logic
}
```

**Usage Example:**
```rust
use ccswarm::subagent::{SubagentManager, DynamicSpawner, WorkloadBalancer};
use std::sync::{Arc, RwLock};

// Create spawner from manager
let manager = Arc::new(RwLock::new(SubagentManager::new()));
let spawner = SubagentManager::create_spawner_from(manager.clone());

// Select agent based on capabilities
let required_caps = vec!["frontend", "react"];
let agent_id = manager.read().await
    .select_agent_for_task(&required_caps).await?;

// Use workload balancer for optimal selection
let balancer = manager.read().await.create_balancer();
let selected = balancer.select_agent(
    &required_capabilities,  // What task needs
    &available_agents,       // Who can do it
    &current_workloads       // Who's least busy
);

println!("Selected agent: {} (load: {})", selected.id, selected.load);
```

**SpawnTask Builder Pattern:**
```rust
use ccswarm::subagent::SpawnTask;

let task = SpawnTask::new("Create a REST API for users")
    .with_id("backend-task-1")
    .with_agent_hint("backend")  // Suggest backend agent
    .with_priority(5)            // 0-10 scale
    .with_timeout_secs(600)      // 10 minutes max
    .with_capabilities(vec!["api", "database"])
    .with_resources(ResourceLimits {
        max_tokens: 100_000,
        max_memory_mb: 512,
    });
```

**Integration TODO:**
- [ ] Wire `ParallelExecutor` to use `DynamicSpawner` for agent selection
- [ ] Implement workload tracking in `SubagentManager`
- [ ] Add balancing strategy configuration to `ccswarm.yaml`
- [ ] Add workload metrics to TUI dashboard

**References:**
- Gap Analysis: `docs/analysis/00-capability-gap-analysis.md` (load balancing missing)
- Code: `crates/ccswarm/src/subagent/spawner.rs` (complete implementation)

---

### 4. Parallel Execution Patterns
**Status:** ✅ Code exists | 🔧 Not integrated

**Location:** `crates/ccswarm/src/subagent/parallel_executor.rs`

**Purpose:** Execute multiple tasks concurrently with true parallelism

**Two Approaches:**

#### A) Command-Based (Simple - for independent tasks)
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
    SpawnTask::new("Set up database schema"),
];

// Execute all tasks in parallel
let results = executor.execute_with_claude(tasks, Some(work_dir)).await?;

// Check results
for (task, result) in results {
    match result {
        Ok(output) => println!("✅ {}: {}", task.description, output),
        Err(e) => println!("❌ {}: {}", task.description, e),
    }
}
```

#### B) PTY-Based (Interactive - for session-aware execution)
```rust
use ai_session::PtyHandle;

// Create PTY for Claude session
let pty = PtyHandle::new(24, 80)?;  // rows, cols

// Spawn Claude with prompt
pty.spawn_claude(
    &prompt,
    &working_dir,
    Some(3)  // max_turns
).await?;

// Read output with timeout
let output = pty.read_with_timeout(timeout_ms).await?;

// PTY supports:
// - Interactive sessions with context
// - Session reuse for token savings
// - Cross-platform terminal support
```

**Benefits:**
- **Parallelism**: Multiple Claude instances run concurrently
- **Efficiency**: Complete independent tasks faster
- **Fault Isolation**: One task failure doesn't affect others
- **Resource Control**: Configurable concurrency limits

**Integration TODO:**
- [ ] Connect `ParallelExecutor` to `ccswarm start` coordination loop
- [ ] Implement task distribution from task queue
- [ ] Add progress tracking to TUI (show parallel tasks)
- [ ] Wire to `DynamicSpawner` for load-balanced agent selection

**References:**
- Gap Analysis: `docs/analysis/00-capability-gap-analysis.md` (parallel execution missing)
- Code: `crates/ccswarm/src/subagent/parallel_executor.rs` (complete implementation)

---

### 5. ai-session Integration (MessageBus & Context Compression)
**Status:** ✅ Code exists (in ai-session crate) | 🔧 Not integrated

**Location:** `crates/ai-session/src/` (workspace crate)

**Purpose:** Token-efficient session management with agent coordination

**Key Components:**

#### A) Context Compression (93% token savings)
```rust
use ai_session::{SessionManager, SessionConfig};

let mut config = SessionConfig::default();
config.enable_ai_features = true;
config.context_config = ContextConfig {
    max_tokens: 4096,
    compression_threshold: 0.8,  // Compress when >80% full
    use_zstd: true,              // zstd algorithm
};

let session = manager.create_session_with_config(config).await?;

// Context automatically compressed
// 93% token reduction through intelligent history management
let context = session.get_ai_context().await?;
println!("Tokens: {} (saved: {}%)",
    context.token_count,
    context.compression_ratio * 100.0
);
```

#### B) MessageBus for Agent Coordination
```rust
use ai_session::coordination::{CoordinationBus, Message, MessageType};

// Create shared message bus
let bus = Arc::new(RwLock::new(CoordinationBus::new()));

// Agents publish messages
bus.write().await.broadcast(Message {
    sender: "frontend-agent".to_string(),
    receiver: Some("backend-agent".to_string()),
    msg_type: MessageType::TaskResult,
    content: json!({
        "status": "ready",
        "port": 3000,
        "api_base": "http://localhost:3000/api"
    }),
    timestamp: Utc::now(),
}).await?;

// Other agents subscribe
let messages = bus.read().await.get_messages_for("backend-agent");
for msg in messages {
    println!("📨 From {}: {:?}", msg.sender, msg.content);
}
```

#### C) Session Persistence
```rust
use ai_session::persistence::PersistenceConfig;

let config = PersistenceConfig {
    storage_path: PathBuf::from("~/.ccswarm/sessions/"),
    compression: true,
    max_history: 1000,
    auto_snapshot_interval_secs: 300,  // Every 5 minutes
};

let session = SessionManager::with_persistence(config)?;

// Automatic:
// - State snapshots every 5 minutes
// - Crash recovery from last snapshot
// - History compression with zstd
```

#### D) Session Resume/Fork/Checkpoint
```rust
// Resume previous session
let session = manager.resume_session("session-id-123").await?;

// Fork session to try alternative approach
let forked = session.fork("try-different-approach").await?;

// Create checkpoint for rollback
let checkpoint = session.create_checkpoint("before-risky-change").await?;

// Restore from checkpoint if needed
session.restore_checkpoint(&checkpoint).await?;
```

**Integration TODO:**
- [ ] Use `CoordinationBus` for inter-agent communication
- [ ] Enable context compression in all agent sessions
- [ ] Implement session persistence to `~/.ccswarm/sessions/`
- [ ] Wire `ccswarm session resume <id>` command
- [ ] Add fork/checkpoint support to CLI

**References:**
- ARCHITECTURE.md: Lines 95-115 (ai-session integration documented)
- ARCHITECTURE.md: Lines 247-251, 319-327 (93% token savings documented)
- Gap Analysis: `docs/analysis/00-capability-gap-analysis.md` (not implemented)
- Code: `crates/ai-session/src/` (complete implementation)

---

### 6. Inter-Process Communication & Daemon Mode
**Status:** 🚧 Planned (infrastructure) | ⚠️ `--daemon` flag exists but ignored

**Location:** Need to implement in `crates/ccswarm/src/daemon/` (new module)

**Purpose:** Enable true background daemon with IPC for CLI communication

**Current Problem (from gap analysis Section 2.3):**
```
| Process A           | Process B            | Can Communicate? |
|---------------------|----------------------|------------------|
| ccswarm tui         | ccswarm task create  | ❌ No            |
| ccswarm start       | ccswarm status       | ❌ No            |
```

**Proposed Solution:**

#### Option A: Unix Socket + JSON-RPC
```rust
// Daemon listens on Unix socket
let socket_path = "~/.ccswarm/daemon.sock";
let listener = UnixListener::bind(socket_path)?;

// CLI connects and sends commands
let stream = UnixStream::connect(socket_path)?;
serde_json::to_writer(&stream, &Request::CreateTask { ... })?;
```

#### Option B: SQLite Database + File Watching
```rust
// Daemon polls SQLite for new tasks
let db = Database::open("~/.ccswarm/tasks.db")?;
loop {
    let pending = db.query("SELECT * FROM tasks WHERE status = 'pending'")?;
    // Process pending tasks
}

// CLI writes to database
db.execute("INSERT INTO tasks VALUES (?)", task)?;
```

#### Option C: Hybrid (Recommended)
- Unix socket for real-time commands (start, stop, status)
- SQLite for task persistence (survives daemon restarts)

**True Daemon Mode:**
```rust
pub async fn run_daemon(config: DaemonConfig) -> Result<()> {
    // Fork to background
    daemonize()?;

    // Write PID file
    write_pid_file(&config.pid_file)?;

    // Set up signal handlers
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sighup = signal(SignalKind::hangup())?;

    // Main event loop
    loop {
        tokio::select! {
            _ = sigterm.recv() => {
                info!("Received SIGTERM, shutting down");
                break;
            }
            _ = sighup.recv() => {
                info!("Received SIGHUP, reloading config");
                reload_config().await?;
            }
            Some(task) = task_queue.recv() => {
                spawn_task_handler(task).await?;
            }
        }
    }

    cleanup_pid_file(&config.pid_file)?;
    Ok(())
}
```

**Integration TODO:**
- [ ] Implement Unix socket server in daemon
- [ ] Add SQLite database for task persistence
- [ ] Wire `--daemon` flag to actually daemonize
- [ ] Wire `--port` flag to status HTTP server
- [ ] Add `ccswarm daemon status|stop|reload` commands
- [ ] Update TUI to connect via IPC instead of in-process

**References:**
- Gap Analysis: `docs/analysis/00-capability-gap-analysis.md` Section 2.3 (no IPC)
- Gap Analysis: `docs/analysis/00-capability-gap-analysis.md` Section 2.4 (--daemon ignored)
- COMMANDS.md: Documents daemon-related commands

---

## Known Limitations (from Gap Analysis)

From `docs/analysis/00-capability-gap-analysis.md`:

| Feature | Documented | Actual | Severity | v0.4.0 Fix |
|---------|-----------|--------|----------|------------|
| Multi-agent orchestration | Working | Non-functional | Critical | Fix start loop |
| `ccswarm start` daemon | Background process | Exits in 1s | Critical | Daemon mode |
| 93% token savings | Implemented | Not integrated | Major | ai-session integration |
| Quality review (30s) | Automatic | No loop | Major | Background task |
| Session persistence | Working | All in-memory | Major | ai-session persistence |
| Sangha voting | Democratic | Creates records only | Major | Future |
| Git worktree isolation | Per agent | Created but unused | Moderate | Use in parallel exec |

---

## Implementation Priority

### Critical Path (blocks all usage):
1. Fix `start` command coordination loop in `crates/ccswarm/src/orchestrator/proactive_master.rs`
2. Wire `ParallelExecutor` to actually spawn agent processes
3. Implement inter-process communication (Unix socket or SQLite)
4. Enable true daemon mode (`--daemon` flag)

### High Value (documented features):
5. Integrate ai-session `MessageBus` for agent coordination
6. Enable context compression (93% token savings)
7. Implement session persistence
8. Add session resume/fork/checkpoint
9. Call `VerificationAgent` after auto-create
10. Wire `HookSystem` into orchestrator

### Polish (improves UX):
11. Fix TUI async key handlers
12. Implement `DynamicSpawner` workload balancing
13. Add configurable verification checks
14. Implement workload metrics collection

---

## References

### Gap Analysis
- **Main**: `docs/analysis/00-capability-gap-analysis.md`
- **Architecture Comparison**: `docs/analysis/01-multi-agent-architecture-comparison.md`
- **Version History**: `docs/analysis/02-version-capability-gap-analysis.md`

### Documentation
- **Architecture**: `docs/ARCHITECTURE.md` (documents all v0.4.0 features)
- **Commands**: `docs/COMMANDS.md` (session resume, daemon commands)
- **Changelog**: `CHANGELOG.md` (v0.3.8 → v0.4.0 history)

### Codebase Locations
- **Hooks**: `crates/ccswarm/src/hooks/` (mod.rs, registry.rs, builtin.rs)
- **Verification**: `crates/ccswarm/src/orchestrator/verification.rs`
- **Spawner**: `crates/ccswarm/src/subagent/spawner.rs`
- **Parallel**: `crates/ccswarm/src/subagent/parallel_executor.rs`
- **ai-session**: `crates/ai-session/src/` (complete crate)

---

## Next Steps

When ready to work on v0.4.0 implementation:

1. **Start with Critical Path**: Fix the `start` command coordination loop
2. **Review code**: Read the implementation files listed above
3. **Check TODOs**: Each pattern has integration TODO checkboxes
4. **Test incrementally**: Don't try to integrate everything at once
5. **Reference gap analysis**: Understand what's broken and why

**For current v0.3.8 development**, see [CLAUDE.md](../CLAUDE.md) for working features and development standards.
