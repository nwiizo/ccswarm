#[cfg(test)]
mod tests {
    use super::super::llm_quality_judge::*;
    use crate::agent::{Priority, Task, TaskResult, TaskType};
    use crate::identity::AgentRole;
    use std::collections::HashMap;

    #[test]
    fn test_quality_issue_parsing() {
        let json = serde_json::json!({
            "severity": "high",
            "category": "TestCoverage",
            "description": "No tests found",
            "suggested_fix": "Add unit tests",
            "affected_areas": ["main.rs"],
            "fix_effort": 45
        });

        let judge = LLMQualityJudge::new();
        let issue = judge.parse_quality_issue(&json).unwrap();

        assert_eq!(issue.severity, IssueSeverity::High);
        assert!(matches!(issue.category, IssueCategory::TestCoverage));
        assert_eq!(issue.description, "No tests found");
        assert_eq!(issue.fix_effort, 45);
    }

    #[test]
    fn test_evaluation_to_issues_conversion() {
        let issues = vec![
            QualityIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Security,
                description: "SQL injection vulnerability".to_string(),
                suggested_fix: "Use parameterized queries".to_string(),
                affected_areas: vec!["db.rs".to_string()],
                fix_effort: 30,
            },
            QualityIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Documentation,
                description: "Missing function documentation".to_string(),
                suggested_fix: "Add rustdoc comments".to_string(),
                affected_areas: vec!["lib.rs".to_string()],
                fix_effort: 15,
            },
        ];

        let evaluation = QualityEvaluation {
            overall_score: 0.7,
            dimension_scores: HashMap::new(),
            issues,
            feedback: "Needs security fixes".to_string(),
            passes_standards: false,
            confidence: 0.9,
            evaluated_at: chrono::Utc::now(),
            metadata: EvaluationMetadata {
                files_analyzed: 2,
                lines_of_code: 200,
                duration_ms: 100,
                tools_used: vec!["Heuristics".to_string()],
                agent_role: "Backend".to_string(),
            },
        };

        let judge = LLMQualityJudge::new();
        let issue_strings = judge.evaluation_to_issues(&evaluation);

        assert_eq!(issue_strings.len(), 2);
        assert!(issue_strings[0].contains("Critical"));
        assert!(issue_strings[0].contains("Security"));
        assert!(issue_strings[1].contains("Medium"));
        assert!(issue_strings[1].contains("Documentation"));
    }

    #[test]
    fn test_fix_instruction_generation() {
        let issues = vec![
            QualityIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Security,
                description: "Hardcoded credentials".to_string(),
                suggested_fix: "Use environment variables".to_string(),
                affected_areas: vec!["config.rs".to_string()],
                fix_effort: 20,
            },
            QualityIssue {
                severity: IssueSeverity::High,
                category: IssueCategory::TestCoverage,
                description: "Test coverage at 45%".to_string(),
                suggested_fix: "Add tests to reach 85% coverage".to_string(),
                affected_areas: vec!["handlers.rs".to_string()],
                fix_effort: 90,
            },
        ];

        let judge = LLMQualityJudge::new();
        let instructions = judge.generate_fix_instructions(&issues, "Backend");

        assert!(instructions.contains("Backend Agent"));
        assert!(instructions.contains("CRITICAL"));
        assert!(instructions.contains("HIGH"));
        assert!(instructions.contains("110 minutes")); // 20 + 90
    }

    #[tokio::test]
    async fn test_heuristic_evaluation_frontend() {
        let mut judge = LLMQualityJudge::new();
        judge.use_claude = false;

        let task = Task::new(
            "ui-task".to_string(),
            "Create responsive navbar".to_string(),
            Priority::High,
            TaskType::Development,
        );

        let result = TaskResult {
            success: true,
            output: serde_json::json!({
                "response": r#"
function Navbar() {
  return (
    <nav className="navbar">
      <div className="nav-links">
        <a href="/">Home</a>
        <a href="/about">About</a>
      </div>
    </nav>
  );
}
"#
            }),
            error: None,
            duration: std::time::Duration::from_secs(2),
        };

        let role = AgentRole::Frontend {
            technologies: vec!["React".to_string()],
            responsibilities: vec![],
            boundaries: vec![],
        };

        let evaluation = judge
            .evaluate_task(&task, &result, &role, "/tmp")
            .await
            .unwrap();

        // Should detect missing tests
        assert!(!evaluation.issues.is_empty());
        assert!(evaluation
            .issues
            .iter()
            .any(|i| matches!(i.category, IssueCategory::TestCoverage)));

        // Should have reasonable scores
        assert!(evaluation.overall_score > 0.4);
        assert!(evaluation.overall_score < 0.9);
    }

    #[tokio::test]
    async fn test_evaluation_caching() {
        let mut judge = LLMQualityJudge::new();
        judge.use_claude = false;

        let task = Task::new(
            "test-cache".to_string(),
            "Test caching".to_string(),
            Priority::Low,
            TaskType::Testing,
        );

        let result = TaskResult {
            success: true,
            output: serde_json::json!({"response": "cached content"}),
            error: None,
            duration: std::time::Duration::from_secs(1),
        };

        let role = AgentRole::QA {
            technologies: vec!["Jest".to_string()],
            responsibilities: vec![],
            boundaries: vec![],
        };

        // First evaluation
        let eval1 = judge
            .evaluate_task(&task, &result, &role, "/tmp")
            .await
            .unwrap();

        // Second evaluation (should be cached)
        let eval2 = judge
            .evaluate_task(&task, &result, &role, "/tmp")
            .await
            .unwrap();

        // Should return same evaluation
        assert_eq!(eval1.overall_score, eval2.overall_score);
        assert_eq!(eval1.issues.len(), eval2.issues.len());
    }

    #[test]
    fn test_severity_ordering() {
        let mut issues = vec![
            QualityIssue {
                severity: IssueSeverity::Low,
                category: IssueCategory::Documentation,
                description: "Minor doc issue".to_string(),
                suggested_fix: "Update docs".to_string(),
                affected_areas: vec![],
                fix_effort: 5,
            },
            QualityIssue {
                severity: IssueSeverity::Critical,
                category: IssueCategory::Security,
                description: "Security issue".to_string(),
                suggested_fix: "Fix immediately".to_string(),
                affected_areas: vec![],
                fix_effort: 30,
            },
            QualityIssue {
                severity: IssueSeverity::Medium,
                category: IssueCategory::Performance,
                description: "Perf issue".to_string(),
                suggested_fix: "Optimize".to_string(),
                affected_areas: vec![],
                fix_effort: 20,
            },
        ];

        // Sort by severity (highest first)
        issues.sort_by(|a, b| b.severity.cmp(&a.severity));

        assert_eq!(issues[0].severity, IssueSeverity::Critical);
        assert_eq!(issues[1].severity, IssueSeverity::Medium);
        assert_eq!(issues[2].severity, IssueSeverity::Low);
    }

    #[test]
    fn test_role_specific_weights() {
        let rubric = QualityRubric::default();

        // Frontend should have accessibility weight
        let frontend_weights = rubric.get_role_weights("Frontend");
        assert!(frontend_weights.contains_key("accessibility"));

        // Backend should prioritize security
        let backend_weights = rubric.get_role_weights("Backend");
        let backend_security = backend_weights.get("security").unwrap();
        let backend_docs = backend_weights.get("documentation").unwrap();
        assert!(backend_security > backend_docs);

        // DevOps should prioritize security even more
        let devops_weights = rubric.get_role_weights("DevOps");
        let devops_security = devops_weights.get("security").unwrap();
        assert_eq!(devops_security, &0.3); // Highest weight
    }

    #[test]
    fn test_issue_categories() {
        // Test all categories can be created
        let categories = vec![
            IssueCategory::Security,
            IssueCategory::Performance,
            IssueCategory::TestCoverage,
            IssueCategory::CodeComplexity,
            IssueCategory::Documentation,
            IssueCategory::ErrorHandling,
            IssueCategory::Architecture,
            IssueCategory::BestPractices,
            IssueCategory::Accessibility,
            IssueCategory::TypeSafety,
        ];

        // Ensure we can format all categories
        for category in categories {
            let formatted = format!("{:?}", category);
            assert!(!formatted.is_empty());
        }
    }
}
