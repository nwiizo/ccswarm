# ccswarm Commands Documentation

これまでの開発・リリース手順と使用ツールのドキュメント集です。

## 開発・運用手順書

### 🔧 開発関連
- **[development-tools.md](development-tools.md)** - 開発で使用するツール一覧（Cargo、Git、デバッグツール等）
- **[ci-troubleshooting.md](ci-troubleshooting.md)** - CI/CDトラブルシューティングガイド
- **[git-workflow.md](git-workflow.md)** - Gitワークフローとブランチ戦略

### 📦 リリース関連
- **[release-procedure.md](release-procedure.md)** - リリース手順の詳細ガイド（v0.3.0での実例含む）

### 🏗️ アーキテクチャ関連
- **[project-architecture.md](project-architecture.md)** - プロジェクトの全体アーキテクチャ解説

### 📖 使用方法
- **[usage-examples.md](usage-examples.md)** - 実用的な使用例とコマンド集

## ccswarm コマンドリファレンス

### 🚀 Getting Started
- [`init`](init.md) - Initialize a new ccswarm project
- [`start`](start.md) - Start the ccswarm orchestrator
- [`stop`](stop.md) - Stop the running orchestrator
- [`status`](status.md) - Show status of orchestrator and agents

### 📋 Task Management
- [`task`](task.md) - Add a task to the queue
- [`delegate`](delegate.md) - Master delegation commands
- [`review`](review.md) - Run quality review

### 🤖 Agent Management
- [`agents`](agents.md) - List agents and their configurations
- [`session`](session.md) - AI session management commands (93% token savings)
- [`ai-session`](ai-session.md) - Direct AI-Session CLI commands
- [`worktree`](worktree.md) - Manage Git worktrees

### 🎨 User Interface
- [`tui`](tui.md) - Start Terminal User Interface
- [`logs`](logs.md) - Show logs

### 🛠️ Configuration
- [`config`](config.md) - Generate configuration template
- [`auto-create`](auto-create.md) - Auto-create application with AI agents

## Quick Reference

### Most Common Commands

```bash
# Initialize project
ccswarm init --name "MyProject" --agents frontend,backend

# Start system
ccswarm start

# Monitor with TUI
ccswarm tui

# Add a task
ccswarm task "Implement user authentication" --priority high

# Check status
ccswarm status --detailed
```

### Task Management

```bash
# Add tasks with modifiers
ccswarm task "Fix bug [high] [bug]"
ccswarm task "Add tests [test] [auto]"

# Manual delegation
ccswarm delegate task "Create API" --agent backend

# Review completed work
ccswarm review trigger
```

### Application Generation

```bash
# Generate TODO app
ccswarm auto-create "Create TODO app" --output ./todo

# Generate with specific template
ccswarm auto-create "Blog platform" --template blog --output ./blog
```

### Session Management

```bash
# View sessions (93% token savings!)
ccswarm session list

# Session statistics
ccswarm session stats --show-savings
```

## Command Help

All commands support the `--help` flag for quick reference:

```bash
ccswarm --help
ccswarm init --help
ccswarm task --help
```

## Environment Variables

Common environment variables that affect commands:

```bash
# API Keys
export ANTHROPIC_API_KEY="your-key"
export OPENAI_API_KEY="your-key"

# Logging
export RUST_LOG=debug  # Enable debug logging
export RUST_LOG=ccswarm::session=trace  # Trace specific module

# Configuration
export CCSWARM_HOME="$HOME/.ccswarm"  # Override home directory
```

## Exit Codes

Standard exit codes used by all commands:

- `0` - Success
- `1` - General error
- `2` - Configuration error
- `3` - Runtime error
- `4` - Network/API error
- `5` - File system error

## Global Options

Options available for all commands:

- `--config <FILE>` - Use specific configuration file
- `--verbose` or `-v` - Enable verbose output
- `--quiet` or `-q` - Suppress non-error output
- `--json` - Output in JSON format (where applicable)
- `--no-color` - Disable colored output
- `--help` or `-h` - Show help information
- `--version` or `-V` - Show version information

## Configuration File Locations

ccswarm looks for configuration in this order:

1. `./ccswarm.json` (current directory)
2. `./.ccswarm/config.json`
3. `$CCSWARM_HOME/config.json`
4. `$HOME/.ccswarm/config.json`

## Troubleshooting

### Command not found
```bash
# Ensure ccswarm is in PATH
which ccswarm

# Or use cargo run
cargo run -- <command>
```

### Permission errors
```bash
# Commands run with --dangerously-skip-permissions by default
# To disable:
ccswarm config update --set agents[0].claude_config.dangerous_skip=false
```

### API errors
```bash
# Check API keys
ccswarm config validate --strict

# Test provider
ccswarm agents test-provider claude_code
```

## Version History

### v0.2.0 (Current)
- Enhanced quality review system
- Improved TUI with better filtering
- Comprehensive command documentation
- Better session pool management
- Extended provider configuration

### v0.1.x
- Initial release
- Basic multi-agent orchestration
- Session persistence
- Auto-create functionality

---

For detailed information about any command, click on the command name above or run `ccswarm <command> --help`.