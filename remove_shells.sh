#!/usr/bin/env bash
set -euo pipefail

echo "=== Removing Shell Scripts from Git ==="
echo "Working directory: $(pwd)"

# 削除対象のシェルスクリプト
SCRIPTS_TO_REMOVE=(
    "git_operations.sh"
    "execute_git_ops.sh"
    "merge_to_master.sh"
    "sync_and_push.sh"
    "remove_shells.sh"  # このスクリプト自身も削除
)

# sample/ディレクトリのスクリプトは残す（デモ用）
echo -e "\n📝 Shell scripts to remove:"
for script in "${SCRIPTS_TO_REMOVE[@]}"; do
    if [ -f "$script" ]; then
        echo "  - $script"
    fi
done

echo -e "\n⚠️ Note: Sample demo scripts in sample/ directory will be kept"

# Gitから削除
echo -e "\n🗑️ Removing scripts from Git..."
for script in "${SCRIPTS_TO_REMOVE[@]}"; do
    if [ -f "$script" ]; then
        git rm "$script" || rm "$script"
        echo "  ✅ Removed: $script"
    fi
done

# .gitignoreに追加
echo -e "\n📝 Adding shell scripts to .gitignore..."
cat >> .gitignore << 'EOF'

# Temporary shell scripts
*.sh
!sample/*.sh
EOF

echo "  ✅ Updated .gitignore"

# 変更をステージング
echo -e "\n📦 Staging changes..."
git add .gitignore

# コミット作成
echo -e "\n💾 Creating commit..."
git commit -m "chore: remove temporary shell scripts from repository

- Remove git operation scripts (not needed in repo)
- Keep sample demo scripts for documentation
- Update .gitignore to exclude shell scripts except in sample/

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>"

echo -e "\n✅ Shell scripts removed from Git repository!"
echo -e "\n📊 Status:"
git status --short

echo -e "\n💡 Next step: Push to remote with 'git push origin master'"