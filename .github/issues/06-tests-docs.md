# Issue #6: Day 9 - ユニットテストとドキュメント

## 概要
Claude Code ACP統合のテストとドキュメントを作成する。

## タスクリスト

### 1. ユニットテスト
- [ ] `adapter.rs`のテスト:
```rust
#[cfg(test)]
mod adapter_tests {
    #[tokio::test]
    async fn test_adapter_creation() { }

    #[tokio::test]
    async fn test_connect() { }

    #[tokio::test]
    async fn test_send_task() { }

    #[tokio::test]
    async fn test_retry_logic() { }
}
```

### 2. 設定テスト
- [ ] 設定ファイルの読み込みテスト
- [ ] 環境変数の読み込みテスト
- [ ] デフォルト値のテスト

### 3. モックサーバー
- [ ] テスト用モックサーバーの作成
- [ ] 各種レスポンスパターンの実装
- [ ] エラーケースのシミュレーション

### 4. ドキュメント
- [ ] README.mdの更新
- [ ] API ドキュメント（rustdoc）
- [ ] 使用例の追加
- [ ] トラブルシューティングガイド

### 5. 設定例
- [ ] `.ccswarm.yml.example`の作成
- [ ] 環境変数の例を`.env.example`に追加

## 受け入れ基準
- [ ] `cargo test --features claude-acp`が成功する
- [ ] テストカバレッジ80%以上
- [ ] `cargo doc --features claude-acp`でドキュメント生成
- [ ] README.mdに使用方法が記載されている

## 見積もり時間
6-8時間

## ラベル
- `task`
- `day-9`
- `testing`
- `documentation`
- `claude-acp`