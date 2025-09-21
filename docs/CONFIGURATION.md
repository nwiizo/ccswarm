# Configuration Reference

This comprehensive guide covers all configuration options available in ccswarm, from basic setup to advanced customization.

## Configuration Files Overview

ccswarm uses several configuration files:

- **`ccswarm.json`** - Main project configuration (required)
- **`~/.ccswarm/config.toml`** - Global user settings (optional)
- **`.env`** - Environment variables (optional)
- **Agent-specific CLAUDE.md** - Per-agent instructions (auto-generated)

## Main Configuration File (ccswarm.json)

The primary configuration file defines your project structure, agents, and behavior.

### Basic Structure

```json
{
  "project": {
    "name": "MyProject",
    "description": "Project description",
    "version": "1.0.0"
  },
  "agents": [...],
  "providers": {...},
  "coordination": {...},
  "session_management": {...},
  "security": {...},
  "quality": {...}
}
```

## Project Configuration

### Basic Project Settings

```json
{
  "project": {
    "name": "MyProject",
    "description": "A sample project using ccswarm",
    "version": "1.0.0",
    "repository": "https://github.com/user/myproject",
    "author": "Your Name",
    "license": "MIT"
  }
}
```

### Master Claude Configuration

Master Claude is the orchestrator that analyzes tasks and delegates to agents.

```json
{
  "project": {
    "master_claude": {
      "role": "technical_lead",
      "quality_threshold": 0.85,
      "think_mode": "ultra_think",
      "permission_level": "supervised",
      "enable_proactive_mode": true,
      "proactive_frequency": 30,
      "high_frequency": 15,
      "autonomous_learning": true,
      "goal_tracking": true,
      "claude_config": {
        "model": "claude-3.5-sonnet",
        "dangerous_skip": true,
        "think_mode": "ultra_think",
        "max_tokens": 4096,
        "temperature": 0.3
      }
    }
  }
}
```

#### Master Claude Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `role` | string | `"technical_lead"` | Master Claude's role perspective |
| `quality_threshold` | float | `0.85` | Minimum quality score for task approval |
| `think_mode` | string | `"ultra_think"` | Claude's thinking depth (`think_hard`, `ultra_think`) |
| `permission_level` | string | `"supervised"` | Execution permission level |
| `enable_proactive_mode` | bool | `true` | Enable autonomous task prediction |
| `proactive_frequency` | int | `30` | Standard analysis interval (seconds) |
| `high_frequency` | int | `15` | High-priority analysis interval (seconds) |
| `autonomous_learning` | bool | `true` | Enable self-learning capabilities |
| `goal_tracking` | bool | `true` | Enable OKR and milestone tracking |

#### Permission Levels

- **`supervised`**: Human approval required for high-risk operations
- **`autonomous`**: Full automation with risk assessment
- **`restricted`**: Limited to read-only and low-risk operations

#### Think Modes

- **`think_hard`**: Deep analysis for complex tasks
- **`ultra_think`**: Maximum reasoning depth for critical decisions

## Agent Configuration

Agents are specialized AI workers with strict role boundaries.

### Agent Definition

```json
{
  "agents": [
    {
      "name": "frontend-specialist",
      "role": "Frontend",
      "provider": "claude_code",
      "enabled": true,
      "priority": 1,
      "max_concurrent_tasks": 3,
      "auto_accept": {
        "enabled": true,
        "risk_threshold": 5,
        "protected_files": [".env", "*.key", ".git/*"],
        "allowed_operations": ["read", "write", "create"],
        "forbidden_operations": ["delete", "execute"]
      },
      "claude_config": {
        "model": "claude-3.5-sonnet",
        "dangerous_skip": true,
        "think_mode": "think_hard",
        "max_tokens": 8192,
        "temperature": 0.1
      },
      "specialization": {
        "frameworks": ["React", "Vue", "Angular"],
        "languages": ["JavaScript", "TypeScript", "CSS"],
        "tools": ["Webpack", "Vite", "ESLint"]
      },
      "constraints": {
        "file_patterns": ["src/**/*.{js,ts,jsx,tsx,css,scss}"],
        "forbidden_patterns": ["server/**", "api/**", "db/**"],
        "max_file_size": "10MB",
        "allowed_commands": ["npm", "yarn", "pnpm", "git"]
      }
    }
  ]
}
```

### Agent Roles

