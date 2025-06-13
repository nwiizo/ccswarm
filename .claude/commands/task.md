# ccswarm task

Add a new task to the ccswarm task queue for agent execution.

## Description

The `task` command adds tasks to the orchestration queue. Tasks are analyzed by Master Claude, delegated to appropriate agents, and executed based on priority and agent availability.

## Usage

```bash
ccswarm task <DESCRIPTION> [OPTIONS]
```

## Options

- `--priority <LEVEL>` - Task priority: high, medium, low (default: medium)
- `--type <TYPE>` - Task type: feature, bug, test, docs, refactor
- `--agent <NAME>` - Assign to specific agent (bypasses delegation)
- `--auto-accept` - Enable auto-accept for this task
- `--no-auto-accept` - Disable auto-accept for this task
- `--parent <TASK-ID>` - Set parent task for dependencies
- `--tags <TAGS>` - Comma-separated tags
- `--estimate <HOURS>` - Time estimate in hours
- `--due <DATE>` - Due date (ISO format)
- `--force-review` - Require quality review after completion

## Task Description Format

Tasks can include inline modifiers:
```
"Description [priority] [type] [options]"
```

## Examples

### Basic task
```bash
ccswarm task "Implement user authentication"
```

### High priority bug fix
```bash
ccswarm task "Fix login timeout issue" --priority high --type bug
```

### Using inline modifiers
```bash
ccswarm task "Add unit tests [high] [test] [auto]"
```

### Assign to specific agent
```bash
ccswarm task "Update API documentation" --agent backend-specialist --type docs
```

### Complex task with all options
```bash
ccswarm task "Refactor payment module" \
  --priority high \
  --type refactor \
  --tags "payment,critical" \
  --estimate 8 \
  --due "2024-02-01T00:00:00Z" \
  --force-review
```

### Subtask with parent
```bash
ccswarm task "Write payment tests" --parent task_123 --type test
```

## Task Modifiers

### Priority Levels
- `[high]` or `[urgent]` - Execute first
- `[medium]` - Normal priority (default)
- `[low]` - Execute when idle

### Task Types
- `[feature]` - New functionality
- `[bug]` - Bug fix
- `[test]` - Testing tasks
- `[docs]` - Documentation
- `[refactor]` - Code improvement

### Execution Options
- `[auto]` - Enable auto-accept
- `[review]` - Force quality review
- `[silent]` - Minimal output
- `[verbose]` - Detailed output

## Task Lifecycle

1. **Creation** - Task added to queue
2. **Analysis** - Master Claude analyzes requirements
3. **Delegation** - Assigned to best-fit agent
4. **Execution** - Agent works on task
5. **Review** - Quality check (if enabled)
6. **Completion** - Task marked complete

## Task Management

### List all tasks
```bash
ccswarm task list
```

### View task details
```bash
ccswarm task show <task-id>
```

### Cancel task
```bash
ccswarm task cancel <task-id>
```

### Update task
```bash
ccswarm task update <task-id> --priority high
```

### Task statistics
```bash
ccswarm task stats
```

## Batch Operations

### Add multiple tasks from file
```bash
ccswarm task batch tasks.txt
```

Where `tasks.txt` contains:
```
Implement login form [high] [feature]
Add password reset [medium] [feature]
Write auth tests [high] [test]
Update auth docs [low] [docs]
```

### Export tasks
```bash
ccswarm task export --format json > tasks.json
```

## Task Dependencies

### Create dependent tasks
```bash
# Parent task
PARENT_ID=$(ccswarm task "Create user model" --json | jq -r '.id')

# Child tasks
ccswarm task "Add user validation" --parent $PARENT_ID
ccswarm task "Create user tests" --parent $PARENT_ID
```

### View dependency tree
```bash
ccswarm task tree <task-id>
```

## Advanced Features

### Task Templates
```bash
# Save as template
ccswarm task template save "security-fix" \
  --priority high \
  --type bug \
  --tags "security,critical" \
  --force-review

# Use template
ccswarm task "Fix SQL injection in search" --template security-fix
```

### Recurring Tasks
```bash
ccswarm task "Run security scan" \
  --recurring daily \
  --time "02:00"
```

### Conditional Tasks
```bash
ccswarm task "Deploy to production" \
  --condition "all-tests-pass" \
  --agent devops
```

## Integration

### Git Integration
```bash
# Create task from commit
ccswarm task from-commit HEAD

# Create tasks from issues
ccswarm task from-issue --repo owner/repo --issue 123
```

### CI/CD Integration
```yaml
# In GitHub Actions
- name: Create deployment task
  run: |
    ccswarm task "Deploy version ${{ github.sha }}" \
      --priority high \
      --type feature \
      --agent devops
```

## Related Commands

- [`delegate`](delegate.md) - Manual task delegation
- [`status`](status.md) - View task queue status
- [`tui`](tui.md) - Interactive task management
- [`review`](review.md) - Quality review tasks

## Notes

- Tasks are persisted across restarts
- Master Claude analyzes all tasks for optimal delegation
- Tasks can be re-delegated if initial agent fails
- Quality review is automatic for critical tasks
- Task history is maintained for analytics