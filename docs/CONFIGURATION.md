# Configuration Reference

Complete guide to configuring ccswarm for your project.

## Quick Start

```bash
# Interactive setup (recommended)
ccswarm setup

# Or quick initialization
ccswarm init --name "MyProject" --agents frontend,backend,devops
```

## Configuration File

ccswarm uses `ccswarm.json` in your project root.

### Full Example

```json
{
  "project": {
    "name": "MyProject",
    "master_claude": {
      "role": "technical_lead",
      "quality_threshold": 0.85,
      "think_mode": "ultra_think",
      "permission_level": "supervised",
      "enable_proactive_mode": true,
      "proactive_frequency": 30,
      "high_frequency": 15,
      "claude_config": {
        "model": "claude-3.5-sonnet",
        "dangerous_skip": true,
        "think_mode": "ultra_think"
      }
    }
  },
  "agents": [
    {
      "name": "frontend-specialist",
      "role": "Frontend",
      "provider": "claude_code",
      "claude_config": {
        "model": "claude-3.5-sonnet",
        "dangerous_skip": true,
        "think_mode": "think_hard"
      },
      "auto_accept": {
        "enabled": true,
        "risk_threshold": 5
      }
    },
    {
      "name": "backend-specialist",
      "role": "Backend",
      "provider": "claude_code",
      "claude_config": {
        "model": "claude-3.5-sonnet"
      }
    }
  ],
  "coordination": {
    "method": "JSON_FILES",
    "delegation_strategy": "Hybrid"
  },
  "session_management": {
    "persistent_sessions": true,
    "max_sessions_per_role": 3
  }
}
```

## Configuration Sections

### Project Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | required | Project name |
| `master_claude.role` | string | `technical_lead` | Orchestrator role |
| `master_claude.quality_threshold` | float | `0.85` | Minimum quality score (0.0-1.0) |
| `master_claude.think_mode` | string | `ultra_think` | Thinking mode for Master Claude |
| `master_claude.enable_proactive_mode` | bool | `true` | Enable proactive task analysis |
| `master_claude.proactive_frequency` | int | `30` | Proactive analysis interval (seconds) |

### Agent Configuration

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | string | required | Unique agent identifier |
| `role` | string | required | `Frontend`, `Backend`, `DevOps`, `QA`, `Search` |
| `provider` | string | `claude_code` | Provider type |
| `auto_accept.enabled` | bool | `false` | Enable auto-accept mode |
| `auto_accept.risk_threshold` | int | `5` | Risk threshold (1-10) |

### Coordination Settings

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `method` | string | `JSON_FILES` | Communication method |
| `delegation_strategy` | string | `Hybrid` | Task delegation strategy |

**Delegation Strategies:**
- `ContentBased`: Keyword matching
- `LoadBalanced`: Workload distribution
- `ExpertiseBased`: Historical performance
- `WorkflowBased`: Task dependencies
- `Hybrid`: Combined approach (recommended)

### Session Management

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `persistent_sessions` | bool | `true` | Persist sessions across restarts |
| `max_sessions_per_role` | int | `3` | Max concurrent sessions per role |

## Provider Configuration

### Claude Code (Default)

```json
{
  "provider": "claude_code",
  "claude_config": {
    "model": "claude-3.5-sonnet",
    "dangerous_skip": true,
    "think_mode": "think_hard"
  }
}
```

**Think Modes:**
- `quick_think`: Fast responses, lower quality
- `think_hard`: Balanced (recommended for agents)
- `ultra_think`: Maximum quality (recommended for Master)

### Aider

```json
{
  "provider": "aider",
  "config": {
    "model": "claude-3-5-sonnet",
    "auto_commit": true,
    "edit_format": "diff"
  }
}
```

### Custom Provider

```json
{
  "provider": "custom",
  "config": {
    "command": "/path/to/custom-tool",
    "args": ["--mode", "agent"]
  }
}
```

## Environment Variables

### Required

| Variable | Description |
|----------|-------------|
| `ANTHROPIC_API_KEY` | Anthropic API key (for Claude providers) |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `OPENAI_API_KEY` | - | OpenAI API key |
| `RUST_LOG` | `info` | Log level (`debug`, `info`, `warn`, `error`) |
| `CCSWARM_HOME` | `~/.ccswarm` | Configuration directory |

### Claude ACP Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `CCSWARM_CLAUDE_ACP_URL` | `ws://localhost:9100` | WebSocket endpoint |
| `CCSWARM_CLAUDE_ACP_AUTO_CONNECT` | `true` | Auto-connect on startup |
| `CCSWARM_CLAUDE_ACP_TIMEOUT` | `30` | Connection timeout (seconds) |
| `CCSWARM_CLAUDE_ACP_MAX_RETRIES` | `3` | Max reconnection attempts |
| `CCSWARM_CLAUDE_ACP_DEBUG` | `false` | Enable debug logging |

## Auto-Accept Mode

Controls automated execution with risk assessment:

```json
{
  "auto_accept": {
    "enabled": true,
    "risk_threshold": 5,
    "protected_patterns": [".env", "*.key", ".git/"]
  }
}
```

**Risk Levels (1-10):**
- 1-3: Safe operations (read, format)
- 4-6: Standard operations (write, modify)
- 7-9: Risky operations (delete, config changes)
- 10: Critical operations (always require approval)

## Execution Mode

By default, ccswarm runs with `dangerous_skip: true`, which adds the `--dangerously-skip-permissions` flag for automated execution. Disable for supervised mode:

```json
{
  "claude_config": {
    "dangerous_skip": false
  }
}
```

## Validation

```bash
# Validate configuration
ccswarm config validate

# Show current configuration
ccswarm config show

# Test provider connections
ccswarm config test-providers
```

## See Also

- [Getting Started](GETTING_STARTED.md) - Initial setup guide
- [Commands Reference](COMMANDS.md) - CLI command reference
- [Claude ACP Guide](CLAUDE_ACP.md) - Claude Code integration
- [Troubleshooting](TROUBLESHOOTING.md) - Common issues
