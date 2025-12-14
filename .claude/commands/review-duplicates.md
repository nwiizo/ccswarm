# 重複コード検出

similarity-rs を使用してコードベース内の意味的に類似したコードを検出し、リファクタリング候補を特定します。

## 実行方法

```bash
# 基本実行（crates/ ディレクトリを解析）
similarity-rs crates/

# 類似度閾値を指定（デフォルト: 0.85）
similarity-rs crates/ --threshold 0.80

# コードを出力に含める
similarity-rs crates/ --print

# 最小行数を指定（デフォルト: 3行）
similarity-rs crates/ --min-lines 5
```

## オプション

| オプション | 説明 | デフォルト |
|-----------|------|-----------|
| `--threshold` | 類似度閾値（0.0-1.0） | 0.85 |
| `--print` | コードを出力に含める | false |
| `--min-lines` | 最小行数 | 3 |
| `--min-tokens` | 最小トークン数 | 30 |
| `--extensions` | 対象ファイル拡張子 | rs |

## 検出パターン

### リファクタリング優先度

| 優先度 | 条件 | アクション |
|-------|------|-----------|
| 高 | 類似度 95%+、10行以上 | 共通関数に抽出 |
| 中 | 類似度 90-95%、5行以上 | トレイト化またはマクロ化を検討 |
| 低 | 類似度 85-90% | 構造の類似性を確認、必要に応じて抽出 |

### 許容される重複

以下は重複として検出されても、リファクタリング不要な場合が多い:

1. **テストコード** - テストの独立性を優先
2. **設定構造体** - 類似した設定パターン
3. **エラーハンドリング** - 文脈依存のエラー処理
4. **ClaudeCodeConfig 初期化** - 明示的なフィールド指定が望ましい

## 出力形式

```json
{
  "duplicate_analysis": {
    "total_functions": N,
    "similar_pairs": N,
    "high_priority": [
      {
        "file1": "パス1",
        "file2": "パス2",
        "lines1": "L10-L30",
        "lines2": "L50-L70",
        "similarity": "97%",
        "action": "共通関数に抽出"
      }
    ],
    "medium_priority": [],
    "low_priority": [],
    "ignored": [
      {
        "reason": "テストコード",
        "count": N
      }
    ]
  }
}
```

## ccswarm 特有のリファクタリング戦略

### 1. Channel-Based パターンへの統合

```rust
// Before: Arc<Mutex> を使用した共有状態
let shared_state = Arc::new(Mutex::new(State::new()));

// After: Channel-Based に統合
let (tx, rx) = tokio::sync::mpsc::channel(100);
```

### 2. Type-State パターンの活用

```rust
// Before: ランタイム状態チェック
if self.state == State::Connected { ... }

// After: コンパイル時検証
impl<S: ConnectionState> Agent<S> {
    fn send(self: Agent<Connected>) -> Result<Response> { ... }
}
```

### 3. Iterator Pipelines での統合

```rust
// Before: 複数箇所で同じフィルタリング
let items: Vec<_> = items.iter().filter(|x| x.active).collect();

// After: 共通イテレータアダプタ
trait ActiveFilter {
    fn active_only(self) -> impl Iterator<Item = Self::Item>;
}
```

## 注意事項

### リファクタリングすべきでないケース

1. **可読性の低下** - 抽象化によりコードが理解しにくくなる
2. **過度な DRY** - 3回未満の重複は許容（YAGNI原則）
3. **異なる変更理由** - 将来的に別々に変更される可能性
4. **テストの独立性** - テストコードは重複を許容

### リファクタリングすべきケース

1. **バグ修正の重複** - 同じ修正を複数箇所に適用する必要がある
2. **ビジネスロジック** - 同じルールが複数箇所に散在
3. **Provider 実装** - 共通の ProviderExecutor パターン

## 使用例

```
subagent_type: "code-refactor-agent"
prompt: "similarity-rs crates/ を実行して重複コードを検出してください。
検出結果を分析し、以下を報告してください:
1. 高優先度のリファクタリング候補（類似度95%以上）
2. 中優先度の候補（類似度90-95%）
3. 無視すべき重複（テストコード等）
4. ccswarm パターンに基づくリファクタリング提案"
```

## 関連

- `/review-all` - 全体レビュー（重複検出含む）
- `.claude/agents/code-refactor-agent.md` - リファクタリングエージェント
