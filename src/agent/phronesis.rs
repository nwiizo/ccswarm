//! 実践知（フロネーシス）システム
//! 
//! 「本で読んだ知識と、実際にやってみて得た知識は違う。
//! 何度も作って、失敗して、コツを掴んで、初めて美味しい料理が作れるようになる」
//! という概念を実装。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 実践的な知恵のエントリー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticalWisdom {
    pub id: String,
    pub category: WisdomCategory,
    pub insight: String,
    pub context: WisdomContext,
    pub confidence: f32,
    pub applications: Vec<WisdomApplication>,
    pub created_at: DateTime<Utc>,
    pub last_applied: Option<DateTime<Utc>>,
}

/// 知恵のカテゴリー
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WisdomCategory {
    TaskExecution,      // タスク実行の知恵
    ErrorHandling,      // エラー対処の知恵
    Communication,      // コミュニケーションの知恵
    Optimization,       // 最適化の知恵
    PatternRecognition, // パターン認識の知恵
    Debugging,          // デバッグの知恵
    Collaboration,      // 協調作業の知恵
}

/// 知恵が適用される文脈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WisdomContext {
    pub task_types: Vec<String>,
    pub conditions: Vec<String>,
    pub constraints: Vec<String>,
}

/// 知恵の適用記録
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WisdomApplication {
    pub applied_at: DateTime<Utc>,
    pub task_id: String,
    pub success: bool,
    pub feedback: Option<String>,
    pub effectiveness_score: f32,
}

/// 学習イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub event_type: LearningEventType,
    pub description: String,
    pub task_context: HashMap<String, String>,
    pub outcome: LearningOutcome,
}

/// 学習イベントのタイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningEventType {
    Success {
        approach: String,
        time_saved: Option<std::time::Duration>,
    },
    Failure {
        error_type: String,
        root_cause: String,
    },
    Discovery {
        finding: String,
        significance: String,
    },
    Refinement {
        original_approach: String,
        improved_approach: String,
    },
}

/// 学習の結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningOutcome {
    pub lesson_learned: String,
    pub actionable_insight: Option<String>,
    pub applicable_situations: Vec<String>,
}

/// 実践知マネージャー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhronesisManager {
    pub agent_id: String,
    pub wisdom_entries: HashMap<String, PracticalWisdom>,
    pub learning_events: Vec<LearningEvent>,
    pub wisdom_by_category: HashMap<WisdomCategory, Vec<String>>,
    pub pattern_library: HashMap<String, Pattern>,
}

/// 認識されたパターン
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub occurrences: Vec<PatternOccurrence>,
    pub reliability: f32,
}

/// パターンの発生記録
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternOccurrence {
    pub timestamp: DateTime<Utc>,
    pub context: HashMap<String, String>,
    pub matched: bool,
}

