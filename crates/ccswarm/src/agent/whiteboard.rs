//! Whiteboard - Visualizing Agent Thinking
//!
//! "When solving complex problems, humans think while writing on paper.
//! Agents need a similar space." This module implements that concept.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Whiteboard entry types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    /// Calculation or expression expansion
    Calculation {
        expression: String,
        result: Option<String>,
    },
    /// Diagram or visual representation
    Diagram {
        description: String,
        elements: Vec<DiagramElement>,
    },
    /// Ideas or notes
    Note { content: String, tags: Vec<String> },
    /// Hypothesis or reasoning
    Hypothesis {
        statement: String,
        confidence: f32,
        evidence: Vec<String>,
    },
    /// TODO list
    TodoList { items: Vec<TodoItem> },
    /// Comparison table
    ComparisonTable {
        options: Vec<String>,
        criteria: Vec<String>,
        scores: HashMap<(String, String), f32>,
    },
    /// Thought trace
    ThoughtTrace {
        thoughts: Vec<String>,
        conclusion: Option<String>,
    },
}

/// Diagram element
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagramElement {
    pub id: String,
    pub element_type: String,
    pub label: String,
    pub connections: Vec<String>,
}

/// TODO item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TodoItem {
    pub id: String,
    pub task: String,
    pub completed: bool,
    pub priority: u8,
}

/// Whiteboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteboardEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub entry_type: EntryType,
    pub annotations: Vec<Annotation>,
    pub revisions: Vec<Revision>,
}

/// Annotation (notes added later)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub content: String,
    pub marker: AnnotationMarker,
}

/// Annotation marker type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnotationMarker {
    Important,    // Important
    Question,     // Question
    Verification, // Needs verification
    Correction,   // Correction
    Insight,      // Insight
}

/// Revision history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub previous_content: Option<String>,
}

/// Whiteboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Whiteboard {
    pub agent_id: String,
    pub created_at: DateTime<Utc>,
    pub entries: HashMap<String, WhiteboardEntry>,
    pub entry_order: VecDeque<String>,
    pub sections: HashMap<String, Section>,
}

/// Section (groups entries together)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub name: String,
    pub entry_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl Whiteboard {
    /// Create a new whiteboard
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            created_at: Utc::now(),
            entries: HashMap::new(),
            entry_order: VecDeque::new(),
            sections: HashMap::new(),
        }
    }

    /// Add an entry
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

    /// Record a calculation
    pub fn add_calculation(&mut self, expression: &str) -> String {
        self.add_entry(EntryType::Calculation {
            expression: expression.to_string(),
            result: None,
        })
    }

    /// Update calculation result
    pub fn update_calculation_result(&mut self, entry_id: &str, result: &str) -> Option<()> {
        let entry = self.entries.get_mut(entry_id)?;

        if let EntryType::Calculation {
            expression,
            result: res,
        } = &mut entry.entry_type
        {
            entry.revisions.push(Revision {
                timestamp: Utc::now(),
                description: format!("Added calculation result: {}", result),
                previous_content: Some(expression.clone()),
            });
            *res = Some(result.to_string());
            Some(())
        } else {
            None
        }
    }

    /// Add a note
    pub fn add_note(&mut self, content: &str, tags: Vec<String>) -> String {
        self.add_entry(EntryType::Note {
            content: content.to_string(),
            tags,
        })
    }

    /// Add a hypothesis
    pub fn add_hypothesis(&mut self, statement: &str, confidence: f32) -> String {
        self.add_entry(EntryType::Hypothesis {
            statement: statement.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            evidence: Vec::new(),
        })
    }

    /// Add evidence to a hypothesis
    pub fn add_evidence(&mut self, entry_id: &str, evidence: &str) -> Option<()> {
        let entry = self.entries.get_mut(entry_id)?;

        if let EntryType::Hypothesis { evidence: ev, .. } = &mut entry.entry_type {
            ev.push(evidence.to_string());
            entry.revisions.push(Revision {
                timestamp: Utc::now(),
                description: format!("Added evidence: {}", evidence),
                previous_content: None,
            });
            Some(())
        } else {
            None
        }
    }

    /// Record a thought trace
    pub fn start_thought_trace(&mut self) -> String {
        self.add_entry(EntryType::ThoughtTrace {
            thoughts: Vec::new(),
            conclusion: None,
        })
    }

    /// Add a thought
    pub fn add_thought(&mut self, entry_id: &str, thought: &str) -> Option<()> {
        let entry = self.entries.get_mut(entry_id)?;

        if let EntryType::ThoughtTrace { thoughts, .. } = &mut entry.entry_type {
            thoughts.push(thought.to_string());
            Some(())
        } else {
            None
        }
    }

    /// Set conclusion
    pub fn set_conclusion(&mut self, entry_id: &str, conclusion: &str) -> Option<()> {
        let entry = self.entries.get_mut(entry_id)?;

        if let EntryType::ThoughtTrace {
            conclusion: conc, ..
        } = &mut entry.entry_type
        {
            *conc = Some(conclusion.to_string());
            entry.revisions.push(Revision {
                timestamp: Utc::now(),
                description: "Set conclusion".to_string(),
                previous_content: None,
            });
            Some(())
        } else {
            None
        }
    }

    /// Add annotation
    pub fn annotate(
        &mut self,
        entry_id: &str,
        content: &str,
        marker: AnnotationMarker,
    ) -> Option<()> {
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

    /// Create a section
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

    /// Add entry to a section
    pub fn add_to_section(&mut self, section_id: &str, entry_id: &str) -> Option<()> {
        let section = self.sections.get_mut(section_id)?;
        if self.entries.contains_key(entry_id) && !section.entry_ids.contains(&entry_id.to_string())
        {
            section.entry_ids.push(entry_id.to_string());
            Some(())
        } else {
            None
        }
    }

    /// Get recent entries
    pub fn recent_entries(&self, count: usize) -> Vec<&WhiteboardEntry> {
        self.entry_order
            .iter()
            .rev()
            .take(count)
            .filter_map(|id| self.entries.get(id))
            .collect()
    }

    /// Search entries by type
    pub fn find_entries_by_type(
        &self,
        entry_type_filter: impl Fn(&EntryType) -> bool,
    ) -> Vec<&WhiteboardEntry> {
        self.entries
            .values()
            .filter(|entry| entry_type_filter(&entry.entry_type))
            .collect()
    }

    /// Generate whiteboard summary
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

/// Whiteboard summary
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
