#[cfg(test)]
mod identity_tests {
    use crate::agent::{Priority, Task, TaskType};
    use crate::identity::boundary::{TaskBoundaryChecker, TaskEvaluation};
    use crate::identity::{
        default_backend_role, default_devops_role, default_frontend_role, default_qa_role,
        AgentIdentity, AgentRole, IdentityMonitor, IdentityStatus, ResponseParser,
    };
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn create_test_identity(role: AgentRole) -> AgentIdentity {
        AgentIdentity {
            agent_id: format!("{}-agent-test", role.name().to_lowercase()),
            specialization: role,
            workspace_path: PathBuf::from("/test/workspace"),
            env_vars: HashMap::new(),
            session_id: Uuid::new_v4().to_string(),
            parent_process_id: "1234".to_string(),
            initialized_at: Utc::now(),
        }
    }

    fn create_test_task(description: &str, task_type: TaskType) -> Task {
        Task::new(
            Uuid::new_v4().to_string(),
            description.to_string(),
            Priority::Medium,
            task_type,
        )
    }

    #[test]
    fn test_agent_role_names() {
        assert_eq!(default_frontend_role().name(), "Frontend");
        assert_eq!(default_backend_role().name(), "Backend");
        assert_eq!(default_devops_role().name(), "DevOps");
        assert_eq!(default_qa_role().name(), "QA");
    }

    #[test]
    fn test_agent_role_technologies() {
        let frontend = default_frontend_role();
        let technologies = frontend.technologies();
        assert!(technologies.contains(&"React".to_string()));
        assert!(technologies.contains(&"TypeScript".to_string()));
        assert!(technologies.contains(&"Tailwind CSS".to_string()));

        let backend = default_backend_role();
        let technologies = backend.technologies();
        assert!(technologies.contains(&"Node.js".to_string()));
        assert!(technologies.contains(&"Express".to_string()));
        assert!(technologies.contains(&"PostgreSQL".to_string()));
    }

    #[test]
    fn test_agent_role_responsibilities() {
        let frontend = default_frontend_role();
        let responsibilities = frontend.responsibilities();
        assert!(responsibilities.contains(&"UI Component Development".to_string()));
        assert!(responsibilities.contains(&"State Management".to_string()));

        let backend = default_backend_role();
        let responsibilities = backend.responsibilities();
        assert!(responsibilities.contains(&"API Development".to_string()));
        assert!(responsibilities.contains(&"Database Design".to_string()));
    }

    #[test]
    fn test_agent_role_boundaries() {
        let frontend = default_frontend_role();
        let boundaries = frontend.boundaries();
        assert!(boundaries.contains(&"No backend API development".to_string()));
        assert!(boundaries.contains(&"No database operations".to_string()));

        let devops = default_devops_role();
        let boundaries = devops.boundaries();
        assert!(boundaries.contains(&"No application code changes".to_string()));
        assert!(boundaries.contains(&"No business logic implementation".to_string()));
    }

    #[test]
    fn test_identity_monitor_creation() {
        let monitor = IdentityMonitor::new("test-agent");
        assert_eq!(monitor.agent_id, "test-agent");
        assert!(monitor.identity_drift_threshold.as_secs() > 0);
    }

    #[test]
    fn test_identity_monitor_header_check() {
        let monitor = IdentityMonitor::new("Frontend");

        let valid_response = "🤖 AGENT: Frontend\n📁 WORKSPACE: /test\n🎯 SCOPE: UI work";
        assert!(monitor.check_identity_header(valid_response));

        let invalid_response = "Working on the task...";
        assert!(!monitor.check_identity_header(invalid_response));
    }

