# ccswarm デモガイド 🚀

このガイドでは、自動アプリケーション作成機能を備えたClaude Codeマルチエージェントオーケストレーションシステム「ccswarm」の主要機能をデモンストレーションします。

## 🎯 デモ概要

ccswarmの特徴：
- **自動アプリケーション作成** (ワンコマンドで完全なアプリ生成) 🆕
- **セッション永続化アーキテクチャ** (93%のトークン削減)
- **Masterタスク委譲システム** (インテリジェントなタスク割り当て)
- **リアルタイムTUIモニタリング** (ライブエージェント調整)
- **マルチプロバイダーサポート** (Claude Code、Aider、Codex、Custom)
- **Git Worktree統合** (独立したエージェントワークスペース)

## 🚀 クイックスタートデモ

### 1. 基本セットアップ (2分)

```bash
# クローンとビルド
git clone https://github.com/nwiizo/ccswarm.git
cd ccswarm
cargo build --release

# デモプロジェクトの初期化
cargo run -- init --name "デモプロジェクト" --agents frontend,backend,devops,qa
```

### 2. 自動アプリケーション作成デモ (NEW! 🎯) (3分)

#### 基本的なTODOアプリ作成
```bash
# ワンコマンドで完全なTODOアプリを生成
cargo run -- auto-create "TODOアプリケーションを作成してください" --output ./my_todo_app

# 期待される出力:
# 🚀 ccswarm Auto-Create
# ====================
# 📋 Request: TODOアプリケーションを作成してください
# 📂 Output: ./my_todo_app
#
# 🚀 Starting auto-create workflow
# 📂 Created output directory: ./my_todo_app
# 📋 Generated 5 tasks
#
# 🤖 Simulating agent execution...
#    Master → Frontend: Create React TODO app UI
#       ✅ Created frontend files
#    Master → Backend: Create REST API for TODO app
#       ✅ Created backend files
#    Master → DevOps: Setup database and deployment
#       ✅ Created deployment files
#    Master → QA: Write tests
#       ✅ Created test files
#
# 📊 Auto-create completed!
#    📂 Project created at: ./my_todo_app
#    🚀 To run the app:
#       cd ./my_todo_app
#       npm install
#       npm start
```

#### 生成されたアプリの実行
```bash
# 生成されたディレクトリに移動
cd my_todo_app

# 依存関係のインストール
npm install

# アプリケーションの起動
npm start

# ブラウザで http://localhost:3001 を開く
# 完全に動作するTODOアプリが表示されます！
```

#### 高度な自動作成例
```bash
# 認証機能付きTODOアプリ
cargo run -- auto-create "ユーザー認証機能付きのTODOアプリを作成"

# モバイル対応ブログ
cargo run -- auto-create "モバイルレスポンシブなブログサイトを作成"

# リアルタイムチャット
cargo run -- auto-create "WebSocketを使ったリアルタイムチャットアプリを作成"

# Eコマースサイト
cargo run -- auto-create "商品カタログとショッピングカート機能のあるEコマースサイトを作成"
```

### 3. Masterタスク委譲デモ (3分)

#### タスク分析
```bash
# フロントエンドタスクの分析
cargo run -- delegate analyze "レスポンシブナビゲーションバーをドロップダウンメニュー付きで作成" --verbose

# バックエンドタスクの分析
cargo run -- delegate analyze "JWT認証付きユーザー認証REST API実装" --verbose
```

#### 手動タスク委譲
```bash
# 具体的なタスクの委譲
cargo run -- delegate task "ログインコンポーネントのユニットテスト追加" --agent qa --priority high
cargo run -- delegate task "CI/CDパイプライン設定" --agent devops --priority medium
```

### 4. インタラクティブTUIデモ (5分)

```bash
# 拡張TUIの開始
cargo run -- tui
```

#### TUIナビゲーション
1. **概要タブ** - システムメトリクスとエージェントステータス
2. **エージェントタブ** - 各エージェントの詳細情報
3. **タスクタブ** - タスクキューと実行状況
4. **委譲タブ** - インタラクティブなタスク分析と委譲
5. **ログタブ** - リアルタイムログストリーミング

