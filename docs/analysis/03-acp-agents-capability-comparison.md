# ACP Agents Capability Comparison

**Document Version:** 1.3
**Date:** 2026-02-01
**Changelog:** v1.3 - Added ACP implementation details (SDK vs CLI, billing modes), updated models (GPT-5.2-Codex, Gemini 3), fixed Qwen Code capabilities
**Related Issue:** [#67 - Multi-Agent System Redesign](https://github.com/nwiizo/ccswarm/issues/67)

## Executive Summary

This report compares the capabilities of major ACP-compatible coding agents:
- **Claude Code** (Anthropic)
- **Codex CLI** (OpenAI)
- **Gemini CLI** (Google)
- **OpenCode** (SST)
- **Qwen Code** (Alibaba)

All agents implement the [Agent Client Protocol (ACP)](https://agentclientprotocol.com), enabling standardized orchestration and multi-agent workflows.

> **See Also:** [01-multi-agent-architecture-comparison.md](./01-multi-agent-architecture-comparison.md) for detailed comparison of ACP/SDK vs Native SDK vs PTY/CLI architectures, including terminology clarification on "Claude Agent SDK" vs "Raw Anthropic API".

---

## 1. Agent Overview

### Claude Code (Anthropic)

| Attribute | Value |
|-----------|-------|
| **Provider** | Anthropic |
| **Model** | Claude (Sonnet, Opus, Haiku) |
| **ACP Support** | Via [zed-industries/claude-code-acp](https://github.com/zed-industries/claude-code-acp) adapter |
| **MCP Support** | ✅ Native |
| **License** | Proprietary (CLI), Apache 2.0 (adapter) |
| **Installation** | `npm install -g @anthropic-ai/claude-code` |

**Key Features:**
- Task tool for spawning subagents
- Session fork and resume
- Multi-file editing
- Git worktree awareness
- Context compaction (auto at 98% usage)

**Unique Capabilities:**
- Hierarchical agent model (Main → Subagents via Task tool)
- Plan mode for structured implementation planning
- Custom plugin-defined agents
- Code review plugin with parallel review agents

### Codex CLI (OpenAI)

| Attribute | Value |
|-----------|-------|
| **Provider** | OpenAI |
| **Model** | GPT-5.2-Codex (xhigh reasoning, context compaction) |
| **ACP Support** | Via [zed-industries/codex-acp](https://github.com/zed-industries/codex-acp) adapter |
| **MCP Support** | ✅ Via `codex mcp` server mode |
| **License** | MIT |
| **Installation** | `npm install -g @openai/codex` |

**Key Features:**
- Streaming terminal output (non-PTY mode)
- @-mention for files and symbols
- Web fetch for context
- Managed installation in Zed

**Unique Capabilities:**
- MCP server mode (`codex mcp`) for multi-agent frameworks
- Integration with OpenAI Agents SDK
- GPT-5.2-Codex with xhigh reasoning effort and context compaction
- Agent skills (reusable instruction bundles)
- Image/screenshot support in CLI

**Limitations:**
- No editing past messages (in Zed)
- No thread history resume
- No checkpointing

### Gemini CLI (Google)

| Attribute | Value |
|-----------|-------|
| **Provider** | Google |
| **Model** | Gemini 3 Pro (thinking_level, Computer Use) |
| **ACP Support** | ✅ Native (`--experimental-acp` flag) |
| **MCP Support** | ✅ Native with OAuth for remote servers |
| **License** | Apache 2.0 |
| **Installation** | `npm install -g @google/gemini-cli` |

**Key Features:**
- Reference ACP implementation
- Rich MCP content (text, images, audio, binary)
- FastMCP integration
- Deferred tool discovery (>10% context = on-demand)
- OAuth 2.0 for remote MCP servers

**Unique Capabilities:**
- First ACP reference implementation
- HTTP/SSE MCP server support
- Tool name prefixing for multi-server conflicts
- 1M token context window (64K output)
- Computer Use tool support
- thinking_level parameter (low/high)

### OpenCode (SST)

| Attribute | Value |
|-----------|-------|
| **Provider** | SST (open source) |
| **Model** | Any LLM (OpenAI, Anthropic, Google, Groq, Azure, Bedrock, OpenRouter) |
| **ACP Support** | ✅ Native |
| **MCP Support** | ✅ Via @ai-sdk/mcp |
| **License** | MIT |
| **Installation** | `go install github.com/sst/opencode@latest` |

**Key Features:**
- Multi-provider LLM support
- TUI with Bubble Tea
- Vim-like editor
- SQLite session persistence
- LSP integration

**Unique Capabilities:**
- **Provider agnostic** - use any LLM
- Built-in agents: `build` (full access), `plan` (read-only)
- Agentic iteration limits for cost control
- NPM-installed dynamic providers

### Qwen Code (Alibaba)

| Attribute | Value |
|-----------|-------|
| **Provider** | Alibaba Cloud (QwenLM) |
| **Model** | Qwen3-Coder (480B MoE, 35B active) |
| **ACP Support** | ✅ Native (fork of Gemini CLI) |
| **MCP Support** | ✅ Via Qwen-Agent framework |
| **License** | Apache 2.0 |
| **Installation** | `npm install -g @qwen-code/qwen-code` |

**Key Features:**
- 256K native context (1M with extrapolation)
- Enhanced parser for Qwen-Coder
- Workflow automation (PR, rebase, formatting)
- Deep codebase comprehension

**Unique Capabilities:**
- State-of-the-art on Agentic Coding benchmarks
- Comparable to Claude Sonnet 4 on agentic tasks
- Free tier: 1M tokens (90-day trial)
- Sandboxed execution (Docker, Podman, macOS Seatbelt)
- Multi-provider support (Qwen, OpenAI, Anthropic, Gemini)
- Task tool for parallel subagent spawning

**Built-in Tools:**
- ShellTool, EditTool, ReadFileTool, WriteFileTool, ListFilesTool
- WebFetchTool, GrepTool, ReadManyFilesTool, TaskTool

---

## 2. Capability Matrix

### Core Features

| Feature | Claude Code | Codex CLI | Gemini CLI | OpenCode | Qwen Code |
|---------|-------------|-----------|------------|----------|-----------|
| **ACP Native** | ❌ (adapter) | ❌ (adapter) | ✅ | ✅ | ✅ |
| **MCP Support** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Multi-file Edit** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Terminal Exec** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Session Resume** | ⚠️ CLI only | ❌ | ✅ | ✅ | ✅ |
| **Session Fork** | ⚠️ CLI only | ❌ | ❌ | ⚠️ Partial | ❌ (resume only) |
| **Streaming** | ✅ | ✅ | ✅ | ✅ | ✅ |

> **Legend:** ⚠️ = Partial support, ❓ = Unverified
>
> **Notes:**
> - Claude Code: Adapter doesn't implement session resume/fork directly; relies on underlying CLI
> - Gemini CLI: Session fork is NOT supported (validated against codebase)
> - OpenCode: Fork via child sessions, full fork is feature request

### Model & Provider

| Aspect | Claude Code | Codex CLI | Gemini CLI | OpenCode | Qwen Code |
|--------|-------------|-----------|------------|----------|-----------|
| **Provider Lock** | Anthropic | OpenAI | Google | None | None (multi-provider) |
| **Model Choice** | Sonnet/Opus/Haiku | GPT-5 | Gemini 3 | Any | Qwen3-Coder |
| **Context Window** | 200K | Multi-context (compaction) | 1M (64K output) | Varies | 256K-1M |
| **Reasoning** | ✅ | ✅ (xhigh effort) | ✅ (thinking_level) | Varies | ✅ |

### Advanced Features

| Feature | Claude Code | Codex CLI | Gemini CLI | OpenCode | Qwen Code |
|---------|-------------|-----------|------------|----------|-----------|
| **Subagent Spawning** | ✅ Task tool | ✅ AgentControl | ✅ SubagentTool | ✅ Task tool | ✅ Task tool |
| **Parallel Agents** | ✅ | ✅ (depth=1) | ✅ | ✅ | ✅ |
| **Plugin System** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Sandboxed Exec** | ❌ | ❌ | ❌ | ❌ | ✅ (Docker/Podman/Seatbelt) |
| **OAuth MCP** | ❌ | ❌ | ✅ | ❌ | ❌ |
| **Cost Control** | ❌ | ❌ | ❌ | ✅ | ✅ (free tier) |

---

## 3. Detailed Comparison

### 3.1 ACP Implementation Quality

| Agent | Implementation | Maturity | Notes |
|-------|---------------|----------|-------|
| **Gemini CLI** | Reference impl | High | First ACP agent, Google partnership |
| **OpenCode** | Native | High | Built for ACP from start |
| **Qwen Code** | Native (fork) | Medium | Forked from Gemini CLI |
| **Claude Code** | Adapter | Medium | Via zed-industries adapter |
| **Codex CLI** | Adapter | Medium | Via zed-industries adapter |

### 3.2 ACP Implementation Details

| Agent | ACP Type | Internal | Billing via ACP | Transport |
|-------|----------|----------|-----------------|-----------|
| **Claude Code** | Adapter (`claude-code-acp`) | SDK (`@anthropic-ai/claude-agent-sdk`) | Sub (`/login`) or PAYG (API key) | stdio |
| **Codex CLI** | Adapter (`codex-acp`) | SDK (`codex-core`) | Sub + PAYG (ChatGPT or API key) | stdio |
| **Gemini CLI** | Native (`@agentclientprotocol/sdk`) | SDK | Sub + PAYG (Gmail/AI Studio/API) | stdio |
| **OpenCode** | Native | SDK | Via provider (Zen gateway or BYO) | stdio |
| **Qwen Code** | Native (Gemini fork) | SDK | PAYG only (DashScope API) | stdio |

> **Key Findings:**
> - **All use SDK internally**, none wrap CLI - enables full capability access
> - **Adapters (Claude, Codex)**: Require additional npm package, add translation layer
> - **Native (Gemini, OpenCode, Qwen)**: Direct ACP implementation, lower complexity
> - **Claude ACP adapter**: Uses local auth from `~/.claude/` (run `claude login` in terminal first), or `ANTHROPIC_API_KEY` for PAYG
> - **Codex ACP adapter**: Supports both subscription (ChatGPT browser login) and API key

### 3.3 MCP Server Support

| Agent | Transport | Auth | Features |
|-------|-----------|------|----------|
| **Gemini CLI** | stdio, HTTP, SSE | OAuth 2.0 | Rich content, deferred discovery |
| **OpenCode** | stdio | None | @ai-sdk/mcp integration |
| **Claude Code** | stdio, HTTP, SSE | OAuth | Per-project and global config |
| **Codex CLI** | stdio | None | MCP server mode available |
| **Qwen Code** | stdio | None | Via Qwen-Agent framework |

### 3.3 Multi-Agent Capabilities

| Agent | Subagents | Parallelism | Coordination |
|-------|-----------|-------------|--------------|
| **Claude Code** | ✅ Task tool | ✅ Parallel subagents | Hierarchical |
| **OpenCode** | ✅ Task tool | ✅ Parallel | Hierarchical |
| **Gemini CLI** | ✅ SubagentTool | ✅ Native | Hierarchical (delegation) |
| **Codex CLI** | ✅ AgentControl | ✅ Orchestrator-Worker | Hierarchical (depth=1) |
| **Qwen Code** | ✅ Task tool | ✅ Concurrent | Hierarchical |

> **Validation Note:** Qwen Code capabilities validated via [DeepWiki](https://deepwiki.com/QwenLM/qwen-code) and [official docs](https://qwenlm.github.io/qwen-code-docs/).

### 3.4 Pricing & Billing Models

| Agent | Subscription | PAYG | Free Tier | BYO Keys |
|-------|--------------|------|-----------|----------|
| **Claude Code** | ✅ | ✅ | ⚠️ Limited | ✅ |
| **Codex CLI** | ✅ | ✅ | ⚠️ Limited | ✅ |
| **Gemini CLI** | ✅ | ✅ | ✅ | ✅ |
| **OpenCode** | ✅ | ✅ | ✅ | ✅ |
| **Qwen Code** | ❌ | ✅ | ✅ | ✅ |

> **Notes:**
> - Claude/Codex: Subscription available, API key switches to PAYG
> - Gemini: Gmail auth (free), AI Studio Pro/Ultra (subscription), or API key (PAYG)
> - OpenCode: [Black subscription](https://opencode.ai/black), or BYO keys
> - Qwen: 1M tokens free (90-day trial, Singapore region)

---

## 4. Use Case Recommendations

### By Task Type

| Task | Recommended Agent | Reasoning |
|------|-------------------|-----------|
| Complex refactoring | Claude Code | Best subagent orchestration |
| Rapid prototyping | Gemini CLI | Large context, fast |
| Multi-provider flexibility | OpenCode or Qwen Code | Both work with multiple LLMs |
| Cost-sensitive projects | Gemini CLI, OpenCode, Qwen Code | Free tiers (Gemini Gmail, Qwen trial) or local LLMs (OpenCode+Ollama) |
| Reasoning-heavy tasks | Codex CLI | GPT-5 adaptive reasoning |

### By Team Size

| Team | Recommended | Reasoning |
|------|-------------|-----------|
| Solo developer | OpenCode | Provider flexibility, cost control |
| Small team | Gemini CLI | Good free tier, native ACP |
| Enterprise | Claude Code | Best tooling, reliability |
| Open source | Qwen Code | Apache license, free tier |

### By Orchestration Needs

| Need | Recommended | Reasoning |
|------|-------------|-----------|
| Custom orchestration | OpenCode or Qwen Code | Provider agnostic |
| Existing Claude infra | Claude Code | Consistency |
| Best subagent support | Claude Code or Gemini CLI | Native hierarchical delegation |

---

## 5. ccswarm Integration Strategy

### Current State
- PTY/CLI architecture defined, **implementation is draft**
- Tightly coupled to Claude Code CLI
- No multi-vendor support
- No ACP implementation

> **Note:** v0.4.x PTY/CLI is in draft branch with spec/architecture defined but implementation not complete.

### Recommended Approach

**Phase 1: ACP Client Implementation**

```rust
pub struct AcpAgentConfig {
    pub name: String,
    pub command: String,        // "gemini", "claude", "opencode", etc.
    pub acp_flag: Option<String>, // "--experimental-acp" for Gemini
    pub transport: AcpTransport,  // Stdio or WebSocket
}

pub enum AcpTransport {
    Stdio,
    WebSocket { url: String },
}
```

**Phase 2: Agent Registry**

```yaml
# ~/.ccswarm/agents.yaml
agents:
  claude-code:
    command: claude
    adapter: "@zed-industries/claude-code-acp"
    features: [subagents, session-fork, plugins]

  gemini-cli:
    command: gemini
    flags: ["--experimental-acp"]
    features: [native-acp, oauth-mcp, large-context]

  opencode:
    command: opencode
    features: [multi-provider, cost-control, native-acp]

  codex-cli:
    command: codex
    adapter: "@zed-industries/codex-acp"
    features: [adaptive-reasoning, mcp-server]

  qwen-code:
    command: qwen-code
    features: [free-tier, code-interpreter, native-acp]
```

**Phase 3: Intelligent Routing**

```rust
impl AgentRouter {
    pub fn select_agent(&self, task: &Task) -> &AgentConfig {
        match task.requirements {
            // Complex multi-step → Claude (best subagents)
            Requirements::MultiStep => &self.agents["claude-code"],

            // Large codebase → Gemini (1M context)
            Requirements::LargeContext => &self.agents["gemini-cli"],

            // Cost sensitive → Qwen (free tier)
            Requirements::CostOptimized => &self.agents["qwen-code"],

            // Custom provider → OpenCode
            Requirements::CustomProvider(_) => &self.agents["opencode"],

            // Reasoning heavy → Codex (GPT-5 adaptive)
            Requirements::DeepReasoning => &self.agents["codex-cli"],

            _ => &self.default_agent,
        }
    }
}
```

### Agent Integration Assessment for ccswarm

| Agent | Integration Value | Integration Complexity | Notes |
|-------|-------------------|------------------------|-------|
| Gemini CLI | High | Low | Reference ACP, native support, good free tier |
| OpenCode | High | Low | Provider agnostic, native ACP, cost control |
| Claude Code | Medium | Medium | Current compatibility, requires adapter |
| Qwen Code | Medium | Low | Free tier, native ACP, good performance |
| Codex CLI | Medium | Medium | Reasoning models, requires adapter |

---

## 6. Conclusion

### Key Findings

1. **ACP is the future** - All major agents support or are adopting ACP
2. **Native ACP preferred** - Gemini, OpenCode, Qwen have native support (lower complexity)
3. **Adapters work well** - Claude and Codex adapters are production-ready
4. **OpenCode & Qwen Code are provider-agnostic** - Both support multiple LLM providers
5. **Qwen has generous free tier** - 1M tokens (90-day trial, regional)

### Actions Assessment for ccswarm

| Action | Value | Complexity | Dependencies |
|--------|-------|------------|--------------|
| Complete PTY/CLI draft | High | Medium | Finish current draft impl |
| Implement ACP client | High | High | Protocol implementation |
| Add Gemini CLI support | High | Low | ACP client |
| Add OpenCode support | High | Low | ACP client |
| Keep PTY/CLI as fallback | Medium | Low | When ACP unavailable |
| Keep ACP as fallback | Medium | Medium | Requires external bridge |
| Add Qwen Code support | Medium | Low | ACP client |
| Add Codex CLI support | Medium | Medium | ACP client + adapter |

---

## References

- [Agent Client Protocol](https://agentclientprotocol.com)
- [ACP Agents List](https://agentclientprotocol.com/get-started/agents)
- [Zed ACP Documentation](https://zed.dev/acp)
- [Gemini CLI MCP Servers](https://geminicli.com/docs/tools/mcp-server/)
- [OpenCode Documentation](https://opencode.ai/docs/)
- [Qwen3-Coder Announcement](https://qwenlm.github.io/blog/qwen3-coder/)
- [Codex in Zed Blog](https://zed.dev/blog/codex-is-live-in-zed)
- [JetBrains ACP Registry](https://blog.jetbrains.com/ai/2026/01/acp-agent-registry/)
- [ACP Progress Report - Zed](https://zed.dev/blog/acp-progress-report)