    #[tokio::test]
    async fn test_identity_monitor_response_analysis() {
        let mut monitor = IdentityMonitor::new("Frontend");

        let healthy_response = r#"
🤖 AGENT: Frontend
📁 WORKSPACE: /test/workspace
🎯 SCOPE: Component development

I'm working on creating a React component for the user profile page.
This is within my frontend specialization.
"#;

        let result = monitor.monitor_response(healthy_response).await.unwrap();
        assert!(matches!(result, IdentityStatus::Healthy));

        let unhealthy_response = "Working on backend API endpoints...";
        let result = monitor.monitor_response(unhealthy_response).await.unwrap();
        assert!(matches!(result, IdentityStatus::DriftDetected(_)));
    }

    #[test]
    fn test_response_parser() {
        let parser = ResponseParser::new();

        let response = r#"
🤖 AGENT: Frontend
📁 WORKSPACE: agents/frontend-agent/
🎯 SCOPE: Component development

Working on React component...
"#;

        let parsed = parser.parse_identity(response);
        assert!(parsed.is_some());

        let (agent, workspace, scope) = parsed.unwrap();
        assert_eq!(agent, "Frontend");
        assert_eq!(workspace, "agents/frontend-agent/");
        assert_eq!(scope, "Component development");
    }

    #[test]
    fn test_response_parser_missing_fields() {
        let parser = ResponseParser::new();

        let incomplete_response = r#"
🤖 AGENT: Frontend
Working on React component...
"#;

        let parsed = parser.parse_identity(incomplete_response);
        assert!(parsed.is_none());
    }

    // === Boundary Testing ===

    #[tokio::test]
    async fn test_frontend_boundary_checker_accepts_ui_tasks() {
        let checker = TaskBoundaryChecker::new(default_frontend_role());

        let ui_tasks = vec![
            create_test_task(
                "Create a React component for user profile",
                TaskType::Development,
            ),
            create_test_task("Implement CSS styling with Tailwind", TaskType::Development),
            create_test_task("Add state management with Redux", TaskType::Development),
            create_test_task("Write Jest tests for components", TaskType::Testing),
        ];

        for task in ui_tasks {
            let result = checker.evaluate_task(&task).await;
            assert!(
                matches!(result, TaskEvaluation::Accept { .. }),
                "Frontend should accept UI task: {}",
                task.description
            );
        }
    }

    #[tokio::test]
    async fn test_frontend_boundary_checker_delegates_backend_tasks() {
        let checker = TaskBoundaryChecker::new(default_frontend_role());

        let backend_tasks = vec![
            create_test_task(
                "Create REST API endpoint for authentication",
                TaskType::Development,
            ),
            create_test_task("Design database schema for users", TaskType::Development),
            create_test_task("Implement JWT token validation", TaskType::Development),
            create_test_task("Set up GraphQL resolver", TaskType::Development),
        ];

        for task in backend_tasks {
            let result = checker.evaluate_task(&task).await;
            assert!(
                matches!(result, TaskEvaluation::Delegate { .. }),
                "Frontend should delegate backend task: {}",
                task.description
            );

            if let TaskEvaluation::Delegate { target_agent, .. } = result {
                assert_eq!(target_agent, "backend-agent");
            }
        }
    }

    #[tokio::test]
    async fn test_backend_boundary_checker_accepts_api_tasks() {
        let checker = TaskBoundaryChecker::new(default_backend_role());

        let api_tasks = vec![
            create_test_task("Create REST API for user management", TaskType::Development),
            create_test_task(
                "Implement database query optimization",
                TaskType::Development,
            ),
            create_test_task("Add OAuth authentication flow", TaskType::Development),
            create_test_task("Set up Prisma ORM models", TaskType::Development),
        ];

        for task in api_tasks {
            let result = checker.evaluate_task(&task).await;
            assert!(
                matches!(result, TaskEvaluation::Accept { .. }),
                "Backend should accept API task: {}",
                task.description
            );
        }
    }

