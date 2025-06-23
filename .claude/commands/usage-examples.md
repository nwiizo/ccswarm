# 使用例集

## 基本的な使用方法

### 1. プロジェクトの初期化
```bash
# 新しいプロジェクトを作成
ccswarm init --name "MyTodoApp" --agents frontend,backend,devops

# 既存プロジェクトで初期化
cd my-existing-project
ccswarm init --name "ExistingProject" --agents frontend,backend,qa
```

### 2. システムの起動
```bash
# オーケストレーターを起動
ccswarm start

# デーモンモードで起動
ccswarm start --daemon

# 本物のAI APIを使用
ccswarm start --use-real-api

# コンテナ分離モード
ccswarm start --isolation container
```

### 3. ターミナルUIでのモニタリング
```bash
# TUIを起動
ccswarm tui

# システム状態を確認
ccswarm status

# 詳細状態を表示
ccswarm status --detailed

# 特定エージェントの状態
ccswarm status --agent frontend-agent-123
```

## タスク管理

### タスクの追加
```bash
# 基本的なタスク
ccswarm task "Implement user authentication"

# 優先度とタイプを指定
ccswarm task "Fix critical security bug" --priority high --type bugfix

# タスク修飾子を使用
ccswarm task "Add authentication [high] [feature] [auto]"

# 複数のタスクを一度に追加
ccswarm task "Create user model, Add auth endpoints, Write tests" --split
```

### タスクの参照と管理
```bash
# タスク一覧を表示
ccswarm task list

# ステータス別にフィルタリング
ccswarm task list --status pending
ccswarm task list --status completed
ccswarm task list --status failed

# 優先度別にフィルタリング
ccswarm task list --priority high

# タイプ別にフィルタリング
ccswarm task list --type feature
ccswarm task list --type bugfix
```

### 直接委託
```bash
# 特定エージェントに直接委託
ccswarm delegate task "Create React component" --agent frontend
ccswarm delegate task "Setup database" --agent backend
ccswarm delegate task "Configure CI/CD" --agent devops

# エージェントIDで指定
ccswarm delegate task "Review code" --agent-id frontend-agent-abc123
```

## 自動アプリケーション作成

### シンプルなアプリ作成
```bash
# TODOアプリを作成
ccswarm auto-create "Create a todo app with React and Node.js" --output ./todo-app

# ブログアプリを作成
ccswarm auto-create "Build a blog with authentication" --output ./blog-app --agents frontend,backend,qa

# Eコマースサイトを作成
ccswarm auto-create "E-commerce site with payment integration" \
  --output ./ecommerce \
  --agents frontend,backend,devops,qa \
  --features "user auth,payment,inventory"
```

### 詳細オプション
```bash
# 技術スタックを指定
ccswarm auto-create "REST API for inventory management" \
  --output ./api \
  --tech-stack "Rust,PostgreSQL,Docker" \
  --agent backend

# テストとドキュメントを含む
ccswarm auto-create "React dashboard with charts" \
  --output ./dashboard \
  --include-tests \
  --include-docs \
  --agent frontend
```

## 品質レビューシステム

### レビュー状態の確認
```bash
# レビューシステムの状態を確認
ccswarm review status

# 失敗したレビューの履歴
ccswarm review history --failed

# 成功したレビューの統計
ccswarm review stats --success
```

### 手動レビュー
```bash
# 全エージェントの作業をレビュー
ccswarm review trigger --all

# 特定エージェントの作業をレビュー
ccswarm review trigger --agent frontend-agent-123

# 特定ファイルをレビュー
ccswarm review file src/main.rs
```

### レビュー基準の設定
```bash
# テストカバレッジ基準を変更
ccswarm review config --test-coverage 90

# 複雑度基準を変更
ccswarm review config --max-complexity 8

# レビュー間隔を変更
ccswarm review config --interval 60  # 60秒ごと
```

## セッション管理

### セッションの状態確認
```bash
# アクティブセッション一覧
ccswarm session list

# セッション統計（トークン削減量等）
ccswarm session stats

# トークン節約量を表示
ccswarm session stats --show-savings
```

