use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

/// Represents a thinking step in the interleaved thinking process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingStep {
    /// The context or observation at this step
    pub observation: String,
    /// The analysis or reflection on the observation
    pub reflection: String,
    /// The decision or next action based on reflection
    pub decision: Decision,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Optional reasoning trace for debugging
    pub reasoning_trace: Option<String>,
}

/// Decisions that can be made during thinking steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    /// Continue with the current approach
    Continue { reason: String },
    /// Refine the approach with new information
    Refine { refinement: String, reason: String },
    /// Request additional context or clarification
    RequestContext { questions: Vec<String> },
    /// Pivot to a different approach
    Pivot {
        new_approach: String,
        reason: String,
    },
    /// Complete the current phase
    Complete { summary: String },
    /// Abort due to issues
    Abort { reason: String },
}

/// Manages interleaved thinking for agents
pub struct InterleavedThinkingEngine {
    /// History of thinking steps
    thinking_history: Vec<ThinkingStep>,
    /// Current context being evaluated
    current_context: String,
    /// Maximum thinking steps before forcing a decision
    max_steps: usize,
    /// Minimum confidence threshold for continuing
    confidence_threshold: f64,
}

impl Default for InterleavedThinkingEngine {
    fn default() -> Self {
        Self {
            thinking_history: Vec::new(),
            current_context: String::new(),
            max_steps: 10,
            confidence_threshold: 0.6,
        }
    }
}

impl InterleavedThinkingEngine {
    /// Create a new thinking engine
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure the engine
    pub fn with_config(mut self, max_steps: usize, confidence_threshold: f64) -> Self {
        self.max_steps = max_steps;
        self.confidence_threshold = confidence_threshold;
        self
    }

    /// Process an observation and generate a thinking step
    pub async fn process_observation(
        &mut self,
        observation: &str,
        agent_role: &str,
    ) -> Result<ThinkingStep> {
        // Update context
        self.current_context = format!(
            "{}\n\nNew observation: {}",
            self.current_context, observation
        );

        // Generate reflection based on role and observation
        let reflection = self.generate_reflection(observation, agent_role).await?;

        // Analyze confidence based on clarity and completeness
        let confidence = self.analyze_confidence(observation, &reflection);

        // Make a decision based on reflection and confidence
        let decision = self
            .make_decision(&reflection, confidence, agent_role)
            .await?;

        // Create thinking step
        let step = ThinkingStep {
            observation: observation.to_string(),
            reflection: reflection.clone(),
            decision,
            confidence,
            reasoning_trace: Some(format!(
                "Role: {}, History length: {}, Confidence: {:.2}",
                agent_role,
                self.thinking_history.len(),
                confidence
            )),
        };

        // Add to history
        self.thinking_history.push(step.clone());

        info!(
            "Thinking step #{}: confidence={:.2}, decision={:?}",
            self.thinking_history.len(),
            confidence,
            step.decision
        );

        Ok(step)
    }

    /// Generate a reflection based on observation and role
    async fn generate_reflection(&self, observation: &str, agent_role: &str) -> Result<String> {
        // Role-specific reflection patterns
        let reflection = match agent_role {
            "Frontend" => {
                if observation.contains("error") || observation.contains("failed") {
                    format!(
                        "UI component issue detected: {}. Need to check React hooks, \
                         state management, and rendering logic.",
                        observation
                    )
                } else if observation.contains("style") || observation.contains("css") {
                    format!(
                        "Styling observation: {}. Should verify responsive design \
                         and cross-browser compatibility.",
                        observation
                    )
                } else {
                    format!(
                        "Frontend progress: {}. Continuing with component implementation.",
                        observation
                    )
                }
            }
            "Backend" => {
                if observation.contains("database") || observation.contains("query") {
                    format!(
                        "Database operation: {}. Need to ensure proper indexing \
                         and query optimization.",
                        observation
                    )
                } else if observation.contains("api") || observation.contains("endpoint") {
                    format!(
                        "API observation: {}. Should validate request/response \
                         schemas and error handling.",
                        observation
                    )
                } else {
                    format!(
                        "Backend progress: {}. Continuing with service implementation.",
                        observation
                    )
                }
            }
            "DevOps" => {
                if observation.contains("docker") || observation.contains("container") {
                    format!(
                        "Container observation: {}. Need to verify image optimization \
                         and security scanning.",
                        observation
                    )
                } else if observation.contains("deploy") || observation.contains("ci") {
                    format!(
                        "Deployment pipeline: {}. Should ensure rollback capabilities \
                         and monitoring integration.",
                        observation
                    )
                } else {
                    format!(
                        "Infrastructure progress: {}. Continuing with automation setup.",
                        observation
                    )
                }
            }
            _ => format!("General observation for {}: {}", agent_role, observation),
        };

        Ok(reflection)
    }

