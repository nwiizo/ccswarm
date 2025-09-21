use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::process::Command;
use tracing::{debug, info, warn};

use crate::agent::{Task, TaskResult};
use crate::identity::AgentRole;

/// Quality evaluation rubric based on Anthropic's research system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRubric {
    /// Dimensions to evaluate with their weights
    pub dimensions: HashMap<String, f64>,
    /// Minimum acceptable score for each dimension
    pub thresholds: HashMap<String, f64>,
    /// Role-specific adjustments
    pub role_weights: HashMap<String, HashMap<String, f64>>,
}

impl Default for QualityRubric {
    fn default() -> Self {
        let mut dimensions = HashMap::new();
        dimensions.insert("correctness".to_string(), 0.3);
        dimensions.insert("maintainability".to_string(), 0.2);
        dimensions.insert("test_quality".to_string(), 0.2);
        dimensions.insert("security".to_string(), 0.15);
        dimensions.insert("performance".to_string(), 0.1);
        dimensions.insert("documentation".to_string(), 0.05);

        let mut thresholds = HashMap::new();
        thresholds.insert("correctness".to_string(), 0.9);
        thresholds.insert("maintainability".to_string(), 0.8);
        thresholds.insert("test_quality".to_string(), 0.85);
        thresholds.insert("security".to_string(), 0.9);
        thresholds.insert("performance".to_string(), 0.7);
        thresholds.insert("documentation".to_string(), 0.7);

        Self {
            dimensions,
            thresholds,
            role_weights: Self::default_role_weights(),
        }
    }
}

impl QualityRubric {
    /// Create role-specific weight adjustments
    fn default_role_weights() -> HashMap<String, HashMap<String, f64>> {
        let mut role_weights = HashMap::new();

        // Frontend weights
        let mut frontend_weights = HashMap::new();
        frontend_weights.insert("correctness".to_string(), 0.25);
        frontend_weights.insert("maintainability".to_string(), 0.2);
        frontend_weights.insert("test_quality".to_string(), 0.15);
        frontend_weights.insert("security".to_string(), 0.1);
        frontend_weights.insert("performance".to_string(), 0.15);
        frontend_weights.insert("documentation".to_string(), 0.05);
        frontend_weights.insert("accessibility".to_string(), 0.1);
        role_weights.insert("Frontend".to_string(), frontend_weights);

        // Backend weights
        let mut backend_weights = HashMap::new();
        backend_weights.insert("correctness".to_string(), 0.3);
        backend_weights.insert("maintainability".to_string(), 0.15);
        backend_weights.insert("test_quality".to_string(), 0.25);
        backend_weights.insert("security".to_string(), 0.2);
        backend_weights.insert("performance".to_string(), 0.1);
        backend_weights.insert("documentation".to_string(), 0.0);
        role_weights.insert("Backend".to_string(), backend_weights);

        // DevOps weights
        let mut devops_weights = HashMap::new();
        devops_weights.insert("correctness".to_string(), 0.25);
        devops_weights.insert("maintainability".to_string(), 0.15);
        devops_weights.insert("test_quality".to_string(), 0.1);
        devops_weights.insert("security".to_string(), 0.3);
        devops_weights.insert("performance".to_string(), 0.15);
        devops_weights.insert("documentation".to_string(), 0.05);
        role_weights.insert("DevOps".to_string(), devops_weights);

        role_weights
    }

    /// Get weights for a specific role
    pub fn get_role_weights(&self, role: &str) -> &HashMap<String, f64> {
        self.role_weights.get(role).unwrap_or(&self.dimensions)
    }
}

/// Result of quality evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityEvaluation {
    /// Overall quality score (0.0 to 1.0)
    pub overall_score: f64,
    /// Individual dimension scores
    pub dimension_scores: HashMap<String, f64>,
    /// Detected issues with severity
    pub issues: Vec<QualityIssue>,
    /// Detailed feedback
    pub feedback: String,
    /// Whether the code passes quality standards
    pub passes_standards: bool,
    /// Confidence in the evaluation (0.0 to 1.0)
    pub confidence: f64,
    /// Timestamp of evaluation
    pub evaluated_at: DateTime<Utc>,
    /// Evaluation metadata
    pub metadata: EvaluationMetadata,
}