    #[tokio::test]
    async fn test_backend_boundary_checker_delegates_ui_tasks() {
        let checker = TaskBoundaryChecker::new(default_backend_role());

        let ui_tasks = vec![
            create_test_task(
                "Create React component for login form",
                TaskType::Development,
            ),
            create_test_task("Style the user dashboard with CSS", TaskType::Development),
            create_test_task("Implement frontend routing", TaskType::Development),
        ];

        for task in ui_tasks {
            let result = checker.evaluate_task(&task).await;
            assert!(
                matches!(result, TaskEvaluation::Delegate { .. }),
                "Backend should delegate UI task: {}",
                task.description
            );

            if let TaskEvaluation::Delegate { target_agent, .. } = result {
                assert_eq!(target_agent, "frontend-agent");
            }
        }
    }

    #[tokio::test]
    async fn test_devops_boundary_checker_accepts_infrastructure_tasks() {
        let checker = TaskBoundaryChecker::new(default_devops_role());

        let infra_tasks = vec![
            create_test_task("Set up Kubernetes cluster", TaskType::Infrastructure),
            create_test_task("Configure Docker containers", TaskType::Infrastructure),
            create_test_task("Deploy application to AWS", TaskType::Infrastructure),
            create_test_task("Set up CI/CD pipeline", TaskType::Infrastructure),
            create_test_task(
                "Configure monitoring with Prometheus",
                TaskType::Infrastructure,
            ),
        ];

        for task in infra_tasks {
            let result = checker.evaluate_task(&task).await;
            assert!(
                matches!(result, TaskEvaluation::Accept { .. }),
                "DevOps should accept infrastructure task: {}",
                task.description
            );
        }
    }

    #[tokio::test]
    async fn test_devops_boundary_checker_delegates_application_tasks() {
        let checker = TaskBoundaryChecker::new(default_devops_role());

        let app_tasks = vec![
            create_test_task("Fix bug in user authentication logic", TaskType::Bugfix),
            create_test_task("Add new feature to the UI", TaskType::Feature),
            create_test_task("Optimize database queries", TaskType::Development),
        ];

        for task in app_tasks {
            let result = checker.evaluate_task(&task).await;
            assert!(
                matches!(result, TaskEvaluation::Delegate { .. }),
                "DevOps should delegate application task: {}",
                task.description
            );
        }
    }

    #[tokio::test]
    async fn test_qa_boundary_checker_accepts_testing_tasks() {
        let checker = TaskBoundaryChecker::new(default_qa_role());

        let testing_tasks = vec![
            create_test_task("Write end-to-end tests with Cypress", TaskType::Testing),
            create_test_task("Perform load testing with K6", TaskType::Testing),
            create_test_task("Execute security penetration testing", TaskType::Testing),
            create_test_task("Create test automation framework", TaskType::Testing),
        ];

        for task in testing_tasks {
            let result = checker.evaluate_task(&task).await;
            assert!(
                matches!(result, TaskEvaluation::Accept { .. }),
                "QA should accept testing task: {}",
                task.description
            );
        }
    }

    #[tokio::test]
    async fn test_qa_boundary_checker_delegates_implementation_tasks() {
        let checker = TaskBoundaryChecker::new(default_qa_role());

        let impl_tasks = vec![
            create_test_task("Implement new user registration feature", TaskType::Feature),
            create_test_task("Fix production bug in payment system", TaskType::Bugfix),
            create_test_task("Deploy application to production", TaskType::Infrastructure),
        ];

        for task in impl_tasks {
            let result = checker.evaluate_task(&task).await;
            assert!(
                matches!(result, TaskEvaluation::Delegate { .. }),
                "QA should delegate implementation task: {}",
                task.description
            );
        }
    }

    #[tokio::test]
    async fn test_ambiguous_task_triggers_clarification() {
        let checker = TaskBoundaryChecker::new(default_frontend_role());

        let ambiguous_tasks = vec![
            create_test_task("Update the user system", TaskType::Development),
            create_test_task("Improve performance", TaskType::Development),
            create_test_task("Fix the login issue", TaskType::Bugfix),
        ];

        for task in ambiguous_tasks {
            let result = checker.evaluate_task(&task).await;
            assert!(
                matches!(result, TaskEvaluation::Clarify { .. }),
                "Ambiguous task should trigger clarification: {}",
                task.description
            );

            if let TaskEvaluation::Clarify { questions, .. } = result {
                assert!(!questions.is_empty());
                assert!(questions.iter().any(|q| q.contains("Frontend")));
            }
        }
    }