    /// Analyze confidence level based on observation and reflection
    fn analyze_confidence(&self, observation: &str, reflection: &str) -> f64 {
        let mut confidence: f64 = 0.8; // Base confidence

        // Decrease confidence for error indicators
        let error_keywords = ["error", "failed", "undefined", "null", "exception", "panic"];
        for keyword in &error_keywords {
            if observation.to_lowercase().contains(keyword) {
                confidence -= 0.15;
            }
        }

        // Increase confidence for success indicators
        let success_keywords = ["success", "complete", "ready", "passed", "working"];
        for keyword in &success_keywords {
            if observation.to_lowercase().contains(keyword) {
                confidence += 0.1;
            }
        }

        // Adjust based on reflection clarity
        if reflection.contains("need to") || reflection.contains("should") {
            confidence -= 0.05; // Uncertainty in next steps
        }

        // Consider history - repeated issues decrease confidence
        if self.thinking_history.len() > 3 {
            let recent_confidence_avg: f64 = self
                .thinking_history
                .iter()
                .rev()
                .take(3)
                .map(|s| s.confidence)
                .sum::<f64>()
                / 3.0;

            if recent_confidence_avg < 0.5 {
                confidence -= 0.1; // Trend of low confidence
            }
        }

        // Clamp to valid range
        confidence.clamp(0.0, 1.0)
    }

    /// Make a decision based on reflection and confidence
    async fn make_decision(
        &self,
        reflection: &str,
        confidence: f64,
        agent_role: &str,
    ) -> Result<Decision> {
        // Check if we've exceeded max steps
        if self.thinking_history.len() >= self.max_steps {
            return Ok(Decision::Complete {
                summary: format!(
                    "Reached maximum thinking steps ({}). Current state: {}",
                    self.max_steps, reflection
                ),
            });
        }

        // Low confidence decisions
        if confidence < self.confidence_threshold {
            if reflection.contains("need to") && reflection.contains("?") {
                // Need clarification
                let questions = self.extract_questions(reflection);
                return Ok(Decision::RequestContext { questions });
            } else if self.thinking_history.len() > 5 && confidence < 0.4 {
                // Persistent low confidence - consider pivoting
                return Ok(Decision::Pivot {
                    new_approach: format!(
                        "Alternative approach for {}: focusing on core functionality first",
                        agent_role
                    ),
                    reason: "Persistent low confidence suggests current approach needs adjustment"
                        .to_string(),
                });
            }
        }

        // Check for completion indicators
        if reflection.contains("complete") || reflection.contains("finished") {
            return Ok(Decision::Complete {
                summary: reflection.to_string(),
            });
        }

        // Check for critical issues
        if reflection.contains("critical") || reflection.contains("abort") {
            return Ok(Decision::Abort {
                reason: reflection.to_string(),
            });
        }

        // Check if refinement is needed
        if reflection.contains("optimize") || reflection.contains("improve") {
            return Ok(Decision::Refine {
                refinement: self.extract_refinement(reflection),
                reason: "Optimization opportunity identified".to_string(),
            });
        }

        // Default: continue with current approach
        Ok(Decision::Continue {
            reason: format!(
                "Confidence level {:.2} is acceptable, proceeding",
                confidence
            ),
        })
    }

    /// Extract questions from reflection
    fn extract_questions(&self, reflection: &str) -> Vec<String> {
        let mut questions = Vec::new();

        // Look for question marks
        for sentence in reflection.split('.') {
            if sentence.contains('?') {
                questions.push(sentence.trim().to_string());
            }
        }

        // If no explicit questions, generate based on uncertainty keywords
        if questions.is_empty() {
            if reflection.contains("unclear") {
                questions.push("What is the expected behavior for this component?".to_string());
            }
            if reflection.contains("missing") {
                questions.push("What additional context or dependencies are needed?".to_string());
            }
        }

        questions
    }

    /// Extract refinement suggestions from reflection
    fn extract_refinement(&self, reflection: &str) -> String {
        if reflection.contains("optimize") {
            "Apply performance optimizations to current implementation".to_string()
        } else if reflection.contains("refactor") {
            "Refactor code for better maintainability".to_string()
        } else if reflection.contains("improve") {
            "Enhance current implementation with better error handling".to_string()
        } else {
            "Refine implementation based on observations".to_string()
        }
    }

