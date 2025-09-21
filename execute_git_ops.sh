#!/usr/bin/env bash
set -euo pipefail

echo "=== Git Operations Script ==="
echo "Working directory: $(pwd)"

# 1. Check current git status and branch
echo "1. Checking current git status and branch..."
git status
git branch

# 2. Switch to main branch
echo "2. Switching to main branch..."
git checkout main

# 3. Clean up worktrees
echo "3. Cleaning up worktrees..."
git worktree prune

# 4. Add all changes
echo "4. Adding all changes..."
git add -A

# 5. Create commit with message
echo "5. Creating commit..."
git commit -m "feat: add Claude Code ACP integration as default communication method

- Add Claude ACP (Agent Client Protocol) integration as default feature
- Remove ai-session dependencies completely
- Create sample directory with demonstration scripts
- Update documentation (README.md, CLAUDE.md)
- Fix cargo fmt and clippy issues

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# 6. List and delete merged feature branches
echo "6. Managing feature branches..."
echo "Current branches:"
git branch -a

echo "Merged branches (safe to delete):"
git branch --merged main | grep -v "\* main" | grep -v "main$" || echo "No merged branches to delete"

# Delete merged branches
git branch --merged main | grep -v "\* main" | grep -v "main$" | xargs -n 1 git branch -d || echo "No branches deleted"

# 7. Show final git status and recent commits
echo "7. Final git status and recent commits..."
git status
echo "Recent commits:"
git log --oneline -5

echo "=== Git Operations Complete ==="