# ccswarm サブエージェント統合仕様書 v2.0
## serenaアーキテクチャ統合版

## 1. 概要

本仕様書は、ccswarmプロジェクトをClaude Codeのネイティブサブエージェント機能とserenaのセマンティックコード分析機能を統合して改修するための技術仕様を定義します。

### 1.1 背景と動機

現在のccswarmは独自のマルチエージェントシステムを実装していますが、以下の統合により飛躍的な性能向上が期待できます：

#### Claude Codeサブエージェント機能の利点
- **コンテキスト管理の改善**: 各サブエージェントが独立したコンテキストウィンドウを持つ
- **システムの簡素化**: Claude Codeの組み込み機能を活用することで、実装の複雑さを軽減
- **パフォーマンス向上**: ネイティブサポートによる効率的なタスク委譲

#### serenaセマンティック分析機能の利点
- **インテリジェントなコード理解**: シンボルレベルでのコード分析と操作
- **効率的なコード探索**: 必要な部分のみを読み込むトークン効率的なアプローチ
- **プロジェクトメモリ**: 長期的な知識の保持と活用
- **MCPプロトコル統合**: 標準化されたツール統合

### 1.2 統合方針

- **段階的移行**: 既存機能を維持しながら、段階的に新機能へ移行
- **セマンティック強化**: serenaの分析機能を各サブエージェントに統合
- **知識共有**: サブエージェント間でのメモリとコンテキストの共有
- **ハイブリッドモード**: 従来のエージェントと新システムの併用をサポート

## 2. 統合アーキテクチャ設計

### 2.1 新しいアーキテクチャ概要

```
┌─────────────────────────────────────────────────────────┐
│              Master Claude (Orchestrator)                │
│  ├─ サブエージェント自動生成・管理                      │
│  ├─ タスク分析・委譲エンジン                            │
│  ├─ セマンティックプロジェクト理解（serena統合）        │
│  └─ プロジェクト全体の進捗管理                          │
├─────────────────────────────────────────────────────────┤
│         セマンティック強化サブエージェント              │
│  ├─ Frontend Agent + serena分析                        │
│  ├─ Backend Agent + serena分析                         │
│  ├─ DevOps Agent + インフラコード分析                  │
│  ├─ QA Agent + テストカバレッジ分析                    │
│  ├─ Security Agent + 脆弱性パターン分析                │
│  └─ Search Agent + コードベース横断検索                │
├─────────────────────────────────────────────────────────┤
│              serena統合レイヤー                         │
│  ├─ セマンティックコード分析エンジン                   │
│  ├─ シンボル操作・リファクタリング機能                 │
│  ├─ プロジェクトメモリ管理                             │
│  └─ MCPプロトコルサーバー                              │
├─────────────────────────────────────────────────────────┤
│             ai-session Manager                          │
│  ├─ サブエージェント実行環境管理                       │
│  ├─ セッション永続化・圧縮（93%削減）                  │
│  └─ コンテキスト最適化                                 │
├─────────────────────────────────────────────────────────┤
│        Sangha Collective Intelligence                   │
│  ├─ サブエージェント間の協調                           │
│  ├─ 知識の民主的共有                                   │
│  └─ コードベース全体の改善提案                         │
└─────────────────────────────────────────────────────────┘
```

### 2.2 主要コンポーネントの統合

#### 2.2.1 Master Claudeの進化

従来の役割に加えて、serenaのセマンティック理解を統合：

```rust
pub struct MasterClaude {
    // 既存のフィールド
    orchestrator: Arc<Orchestrator>,
    
    // serena統合
    semantic_analyzer: Arc<SemanticAnalyzer>,
    project_memory: Arc<ProjectMemory>,
    symbol_index: Arc<SymbolIndex>,
}

impl MasterClaude {
    /// セマンティック分析を活用したタスク委譲
    async fn delegate_task_with_context(&self, task: &Task) -> Result<DelegationResult> {
        // 1. タスクに関連するコードシンボルを特定
        let relevant_symbols = self.semantic_analyzer
            .find_relevant_symbols(&task.description)
            .await?;
        
        // 2. プロジェクトメモリから関連知識を取得
        let context = self.project_memory
            .retrieve_context(&relevant_symbols)
            .await?;
        
        // 3. 最適なサブエージェントを選択
        let subagent = self.select_optimal_subagent(&task, &context).await?;
        
        // 4. セマンティックコンテキストと共に委譲
        let enriched_task = EnrichedTask {
            original: task.clone(),
            symbols: relevant_symbols,
            context,
            suggested_approach: self.generate_approach(&task, &context).await?,
        };
        
        self.delegate_to_subagent(&subagent, &enriched_task).await
    }
}
```

