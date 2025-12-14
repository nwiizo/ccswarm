# 実装チェック

ccswarm ワークスペースに対して基本チェックを実行します。

## 実行内容

```bash
# 1. フォーマット・リント
cargo fmt --all --check && cargo clippy --workspace -- -D warnings

# 2. テスト
cargo test --workspace

# 3. ビルド確認
cargo build --workspace --release
```

## チェック項目

| 項目 | コマンド | 基準 |
|-----|---------|------|
| フォーマット | `cargo fmt --all --check` | エラーなし |
| リント | `cargo clippy --workspace -- -D warnings` | 警告なし |
| テスト | `cargo test --workspace` | 全パス |
| ビルド | `cargo build --workspace` | エラーなし |

## Rust 2024 Edition 対応

ccswarm は Rust 2024 Edition を使用しています。以下の点に注意:

- `std::env::set_var` は `unsafe` ブロックが必要
- パターンマッチングで暗黙の借用が行われるため `ref`/`ref mut` は不要
- 明示的な型注釈が必要な場面が増加

## 出力形式

```json
{
  "format": "OK|NG",
  "clippy": {
    "warnings": N,
    "errors": N
  },
  "test": {
    "passed": N,
    "failed": N,
    "ignored": N
  },
  "build": "OK|NG",
  "overall": "OK|NG"
}
```

## 関連

- `/review-all` - 全体レビュー（設計準拠・品質含む）
- `/review-duplicates` - 重複コード検出