    /// Get a summary of the thinking process
    pub fn get_thinking_summary(&self) -> ThinkingSummary {
        let total_steps = self.thinking_history.len();
        let avg_confidence = if total_steps > 0 {
            self.thinking_history
                .iter()
                .map(|s| s.confidence)
                .sum::<f64>()
                / total_steps as f64
        } else {
            0.0
        };

        let decision_counts = self.count_decisions();

        ThinkingSummary {
            total_steps,
            avg_confidence,
            decision_counts,
            final_decision: self.thinking_history.last().map(|s| s.decision.clone()),
            key_insights: self.extract_key_insights(),
        }
    }

    /// Count decision types
    fn count_decisions(&self) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();

        for step in &self.thinking_history {
            let decision_type = match &step.decision {
                Decision::Continue { .. } => "Continue",
                Decision::Refine { .. } => "Refine",
                Decision::RequestContext { .. } => "RequestContext",
                Decision::Pivot { .. } => "Pivot",
                Decision::Complete { .. } => "Complete",
                Decision::Abort { .. } => "Abort",
            };
            *counts.entry(decision_type.to_string()).or_insert(0) += 1;
        }

        counts
    }

    /// Extract key insights from thinking history
    fn extract_key_insights(&self) -> Vec<String> {
        let mut insights = Vec::new();

        // Look for patterns in decisions
        let refine_count = self
            .thinking_history
            .iter()
            .filter(|s| matches!(s.decision, Decision::Refine { .. }))
            .count();

        if refine_count > 2 {
            insights.push(format!(
                "Multiple refinements ({}) suggest iterative improvement approach",
                refine_count
            ));
        }

        // Check confidence trends
        if self.thinking_history.len() > 3 {
            let early_confidence: f64 = self
                .thinking_history
                .iter()
                .take(3)
                .map(|s| s.confidence)
                .sum::<f64>()
                / 3.0;

            let late_confidence: f64 = self
                .thinking_history
                .iter()
                .rev()
                .take(3)
                .map(|s| s.confidence)
                .sum::<f64>()
                / 3.0;

            if late_confidence > early_confidence + 0.2 {
                insights.push("Confidence improved significantly during execution".to_string());
            } else if early_confidence > late_confidence + 0.2 {
                insights
                    .push("Confidence decreased during execution - may need review".to_string());
            }
        }

        insights
    }

    /// Reset the thinking engine for a new task
    pub fn reset(&mut self) {
        self.thinking_history.clear();
        self.current_context.clear();
    }
}

/// Summary of the thinking process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingSummary {
    pub total_steps: usize,
    pub avg_confidence: f64,
    pub decision_counts: std::collections::HashMap<String, usize>,
    pub final_decision: Option<Decision>,
    pub key_insights: Vec<String>,
}

/// Extension trait for integrating thinking into task execution
#[async_trait::async_trait]
pub trait WithInterleavedThinking {
    /// Execute with thinking steps
    async fn execute_with_thinking(
        &mut self,
        task: &str,
        thinking_engine: &mut InterleavedThinkingEngine,
    ) -> Result<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_thinking_process() {
        let mut engine = InterleavedThinkingEngine::new();

        // Test Frontend agent thinking
        let step1 = engine
            .process_observation("Component rendering successfully", "Frontend")
            .await
            .unwrap();

        assert!(matches!(step1.decision, Decision::Continue { .. }));
        assert!(step1.confidence > 0.7);

        // Test error observation
        let step2 = engine
            .process_observation("Error: useState hook called conditionally", "Frontend")
            .await
            .unwrap();

        assert!(step2.confidence < 0.7);
    }

    #[tokio::test]
    async fn test_confidence_analysis() {
        let engine = InterleavedThinkingEngine::new();

        // Test various observations
        let high_conf = engine.analyze_confidence(
            "API endpoint created successfully",
            "Backend progress: API endpoint created successfully. Continuing with service implementation."
        );
        assert!(high_conf > 0.7);

        let low_conf = engine.analyze_confidence(
            "Database connection failed with error",
            "Database operation: Database connection failed with error. Need to ensure proper indexing and query optimization."
        );
        assert!(low_conf < 0.6);
    }

    #[tokio::test]
    async fn test_decision_making() {
        let mut engine = InterleavedThinkingEngine::new();

        // Test multiple low confidence observations
        for i in 0..4 {
            let _ = engine
                .process_observation(&format!("Error in iteration {}", i), "Backend")
                .await
                .unwrap();
        }

        // Should have low confidence after persistent issues
        let step = engine
            .process_observation("Another error occurred", "Backend")
            .await
            .unwrap();

        // Confidence should be reduced after multiple errors
        assert!(step.confidence < 0.7);
    }
}
