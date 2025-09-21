//! Semantic task analysis engine
//!
//! Analyzes tasks to determine relevant code elements and optimal approach

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::analyzer::{ImpactAnalysis, ImpactSeverity, SemanticAnalyzer, Symbol};
use super::memory::ProjectMemory;
use super::symbol_index::SymbolIndex;
use super::SemanticResult;

/// Task context with semantic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    /// Original task
    pub task: Task,

    /// Related symbols identified
    pub related_symbols: Vec<Symbol>,

    /// Impact analysis
    pub impact: ImpactAnalysis,

    /// Recommended approach
    pub recommended_approach: TaskApproach,
}

/// Task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: TaskPriority,
    pub tags: Vec<String>,
    pub assigned_agent: Option<String>,
}

/// Task priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Recommended approach for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskApproach {
    /// Suggested steps
    pub steps: Vec<TaskStep>,

    /// Estimated complexity
    pub complexity: ComplexityLevel,

    /// Recommended agent
    pub recommended_agent: String,

    /// Required capabilities
    pub required_capabilities: Vec<String>,

    /// Potential risks
    pub risks: Vec<Risk>,
}

/// Task step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStep {
    pub order: usize,
    pub description: String,
    pub action_type: ActionType,
    pub target_symbols: Vec<String>,
}

/// Type of action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    Analyze,
    Modify,
    Create,
    Delete,
    Refactor,
    Test,
    Document,
}

/// Complexity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComplexityLevel {
    Trivial,
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Risk definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub description: String,
    pub severity: RiskSeverity,
    pub mitigation: String,
}

/// Risk severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
}

/// Enriched task with semantic context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedTask {
    pub original: Task,
    pub symbols: Vec<Symbol>,
    pub context: super::memory::ProjectContext,
    pub suggested_approach: TaskApproach,
}

/// Semantic task analyzer
pub struct SemanticTaskAnalyzer {
    analyzer: Arc<SemanticAnalyzer>,
    index: Arc<SymbolIndex>,
    memory: Arc<ProjectMemory>,
}

impl SemanticTaskAnalyzer {
    /// Create a new task analyzer
    pub fn new(
        analyzer: Arc<SemanticAnalyzer>,
        index: Arc<SymbolIndex>,
        memory: Arc<ProjectMemory>,
    ) -> Self {
        Self {
            analyzer,
            index,
            memory,
        }
    }

    /// Analyze a task to extract semantic context
    pub async fn analyze_task(&self, task: &Task) -> SemanticResult<TaskContext> {
        // Extract keywords from task description
        let keywords = self.extract_keywords(&task.description);

        // Find related symbols
        let symbols = self.find_related_symbols(&keywords).await?;

        // Analyze impact
        let impact_analysis = self.analyze_impact(&symbols).await?;

        // Generate recommended approach
        let approach = self
            .generate_approach(task, &symbols, &impact_analysis)
            .await?;

        Ok(TaskContext {
            task: task.clone(),
            related_symbols: symbols,
            impact: impact_analysis,
            recommended_approach: approach,
        })
    }

    /// Extract keywords from task description
    fn extract_keywords(&self, description: &str) -> Vec<String> {
        // Simple keyword extraction
        description
            .split_whitespace()
            .filter(|word| word.len() > 3)
            .map(|word| word.to_lowercase())
            .filter(|word| !STOP_WORDS.contains(&word.as_str()))
            .collect()
    }

    /// Find symbols related to keywords
    async fn find_related_symbols(&self, keywords: &[String]) -> SemanticResult<Vec<Symbol>> {
        let mut related_symbols = Vec::new();

        for keyword in keywords {
            // Search by name
            let by_name = self.index.find_by_name(keyword).await?;
            related_symbols.extend(by_name);

            // Search in all symbols
            let all_symbols = self.index.get_all_symbols().await?;
            for symbol in all_symbols {
                if symbol.path.contains(keyword)
                    && !related_symbols.iter().any(|s| s.path == symbol.path)
                {
                    related_symbols.push(symbol);
                }
            }
        }

        // Deduplicate
        related_symbols.sort_by(|a, b| a.path.cmp(&b.path));
        related_symbols.dedup_by(|a, b| a.path == b.path);

        Ok(related_symbols)
    }

