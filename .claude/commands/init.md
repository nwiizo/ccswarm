# ccswarm init

Initialize a new ccswarm project with AI agents and orchestration configuration.

## Description

The `init` command creates a new ccswarm project structure with a configuration file, agent definitions, and optional templates. It sets up the foundation for multi-agent orchestration with specialized roles.

## Usage

```bash
ccswarm init [OPTIONS]
```

## Options

- `--name <NAME>` - Project name (required)
- `--agents <AGENTS>` - Comma-separated list of agent types (default: frontend,backend)
- `--template <TEMPLATE>` - Project template to use
- `--output <PATH>` - Output directory (default: current directory)
- `--provider <PROVIDER>` - Default provider for agents (default: claude_code)
- `--auto-accept` - Enable auto-accept mode for all agents
- `--verbose` - Show detailed initialization process

## Agent Types

- `frontend` - UI development specialist
- `backend` - API and server development
- `devops` - Infrastructure and deployment
- `qa` - Testing and quality assurance

## Templates

- `default` - Standard multi-agent setup
- `aider-focused` - Optimized for Aider provider
- `full-stack` - Complete web application setup
- `microservices` - Distributed architecture
- `custom` - Minimal setup for customization

## Examples

### Basic initialization
```bash
ccswarm init --name "MyProject" --agents frontend,backend
```

### Full-stack project with all agents
```bash
ccswarm init --name "WebApp" --agents frontend,backend,devops,qa
```

### Aider-focused template
```bash
ccswarm init --name "AiderProject" --template aider-focused
```

### Custom output directory
```bash
ccswarm init --name "API" --agents backend --output ./projects/api
```

### With auto-accept enabled
```bash
ccswarm init --name "FastDev" --agents frontend,backend --auto-accept
```

## Generated Structure

```
project-dir/
├── ccswarm.json          # Main configuration file
├── .ccswarm/             # ccswarm metadata
│   ├── sessions/         # Persistent sessions
│   └── logs/             # System logs
├── agents/               # Agent-specific configs
│   ├── frontend.md       # Frontend agent CLAUDE.md
│   ├── backend.md        # Backend agent CLAUDE.md
│   └── ...
└── coordination/         # Inter-agent communication
    └── bus.json          # Message bus state
```

## Configuration File

The generated `ccswarm.json` includes:

```json
{
  "project": {
    "name": "MyProject",
    "description": "AI-orchestrated development project",
    "master_claude_instructions": "..."
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
    }
  ],
  "coordination": {
    "method": "JSON_FILES",
    "delegation_strategy": "Hybrid"
  }
}
```

## Related Commands

- [`start`](start.md) - Start the orchestrator after initialization
- [`agents`](agents.md) - List and manage agents
- [`config`](config.md) - Modify configuration

## Notes

- The init command creates a git-friendly structure
- Agent CLAUDE.md files can be customized after generation
- Session persistence is enabled by default
- The project name must be unique within the directory