#### 2.2.2 セマンティック強化サブエージェント

各サブエージェントにserenaの能力を統合：

```rust
pub struct SemanticSubAgent {
    base_agent: SubAgent,
    semantic_tools: SemanticTools,
    memory_access: MemoryAccess,
}

pub struct SemanticTools {
    /// シンボルレベルでのコード操作
    symbol_manipulator: SymbolManipulator,
    
    /// インテリジェントな検索
    code_searcher: CodeSearcher,
    
    /// リファクタリング提案
    refactoring_advisor: RefactoringAdvisor,
    
    /// 依存関係分析
    dependency_analyzer: DependencyAnalyzer,
}
```

## 3. セマンティック強化サブエージェント定義

### 3.1 ファイル構造

```
project_root/
├── .claude/
│   ├── agents/                          # サブエージェント定義
│   │   ├── frontend-specialist.md
│   │   ├── backend-specialist.md
│   │   └── ...
│   └── memories/                        # プロジェクトメモリ（serena形式）
│       ├── architecture_overview.md
│       ├── coding_conventions.md
│       └── domain_knowledge.md
├── .serena/                            # serena設定とキャッシュ
│   ├── config.yaml
│   └── cache/
├── ccswarm.json                        # 後方互換性設定
└── src/
```

### 3.2 セマンティック強化サブエージェント定義例

#### frontend-specialist.md (セマンティック強化版)

```markdown
---
name: frontend-specialist
description: |
  Frontend development specialist with semantic code understanding.
  MUST BE USED PROACTIVELY for all frontend-related tasks.
tools: 
  - standard: write_file,read_file,execute_command,browser_action
  - semantic: find_symbol,replace_symbol_body,find_referencing_symbols,search_for_pattern
  - memory: read_memory,write_memory,list_memories
capabilities:
  - React component architecture with symbol-level understanding
  - TypeScript type system analysis
  - Performance optimization through code pattern analysis
  - Accessibility compliance verification
---

# Frontend Specialist with Semantic Intelligence

You are a frontend development expert enhanced with semantic code analysis capabilities.

## Semantic Analysis Guidelines

### 1. Code Exploration Strategy
NEVER read entire files. Instead:
1. Use `get_symbols_overview` to understand file structure
2. Use `find_symbol` to locate specific components/functions
3. Use `find_referencing_symbols` to understand usage patterns
4. Only read symbol bodies when necessary for implementation

### 2. Component Development Workflow
1. **Analyze existing patterns**:
   ```
   - Search for similar components using search_for_pattern
   - Analyze their structure with get_symbols_overview
   - Identify reusable patterns and conventions
   ```

2. **Implement with context**:
   ```
   - Use replace_symbol_body for precise modifications
   - Maintain consistency with existing code patterns
   - Update all references using find_referencing_symbols
   ```

3. **Knowledge preservation**:
   ```
   - Document new patterns in project memory
   - Update architecture decisions
   - Share insights with other agents
   ```

## Task Execution with Semantic Context

When assigned a frontend task:

1. **Semantic Analysis Phase**
   - Identify affected components using symbol search
   - Analyze component dependencies
   - Check for similar implementations in codebase

2. **Implementation Phase**
   - Use symbol-level operations for precise changes
   - Maintain type safety with TypeScript analysis
   - Ensure consistent patterns across codebase

3. **Validation Phase**
   - Verify all symbol references are updated
   - Check for breaking changes in component APIs
   - Run type checking and tests

4. **Knowledge Capture**
   - Document architectural decisions
   - Update component usage patterns
   - Share learnings via project memory
```

### 3.3 統合されたセマンティックツール

