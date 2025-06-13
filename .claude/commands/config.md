# ccswarm config

Generate, validate, and manage ccswarm configuration files.

## Description

The `config` command provides tools for working with ccswarm configuration files. It can generate templates, validate existing configurations, migrate from older formats, and update settings programmatically.

## Usage

```bash
ccswarm config [SUBCOMMAND] [OPTIONS]
```

## Subcommands

- `generate` - Generate a configuration template
- `validate` - Validate a configuration file
- `show` - Display current configuration
- `update` - Update configuration values
- `migrate` - Migrate from old format
- `export` - Export configuration
- `import` - Import configuration

## Options

### For `generate`
- `--template <TYPE>` - Template type (minimal, default, full)
- `--output <FILE>` - Output file (default: ccswarm.json)
- `--agents <LIST>` - Comma-separated agent roles
- `--provider <PROVIDER>` - Default provider

### For `validate`
- `--file <FILE>` - Configuration file to validate
- `--strict` - Enable strict validation
- `--fix` - Attempt to fix issues

### For `update`
- `--file <FILE>` - Configuration file to update
- `--set <KEY=VALUE>` - Set configuration value
- `--add-agent <JSON>` - Add new agent
- `--remove-agent <NAME>` - Remove agent

### For `migrate`
- `--input <FILE>` - Old configuration file
- `--output <FILE>` - New configuration file
- `--backup` - Create backup of original

## Examples

### Generate default configuration
```bash
ccswarm config generate
```

### Generate full configuration template
```bash
ccswarm config generate --template full --output my-config.json
```

### Validate configuration
```bash
ccswarm config validate --file ccswarm.json
```

### Show current configuration
```bash
ccswarm config show
```

### Update configuration value
```bash
ccswarm config update --set project.name="MyNewProject"
```

### Add new agent
```bash
ccswarm config update --add-agent '{
  "name": "ml-specialist",
  "role": "Backend",
  "provider": "claude_code"
}'
```

## Configuration Templates

### Minimal Template
```json
{
  "project": {
    "name": "MyProject"
  },
  "agents": [
    {
      "name": "frontend",
      "role": "Frontend",
      "provider": "claude_code"
    }
  ]
}
```

### Default Template
```json
{
  "project": {
    "name": "MyProject",
    "description": "AI-orchestrated project",
    "master_claude_instructions": "Efficiently orchestrate agents..."
  },
  "agents": [
    {
      "name": "frontend-specialist",
      "role": "Frontend",
      "provider": "claude_code",
      "claude_config": {
        "model": "claude-3.5-sonnet",
        "dangerous_skip": true
      }
    },
    {
      "name": "backend-api",
      "role": "Backend",
      "provider": "claude_code"
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

### Full Template
Includes all possible configuration options:
```bash
ccswarm config generate --template full
```

## Configuration Structure

### Project Section
```json
{
  "project": {
    "name": "Required project name",
    "description": "Optional description",
    "version": "1.0.0",
    "master_claude_instructions": "Custom instructions for Master Claude",
    "repository": "https://github.com/user/repo"
  }
}
```

### Agents Section
```json
{
  "agents": [
    {
      "name": "unique-agent-name",
      "role": "Frontend|Backend|DevOps|QA",
      "provider": "claude_code|aider|custom",
      "provider_config": {
        // Provider-specific settings
      },
      "auto_accept": {
        "enabled": true,
        "risk_threshold": 5,
        "protected_files": ["*.env", "*.key"]
      },
      "worktree_config": {
        "enabled": true,
        "base_branch": "main"
      }
    }
  ]
}
```

### Provider Configurations

#### Claude Code
```json
{
  "claude_config": {
    "model": "claude-3.5-sonnet",
    "dangerous_skip": true,
    "think_mode": "think_hard|think|none",
    "memory": "persistent|conversation|none",
    "context_window": 200000
  }
}
```

#### Aider
```json
{
  "aider_config": {
    "model": "claude-3-5-sonnet",
    "edit_format": "diff|whole|patch",
    "auto_commit": true,
    "auto_lint": true,
    "git_branch": "feature/aider"
  }
}
```

### Advanced Settings
```json
{
  "coordination": {
    "method": "JSON_FILES|GRPC|REST",
    "delegation_strategy": "Hybrid|ContentBased|LoadBalanced",
    "bus_config": {
      "message_retention": "7d",
      "max_message_size": "10MB"
    }
  },
  "session_management": {
    "persistent_sessions": true,
    "max_sessions_per_role": 3,
    "session_timeout": "30m",
    "cleanup_interval": "1h"
  },
  "quality_review": {
    "enabled": true,
    "interval": 30,
    "standards": {
      "min_test_coverage": 0.85,
      "max_complexity": 10
    }
  },
  "monitoring": {
    "metrics_enabled": true,
    "log_level": "info",
    "telemetry": false
  }
}
```

## Validation Rules

### Required Fields
- `project.name` - Must be non-empty
- `agents[]` - At least one agent required
- `agents[].name` - Must be unique
- `agents[].role` - Must be valid role

### Validation Examples
```bash
# Basic validation
ccswarm config validate

# Strict validation (checks API keys, etc.)
ccswarm config validate --strict

# Auto-fix common issues
ccswarm config validate --fix
```

## Migration

### From v0.1.x to v0.2.x
```bash
ccswarm config migrate --input old-config.json --output ccswarm.json
```

### Migration changes:
- Renames `dangerous_mode` to `dangerous_skip`
- Adds `quality_review` section
- Updates provider configurations
- Preserves custom settings

## Programmatic Updates

### Update single value
```bash
ccswarm config update --set coordination.delegation_strategy=LoadBalanced
```

### Update nested values
```bash
ccswarm config update --set agents[0].auto_accept.enabled=true
```

### Add multiple agents
```bash
for role in frontend backend devops; do
  ccswarm config update --add-agent "{
    \"name\": \"$role-expert\",
    \"role\": \"${role^}\",
    \"provider\": \"claude_code\"
  }"
done
```

## Environment Variables

Override configuration with environment variables:
```bash
# Override project name
export CCSWARM_PROJECT_NAME="OverrideProject"

# Override log level
export CCSWARM_LOG_LEVEL="debug"

# Override provider
export CCSWARM_DEFAULT_PROVIDER="aider"
```

## Backup and Restore

### Create backup
```bash
ccswarm config export --output ccswarm-backup-$(date +%Y%m%d).json
```

### Restore from backup
```bash
ccswarm config import --file ccswarm-backup-20240115.json
```

## Related Commands

- [`init`](init.md) - Initialize with configuration
- [`agents`](agents.md) - Manage agents in config
- [`start`](start.md) - Use configuration to start
- [`validate`](review.md) - Validate project setup

## Notes

- Configuration is validated on every start
- Changes require orchestrator restart
- Sensitive values can use environment variables
- Comments are preserved in JSON5 format
- Backup created automatically before updates