# Multi-Agent Architecture Comparison: ACP/SDK vs Native SDK vs PTY/CLI

**Document Version:** 1.1
**Date:** 2026-02-01
**Changelog:** v1.1 - Fixed incorrect Native SDK limitations; clarified Claude Agent SDK vs Raw Anthropic API capabilities
**Related Issue:** [#67 - Multi-Agent System Redesign](https://github.com/nwiizo/ccswarm/issues/67)

## Executive Summary

This report analyzes three architectural approaches for multi-agent orchestration with Claude:
1. **ACP/SDK** - Agent Client Protocol via WebSocket/stdio with Claude Agent SDK
2. **Native SDK** - Direct Anthropic API via official SDKs
3. **PTY/CLI** - Process-based execution via Claude Code CLI

Each approach has distinct trade-offs for concurrency, session management, and integration complexity.

---

## Terminology Clarification

> **⚠️ Important:** This document distinguishes between two different "SDKs":
>
> | Term | Package | Capabilities |
> |------|---------|--------------|
> | **Claude Agent SDK** | `@anthropic-ai/claude-agent-sdk` | Full agentic framework with built-in tools (Bash, Read, Write, Search, LSP, Task), session management |
> | **Raw Anthropic API** | `anthropic-sdk-python`, HTTP | Stateless Messages API only, no built-in tools |
>
> The "Native SDK" section covers BOTH, as they have very different capabilities.

---

## 1. ACP/SDK Architecture (WebSocket/stdio)

### Overview

The Agent Client Protocol (ACP) is a standardized protocol for editor-agent communication, implemented by adapters like [zed-industries/claude-code-acp](https://github.com/zed-industries/claude-code-acp).

```
┌─────────────┐     ACP (stdio/WS)    ┌──────────────────┐
│  Orchestrator│◄────────────────────►│ claude-code-acp  │
│  (ccswarm)   │                      │    (adapter)     │
└─────────────┘                       └────────┬─────────┘
                                               │
                              ┌────────────────┼────────────────┐
                              ▼                ▼                ▼
                        ┌──────────┐    ┌──────────┐    ┌──────────┐
                        │ Session 1│    │ Session 2│    │ Session N│
                        │ (Agent A)│    │ (Agent B)│    │ (Agent N)│
                        └──────────┘    └──────────┘    └──────────┘
```

### Protocol Stack

| Layer | Protocol | Purpose |
|-------|----------|---------|
| Transport | stdio / WebSocket | Communication channel |
| Message | ACP (JSON-RPC style) | Editor ↔ Agent messages |
| Tools | MCP (Model Context Protocol) | Tool/resource management |
| AI | Claude Agent SDK | Claude API interaction |

### Capabilities

| Capability | Support | Notes |
|------------|---------|-------|
| Concurrent sessions | ✅ Yes | Each session has unique `sessionId` |
| Session isolation | ✅ Yes | Independent Query instance per session |
| Streaming responses | ✅ Yes | Native event streaming |
| Tool execution | ✅ Yes | Via MCP integration |
| Session resume | ⚠️ PARTIAL | Protocol supports, not all agents implement |
| Session fork | ⚠️ PARTIAL | Protocol supports, not all agents implement |
| Bidirectional comms | ✅ Yes | Real-time message exchange |
| Cancel mid-execution | ✅ Yes | `session/cancel` baseline, `$/cancel_request` PARTIAL |

> **Note:** Session resume/fork are marked PARTIAL in ACP spec. Agents must advertise capability support.

### Concurrency Model (claude-code-acp)

```typescript
// From zed-industries/claude-code-acp
class ClaudeAcpAgent {
  sessions: Record<SessionId, Session> = {};

  async newSession(sessionId: SessionId): Promise<Session> {
    const session: Session = {
      sessionId,
      query: new Query(),           // Independent Claude SDK instance
      inputStream: pushable(),      // Per-session input
      cancelled: false,
      permissionMode: 'default'
    };
    this.sessions[sessionId] = session;
    return session;
  }
}
```

### Limitations

| Limitation | Impact | Mitigation |
|------------|--------|------------|
| Adapter dependency | Requires claude-code-acp or similar | Use official adapter |
| Protocol complexity | Three-layer translation | Higher maintenance |
| Server requirement | ACP server must be running | Process management |
| Not native to Claude Code | Feature request pending ([#6686](https://github.com/anthropics/claude-code/issues/6686)) | Use third-party adapter |

### Best For

- IDE/editor integrations (Zed, Emacs, Neovim)
- Standardized multi-client access
- Real-time collaborative scenarios

---

## 2. Native SDK Architecture (Direct API)

### Overview

Direct use of Anthropic's official SDKs (`@anthropic-ai/claude-agent-sdk` or `anthropic-sdk-python`) for API access.

```
┌─────────────┐     HTTP/REST      ┌──────────────────┐
│  Orchestrator│◄─────────────────►│  Anthropic API   │
│  (ccswarm)   │                   │  (api.anthropic) │
└──────┬──────┘                    └──────────────────┘
       │
       │  Spawn multiple SDK clients
       ▼
┌──────────────────────────────────────────────────┐
│  ┌─────────┐  ┌─────────┐  ┌─────────┐          │
│  │Client 1 │  │Client 2 │  │Client N │          │
│  │(Agent A)│  │(Agent B)│  │(Agent N)│          │
│  └─────────┘  └─────────┘  └─────────┘          │
└──────────────────────────────────────────────────┘
```

### SDK Options

| SDK | Language | Type | Built-in Tools | Multi-Agent Support |
|-----|----------|------|----------------|---------------------|
| `@anthropic-ai/claude-agent-sdk` | TypeScript | Agent SDK | ✅ Bash, Read, Write, Search, LSP, Task | ✅ Full (session resume/fork) |
| `anthropic-sdk-python` | Python | API SDK | ❌ None (implement yourself) | ⚠️ Via batches (stateless) |
| `anthropic-rs` (unofficial) | Rust | API SDK | ❌ None (implement yourself) | ⚠️ Limited |

> **Critical Distinction:**
> - **Claude Agent SDK** (`@anthropic-ai/claude-agent-sdk`): Full agentic framework with:
>   - Built-in tools: `Bash` (terminal), `Read`/`Write`/`Search` (files), `LSP` (code intelligence), `Task` (subagents)
>   - Session management: `resume` and `forkSession` options
>   - Permission system with sandbox mode
> - **Anthropic Messages API** (raw SDK): Stateless HTTP API only
>   - No built-in tools - must implement all tool handlers yourself
>   - No session state - must pass full message history each request
>   - Best for: custom implementations, chat apps, or when full control needed

### Capabilities

| Capability | Claude Agent SDK | Raw Anthropic API | Notes |
|------------|------------------|-------------------|-------|
| Concurrent sessions | ✅ Yes | ✅ Yes | Multiple client instances |
| Session resume | ✅ Yes | ❌ No | Agent SDK `resume` option |
| Session fork | ✅ Yes | ❌ No | Agent SDK `forkSession` option |
| Streaming responses | ✅ Yes | ✅ Yes | SSE streaming |
| Tool execution | ✅ Built-in | ✅ DIY | Agent SDK: Bash, Read, Write, Search, LSP, Task |
| Terminal execution | ✅ Bash tool | ⚠️ Beta | `code-execution-2025-05-22` (sandboxed, limited) |
| File system access | ✅ Read/Write/Search | ❌ No | Agent SDK has native tools |
| Code intelligence | ✅ LSP tool | ❌ No | Go-to-definition, references, hover |
| Subagent spawning | ✅ Task tool | ❌ No | Spawn agents with model/tool restrictions |
| Message batches | N/A | ✅ Yes | Async bulk processing |
| Git worktree integration | ❌ Manual | ❌ Manual | Must implement separately |

### Concurrency Model

```python
# Python SDK - Multiple concurrent clients
import asyncio
from anthropic import AsyncAnthropic

async def run_agent(client, agent_config, task):
    return await client.messages.create(
        model="claude-sonnet-4-5-20250929",
        max_tokens=4096,
        system=agent_config.system_prompt,
        messages=[{"role": "user", "content": task}]
    )

async def orchestrate(tasks):
    client = AsyncAnthropic()
    agents = [run_agent(client, config, task) for config, task in tasks]
    return await asyncio.gather(*agents)
```

### Limitations

| Limitation | Claude Agent SDK | Raw Anthropic API | Mitigation |
|------------|------------------|-------------------|------------|
| File system access | ✅ Built-in (Read, Write, Search) | ❌ No | Use Agent SDK |
| Terminal execution | ✅ Built-in (Bash tool) | ⚠️ Beta (`code-execution-2025-05-22`) | Use Agent SDK |
| API costs | Subscription or PAYG | PAYG only (per-token) | Use subscription for predictable costs |
| Rate limits | Same caps | Same caps | Implement backoff |
| No native Rust SDK | TypeScript only | HTTP directly | Use unofficial crates |
| Git worktree integration | ❌ Manual | ❌ Manual | Implement separately |

> **Important:** Claude Agent SDK includes built-in tools: Bash, Read, Write, Search, LSP, Task, AskUserQuestion. Raw Anthropic API requires implementing all tools yourself.

### Best For

**Claude Agent SDK:**
- Full agentic workflows with file/terminal access
- Session resume and fork capabilities
- When Claude Code-like features needed without CLI

**Raw Anthropic API:**
- Custom agent implementations with full control
- API-first architectures
- Non-agentic chat applications
- Cost optimization (no SDK overhead)

---

## 3. PTY/CLI Architecture (Process-based)

### Overview

Spawn Claude Code CLI processes in pseudo-terminals for parallel execution.

```
┌─────────────┐
│  Orchestrator│
│  (ccswarm)   │
└──────┬──────┘
       │ spawn
       ▼
┌──────────────────────────────────────────────────┐
│  ┌─────────────┐  ┌─────────────┐  ┌───────────┐│
│  │ PTY 1       │  │ PTY 2       │  │ PTY N     ││
│  │ claude -p   │  │ claude -p   │  │ claude -p ││
│  │ (Agent A)   │  │ (Agent B)   │  │ (Agent N) ││
│  │ worktree/a  │  │ worktree/b  │  │ worktree/n││
│  └─────────────┘  └─────────────┘  └───────────┘│
└──────────────────────────────────────────────────┘
```

### Implementation (ai-session crate)

```rust
// From crates/ai-session/src/core/pty.rs
impl PtyHandle {
    pub async fn spawn_claude(
        &self,
        prompt: &str,
        working_dir: &Path,
        max_turns: Option<u32>,
    ) -> Result<()> {
        let mut cmd = CommandBuilder::new("claude");
        cmd.arg("--dangerously-skip-permissions")
           .arg("-p").arg(prompt)
           .arg("--output-format").arg("json");
        if let Some(turns) = max_turns {
            cmd.arg("--max-turns").arg(turns.to_string());
        }
        cmd.cwd(working_dir);
        self.spawn_command(cmd).await
    }
}
```

### Capabilities

| Capability | Support | Notes |
|------------|---------|-------|
| True parallelism | ✅ Yes | Independent OS processes |
| Process isolation | ✅ Yes | Full process boundaries |
| Git worktree per agent | ✅ Yes | Native cwd support |
| File system access | ✅ Yes | Full Claude Code capabilities |
| Terminal emulation | ✅ Yes | Real PTY via portable-pty |
| Session resume | ✅ Yes | `--resume <session-id>` |
| No server dependency | ✅ Yes | Just CLI binary |
| Streaming output | ✅ Yes | PTY read loop |

### Concurrency Model

```rust
// From crates/ccswarm/src/subagent/parallel_executor.rs
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

    // Collect results as they complete
    while let Some(result) = futures.next().await { ... }
}
```

### Limitations

| Limitation | Impact | Mitigation |
|------------|--------|------------|
| Process overhead | Memory per process | Limit concurrent agents |
| Startup latency | CLI boot time (~1-2s) | Session reuse |
| No bidirectional comms | Input only at start | Use --max-turns |
| Output parsing | Must parse stdout | --output-format json |
| Platform-specific | PTY differences | portable-pty abstraction |

### Best For

- Parallel task execution
- Git worktree isolation
- Serverless deployments
- When Claude Code features needed

---

## Comparative Analysis

### Feature Matrix

| Feature | ACP/SDK | Claude Agent SDK | Raw Anthropic API | PTY/CLI |
|---------|---------|------------------|-------------------|---------|
| Concurrent sessions | ✅ | ✅ | ✅ | ✅ |
| Bidirectional comms | ✅ | ✅ | ✅ | ❌ |
| File system access | ✅ (tools) | ✅ (Read/Write/Search) | ❌ | ✅ |
| Terminal execution | ✅ (tools) | ✅ (Bash tool) | ⚠️ Beta | ✅ |
| Code intelligence | ⚠️ | ✅ (LSP tool) | ❌ | ✅ |
| Subagent spawning | ✅ | ✅ (Task tool) | ❌ | ⚠️ |
| Git worktree native | ⚠️ | ❌ | ❌ | ✅ |
| No server required | ❌ | ✅ | ✅ | ✅ |
| Streaming | ✅ | ✅ | ✅ | ✅ |
| Session resume | ⚠️ PARTIAL | ✅ | ❌ | ✅ |
| Session fork | ⚠️ PARTIAL | ✅ | ❌ | ✅ |
| Cancel mid-run | ✅ | ✅ | ✅ | ⚠️ |
| Memory efficiency | ✅ | ✅ | ✅ | ❌ |
| Startup latency | Low | Low | Low | Medium |
| Complexity | High | Medium | Low | Low |

> **Legend:** ⚠️ PARTIAL = Protocol supports but not all agents implement fully
>
> **SDK Clarification:**
> - **Claude Agent SDK** (`@anthropic-ai/claude-agent-sdk`) = Full agentic SDK with built-in tools (Bash, Read, Write, Search, LSP, Task, AskUserQuestion)
> - **Raw Anthropic API** (`anthropic-sdk-python` or HTTP) = Stateless Messages API, no built-in tools, must implement tool handlers yourself

### Billing Model

| Aspect | ACP/SDK | Claude Agent SDK | Raw API | PTY/CLI |
|--------|---------|------------------|---------|---------|
| **Subscription** | ✅ | ✅ | ❌ | ✅ |
| **PAYG** | ✅ | ✅ | ✅ | ✅ |

> **Auth modes:**
> - **Subscription** (Pro/Max plan): `claude login` - works with ACP/SDK, Claude Agent SDK, PTY/CLI
> - **PAYG** (API key): `ANTHROPIC_API_KEY` env var - works with all approaches
> - Raw Anthropic API requires PAYG only (no subscription option)

### Resource Usage (Estimates - No Official Benchmarks)

| Metric | ACP/SDK | Native SDK | PTY/CLI |
|--------|---------|------------|---------|
| Memory per agent | ~50MB (shared)* | ~20MB (API client)* | ~200MB (process)* |
| Startup time | <100ms (connected)* | <100ms* | 1-2s* |
| Network overhead | WebSocket frames | HTTP requests | None (local) |
| CPU overhead | Low | Low | Medium |

> **⚠️ Note:** Memory and latency figures are estimates based on typical usage patterns. No official benchmarks available. Actual values depend on model, context size, and system configuration.

### Decision Matrix

| Use Case | Recommended | Reasoning |
|----------|-------------|-----------|
| IDE integration | ACP/SDK | Standardized protocol |
| Parallel file operations | PTY/CLI | Native FS access |
| Custom agent logic | Native SDK | Full control |
| CI/CD pipelines | PTY/CLI | No server needed |
| Real-time collaboration | ACP/SDK | Bidirectional |
| Cost optimization | Native SDK | Direct API, no wrappers |
| Git worktree isolation | PTY/CLI | Native cwd support |

---

## 4. Multi-Vendor Compatibility: ACP vs Tight Coupling

### The Vendor Lock-in Problem

Current ccswarm implementation is **tightly coupled to Claude Code CLI**:

```rust
// Tight coupling - Claude-specific
CommandBuilder::new("claude")
    .arg("--dangerously-skip-permissions")
    .arg("-p").arg(prompt)
```

This creates:
- **Vendor lock-in**: Can only use Claude Code
- **No fallback**: If Claude unavailable, system fails
- **Limited choice**: Users forced into single provider

### ACP: The "LSP for AI Agents"

ACP enables **vendor-agnostic orchestration**:

```
┌─────────────────────────────────────────────────────────────────┐
│                      ccswarm Orchestrator                        │
│                    (ACP Client Implementation)                   │
└───────────────────────────┬─────────────────────────────────────┘
                            │ ACP Protocol (JSON-RPC)
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│  Claude Code  │   │  Gemini CLI   │   │   OpenCode    │
│  (Anthropic)  │   │   (Google)    │   │    (SST)      │
└───────────────┘   └───────────────┘   └───────────────┘
        │                   │                   │
        ▼                   ▼                   ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│  Codex CLI    │   │  Qwen Code    │   │  Mistral Vibe │
│   (OpenAI)    │   │  (Alibaba)    │   │   (Mistral)   │
└───────────────┘   └───────────────┘   └───────────────┘
```

### Comparison: Tight Coupling vs ACP

| Aspect | PTY/CLI (Tight Coupling) | ACP (Unified Protocol) |
|--------|--------------------------|------------------------|
| **Vendor support** | Claude only | 20+ agents |
| **Switching cost** | High (code changes) | Low (config change) |
| **Fallback** | None | Automatic failover |
| **API consistency** | N/A | Standardized JSON-RPC |
| **MCP integration** | Per-agent impl | Protocol-level |
| **Editor support** | Manual | Zed, Neovim, JetBrains, Emacs |
| **Maintenance** | Per-vendor code | Single protocol impl |

### ACP Protocol Benefits

1. **Standardized Communication**
   ```json
   // Same message format for ANY agent
   {
     "jsonrpc": "2.0",
     "method": "prompt",
     "params": {
       "sessionId": "session-123",
       "content": "Implement user authentication"
     }
   }
   ```

2. **Unified Tool Interface (MCP)**
   - All ACP agents support MCP servers
   - Tools work across vendors
   - No per-agent tool implementation

3. **Session Portability**
   - Session state is protocol-level
   - Can migrate between agents
   - Resume with different provider

### Implementation Effort

| Approach | Initial Effort | Maintenance | Vendor Support |
|----------|---------------|-------------|----------------|
| Current (Claude PTY) | Done | Per-vendor | 1 |
| Add more PTY agents | Medium each | O(n) vendors | n |
| Implement ACP client | High once | O(1) | 20+ |

### Migration Strategy for ccswarm

**Phase 1: ACP Client Layer**
```rust
// Abstract agent interface
pub trait AgentClient {
    async fn create_session(&mut self) -> Result<SessionId>;
    async fn send_prompt(&self, session: SessionId, prompt: &str) -> Result<Response>;
    async fn cancel(&self, session: SessionId) -> Result<()>;
}

// ACP implementation works with ANY agent
pub struct AcpClient { /* ... */ }
impl AgentClient for AcpClient { /* ... */ }

// Legacy PTY for fallback
pub struct PtyClient { /* ... */ }
impl AgentClient for PtyClient { /* ... */ }
```

**Phase 2: Agent Selection**
```yaml
# ccswarm.yaml
agents:
  frontend:
    provider: gemini-cli      # Google
    fallback: claude-code     # Anthropic
  backend:
    provider: codex-cli       # OpenAI
    fallback: opencode        # SST (any LLM)
  devops:
    provider: qwen-code       # Alibaba
```

**Phase 3: Intelligent Routing**
- Route by task type to best-suited agent
- Automatic failover on errors
- Cost optimization across providers

### Recommendation

**Adopt ACP for multi-vendor support:**

| Action | Value | Complexity | Notes |
|--------|-------|------------|-------|
| Implement ACP client | High | High | Unlocks 20+ agents, requires protocol impl |
| Keep PTY as fallback | Medium | Low | Already working, minimal changes |
| Native SDK for API-only | Medium | Medium | Optional, for cost-sensitive workflows |

See also: [03-acp-agents-capability-comparison.md](./03-acp-agents-capability-comparison.md)

---

## Recommendations for ccswarm

### Current State (v0.4.x)
- PTY/CLI architecture defined, implementation in **draft branch**
- ACP code exists but uses CLI wrapper (not real ACP)
- No Native SDK integration

> **Note:** PTY/CLI has spec and architecture defined but implementation is not complete. Current branch (`fix/worktree-already-checked-out`) contains draft implementation.

### Recommended Architecture

**Hybrid approach:**

1. **Primary: PTY/CLI** for parallel task execution
   - True process isolation
   - Git worktree per agent
   - Full Claude Code capabilities

2. **Optional: ACP/SDK** for advanced orchestration
   - Real-time session management
   - Bidirectional communication
   - Use `@zed-industries/claude-code-acp` or similar

3. **Future: Native SDK** for API-only workflows
   - When Claude Code features not needed
   - Cost optimization
   - Custom agent logic

### Migration Path (Value/Complexity Assessment)

| Phase | Description | Value | Complexity | Status |
|-------|-------------|-------|------------|--------|
| 1 | PTY/CLI implementation | Baseline | Medium | **Draft** (spec/arch defined, impl in progress) |
| 2 | Add ACP via claude-code-acp | High | High | Not started (requires Node.js runtime) |
| 3 | Native SDK option | Medium | Medium | Not started (Phase 2 not required) |

---

## References

- [Agent Client Protocol - ACP](https://ai-sdk.dev/providers/community-providers/acp)
- [zed-industries/claude-code-acp](https://github.com/zed-industries/claude-code-acp)
- [Xuanwo/acp-claude-code](https://github.com/Xuanwo/acp-claude-code)
- [Claude Code Feature Request #6686](https://github.com/anthropics/claude-code/issues/6686)
- [Anthropic Claude Agent SDK](https://www.npmjs.com/package/@anthropic-ai/claude-agent-sdk)
- [ccswarm Issue #67](https://github.com/nwiizo/ccswarm/issues/67)
