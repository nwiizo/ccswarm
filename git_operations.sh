#!/bin/bash
set -e

echo "🔧 Git Operations for ccswarm"
echo "============================"

cd /Users/nwiizo/ghq/github.com/nwiizo/ccswarm

# Show current branch
echo "Current branch:"
git branch --show-current

# List all branches
echo -e "\nAll branches:"
git branch -a

# Show worktrees
echo -e "\nWorktrees:"
git worktree list

# Clean up worktrees
echo -e "\n🧹 Cleaning up worktrees..."
git worktree prune

# Switch to main branch
echo -e "\n🔄 Switching to main branch..."
git checkout main || git checkout master || git checkout -b main

# Merge feature branches
echo -e "\n🔀 Merging feature branches..."
for branch in $(git branch | grep -E "feature/" | sed 's/\*//g'); do
    echo "Merging $branch..."
    git merge --no-ff "$branch" -m "Merge branch '$branch'" || echo "Failed to merge $branch"
done

# Delete merged branches
echo -e "\n🗑️ Deleting merged branches..."
git branch --merged | grep -E "feature/" | xargs -n 1 git branch -d || true

# Add all changes
echo -e "\n📝 Adding changes..."
git add -A

# Create commit
echo -e "\n💾 Creating commit..."
git commit -m "feat: add Claude Code ACP integration as default communication method

- Add Claude ACP (Agent Client Protocol) integration as default feature
- Remove ai-session dependencies completely
- Create sample directory with demonstration scripts
- Update documentation (README.md, CLAUDE.md)
- Fix cargo fmt and clippy issues

Major changes:
* Claude Code via ACP is now the default communication method
* WebSocket-based real-time communication with Claude Code
* Zero external dependencies (removed tmux/ai-session)
* Sample scripts for demonstration in sample/ directory

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>" || echo "Nothing to commit"

# Show final status
echo -e "\n📊 Final status:"
git status --short
git log --oneline -5

echo -e "\n✅ Git operations complete!"