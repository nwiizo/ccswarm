//! ホワイトボード - エージェントの思考の見える化
//! 
//! 「複雑な問題を解くとき、人間は紙に書きながら考える。
//! エージェントにも同じような場所が必要だ」という概念を実装。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// ホワイトボードのエントリータイプ
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    /// 計算や式の展開
    Calculation {
        expression: String,
        result: Option<String>,
    },
    /// 図やダイアグラム
    Diagram {
        description: String,
        elements: Vec<DiagramElement>,
    },
    /// アイデアやメモ
    Note {
        content: String,
        tags: Vec<String>,
    },
    /// 仮説や推論
    Hypothesis {
        statement: String,
        confidence: f32,
        evidence: Vec<String>,
    },
    /// TODOリスト
    TodoList {
        items: Vec<TodoItem>,
    },
    /// 比較表
    ComparisonTable {
        options: Vec<String>,
        criteria: Vec<String>,
        scores: HashMap<(String, String), f32>,
    },
    /// 思考の軌跡
    ThoughtTrace {
        thoughts: Vec<String>,
        conclusion: Option<String>,
    },
}

/// 図の要素
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagramElement {
    pub id: String,
    pub element_type: String,
    pub label: String,
    pub connections: Vec<String>,
}

/// TODOアイテム
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TodoItem {
    pub id: String,
    pub task: String,
    pub completed: bool,
    pub priority: u8,
}

/// ホワイトボードのエントリー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub entry_type: EntryType,
    pub annotations: Vec<Annotation>,
    pub revisions: Vec<Revision>,
}

/// 注釈（後から追加されるメモ）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub marker: AnnotationMarker,
}

/// 注釈のマーカータイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationMarker {
    Important,      // 重要
    Question,       // 疑問
    Verification,   // 要検証
    Correction,     // 訂正
    Insight,        // 洞察
}

/// 修正履歴
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub previous_content: Option<String>,
}

/// ホワイトボード
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whiteboard {
    pub agent_id: String,
    pub created_at: DateTime<Utc>,
    pub entries: HashMap<String, WhiteboardEntry>,
    pub entry_order: VecDeque<String>,
    pub sections: HashMap<String, Section>,
}

/// セクション（エントリーをグループ化）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub name: String,
    pub entry_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl Whiteboard {
    /// 新しいホワイトボードを作成
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            created_at: Utc::now(),
            entries: HashMap::new(),
            entry_order: VecDeque::new(),
            sections: HashMap::new(),
        }
    }

    /// エントリーを追加
    pub fn add_entry(&mut self, entry_type: EntryType) -> String {
        let entry_id = Uuid::new_v4().to_string();
        let entry = WhiteboardEntry {
            id: entry_id.clone(),
            timestamp: Utc::now(),
            entry_type,
            annotations: Vec::new(),
            revisions: Vec::new(),
        };
        
        self.entries.insert(entry_id.clone(), entry);
        self.entry_order.push_back(entry_id.clone());
        
        entry_id
    }

    /// 計算を記録
    pub fn add_calculation(&mut self, expression: &str) -> String {
        self.add_entry(EntryType::Calculation {
            expression: expression.to_string(),
            result: None,
        })
    }

    /// 計算結果を更新
    pub fn update_calculation_result(&mut self, entry_id: &str, result: &str) -> Option<()> {
        let entry = self.entries.get_mut(entry_id)?;
        
        if let EntryType::Calculation { expression, result: ref mut res } = &mut entry.entry_type {
            entry.revisions.push(Revision {
                timestamp: Utc::now(),
                description: format!("計算結果を追加: {}", result),
                previous_content: Some(expression.clone()),
            });
            *res = Some(result.to_string());
            Some(())
        } else {
            None
        }
    }

    /// メモを追加
    pub fn add_note(&mut self, content: &str, tags: Vec<String>) -> String {
        self.add_entry(EntryType::Note {
            content: content.to_string(),
            tags,
        })
    }

    /// 仮説を追加
    pub fn add_hypothesis(&mut self, statement: &str, confidence: f32) -> String {
        self.add_entry(EntryType::Hypothesis {
            statement: statement.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            evidence: Vec::new(),
        })
    }

    /// 仮説に証拠を追加
    pub fn add_evidence(&mut self, entry_id: &str, evidence: &str) -> Option<()> {
        let entry = self.entries.get_mut(entry_id)?;
        
        if let EntryType::Hypothesis { evidence: ref mut ev, .. } = &mut entry.entry_type {
            ev.push(evidence.to_string());
            entry.revisions.push(Revision {
                timestamp: Utc::now(),
                description: format!("証拠を追加: {}", evidence),
                previous_content: None,
            });
            Some(())
        } else {
            None
        }
    }

    /// 思考の軌跡を記録
    pub fn start_thought_trace(&mut self) -> String {
        self.add_entry(EntryType::ThoughtTrace {
            thoughts: Vec::new(),
            conclusion: None,
        })
    }

    /// 思考を追加
    pub fn add_thought(&mut self, entry_id: &str, thought: &str) -> Option<()> {
        let entry = self.entries.get_mut(entry_id)?;
        
        if let EntryType::ThoughtTrace { thoughts, .. } = &mut entry.entry_type {
            thoughts.push(thought.to_string());
            Some(())
        } else {
            None
        }
    }

    /// 結論を設定
    pub fn set_conclusion(&mut self, entry_id: &str, conclusion: &str) -> Option<()> {
        let entry = self.entries.get_mut(entry_id)?;
        
        if let EntryType::ThoughtTrace { conclusion: ref mut conc, .. } = &mut entry.entry_type {
            *conc = Some(conclusion.to_string());
            entry.revisions.push(Revision {
                timestamp: Utc::now(),
                description: "結論を設定".to_string(),
                previous_content: None,
            });
            Some(())
        } else {
            None
        }
    }

    /// 注釈を追加
    pub fn annotate(&mut self, entry_id: &str, content: &str, marker: AnnotationMarker) -> Option<()> {
        let entry = self.entries.get_mut(entry_id)?;
        
        let annotation = Annotation {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            content: content.to_string(),
            marker,
        };
        
        entry.annotations.push(annotation);
        Some(())
    }

    /// セクションを作成
    pub fn create_section(&mut self, name: &str) -> String {
        let section_id = Uuid::new_v4().to_string();
        let section = Section {
            id: section_id.clone(),
            name: name.to_string(),
            entry_ids: Vec::new(),
            created_at: Utc::now(),
        };
        
        self.sections.insert(section_id.clone(), section);
        section_id
    }

    /// エントリーをセクションに追加
    pub fn add_to_section(&mut self, section_id: &str, entry_id: &str) -> Option<()> {
        let section = self.sections.get_mut(section_id)?;
        if self.entries.contains_key(entry_id) && !section.entry_ids.contains(&entry_id.to_string()) {
            section.entry_ids.push(entry_id.to_string());
            Some(())
        } else {
            None
        }
    }

    /// 最近のエントリーを取得
    pub fn recent_entries(&self, count: usize) -> Vec<&WhiteboardEntry> {
        self.entry_order
            .iter()
            .rev()
            .take(count)
            .filter_map(|id| self.entries.get(id))
            .collect()
    }

    /// 特定のタイプのエントリーを検索
    pub fn find_entries_by_type(&self, entry_type_filter: impl Fn(&EntryType) -> bool) -> Vec<&WhiteboardEntry> {
        self.entries
            .values()
            .filter(|entry| entry_type_filter(&entry.entry_type))
            .collect()
    }

    /// ホワイトボードの要約を生成
    pub fn summarize(&self) -> WhiteboardSummary {
        let mut type_counts = HashMap::new();
        let mut total_annotations = 0;
        let mut total_revisions = 0;

        for entry in self.entries.values() {
            let type_name = match &entry.entry_type {
                EntryType::Calculation { .. } => "calculations",
                EntryType::Diagram { .. } => "diagrams",
                EntryType::Note { .. } => "notes",
                EntryType::Hypothesis { .. } => "hypotheses",
                EntryType::TodoList { .. } => "todo_lists",
                EntryType::ComparisonTable { .. } => "comparisons",
                EntryType::ThoughtTrace { .. } => "thought_traces",
            };
            
            *type_counts.entry(type_name.to_string()).or_insert(0) += 1;
            total_annotations += entry.annotations.len();
            total_revisions += entry.revisions.len();
        }

        WhiteboardSummary {
            agent_id: self.agent_id.clone(),
            created_at: self.created_at,
            total_entries: self.entries.len(),
            type_counts,
            total_annotations,
            total_revisions,
            section_count: self.sections.len(),
        }
    }
}

