# Capability Gap Analysis: Documentation vs Reality

**Document Version:** 1.0
**Date:** 2026-02-01
**Analyzed Version:** 0.3.8 (current)
**Related Issue:** [#67 - Multi-Agent System Redesign](https://github.com/nwiizo/ccswarm/issues/67)

## Executive Summary

This report analyzes the **current state** of ccswarm v0.3.8, comparing documented capabilities against actual implementation. Significant gaps exist between what documentation promises and what the codebase delivers.

**Key Finding:** Core orchestration is non-functional. The `start` command exits immediately, agents spawn but die with the process, and there is no inter-process communication.

> **See Also:** [02-version-capability-gap-analysis.md](./02-version-capability-gap-analysis.md) for historical v0.3.8 plan analysis.

---

## 1. Documentation Claims vs Reality

### Feature Status Matrix

| Documented Feature | Documentation Source | Actual Status | Gap Severity |
|--------------------|---------------------|---------------|--------------|
| Multi-agent orchestration | CLAUDE.md, ARCHITECTURE.md | Agents spawn then die | Critical |
| `ccswarm start` daemon | CLI help, docs | Exits in ~1 second | Critical |
| ACP WebSocket to Claude Code | CLAUDE.md | Never used by agents | Critical |
| 93% token savings | APPLICATION_SPEC.md | Not implemented | Major |
| Quality review every 30s | ARCHITECTURE.md | No background loop | Major |
| Session persistence | APPLICATION_SPEC.md | All state in-memory | Major |
| Sangha democratic voting | CLAUDE.md | Creates records only | Major |
| Git worktree isolation | ARCHITECTURE.md | Created but unused | Moderate |
| TUI real-time monitoring | README.md | Broken key handlers | Moderate |
| ai-session integration | APPLICATION_SPEC.md | Crate exists, not integrated | Major |

---

## 2. Critical Architecture Issues

### 2.1 `ccswarm start` Exits Immediately

**Location:** `crates/ccswarm/src/orchestrator/proactive_master.rs:1201-1205`

| Aspect | Expected | Actual |
|--------|----------|--------|
| Behavior | Block indefinitely, run event loop | Returns `Ok(())` immediately |
| Signal handling | Listen for SIGTERM, Ctrl+C | None |
| Agent lifetime | Persist across commands | Die with process |

### 2.2 Two Disconnected Orchestration Systems

| Component | Location | Has Agents | Has Task Execution | Used By |
|-----------|----------|------------|-------------------|---------|
| `ProactiveMaster` | `orchestrator/proactive_master.rs` | Empty `Vec<String>` | No | `ccswarm start` |
| `ExecutionEngine` | `execution/task_executor.rs` | `AgentPool` (populated) | Yes | TUI, task commands |

**Impact:** `start` uses `ProactiveMaster` which is disconnected from `ExecutionEngine`.

### 2.3 No Inter-Process Communication

| Process A | Process B | Can Communicate? |
|-----------|-----------|-----------------|
| `ccswarm tui` | `ccswarm task create` | No |
| `ccswarm start` | `ccswarm status` | No |
| Any `ccswarm` | Any other `ccswarm` | No |

**Missing Infrastructure:**
- No Unix socket / TCP server
- No shared database (SQLite)
- No file-based task persistence
- No message queue

### 2.4 `--daemon` and `--port` Flags Ignored

**Location:** `crates/ccswarm/src/cli/mod.rs:1318`

| Flag | Documented Purpose | Implementation |
|------|-------------------|----------------|
| `--daemon` | Run in background | Unused parameter |
| `--port` | Status server port | Unused parameter |

---

## 3. Claude Integration Confusion

### Three Integration Paths

| Mode | Configuration | Implementation | Actually Used |
|------|--------------|----------------|---------------|
| CLI (Default) | None required | `Command::new("claude") -p ...` | Yes, by agents |
| HTTP API | `--use-real-api` | `POST api.anthropic.com/v1/messages` | Yes, optionally |
| ACP WebSocket | `ccswarm claude-acp` | WebSocket to `ws://localhost:9100` | No, isolated command |

**Issue:** Documentation suggests ACP is primary integration, but it's never used by agents.

---

## 4. TUI Status

### Key Handler Analysis

| Key | Purpose | Works? | Issue |
|-----|---------|--------|-------|
| Tab/BackTab | Switch tabs | Yes | Synchronous |
| Up/Down/j/k | Move selection | Yes | Synchronous |
| q | Quit | Yes | Synchronous |
| n | Create session | No | `clone_for_async()` loses state |
| t | Add task | No | `clone_for_async()` loses state |
| : | Command mode | No | `clone_for_async()` loses state |
| Enter | Activate | No | `clone_for_async()` loses state |

**Root Cause:** Async handlers clone app state, modify the clone, then discard it.

### Other TUI Issues

| Issue | Location | Description |
|-------|----------|-------------|
| Hardcoded agents | `tui/app.rs:470-495` | Displays fake agent list |
| TODO stubs | Throughout | `start_orchestrator()`, `stop_orchestrator()`, etc. do nothing |

---

## 5. Command Status (Verified)

### Working Commands

| Command | Notes |
|---------|-------|
| `--help` | CLI help |
| `init` | Creates `ccswarm.json` |
| `setup` | Interactive wizard |
| `tutorial` | Interactive tutorial |
| `doctor` | Error lookup + API check |
| `template list` | 15 hardcoded templates |

### Partially Working Commands

| Command | Issue |
|---------|-------|
| `auto-create` | Writes boilerplate, no actual agent execution |
| `health` | Checks files that don't exist |
| `tui` | Renders but async key handlers broken |
| `delegate analyze` | Returns hardcoded response |

### Broken Commands

| Command | Why Broken |
|---------|-----------|
| `start` | Exits immediately |
| `stop` | TODO stub, just prints message |
| `task *` | Queue dies with process |
| `session *` | No persistence |
| `sangha *` | No voting loop runs |
| `extend *` | Requires running agents |
| `proactive *` | No background loop |
| `claude-acp *` | Requires Claude Code on :9100 |

---

## 6. ai-session Crate Status

### Integration Gap

| Documented | Reality |
|------------|---------|
| Integrated with ccswarm orchestrator | Crate exists independently |
| 93% token savings active | Compression not wired |
| MCP HTTP server for agents | Server exists, not used |
| Multi-agent coordination bus | Bus exists, not connected |

### Crate Components (Exist but Unused)

| Component | Location | Purpose | Used by ccswarm? |
|-----------|----------|---------|------------------|
| `SessionManager` | `ai-session/src/core/` | PTY management | No |
| `ContextManager` | `ai-session/src/context/` | Token compression | No |
| `CoordinationBus` | `ai-session/src/coordination/` | Agent messaging | No |
| `MCP Server` | `ai-session/src/mcp/` | HTTP API | No |

---

## 7. Demo Scripts Analysis

### `sample/multi_agent_demo.sh`

| Step | What It Claims | What Actually Happens |
|------|---------------|----------------------|
| 1. Start daemon | Background orchestrator | Exits immediately |
| 2. Create tasks | Queue tasks for agents | Creates records in new process |
| 3. Monitor | Watch agents work | Nothing to monitor |
| 4. Cleanup | Stop daemon | Nothing running |

---

## 8. Gap Severity Summary

### Critical (Blocks All Usage)

| ID | Gap | Impact |
|----|-----|--------|
| C1 | `start` exits immediately | No orchestration |
| C2 | Dual orchestration systems | Agents spawn but unused |
| C3 | `--daemon` ignored | No background mode |
| C4 | No IPC | Commands isolated |

### Major (Feature Non-Functional)

| ID | Gap | Impact |
|----|-----|--------|
| M1 | ai-session not integrated | No 93% token savings |
| M2 | ACP isolated | WebSocket claims false |
| M3 | No persistence | State lost on exit |
| M4 | Quality review loop absent | No auto-review |
| M5 | Sangha voting stub | No democratic decisions |

### Moderate (Degraded UX)

| ID | Gap | Impact |
|----|-----|--------|
| L1 | TUI async handlers broken | Limited interactivity |
| L2 | Hardcoded TUI data | Display misleading |
| L3 | Worktrees unused | Created but wasted |

---

## 9. Documentation Requiring Updates

| Document | Required Changes |
|----------|-----------------|
| CLAUDE.md | Remove ACP auto-connect claims, clarify what actually works |
| ARCHITECTURE.md | Mark theoretical vs implemented components |
| APPLICATION_SPEC.md | Remove 93% token savings claim, clarify ai-session status |
| README.md | Reflect actual command capabilities |

---

## 10. Recommended Fix Priority

### Phase 1: Make Core Functional

1. Fix `start_coordination()` to block and run event loop
2. Unify `ProactiveMaster` and `ExecutionEngine`
3. Add shutdown signal handling

### Phase 2: Enable Multi-Process

4. Implement daemon mode with Unix socket
5. Add IPC for command communication
6. Add task persistence (SQLite or file-based)

### Phase 3: Deliver Promised Features

7. Wire ai-session integration
8. Enable token compression
9. Connect coordination bus
10. Fix TUI async handlers

---

## References

- [ccswarm Issue #67](https://github.com/nwiizo/ccswarm/issues/67)
- [ARCHITECTURE_GAPS.md](../../hotfixes/claude/docs/analysis/ARCHITECTURE_GAPS.md)
- [ISSUE_SUMMARY.md](../../hotfixes/claude/docs/analysis/ISSUE_SUMMARY.md)
- Source analysis: `crates/ccswarm/src/`
