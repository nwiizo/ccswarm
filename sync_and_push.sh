#!/usr/bin/env bash
set -euo pipefail

echo "=== Syncing and Pushing to Remote Master ==="
echo "Working directory: $(pwd)"

# 現在のブランチを確認
echo -e "\n📍 Current branch:"
git branch --show-current

# リモートの最新情報を取得
echo -e "\n🔄 Fetching latest from remote..."
git fetch origin

# リモートのmasterブランチの状態を確認
echo -e "\n📊 Remote master status:"
git log origin/master --oneline -5

# ローカルのmasterブランチの状態を確認
echo -e "\n📊 Local master status:"
git log master --oneline -5

# リモートの変更をマージ（--allow-unrelated-historiesオプション付き）
echo -e "\n🔀 Merging remote changes..."
git pull origin master --no-rebase --allow-unrelated-histories -m "Merge remote master with local Claude ACP changes" || {
    echo "⚠️ Merge conflict detected. Attempting to resolve..."

    # コンフリクトがある場合は、ローカルの変更を優先
    echo "Using local changes for conflicts..."
    git checkout --ours .
    git add -A
    git commit -m "Merge remote master - resolved conflicts with local changes"
}

# プッシュ
echo -e "\n📤 Pushing to remote master..."
git push origin master || {
    echo -e "\n⚠️ Push failed. Trying with force-with-lease (safer than force)..."
    echo "This will only force push if no one else has pushed since we fetched."
    read -p "Do you want to force push with lease? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git push origin master --force-with-lease
    else
        echo "❌ Push cancelled. Please resolve manually."
        exit 1
    fi
}

echo -e "\n✅ Successfully synced and pushed to remote master!"
echo -e "\n📊 Final status:"
git status --short
echo -e "\n📝 Latest commits:"
git log --oneline -5

echo -e "\n🌐 Remote repository updated!"
echo "View at: https://github.com/nwiizo/ccswarm"