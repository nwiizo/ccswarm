# Release Notes v0.3.7

## 🎉 ccswarm v0.3.7 - Search Agent & Enhanced Communication

### Release Date: 2025-06-26

## Overview

This release introduces the **Search Agent**, a new specialized agent that brings web search capabilities to ccswarm. The Search Agent integrates with Gemini CLI to help other agents and Master Claude find information from the web, enabling research-driven development and informed decision-making.

## ✨ Key Features

### 1. **Search Agent Implementation**
- New specialized agent for web search using Gemini CLI
- Seamless integration with the coordination bus for inter-agent communication
- Support for filtered searches:
  - Domain-specific searches
  - Date range filtering
  - Language preferences
  - File type filtering
- Intelligent search result parsing with relevance scoring
- Request/response message protocol for agent collaboration

### 2. **Search Agent Sangha Participation**
- Autonomous decision-making capabilities for search agents
- Intelligent proposal analysis using web search results
- Evidence-based voting with supporting search data
- Knowledge gap detection and initiative proposals
- Full integration with Sangha collective intelligence system

### 3. **Enhanced Agent Communication**
- Improved inter-agent messaging system with two-layer architecture:
  - **ccswarm layer**: File-based persistence with async channels
  - **ai-session layer**: High-performance message bus with crossbeam
- **AICoordinationBridge** for seamless integration between layers
- Low-latency coordination (<100ms)
- Automatic message persistence and recovery

### 4. **Backend Agent Status Reporting**
- Comprehensive status reporting for backend agents
- Real-time monitoring of:
  - API endpoint health and response times
  - Database connection status and pool metrics
  - Server resource usage (CPU, memory)
  - Active services and their dependencies
  - Recent API call history
- Health assessment based on multiple factors
- Enhanced CLI status command with backend-specific information

## 🔧 Technical Improvements

### Architecture Updates
- Updated architecture documentation to include Search Agent
- Enhanced coordination bus with new message types for search requests
- Improved agent role system with Search Agent boundaries
- Refined Sangha participation for automated research
- Added specialized status reporting for backend agents

### Bug Fixes
- Fixed agent communication synchronization issues
- Resolved message persistence timing in coordination bus
- Fixed identity boundary enforcement for new agent types

## 📖 Documentation

New documentation added:
- `docs/SEARCH_AGENT.md` - Complete guide to Search Agent usage
- `docs/SEARCH_AGENT_SANGHA.md` - Search Agent's Sangha participation
- `examples/search_agent_example.rs` - Basic usage example
- `examples/search_agent_sangha_demo.rs` - Sangha integration demo
- `examples/backend_status_demo.rs` - Backend status monitoring demo
- `BACKEND_STATUS_IMPLEMENTATION.md` - Implementation details for backend status

## 🚀 Getting Started with New Features

### Backend Status Monitoring
```bash
# View status for all agents
ccswarm status

# View detailed backend agent status
ccswarm status --agent backend-agent-id --detailed

# Backend agents will show:
# - API Health: 95.2% (19/20 endpoints)
# - Database: Connected (PostgreSQL)
# - Server: 256.5MB RAM, 15.3% CPU
# - Active Services: 3
# - Recent API Calls: 42
```

### Search Agent Basic Usage
```rust
// Create search agent
let mut search_agent = SearchAgent::new("search-agent-001".to_string(), coordination_bus.clone());

// Initialize
search_agent.initialize().await?;

// Send search request via coordination bus
let request = SearchRequest {
    requesting_agent: "frontend-agent".to_string(),
    query: "React hooks best practices".to_string(),
    max_results: Some(10),
    filters: Some(SearchFilters {
        domains: Some(vec!["reactjs.org".to_string()]),
        date_range: Some("past month".to_string()),
        language: Some("en".to_string()),
        file_type: None,
    }),
    context: Some("Need current best practices for hooks".to_string()),
};
```

### Sangha Participation
```rust
// Enable Sangha participation
search_agent.enable_sangha_participation();

// Start monitoring proposals
search_agent.start_sangha_monitoring(sangha).await?;
```

## 📊 Performance

- **Communication latency**: <100ms between agents
- **Message persistence**: Automatic with 1000 message history
- **Search response time**: Depends on Gemini CLI and query complexity
- **Memory efficiency**: Optimized message bus with concurrent access

## ⚡ Migration Guide

This release is backward compatible. To use the new Search Agent:

1. Ensure Gemini CLI is installed and available in PATH
2. Add Search Agent to your agent configuration
3. Enable Search Agent in your orchestration setup

## 🙏 Acknowledgments

Special thanks to all contributors who helped make this release possible.

---

For complete details, see the [CHANGELOG.md](CHANGELOG.md) and updated documentation.