| Role | Responsibilities | Restrictions |
|------|-----------------|--------------|
| `Frontend` | UI/UX, React/Vue, CSS, client-side | No backend APIs, no server config |
| `Backend` | APIs, databases, server logic | No UI components, no styling |
| `DevOps` | Docker, CI/CD, infrastructure | No application logic, no features |
| `QA` | Testing, quality assurance | No feature implementation |
| `Security` | Security scanning, vulnerability assessment | Read-only analysis mode |

### Auto-Accept Configuration

Auto-accept allows agents to execute tasks automatically based on risk assessment.

```json
{
  "auto_accept": {
    "enabled": true,
    "risk_threshold": 5,
    "protected_files": [
      ".env*",
      "*.key",
      "*.pem",
      ".git/*",
      "node_modules/*",
      "target/*"
    ],
    "allowed_operations": ["read", "write", "create"],
    "forbidden_operations": ["delete", "execute", "sudo"],
    "require_confirmation": [
      "rm *",
      "sudo *",
      "chmod +x *",
      "docker run *"
    ],
    "safe_patterns": [
      "*.md",
      "*.txt",
      "*.json",
      "src/**/*.{js,ts,jsx,tsx}"
    ]
  }
}
```

#### Risk Assessment Scale

- **1-2**: Safe operations (documentation, comments)
- **3-4**: Low risk (code changes, configuration)
- **5-6**: Medium risk (file creation, package installation)
- **7-8**: High risk (system commands, file deletion)
- **9-10**: Critical risk (production deployment, data migration)

## Provider Configuration

ccswarm supports multiple AI providers for different use cases.

### Claude Code Provider (Default)

```json
{
  "providers": {
    "claude_code": {
      "type": "claude_code",
      "enabled": true,
      "default_model": "claude-3.5-sonnet",
      "config": {
        "dangerous_skip": true,
        "think_mode": "think_hard",
        "max_tokens": 8192,
        "temperature": 0.1,
        "timeout": 300,
        "retry_attempts": 3
      },
      "rate_limits": {
        "requests_per_minute": 60,
        "tokens_per_minute": 100000
      }
    }
  }
}
```

### Aider Provider

```json
{
  "providers": {
    "aider": {
      "type": "aider",
      "enabled": true,
      "config": {
        "model": "claude-3-5-sonnet",
        "auto_commit": true,
        "edit_format": "diff",
        "show_diffs": true,
        "pretty": true,
        "stream": true
      },
      "git_config": {
        "auto_add_all": false,
        "commit_message_template": "feat: {description}",
        "branch_protection": true
      }
    }
  }
}
```

### OpenAI Provider

```json
{
  "providers": {
    "openai": {
      "type": "openai",
      "enabled": false,
      "config": {
        "model": "gpt-4",
        "max_tokens": 8192,
        "temperature": 0.2,
        "top_p": 1.0,
        "frequency_penalty": 0.0,
        "presence_penalty": 0.0
      }
    }
  }
}
```

### Custom Provider

```json
{
  "providers": {
    "custom": {
      "type": "custom",
      "enabled": true,
      "command": "/path/to/custom/tool",
      "args": ["--interactive", "--model", "custom-model"],
      "config": {
        "timeout": 600,
        "retry_attempts": 2,
        "input_format": "json",
        "output_format": "json"
      }
    }
  }
}
```

## Coordination Configuration

Controls how agents communicate and coordinate tasks.

```json
{
  "coordination": {
    "method": "JSON_FILES",
    "delegation_strategy": "Hybrid",
    "task_queue": {
      "max_size": 1000,
      "priority_levels": 5,
      "auto_requeue": true,
      "retry_failed": true,
      "max_retries": 3
    },
    "communication": {
      "message_bus": "redis",
      "message_retention": "24h",
      "broadcast_updates": true
    },
    "dependencies": {
      "auto_resolve": true,
      "parallel_execution": true,
      "max_parallel": 5,
      "dependency_timeout": "30m"
    }
  }
}
```

### Delegation Strategies

| Strategy | Description | Best For |
|----------|-------------|----------|
| `ContentBased` | Keyword and content analysis | Simple projects |
| `LoadBalanced` | Even workload distribution | High-throughput |
| `ExpertiseBased` | Historical performance analysis | Complex projects |
| `WorkflowBased` | Task dependency analysis | Sequential workflows |
| `Hybrid` | Combined approach (recommended) | Most scenarios |

## Session Management

Configure AI session behavior and optimization.

```json
{
  "session_management": {
    "persistent_sessions": true,
    "max_sessions_per_role": 3,
    "session_timeout": "2h",
    "auto_cleanup": true,
    "compression": {
      "enabled": true,
      "algorithm": "zstd",
      "level": 3,
      "threshold": 0.8
    },
    "history": {
      "max_messages": 50,
      "sliding_window": true,
      "context_preservation": true
    },
    "pooling": {
      "enabled": true,
      "min_pool_size": 1,
      "max_pool_size": 5,
      "idle_timeout": "10m"
    }
  }
}
```