### セッションの管理
```bash
# 特定エージェントのセッションをリセット
ccswarm session reset --agent frontend-agent-123

# 全セッションをリセット
ccswarm session reset --all

# セッションのバックアップ
ccswarm session backup --output ./session-backup.json

# セッションの復元
ccswarm session restore ./session-backup.json
```

## ログとデバッグ

### ログの参照
```bash
# 最新のログを表示
ccswarm logs

# 特定行数のログ
ccswarm logs --tail 100

# フィルタリング
ccswarm logs --filter error,warning
ccswarm logs --filter "task execution"

# リアルタイムログ
ccswarm logs --follow
```

### デバッグモード
```bash
# デバッグログ付きで起動
RUST_LOG=debug ccswarm start

# 特定モジュールのデバッグ
RUST_LOG=ccswarm::session=trace ccswarm start

# バックトレース付き
RUST_BACKTRACE=1 ccswarm start
```

## システム管理

### システムの停止
```bash
# 穏やかな停止
ccswarm stop

# 強制停止
ccswarm stop --force

# 特定エージェントのみ停止
ccswarm stop --agent frontend-agent-123
```

### システムのリセット
```bash
# 全システムをリセット
ccswarm reset

# セッションのみリセット
ccswarm reset --sessions-only

# エージェントのみリセット
ccswarm reset --agents-only
```

### 設定の管理
```bash
# 設定を表示
ccswarm config show

# 設定を編集
ccswarm config edit

# 設定をリセット
ccswarm config reset

# 設定を検証
ccswarm config validate
```

## リアルワールドの使用例

### Webアプリケーション開発
```bash
# 1. プロジェクト初期化
ccswarm init --name "EcommerceApp" --agents frontend,backend,devops,qa

# 2. フロントエンド開発
ccswarm task "Create React app with routing [high] [feature]"
ccswarm task "Implement product listing page [medium] [feature]"
ccswarm task "Add shopping cart functionality [high] [feature]"

# 3. バックエンド開発
ccswarm task "Setup REST API with Express [high] [feature]"
ccswarm task "Implement user authentication [critical] [feature]"
ccswarm task "Add payment processing [high] [feature]"

# 4. インフラ設定
ccswarm task "Setup Docker containers [medium] [infrastructure]"
ccswarm task "Configure CI/CD pipeline [medium] [infrastructure]"

# 5. テスト
ccswarm task "Write unit tests for all components [high] [testing]"
ccswarm task "Implement E2E tests [medium] [testing]"
```

### バグ修正ワークフロー
```bash
# 1. バグレポート
ccswarm task "Fix login issue - users can't authenticate [critical] [bugfix]"

# 2. 特定エージェントに委託
ccswarm delegate task "Debug authentication flow" --agent backend

# 3. 進捗状態をモニタリング
ccswarm status --agent backend-agent-123

# 4. 修正後のテスト
ccswarm task "Test login functionality after bug fix [high] [testing]"
```

### コードレビューワークフロー
```bash
# 1. 定期レビューをトリガー
ccswarm review trigger --all

# 2. 失敗したレビューを確認
ccswarm review history --failed

# 3. 改善タスクを確認
ccswarm task list --type remediation

# 4. 改善タスクを実行
ccswarm task "Add unit tests for UserService [auto] [testing] [remediation]"
```

## トラブルシューティング

### エージェントが応答しない場合
```bash
# エージェントの状態を確認
ccswarm status --agent stuck-agent-123

# エージェントをリスタート
ccswarm restart --agent stuck-agent-123

# セッションをリセット
ccswarm session reset --agent stuck-agent-123
```

### パフォーマンス問題
```bash
# セッション統計を確認
ccswarm session stats

# メモリ使用量を確認
ccswarm stats --memory

# 不要なセッションをクリーンアップ
ccswarm session cleanup
```

### 設定問題
```bash
# 設定を検証
ccswarm config validate

# デフォルト設定にリセット
ccswarm config reset

# 設定ファイルを直接編集
vim ccswarm.json
ccswarm config validate
```