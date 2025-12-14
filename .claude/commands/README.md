# Claude Code Commands

ccswarm プロジェクト用のスラッシュコマンドです。

## コマンド一覧

| コマンド | 説明 |
|---------|------|
| `/review-all` | 全体レビュー（設計準拠・品質・重複検出） |
| `/review-duplicates` | 重複コード検出（similarity-rs によるリファクタリング候補特定） |
| `/review-architecture` | アーキテクチャレビュー（Type-State、Channel-Based等のパターン準拠確認） |
| `/check-impl` | 実装チェック（fmt, clippy, test） |
| `/check-production-ready` | プロダクションレディチェック（Rust ベストプラクティス） |

## 使い方

Claude Code で以下のように実行:

```
/review-all
```

## フロー

### 開発フロー

1. 開発作業
2. `/check-impl` - 基本チェック
3. `/review-duplicates` - 重複コード検出
4. `/review-all` - 全体レビュー

### レビューフロー

`/review-all` は以下を実行:
- CLAUDE.md 設計準拠確認
- Rust ベストプラクティス確認
- 重複コード検出（`/review-duplicates`）
- アーキテクチャパターン準拠確認

## 関連

- `.claude/agents/` - エージェント定義
- `CLAUDE.md` - プロジェクトガイドライン
- `docs/ARCHITECTURE.md` - アーキテクチャ設計
