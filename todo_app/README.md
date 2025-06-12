# 🤖 ccswarm TODO App

マルチエージェントシステム ccswarm によって開発されたTODOアプリケーション

## 🎯 概要

このTODOアプリケーションは、ccswarmシステムの以下の専門エージェントによって協調開発されました：

- **🎨 Frontend Agent**: HTML, CSS, JavaScript の開発
- **⚙️ Backend Agent**: Node.js Express サーバーとAPI開発  
- **🚀 DevOps Agent**: デプロイメントスクリプトとドキュメント作成

## 🛠️ 技術スタック

- **フロントエンド**: HTML5, CSS3, Vanilla JavaScript
- **バックエンド**: Node.js, Express.js
- **データ永続化**: JSON ファイル
- **スタイリング**: レスポンシブCSS

## 📋 機能

- ✅ TODOタスクの追加
- ✅ タスクの完了/未完了切り替え  
- ✅ タスクの削除
- ✅ タスク統計表示
- ✅ データの永続化
- ✅ レスポンシブデザイン

## 🚀 起動方法

### 必要な環境

- Node.js (v14.0.0 以上)

### インストールと起動

1. **依存関係のインストール**
   ```bash
   npm install
   ```

2. **サーバー起動**
   ```bash
   npm start
   ```
   
   または
   
   ```bash
   node server.js
   ```

3. **起動スクリプト使用 (Unix/Linux/macOS)**
   ```bash
   ./run.sh
   ```

4. **ブラウザでアクセス**
   ```
   http://localhost:3000
   ```

## 📁 プロジェクト構造

```
todo_app/
├── index.html      # メインHTMLファイル
├── styles.css      # スタイルシート
├── app.js          # フロントエンドJavaScript
├── server.js       # Express サーバー
├── package.json    # Node.js 依存関係
├── run.sh          # 起動スクリプト
├── todos.json      # データファイル (自動生成)
└── README.md       # このファイル
```

## 🔧 API エンドポイント

- `GET /api/todos` - 全TODOを取得
- `POST /api/todos` - 新しいTODOを作成
- `PUT /api/todos/:id` - TODOを更新
- `DELETE /api/todos/:id` - TODOを削除

## 🎨 特徴

- **マルチエージェント開発**: 各専門分野のエージェントが協調して開発
- **完全な動作**: 実際にブラウザでアクセス可能
- **データ永続化**: サーバー再起動後もデータを保持
- **エラーハンドリング**: API障害時はローカルストレージを使用

## 🤖 ccswarm について

このアプリケーションは ccswarm マルチエージェントシステムによって開発されました。ccswarmは以下の特徴を持つ開発システムです：

- **エージェント特化**: 各エージェントが専門分野に特化
- **協調開発**: エージェント間での自動的なタスク振り分け
- **品質保証**: 専門性に基づく品質管理
- **効率的開発**: 並列作業による高速開発

## 📄 ライセンス

MIT License

---

🎉 **ccswarm multi-agent system で開発完了！**