### Session Optimization

- **Compression**: Reduces token usage by 93% through intelligent history compression
- **Pooling**: Reuses similar sessions for related tasks
- **History Management**: Maintains context while managing memory usage

## Security Configuration

Comprehensive security settings for safe automation.

```json
{
  "security": {
    "enabled": true,
    "security_agent": {
      "enabled": true,
      "scan_frequency": "30m",
      "owasp_top_10": true,
      "dependency_scanning": true,
      "real_time_monitoring": true
    },
    "vulnerability_scanning": {
      "npm_audit": true,
      "cargo_audit": true,
      "pip_audit": true,
      "composer_audit": true
    },
    "risk_assessment": {
      "file_operations": true,
      "command_execution": true,
      "network_requests": true,
      "system_changes": true
    },
    "protection": {
      "sensitive_files": [
        ".env*",
        "*.key",
        "*.pem",
        "secrets/*",
        "config/production/*"
      ],
      "forbidden_commands": [
        "rm -rf /",
        "sudo rm",
        "chmod 777",
        "curl | sh"
      ],
      "emergency_stop": {
        "enabled": true,
        "triggers": ["high_risk", "multiple_failures", "security_alert"]
      }
    }
  }
}
```

### Security Levels

- **Low**: Basic file protection
- **Medium**: Command validation and risk scoring
- **High**: Real-time monitoring and intervention
- **Paranoid**: Human approval for all operations

## Quality Configuration

LLM-as-Judge quality evaluation settings.

```json
{
  "quality": {
    "enabled": true,
    "judge": {
      "model": "claude-3.5-sonnet",
      "evaluation_frequency": "30s",
      "dimensions": {
        "correctness": { "weight": 0.25, "threshold": 0.8 },
        "maintainability": { "weight": 0.20, "threshold": 0.75 },
        "test_quality": { "weight": 0.15, "threshold": 0.8 },
        "security": { "weight": 0.15, "threshold": 0.9 },
        "performance": { "weight": 0.10, "threshold": 0.7 },
        "documentation": { "weight": 0.10, "threshold": 0.7 },
        "architecture": { "weight": 0.05, "threshold": 0.75 }
      }
    },
    "standards": {
      "minimum_overall_score": 0.85,
      "critical_threshold": 0.6,
      "auto_remediation": true,
      "failure_escalation": true
    },
    "reporting": {
      "detailed_feedback": true,
      "improvement_suggestions": true,
      "historical_tracking": true
    }
  }
}
```

## Sangha Configuration

Democratic decision-making system configuration.

```json
{
  "sangha": {
    "enabled": true,
    "consensus": {
      "algorithm": "simple_majority",
      "threshold": 0.51,
      "minimum_participants": 2,
      "voting_timeout": "24h"
    },
    "proposals": {
      "auto_create": true,
      "categories": ["feature", "improvement", "fix", "documentation"],
      "required_fields": ["title", "description", "impact", "risk"]
    },
    "voting": {
      "anonymous": false,
      "weighted": true,
      "expertise_weights": {
        "Frontend": 1.0,
        "Backend": 1.0,
        "DevOps": 0.8,
        "QA": 0.9
      }
    }
  }
}
```

### Consensus Algorithms

- **`simple_majority`**: 51%+ approval
- **`supermajority`**: 67%+ approval  
- **`unanimous`**: 100% approval
- **`byzantine_fault_tolerant`**: 67%+ with fault tolerance

## Environment Variables

ccswarm supports configuration through environment variables:

### Required Variables

```bash
# API Keys
export ANTHROPIC_API_KEY="your-anthropic-key"
export OPENAI_API_KEY="your-openai-key"  # Optional
```

### Optional Variables

```bash
# Logging
export RUST_LOG="debug"
export RUST_LOG="ccswarm::session=trace"

# Configuration
export CCSWARM_HOME="~/.ccswarm"
export CCSWARM_CONFIG="/path/to/ccswarm.json"

# Features  
export CCSWARM_SIMULATION="true"  # Run in simulation mode
export CCSWARM_OFFLINE="true"     # Disable network features

# Performance
export CCSWARM_MAX_SESSIONS="10"
export CCSWARM_COMPRESSION="true"
export CCSWARM_POOL_SIZE="5"

# Security
export CCSWARM_SECURITY_LEVEL="high"
export CCSWARM_AUTO_ACCEPT="false"
export CCSWARM_RISK_THRESHOLD="3"
```

