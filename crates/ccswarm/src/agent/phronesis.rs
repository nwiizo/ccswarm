use serde::{Deserialize, Serialize};

/// Phronesis - practical wisdom module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phronesis {
    pub wisdom_points: Vec<WisdomPoint>,
    pub experience_level: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WisdomPoint {
    pub context: String,
    pub lesson: String,
    pub confidence: f32,
}

impl Phronesis {
    pub fn new() -> Self {
        Self {
            wisdom_points: Vec::new(),
            experience_level: 0,
        }
    }

    pub fn add_wisdom(&mut self, context: String, lesson: String, confidence: f32) {
        self.wisdom_points.push(WisdomPoint {
            context,
            lesson,
            confidence,
        });
        self.experience_level += 1;
    }

    pub fn get_relevant_wisdom(&self, context: &str) -> Vec<&WisdomPoint> {
        self.wisdom_points
            .iter()
            .filter(|w| w.context.contains(context))
            .collect()
    }

    pub fn record_learning_event(&mut self, context: String, outcome: bool) {
        let confidence = if outcome { 0.8 } else { 0.3 };
        let lesson = if outcome {
            format!("Successfully handled {}", context)
        } else {
            format!("Failed to handle {}", context)
        };
        self.add_wisdom(context, lesson, confidence);
    }

    pub fn record_success(&mut self, context: String) {
        self.record_learning_event(context, true);
    }

    pub fn record_failure(&mut self, context: String) {
        self.record_learning_event(context, false);
    }

    pub fn summarize(&self) -> String {
        format!(
            "Experience level: {}, Wisdom points: {}",
            self.experience_level,
            self.wisdom_points.len()
        )
    }
}

impl Default for Phronesis {
    fn default() -> Self {
        Self::new()
    }
}

/// Learning event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningEventType {
    Discovery {
        concept: String,
        context: String,
    },
    Success {
        action: String,
        outcome: String,
    },
    Failure {
        action: String,
        reason: String,
    },
    Adaptation {
        old_approach: String,
        new_approach: String,
    },
}

/// Phronesis manager
pub type PhronesisManager = Phronesis;

/// Practical wisdom
pub type PracticalWisdom = Phronesis;

/// Wisdom category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WisdomCategory {
    Technical,
    Process,
    Communication,
    ProblemSolving,
    Design,
}

/// Learning outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningOutcome {
    pub success: bool,
    pub insights: Vec<String>,
    pub improvements: Vec<String>,
    pub confidence: f32,
    pub lesson_learned: Option<String>,
    pub actionable_insight: Option<String>,
    pub applicable_situations: Vec<String>,
}

impl LearningOutcome {
    pub fn new(success: bool) -> Self {
        Self {
            success,
            insights: Vec::new(),
            improvements: Vec::new(),
            confidence: 0.5,
            lesson_learned: None,
            actionable_insight: None,
            applicable_situations: Vec::new(),
        }
    }

    pub fn add_insight(&mut self, insight: String) {
        self.insights.push(insight);
    }

    pub fn add_improvement(&mut self, improvement: String) {
        self.improvements.push(improvement);
    }
}