    /// Analyze impact of working with these symbols
    async fn analyze_impact(&self, symbols: &[Symbol]) -> SemanticResult<ImpactAnalysis> {
        // Simplified impact analysis
        let severity = if symbols.is_empty() {
            ImpactSeverity::Low
        } else if symbols.len() < 3 {
            ImpactSeverity::Medium
        } else {
            ImpactSeverity::High
        };

        let suggested_actions = vec![
            "Review affected symbols".to_string(),
            "Run tests after changes".to_string(),
            "Update documentation".to_string(),
        ];

        Ok(ImpactAnalysis {
            change: super::analyzer::SymbolChange {
                symbol: symbols.first().cloned().unwrap_or_else(|| Symbol {
                    name: "unknown".to_string(),
                    path: "unknown".to_string(),
                    kind: super::analyzer::SymbolKind::Other("unknown".to_string()),
                    file_path: String::new(),
                    line: 0,
                    body: None,
                    references: Vec::new(),
                    metadata: HashMap::new(),
                }),
                change_type: super::analyzer::ChangeType::Modified,
                old_value: None,
                new_value: None,
            },
            affected_symbols: symbols.to_vec(),
            severity,
            suggested_actions,
        })
    }

    /// Generate recommended approach for the task
    pub async fn generate_approach(
        &self,
        task: &Task,
        symbols: &[Symbol],
        impact: &ImpactAnalysis,
    ) -> SemanticResult<TaskApproach> {
        let mut steps = Vec::new();

        // Step 1: Analyze existing code
        if !symbols.is_empty() {
            steps.push(TaskStep {
                order: 1,
                description: "Analyze existing implementations".to_string(),
                action_type: ActionType::Analyze,
                target_symbols: symbols.iter().map(|s| s.path.clone()).collect(),
            });
        }

        // Step 2: Determine action based on task description
        let action_type = self.determine_action_type(&task.description);
        steps.push(TaskStep {
            order: 2,
            description: format!(
                "Perform {} operation",
                format!("{:?}", action_type).to_lowercase()
            ),
            action_type: action_type.clone(),
            target_symbols: symbols.iter().map(|s| s.path.clone()).take(3).collect(),
        });

        // Step 3: Test changes
        steps.push(TaskStep {
            order: 3,
            description: "Test the changes".to_string(),
            action_type: ActionType::Test,
            target_symbols: Vec::new(),
        });

        // Step 4: Document if needed
        if matches!(impact.severity, ImpactSeverity::High) {
            steps.push(TaskStep {
                order: 4,
                description: "Update documentation".to_string(),
                action_type: ActionType::Document,
                target_symbols: Vec::new(),
            });
        }

        // Determine complexity
        let complexity = self.calculate_complexity(symbols.len(), &impact.severity);

        // Determine recommended agent
        let recommended_agent = self.recommend_agent(task, &action_type);

        // Determine required capabilities
        let required_capabilities = self.determine_capabilities(&action_type, task);

        // Identify risks
        let risks = self.identify_risks(impact);

        Ok(TaskApproach {
            steps,
            complexity,
            recommended_agent,
            required_capabilities,
            risks,
        })
    }

    /// Determine action type from task description
    fn determine_action_type(&self, description: &str) -> ActionType {
        let lower = description.to_lowercase();

        if lower.contains("create") || lower.contains("add") || lower.contains("implement") {
            ActionType::Create
        } else if lower.contains("modify") || lower.contains("update") || lower.contains("change") {
            ActionType::Modify
        } else if lower.contains("delete") || lower.contains("remove") {
            ActionType::Delete
        } else if lower.contains("refactor") || lower.contains("improve") {
            ActionType::Refactor
        } else if lower.contains("test") {
            ActionType::Test
        } else if lower.contains("document") {
            ActionType::Document
        } else {
            ActionType::Analyze
        }
    }

