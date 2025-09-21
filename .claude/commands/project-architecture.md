# プロジェクトアーキテクチャ

## モジュール構成

```
src/
├── agent/              # エージェントのコア機能
│   ├── claude.rs       # Claude Codeエージェントの実装
│   ├── personality.rs  # エージェントのパーソナリティシステム
│   ├── whiteboard.rs   # 思考の見える化
│   ├── phronesis.rs    # 実践的知恵（経験学習）
│   ├── isolation.rs    # エージェントの分離機能
│   └── ...
├── coordination/       # エージェント間の連携
│   ├── dialogue.rs     # 対話システム
│   └── mod.rs
├── identity/           # エージェントの役割と境界
├── orchestrator/       # マスターエージェント
├── providers/          # AIプロバイダ（Claude, OpenAI等）
├── session/            # セッション管理
├── tui/                # ターミナルUI
├── container/          # Dockerコンテナ分離（オプション）
└── cli/                # CLIインターフェース
```

## コアコンセプト

### 1. エージェントパーソナリティシステム
```rust
// src/agent/personality.rs
pub struct AgentPersonality {
    pub skills: HashMap<String, Skill>,           // スキルと経験値
    pub traits: PersonalityTraits,                // 性格特性
    pub working_style: WorkingStyle,              // 作業スタイル
    pub experience_points: u32,                   // 総経験値
    pub adaptation_history: Vec<AdaptationRecord>, // 適応履歴
}

// スキルレベルシステム
pub enum SkillLevel {
    Novice,       // 0-100 XP
    Beginner,     // 101-300 XP
    Intermediate, // 301-700 XP
    Advanced,     // 701-1500 XP
    Expert,       // 1501-3000 XP
    Master,       // 3000+ XP
}
```

### 2. ホワイトボード（思考の見える化）
```rust
// src/agent/whiteboard.rs
pub enum EntryType {
    Calculation { expression: String, result: Option<String> },
    Diagram { description: String, elements: Vec<DiagramElement> },
    Note { content: String, tags: Vec<String> },
    Hypothesis { statement: String, confidence: f32, evidence: Vec<String> },
    ThoughtTrace { thoughts: Vec<String>, conclusion: Option<String> },
}

// エージェントが計算、仮説、思考の流れを記録
```

### 3. フロネシス（実践的知恵）
```rust
// src/agent/phronesis.rs
pub struct PhronesisManager {
    pub wisdom_base: HashMap<WisdomCategory, Vec<WisdomItem>>,
    pub learning_history: Vec<LearningEvent>,
    pub success_patterns: HashMap<String, SuccessPattern>,
    pub failure_analyses: Vec<FailureAnalysis>,
}

// 4種類のメモリーシステム
pub enum MemoryType {
    Working,    // 現在のコンテキスト（10項目）
    Episodic,   // 最近の経験（24時間以内）
    Semantic,   // 概念と知識
    Procedural, // 手順とパターン
}
```

### 4. 対話ダンスシステム
```rust
// src/coordination/dialogue.rs
pub struct DialogueCoordinationBus {
    pub conversations: HashMap<String, Conversation>,
    pub dialogue_patterns: Vec<DialoguePattern>,
    pub agent_profiles: HashMap<String, AgentDialogueProfile>,
}

// コンテキストを理解したマルチエージェント対話
```

## エージェントの分離機能

### Git Worktree分離
```bash
# 各エージェントが独立したworktreeで動作
agents/
├── frontend-agent-abc123/
├── backend-agent-def456/
├── devops-agent-ghi789/
└── qa-agent-jkl012/
```

### コンテナ分離（オプション）
```rust
// Cargo.toml
[features]
default = []
container = ["bollard", "tempfile"]

// 条件付きコンパイル
#[cfg(feature = "container")]
pub mod container;
```

## 品質レビューシステム

### LLM品質判定
```rust
// コード品質を8つの角度で評価
pub struct QualityMetrics {
    pub correctness: f32,       // 正しさ
    pub maintainability: f32,   // 保守性
    pub performance: f32,       // パフォーマンス
    pub security: f32,          // セキュリティ
    pub readability: f32,       // 可読性
    pub test_coverage: f32,     // テストカバレッジ
    pub documentation: f32,     // ドキュメント
    pub complexity: f32,        // 複雑さ
}

// デフォルト基準: 85%テストカバレッジ、複雑度 < 10
```

### 自動改善タスク生成
```rust
// 品質レビュー失敗時に自動生成される改善タスク
pub enum RemediationTaskType {
    AddTests,           // テスト追加
    ImproveDocumentation, // ドキュメント改善
    RefactorComplexity,   // 複雑度減少
    AddErrorHandling,     // エラーハンドリング強化
    SecurityAudit,        // セキュリティ監査
}
```

## セッション管理

### 93%のAPIトークン削減
```rust
pub struct SessionManager {
    pub active_sessions: HashMap<String, AgentSession>,
    pub conversation_history: VecDeque<Message>, // 50メッセージ保持
    pub context_compression: ContextCompressor,   // コンテキスト圧縮
}

// バッチタスク実行で効率化
```

## プロバイダー抽象化

### 対応プロバイダー
```rust
pub trait Provider {
    async fn execute_task(&self, task: &Task, context: &AgentContext) -> Result<TaskResult>;
    async fn health_check(&self) -> Result<ProviderStatus>;
}

// 実装中
pub struct ClaudeCodeProvider;  // Claude Code CLI
pub struct AiderProvider;       // Aider
pub struct OpenAICodexProvider; // OpenAI Codex
pub struct ClaudeApiProvider;   // Claude API直接
```

## ターミナルUI

### リアルタイムモニタリング
```rust
// src/tui/mod.rs
// ratatuiを使用したターミナルUI
pub struct TuiApp {
    pub agent_status_panel: AgentStatusPanel,
    pub task_queue_panel: TaskQueuePanel,
    pub log_panel: LogPanel,
    pub metrics_panel: MetricsPanel,
}

// tmuxセッション管理
// エージェントのリアルタイム状態表示
```

## パフォーマンス考慮事項

### メモリ使用量
- セッション再利用でAPIコストを~93%削減
- Git worktreeはエージェントあたり~100MB
- JSON連携は<100msのレイテンシ

### スケーラビリティ
- エージェントプールによる負荷分散
- 非同期タスク実行
- 自動スケーリングに対応

## セキュリティ

### アクセス制御
```rust
// エージェントの役割境界強制
pub struct TaskBoundaryChecker {
    pub allowed_roles: Vec<AgentRole>,
    pub forbidden_patterns: Vec<String>,
    pub risk_assessment: RiskAssessment,
}

// ファイル保護パターン
protected_files: [".env", "*.key", "config/secrets/*"]
```

### リスク評価
```rust
// 1-10スケールのリスク評価
pub struct RiskAssessment {
    pub risk_level: u8,        // 1-10
    pub auto_approve: bool,    // 自動承認可能か
    pub requires_human: bool,  // 人間の承認が必要か
}
```

## デプロイメントアーキテクチャ

### シングルバイナリ
```bash
# シンプルなインストール
cargo install ccswarm

# 機能フラグ付き
cargo install ccswarm --features container
```

### Dockerコンテナ対応
```dockerfile
# コンテナベースのエージェント実行環境
# セキュリティと分離の強化
```

### クラウドデプロイ
```yaml
# Kubernetes対応
# AWS/GCP/Azureサポート
# スケーリング対応
```