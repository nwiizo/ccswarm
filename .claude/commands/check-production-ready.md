# プロダクションレディチェック

ccswarm のプロダクション品質基準を確認します。

## チェック項目

CLAUDE.md に基づく7つの品質基準:

| # | 項目 | 基準 | チェックコマンド |
|---|-----|------|-----------------|
| 1 | unwrap() 排除 | プロダクションコードで使用禁止 | `grep -r "\.unwrap()" crates/ccswarm/src/` |
| 2 | Result/Error handling | thiserror でカスタムエラー型 | `grep -r "thiserror" crates/` |
| 3 | Async patterns | tokio ランタイム、async-trait | `cargo check` |
| 4 | Documentation | 公開 API に rustdoc | `cargo doc --workspace` |
| 5 | Clippy clean | 警告なし | `cargo clippy --workspace -- -D warnings` |
| 6 | Channel-Based | Arc<Mutex> より Channel 優先 | `grep -r "Arc<Mutex" crates/` |
| 7 | Minimal tests | 8-10 テスト程度 | `cargo test --workspace 2>&1 | grep "test result"` |

## 実行方法

```bash
# 1. unwrap() チェック
echo "=== unwrap() count ==="
grep -r "\.unwrap()" crates/ccswarm/src/ --include="*.rs" | grep -v "test" | wc -l

# 2. エラーハンドリング確認
echo "=== thiserror usage ==="
grep -r "use thiserror" crates/

# 3. Clippy チェック
echo "=== Clippy check ==="
cargo clippy --workspace -- -D warnings

# 4. ドキュメント生成
echo "=== Documentation ==="
cargo doc --workspace --no-deps

# 5. Arc<Mutex> カウント
echo "=== Arc<Mutex> count ==="
grep -r "Arc<Mutex" crates/ccswarm/src/ | wc -l

# 6. テスト数確認
echo "=== Test count ==="
cargo test --workspace 2>&1 | grep "test result"
```

## 出力形式

```json
{
  "production_ready": {
    "unwrap_elimination": {
      "status": "OK|NG",
      "count": N,
      "locations": ["問題箇所"]
    },
    "error_handling": {
      "status": "OK|NG",
      "thiserror_used": true|false,
      "custom_errors": ["ErrorType1", "ErrorType2"]
    },
    "async_patterns": {
      "status": "OK|NG",
      "tokio_version": "1.x",
      "async_trait_used": true|false
    },
    "documentation": {
      "status": "OK|NG",
      "public_api_coverage": "N%"
    },
    "clippy_clean": {
      "status": "OK|NG",
      "warnings": N,
      "errors": N
    },
    "channel_based": {
      "status": "OK|NG",
      "arc_mutex_count": N,
      "channel_usage": N,
      "ratio": "channels/arc_mutex"
    },
    "minimal_testing": {
      "status": "OK|NG",
      "test_count": N,
      "target_range": "8-10"
    }
  },
  "score": "N/7",
  "overall_status": "PRODUCTION_READY|NEEDS_WORK|CRITICAL"
}
```

## 判定基準

| スコア | 判定 | 説明 |
|-------|------|------|
| 7/7 | PRODUCTION_READY | 本番環境にデプロイ可能 |
| 5-6/7 | NEEDS_WORK | 軽微な改善が必要 |
| 0-4/7 | CRITICAL | 重大な改善が必要 |

## 使用例

```
subagent_type: "rust-fix-agent"
prompt: "ccswarm のプロダクションレディチェックを実行してください。
7つの基準それぞれについて確認し、問題があれば修正してください。
結果をJSON形式でレポートしてください。"
```

## 関連

- `/check-impl` - 基本チェック
- `/review-all` - 全体レビュー
- `.claude/agents/rust-fix-agent.md` - Rust 修正エージェント
