# Epic: Claude Code ACP Integration MVP

## Overview
Implement minimal functionality to integrate Claude Code and ccswarm via Agent Client Protocol (ACP), enabling task sending and result receiving.

## Goals
- [ ] Send tasks from ccswarm to Claude Code via ACP protocol
- [ ] Receive Claude Code execution results in ccswarm
- [ ] Don't break existing ccswarm functionality

## Timeline
**Target Period**: 1-2 weeks
- Week 1: Basic implementation (Day 1-5)
- Week 2: Integration and improvements (Day 6-10)

## Definition of Success
```bash
# This should work
ccswarm claude-acp start
ccswarm task "Create a simple TODO app" --via-acp
```

## Architecture
```
┌──────────────────┐    ACP/JSON-RPC    ┌─────────────────┐
│  ccswarm Master  │◄──────────────────►│  Claude Code    │
│  (ACP Client)    │                    │  (ACP Server)   │
└──────────────────┘                    └─────────────────┘
         └── localhost:9100 ──────────────┘
```

## Related Issues
- #1 Day 1: Project setup and dependency addition
- #2 Day 2-3: Claude Code ACP adapter implementation
- #3 Day 4-5: CLI command implementation
- #4 Day 6-7: Integration with existing system
- #5 Day 8: Error handling enhancement
- #6 Day 9: Unit tests and documentation
- #7 Day 10: Integration tests and demo

## Labels
- `epic`
- `claude-acp`
- `mvp`
- `enhancement`
