# Epic: Claude Code ACP Integration MVP

## 概要
Claude CodeとccswarmをAgent Client Protocol (ACP)で統合し、タスクの送信と結果の受信を可能にする最小限の実装を行う。

## ゴール
- [ ] ccswarmからClaude CodeにACPプロトコル経由でタスクを送信できる
- [ ] Claude Codeの実行結果をccswarmで受け取れる
- [ ] 既存のccswarm機能を壊さない

## タイムライン
**目標期間**: 1-2週間
- Week 1: 基礎実装 (Day 1-5)
- Week 2: 統合と改善 (Day 6-10)

## 成功の定義
```bash
# これが動作すること
ccswarm claude-acp start
ccswarm task "Create a simple TODO app" --via-acp
```

## アーキテクチャ
```
┌──────────────────┐    ACP/JSON-RPC    ┌─────────────────┐
│  ccswarm Master  │◄──────────────────►│  Claude Code    │
│  (ACP Client)    │                    │  (ACP Server)   │
└──────────────────┘                    └─────────────────┘
         └── localhost:9100 ──────────────┘
```

## 関連Issues
- #1 Day 1: プロジェクト設定と依存関係追加
- #2 Day 2-3: Claude Code ACPアダプター実装
- #3 Day 4-5: CLIコマンド実装
- #4 Day 6-7: 既存システムとの統合
- #5 Day 8: エラーハンドリング強化
- #6 Day 9: ユニットテストとドキュメント
- #7 Day 10: 統合テストとデモ

## ラベル
- `epic`
- `claude-acp`
- `mvp`
- `enhancement`