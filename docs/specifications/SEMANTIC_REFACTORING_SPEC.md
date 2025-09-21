# ccswarm セマンティックリファクタリング仕様書
## similarity-rs分析結果とserena統合アプローチ

## 1. エグゼクティブサマリー

### 1.1 現状の課題

similarity-rs分析により、ccswarmコードベースに以下の重大な重複が検出されました：

- **60+ 重複メソッド** in `user_error.rs` (83-97% 類似性)
- **85%以上の類似性** を持つ複数のモジュール
- **トークン使用量の非効率性**: 重複コードによる無駄なコンテキスト消費
- **保守性の低下**: 同じロジックの複数実装による一貫性リスク

### 1.2 serenaアプローチの導入

serenaのセマンティック分析手法を導入することで：
- **90%+ トークン効率性**: シンボルレベル操作による劇的な効率化
- **インテリジェントなコード理解**: 必要な部分のみを読み込む
- **永続的な知識管理**: プロジェクトメモリによる長期的な改善
- **自動リファクタリング**: パターン認識と自動修正

## 2. リファクタリング戦略

### 2.1 優先度マトリックス

| モジュール | 重複度 | 影響度 | 優先度 | アクション |
|-----------|--------|--------|--------|------------|
| `user_error.rs` | 97% | Critical | P0 | マクロ化・共通化 |
| `resource/mod.rs` | 96% | High | P1 | トレイト抽出 |
| `backend_search_demo.rs` | 91% | Medium | P2 | パターン統合 |
| `sangha/mod.rs` | 92% | Medium | P2 | 状態機械化 |
| `semantic/knowledge_sharing.rs` | 93% | High | P1 | 知識共有API統一 |

### 2.2 セマンティックリファクタリング手法

#### 2.2.1 エラーハンドリングの統一

**現状の問題:**
```rust
// 60+ 重複メソッドが存在
impl CommonErrors {
    pub fn session_not_found() -> UserError { /* 95% 類似 */ }
    pub fn agent_busy() -> UserError { /* 93% 類似 */ }
    pub fn config_not_found() -> UserError { /* 96% 類似 */ }
    // ... 他57メソッド
}
```

**セマンティック解決策:**
```rust
// マクロによる自動生成
macro_rules! define_error {
    ($name:ident, $code:expr, $message:expr, $category:expr) => {
        pub fn $name() -> UserError {
            UserError::builder()
                .code($code)
                .message($message)
                .category($category)
                .build()
        }
    };
}

// 単一の定義から全エラーを生成
define_errors! {
    session_not_found => (404, "Session not found", ErrorCategory::Resource),
    agent_busy => (503, "Agent is busy", ErrorCategory::Availability),
    config_not_found => (404, "Configuration not found", ErrorCategory::Config),
    // ... 定義のみで実装を自動生成
}
```

#### 2.2.2 リソースモニタリングの抽象化

**現状の問題:**
```rust
// 96% 類似のメソッドが複数存在
impl ResourceMonitor {
    async fn update_all_agents() { /* 96% 類似 */ }
    async fn find_agent_process_usage() { /* 90% 類似 */ }
    async fn get_process_usage() { /* 83% 類似 */ }
}
```

**セマンティック解決策:**
```rust
// ジェネリックトレイトによる統一
trait MonitoringTarget {
    type Metric;
    fn identifier(&self) -> &str;
    fn collect_metric(&self) -> Self::Metric;
}

// 単一の実装で全モニタリング処理
impl<T: MonitoringTarget> Monitor<T> {
    async fn update(&mut self, target: &T) -> Result<()> {
        let metric = target.collect_metric();
        self.store_metric(target.identifier(), metric).await
    }
}
```

## 3. セマンティック統合アーキテクチャ

### 3.1 コンポーネント階層

```
┌─────────────────────────────────────────────────────┐
│           Semantic Analysis Layer (serena)          │
│  ├─ Symbol Index: 高速シンボル検索                 │
│  ├─ Pattern Recognition: 重複パターン検出          │
│  ├─ Refactoring Engine: 自動リファクタリング       │
│  └─ Knowledge Memory: 改善履歴の永続化             │
├─────────────────────────────────────────────────────┤
│            Code Generation Layer                    │
│  ├─ Macro System: コード生成マクロ                 │
│  ├─ Template Engine: パターンテンプレート          │
│  └─ DSL Compiler: ドメイン特化言語                 │
├─────────────────────────────────────────────────────┤
│           Runtime Optimization Layer                │
│  ├─ Token Compression: 93% 削減                    │
│  ├─ Context Caching: シンボルキャッシュ            │
│  └─ Lazy Loading: 必要時のみロード                 │
└─────────────────────────────────────────────────────┘
```

### 3.2 実装フェーズ

#### Phase 1: 基盤構築 (Week 1-2)
- [ ] セマンティックインデックスの構築
- [ ] シンボル操作APIの実装
- [ ] プロジェクトメモリの初期化

#### Phase 2: パターン抽出 (Week 3-4)
- [ ] 共通パターンの識別と抽出
- [ ] マクロシステムの実装
- [ ] テンプレートエンジンの構築

#### Phase 3: 自動リファクタリング (Week 5-6)
- [ ] エラーハンドリングの統一
- [ ] リソースモニタリングの抽象化
- [ ] Sangha投票ロジックの最適化

#### Phase 4: 知識統合 (Week 7-8)
- [ ] セマンティック知識共有の実装
- [ ] クロスモジュール最適化
- [ ] パフォーマンス検証

## 4. 技術的実装詳細

### 4.1 セマンティック操作API

