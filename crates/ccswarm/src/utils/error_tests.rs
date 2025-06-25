#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_user_error_with_diagram() {
        let error = UserError::new("Test Error")
            .with_details("This is a test error")
            .suggest("Try this fix")
            .with_code("TEST001")
            .with_diagram("Test diagram".to_string())
            .auto_fixable();

        assert_eq!(error.title, "Test Error");
        assert_eq!(error.error_code, Some("TEST001".to_string()));
        assert!(error.can_auto_fix);
        assert!(error.diagram.is_some());
    }

    #[test]
    fn test_common_errors_have_diagrams() {
        let errors = vec![
            CommonErrors::api_key_missing("Anthropic"),
            CommonErrors::session_not_found("test-123"),
            CommonErrors::config_not_found(),
            CommonErrors::git_not_initialized(),
            CommonErrors::permission_denied("/test/path"),
            CommonErrors::network_error("https://example.com"),
            CommonErrors::worktree_conflict("feature/test"),
            CommonErrors::agent_busy("Frontend"),
        ];

        for error in errors {
            assert!(
                error.diagram.is_some(),
                "Error {} should have a diagram",
                error.title
            );
            assert!(
                error.error_code.is_some(),
                "Error {} should have a code",
                error.title
            );
        }
    }

    #[test]
    fn test_auto_fixable_errors() {
        let auto_fixable = vec![
            CommonErrors::session_not_found("test"),
            CommonErrors::config_not_found(),
            CommonErrors::git_not_initialized(),
            CommonErrors::permission_denied("/test"),
            CommonErrors::worktree_conflict("branch"),
        ];

        for error in auto_fixable {
            assert!(
                error.can_auto_fix,
                "Error {} should be auto-fixable",
                error.title
            );
        }
    }

    #[test]
    fn test_non_auto_fixable_errors() {
        let non_fixable = vec![
            CommonErrors::api_key_missing("Provider"),
            CommonErrors::agent_busy("Agent"),
            CommonErrors::network_error("url"),
            CommonErrors::invalid_task_format(),
            CommonErrors::ai_response_error(),
        ];

        for error in non_fixable {
            assert!(
                !error.can_auto_fix,
                "Error {} should not be auto-fixable",
                error.title
            );
        }
    }

    #[tokio::test]
    async fn test_error_recovery_db() {
        use error_recovery::ErrorRecoveryDB;

        let db = ErrorRecoveryDB::new();

        // Verify all common error codes have recovery actions
        let codes = vec![
            "ENV001", "SES001", "CFG001", "GIT001", "PRM001", "NET001", "WRK001", "AI001",
        ];

        for code in codes {
            assert!(
                db.get_recovery(code).is_some(),
                "Recovery action missing for code: {}",
                code
            );
        }
    }

    #[test]
    fn test_error_diagrams_render_without_panic() {
        use error_diagrams::ErrorDiagrams;

        // Just ensure they don't panic when rendering
        let diagrams = vec![
            ErrorDiagrams::network_error(),
            ErrorDiagrams::session_error(),
            ErrorDiagrams::git_worktree_error(),
            ErrorDiagrams::permission_error(),
            ErrorDiagrams::config_error(),
            ErrorDiagrams::task_error(),
            ErrorDiagrams::api_key_error(),
            ErrorDiagrams::agent_error(),
        ];

        for diagram in diagrams {
            assert!(!diagram.is_empty());
            assert!(diagram.contains("│") || diagram.contains("─")); // Has box drawing
        }
    }

    #[test]
    fn test_risk_levels() {
        use error_recovery::RiskLevel;

        assert_eq!(RiskLevel::Safe.icon(), "✅");
        assert_eq!(RiskLevel::Low.icon(), "🟢");
        assert_eq!(RiskLevel::Medium.icon(), "🟡");
        assert_eq!(RiskLevel::High.icon(), "🔴");
    }
}
