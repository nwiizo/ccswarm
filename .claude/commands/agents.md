# ccswarm agents

List and manage AI agents in the ccswarm system.

## Description

The `agents` command provides comprehensive management of AI agents, including listing active agents, viewing their status, modifying configurations, and controlling their execution state.

## Usage

```bash
ccswarm agents [SUBCOMMAND] [OPTIONS]
```

## Subcommands

- `list` - List all configured agents (default)
- `show <NAME>` - Show detailed agent information
- `create <ROLE>` - Create a new agent
- `pause <NAME>` - Pause an agent
- `resume <NAME>` - Resume a paused agent
- `remove <NAME>` - Remove an agent
- `update <NAME>` - Update agent configuration
- `stats <NAME>` - Show agent statistics

## Options

### For `list`
- `--status <STATUS>` - Filter by status (active, paused, error)
- `--role <ROLE>` - Filter by role (frontend, backend, devops, qa)
- `--provider <PROVIDER>` - Filter by provider
- `--format <FORMAT>` - Output format (table, json, yaml)
- `--verbose` - Show detailed information

### For `create`
- `--name <NAME>` - Agent name (auto-generated if not provided)
- `--role <ROLE>` - Agent role (required)
- `--provider <PROVIDER>` - Provider type (default: claude_code)
- `--auto-accept` - Enable auto-accept mode
- `--config <JSON>` - Provider-specific configuration

### For `update`
- `--provider <PROVIDER>` - Change provider
- `--auto-accept <BOOL>` - Update auto-accept setting
- `--config <JSON>` - Update provider configuration
- `--risk-threshold <NUM>` - Set auto-accept risk threshold

## Examples

### List all agents
```bash
ccswarm agents
```

Output:
```
Name                 Role      Status   Provider      Current Task
frontend-specialist  Frontend  Active   claude_code   Creating navbar
backend-api         Backend   Active   aider         Adding auth endpoint
devops-expert       DevOps    Idle     claude_code   -
qa-tester           QA        Paused   claude_code   -
```

### Show detailed agent info
```bash
ccswarm agents show frontend-specialist
```

### Create new agent
```bash
ccswarm agents create frontend --name "ui-expert" --provider claude_code
```

### Create agent with custom config
```bash
ccswarm agents create backend \
  --name "api-specialist" \
  --provider aider \
  --config '{"model": "claude-3-5-sonnet", "edit_format": "diff"}'
```

### Pause/Resume agent
```bash
ccswarm agents pause frontend-specialist
ccswarm agents resume frontend-specialist
```

### Update agent configuration
```bash
ccswarm agents update backend-api \
  --auto-accept true \
  --risk-threshold 7
```

### View agent statistics
```bash
ccswarm agents stats frontend-specialist
```

## Agent Roles

### Frontend
- UI/UX development
- React, Vue, Angular
- CSS and styling
- Client-side logic

### Backend
- API development
- Database design
- Server logic
- Authentication

### DevOps
- Infrastructure
- CI/CD pipelines
- Deployment
- Monitoring

### QA
- Testing strategies
- Test implementation
- Quality assurance
- Bug verification

## Agent Configuration

### Claude Code Provider
```json
{
  "provider": "claude_code",
  "claude_config": {
    "model": "claude-3.5-sonnet",
    "dangerous_skip": true,
    "think_mode": "think_hard",
    "memory": "persistent"
  },
  "auto_accept": {
    "enabled": true,
    "risk_threshold": 5
  }
}
```

### Aider Provider
```json
{
  "provider": "aider",
  "config": {
    "model": "claude-3-5-sonnet",
    "auto_commit": true,
    "edit_format": "diff",
    "git_branch": "feature/aider"
  }
}
```

## Agent Statistics

View detailed performance metrics:

```bash
ccswarm agents stats backend-api --period 7d
```

Output:
```
Agent: backend-api
Period: Last 7 days

Tasks Completed: 47
Success Rate: 94%
Average Time: 12m 30s
Token Usage: 1.2M
Quality Score: 8.5/10

Top Task Types:
- API endpoints: 18 (38%)
- Database queries: 12 (26%)
- Authentication: 9 (19%)
- Testing: 8 (17%)
```

## Bulk Operations

### Pause all agents
```bash
ccswarm agents list --format json | \
  jq -r '.[] | .name' | \
  xargs -I {} ccswarm agents pause {}
```

### Update all agents to use specific provider
```bash
for agent in $(ccswarm agents list --format json | jq -r '.[] | .name'); do
  ccswarm agents update $agent --provider claude_code
done
```

## Agent Health Monitoring

### Check agent health
```bash
ccswarm agents health
```

### Monitor agent performance
```bash
watch -n 5 'ccswarm agents list --verbose'
```

## Troubleshooting

### Agent not responding
```bash
# Check agent logs
ccswarm logs --agent frontend-specialist

# Restart agent
ccswarm agents restart frontend-specialist
```

### Provider connection issues
```bash
# Test provider connection
ccswarm agents test-provider claude_code

# Verify API keys
ccswarm agents verify-auth
```

## Related Commands

- [`init`](init.md) - Initialize with agents
- [`task`](task.md) - Assign tasks to agents
- [`session`](session.md) - Manage agent sessions
- [`status`](status.md) - View system status
- [`tui`](tui.md) - Interactive agent management

## Notes

- Agents are automatically created during init
- Each agent runs in an isolated tmux session
- Sessions persist across agent restarts
- Agent configurations are stored in ccswarm.json
- Performance metrics are collected continuously