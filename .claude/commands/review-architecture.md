# アーキテクチャレビュー

ccswarm のアーキテクチャパターン準拠を詳細にレビューします。

## 実行内容

CLAUDE.md と docs/ARCHITECTURE.md に基づき、以下のパターン準拠を確認:

1. **Type-State Pattern** - コンパイル時状態検証
2. **Channel-Based Orchestration** - メッセージパッシング
3. **Iterator Pipelines** - ゼロコスト抽象化
4. **Actor Model** - ロックフリー設計
5. **Minimal Testing** - 必要最小限のテスト

## チェック項目

### 1. Type-State Pattern

```rust
// 期待: コンパイル時に状態遷移を検証
struct Agent<S: State> {
    state: PhantomData<S>,
    // ...
}

impl Agent<Uninitialized> {
    fn initialize(self) -> Agent<Ready> { ... }
}

impl Agent<Ready> {
    fn execute(self, task: Task) -> Agent<Running> { ... }
}
```

検索パターン:
```bash
grep -r "PhantomData" crates/
grep -r "impl.*<.*State>" crates/
```

### 2. Channel-Based Orchestration

```rust
// 期待: Arc<Mutex> より Channel 優先
let (tx, rx) = tokio::sync::mpsc::channel(100);

// 避けるべき:
// let shared = Arc::new(Mutex::new(state));
```

検索パターン:
```bash
grep -r "Arc<Mutex" crates/ | wc -l  # 少ないほど良い
grep -r "mpsc::channel\|broadcast::channel" crates/
```

### 3. Iterator Pipelines

```rust
// 期待: iterator chains でゼロコスト抽象化
let results: Vec<_> = items
    .iter()
    .filter(|x| x.active)
    .map(|x| process(x))
    .collect();

// 避けるべき:
// for item in items { if item.active { ... } }
```

### 4. Actor Model

```rust
// 期待: 各エージェントが独立したアクター
struct AgentActor {
    receiver: mpsc::Receiver<Message>,
}

impl AgentActor {
    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle(msg).await;
        }
    }
}
```

### 5. Minimal Testing

| 基準 | 期待値 |
|-----|-------|
| 総テスト数 | 8-10程度 |
| 統合テスト | コア機能のみ |
| 単体テスト | 複雑なロジックのみ |

## 出力形式

```json
{
  "patterns": {
    "type_state": {
      "status": "OK|PARTIAL|NG",
      "usage_count": N,
      "examples": ["Agent<Ready>", "Session<Connected>"],
      "missing": ["推奨される適用箇所"]
    },
    "channel_based": {
      "status": "OK|PARTIAL|NG",
      "channel_count": N,
      "arc_mutex_count": N,
      "refactor_candidates": ["共有状態の Channel 化候補"]
    },
    "iterator_pipelines": {
      "status": "OK|PARTIAL|NG",
      "iterator_usage": N,
      "loop_usage": N,
      "refactor_candidates": ["Iterator 化候補"]
    },
    "actor_model": {
      "status": "OK|PARTIAL|NG",
      "actor_count": N,
      "message_types": ["TaskMessage", "StatusMessage"]
    },
    "minimal_testing": {
      "status": "OK|PARTIAL|NG",
      "test_count": N,
      "integration_tests": N,
      "unit_tests": N,
      "recommendation": "テスト追加/削除の推奨"
    }
  },
  "score": "N/5",
  "recommendations": ["優先対応事項"]
}
```

## 使用例

```
subagent_type: "Explore"
prompt: "ccswarm のアーキテクチャパターン準拠をレビューしてください。
1. Type-State Pattern の使用状況
2. Channel-Based vs Arc<Mutex> の比率
3. Iterator Pipelines の活用度
4. Actor Model の実装状況
5. テスト数と品質
JSON形式でレポートを作成してください。"
```

## 関連

- `/review-all` - 全体レビュー
- `CLAUDE.md` - アーキテクチャガイドライン
- `docs/ARCHITECTURE.md` - 詳細アーキテクチャ