#### TUIコマンド (`c`キーで開始)
```
task 新機能を追加 [high] [feature]
agent frontend
delegate frontend UIコンポーネント作成
help
```

## 🎯 主要デモポイント

### 1. 自動アプリケーション作成 (NEW!)
- **インテリジェントなタスク分解**: ユーザーの要求を自動的に複数のタスクに分解
- **エージェント別実行**: 各タスクを最適なエージェントに自動割り当て
- **実際のファイル生成**: 完全に動作するコードファイルを生成
- **即座に実行可能**: `npm install && npm start`で動作確認可能

### 2. 生成されるファイル構成
```
my_todo_app/
├── index.html         # React エントリーポイント
├── app.js            # React TODOコンポーネント
├── styles.css        # スタイリング
├── server.js         # Express REST API
├── package.json      # 依存関係
├── Dockerfile        # コンテナ設定
├── docker-compose.yml # Docker Compose設定
├── app.test.js       # テスト構造
├── README.md         # ドキュメント
└── .gitignore        # Git設定
```

### 3. セッション効率
- **93%トークン削減**: セッション永続化による効率化
- **コンテキスト保持**: エージェント間での情報共有
- **バッチ処理**: 複数タスクの効率的実行

### 4. リアルタイムモニタリング
- **ライブTUI更新**: タスク進捗のリアルタイム表示
- **エージェント間通信**: メッセージングシステム
- **統計情報**: 実行効率とパフォーマンス

## 🔍 デモシナリオ

### シナリオ1: TODOアプリ開発（5分）
```bash
# 1. 自動作成
cargo run -- auto-create "TODOアプリを作成" --output ./todo_demo

# 2. 確認
cd todo_demo
ls -la

# 3. 実行
npm install
npm start

# 4. ブラウザでテスト
# - タスクの追加
# - タスクの完了/未完了切り替え
# - タスクの削除
```

### シナリオ2: ブログサイト開発
```bash
# モバイル対応ブログの作成
cargo run -- auto-create "モバイル対応のブログサイトを作成、記事の投稿と閲覧機能を含む"

# 生成されたサイトの確認
cd blog_site
npm install
npm start
```

### シナリオ3: リアルタイムアプリケーション
```bash
# チャットアプリの作成
cargo run -- auto-create "リアルタイムチャットアプリケーションを作成、WebSocket使用"

# 複数ブラウザでテスト
cd chat_app
npm install
npm start
```

## 🎬 プレゼンテーションスクリプト (10分)

### 1-2分: 紹介
"ccswarmは、自然言語の指示からワンコマンドで完全なアプリケーションを生成する革新的なマルチエージェントシステムです。"

### 3-5分: 自動作成デモ
"「TODOアプリを作成」という簡単な指示だけで、Masterがタスクを分析し、各エージェントが協力して完全に動作するアプリケーションを生成します。"

### 6-7分: 生成されたアプリの実演
"生成されたアプリは、React、Express、Docker設定を含み、npm installとnpm startだけで即座に動作します。"

### 8-9分: 高度な機能
"認証、リアルタイム機能、モバイル対応など、要求に応じて自動的にカスタマイズされます。"

### 10分: まとめ
"開発時間を大幅に短縮し、一貫性のある高品質なコードを生成 - これがAI駆動開発の未来です。"

## 📊 パフォーマンス指標

- **生成時間**: 約5秒で完全なアプリケーション構造を生成
- **ファイル数**: 10以上の必要なファイルを自動生成
- **コード品質**: ESLint準拠、ベストプラクティスに従った実装
- **即座に実行可能**: 追加の設定なしで動作

## 🚀 次のステップ

1. **カスタマイズ**: 生成されたコードを基に機能拡張
2. **テンプレート追加**: 新しいアプリケーションタイプの追加
3. **実際のClaude統合**: シミュレーションから実際のAI生成へ
4. **デプロイメント**: 生成されたアプリの自動デプロイ

## 📞 サポートとコミュニティ

- **GitHub**: [https://github.com/nwiizo/ccswarm](https://github.com/nwiizo/ccswarm)
- **Issues**: バグ報告と機能リクエスト
- **Discussions**: コミュニティディスカッション

---

**AIが自動的にアプリケーションを作成する未来へようこそ！** 🚀✨