/// Quality issue detected during evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    /// Severity level
    pub severity: IssueSeverity,
    /// Category of issue
    pub category: IssueCategory,
    /// Description of the issue
    pub description: String,
    /// Suggested fix
    pub suggested_fix: String,
    /// Affected files or components
    pub affected_areas: Vec<String>,
    /// Estimated effort to fix (in minutes)
    pub fix_effort: u32,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueSeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl PartialOrd for IssueSeverity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IssueSeverity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (self, other) {
            (IssueSeverity::Critical, IssueSeverity::Critical) => Ordering::Equal,
            (IssueSeverity::Critical, _) => Ordering::Greater,
            (_, IssueSeverity::Critical) => Ordering::Less,
            (IssueSeverity::High, IssueSeverity::High) => Ordering::Equal,
            (IssueSeverity::High, _) => Ordering::Greater,
            (_, IssueSeverity::High) => Ordering::Less,
            (IssueSeverity::Medium, IssueSeverity::Medium) => Ordering::Equal,
            (IssueSeverity::Medium, IssueSeverity::Low) => Ordering::Greater,
            (IssueSeverity::Low, IssueSeverity::Medium) => Ordering::Less,
            (IssueSeverity::Low, IssueSeverity::Low) => Ordering::Equal,
        }
    }
}

/// Categories of quality issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    Security,
    Performance,
    TestCoverage,
    CodeComplexity,
    Documentation,
    ErrorHandling,
    Architecture,
    BestPractices,
    Accessibility,
    TypeSafety,
}

/// Metadata about the evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationMetadata {
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Lines of code evaluated
    pub lines_of_code: usize,
    /// Evaluation duration in milliseconds
    pub duration_ms: u64,
    /// Tools used for evaluation
    pub tools_used: Vec<String>,
    /// Agent role being evaluated
    pub agent_role: String,
}

/// LLM-based quality judge
pub struct LLMQualityJudge {
    /// Quality rubric to use
    rubric: QualityRubric,
    /// Cache of recent evaluations
    evaluation_cache: HashMap<String, QualityEvaluation>,
    /// Whether to use Claude for evaluation
    pub use_claude: bool,
}

impl Default for LLMQualityJudge {
    fn default() -> Self {
        Self {
            rubric: QualityRubric::default(),
            evaluation_cache: HashMap::new(),
            use_claude: true,
        }
    }
}

impl LLMQualityJudge {
    /// Create a new quality judge
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with custom rubric
    pub fn with_rubric(rubric: QualityRubric) -> Self {
        Self {
            rubric,
            evaluation_cache: HashMap::new(),
            use_claude: true,
        }
    }

    /// Evaluate task output quality
    pub async fn evaluate_task(
        &mut self,
        task: &Task,
        result: &TaskResult,
        agent_role: &AgentRole,
        workspace_path: &str,
    ) -> Result<QualityEvaluation> {
        let start_time = std::time::Instant::now();

        // Check cache first
        let cache_key = format!("{}-{}", task.id, result.output);
        if let Some(cached) = self.evaluation_cache.get(&cache_key) {
            debug!("Using cached evaluation for task {}", task.id);
            return Ok(cached.clone());
        }

        info!(
            "Evaluating quality for task {} by {} agent",
            task.id,
            agent_role.name()
        );

        // Extract code or content from result
        let content = self.extract_content_from_result(result)?;

        // Perform LLM-based evaluation
        let evaluation = if self.use_claude {
            self.evaluate_with_claude(&content, task, agent_role, workspace_path)
                .await?
        } else {
            self.evaluate_with_heuristics(&content, task, agent_role)?
        };

        // Cache the evaluation
        self.evaluation_cache.insert(cache_key, evaluation.clone());

        let duration_ms = start_time.elapsed().as_millis() as u64;
        info!(
            "Quality evaluation completed in {}ms: score={:.2}, passed={}",
            duration_ms, evaluation.overall_score, evaluation.passes_standards
        );

        Ok(evaluation)
    }

