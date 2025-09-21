# Issue #5: Day 8 - エラーハンドリング強化

## 概要
Claude Code ACP統合のエラーハンドリングとリトライロジックを強化する。

## タスクリスト

### 1. エラー型の定義
- [ ] `ACPError`列挙型の実装:
```rust
#[derive(Error, Debug)]
pub enum ACPError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tungstenite::Error),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    #[error("Claude Code not running")]
    ServiceNotAvailable,
}
```

### 2. リトライメカニズム
- [ ] 指数バックオフの実装
- [ ] 最大リトライ回数の設定
- [ ] リトライ間隔の設定

### 3. 接続監視
- [ ] ハートビート機能の実装
- [ ] 自動再接続の実装
- [ ] 接続状態の監視

### 4. ユーザー向けエラーメッセージ
- [ ] わかりやすいエラーメッセージ
- [ ] トラブルシューティングのヒント
- [ ] 診断コマンドの実装

### 5. ロギング
- [ ] デバッグレベルのログ追加
- [ ] エラー時の詳細情報記録
- [ ] パフォーマンスメトリクス

## 受け入れ基準
- [ ] 接続失敗時に自動リトライが動作する
- [ ] エラーメッセージが分かりやすい
- [ ] `RUST_LOG=debug`で詳細なログが出力される
- [ ] 接続が切れても自動復旧する

## 見積もり時間
6-8時間

## ラベル
- `task`
- `day-8`
- `error-handling`
- `claude-acp`