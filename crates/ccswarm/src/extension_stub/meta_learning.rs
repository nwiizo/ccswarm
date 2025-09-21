use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Meta-learning system for agent improvement
#[derive(Debug, Clone)]
pub struct MetaLearningSystem {
    pub learning_history: Vec<LearningEvent>,
    pub patterns: HashMap<String, Pattern>,
    pub insights: Vec<Insight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: LearningEventType,
    pub context: String,
    pub outcome: Outcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningEventType {
    TaskCompleted,
    ErrorEncountered,
    PerformanceImproved,
    NewCapabilityLearned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Outcome {
    Success,
    Failure,
    Partial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub occurrences: u32,
    pub success_rate: f32,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub title: String,
    pub description: String,
    pub confidence: f32,
    pub actionable: bool,
}

impl MetaLearningSystem {
    pub fn new() -> Self {
        Self {
            learning_history: Vec::new(),
            patterns: HashMap::new(),
            insights: Vec::new(),
        }
    }

    pub fn record_event(&mut self, event: LearningEvent) {
        self.learning_history.push(event);
        self.analyze_patterns();
    }

    pub fn analyze_patterns(&mut self) {
        // Stub implementation - would analyze learning history for patterns
        // and generate insights
    }

    pub fn get_recommendations(&self) -> Vec<String> {
        self.patterns
            .values()
            .flat_map(|p| p.recommendations.clone())
            .collect()
    }

    pub fn get_insights(&self) -> &[Insight] {
        &self.insights
    }
}

impl Default for MetaLearningSystem {
    fn default() -> Self {
        Self::new()
    }
}