    /// Extract content from task result
    fn extract_content_from_result(&self, result: &TaskResult) -> Result<String> {
        // Extract the response field from the JSON output
        if let Some(response) = result.output.get("response") {
            Ok(response.as_str().unwrap_or("").to_string())
        } else {
            Ok(result.output.to_string())
        }
    }

    /// Evaluate using Claude as the judge
    async fn evaluate_with_claude(
        &self,
        content: &str,
        task: &Task,
        agent_role: &AgentRole,
        workspace_path: &str,
    ) -> Result<QualityEvaluation> {
        let prompt = self.generate_evaluation_prompt(content, task, agent_role);

        // Execute Claude for evaluation
        let mut cmd = Command::new("claude");
        cmd.current_dir(workspace_path)
            .arg("-p")
            .arg(&prompt)
            .arg("--json");

        let output = cmd
            .output()
            .await
            .context("Failed to execute Claude for quality evaluation")?;

        if !output.status.success() {
            warn!("Claude evaluation failed, falling back to heuristics");
            return self.evaluate_with_heuristics(content, task, agent_role);
        }

        // Parse Claude's response
        let response = String::from_utf8_lossy(&output.stdout);
        self.parse_claude_evaluation(&response, agent_role)
    }

    /// Generate evaluation prompt for Claude
    fn generate_evaluation_prompt(
        &self,
        content: &str,
        task: &Task,
        agent_role: &AgentRole,
    ) -> String {
        let role_weights = self.rubric.get_role_weights(agent_role.name());
        let dimensions: Vec<String> = role_weights.keys().cloned().collect();

        format!(
            r#"You are a code quality judge evaluating the output of a {} agent.

## Task Description
{}

## Code/Content to Evaluate
```
{}
```

## Evaluation Rubric
Please evaluate the code on these dimensions (0.0 to 1.0 scale):
{}

## Required Output Format
Provide your evaluation as a JSON object with this structure:
{{
  "overall_score": 0.0-1.0,
  "dimensions": {{
    "correctness": 0.0-1.0,
    "maintainability": 0.0-1.0,
    "test_quality": 0.0-1.0,
    "security": 0.0-1.0,
    "performance": 0.0-1.0,
    "documentation": 0.0-1.0,
    "architecture": 0.0-1.0,
    "error_handling": 0.0-1.0
  }},
  "issues": [
    {{
      "severity": "critical|high|medium|low",
      "category": "Security|Performance|TestCoverage|CodeComplexity|Documentation|ErrorHandling|Architecture|BestPractices|Accessibility|TypeSafety",
      "description": "Clear description of the issue",
      "suggested_fix": "Specific suggestion for fixing the issue",
      "affected_areas": ["file.js", "component.tsx"],
      "fix_effort": 30
    }}
  ],
  "feedback": "Overall assessment and recommendations",
  "passes_standards": true|false,
  "confidence": 0.0-1.0
}}

Focus on {} agent-specific concerns. Be thorough but constructive."#,
            agent_role.name(),
            task.description,
            content,
            dimensions.join("\n"),
            agent_role.name()
        )
    }

    /// Parse Claude's evaluation response
    fn parse_claude_evaluation(
        &self,
        response: &str,
        agent_role: &AgentRole,
    ) -> Result<QualityEvaluation> {
        // Try to parse JSON response
        let json_result: serde_json::Value = serde_json::from_str(response)
            .context("Failed to parse Claude's evaluation response")?;

        let overall_score = json_result["overall_score"].as_f64().unwrap_or(0.5);

        let mut dimension_scores = HashMap::new();
        if let Some(dimensions) = json_result["dimensions"].as_object() {
            for (key, value) in dimensions {
                dimension_scores.insert(key.clone(), value.as_f64().unwrap_or(0.5));
            }
        }

        let mut issues = Vec::new();
        if let Some(issues_array) = json_result["issues"].as_array() {
            for issue_json in issues_array {
                if let Ok(issue) = self.parse_quality_issue(issue_json) {
                    issues.push(issue);
                }
            }
        }

        let feedback = json_result["feedback"]
            .as_str()
            .unwrap_or("No detailed feedback provided")
            .to_string();

        let passes_standards = json_result["passes_standards"]
            .as_bool()
            .unwrap_or(overall_score >= 0.85);

        let confidence = json_result["confidence"].as_f64().unwrap_or(0.8);

        Ok(QualityEvaluation {
            overall_score,
            dimension_scores,
            issues,
            feedback,
            passes_standards,
            confidence,
            evaluated_at: Utc::now(),
            metadata: EvaluationMetadata {
                files_analyzed: 1,
                lines_of_code: response.lines().count(),
                duration_ms: 0, // Will be set by caller
                tools_used: vec!["Claude".to_string()],
                agent_role: agent_role.name().to_string(),
            },
        })
    }

