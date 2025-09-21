# ccswarm サブエージェント統合仕様書

## 1. 概要

本仕様書は、ccswarmプロジェクトをClaude Codeのネイティブサブエージェント機能を活用するように改修するための技術仕様を定義します。

### 1.1 背景

現在のccswarmは独自のマルチエージェントシステムを実装していますが、Claude Codeの公式サブエージェント機能を使用することで、以下の利点が得られます：

- **コンテキスト管理の改善**: 各サブエージェントが独立したコンテキストウィンドウを持つ
- **システムの簡素化**: Claude Codeの組み込み機能を活用することで、実装の複雑さを軽減
- **パフォーマンス向上**: ネイティブサポートによる効率的なタスク委譲
- **保守性の向上**: 公式APIに準拠することで、将来的なアップデートへの対応が容易

### 1.2 移行方針

- **段階的移行**: 既存機能を維持しながら、段階的にサブエージェント機能へ移行
- **後方互換性**: 既存のccswarm.json設定ファイルとの互換性を維持
- **ハイブリッドモード**: 従来のエージェントとサブエージェントの併用をサポート

## 2. アーキテクチャ設計

### 2.1 新しいアーキテクチャ概要

```
┌─────────────────────────────────────────┐
│      Master Claude (Orchestrator)       │
│  ├─ サブエージェント自動生成・管理     │
│  ├─ タスク分析・委譲エンジン           │
│  └─ プロジェクト全体の進捗管理         │
├─────────────────────────────────────────┤
│        Claude Code サブエージェント     │
│  ├─ Frontend Agent (.claude/agents/)   │
│  ├─ Backend Agent                      │
│  ├─ DevOps Agent                       │
│  ├─ QA Agent                           │
│  ├─ Security Agent                     │
│  └─ Search Agent (Gemini CLI統合)      │
├─────────────────────────────────────────┤
│         ai-session Manager              │
│  ├─ サブエージェント実行環境管理       │
│  ├─ セッション永続化・圧縮             │
│  └─ MCPプロトコルサポート              │
├─────────────────────────────────────────┤
│      Sangha Collective Intelligence     │
│  ├─ サブエージェント間の協調           │
│  └─ 民主的意思決定                     │
└─────────────────────────────────────────┘
```

### 2.2 主要コンポーネントの変更

#### 2.2.1 Master Claude の役割変更

従来の直接的なエージェント制御から、サブエージェントの管理・調整役へ：

- **サブエージェント定義の自動生成**: プロジェクト要件に基づいて最適なサブエージェント構成を生成
- **動的サブエージェント作成**: タスクに応じて新しいサブエージェントを動的に作成
- **タスク分析と委譲**: 自然言語でのタスク記述を解析し、適切なサブエージェントへ委譲
- **進捗モニタリング**: 各サブエージェントの作業進捗を統合的に管理

#### 2.2.2 エージェントシステムの移行

現在のエージェント実装をClaude Codeサブエージェントへ移行：

```rust
// 従来の実装
pub enum AgentRole {
    Frontend,
    Backend,
    DevOps,
    QA,
    Search,
    Master,
}

// 新しい実装
pub struct SubAgent {
    name: String,
    description: String,
    tools: Vec<String>,
    system_prompt: String,
    file_path: PathBuf,  // .claude/agents/内のファイルパス
}
```

## 3. サブエージェント定義

### 3.1 ファイル構造

```
project_root/
├── .claude/
│   └── agents/
│       ├── frontend-specialist.md
│       ├── backend-specialist.md
│       ├── devops-engineer.md
│       ├── qa-engineer.md
│       ├── security-auditor.md
│       └── search-researcher.md
├── ccswarm.json  # 後方互換性のため維持
└── src/
```

### 3.2 サブエージェント定義例

#### frontend-specialist.md

