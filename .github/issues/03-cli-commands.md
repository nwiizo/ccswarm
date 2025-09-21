# Issue #3: Day 4-5 - CLIコマンド実装

## 概要
Claude Code ACPを操作するためのCLIコマンドを実装する。

## タスクリスト

### 1. CLIコマンド構造の追加
- [ ] `src/cli/mod.rs`に以下を追加:
```rust
#[derive(Subcommand)]
pub enum ClaudeACPCommands {
    Start { url: Option<String> },
    Test,
    Send { task: String },
    Stop,
    Status,
    Diagnose,
}
```

### 2. コマンドハンドラーの実装
- [ ] `claude-acp start` - ACP接続を開始
- [ ] `claude-acp test` - 接続テスト
- [ ] `claude-acp send` - タスク送信
- [ ] `claude-acp stop` - 接続終了
- [ ] `claude-acp status` - 現在の状態表示
- [ ] `claude-acp diagnose` - トラブルシューティング

### 3. 出力フォーマット
- [ ] カラフルな出力（絵文字付き）
- [ ] プログレスバー表示
- [ ] エラーメッセージの整形

### 4. ヘルプメッセージ
- [ ] 各コマンドの詳細なヘルプ
- [ ] 使用例の追加
- [ ] トラブルシューティングガイド

## 受け入れ基準
- [ ] `ccswarm claude-acp --help`でヘルプが表示される
- [ ] 各コマンドが正しく動作する
- [ ] エラー時に適切なメッセージが表示される

## 見積もり時間
8-10時間

## ラベル
- `task`
- `day-4-5`
- `cli`
- `claude-acp`