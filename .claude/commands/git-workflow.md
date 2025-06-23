# Gitワークフロー

## ブランチ戦略

### 基本ブランチ
- `main`: 安定版リリースブランチ
- `develop`: 開発用ブランチ（オプション）

### 機能ブランチ
```bash
# 機能開発
git checkout -b feature/agent-personality

# バグ修正
git checkout -b fix/compilation-errors

# ドキュメント
git checkout -b docs/update-readme

# リファクタリング
git checkout -b refactor/module-structure
```

## Git Worktreeの使用

ccswarmはGit Worktreeを積極的に活用します。

### Worktreeの作成
```bash
# エージェント用worktreeを作成
git worktree add agents/frontend-agent feature/frontend-work

# テスト用worktreeを作成
git worktree add test-env test/integration-fixes
```

### Worktreeの管理
```bash
# Worktree一覧
git worktree list

# 不要なworktreeを削除
git worktree remove agents/old-agent

# 古いworktree情報をクリーンアップ
git worktree prune
```

## コミット規約

### Conventional Commits
```bash
# 機能追加
git commit -m "feat: add whiteboard visualization for agents"

# バグ修正
git commit -m "fix: resolve Docker dependency issues"

# ドキュメント
git commit -m "docs: update API documentation"

# パフォーマンス改善
git commit -m "perf: optimize agent task execution"

# リファクタリング
git commit -m "refactor: extract personality module"

# テスト
git commit -m "test: add personality trait tests"

# ビルド/CI
git commit -m "chore: update dependencies"
```

### 詳細なコミットメッセージ
```bash
git commit -m "feat: integrate AI agent personality system

- Add skill-based experience system
- Implement adaptive working styles
- Create personality traits that influence decisions
- Enable dynamic improvement through tasks

This enables agents to develop unique characteristics
and improve their capabilities over time."
```

## マージ戦略

### Pull Requestマージ
```bash
# featureブランチをmainにマージ
git checkout main
git merge --no-ff feature/agent-personality

# コンフリクト解決後
git add .
git commit -m "merge: integrate agent personality feature"
```

### Squashマージ（クリーンな履歴）
```bash
# GitHub上でPRをSquash and merge
# またはローカルで
git merge --squash feature/experimental
git commit -m "feat: add experimental feature (squashed)"
```

## タグ管理

### バージョンタグ
```bash
# 軽量タグ
git tag v0.3.0

# 注釈付きタグ（推奨）
git tag -a v0.3.0 -m "Release version 0.3.0

Major features:
- Agent personality system
- Whiteboard visualization
- Phronesis implementation"

# タグをpush
git push origin v0.3.0
```

### タグの管理
```bash
# タグ一覧
git tag -l

# 特定パターンのタグを検索
git tag -l "v0.3.*"

# タグの詳細を表示
git show v0.3.0

# タグを削除（ローカル）
git tag -d v0.3.0-beta

# リモートからタグを削除
git push origin :refs/tags/v0.3.0-beta
```

## リベースとチェリーピック

### リベース（履歴を整理）
```bash
# インタラクティブリベース（注意して使用）
git rebase -i HEAD~3

# mainの最新を取り込む
git checkout feature/branch
git rebase main
```

### チェリーピック（特定コミットを適用）
```bash
# 特定のコミットを現在のブランチに適用
git cherry-pick abc123

# 複数のコミットを適用
git cherry-pick abc123..def456
```

## トラブルシューティング

### コンフリクト解決
```bash
# コンフリクトを確認
git status

# ファイルを編集してコンフリクトを解決
# <<<<<<<, =======, >>>>>>>マーカーを削除

# 解決後
git add <resolved-file>
git commit
```

### 変更の取り消し
```bash
# ステージングされていない変更を取り消し
git checkout -- file.rs

# すべての変更を取り消し
git reset --hard

# 直前のコミットを取り消し（push前のみ）
git reset --soft HEAD~1
```

### 失われたコミットの復元
```bash
# reflogで履歴を確認
git reflog

# 特定のコミットに戻る
git reset --hard HEAD@{2}
```

## ベストプラクティス

1. **频繁なコミット**: 小さな変更をこまめにコミット
2. **わかりやすいメッセージ**: 変更内容が明確にわかるメッセージ
3. **ブランチの整理**: 不要なブランチは削除
4. **リモートとの同期**: 定期的にpull/push
5. **.gitignoreの活用**: 不要なファイルをバージョン管理から除外

## ccswarm固有のワークフロー

### エージェント開発フロー
```bash
# 1. エージェント用ブランチを作成
git checkout -b feature/agent-enhancement

# 2. worktreeを作成
git worktree add agents/test-agent feature/agent-enhancement

# 3. 開発とテスト
cd agents/test-agent
cargo test

# 4. コミット
git add .
git commit -m "feat: enhance agent capabilities"

# 5. mainにマージ
cd ../..
git checkout main
git merge --no-ff feature/agent-enhancement
```