# ccswarm help-topic

Enhanced help system with detailed examples and tips.

## Usage
```bash
ccswarm help-topic [TOPIC] [OPTIONS]
```

## Options
- `-s, --search <QUERY>` - Search for help topics

## Arguments
- `TOPIC` - Specific topic to get help on

## Description
Provides comprehensive help with examples, tips, and related topics. Available topics include:
- getting-started
- tasks
- agents
- sessions
- quality
- troubleshooting

## Examples

### List All Topics
```bash
$ ccswarm help-topic
📚 Available Help Topics

  getting-started - Learn the basics and set up your first project
           tasks - Create and manage tasks for AI agents
          agents - Work with specialized AI agents
        sessions - Manage AI-powered terminal sessions
         quality - Code quality and automated reviews
 troubleshooting - Fix common problems

💡 Usage:
  ccswarm help-topic <topic>     View detailed help
  ccswarm help-topic --search <q>  Search help topics
```

### View Specific Topic
```bash
$ ccswarm help-topic tasks

Task Management
───────────────
Create, delegate, and monitor tasks across agents

📝 Examples:

  ccswarm task "Create user authentication"
  Create a simple task

  ccswarm task "Add login form" --priority high --type feature
  Task with priority and type

  ✅ Task created: task-a4b2c3d4
  🤖 Delegating to frontend agent...

  ccswarm task list --status pending
  View pending tasks

💡 Tips:
  • Use clear, actionable task descriptions
  • Add [high], [bug], or [test] modifiers in task text
  • Tasks are automatically routed to appropriate agents

🔗 Related topics:
  agents, quality
```

### Search Topics
```bash
$ ccswarm help-topic --search "session"

🔍 Search results for "session":

1. sessions - Manage AI-powered terminal sessions
   Sessions persist context across tasks...

2. troubleshooting - Fix common problems
   ...Session not found errors...

3. agents - Work with specialized AI agents
   ...agent sessions in isolated environments...
```

## Available Topics

### getting-started
- Project initialization
- Environment setup
- First steps with ccswarm

### tasks
- Creating tasks
- Task modifiers and priorities
- Delegation strategies
- Task monitoring

### agents
- Agent specializations
- Managing agent lifecycle
- Agent configuration
- Performance monitoring

### sessions
- AI-session benefits (93% token savings)
- Session persistence
- Session management commands
- Recovery mechanisms

### quality
- Automated code review
- Quality thresholds
- Remediation tasks
- LLM-as-judge system

### troubleshooting
- Common errors and solutions
- Debug techniques
- System recovery
- Performance issues

## Related Commands
- `ccswarm tutorial` - Interactive learning
- `ccswarm setup` - Configuration wizard
- `ccswarm doctor` - System diagnostics