# Issue #2: Day 2-3 - Claude Code ACPアダプター実装

## 概要
Claude Codeと通信するためのACPアダプターを実装する。

## タスクリスト

### 1. ClaudeCodeAdapterの実装
- [ ] `adapter.rs`に以下を実装:
```rust
pub struct ClaudeCodeAdapter {
    connection: Option<ClientSideConnection>,
    session_id: Option<String>,
    config: ClaudeACPConfig,
}
```

### 2. 接続管理
- [ ] `connect()`メソッドの実装
  - [ ] WebSocket接続の確立
  - [ ] ACPプロトコルの初期化
  - [ ] セッション作成
- [ ] `disconnect()`メソッドの実装
- [ ] `is_connected()`メソッドの実装

### 3. タスク送信
- [ ] `send_task()`メソッドの実装
  - [ ] PromptRequestの作成
  - [ ] レスポンスの処理
  - [ ] エラーハンドリング

### 4. 自動リトライ機能
- [ ] `connect_with_retry()`メソッドの実装
- [ ] 指数バックオフの実装
- [ ] タイムアウト処理

### 5. 設定管理
- [ ] 環境変数からの設定読み込み
- [ ] デフォルト値の設定
- [ ] 設定のバリデーション

## テストケース
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_adapter_creation() { /* ... */ }

    #[tokio::test]
    async fn test_config_from_env() { /* ... */ }

    #[tokio::test]
    async fn test_retry_logic() { /* ... */ }
}
```

## 受け入れ基準
- [ ] アダプターが正しく初期化される
- [ ] モックサーバーへの接続が成功する
- [ ] タスクの送受信が動作する
- [ ] リトライロジックが機能する

## 見積もり時間
8-12時間

## ラベル
- `task`
- `day-2-3`
- `implementation`
- `claude-acp`