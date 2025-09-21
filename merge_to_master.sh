#!/usr/bin/env bash
set -euo pipefail

echo "=== Merging main to master ==="
echo "Working directory: $(pwd)"

# 現在のブランチを確認
echo -e "\n📍 Current branch:"
git branch --show-current

# masterブランチの存在を確認
echo -e "\n🔍 Checking if master branch exists..."
if git show-ref --verify --quiet refs/heads/master; then
    echo "✅ master branch exists"
else
    echo "❌ master branch does not exist. Creating from main..."
    git branch master main
fi

# masterブランチにチェックアウト
echo -e "\n🔄 Switching to master branch..."
git checkout master

# mainブランチの変更をマージ
echo -e "\n🔀 Merging main branch into master..."
git merge main --no-ff -m "Merge main branch with Claude ACP integration into master

Features merged:
- Claude Code ACP integration as default communication method
- Removed ai-session dependencies completely
- Sample directory with demonstration scripts
- Updated documentation (README.md, CLAUDE.md)
- Fixed cargo fmt and clippy issues

This merge brings all the Claude Code integration features to the master branch."

# マージ結果を確認
echo -e "\n✅ Merge completed successfully!"
echo -e "\n📊 Current status:"
git status

echo -e "\n📝 Recent commits on master:"
git log --oneline -5

echo -e "\n🎯 Branch comparison:"
echo "Commits in master but not in main:"
git log main..master --oneline || echo "None (branches are in sync)"

echo -e "\n✨ Merge to master complete!"
echo "You can now push to remote with: git push origin master"