#!/usr/bin/env bash
set -euo pipefail

echo "=== Committing and Pushing Changes ==="
echo "Working directory: $(pwd)"

# 現在のブランチを確認
echo -e "\n📍 Current branch:"
git branch --show-current

# 変更をステージング
echo -e "\n📦 Staging all changes..."
git add -A

# ステータス確認
echo -e "\n📊 Changes to be committed:"
git status --short

# コミット作成
echo -e "\n💾 Creating commit..."
git commit -m "docs: update README.md to reflect Claude ACP integration

- Update workspace structure to show Claude ACP as default
- Remove all ai-session references
- Update architecture diagram with Claude ACP Integration
- Clarify installation instructions (build from source)
- Update What's New section for v0.3.7
- Fix session management descriptions
- Update core commands documentation

This commit ensures README accurately reflects the current implementation
with Claude Code via ACP as the default communication method.

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>" || echo "Nothing to commit"

# 最新のコミットを表示
echo -e "\n📝 Latest commit:"
git log --oneline -1

# リモートにプッシュ
echo -e "\n📤 Pushing to remote..."
git push origin master || {
    echo -e "\n⚠️ Push failed. Attempting to pull and merge first..."
    git pull origin master --no-rebase
    git push origin master
}

echo -e "\n✅ Successfully committed and pushed!"
echo -e "\n🌐 View changes at: https://github.com/nwiizo/ccswarm"