    /// Parse a quality issue from JSON
    pub fn parse_quality_issue(&self, json: &serde_json::Value) -> Result<QualityIssue> {
        let severity = match json["severity"].as_str().unwrap_or("medium") {
            "critical" => IssueSeverity::Critical,
            "high" => IssueSeverity::High,
            "low" => IssueSeverity::Low,
            _ => IssueSeverity::Medium,
        };

        let category = match json["category"].as_str().unwrap_or("BestPractices") {
            "Security" => IssueCategory::Security,
            "Performance" => IssueCategory::Performance,
            "TestCoverage" => IssueCategory::TestCoverage,
            "CodeComplexity" => IssueCategory::CodeComplexity,
            "Documentation" => IssueCategory::Documentation,
            "ErrorHandling" => IssueCategory::ErrorHandling,
            "Architecture" => IssueCategory::Architecture,
            "Accessibility" => IssueCategory::Accessibility,
            "TypeSafety" => IssueCategory::TypeSafety,
            _ => IssueCategory::BestPractices,
        };

        let affected_areas = json["affected_areas"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(QualityIssue {
            severity,
            category,
            description: json["description"].as_str().unwrap_or("").to_string(),
            suggested_fix: json["suggested_fix"].as_str().unwrap_or("").to_string(),
            affected_areas,
            fix_effort: json["fix_effort"].as_u64().unwrap_or(30) as u32,
        })
    }

    /// Fallback heuristic-based evaluation
    fn evaluate_with_heuristics(
        &self,
        content: &str,
        _task: &Task,
        agent_role: &AgentRole,
    ) -> Result<QualityEvaluation> {
        let mut dimension_scores = HashMap::new();
        let mut issues = Vec::new();

        // Basic heuristics
        let lines = content.lines().count();
        let has_tests = content.contains("test") || content.contains("spec");
        let has_error_handling = content.contains("try")
            || content.contains("catch")
            || content.contains("Result")
            || content.contains("Option");
        let has_comments =
            content.contains("//") || content.contains("/*") || content.contains("#");

        // Score dimensions based on heuristics
        dimension_scores.insert("correctness".to_string(), 0.8); // Assume mostly correct
        dimension_scores.insert(
            "maintainability".to_string(),
            if lines < 200 { 0.9 } else { 0.7 },
        );
        dimension_scores.insert(
            "test_quality".to_string(),
            if has_tests { 0.8 } else { 0.3 },
        );
        dimension_scores.insert("security".to_string(), 0.7); // Conservative estimate
        dimension_scores.insert("performance".to_string(), 0.7);
        dimension_scores.insert(
            "documentation".to_string(),
            if has_comments { 0.7 } else { 0.4 },
        );
        dimension_scores.insert(
            "error_handling".to_string(),
            if has_error_handling { 0.8 } else { 0.5 },
        );
        dimension_scores.insert("architecture".to_string(), 0.7);

        // Generate issues based on low scores
        if !has_tests {
            issues.push(QualityIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::TestCoverage,
                description: "No tests found for the implementation".to_string(),
                suggested_fix: "Add unit tests covering main functionality and edge cases"
                    .to_string(),
                affected_areas: vec![],
                fix_effort: 60,
            });
        }

        if !has_error_handling {
            issues.push(QualityIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::ErrorHandling,
                description: "Limited error handling detected".to_string(),
                suggested_fix: "Add proper error handling for edge cases and failures".to_string(),
                affected_areas: vec![],
                fix_effort: 30,
            });
        }