## Global Configuration (~/.ccswarm/config.toml)

User-specific global settings:

```toml
[user]
name = "Your Name"
email = "your.email@example.com"
github_username = "yourusername"

[defaults]
think_mode = "think_hard"
quality_threshold = 0.85
auto_accept_risk = 5
default_agents = ["frontend", "backend"]

[providers]
preferred = "claude_code"
backup = "aider"

[ui]
theme = "dark"
show_progress = true
auto_refresh = true
refresh_interval = 5

[performance]
max_concurrent_sessions = 5
compression_enabled = true
history_limit = 100

[security]
default_level = "medium"
scan_dependencies = true
require_confirmation = ["delete", "execute"]
```

## Template Configurations

### Minimal Configuration

For simple projects:

```json
{
  "project": {
    "name": "SimpleProject"
  },
  "agents": [
    {
      "name": "full-stack",
      "role": "Frontend",
      "provider": "claude_code"
    }
  ]
}
```

### Full-Featured Configuration

For enterprise projects:

```json
{
  "project": {
    "name": "EnterpriseProject",
    "master_claude": {
      "enable_proactive_mode": true,
      "quality_threshold": 0.9,
      "permission_level": "supervised"
    }
  },
  "agents": [
    {
      "name": "frontend-lead",
      "role": "Frontend",
      "provider": "claude_code",
      "max_concurrent_tasks": 5,
      "auto_accept": {
        "enabled": true,
        "risk_threshold": 3
      }
    },
    {
      "name": "backend-lead", 
      "role": "Backend",
      "provider": "claude_code",
      "max_concurrent_tasks": 5
    },
    {
      "name": "devops-specialist",
      "role": "DevOps", 
      "provider": "aider",
      "max_concurrent_tasks": 3
    },
    {
      "name": "qa-engineer",
      "role": "QA",
      "provider": "claude_code",
      "max_concurrent_tasks": 3
    }
  ],
  "security": {
    "enabled": true,
    "security_agent": {
      "enabled": true,
      "scan_frequency": "15m"
    }
  },
  "quality": {
    "enabled": true,
    "standards": {
      "minimum_overall_score": 0.9
    }
  },
  "sangha": {
    "enabled": true,
    "consensus": {
      "algorithm": "supermajority",
      "threshold": 0.67
    }
  }
}
```

## Configuration Validation

ccswarm validates configuration on startup:

```bash
# Validate configuration
ccswarm config validate

# Show current configuration
ccswarm config show

# Test provider connections
ccswarm config test-providers

# Generate default configuration
ccswarm config init --template enterprise
```

## Dynamic Configuration Updates

Some settings can be updated at runtime:

```bash
# Update Master Claude settings
ccswarm config set master_claude.proactive_frequency 15

# Modify agent settings
ccswarm config set agents.frontend.auto_accept.risk_threshold 3

# Change quality thresholds
ccswarm config set quality.standards.minimum_overall_score 0.9

# Apply changes
ccswarm config reload
```

## Configuration Best Practices

### Security
- Never commit API keys to version control
- Use environment variables for sensitive data
- Set appropriate risk thresholds for your environment
- Enable security scanning for production projects

### Performance
- Enable session compression for token savings
- Use session pooling for high-throughput scenarios
- Set appropriate timeout values for your network
- Monitor memory usage with long-running sessions

### Quality
- Set quality thresholds based on project criticality
- Enable auto-remediation for faster feedback
- Use appropriate evaluation frequencies
- Track quality metrics over time

### Collaboration
- Use Sangha for team decision-making
- Set clear agent role boundaries
- Enable proactive mode for autonomous operation
- Document configuration decisions

## Troubleshooting Configuration

### Common Issues

**Invalid JSON syntax**
```bash
# Validate JSON syntax
ccswarm config validate
# Shows specific syntax errors and line numbers
```

**Missing required fields**
```bash
# Check required configuration
ccswarm config check --strict
# Lists all missing required fields
```

**Provider connection issues**
```bash
# Test provider connectivity
ccswarm config test-providers --verbose
# Shows detailed connection test results
```

**Permission issues**
```bash
# Check file permissions
ccswarm doctor --check permissions
# Diagnoses and suggests fixes
```

---

This configuration reference covers all available options in ccswarm. For specific use cases and examples, see the [Getting Started Guide](GETTING_STARTED.md) and [Troubleshooting Guide](TROUBLESHOOTING.md).