impl PhronesisManager {
    /// 新しい実践知マネージャーを作成
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            wisdom_entries: HashMap::new(),
            learning_events: Vec::new(),
            wisdom_by_category: HashMap::new(),
            pattern_library: HashMap::new(),
        }
    }

    /// 学習イベントを記録
    pub fn record_learning_event(
        &mut self,
        event_type: LearningEventType,
        description: &str,
        task_context: HashMap<String, String>,
        outcome: LearningOutcome,
    ) -> String {
        let event_id = Uuid::new_v4().to_string();
        let event = LearningEvent {
            id: event_id.clone(),
            timestamp: Utc::now(),
            event_type,
            description: description.to_string(),
            task_context,
            outcome,
        };

        self.learning_events.push(event.clone());

        // 学習イベントから実践知を抽出
        if let Some(wisdom) = self.extract_wisdom_from_event(&event) {
            self.add_wisdom(wisdom);
        }

        event_id
    }

    /// 成功体験を記録
    pub fn record_success(
        &mut self,
        task_id: &str,
        approach: &str,
        description: &str,
        lesson: &str,
    ) -> String {
        let mut context = HashMap::new();
        context.insert("task_id".to_string(), task_id.to_string());

        self.record_learning_event(
            LearningEventType::Success {
                approach: approach.to_string(),
                time_saved: None,
            },
            description,
            context,
            LearningOutcome {
                lesson_learned: lesson.to_string(),
                actionable_insight: Some(format!("{}のアプローチが効果的", approach)),
                applicable_situations: vec!["類似のタスク".to_string()],
            },
        )
    }

    /// 失敗体験を記録
    pub fn record_failure(
        &mut self,
        task_id: &str,
        error_type: &str,
        root_cause: &str,
        lesson: &str,
    ) -> String {
        let mut context = HashMap::new();
        context.insert("task_id".to_string(), task_id.to_string());

        self.record_learning_event(
            LearningEventType::Failure {
                error_type: error_type.to_string(),
                root_cause: root_cause.to_string(),
            },
            &format!("エラー: {} - 原因: {}", error_type, root_cause),
            context,
            LearningOutcome {
                lesson_learned: lesson.to_string(),
                actionable_insight: Some(format!("{}を避けるため、事前にチェックする", root_cause)),
                applicable_situations: vec!["エラー防止".to_string()],
            },
        )
    }

    /// 発見を記録
    pub fn record_discovery(
        &mut self,
        finding: &str,
        significance: &str,
        lesson: &str,
    ) -> String {
        self.record_learning_event(
            LearningEventType::Discovery {
                finding: finding.to_string(),
                significance: significance.to_string(),
            },
            &format!("発見: {}", finding),
            HashMap::new(),
            LearningOutcome {
                lesson_learned: lesson.to_string(),
                actionable_insight: Some(finding.to_string()),
                applicable_situations: vec!["新しいアプローチ".to_string()],
            },
        )
    }

    /// 学習イベントから実践知を抽出
    fn extract_wisdom_from_event(&self, event: &LearningEvent) -> Option<PracticalWisdom> {
        let category = match &event.event_type {
            LearningEventType::Success { .. } => WisdomCategory::TaskExecution,
            LearningEventType::Failure { .. } => WisdomCategory::ErrorHandling,
            LearningEventType::Discovery { .. } => WisdomCategory::PatternRecognition,
            LearningEventType::Refinement { .. } => WisdomCategory::Optimization,
        };

        let wisdom = PracticalWisdom {
            id: Uuid::new_v4().to_string(),
            category,
            insight: event.outcome.lesson_learned.clone(),
            context: WisdomContext {
                task_types: vec![],
                conditions: event.outcome.applicable_situations.clone(),
                constraints: vec![],
            },
            confidence: 0.5, // 初期信頼度
            applications: vec![],
            created_at: Utc::now(),
            last_applied: None,
        };

        Some(wisdom)
    }

    /// 実践知を追加
    pub fn add_wisdom(&mut self, wisdom: PracticalWisdom) {
        let wisdom_id = wisdom.id.clone();
        let category = wisdom.category.clone();

        self.wisdom_entries.insert(wisdom_id.clone(), wisdom);
        self.wisdom_by_category
            .entry(category)
            .or_insert_with(Vec::new)
            .push(wisdom_id);
    }

    /// 状況に適した実践知を検索
    pub fn find_applicable_wisdom(
        &self,
        task_type: &str,
        conditions: &[String],
    ) -> Vec<&PracticalWisdom> {
        self.wisdom_entries
            .values()
            .filter(|wisdom| {
                // 文脈がマッチするかチェック
                wisdom.context.task_types.is_empty()
                    || wisdom.context.task_types.contains(&task_type.to_string())
            })
            .filter(|wisdom| {
                // 条件が満たされているかチェック
                conditions.iter().any(|cond| {
                    wisdom.context.conditions.contains(cond)
                })
            })
            .collect()
    }

    /// 実践知を適用して結果を記録
    pub fn apply_wisdom(
        &mut self,
        wisdom_id: &str,
        task_id: &str,
        success: bool,
        feedback: Option<String>,
    ) -> Option<()> {
        let wisdom = self.wisdom_entries.get_mut(wisdom_id)?;

        let application = WisdomApplication {
            applied_at: Utc::now(),
            task_id: task_id.to_string(),
            success,
            feedback,
            effectiveness_score: if success { 1.0 } else { 0.0 },
        };

        wisdom.applications.push(application);
        wisdom.last_applied = Some(Utc::now());

        // 信頼度を更新（成功率に基づく）
        let success_count = wisdom
            .applications
            .iter()
            .filter(|app| app.success)
            .count() as f32;
        let total_count = wisdom.applications.len() as f32;
        wisdom.confidence = (success_count / total_count).min(1.0);

        Some(())
    }

    /// パターンを認識して記録
    pub fn recognize_pattern(&mut self, name: &str, description: &str) -> String {
        let pattern_id = Uuid::new_v4().to_string();
        let pattern = Pattern {
            id: pattern_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            occurrences: vec![],
            reliability: 0.0,
        };

        self.pattern_library.insert(pattern_id.clone(), pattern);
        pattern_id
    }

    /// パターンの発生を記録
    pub fn record_pattern_occurrence(
        &mut self,
        pattern_id: &str,
        context: HashMap<String, String>,
        matched: bool,
    ) -> Option<()> {
        let pattern = self.pattern_library.get_mut(pattern_id)?;

        let occurrence = PatternOccurrence {
            timestamp: Utc::now(),
            context,
            matched,
        };

        pattern.occurrences.push(occurrence);

        // 信頼性を更新
        let match_count = pattern
            .occurrences
            .iter()
            .filter(|occ| occ.matched)
            .count() as f32;
        let total_count = pattern.occurrences.len() as f32;
        pattern.reliability = if total_count > 0.0 {
            match_count / total_count
        } else {
            0.0
        };

        Some(())
    }

    /// 最も信頼できる実践知を取得
    pub fn get_most_reliable_wisdom(&self, category: &WisdomCategory) -> Option<&PracticalWisdom> {
        self.wisdom_by_category
            .get(category)?
            .iter()
            .filter_map(|id| self.wisdom_entries.get(id))
            .max_by(|a, b| {
                a.confidence
                    .partial_cmp(&b.confidence)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// 実践知の要約を生成
    pub fn summarize(&self) -> PhronesisSummary {
        let mut category_counts = HashMap::new();
        let mut total_applications = 0;
        let mut successful_applications = 0;

        for wisdom in self.wisdom_entries.values() {
            *category_counts
                .entry(format!("{:?}", wisdom.category))
                .or_insert(0) += 1;
            
            for app in &wisdom.applications {
                total_applications += 1;
                if app.success {
                    successful_applications += 1;
                }
            }
        }

        let overall_success_rate = if total_applications > 0 {
            successful_applications as f32 / total_applications as f32
        } else {
            0.0
        };

        PhronesisSummary {
            agent_id: self.agent_id.clone(),
            total_wisdom_entries: self.wisdom_entries.len(),
            category_counts,
            total_learning_events: self.learning_events.len(),
            pattern_count: self.pattern_library.len(),
            overall_success_rate,
        }
    }
}

/// 実践知の要約
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhronesisSummary {
    pub agent_id: String,
    pub total_wisdom_entries: usize,
    pub category_counts: HashMap<String, usize>,
    pub total_learning_events: usize,
    pub pattern_count: usize,
    pub overall_success_rate: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_success_and_extract_wisdom() {
        let mut manager = PhronesisManager::new("test-agent".to_string());
        
        let event_id = manager.record_success(
            "task-123",
            "早期リターン",
            "複雑な条件分岐を早期リターンで簡潔に",
            "ネストを深くするより、早期リターンで可読性を上げる",
        );
        
        assert!(!event_id.is_empty());
        assert_eq!(manager.learning_events.len(), 1);
        assert_eq!(manager.wisdom_entries.len(), 1);
    }

    #[test]
    fn test_wisdom_application_and_confidence() {
        let mut manager = PhronesisManager::new("test-agent".to_string());
        
        // 実践知を直接追加
        let wisdom = PracticalWisdom {
            id: "wisdom-1".to_string(),
            category: WisdomCategory::TaskExecution,
            insight: "テストデータは最小限に".to_string(),
            context: WisdomContext {
                task_types: vec!["testing".to_string()],
                conditions: vec!["unit_test".to_string()],
                constraints: vec![],
            },
            confidence: 0.5,
            applications: vec![],
            created_at: Utc::now(),
            last_applied: None,
        };
        
        manager.add_wisdom(wisdom);
        
        // 成功と失敗を記録
        manager.apply_wisdom("wisdom-1", "task-1", true, None);
        manager.apply_wisdom("wisdom-1", "task-2", true, None);
        manager.apply_wisdom("wisdom-1", "task-3", false, Some("特殊ケースでは適用不可".to_string()));
        
        let wisdom = &manager.wisdom_entries["wisdom-1"];
        assert_eq!(wisdom.applications.len(), 3);
        assert!((wisdom.confidence - 0.666).abs() < 0.01); // 2/3 ≈ 0.666
    }

    #[test]
    fn test_pattern_recognition() {
        let mut manager = PhronesisManager::new("test-agent".to_string());
        
        let pattern_id = manager.recognize_pattern(
            "Friday Afternoon Pattern",
            "金曜午後のタスクは月曜朝の対応を期待される",
        );
        
        // パターンの発生を記録
        let mut context = HashMap::new();
        context.insert("day".to_string(), "Friday".to_string());
        context.insert("time".to_string(), "afternoon".to_string());
        
        manager.record_pattern_occurrence(&pattern_id, context.clone(), true);
        manager.record_pattern_occurrence(&pattern_id, context.clone(), true);
        manager.record_pattern_occurrence(&pattern_id, context, false);
        
        let pattern = &manager.pattern_library[&pattern_id];
        assert_eq!(pattern.occurrences.len(), 3);
        assert!((pattern.reliability - 0.666).abs() < 0.01);
    }
}