/// ホワイトボードの要約
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardSummary {
    pub agent_id: String,
    pub created_at: DateTime<Utc>,
    pub total_entries: usize,
    pub type_counts: HashMap<String, usize>,
    pub total_annotations: usize,
    pub total_revisions: usize,
    pub section_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whiteboard_calculation() {
        let mut whiteboard = Whiteboard::new("test-agent".to_string());
        
        // 計算を追加
        let entry_id = whiteboard.add_calculation("317 × 456");
        assert_eq!(whiteboard.entries.len(), 1);
        
        // 結果を更新
        whiteboard.update_calculation_result(&entry_id, "144,552");
        
        let entry = &whiteboard.entries[&entry_id];
        if let EntryType::Calculation { expression, result } = &entry.entry_type {
            assert_eq!(expression, "317 × 456");
            assert_eq!(result.as_ref().unwrap(), "144,552");
        } else {
            panic!("Wrong entry type");
        }
        
        assert_eq!(entry.revisions.len(), 1);
    }

    #[test]
    fn test_thought_trace() {
        let mut whiteboard = Whiteboard::new("test-agent".to_string());
        
        let trace_id = whiteboard.start_thought_trace();
        whiteboard.add_thought(&trace_id, "最初は単純な解法を考えた");
        whiteboard.add_thought(&trace_id, "でも、エッジケースで問題が発生");
        whiteboard.add_thought(&trace_id, "別のアプローチを試してみる");
        whiteboard.set_conclusion(&trace_id, "再帰的な解法が最適");
        
        let entry = &whiteboard.entries[&trace_id];
        if let EntryType::ThoughtTrace { thoughts, conclusion } = &entry.entry_type {
            assert_eq!(thoughts.len(), 3);
            assert_eq!(conclusion.as_ref().unwrap(), "再帰的な解法が最適");
        }
    }

    #[test]
    fn test_annotations() {
        let mut whiteboard = Whiteboard::new("test-agent".to_string());
        
        let note_id = whiteboard.add_note("APIのレスポンスが遅い", vec!["performance".to_string()]);
        whiteboard.annotate(&note_id, "キャッシュを検討", AnnotationMarker::Important);
        whiteboard.annotate(&note_id, "本当に遅いのか測定が必要", AnnotationMarker::Verification);
        
        let entry = &whiteboard.entries[&note_id];
        assert_eq!(entry.annotations.len(), 2);
    }
}