各サブエージェントが利用可能な統合ツール：

```yaml
semantic_tools:
  code_analysis:
    - find_symbol: "シンボル名からコード要素を検索"
    - get_symbols_overview: "ファイル/ディレクトリのシンボル概要"
    - find_referencing_symbols: "シンボルの参照箇所を検索"
    - search_for_pattern: "正規表現パターンでコード検索"
    
  code_modification:
    - replace_symbol_body: "シンボル本体の置換"
    - insert_before_symbol: "シンボル前に挿入"
    - insert_after_symbol: "シンボル後に挿入"
    - replace_regex: "正規表現ベースの置換"
    
  memory_management:
    - read_memory: "プロジェクト知識の読み取り"
    - write_memory: "新しい知識の記録"
    - list_memories: "利用可能な知識一覧"
    - delete_memory: "古い知識の削除"
    
  project_understanding:
    - get_project_structure: "プロジェクト構造の理解"
    - analyze_dependencies: "依存関係の分析"
    - find_similar_code: "類似コードパターンの検索"
```

## 4. serena統合の実装詳細

### 4.1 セマンティック初期化

```bash
# 新しい初期化コマンド（serena統合）
ccswarm init --name "MyProject" --with-subagents --enable-semantic

# これにより以下が自動実行される：
# 1. serena設定の初期化
# 2. コードベースの初期分析とインデックス作成
# 3. プロジェクトメモリの初期化
# 4. セマンティック強化サブエージェントの生成
```

### 4.2 セマンティックタスク分析

```rust
impl SemanticTaskAnalyzer {
    /// タスクの記述から関連するコード要素を特定
    async fn analyze_task(&self, task: &Task) -> Result<TaskContext> {
        // 1. タスク記述からキーワード抽出
        let keywords = self.extract_keywords(&task.description);
        
        // 2. 関連シンボルの検索
        let symbols = self.find_related_symbols(&keywords).await?;
        
        // 3. 影響範囲の分析
        let impact_analysis = self.analyze_impact(&symbols).await?;
        
        // 4. 推奨アプローチの生成
        let approach = self.generate_approach(&task, &symbols, &impact_analysis).await?;
        
        Ok(TaskContext {
            task: task.clone(),
            related_symbols: symbols,
            impact: impact_analysis,
            recommended_approach: approach,
        })
    }
}
```

### 4.3 サブエージェント間の知識共有

```rust
/// サブエージェント間でセマンティック知識を共有
pub struct SemanticKnowledgeSharing {
    shared_memory: Arc<ProjectMemory>,
    symbol_registry: Arc<SymbolRegistry>,
    pattern_library: Arc<PatternLibrary>,
}

impl SemanticKnowledgeSharing {
    /// フロントエンドの変更をバックエンドに通知
    async fn propagate_api_changes(
        &self,
        frontend_changes: &[SymbolChange],
    ) -> Result<Vec<BackendTask>> {
        let mut backend_tasks = Vec::new();
        
        for change in frontend_changes {
            if change.affects_api() {
                // APIの変更を検出
                let api_impact = self.analyze_api_impact(change).await?;
                
                // バックエンドタスクを生成
                let task = self.generate_backend_task(&api_impact).await?;
                backend_tasks.push(task);
                
                // 共有メモリに記録
                self.shared_memory.record_api_change(change, &api_impact).await?;
            }
        }
        
        Ok(backend_tasks)
    }
}
```

## 5. Sanghaシステムのセマンティック拡張

### 5.1 コード品質の民主的評価

```rust
/// セマンティック分析に基づく提案システム
pub struct SemanticSangha {
    proposal_analyzer: ProposalAnalyzer,
    impact_evaluator: ImpactEvaluator,
    consensus_builder: ConsensusBuilder,
}

impl SemanticSangha {
    /// コード変更提案の自動生成と評価
    async fn propose_improvement(&self, issue: &CodeIssue) -> Result<Proposal> {
        // 1. 問題のセマンティック分析
        let analysis = self.proposal_analyzer.analyze_issue(issue).await?;
        
        // 2. 改善提案の生成
        let proposals = self.generate_proposals(&analysis).await?;
        
        // 3. 各サブエージェントからの評価収集
        let evaluations = self.collect_agent_evaluations(&proposals).await?;
        
        // 4. コンセンサス形成
        let consensus = self.consensus_builder.build(&evaluations).await?;
        
        Ok(consensus.best_proposal)
    }
}
```