```rust
/// シンボルレベルでの操作を提供
pub trait SemanticOperations {
    /// シンボルの検索
    async fn find_symbol(&self, pattern: &str) -> Result<Vec<Symbol>>;
    
    /// シンボル本体の置換
    async fn replace_symbol_body(&self, symbol: &Symbol, new_body: &str) -> Result<()>;
    
    /// 参照の更新
    async fn update_references(&self, old: &Symbol, new: &Symbol) -> Result<usize>;
    
    /// パターンマッチング
    async fn find_similar_patterns(&self, threshold: f64) -> Result<Vec<PatternGroup>>;
}
```

### 4.2 知識永続化システム

```yaml
# .serena/memories/refactoring_patterns.yaml
patterns:
  error_handling:
    template: "error_macro"
    instances: 60
    reduction: "97%"
    applied: "2025-08-15"
    
  resource_monitoring:
    template: "monitor_trait"
    instances: 12
    reduction: "85%"
    applied: "2025-08-15"
    
  state_management:
    template: "state_machine"
    instances: 8
    reduction: "80%"
    planned: "2025-08-20"
```

### 4.3 自動リファクタリングエンジン

```rust
pub struct RefactoringEngine {
    analyzer: SemanticAnalyzer,
    pattern_db: PatternDatabase,
    memory: ProjectMemory,
}

impl RefactoringEngine {
    /// 重複コードの自動検出と修正
    pub async fn auto_refactor(&self, threshold: f64) -> Result<RefactoringReport> {
        // 1. パターン検出
        let patterns = self.analyzer.find_duplicates(threshold).await?;
        
        // 2. 最適な抽象化の選択
        let abstractions = self.select_abstractions(&patterns).await?;
        
        // 3. コード生成
        let generated = self.generate_unified_code(&abstractions).await?;
        
        // 4. 既存コードの置換
        let replacements = self.replace_duplicates(&generated).await?;
        
        // 5. 知識の記録
        self.memory.record_refactoring(&replacements).await?;
        
        Ok(RefactoringReport {
            patterns_found: patterns.len(),
            code_reduced: self.calculate_reduction(&replacements),
            tokens_saved: self.estimate_token_savings(&replacements),
        })
    }
}
```

## 5. 期待される成果

### 5.1 定量的指標

| メトリクス | 現状 | 目標 | 改善率 |
|-----------|------|------|--------|
| コード重複率 | 85%+ | <20% | 76% 削減 |
| トークン使用量 | 100% | 10% | 90% 削減 |
| テストカバレッジ | 70% | 90%+ | 28% 向上 |
| ビルド時間 | 120s | 60s | 50% 短縮 |
| メモリ使用量 | 1GB | 300MB | 70% 削減 |

### 5.2 定性的改善

- **保守性**: 単一責任原則の徹底による保守性向上
- **拡張性**: モジュール化による新機能追加の容易化
- **信頼性**: エラーハンドリングの一貫性による信頼性向上
- **開発速度**: パターン再利用による開発速度の向上

## 6. リスクと緩和策

### 6.1 技術的リスク

| リスク | 影響度 | 緩和策 |
|--------|--------|--------|
| 互換性の破壊 | High | 段階的移行とテストカバレッジ |
| パフォーマンス低下 | Medium | ベンチマークによる継続的監視 |
| 複雑性の増加 | Low | ドキュメントと教育 |

### 6.2 移行戦略

1. **Canary デプロイメント**: 一部のモジュールから段階的に適用
2. **Feature フラグ**: 新旧システムの切り替え可能
3. **ロールバック計画**: 問題発生時の即座の復元

## 7. 実装ロードマップ

### 7.1 即座に実行可能なタスク

```bash
# 1. エラーハンドリングマクロの実装
vibe-ticket start refactor-duplicate-error-handlers

# 2. リソースモニタリングの抽象化
vibe-ticket start refactor-resource-monitoring

# 3. セマンティック仕様の策定
vibe-ticket start semantic-integration-spec

# 4. 共通パターンの抽出
vibe-ticket start extract-common-patterns
```

### 7.2 長期的な改善計画

- **Q3 2025**: セマンティック基盤の完成
- **Q4 2025**: 全モジュールのリファクタリング完了
- **Q1 2026**: AI駆動の自動最適化システム稼働

## 8. 成功基準

### 8.1 必須要件
- [ ] 全テストがパスすること
- [ ] パフォーマンスが低下しないこと
- [ ] APIの後方互換性を維持すること

### 8.2 目標要件
- [ ] コード量を30%以上削減
- [ ] トークン使用量を90%以上削減
- [ ] 新規開発速度を2倍に向上

## 9. まとめ

本仕様書は、similarity-rsによる分析結果とserenaのセマンティックアプローチを組み合わせた、ccswarmの包括的なリファクタリング計画を定義しています。この計画の実施により、コードベースの品質、効率性、保守性が大幅に向上することが期待されます。

---

## 付録A: similarity-rs分析結果サマリー

```
総ファイル数: 230
重複検出数: 200+
最高類似度: 97.67%
平均類似度: 87.3%
影響モジュール: 15/20 (75%)
```

## 付録B: vibe-ticket タスク一覧

現在作成されたリファクタリングチケット：
- `refactor-duplicate-error-handlers` (P0)
- `refactor-resource-monitoring` (P1)
- `semantic-integration-spec` (P0)
- `extract-common-patterns` (P2)

## 付録C: 参考資料

- serena アーキテクチャドキュメント
- similarity-rs 分析レポート
- ccswarm 現行アーキテクチャ仕様
- Claude Code サブエージェント統合ガイド