        // Calculate overall score
        let weights = self.rubric.get_role_weights(agent_role.name());
        let overall_score = dimension_scores
            .iter()
            .map(|(dim, score)| weights.get(dim).unwrap_or(&0.1) * score)
            .sum::<f64>()
            / weights.values().sum::<f64>();

        let passes_standards = overall_score >= 0.85
            && issues
                .iter()
                .filter(|i| matches!(i.severity, IssueSeverity::Critical | IssueSeverity::High))
                .count()
                == 0;

        Ok(QualityEvaluation {
            overall_score,
            dimension_scores,
            issues: issues.clone(),
            feedback: format!(
                "Heuristic evaluation for {} agent: {} issues found. {}",
                agent_role.name(),
                issues.len(),
                if passes_standards {
                    "Meets standards."
                } else {
                    "Needs improvement."
                }
            ),
            passes_standards,
            confidence: 0.6, // Lower confidence for heuristic evaluation
            evaluated_at: Utc::now(),
            metadata: EvaluationMetadata {
                files_analyzed: 1,
                lines_of_code: lines,
                duration_ms: 0,
                tools_used: vec!["Heuristics".to_string()],
                agent_role: agent_role.name().to_string(),
            },
        })
    }

    /// Convert evaluation to issue strings for messaging
    pub fn evaluation_to_issues(&self, evaluation: &QualityEvaluation) -> Vec<String> {
        evaluation
            .issues
            .iter()
            .map(|issue| {
                format!(
                    "[{:?}] {:?}: {}",
                    issue.severity, issue.category, issue.description
                )
            })
            .collect()
    }

    /// Generate fix instructions for issues
    pub fn generate_fix_instructions(&self, issues: &[QualityIssue], agent_role: &str) -> String {
        if issues.is_empty() {
            return "No issues found. Great work!".to_string();
        }

        let mut instructions = format!(
            "## Quality Review - Fix Instructions for {} Agent\n\n",
            agent_role
        );

        // Group issues by severity
        let mut critical_issues = Vec::new();
        let mut high_issues = Vec::new();
        let mut medium_issues = Vec::new();
        let mut low_issues = Vec::new();

        for issue in issues {
            match issue.severity {
                IssueSeverity::Critical => critical_issues.push(issue),
                IssueSeverity::High => high_issues.push(issue),
                IssueSeverity::Medium => medium_issues.push(issue),
                IssueSeverity::Low => low_issues.push(issue),
            }
        }

        // Add instructions by priority
        if !critical_issues.is_empty() {
            instructions.push_str("### 🚨 CRITICAL Issues (Fix Immediately)\n");
            for issue in critical_issues {
                instructions.push_str(&format!(
                    "- **{:?}**: {}\n  - Fix: {}\n  - Effort: {} minutes\n\n",
                    issue.category, issue.description, issue.suggested_fix, issue.fix_effort
                ));
            }
        }

        if !high_issues.is_empty() {
            instructions.push_str("### ⚠️  HIGH Priority Issues\n");
            for issue in high_issues {
                instructions.push_str(&format!(
                    "- **{:?}**: {}\n  - Fix: {}\n  - Effort: {} minutes\n\n",
                    issue.category, issue.description, issue.suggested_fix, issue.fix_effort
                ));
            }
        }

        if !medium_issues.is_empty() {
            instructions.push_str("### 📝 MEDIUM Priority Issues\n");
            for issue in medium_issues {
                instructions.push_str(&format!(
                    "- **{:?}**: {}\n  - Fix: {}\n\n",
                    issue.category, issue.description, issue.suggested_fix
                ));
            }
        }

        instructions.push_str(&format!(
            "\n### Estimated Total Fix Time: {} minutes\n",
            issues.iter().map(|i| i.fix_effort).sum::<u32>()
        ));

        instructions
    }
}

