# ccswarm Project Overview

## Purpose
ccswarm is an AI Multi-Agent Orchestration System that coordinates specialized AI agents (Frontend, Backend, DevOps, QA) using a Master Claude coordinator. Built in Rust for performance and reliability.

## Tech Stack
- **Language**: Rust (1.70+)
- **Build System**: Cargo workspace with two crates (ccswarm and ai-session)
- **Architecture**: Microkernel with pluggable providers
- **Async Runtime**: Tokio
- **Key Dependencies**: serde, tokio, colored, portable-pty

## Key Features
- Multi-agent orchestration with specialized roles
- Native AI-session management (93% token savings)
- Cross-platform support (Linux, macOS)
- Model Context Protocol (MCP) support
- Git worktree isolation for agents
- Sangha collective intelligence
- Auto-create application generator

## Development Environment
- Platform: Darwin (macOS)
- Working directory: /Users/nwiizo/ghq/github.com/nwiizo/ccswarm
- Workspace structure:
  - `crates/ccswarm/` - Main orchestration system
  - `crates/ai-session/` - Terminal session management library