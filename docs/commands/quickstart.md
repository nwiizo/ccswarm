# ccswarm quickstart Command

The `quickstart` command provides a streamlined, one-command setup experience for new ccswarm projects. It combines system checking, project initialization, and configuration into a single interactive process.

## Overview

```bash
ccswarm quickstart [OPTIONS]
```

This command:
1. Checks system requirements (Git, API keys)
2. Initializes a Git repository
3. Creates project configuration
4. Sets up AI agents
5. Creates initial project structure
6. Optionally runs tests

## Options

### `--name, -n <NAME>`
Specify the project name. If not provided, it will be inferred from the current directory name or prompted interactively.

```bash
ccswarm quickstart --name "my-awesome-project"
```

### `--no-prompt`
Skip all interactive prompts and use sensible defaults. Perfect for automation and CI/CD pipelines.

```bash
ccswarm quickstart --no-prompt
```

Default behavior with `--no-prompt`:
- Project name: Current directory name
- Agents: frontend and backend (the most common setup)
- API key: Uses existing environment variable

### `--all-agents`
Enable all available agents (frontend, backend, devops, qa) instead of selecting individually.

```bash
ccswarm quickstart --all-agents
```

### `--with-tests`
Run initial tests after setup to verify everything is working correctly.

```bash
ccswarm quickstart --with-tests
```

## Usage Examples

### Interactive Setup (Recommended for First Time)
```bash
ccswarm quickstart
```

This will:
- Check for Git and API keys
- Prompt for project name
- Let you select which agents to enable
- Create all necessary files
- Show next steps

### Quick Setup with Defaults
```bash
ccswarm quickstart --name "todo-app" --no-prompt
```

Creates a project named "todo-app" with frontend and backend agents using all defaults.

### Full Setup for Complex Project
```bash
ccswarm quickstart --name "enterprise-app" --all-agents --with-tests
```

Sets up a project with all agents enabled and runs verification tests.

### CI/CD Pipeline Setup
```bash
export ANTHROPIC_API_KEY=your-key
ccswarm quickstart --no-prompt --all-agents --with-tests
```

Perfect for automated setups in continuous integration environments.

## What Gets Created

The quickstart command creates:

1. **ccswarm.json** - Project configuration file
2. **.gitignore** - Properly configured for ccswarm projects
3. **README.md** - Project documentation with usage instructions
4. **agents/** - Directory for agent workspaces
5. **Initial Git commit** - Clean starting point for version control

## Configuration Details

The generated configuration includes:
- **Proactive mode enabled** - AI agents suggest next steps automatically
- **Quality threshold: 85%** - Balanced quality standards
- **Think mode: ThinkHard** - Optimal reasoning for most tasks
- **Session management** - Configured for 93% token savings

## System Requirements

Before running quickstart, ensure you have:
- Git installed and available in PATH
- ANTHROPIC_API_KEY environment variable (or be ready to input it)
- Write permissions in the current directory

## Comparison with Other Setup Methods

| Feature | quickstart | init | setup |
|---------|------------|------|-------|
| One command | ✅ | ❌ | ❌ |
| System checks | ✅ | ❌ | ✅ |
| Interactive | Optional | ❌ | ✅ |
| Creates structure | ✅ | Partial | ❌ |
| Configures agents | ✅ | ✅ | ✅ |
| Git initialization | ✅ | ❌ | ❌ |
| Initial commit | ✅ | ❌ | ❌ |
| Tests available | ✅ | ❌ | ❌ |

## Troubleshooting

### "Git is required for ccswarm"
Install Git from https://git-scm.com before running quickstart.

### "ANTHROPIC_API_KEY not found"
Either:
1. Set the environment variable: `export ANTHROPIC_API_KEY=your-key`
2. Enter it when prompted during quickstart
3. Use `--no-prompt` to continue without it (limited functionality)

### "Configuration already exists"
The quickstart command is designed for new projects. For existing projects, use:
- `ccswarm doctor --fix` to repair issues
- `ccswarm setup` to reconfigure

## Next Steps

After running quickstart, you'll see personalized next steps based on your setup. Typically:

1. **Start the orchestrator**: `ccswarm start`
2. **Create your first task**: `ccswarm task "Build a feature"`
3. **Monitor progress**: `ccswarm tui`

For more help:
- Run `ccswarm tutorial` for an interactive guide
- Check `ccswarm doctor` for system health
- Visit https://github.com/nwiizo/ccswarm for documentation