```markdown
---
name: frontend-specialist
description: |
  Specialized in modern frontend development with React, TypeScript, and UI/UX best practices.
  MUST BE USED PROACTIVELY for all frontend-related tasks including:
  - Component development and architecture
  - State management implementation
  - Performance optimization
  - Accessibility compliance
  - Responsive design
tools: write_file,read_file,execute_command,browser_action,list_directory
---

# Frontend Specialist

You are a highly skilled frontend developer specializing in modern web development.

## Core Competencies

- **React & TypeScript**: Expert-level knowledge of React 18+, hooks, and TypeScript
- **State Management**: Redux Toolkit, Zustand, Context API
- **Styling**: CSS-in-JS, Tailwind CSS, CSS Modules
- **Testing**: Jest, React Testing Library, Cypress
- **Performance**: Code splitting, lazy loading, bundle optimization
- **Accessibility**: WCAG 2.1 compliance, ARIA best practices

## Development Guidelines

1. **Component Architecture**
   - Use functional components with hooks
   - Implement proper prop typing with TypeScript
   - Follow atomic design principles
   - Ensure components are reusable and testable

2. **Code Quality**
   - Write comprehensive tests (minimum 80% coverage)
   - Use ESLint and Prettier configurations
   - Implement proper error boundaries
   - Add meaningful comments and documentation

3. **Performance Optimization**
   - Implement React.memo for expensive components
   - Use useMemo and useCallback appropriately
   - Optimize bundle size with dynamic imports
   - Monitor and improve Core Web Vitals

## Task Execution

When assigned a frontend task:
1. Analyze requirements and plan component structure
2. Implement with best practices and patterns
3. Write tests alongside implementation
4. Ensure accessibility standards are met
5. Optimize for performance
6. Document component usage and props
```

### 3.3 サブエージェント管理コマンド

```bash
# サブエージェント一覧表示
ccswarm agents list

# 新規サブエージェント作成（インタラクティブ）
ccswarm agents create

# サブエージェント編集
ccswarm agents edit frontend-specialist

# サブエージェント削除
ccswarm agents delete frontend-specialist

# サブエージェントの自動生成
ccswarm agents generate --role "データ分析専門家" --tools "read_file,execute_command"

# サブエージェント設定の検証
ccswarm agents validate
```

## 4. 統合実装詳細

### 4.1 ccswarm初期化の変更

```bash
# 新しい初期化コマンド
ccswarm init --name "MyProject" --with-subagents

# これにより以下が自動生成される：
# 1. .claude/agents/ ディレクトリ
# 2. プロジェクトに適したサブエージェント定義
# 3. ccswarm.json（後方互換性のため）
```

### 4.2 タスク委譲の新しいフロー

```rust
// Master Claudeによるタスク分析と委譲
impl MasterClaude {
    async fn delegate_task(&self, task: &Task) -> Result<DelegationResult> {
        // 1. タスクの内容を分析
        let analysis = self.analyze_task(task).await?;
        
        // 2. 適切なサブエージェントを選択または作成
        let subagent = if let Some(existing) = self.find_suitable_subagent(&analysis) {
            existing
        } else {
            self.create_dynamic_subagent(&analysis).await?
        };
        
        // 3. Claude Codeのサブエージェント機能を使用して委譲
        let delegation_prompt = format!(
            "Please delegate this task to the {} sub-agent: {}",
            subagent.name,
            task.description
        );
        
        // 4. 結果を追跡
        self.track_delegation(task.id, &subagent).await?;
        
        Ok(DelegationResult {
            subagent: subagent.name,
            status: DelegationStatus::InProgress,
        })
    }
}
```

### 4.3 自動サブエージェント生成

Master Claudeが必要に応じて新しいサブエージェントを動的に生成：

```rust
impl MasterClaude {
    async fn create_dynamic_subagent(&self, analysis: &TaskAnalysis) -> Result<SubAgent> {
        // タスクに最適なサブエージェント定義を生成
        let agent_spec = self.generate_agent_specification(analysis).await?;
        
        // .claude/agents/ にファイルを作成
        let file_path = format!(".claude/agents/{}.md", agent_spec.name);
        fs::write(&file_path, agent_spec.to_markdown())?;
        
        // ccswarmの内部レジストリに登録
        self.register_subagent(agent_spec)?;
        
        Ok(agent_spec)
    }
}
```

## 5. Sangha統合

### 5.1 サブエージェント間の協調

