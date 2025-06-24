//! 学習するエージェントのシミュレーション
//! 複数のタスクを通じてエージェントが成長する過程を観察

use ccswarm::agent::{
    ClaudeCodeAgent, Priority, Task, TaskType,
    AnnotationMarker, EntryType, SkillCategory, WisdomCategory,
};
use ccswarm::config::ClaudeConfig;
use ccswarm::identity::default_frontend_role;
use anyhow::Result;
use std::collections::HashMap;
use tempfile::TempDir;
use uuid::Uuid;
use colored::*;

#[tokio::main]
async fn main() -> Result<()> {
    // ログの初期化
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("{}", "🚀 学習エージェントシミュレーション".bold().cyan());
    println!("{}", "エージェントが複数のプロジェクトを通じて成長する様子を観察します\n".dimmed());

    let temp_dir = TempDir::new()?;
    let workspace = temp_dir.path().to_path_buf();

    let mut agent = ClaudeCodeAgent::new(
        default_frontend_role(),
        &workspace,
        "simulation",
        ClaudeConfig::default(),
    )
    .await?;

    // プロジェクトのシミュレーション
    let projects = vec![
        ("📱 モバイルアプリUI", vec![
            ("ログイン画面の実装", TaskType::Feature, Priority::High),
            ("レスポンシブ対応", TaskType::Feature, Priority::Medium),
            ("UIテストの作成", TaskType::Testing, Priority::Medium),
            ("パフォーマンス改善", TaskType::Bugfix, Priority::High),
        ]),
        ("🛒 ECサイトフロントエンド", vec![
            ("商品一覧の実装", TaskType::Feature, Priority::High),
            ("カート機能の実装", TaskType::Feature, Priority::Critical),
            ("決済フローのバグ修正", TaskType::Bugfix, Priority::Critical),
            ("ドキュメント作成", TaskType::Documentation, Priority::Low),
        ]),
        ("📊 ダッシュボード開発", vec![
            ("グラフコンポーネント実装", TaskType::Feature, Priority::High),
            ("リアルタイム更新機能", TaskType::Feature, Priority::High),
            ("アクセシビリティ改善", TaskType::Bugfix, Priority::Medium),
            ("統合テスト作成", TaskType::Testing, Priority::High),
        ]),
    ];

    for (project_idx, (project_name, tasks)) in projects.iter().enumerate() {
        println!("{}", format!("\n=== プロジェクト{}: {} ===", project_idx + 1, project_name).bold());
        
        // プロジェクト用のホワイトボードセクション作成
        let section_id = agent.whiteboard.create_section(project_name);
        
        // 思考の軌跡を開始
        let thought_trace = agent.whiteboard.start_thought_trace();
        agent.whiteboard.add_to_section(&section_id, &thought_trace);
        agent.whiteboard.add_thought(&thought_trace, &format!("{}の要件を分析中...", project_name));

        for (task_desc, task_type, priority) in tasks {
            let task = Task::new(
                Uuid::new_v4().to_string(),
                task_desc.to_string(),
                priority.clone(),
                task_type.clone(),
            );

            println!("\n  {} {}", "▶".green(), task_desc);
            println!("    タイプ: {:?}, 優先度: {:?}", task_type, priority);

            // タスク実行前のスキルレベルを記録
            let skill_before = agent.personality.skills.values()
                .map(|s| (s.category.clone(), s.level.clone(), s.experience_points))
                .collect::<Vec<_>>();

            // ホワイトボードに思考を追加
            agent.whiteboard.add_thought(&thought_trace, &format!("{}に取り組む", task_desc));

            // タスクごとの仮説を立てる
            let hypothesis = match task_type {
                TaskType::Feature => agent.whiteboard.add_hypothesis(
                    &format!("{}にはコンポーネント設計が重要", task_desc),
                    0.7,
                ),
                TaskType::Testing => agent.whiteboard.add_hypothesis(
                    "テストカバレッジ80%以上を目指す",
                    0.9,
                ),
                TaskType::Bugfix => agent.whiteboard.add_hypothesis(
                    "問題の根本原因を特定することが重要",
                    0.8,
                ),
                _ => agent.whiteboard.add_hypothesis(
                    "標準的なアプローチで対応可能",
                    0.6,
                ),
            };
            agent.whiteboard.add_to_section(&section_id, &hypothesis);

            // タスクを実行（経験値を獲得）
            agent.update_agent_experience(&task);

            // 実行結果に基づいて実践知を記録
            match (task_type, priority) {
                (TaskType::Feature, Priority::Critical) => {
                    agent.phronesis.record_success(
                        &task.id,
                        "段階的実装",
                        &format!("{}を小さなコンポーネントに分割して実装", task_desc),
                        "複雑な機能は段階的に実装することで品質が向上",
                    );
                }
                (TaskType::Bugfix, Priority::Critical | Priority::High) => {
                    if rand::random::<f32>() > 0.3 {  // 70%の確率で成功
                        agent.phronesis.record_success(
                            &task.id,
                            "デバッグ手法",
                            "ログとブレークポイントを活用",
                            "システマティックなデバッグアプローチが効果的",
                        );
                    } else {
                        agent.phronesis.record_failure(
                            &task.id,
                            "不完全な修正",
                            "表面的な修正のみ実施",
                            "根本原因の分析が不十分だった",
                        );
                    }
                }
                _ => {}
            }

            // スキルの成長を表示
            let skill_after = agent.personality.skills.values()
                .map(|s| (s.category.clone(), s.level.clone(), s.experience_points))
                .collect::<Vec<_>>();

            for (before, after) in skill_before.iter().zip(skill_after.iter()) {
                if before.2 != after.2 {  // 経験値が変化した場合
                    let exp_gain = after.2 - before.2;
                    println!("    {} +{} 経験値 ({:?})", 
                        "💡".yellow(), 
                        exp_gain,
                        before.0
                    );
                    
                    if before.1 != after.1 {  // レベルアップした場合
                        println!("    {} レベルアップ！ {:?} → {:?}", 
                            "🎉".bright_yellow(),
                            before.1,
                            after.1
                        );
                    }
                }
            }
        }

        // プロジェクト完了時の振り返り
        agent.whiteboard.set_conclusion(
            &thought_trace,
            &format!("{}を完了。多くの学びを得た", project_name),
        );

        // 重要な学びに注釈
        agent.whiteboard.annotate(
            &thought_trace,
            "このプロジェクトで得た知見は次に活かせる",
            AnnotationMarker::Insight,
        );

        // プロジェクト終了時の状態を表示
        println!("\n  {}", format!("プロジェクト完了時の状態:").bold());
        println!("    個性: {}", agent.personality.describe_personality());
        println!("    実践知エントリー: {}", agent.phronesis.wisdom_entries.len());
        
        // パターン認識
        if project_idx > 0 {
            let pattern_id = agent.phronesis.recognize_pattern(
                &format!("{}では品質重視のアプローチが有効", project_name),
                "時間をかけても品質を優先することで、後の手戻りが減る",
            );
            
            let mut context = HashMap::new();
            context.insert("project".to_string(), project_name.to_string());
            context.insert("approach".to_string(), "quality_first".to_string());
            agent.phronesis.record_pattern_occurrence(&pattern_id, context, true);
        }
    }

    // 最終的な成長の総括
    println!("{}", "\n=== 🎓 最終的な成長の総括 ===".bold().green());
    
    println!("\n{}", "スキルの成長:".bold());
    for (name, skill) in &agent.personality.skills {
        let level_str = format!("{:?}", skill.level);
        let bar_length = (skill.experience_points as f32 / 100.0).min(20.0) as usize;
        let progress_bar = "█".repeat(bar_length).green().to_string() + &"░".repeat(20 - bar_length).dimmed().to_string();
        
        println!("  {} {}: {} {} ({}exp)", 
            match &skill.category {
                SkillCategory::Technical => "⚙️ ",
                SkillCategory::Creative => "🎨",
                SkillCategory::Analytical => "🔍",
                _ => "📊",
            },
            name.pad_to_width(20),
            level_str.pad_to_width(10),
            progress_bar,
            skill.experience_points
        );
    }

    println!("\n{}", "実践知の蓄積:".bold());
    let summary = agent.phronesis.summarize();
    println!("  総エントリー: {}", summary.total_wisdom_entries);
    println!("  学習イベント: {}", summary.total_learning_events);
    println!("  認識パターン: {}", summary.pattern_count);
    println!("  成功率: {:.1}%", summary.overall_success_rate * 100.0);

    // カテゴリごとの最も信頼できる知恵を表示
    println!("\n{}", "獲得した主要な実践知:".bold());
    for category in &[
        WisdomCategory::TaskExecution,
        WisdomCategory::ErrorHandling,
        WisdomCategory::PatternRecognition,
    ] {
        if let Some(wisdom) = agent.phronesis.get_most_reliable_wisdom(category) {
            println!("  {} {:?}:", 
                match category {
                    WisdomCategory::TaskExecution => "🎯",
                    WisdomCategory::ErrorHandling => "🛠️",
                    WisdomCategory::PatternRecognition => "🔮",
                    _ => "📝",
                },
                category
            );
            println!("    「{}」", wisdom.insight);
            println!("    信頼度: {:.0}%, 適用回数: {}", 
                wisdom.confidence * 100.0,
                wisdom.applications.len()
            );
        }
    }

    println!("\n{}", "ホワイトボードの活用:".bold());
    let wb_summary = agent.whiteboard.summarize();
    println!("  総エントリー: {}", wb_summary.total_entries);
    println!("  思考セクション: {}", wb_summary.section_count);
    println!("  注釈: {}", wb_summary.total_annotations);
    
    // 各タイプのエントリー数を表示
    for (entry_type, count) in &wb_summary.type_counts {
        println!("  - {}: {}", entry_type, count);
    }

    println!("\n{}", "✨ シミュレーション完了！".bold().bright_green());
    println!("{}", "エージェントは3つのプロジェクトを通じて大きく成長しました。".dimmed());
    println!("{}", format!("最終的な個性: {}", agent.personality.describe_personality()).italic());

    Ok(())
}

// パディング用のトレイト
trait PadToWidth {
    fn pad_to_width(&self, width: usize) -> String;
}

impl PadToWidth for String {
    fn pad_to_width(&self, width: usize) -> String {
        format!("{:<width$}", self, width = width)
    }
}

impl PadToWidth for &str {
    fn pad_to_width(&self, width: usize) -> String {
        format!("{:<width$}", self, width = width)
    }
}