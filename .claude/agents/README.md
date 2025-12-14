# Claude Code Agents

ccswarm プロジェクト用のサブエージェントです。

## エージェント一覧

| エージェント | 説明 | コマンド |
|-------------|------|---------|
| `all-reviewer` | 全レビュー統合（設計準拠・品質・アーキテクチャ） | `/review-all` |
| `architecture-reviewer` | アーキテクチャパターン準拠レビュー | `/review-architecture` |
| `rust-fix-agent` | Rust ビルド・clippy エラー修正 | `/check-impl` |
| `code-refactor-agent` | 重複コード検出・リファクタリング | `/review-duplicates` |

## 使い方

Task ツールで呼び出します:

```
subagent_type: "エージェント名"
prompt: "[タスク内容]"
```

## エージェント詳細

### all-reviewer

全てのレビューを一括実行する統合エージェント。

**チェック項目:**
- CLAUDE.md 設計準拠
- docs/ARCHITECTURE.md アーキテクチャ準拠
- Rust ベストプラクティス
- 重複コード検出

**出力**: 設計準拠、コード品質、アーキテクチャの統合レポート（JSON）

### architecture-reviewer

アーキテクチャパターン専門レビュー。

**チェック項目:**
- Type-State Pattern の使用状況
- Channel-Based vs Arc<Mutex> の比率
- Iterator Pipelines の活用度
- Actor Model の実装状況
- Minimal Testing の準拠

**出力**: 各パターンの評価とスコア（JSON）

### rust-fix-agent

Rust のビルドエラーと clippy 警告を修正する専門エージェント。

**機能:**
- コンパイルエラーの修正
- Clippy 警告の対処
- Rust 2024 Edition 対応

**原則**: YAGNI（You Aren't Gonna Need It）- 必要最小限の修正

### code-refactor-agent

重複コード検出とリファクタリングを行うエージェント。

**機能:**
- similarity-rs による意味的類似コード検出
- DRY 原則に基づくリファクタリング提案
- ccswarm パターンへの変換

**検出カテゴリ:**
| カテゴリ | 検出パターン |
|---------|------------|
| 共通関数抽出 | 類似度 95%+、10行以上 |
| トレイト化 | 類似度 90-95%、5行以上 |
| Channel 化 | Arc<Mutex> の共有状態 |

## モデル設定

| エージェント | モデル | 理由 |
|-------------|-------|------|
| all-reviewer | sonnet | バランスの取れたレビュー |
| architecture-reviewer | sonnet | パターン分析に適切 |
| rust-fix-agent | opus | 複雑な修正に高精度 |
| code-refactor-agent | opus | セマンティック分析に高精度 |

## 関連

- `.claude/commands/` - スラッシュコマンド定義
- `CLAUDE.md` - プロジェクトガイドライン
- `docs/ARCHITECTURE.md` - アーキテクチャ設計
