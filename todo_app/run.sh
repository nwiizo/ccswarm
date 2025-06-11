#!/bin/bash
# ccswarm TODO App 起動スクリプト

echo "🤖 ccswarm TODO App を起動中..."

# Node.jsの存在確認
if ! command -v node &> /dev/null; then
    echo "❌ Node.js がインストールされていません"
    echo "📦 Node.js をインストールしてください: https://nodejs.org/"
    exit 1
fi

# 依存関係のインストール
if [ ! -d "node_modules" ]; then
    echo "📦 依存関係をインストール中..."
    npm install
fi

# サーバー起動
echo "🚀 サーバーを起動中..."
echo "📍 アクセスURL: http://localhost:3000"
echo "🛑 終了するには Ctrl+C を押してください"
echo ""

node server.js