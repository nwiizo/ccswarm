# Issue #4: Day 6-7 - 既存システムとの統合

## 概要
Claude Code ACPをccswarmの既存のタスクシステムと統合する。

## タスクリスト

### 1. MasterClaudeへの統合
- [ ] `src/orchestrator/mod.rs`を修正:
```rust
pub struct MasterClaude {
    #[cfg(feature = "claude-acp")]
    claude_acp: Option<Arc<Mutex<ClaudeCodeAdapter>>>,
}
```

### 2. タスク委譲ロジック
- [ ] `delegate_task()`メソッドの修正
- [ ] Claude Code優先モードの実装
- [ ] `delegate_to_claude_acp()`メソッドの追加

### 3. 設定ファイルサポート
- [ ] `.ccswarm.yml`の読み込み
```yaml
claude_acp:
  enabled: true
  url: "ws://localhost:9100"
  auto_connect: true
  prefer_claude: true
```

### 4. 環境変数サポート
- [ ] `CCSWARM_CLAUDE_ACP_URL`
- [ ] `CCSWARM_CLAUDE_ACP_ENABLED`
- [ ] `CCSWARM_CLAUDE_ACP_AUTO_CONNECT`

### 5. 既存コマンドの拡張
- [ ] `ccswarm task`に`--via-acp`フラグを追加
- [ ] `ccswarm status`にACP状態を追加

## 受け入れ基準
- [ ] 既存の機能が壊れない
- [ ] Claude Code経由でタスクが実行できる
- [ ] 設定ファイルが正しく読み込まれる
- [ ] 環境変数が反映される

## 見積もり時間
8-10時間

## ラベル
- `task`
- `day-6-7`
- `integration`
- `claude-acp`