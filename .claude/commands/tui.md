# ccswarm tui

Start the Terminal User Interface for real-time monitoring and control of ccswarm.

## Description

The `tui` command launches an interactive terminal interface that provides real-time monitoring of agent activity, task management, and system control. It offers a comprehensive view of the orchestration system with keyboard-driven navigation.

## Usage

```bash
ccswarm tui [OPTIONS]
```

## Options

- `--config <FILE>` - Path to configuration file (default: ./ccswarm.json)
- `--refresh-rate <MS>` - UI refresh rate in milliseconds (default: 100)
- `--theme <THEME>` - Color theme (default, dark, light)
- `--layout <LAYOUT>` - Initial layout (default, compact, expanded)
- `--filter <PATTERN>` - Initial output filter pattern
- `--agent <NAME>` - Focus on specific agent at startup
- `--reset` - Reset TUI state and preferences

## Key Bindings

### Navigation
- `Tab` / `Shift+Tab` - Switch between tabs
- `↑` / `↓` or `j` / `k` - Navigate lists
- `←` / `→` or `h` / `l` - Switch panels
- `Enter` - Select/Activate item
- `Esc` - Cancel/Back
- `q` - Quit TUI

### Actions
- `c` - Enter command mode
- `t` - Quick add task
- `a` - Add new agent
- `f` - Set filter
- `r` - Refresh display
- `/` - Search in current view
- `?` - Show help

### View Controls
- `1-5` - Switch to tab by number
- `Space` - Toggle item selection
- `Ctrl+C` - Copy selected text
- `Ctrl+L` - Clear current view
- `PageUp/PageDown` - Scroll quickly

## Command Mode

Enter command mode by pressing `c`:

### Task Commands
```
task <description> [modifiers]
task Fix login bug [high] [bug]
task Add documentation [docs] [medium]
task Optimize database [feature] [auto]
```

### Agent Commands
```
agent create frontend
agent pause backend-specialist
agent resume frontend-expert
agent remove qa-tester
```

### Session Commands
```
session list
session attach <session-id>
session pause <session-id>
session resume <session-id>
session stats
```

### Filter Commands
```
filter error
filter agent:frontend
filter priority:high
filter clear
```

### Review Commands
```
review status
review history
review trigger
review task <task-id>
```

### Other Commands
```
worktree list
worktree clean
monitor <agent-name>
delegate stats
help
quit
```

## Task Modifiers

When adding tasks, use these modifiers:

### Priority
- `[high]` - High priority
- `[medium]` - Medium priority
- `[low]` - Low priority
- `[urgent]` - Bypass queue

### Type
- `[feature]` - New feature
- `[bug]` - Bug fix
- `[test]` - Testing task
- `[docs]` - Documentation
- `[refactor]` - Code refactoring

### Options
- `[auto]` - Enable auto-accept
- `[review]` - Force quality review
- `[silent]` - Minimal output

## TUI Layout

```
┌─────────────────────────────────────────────┐
│  ccswarm TUI v0.2.0  │ Status: Running      │
├─────────────────────┴───────────────────────┤
│ Overview │ Agents │ Tasks │ Output │ Logs   │
├─────────────────────────────────────────────┤
│                                             │
│  Main Content Area                          │
│  (Changes based on selected tab)           │
│                                             │
├─────────────────────────────────────────────┤
│ [c]ommand [t]ask [a]gent [f]ilter [?]help  │
└─────────────────────────────────────────────┘
```

## Examples

### Basic TUI launch
```bash
ccswarm tui
```

### Focus on specific agent
```bash
ccswarm tui --agent frontend-specialist
```

### With output filtering
```bash
ccswarm tui --filter "error|warning"
```

### Custom refresh rate (slower)
```bash
ccswarm tui --refresh-rate 500
```

### Dark theme with compact layout
```bash
ccswarm tui --theme dark --layout compact
```

## Tab Descriptions

### Overview Tab
- System status and health
- Active agent count
- Task queue summary
- Recent completions
- Performance metrics

### Agents Tab
- List of all agents
- Status indicators
- Current tasks
- Session information
- Performance stats

### Tasks Tab
- Task queue
- In-progress tasks
- Completed tasks
- Task details
- Priority sorting

### Output Tab
- Real-time agent output
- Filterable logs
- Color-coded messages
- Search functionality
- Export options

### Logs Tab
- System logs
- Error tracking
- Debug information
- Log filtering
- Time-based view

## Advanced Features

### Multi-Select Mode
Hold `Shift` while navigating to select multiple items for bulk operations.

### Quick Filters
- `Ctrl+1` - Show only errors
- `Ctrl+2` - Show only warnings
- `Ctrl+3` - Show only info
- `Ctrl+0` - Clear filters

### Export Functions
- `Ctrl+S` - Save current view
- `Ctrl+E` - Export logs
- `Ctrl+R` - Generate report

## Related Commands

- [`start`](start.md) - Start orchestrator (required for TUI)
- [`status`](status.md) - Quick status check
- [`monitor`](logs.md) - Simple output monitoring
- [`task`](task.md) - Add tasks via CLI

## Notes

- TUI requires orchestrator to be running
- Settings are persisted between sessions
- Performance impact is minimal (<3% overhead)
- Supports terminal resizing
- Works over SSH connections