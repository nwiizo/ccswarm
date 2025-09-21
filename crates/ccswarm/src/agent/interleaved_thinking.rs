use serde::{Deserialize, Serialize};

/// Thinking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingConfig {
    pub max_depth: usize,
    pub timeout_ms: u64,
    pub parallel_thoughts: bool,
}

impl Default for ThinkingConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            timeout_ms: 5000,
            parallel_thoughts: false,
        }
    }
}

/// Interleaved thinking module for agent reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterleavedThinking {
    pub thoughts: Vec<String>,
    pub current_context: String,
}

impl InterleavedThinking {
    pub fn new() -> Self {
        Self {
            thoughts: Vec::new(),
            current_context: String::new(),
        }
    }

    pub fn add_thought(&mut self, thought: String) {
        self.thoughts.push(thought);
    }

    pub fn update_context(&mut self, context: String) {
        self.current_context = context;
    }

    pub fn with_config(self, _config: ThinkingConfig) -> Self {
        // Apply configuration
        self
    }

    pub fn process_observation(&mut self, observation: String) {
        self.thoughts.push(format!("Observation: {}", observation));
    }

    pub fn get_thinking_summary(&self) -> String {
        self.thoughts.join("\n")
    }
}

impl Default for InterleavedThinking {
    fn default() -> Self {
        Self::new()
    }
}

/// Decision structure for interleaved thinking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub choice: String,
    pub reasoning: Vec<String>,
    pub confidence: f32,
}

impl Decision {
    pub fn new(choice: String) -> Self {
        Self {
            choice,
            reasoning: Vec::new(),
            confidence: 0.5,
        }
    }

    pub fn with_reasoning(mut self, reason: String) -> Self {
        self.reasoning.push(reason);
        self
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }
}

/// Interleaved thinking engine
pub type InterleavedThinkingEngine = InterleavedThinking;

/// Decision type for agent thinking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionType {
    Continue {
        reason: String,
    },
    RequestContext {
        questions: Vec<String>,
    },
    Refine {
        refinement: String,
        reason: String,
    },
    Complete {
        summary: String,
    },
    Pivot {
        new_direction: String,
        reason: String,
    },
    Abort {
        reason: String,
    },
}

/// Thinking step in the process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingStep {
    pub observation: String,
    pub analysis: String,
    pub decision: DecisionType,
}

impl ThinkingStep {
    pub fn new(observation: String, analysis: String, decision: DecisionType) -> Self {
        Self {
            observation,
            analysis,
            decision,
        }
    }
}