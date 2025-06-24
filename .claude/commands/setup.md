# ccswarm setup

Interactive setup wizard for first-time users.

## Usage
```bash
ccswarm setup
```

## Description
Guides you through creating a new ccswarm project with an interactive wizard that helps you:
- Set project name
- Configure repository
- Select AI agents (Frontend, Backend, DevOps, QA)
- Choose AI provider (Claude Code, Aider, Custom)
- Configure advanced options (proactive mode, quality thresholds, think modes)

## Examples

### Basic Setup
```bash
$ ccswarm setup
🚀 Welcome to ccswarm!
Let's set up your AI-powered multi-agent orchestration system.

What's your project name? [MyProject]: TodoApp
📁 Repository Configuration
Repository URL (or '.' for current directory) [.]: .
🤖 Agent Selection
Which specialized agents do you want to enable?
Frontend Agent (React, Vue, UI development)? [Y/n]: y
Backend Agent (APIs, databases, server logic)? [Y/n]: y
DevOps Agent (Docker, Kubernetes, CI/CD)? [Y/n]: n
QA Agent (Testing, quality assurance)? [y/N]: n
```

### Advanced Configuration
```bash
Configure advanced options? [y/N]: y
⚙️  Advanced Configuration
Enable proactive mode? (AI predicts and suggests next tasks automatically) [Y/n]: y
Quality threshold (0.0-1.0) [85]: 90
Thinking modes:
  1. Think - Fast responses
  2. Think Hard - Better reasoning
  3. Ultra Think - Maximum intelligence
Select thinking mode [2]: 3
```

## Features
- Smart defaults based on common patterns
- Validates configuration before saving
- Creates `ccswarm.json` automatically
- Shows next steps after setup

## Related Commands
- `ccswarm init` - Quick initialization without wizard
- `ccswarm tutorial` - Learn ccswarm basics
- `ccswarm doctor` - Check system requirements