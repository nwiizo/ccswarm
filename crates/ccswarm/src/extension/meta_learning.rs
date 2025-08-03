//! Meta-learning capabilities for agents to learn how to learn better

use super::*;
use std::collections::HashMap;

/// Meta-learning system that helps agents learn from their extension experiences
#[derive(Debug)]
pub struct MetaLearningSystem {
    /// Learning patterns database
    patterns: Arc<RwLock<LearningPatternDB>>,
    /// Success/failure analyzer
    analyzer: ExperienceAnalyzer,
    /// Pattern matcher
    matcher: PatternMatcher,
    /// Evolution tracker
    #[allow(dead_code)]
    evolution_tracker: EvolutionTracker,
}

/// Database of learning patterns
#[derive(Debug, Default)]
pub struct LearningPatternDB {
    /// Successful extension patterns
    success_patterns: Vec<SuccessPattern>,
    /// Failure patterns to avoid
    failure_patterns: Vec<FailurePattern>,
    /// Meta-patterns (patterns about patterns)
    #[allow(dead_code)]
    meta_patterns: Vec<MetaPattern>,
}

/// A successful extension pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessPattern {
    pub id: Uuid,
    pub pattern_type: PatternType,
    pub context: ExtensionContext,
    pub approach: LearningApproach,
    pub key_factors: Vec<String>,
    pub success_rate: f64,
    pub average_duration: String,
    pub replication_count: u32,
}

/// A failure pattern to avoid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    pub id: Uuid,
    pub pattern_type: PatternType,
    pub context: ExtensionContext,
    pub failure_indicators: Vec<String>,
    pub root_causes: Vec<String>,
    pub avoidance_strategies: Vec<String>,
    pub occurrence_count: u32,
}

/// Meta-pattern (pattern about learning patterns)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaPattern {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub pattern_evolution: Vec<PatternEvolution>,
    pub effectiveness_trend: Trend,
    pub applicability_conditions: Vec<String>,
}

/// Types of patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    Sequential,  // Step-by-step learning
    Parallel,    // Multiple simultaneous learnings
    Iterative,   // Repeated refinement
    Exploratory, // Trial and error
    Analytical,  // Deep analysis first
    Synthetic,   // Combining existing knowledge
}

