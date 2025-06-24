//! エージェントの個性・ホワイトボード・実践知システムのデモ

use ccswarm::agent::{
    ClaudeCodeAgent, Priority, Task, TaskType,
    AnnotationMarker, EntryType, SkillCategory, WisdomCategory,
};
use ccswarm::config::ClaudeConfig;
use ccswarm::identity::{default_frontend_role, default_backend_role};
use anyhow::Result;
use std::collections::HashMap;
use tempfile::TempDir;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // ログの初期化
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🎭 エージェントの個性・思考・学習システムデモ\n");

    // 一時ディレクトリを作成
    let temp_dir = TempDir::new()?;
    let workspace = temp_dir.path().to_path_buf();

    // フロントエンドエージェントを作成
    let mut frontend_agent = ClaudeCodeAgent::new(
        default_frontend_role(),
        &workspace,
        "demo",
        ClaudeConfig::default(),
    )
    .await?;

    // バックエンドエージェントを作成
    let mut backend_agent = ClaudeCodeAgent::new(
        default_backend_role(),
        &workspace,
        "demo",
        ClaudeConfig::default(),
    )
    .await?;

    println!("=== 1. エージェントの初期個性 ===\n");
    
    println!("Frontend Agent:");
    println!("  {}", frontend_agent.personality.describe_personality());
    println!("  作業スタイル: {:?}", frontend_agent.personality.working_style);
    println!("  スキル:");
    for (name, skill) in &frontend_agent.personality.skills {
        println!("    - {} ({}): レベル {:?}", 
            name, 
            match &skill.category {
                SkillCategory::Technical => "技術",
                SkillCategory::Creative => "創造",
                SkillCategory::Analytical => "分析",
                SkillCategory::Communication => "コミュニケーション",
                SkillCategory::Leadership => "リーダーシップ",
            },
            skill.level
        );
    }

    println!("\nBackend Agent:");
    println!("  {}", backend_agent.personality.describe_personality());
    println!("  作業スタイル: {:?}", backend_agent.personality.working_style);

    println!("\n=== 2. ホワイトボードでの思考プロセス ===\n");

    // タスクを作成
    let task = Task::new(
        Uuid::new_v4().to_string(),
        "ユーザー認証フォームの実装".to_string(),
        Priority::High,
        TaskType::Feature,
    );

    // フロントエンドエージェントがホワイトボードを使用
    let section_id = frontend_agent.whiteboard.create_section("認証フォーム設計");
    
    // 思考の軌跡を開始
    let thought_trace = frontend_agent.whiteboard.start_thought_trace();
    frontend_agent.whiteboard.add_to_section(&section_id, &thought_trace);

    // 思考を追加
    frontend_agent.whiteboard.add_thought(&thought_trace, "まずはUIのワイヤーフレームを考える");
    frontend_agent.whiteboard.add_thought(&thought_trace, "メールアドレスとパスワードの入力フィールドが必要");
    frontend_agent.whiteboard.add_thought(&thought_trace, "バリデーションはリアルタイムで行いたい");
    
    // 仮説を追加
    let hypothesis = frontend_agent.whiteboard.add_hypothesis(
        "React Hook Formを使えば、バリデーションが簡潔に実装できる",
        0.8,
    );
    frontend_agent.whiteboard.add_to_section(&section_id, &hypothesis);
    
    // 計算を記録
    let calc = frontend_agent.whiteboard.add_calculation("フォームの高さ = ヘッダー(60px) + 入力欄(40px × 2) + ボタン(48px) + 余白(20px × 4)");
    frontend_agent.whiteboard.add_to_section(&section_id, &calc);
    frontend_agent.whiteboard.update_calculation_result(&calc, "268px");
    
    // 結論を設定
    frontend_agent.whiteboard.set_conclusion(
        &thought_trace,
        "React Hook Form + Material-UIで実装する方針に決定",
    );

    // ホワイトボードの内容を表示
    println!("Frontend Agentのホワイトボード:");
    let recent_entries = frontend_agent.whiteboard.recent_entries(5);
    for entry in recent_entries {
        match &entry.entry_type {
            EntryType::ThoughtTrace { thoughts, conclusion } => {
                println!("  思考の軌跡:");
                for thought in thoughts {
                    println!("    - {}", thought);
                }
                if let Some(conc) = conclusion {
                    println!("    結論: {}", conc);
                }
            }
            EntryType::Hypothesis { statement, confidence, .. } => {
                println!("  仮説: {} (信頼度: {:.0}%)", statement, confidence * 100.0);
            }
            EntryType::Calculation { expression, result } => {
                println!("  計算: {} = {}", expression, result.as_ref().unwrap_or(&"未計算".to_string()));
            }
            _ => {}
        }
    }

    // 重要な発見に注釈を追加
    frontend_agent.whiteboard.annotate(
        &hypothesis,
        "このライブラリは学習コストが低い",
        AnnotationMarker::Important,
    );

    println!("\n=== 3. 実践知の蓄積 ===\n");

    // 成功体験を記録
    frontend_agent.phronesis.record_success(
        &task.id,
        "React Hook Form",
        "フォームバリデーションを効率的に実装",
        "宣言的なバリデーションルールにより、コードが簡潔になる",
    );

    // 失敗体験も記録
    backend_agent.phronesis.record_failure(
        "task-456",
        "N+1クエリ",
        "関連データの取得でループ内でクエリを発行",
        "Eager Loadingを使用して、事前に関連データを取得する",
    );

    // 発見を記録
    backend_agent.phronesis.record_discovery(
        "GraphQLのDataLoaderパターン",
        "N+1問題を自動的に解決できる",
        "バッチ処理により、パフォーマンスが大幅に向上",
    );

    // パターンを認識
    let pattern_id = backend_agent.phronesis.recognize_pattern(
        "朝一番のデプロイは成功率が高い",
        "朝の時間帯はシステム負荷が低く、問題が起きても対応しやすい",
    );

    // パターンの発生を記録
    let mut context = HashMap::new();
    context.insert("time".to_string(), "09:00".to_string());
    context.insert("day".to_string(), "Monday".to_string());
    backend_agent.phronesis.record_pattern_occurrence(&pattern_id, context.clone(), true);
    
    context.insert("time".to_string(), "09:30".to_string());
    backend_agent.phronesis.record_pattern_occurrence(&pattern_id, context.clone(), true);
    
    context.insert("time".to_string(), "17:00".to_string());
    context.insert("day".to_string(), "Friday".to_string());
    backend_agent.phronesis.record_pattern_occurrence(&pattern_id, context, false);

    // 実践知の要約を表示
    println!("Frontend Agentの実践知:");
    let frontend_summary = frontend_agent.phronesis.summarize();
    println!("  総エントリー数: {}", frontend_summary.total_wisdom_entries);
    println!("  学習イベント数: {}", frontend_summary.total_learning_events);
    println!("  成功率: {:.1}%", frontend_summary.overall_success_rate * 100.0);

    println!("\nBackend Agentの実践知:");
    let backend_summary = backend_agent.phronesis.summarize();
    println!("  総エントリー数: {}", backend_summary.total_wisdom_entries);
    println!("  パターン数: {}", backend_summary.pattern_count);
    
    // 適用可能な知恵を検索
    let applicable_wisdom = backend_agent.phronesis.find_applicable_wisdom(
        "deployment",
        &vec!["morning".to_string()],
    );
    if !applicable_wisdom.is_empty() {
        println!("\n  デプロイに関する知恵:");
        for wisdom in applicable_wisdom {
            println!("    - {}", wisdom.insight);
        }
    }

    println!("\n=== 4. タスク実行による成長 ===\n");

    // タスクを実行して経験値を獲得
    let tasks = vec![
        Task::new(
            Uuid::new_v4().to_string(),
            "Reactコンポーネントのテスト作成".to_string(),
            Priority::Medium,
            TaskType::Testing,
        ),
        Task::new(
            Uuid::new_v4().to_string(),
            "UIのレスポンシブ対応".to_string(),
            Priority::High,
            TaskType::Feature,
        ),
        Task::new(
            Uuid::new_v4().to_string(),
            "パフォーマンス最適化".to_string(),
            Priority::High,
            TaskType::Bugfix,
        ),
    ];

    for task in tasks {
        println!("タスク実行: {}", task.description);
        frontend_agent.update_agent_experience(&task);
    }

    println!("\n成長後のFrontend Agent:");
    println!("  {}", frontend_agent.personality.describe_personality());
    println!("  スキルレベル:");
    for (name, skill) in &frontend_agent.personality.skills {
        println!("    - {}: {:?} (経験値: {})", 
            name, 
            skill.level,
            skill.experience_points
        );
    }

    println!("\n=== 5. 統合的な状態レポート ===\n");

    // エージェントの総合的な状態を表示
    println!("Frontend Agent 総合レポート:");
    println!("  個性スコア: {:.1}", frontend_agent.personality.composability_score());
    
    let whiteboard_summary = frontend_agent.whiteboard.summarize();
    println!("  ホワイトボード使用状況:");
    println!("    - 総エントリー数: {}", whiteboard_summary.total_entries);
    println!("    - 注釈数: {}", whiteboard_summary.total_annotations);
    println!("    - 修正数: {}", whiteboard_summary.total_revisions);
    
    let phronesis_summary = frontend_agent.phronesis.summarize();
    println!("  実践知の蓄積:");
    println!("    - 知恵のエントリー: {}", phronesis_summary.total_wisdom_entries);
    println!("    - 学習イベント: {}", phronesis_summary.total_learning_events);

    // 最も信頼できる知恵を取得
    if let Some(best_wisdom) = frontend_agent.phronesis.get_most_reliable_wisdom(&WisdomCategory::TaskExecution) {
        println!("\n  最も信頼できる実践知:");
        println!("    {}", best_wisdom.insight);
        println!("    信頼度: {:.1}%", best_wisdom.confidence * 100.0);
    }

    println!("\n✨ デモ完了！エージェントは個性を持ち、思考し、経験から学ぶ存在になりました。");

    Ok(())
}