    /// Calculate task complexity
    fn calculate_complexity(
        &self,
        symbol_count: usize,
        severity: &ImpactSeverity,
    ) -> ComplexityLevel {
        match (symbol_count, severity) {
            (0, _) => ComplexityLevel::Simple,
            (1..=2, ImpactSeverity::Low) => ComplexityLevel::Simple,
            (1..=2, ImpactSeverity::Medium) => ComplexityLevel::Moderate,
            (1..=2, ImpactSeverity::High) => ComplexityLevel::Complex,
            (3..=5, ImpactSeverity::Low) => ComplexityLevel::Moderate,
            (3..=5, ImpactSeverity::Medium) => ComplexityLevel::Complex,
            (3..=5, ImpactSeverity::High) => ComplexityLevel::VeryComplex,
            _ => ComplexityLevel::VeryComplex,
        }
    }

    /// Recommend agent for the task
    fn recommend_agent(&self, task: &Task, action_type: &ActionType) -> String {
        // Check tags first
        for tag in &task.tags {
            let lower = tag.to_lowercase();
            if lower.contains("frontend") || lower.contains("ui") {
                return "frontend-specialist".to_string();
            }
            if lower.contains("backend") || lower.contains("api") {
                return "backend-specialist".to_string();
            }
            if lower.contains("devops") || lower.contains("deploy") {
                return "devops-specialist".to_string();
            }
            if lower.contains("test") || lower.contains("qa") {
                return "qa-specialist".to_string();
            }
        }

        // Default based on action type
        match action_type {
            ActionType::Test => "qa-specialist".to_string(),
            ActionType::Document => "documentation-specialist".to_string(),
            _ => "general-specialist".to_string(),
        }
    }

    /// Determine required capabilities
    fn determine_capabilities(&self, action_type: &ActionType, task: &Task) -> Vec<String> {
        let mut capabilities = Vec::new();

        match action_type {
            ActionType::Create | ActionType::Modify => {
                capabilities.push("Code generation".to_string());
                capabilities.push("Pattern recognition".to_string());
            }
            ActionType::Refactor => {
                capabilities.push("Code analysis".to_string());
                capabilities.push("Pattern extraction".to_string());
            }
            ActionType::Test => {
                capabilities.push("Test generation".to_string());
                capabilities.push("Coverage analysis".to_string());
            }
            ActionType::Document => {
                capabilities.push("Documentation generation".to_string());
            }
            _ => {}
        }

        // Add task-specific capabilities
        for tag in &task.tags {
            capabilities.push(format!("{} expertise", tag));
        }

        capabilities
    }

    /// Identify risks
    fn identify_risks(&self, impact: &ImpactAnalysis) -> Vec<Risk> {
        let mut risks = Vec::new();

        match impact.severity {
            ImpactSeverity::High => {
                risks.push(Risk {
                    description: "High impact change affecting multiple components".to_string(),
                    severity: RiskSeverity::High,
                    mitigation: "Thorough testing and staged rollout recommended".to_string(),
                });
            }
            ImpactSeverity::Medium => {
                risks.push(Risk {
                    description: "Moderate impact on existing functionality".to_string(),
                    severity: RiskSeverity::Medium,
                    mitigation: "Test affected components before deployment".to_string(),
                });
            }
            _ => {}
        }

        if impact.affected_symbols.len() > 5 {
            risks.push(Risk {
                description: "Many symbols affected by this change".to_string(),
                severity: RiskSeverity::Medium,
                mitigation: "Consider breaking into smaller changes".to_string(),
            });
        }

        risks
    }

    /// Create enriched task with full context
    pub async fn create_enriched_task(&self, task: &Task) -> SemanticResult<EnrichedTask> {
        let context = self.analyze_task(task).await?;
        let project_context = self
            .memory
            .retrieve_context(&context.related_symbols)
            .await?;

        Ok(EnrichedTask {
            original: task.clone(),
            symbols: context.related_symbols,
            context: project_context,
            suggested_approach: context.recommended_approach,
        })
    }
}

// Common stop words to filter out
const STOP_WORDS: &[&str] = &[
    "the", "and", "for", "with", "from", "into", "this", "that", "these", "those", "will", "would",
    "could", "should", "must", "can", "may", "might",
];
