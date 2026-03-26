---
name: git-worktree
description: Parallel development workflow using git worktrees. Each agent gets an isolated worktree for safe concurrent work.
user-invocable: true
argument-hint: "[feature-name]"
---

Multi-step workflow for parallel development using git worktrees.

## Create Worktree

```bash
# Feature worktree
git worktree add ../ccswarm-feature-$ARGUMENTS feature/$ARGUMENTS

# Bug fix worktree
git worktree add ../ccswarm-bugfix-$ARGUMENTS hotfix/$ARGUMENTS
```

## Agent Team Integration

With Agent Teams, each agent automatically gets `isolation: worktree`. For manual worktree setup:

```bash
git worktree add ../ccswarm-frontend feature/ui-redesign    # Frontend agent
git worktree add ../ccswarm-backend feature/api-enhancement  # Backend agent
git worktree add ../ccswarm-devops feature/ci-cd-improvement # DevOps agent
```

## Management

```bash
git worktree list                              # List all worktrees
git worktree remove ../ccswarm-feature-$ARGUMENTS  # Remove after merge
git worktree prune                             # Clean stale entries
```

## Best Practices

1. One worktree per feature/bug
2. Naming: `ccswarm-<type>-<description>`
3. Clean up after merging
4. Run `git worktree prune` periodically