/// Context in which extension occurs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionContext {
    pub agent_role: AgentRole,
    pub domain: String,
    pub complexity_level: ComplexityLevel,
    pub prerequisites_met: bool,
    pub resource_availability: ResourceAvailability,
    pub time_pressure: TimePressure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Trivial,
    Simple,
    Moderate,
    Complex,
    Extreme,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceAvailability {
    Abundant,
    Sufficient,
    Limited,
    Scarce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimePressure {
    Relaxed,
    Normal,
    Urgent,
    Critical,
}

/// Learning approach used
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningApproach {
    pub strategy: LearningStrategy,
    pub resource_utilization: Vec<ResourceUtilization>,
    pub collaboration_level: CollaborationLevel,
    pub iteration_count: u32,
    pub adaptation_rate: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearningStrategy {
    TopDown,       // Start with theory, then practice
    BottomUp,      // Start with examples, derive theory
    MiddleOut,     // Start with core concepts, expand
    Holistic,      // Consider entire system
    Incremental,   // Small steps
    Revolutionary, // Major leap
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub resource_type: String,
    pub usage_intensity: f64,
    pub effectiveness: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollaborationLevel {
    Solo,
    Mentored,
    PeerAssisted,
    TeamBased,
    CommunityDriven,
}

/// Pattern evolution over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternEvolution {
    pub version: u32,
    pub timestamp: DateTime<Utc>,
    pub changes: Vec<String>,
    pub improvement_delta: f64,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trend {
    Improving,
    Stable,
    Declining,
    Volatile,
}

/// Experience analyzer
#[derive(Debug)]
pub struct ExperienceAnalyzer {
    analysis_strategies: Vec<Box<dyn AnalysisStrategy>>,
}

impl ExperienceAnalyzer {
    /// Analyze an extension experience
    pub fn analyze(&self, experience: &ExtensionExperience) -> AnalysisResult {
        let mut combined_insights = Vec::new();
        let mut combined_patterns = Vec::new();
        let mut overall_success_score = 0.0;

        for strategy in &self.analysis_strategies {
            let result = strategy.analyze(experience);
            combined_insights.extend(result.key_insights);
            combined_patterns.extend(result.patterns);
            overall_success_score += result.success_score;
        }

        AnalysisResult {
            key_insights: combined_insights,
            patterns: combined_patterns,
            success_score: overall_success_score / self.analysis_strategies.len() as f64,
            recommendations: vec![],
        }
    }
}

/// Analysis strategy trait
pub trait AnalysisStrategy: Send + Sync + std::fmt::Debug {
    fn analyze(&self, experience: &ExtensionExperience) -> AnalysisResult;
    fn name(&self) -> &str;
}

/// Extension experience to analyze
#[derive(Debug, Clone)]
pub struct ExtensionExperience {
    pub extension_id: Uuid,
    pub agent_id: String,
    pub extension_type: ExtensionType,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub outcome: ExtensionOutcome,
    pub metrics: HashMap<String, f64>,
    pub events: Vec<ExtensionEvent>,
    pub resources_used: Vec<String>,
}

/// Events during extension
#[derive(Debug, Clone)]
pub struct ExtensionEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: EventType,
    pub description: String,
    pub impact: Impact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventType {
    Breakthrough,
    Obstacle,
    Pivot,
    Milestone,
    Failure,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Impact {
    Positive,
    Neutral,
    Negative,
}

/// Analysis result
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub key_insights: Vec<String>,
    pub patterns: Vec<ExtensionPattern>,
    pub success_score: f64,
    pub recommendations: Vec<String>,
}

/// Pattern matcher
#[derive(Debug)]
pub struct PatternMatcher {
    matching_threshold: f64,
}

impl PatternMatcher {
    pub fn new(matching_threshold: f64) -> Self {
        Self { matching_threshold }
    }

    /// Match current situation with known patterns
    pub async fn match_patterns(
        &self,
        context: &ExtensionContext,
        patterns: &LearningPatternDB,
    ) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        // Match against success patterns
        for pattern in &patterns.success_patterns {
            let similarity = self.calculate_similarity(&pattern.context, context);
            if similarity >= self.matching_threshold {
                matches.push(PatternMatch {
                    pattern_id: pattern.id,
                    pattern_type: MatchType::Success,
                    similarity_score: similarity,
                    applicable_strategies: self.extract_strategies(pattern),
                });
            }
        }

        // Match against failure patterns (to avoid)
        for pattern in &patterns.failure_patterns {
            let similarity = self.calculate_similarity(&pattern.context, context);
            if similarity >= self.matching_threshold {
                matches.push(PatternMatch {
                    pattern_id: pattern.id,
                    pattern_type: MatchType::Failure,
                    similarity_score: similarity,
                    applicable_strategies: vec![],
                });
            }
        }

        // Sort by similarity
        matches.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        matches
    }

    fn calculate_similarity(
        &self,
        pattern_context: &ExtensionContext,
        current_context: &ExtensionContext,
    ) -> f64 {
        let mut score = 0.0;
        let mut factors = 0.0;

        // Role match
        if pattern_context.agent_role == current_context.agent_role {
            score += 0.3;
        }
        factors += 0.3;

        // Domain similarity
        if pattern_context.domain == current_context.domain {
            score += 0.2;
        }
        factors += 0.2;

        // Complexity match
        if pattern_context.complexity_level == current_context.complexity_level {
            score += 0.2;
        }
        factors += 0.2;

        // Resource availability
        if pattern_context.resource_availability == current_context.resource_availability {
            score += 0.15;
        }
        factors += 0.15;

        // Time pressure
        if pattern_context.time_pressure == current_context.time_pressure {
            score += 0.15;
        }
        factors += 0.15;

        score / factors
    }

    fn extract_strategies(&self, pattern: &SuccessPattern) -> Vec<String> {
        pattern.key_factors.clone()
    }
}

/// Pattern match result
#[derive(Debug, Clone)]
pub struct PatternMatch {
    pub pattern_id: Uuid,
    pub pattern_type: MatchType,
    pub similarity_score: f64,
    pub applicable_strategies: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    Success,
    Failure,
    Meta,
}

/// Evolution tracker
#[derive(Debug)]
pub struct EvolutionTracker {
    #[allow(dead_code)]
    history: Arc<RwLock<Vec<EvolutionRecord>>>,
}

/// Evolution record
#[derive(Debug, Clone)]
pub struct EvolutionRecord {
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub capability_before: Vec<String>,
    pub capability_after: Vec<String>,
    pub learning_efficiency: f64,
    pub adaptation_speed: f64,
}

impl Default for MetaLearningSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaLearningSystem {
    pub fn new() -> Self {
        Self {
            patterns: Arc::new(RwLock::new(LearningPatternDB::default())),
            analyzer: ExperienceAnalyzer {
                analysis_strategies: vec![],
            },
            matcher: PatternMatcher::new(0.7),
            evolution_tracker: EvolutionTracker {
                history: Arc::new(RwLock::new(Vec::new())),
            },
        }
    }

    /// Learn from an extension experience
    pub async fn learn_from_experience(
        &self,
        experience: ExtensionExperience,
    ) -> Result<LearningOutcome> {
        // Analyze the experience
        let analysis = self.analyzer.analyze(&experience);

        // Update patterns based on outcome
        match &experience.outcome {
            ExtensionOutcome::Success { improvements } => {
                self.record_success_pattern(&experience, &analysis, improvements)
                    .await?;
            }
            ExtensionOutcome::Failure { reasons } => {
                self.record_failure_pattern(&experience, &analysis, reasons)
                    .await?;
            }
            ExtensionOutcome::PartialSuccess {
                achievements,
                issues,
            } => {
                self.record_mixed_pattern(&experience, &analysis, achievements, issues)
                    .await?;
            }
        }

        // Extract meta-patterns
        self.extract_meta_patterns().await?;

        // Track evolution
        self.track_evolution(&experience).await?;

        Ok(LearningOutcome {
            new_patterns_discovered: analysis.patterns.len(),
            insights_gained: analysis.key_insights,
            recommendations: analysis.recommendations,
            evolution_metrics: self.calculate_evolution_metrics(&experience).await?,
        })
    }

    /// Get learning recommendations for a new extension
    pub async fn get_recommendations(
        &self,
        context: &ExtensionContext,
    ) -> Result<LearningRecommendations> {
        let patterns = self.patterns.read().await;

        // Find matching patterns
        let matches = self.matcher.match_patterns(context, &patterns).await;

        // Generate recommendations based on matches
        let mut recommendations = LearningRecommendations {
            recommended_approach: None,
            patterns_to_follow: vec![],
            patterns_to_avoid: vec![],
            estimated_success_rate: 0.0,
            suggested_resources: vec![],
            risk_mitigation: vec![],
        };

        for match_result in matches {
            match match_result.pattern_type {
                MatchType::Success => {
                    recommendations
                        .patterns_to_follow
                        .push(match_result.pattern_id);
                    if recommendations.recommended_approach.is_none() {
                        // Use the highest matching success pattern's approach
                        if let Some(pattern) = patterns
                            .success_patterns
                            .iter()
                            .find(|p| p.id == match_result.pattern_id)
                        {
                            recommendations.recommended_approach = Some(pattern.approach.clone());
                            recommendations.estimated_success_rate =
                                pattern.success_rate * match_result.similarity_score;
                        }
                    }
                }
                MatchType::Failure => {
                    recommendations
                        .patterns_to_avoid
                        .push(match_result.pattern_id);
                    if let Some(pattern) = patterns
                        .failure_patterns
                        .iter()
                        .find(|p| p.id == match_result.pattern_id)
                    {
                        recommendations
                            .risk_mitigation
                            .extend(pattern.avoidance_strategies.clone());
                    }
                }
                MatchType::Meta => {
                    // Apply meta-pattern insights
                }
            }
        }

        Ok(recommendations)
    }

    async fn record_success_pattern(
        &self,
        _experience: &ExtensionExperience,
        _analysis: &AnalysisResult,
        _improvements: &[String],
    ) -> Result<()> {
        // Record successful pattern
        let _patterns = self.patterns.write().await;

        // Check if similar pattern exists
        // If yes, update it; if no, create new

        Ok(())
    }

    async fn record_failure_pattern(
        &self,
        _experience: &ExtensionExperience,
        _analysis: &AnalysisResult,
        _reasons: &[String],
    ) -> Result<()> {
        // Record failure pattern
        Ok(())
    }

    async fn record_mixed_pattern(
        &self,
        _experience: &ExtensionExperience,
        _analysis: &AnalysisResult,
        _achievements: &[String],
        _issues: &[String],
    ) -> Result<()> {
        // Record mixed outcome pattern
        Ok(())
    }

    async fn extract_meta_patterns(&self) -> Result<()> {
        // Analyze patterns to find meta-patterns
        Ok(())
    }

    async fn track_evolution(&self, _experience: &ExtensionExperience) -> Result<()> {
        // Track capability evolution
        Ok(())
    }

    async fn calculate_evolution_metrics(
        &self,
        _experience: &ExtensionExperience,
    ) -> Result<EvolutionMetrics> {
        Ok(EvolutionMetrics {
            learning_velocity: 0.0,
            adaptation_rate: 0.0,
            pattern_recognition_improvement: 0.0,
            failure_avoidance_rate: 0.0,
        })
    }
}

/// Learning outcome from meta-learning
#[derive(Debug, Clone)]
pub struct LearningOutcome {
    pub new_patterns_discovered: usize,
    pub insights_gained: Vec<String>,
    pub recommendations: Vec<String>,
    pub evolution_metrics: EvolutionMetrics,
}

/// Learning recommendations
#[derive(Debug, Clone)]
pub struct LearningRecommendations {
    pub recommended_approach: Option<LearningApproach>,
    pub patterns_to_follow: Vec<Uuid>,
    pub patterns_to_avoid: Vec<Uuid>,
    pub estimated_success_rate: f64,
    pub suggested_resources: Vec<String>,
    pub risk_mitigation: Vec<String>,
}

/// Evolution metrics
#[derive(Debug, Clone)]
pub struct EvolutionMetrics {
    pub learning_velocity: f64,
    pub adaptation_rate: f64,
    pub pattern_recognition_improvement: f64,
    pub failure_avoidance_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_matcher() {
        let matcher = PatternMatcher::new(0.7);

        let pattern_context = ExtensionContext {
            agent_role: AgentRole::Frontend {
                technologies: vec!["React".to_string()],
                responsibilities: vec!["UI development".to_string()],
                boundaries: vec!["Frontend only".to_string()],
            },
            domain: "web".to_string(),
            complexity_level: ComplexityLevel::Moderate,
            prerequisites_met: true,
            resource_availability: ResourceAvailability::Sufficient,
            time_pressure: TimePressure::Normal,
        };

        let current_context = ExtensionContext {
            agent_role: AgentRole::Frontend {
                technologies: vec!["React".to_string()],
                responsibilities: vec!["UI development".to_string()],
                boundaries: vec!["Frontend only".to_string()],
            },
            domain: "web".to_string(),
            complexity_level: ComplexityLevel::Moderate,
            prerequisites_met: true,
            resource_availability: ResourceAvailability::Sufficient,
            time_pressure: TimePressure::Normal,
        };

        let similarity = matcher.calculate_similarity(&pattern_context, &current_context);
        assert_eq!(similarity, 1.0);
    }
}
