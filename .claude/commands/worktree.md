# ccswarm worktree

Manage Git worktrees for agent isolation and parallel development.

## Description

The `worktree` command manages Git worktrees that provide isolated development environments for each agent. This allows multiple agents to work on different features simultaneously without conflicts.

## Usage

```bash
ccswarm worktree [SUBCOMMAND] [OPTIONS]
```

## Subcommands

- `list` - List all worktrees (default)
- `create <NAME>` - Create a new worktree
- `remove <NAME>` - Remove a worktree
- `clean` - Clean up abandoned worktrees
- `status` - Show worktree status
- `sync` - Sync worktrees with main branch
- `assign <NAME> <AGENT>` - Assign worktree to agent

## Options

### For `list`
- `--verbose` - Show detailed information
- `--format <FORMAT>` - Output format (table, json)
- `--filter <PATTERN>` - Filter by name pattern

### For `create`
- `--branch <BRANCH>` - Base branch (default: main)
- `--path <PATH>` - Custom worktree path
- `--agent <AGENT>` - Assign to agent immediately

### For `remove`
- `--force` - Force removal even with changes
- `--keep-branch` - Keep the branch after removal

### For `clean`
- `--dry-run` - Show what would be cleaned
- `--older-than <DAYS>` - Clean worktrees older than days
- `--orphaned` - Only clean orphaned worktrees

### For `sync`
- `--rebase` - Use rebase instead of merge
- `--worktree <NAME>` - Sync specific worktree only

## Examples

### List all worktrees
```bash
ccswarm worktree list
```

Output:
```
Name                    Branch                  Agent               Status
frontend-feature-nav    feature/navigation      frontend-specialist Active
backend-auth           feature/authentication   backend-api         Active
devops-ci              feature/ci-pipeline      devops-expert       Idle
qa-test-suite          feature/test-framework   -                   Orphaned
```

### Create new worktree
```bash
ccswarm worktree create feature-payment --agent backend-api
```

### Create with custom path
```bash
ccswarm worktree create experimental \
  --branch develop \
  --path ../ccswarm-experimental
```

### Remove worktree
```bash
ccswarm worktree remove feature-nav
```

### Clean orphaned worktrees
```bash
ccswarm worktree clean --orphaned
```

### Check worktree status
```bash
ccswarm worktree status
```

## Worktree Structure

```
project/
├── .git/                    # Main repository
├── main-branch/            # Primary working directory
└── .ccswarm-worktrees/     # Agent worktrees
    ├── frontend-feature/   # Frontend agent workspace
    ├── backend-api/        # Backend agent workspace
    └── devops-infra/       # DevOps agent workspace
```

## Agent Assignment

### Automatic Assignment
During task execution, worktrees are automatically created:
```bash
# When task is assigned to frontend agent
# System creates: .ccswarm-worktrees/frontend-task-123
```

### Manual Assignment
```bash
ccswarm worktree assign feature-ui frontend-specialist
```

### View Assignments
```bash
ccswarm worktree list --verbose
```

## Synchronization

### Sync all worktrees with main
```bash
ccswarm worktree sync
```

### Sync specific worktree
```bash
ccswarm worktree sync --worktree frontend-feature
```

### Resolve conflicts
```bash
# If sync fails due to conflicts
cd .ccswarm-worktrees/frontend-feature
git status
# Resolve conflicts manually
git add .
git rebase --continue
```

## Cleanup Operations

### Remove old worktrees
```bash
ccswarm worktree clean --older-than 7
```

### Dry run to see what would be cleaned
```bash
ccswarm worktree clean --dry-run
```

### Force clean all orphaned
```bash
ccswarm worktree clean --orphaned --force
```

## Integration with Tasks

### Task-based worktree creation
```bash
# Worktrees are named after tasks
task_123 → .ccswarm-worktrees/task-123-auth
task_124 → .ccswarm-worktrees/task-124-ui
```

### Worktree lifecycle
1. Task assigned to agent
2. Worktree created from main branch
3. Agent works in isolated environment
4. Changes committed to feature branch
5. Worktree cleaned after merge

## Advanced Usage

### Custom worktree strategies
```bash
# Configure in ccswarm.json
{
  "worktree_strategy": {
    "naming": "task-based",
    "base_branch": "develop",
    "auto_cleanup": true,
    "cleanup_after_days": 7
  }
}
```

### Worktree templates
```bash
# Create template worktree
ccswarm worktree create template-frontend \
  --template \
  --include "src/,tests/,package.json"

# Use template for new worktrees
ccswarm worktree create feature-ui --from-template template-frontend
```

## Troubleshooting

### Worktree not found
```bash
# Verify worktree exists
git worktree list

# Repair worktree links
git worktree repair
```

### Cannot remove worktree
```bash
# Check for uncommitted changes
cd .ccswarm-worktrees/feature-name
git status

# Force removal
ccswarm worktree remove feature-name --force
```

### Sync conflicts
```bash
# Manual conflict resolution
cd .ccswarm-worktrees/conflicted-worktree
git rebase --abort  # Cancel rebase
git merge main      # Try merge instead
```

## Best Practices

1. **Regular Cleanup** - Run `worktree clean` weekly
2. **Descriptive Names** - Use feature/purpose in worktree names
3. **Sync Often** - Keep worktrees updated with main branch
4. **Monitor Disk Usage** - Each worktree uses ~100MB+
5. **Branch Strategy** - Create worktrees from stable branches

## Related Commands

- [`agents`](agents.md) - View agent-worktree assignments
- [`task`](task.md) - Tasks create worktrees automatically
- [`status`](status.md) - Includes worktree information
- [`git`](logs.md) - Direct git operations

## Notes

- Worktrees provide complete isolation between agents
- Each worktree maintains its own Git index
- Changes in worktrees don't affect main working directory
- Worktrees are cleaned automatically after task completion
- Supports all standard Git operations within worktrees