    // === Identity Persistence Tests ===

    #[test]
    fn test_identity_serialization() {
        let identity = create_test_identity(default_frontend_role());

        let serialized = serde_json::to_string(&identity).unwrap();
        let deserialized: AgentIdentity = serde_json::from_str(&serialized).unwrap();

        assert_eq!(identity.agent_id, deserialized.agent_id);
        assert_eq!(identity.session_id, deserialized.session_id);
        assert_eq!(
            identity.specialization.name(),
            deserialized.specialization.name()
        );
    }

    #[test]
    fn test_correction_prompt_generation() {
        let monitor = IdentityMonitor::new("frontend-agent");
        let prompt = monitor.generate_correction_prompt("/test/workspace", "Frontend");

        assert!(prompt.contains("IDENTITY DRIFT DETECTED"));
        assert!(prompt.contains("frontend-agent"));
        assert!(prompt.contains("Frontend"));
        assert!(prompt.contains("/test/workspace"));
        assert!(prompt.contains("🤖 AGENT:"));
    }

    // === Edge Cases ===

    #[tokio::test]
    async fn test_boundary_checker_with_mixed_keywords() {
        let checker = TaskBoundaryChecker::new(default_frontend_role());

        // Task that contains both frontend and backend keywords
        let mixed_task = create_test_task(
            "Create a React component that calls the backend API",
            TaskType::Development,
        );

        let result = checker.evaluate_task(&mixed_task).await;

        // Should accept because "React component" is strongly frontend
        assert!(matches!(result, TaskEvaluation::Accept { .. }));
    }

    #[tokio::test]
    async fn test_boundary_checker_case_insensitive() {
        let checker = TaskBoundaryChecker::new(default_frontend_role());

        let tasks = vec![
            create_test_task("CREATE A REACT COMPONENT", TaskType::Development),
            create_test_task("create a react component", TaskType::Development),
            create_test_task("Create A React Component", TaskType::Development),
        ];

        for task in tasks {
            let result = checker.evaluate_task(&task).await;
            assert!(
                matches!(result, TaskEvaluation::Accept { .. }),
                "Should be case insensitive: {}",
                task.description
            );
        }
    }

    #[test]
    fn test_identity_env_vars() {
        let identity = create_test_identity(default_frontend_role());

        // Check if environment variables are set up correctly
        assert!(identity.env_vars.contains_key("CCSWARM_AGENT_ID"));
        assert!(identity.env_vars.contains_key("CCSWARM_SESSION_ID"));
        assert!(identity.env_vars.contains_key("CCSWARM_ROLE"));
    }

    // === Performance Tests ===

    #[tokio::test]
    async fn test_boundary_checker_performance() {
        let checker = TaskBoundaryChecker::new(default_frontend_role());
        let task = create_test_task("Create React component", TaskType::Development);

        let start = std::time::Instant::now();

        // Run 100 evaluations
        for _ in 0..100 {
            let _ = checker.evaluate_task(&task).await;
        }

        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 1000,
            "100 boundary checks should complete in under 1 second"
        );
    }

    #[test]
    fn test_response_parser_performance() {
        let parser = ResponseParser::new();
        let response = r#"
🤖 AGENT: Frontend
📁 WORKSPACE: agents/frontend-agent/
🎯 SCOPE: Component development
"#;

        let start = std::time::Instant::now();

        // Run 1000 parses
        for _ in 0..1000 {
            let _ = parser.parse_identity(response);
        }

        let duration = start.elapsed();
        assert!(
            duration.as_millis() < 100,
            "1000 response parses should complete in under 100ms"
        );
    }
}
