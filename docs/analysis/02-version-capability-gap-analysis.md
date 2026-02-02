# Capability Gap & Trade-off Analysis: v0.3.8 (Plan) vs v0.3.8 (Reality) vs v0.4.x (New)

**Document Version:** 1.1
**Date:** 2026-02-01
**Changelog:** v1.1 - Added cross-reference to architecture comparison doc
**Related Issue:** [#67 - Multi-Agent System Redesign](https://github.com/nwiizo/ccswarm/issues/67)

## Executive Summary

This report analyzes the capability gaps between:
1. **v0.3.8 Original Plan** - What the architecture documentation promised
2. **v0.3.8 As Implemented** - What was actually built
3. **v0.4.x New Design** - PTY-based architecture (draft implementation)

Key finding: Significant documentation-implementation gap in v0.3.8 led to architectural pivot in v0.4.x.

> **See Also:** [01-multi-agent-architecture-comparison.md](./01-multi-agent-architecture-comparison.md) for terminology clarification on "Claude Agent SDK" vs "Raw Anthropic API".

---

## 1. v0.3.8 Original Plan (INTENDED)

### Documented Architecture

Based on CLAUDE.md and ARCHITECTURE.md documentation:

```
┌─────────────────────────────────────────────────────────┐
│                    Master Claude (Orchestrator)          │
│  - Task analysis and delegation                          │
│  - Quality review coordination                           │
│  - Sangha consensus management                           │
└─────────────────────┬───────────────────────────────────┘
                      │
        ┌─────────────┴─────────────┬─────────────┐
        │                           │             │
┌───────▼────────┐  ┌──────────────▼──┐  ┌──────▼─────┐
│Frontend Agent  │  │Backend Agent    │  │DevOps Agent│
│  WebSocket     │  │  WebSocket      │  │  WebSocket │
└────────────────┘  └─────────────────┘  └────────────┘
        │                           │             │
        └───────────────────────────┴─────────────┘
                      │
┌─────────────────────▼───────────────────────────────────┐
│              ACP WebSocket Server (port 9100)            │
│  - Session management                                    │
│  - Message routing                                       │
│  - Multi-agent coordination                              │
└─────────────────────────────────────────────────────────┘
```

### Promised Capabilities

| Feature | Documentation Claim | Source |
|---------|---------------------|--------|
| **ACP WebSocket** | "WebSocket URL for Claude Code (default: ws://localhost:9100)" | config.rs |
| **Multi-session** | "Session management for each agent" | ARCHITECTURE.md |
| **Retry logic** | "connect_with_retry" with configurable attempts | config.rs |
| **Sangha system** | "Democratic decision-making" | CLAUDE.md |
| **Self-extension** | "Agents autonomously propose new capabilities" | CLAUDE.md |
| **Quality judge** | "Evaluates code on 8 dimensions" | ARCHITECTURE.md |
| **93% token savings** | "Intelligent session reuse and context compression" | APPLICATION_SPEC.md |

### ACP Configuration (Planned)

```rust
// From v0.3.8 config.rs
pub struct ClaudeACPConfig {
    pub url: String,              // "ws://localhost:9100"
    pub auto_connect: bool,       // true
    pub timeout: u64,             // 30 seconds
    pub max_retries: u32,         // 3 attempts
    pub retry_delay: u64,         // 2 seconds
    pub prefer_claude: bool,      // true
}
```

---

## 2. v0.3.8 As Implemented (REALITY)

### Actual Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Master Claude (Orchestrator)          │
│  - Basic task parsing                                    │
│  - Sequential delegation                                 │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│              SimplifiedClaudeAdapter                     │
│  - Single WebSocket connection                           │
│  - Single session_id                                     │
│  - Mutex-locked serial access                            │
└─────────────────────────────────────────────────────────┘
                      │
                      ▼ (assumed server at port 9100)
                      ?
              [No actual server implementation found]
```

### Implementation Analysis

**ACP Adapter (v0.3.8 adapter.rs):**

```rust
pub struct SimplifiedClaudeAdapter {
    // Single WebSocket - not multi-session capable
    ws_stream: Option<Arc<Mutex<WebSocketStream<...>>>>,

    // Single session - not per-agent
    session_id: Option<String>,

    config: ClaudeACPConfig,
    connected: bool,
}

// Serial message handling - blocks on Mutex
pub async fn send_task(&self, task: &str) -> ACPResult<String> {
    let ws_stream = self.ws_stream.as_ref().ok_or(ACPError::NotConnected)?;
    let mut stream = ws_stream.lock().await;  // Blocks other tasks
    stream.send(message).await?;
    // Wait for next message - no correlation
    if let Some(Ok(Message::Text(response))) = stream.next().await {
        return Ok(response);
    }
}
```

### Gap Analysis: Plan vs Reality

| Feature | Planned | Implemented | Gap |
|---------|---------|-------------|-----|
| **WebSocket connection** | ✅ | ✅ | None |
| **Multi-session support** | ✅ | ❌ Single session | Critical |
| **Concurrent agents** | ✅ | ❌ Serial only | Critical |
| **Message correlation** | ✅ (JSON-RPC id) | ❌ Sequential | Major |
| **Retry logic** | ✅ | ✅ | None |
| **Session per agent** | ✅ | ❌ Shared | Critical |
| **ACP server** | Assumed | Not found | Critical |
| **Sangha consensus** | ✅ | ⚠️ Code exists, unused | Major |
| **Self-extension** | ✅ | ⚠️ Code exists, unused | Major |
| **Quality judge** | ✅ | ⚠️ Partial | Moderate |
| **Token compression** | ✅ | ❌ Not implemented | Major |

### Root Causes

1. **No ACP server**: Claude Code doesn't expose WebSocket API on port 9100
2. **MVP shortcuts**: "Simplified" adapter was proof-of-concept, never extended
3. **Single session design**: Not architected for multi-agent from start
4. **Unused modules**: Sangha, self-extension built but not integrated

### Issue #67 Summary

From the GitHub issue:
> "現在は複数の「エージェント」と称しているが、実態は同じClaude Codeへの異なるプロンプトでの呼び出し"
> (Multiple agents are named, but actually invoke the same Claude Code with different prompts)

**Identified problems:**
1. Single AI dependency - all "agents" share one connection
2. Sequential execution - no parallelism
3. No state sharing - context doesn't transfer

---

## 3. v0.4.x New Design (CURRENT)

### Architecture Pivot

```
┌─────────────────────────────────────────────────────────┐
│                    ccswarm Orchestrator                  │
│  - ParallelExecutor with Semaphore                      │
│  - DynamicSpawner + WorkloadBalancer                    │
└─────────────────────┬───────────────────────────────────┘
                      │
        ┌─────────────┼─────────────┬─────────────┐
        │             │             │             │
        ▼             ▼             ▼             ▼
┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐
│  PTY 1    │  │  PTY 2    │  │  PTY 3    │  │  PTY N    │
│  claude   │  │  claude   │  │  claude   │  │  claude   │
│  -p task1 │  │  -p task2 │  │  -p task3 │  │  -p taskN │
│ worktree/a│  │ worktree/b│  │ worktree/c│  │ worktree/n│
└───────────┘  └───────────┘  └───────────┘  └───────────┘
      │             │             │             │
      └─────────────┴─────────────┴─────────────┘
                      │
              ai-session crate
         (PTY management, coordination)
```

### New Components (v0.4.x)

| Component | File | Purpose |
|-----------|------|---------|
| `PtyHandle` | `ai-session/src/core/pty.rs` | Native PTY terminal emulation |
| `ParallelExecutor` | `subagent/parallel_executor.rs` | FuturesUnordered task batching |
| `DynamicSpawner` | `subagent/spawner.rs` | Task-time agent generation |
| `WorkloadBalancer` | `subagent/workload_balancer.rs` | Task distribution |
| `SessionFork` | `session/fork.rs` | Session branching |
| `Checkpoint` | `session/checkpoint.rs` | State snapshots |

### Implementation

```rust
// Parallel execution with PTY (v0.4.x)
pub async fn execute_with_claude_pty(
    &self,
    tasks: Vec<SpawnTask>,
    working_dir: Option<PathBuf>,
    max_turns: Option<u32>,
) -> SubagentResult<ParallelExecutionResult> {
    let mut futures = FuturesUnordered::new();

    for task in tasks {
        let semaphore = Arc::clone(&self.semaphore);
        let future = async move {
            let _permit = semaphore.acquire().await;  // Concurrency control
            let pty = PtyHandle::new(24, 80)?;
            pty.spawn_claude_and_wait(&prompt, &work_dir, max_turns, timeout).await
        };
        futures.push(future);
    }

    // True parallel execution
    while let Some(result) = futures.next().await { ... }
}
```

### ACP Status (v0.4.x)

The ACP adapter was **replaced with CLI wrapper**:

```rust
// Current adapter.rs (v0.4.x)
//! This adapter executes `claude` CLI directly instead of WebSocket connection.

pub async fn send_task(&self, task: &str) -> ACPResult<String> {
    let mut cmd = Command::new("claude");
    cmd.arg("-p").arg(task)
       .arg("--output-format").arg("text")
       .arg("--dangerously-skip-permissions");
    cmd.output().await  // CLI execution, not WebSocket
}
```

---

## 4. Comparative Analysis

### Feature Comparison

| Feature | v0.3.8 Plan | v0.3.8 Reality | v0.4.x |
|---------|-------------|----------------|--------|
| **Multi-agent execution** | WebSocket sessions | ❌ Single serial | ✅ PTY parallel |
| **Concurrency model** | Session multiplexing | ❌ Mutex lock | ✅ Semaphore + FuturesUnordered |
| **Agent isolation** | Session ID | ❌ Shared | ✅ Process boundaries |
| **Git worktree** | Not specified | ❌ | ✅ Native cwd |
| **Session resume** | Assumed | ❌ | ✅ --resume flag |
| **Token compression** | Claimed 93% | ❌ | ⚠️ Partial (context module) |
| **ACP protocol** | Planned | ⚠️ Broken | ❌ Replaced with CLI |
| **Bidirectional comms** | Planned | ❌ | ❌ Input at start only |
| **Real-time streaming** | Planned | ❌ | ✅ PTY read loop |

### Trade-off Analysis

#### v0.3.8 Plan → v0.3.8 Reality

| Lost Capability | Impact | Root Cause |
|-----------------|--------|------------|
| Multi-session | Critical | MVP design, no server |
| Parallelism | Critical | Mutex serialization |
| ACP compliance | Major | Client incomplete (single session), requires external ACP bridge |
| Sangha integration | Moderate | Built but not wired |

#### v0.3.8 Reality → v0.4.x

| Trade-off | Gained | Lost |
|-----------|--------|------|
| Architecture | True parallelism | Bidirectional comms |
| Protocol | Simplicity (CLI) | WebSocket features |
| Resources | Process isolation | Memory efficiency |
| Complexity | Lower maintenance | Real-time session mgmt |

### Capability Matrix

```
                    Plan    Reality   v0.4.x
                    v0.3.8  v0.3.8
Parallel Agents     [====]  [    ]    [====]
Session Management  [====]  [=   ]    [==  ]
Git Worktree        [    ]  [    ]    [====]
Token Compression   [====]  [    ]    [==  ]
Bidirectional       [====]  [    ]    [    ]
Process Isolation   [    ]  [    ]    [====]
No Server Needed    [    ]  [    ]    [====]
Real-time Stream    [====]  [    ]    [====]
```

---

## 5. Gap Remediation Options

### Option A: Complete PTY/CLI Implementation (Current Path)

**Effort:** Medium (draft exists, needs completion)
**Risk:** Low

Complete v0.4.x draft implementation, then enhance:
- Finish parallel executor implementation
- Add proper session resume integration
- Implement context compression in ai-session
- Build coordination bus for agent communication

```
Pros:
✅ Architecture defined
✅ No server dependency
✅ Git worktree native
✅ Lower complexity

Cons:
⚠️ Implementation incomplete (draft)
❌ No bidirectional comms
❌ Higher memory usage
❌ No real-time session management
```

### Option B: Implement Real ACP

**Effort:** High
**Risk:** Medium

Integrate `@zed-industries/claude-code-acp` or similar:
- Add TypeScript/Node dependency
- Implement ACP message handling
- Build session multiplexing

```
Pros:
✅ Proper protocol compliance
✅ Bidirectional communication
✅ Memory efficient
✅ Real-time session management

Cons:
❌ Additional runtime dependency
❌ Higher complexity
❌ Server process management
```

### Option C: Hybrid Approach (Recommended)

**Effort:** Medium
**Risk:** Low

```
┌─────────────────────────────────────────────────────────┐
│                    ccswarm Orchestrator                  │
└─────────────────────┬───────────────────────────────────┘
                      │
        ┌─────────────┴─────────────┐
        │                           │
        ▼                           ▼
┌───────────────────┐      ┌───────────────────┐
│  PTY/CLI Engine   │      │  ACP Adapter      │
│  (default)        │      │  (optional)       │
│                   │      │                   │
│  - Parallel tasks │      │  - Real-time ops  │
│  - Git worktrees  │      │  - Bidirectional  │
│  - Isolation      │      │  - Session mgmt   │
└───────────────────┘      └───────────────────┘
```

Phase 1: Stabilize PTY/CLI (current)
Phase 2: Add optional ACP for advanced use cases
Phase 3: Allow per-workflow engine selection

---

## 6. Recommendations (Value/Complexity Assessment)

### Gap Remediation Actions

| Action | Value | Complexity | Category |
|--------|-------|------------|----------|
| Document actual capabilities | High | Low | Documentation |
| Remove misleading ACP config | Medium | Low | Cleanup |
| Add capability flags | Medium | Low | Documentation |
| Enhance ai-session compression | High | Medium | Feature |
| Session resume integration | Medium | Medium | Feature |
| Agent coordination (MessageBus) | High | Medium | Feature |
| Optional ACP support | High | High | Architecture |
| Engine selection (PTY vs ACP) | Medium | Medium | Feature |
| Execution metrics | Low | Low | Observability |

### Documentation Updates

| Document | Required Change | Value | Complexity |
|----------|-----------------|-------|------------|
| CLAUDE.md | Remove WebSocket claims, document PTY approach | High | Low |
| ARCHITECTURE.md | Update diagrams for v0.4.x | Medium | Low |
| APPLICATION_SPEC.md | Clarify actual token savings | Medium | Low |
| README.md | Reflect current capabilities | High | Low |

---

## 7. Conclusion

### Key Findings

1. **v0.3.8 had significant documentation-implementation gap** - WebSocket ACP was never fully functional
2. **v0.4.x pivot to PTY/CLI was pragmatic** - Achieved parallelism without server dependency
3. **Trade-offs are acceptable** - Lost bidirectional comms, gained true isolation

### Success Metrics for v0.4.x

| Metric | Target | Current |
|--------|--------|---------|
| Parallel agent execution | ✅ | ⚠️ Draft (arch defined, impl in progress) |
| Git worktree isolation | ✅ | ⚠️ Draft (arch defined, impl in progress) |
| Session resume | ✅ | ⚠️ Partial |

> **Note:** v0.4.x PTY/CLI implementation is in draft branch. Architecture and spec are defined, but implementation is not production-ready.

| Metric | Target | Current |
|--------|--------|---------|
| Documentation accuracy | ✅ | ⚠️ Needs update |
| Test coverage | 85% | ~70% |

### Final Recommendation

**Continue with Option C (Hybrid)** - PTY/CLI as primary engine, optional ACP for advanced scenarios. Prioritize documentation alignment and stability over new features.

---

## References

- [ccswarm Issue #67](https://github.com/nwiizo/ccswarm/issues/67)
- [zed-industries/claude-code-acp](https://github.com/zed-industries/claude-code-acp)
- [Claude Code Feature Request #6686](https://github.com/anthropics/claude-code/issues/6686)
- v0.3.8 source: `git show dacef5d:crates/ccswarm/src/acp_claude/`
- v0.4.x source: `crates/ccswarm/src/subagent/parallel_executor.rs`
