# 全体レビュー

ccswarm コードベースに対して全てのレビューを一括実行します。

## 実行内容

以下のレビューを実行します:

1. **設計準拠チェック** - CLAUDE.md, docs/ARCHITECTURE.md との整合性
2. **コード品質** - Rust ベストプラクティス準拠
3. **重複コード検出** - `/review-duplicates` コマンドを実行（similarity-rs）
4. **アーキテクチャパターン** - Type-State, Channel-Based 等のパターン準拠

## 実行方法

Task ツールで `all-reviewer` エージェントを実行:

```
subagent_type: "Explore"
prompt: "ccswarm に対して全体レビューを実行してください。
1. CLAUDE.md との設計準拠確認
2. docs/ARCHITECTURE.md アーキテクチャパターン準拠確認
3. Rust ベストプラクティス準拠確認
4. /review-duplicates - 重複コード検出
各カテゴリの結果をJSON形式でまとめてください。"
```

## アーキテクチャパターンチェック項目

CLAUDE.md に基づくパターン:

| # | パターン | 基準 |
|---|---------|------|
| 1 | Type-State Pattern | コンパイル時状態検証、ゼロランタイムコスト |
| 2 | Channel-Based Orchestration | Arc<Mutex> より Channel 優先 |
| 3 | Iterator Pipelines | ゼロコスト抽象化 |
| 4 | Actor Model | ロックよりメッセージパッシング |
| 5 | Minimal Testing | 8-10 テスト程度、コア機能に集中 |

## Rust ベストプラクティスチェック項目

| # | 項目 | 基準 |
|---|-----|------|
| 1 | unwrap() 排除 | プロダクションコードで使用禁止 |
| 2 | Result<T, E> | thiserror でカスタムエラー型 |
| 3 | async/await | tokio ランタイム使用 |
| 4 | Clippy clean | 警告なし |
| 5 | Documentation | 公開 API に rustdoc |

## 出力形式

```json
{
  "compliance": {
    "claude_md": {"compliant": N, "partial": N, "non_compliant": N},
    "architecture": {"compliant": N, "partial": N, "non_compliant": N}
  },
  "code_quality": {
    "rust": {
      "clippy_warnings": N,
      "unsafe_count": N,
      "unwrap_count": N
    },
    "similarity": {
      "duplicate_patterns": N,
      "refactoring_candidates": ["候補1", "候補2"]
    }
  },
  "architecture_patterns": {
    "type_state": "OK|NG",
    "channel_based": "OK|NG",
    "iterator_pipelines": "OK|NG",
    "actor_model": "OK|NG",
    "minimal_testing": "OK|NG",
    "score": "N/5"
  },
  "summary": {
    "overall_status": "OK|WARNING|CRITICAL",
    "priority_actions": []
  }
}
```

## 関連

- `/check-impl` - 基本チェック（フォーマット、リント、テスト）
- `/review-duplicates` - 重複コード検出（similarity-rs）
- `/review-architecture` - アーキテクチャパターン詳細レビュー
