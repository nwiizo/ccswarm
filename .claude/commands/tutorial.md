# ccswarm tutorial

Interactive tutorial system for learning ccswarm.

## Usage
```bash
ccswarm tutorial [OPTIONS]
```

## Options
- `-c, --chapter <CHAPTER>` - Start from specific chapter (1-4)

## Description
An interactive, hands-on tutorial that teaches you how to use ccswarm through practical examples. The tutorial consists of 4 chapters:

1. **Setting Up Your First Project** (~3-5 minutes)
   - Environment check
   - Creating a project
   - Understanding project structure

2. **Working with AI Agents** (~3-5 minutes)
   - Understanding agent specializations
   - Starting the system
   - Viewing agent status

3. **Creating and Managing Tasks** (~3-5 minutes)
   - Creating your first task
   - Task modifiers and priorities
   - Monitoring task progress

4. **Advanced Features** (~3-5 minutes)
   - Quality checks
   - Session management
   - Auto-create system

## Examples

### Start Tutorial
```bash
$ ccswarm tutorial

╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║              🚀 Welcome to ccswarm Tutorial! 🚀              ║
║                                                               ║
║         AI-Powered Multi-Agent Orchestration System           ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝

This interactive tutorial will teach you:

  📚 Chapter 1: Setting Up Your First Project
  🤖 Chapter 2: Working with AI Agents
  📝 Chapter 3: Creating and Managing Tasks
  🎯 Chapter 4: Advanced Features

Each chapter takes ~3-5 minutes.

Press ENTER to begin...
```

### Jump to Specific Chapter
```bash
ccswarm tutorial --chapter 3
```

## Features
- Interactive command demonstrations
- Progress tracking
- Simulated command execution
- Contextual tips and best practices
- Completion tracking

## Tips
- The tutorial saves your progress
- You can exit and resume anytime
- Each chapter builds on the previous one
- Random tips appear throughout

## Related Commands
- `ccswarm setup` - Configure a new project
- `ccswarm help-topic` - Detailed help on specific topics
- `ccswarm doctor` - Check system requirements