### 5.2 自動リファクタリング提案

```yaml
# sangha-refactoring-coordinator.md
---
name: sangha-refactoring-coordinator
description: |
  Coordinates semantic refactoring proposals across all agents.
  Uses code analysis to identify improvement opportunities.
tools: 
  - semantic: all
  - voting: propose,vote,tally
capabilities:
  - Duplicate code detection across agents
  - Pattern inconsistency identification
  - Performance bottleneck analysis
  - Security vulnerability scanning
---
```

## 6. 移行計画（serena統合版）

### 6.1 フェーズ1: セマンティック基盤（3週間）

- [ ] serena統合レイヤーの実装
- [ ] MCPプロトコルサーバーの統合
- [ ] プロジェクトメモリシステムの実装
- [ ] 既存エージェントのセマンティック拡張

### 6.2 フェーズ2: サブエージェント統合（4週間）

- [ ] Claude Codeサブエージェント形式のパーサー
- [ ] セマンティックツールのサブエージェント統合
- [ ] 知識共有メカニズムの実装
- [ ] 動的サブエージェント生成（セマンティック対応）

### 6.3 フェーズ3: インテリジェント機能（3週間）

- [ ] セマンティックタスク分析エンジン
- [ ] 自動リファクタリング提案システム
- [ ] コードベース横断的な最適化
- [ ] Sanghaセマンティック投票システム

### 6.4 フェーズ4: 最適化とドキュメント（2週間）

- [ ] パフォーマンスチューニング
- [ ] セマンティック機能のドキュメント
- [ ] 移行ガイドとベストプラクティス
- [ ] 実プロジェクトでの検証

## 7. 期待される効果（serena統合版）

### 7.1 開発効率の飛躍的向上

- **トークン効率**: serenaの選択的読み込みにより90%以上のトークン削減
- **精密な変更**: シンボルレベルの操作により副作用を最小化
- **知識の蓄積**: プロジェクト固有の知識が自動的に蓄積・活用

### 7.2 コード品質の自動改善

- **一貫性の維持**: パターン分析による自動的な一貫性チェック
- **リファクタリング提案**: 重複コードや改善可能なパターンの自動検出
- **影響分析**: 変更の影響を事前に正確に予測

### 7.3 チーム協調の革新

- **知識の民主化**: 全てのサブエージェントが同じ知識基盤を共有
- **自動調整**: API変更などが自動的に関連エージェントに伝播
- **継続的学習**: プロジェクトの成長と共に知識も進化

## 8. 技術的詳細

### 8.1 serena設定例

```yaml
# .serena/config.yaml
project:
  name: "ccswarm"
  language: "rust"
  
semantic_analysis:
  enabled: true
  cache_size: "1GB"
  index_on_startup: true
  
memory:
  max_memories: 100
  auto_cleanup: true
  sharing_enabled: true
  
mcp_server:
  enabled: true
  port: 8080
  auth_required: false
```

### 8.2 統合API例

```rust
// サブエージェントからのセマンティック操作
let frontend_agent = SubAgent::get("frontend-specialist");

// コンポーネントの検索と更新
let button_component = frontend_agent
    .find_symbol("Button", SymbolKind::Component)
    .await?;

let updated_body = frontend_agent
    .enhance_accessibility(&button_component)
    .await?;

frontend_agent
    .replace_symbol_body(&button_component, &updated_body)
    .await?;

// 変更の影響を他のエージェントに通知
frontend_agent
    .notify_change(&button_component, ChangeType::ApiModification)
    .await?;
```

## 9. まとめ

このserena統合版の仕様により、ccswarmは単なるマルチエージェントシステムから、セマンティックな理解を持つインテリジェントな開発支援システムへと進化します。各サブエージェントがコードの深い理解を持ち、効率的に協調することで、開発生産性と品質の両方で飛躍的な向上が期待できます。