```markdown
# sangha-coordinator.md
---
name: sangha-coordinator
description: |
  Facilitates democratic decision-making among sub-agents.
  Manages proposals, voting, and consensus building.
  MUST BE USED for all collective decisions and major architectural changes.
tools: read_file,write_file,execute_command
---

# Sangha Coordinator

You coordinate democratic decision-making among all sub-agents in the project.

## Responsibilities

1. **Proposal Management**
   - Collect improvement proposals from sub-agents
   - Structure proposals for voting
   - Track voting progress

2. **Consensus Building**
   - Facilitate discussions between sub-agents
   - Identify common ground and conflicts
   - Drive towards consensus

3. **Decision Implementation**
   - Ensure approved proposals are implemented
   - Track implementation progress
   - Validate outcomes
```

### 5.2 投票メカニズムの実装

```bash
# サブエージェントが提案を作成
ccswarm sangha propose --from frontend-specialist --title "Migrate to React 19"

# 他のサブエージェントが投票
ccswarm sangha vote --as backend-specialist --proposal-id xyz --vote aye

# 結果の確認
ccswarm sangha results --proposal-id xyz
```

## 6. 移行計画

### 6.1 フェーズ1: 基盤整備（2週間）

- [ ] Claude Codeサブエージェント形式のパーサー実装
- [ ] サブエージェント管理コマンドの実装
- [ ] 既存エージェントからサブエージェント定義への変換ツール

### 6.2 フェーズ2: 統合実装（3週間）

- [ ] Master Claudeのサブエージェント委譲機能
- [ ] ai-sessionとの統合
- [ ] 動的サブエージェント生成機能

### 6.3 フェーズ3: 移行とテスト（2週間）

- [ ] 既存プロジェクトの移行ツール
- [ ] 包括的なテストスイート
- [ ] パフォーマンス最適化

### 6.4 フェーズ4: ドキュメント更新（1週間）

- [ ] ユーザーガイドの更新
- [ ] APIドキュメントの更新
- [ ] 移行ガイドの作成

## 7. 互換性とアップグレードパス

### 7.1 既存プロジェクトの移行

```bash
# 自動移行コマンド
ccswarm migrate --to-subagents

# これにより：
# 1. 既存のccswarm.jsonを解析
# 2. 各エージェント設定をサブエージェント定義に変換
# 3. .claude/agents/ ディレクトリに配置
# 4. 移行レポートを生成
```

### 7.2 ハイブリッドモード

移行期間中は両方のシステムを併用可能：

```json
{
  "project": {
    "name": "MyProject",
    "mode": "hybrid",  // "legacy", "subagents", "hybrid"
    "subagents": {
      "enabled": true,
      "fallback_to_legacy": true
    }
  }
}
```

## 8. 期待される効果

### 8.1 パフォーマンス改善

- **コンテキスト効率**: 各サブエージェントが独立したコンテキストを持つため、長時間のセッションが可能
- **並列処理**: 複数のサブエージェントが同時に異なるタスクを処理
- **メモリ使用量削減**: 必要な時のみサブエージェントをロード

### 8.2 開発体験の向上

- **明確な責任分離**: 各サブエージェントの役割が明確に定義
- **カスタマイズ性**: プロジェクトごとに最適なサブエージェント構成
- **再利用性**: サブエージェント定義を他のプロジェクトで再利用可能

### 8.3 保守性の向上

- **公式API準拠**: Claude Codeの将来的なアップデートに対応しやすい
- **シンプルな実装**: 独自のエージェント管理システムが不要
- **デバッグの容易さ**: 各サブエージェントの動作を個別に検証可能

## 9. リスクと対策

### 9.1 移行リスク

- **リスク**: 既存プロジェクトの動作に影響
- **対策**: ハイブリッドモードによる段階的移行

### 9.2 学習曲線

- **リスク**: 新しいシステムへの適応に時間がかかる
- **対策**: 包括的なドキュメントと移行ツールの提供

### 9.3 機能制限

- **リスク**: Claude Codeのサブエージェント機能の制限
- **対策**: 必要に応じて独自拡張を維持

## 10. 次のステップ

1. **技術検証**: プロトタイプの作成と検証
2. **コミュニティフィードバック**: 提案に対する意見収集
3. **実装開始**: フェーズ1の開発開始
4. **段階的リリース**: ベータ版から正式版への移行