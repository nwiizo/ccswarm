#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::ClaudeConfig;
    use crate::identity::default_frontend_role;
    use tempfile::TempDir;

    /// Create a mock agent for testing
    async fn create_test_agent() -> Result<ClaudeCodeAgent> {
        let temp_dir = TempDir::new()?;
        let workspace = temp_dir.path().to_path_buf();
        let config = ClaudeConfig::default();

        ClaudeCodeAgent::new(default_frontend_role(), &workspace, "test", config).await
    }

    #[tokio::test]
    async fn test_interleaved_thinking_in_task_execution() {
        let _agent = create_test_agent().await.unwrap();

        // Create a test task
        let _task = Task::new(
            "test-123".to_string(),
            "Create a React component with error handling".to_string(),
            Priority::High,
            TaskType::Development,
        );

        // Mock the execute_claude_command to return different outputs
        // In real tests, we'd use a trait and mock implementation

        // The task execution should now include thinking steps
        // This is a integration point test - actual execution would require Claude
    }

    #[tokio::test]
    async fn test_thinking_engine_integration() {
        let mut thinking_engine = InterleavedThinkingEngine::new().with_config(10, 0.7);

        // Simulate agent observations during task execution
        let observations = vec![
            "Starting React component development",
            "Component structure created successfully",
            "Error: useState hook called conditionally",
            "Refactoring hook usage",
            "Component tests passing",
        ];

        for obs in observations {
            let step = thinking_engine
                .process_observation(obs, "Frontend")
                .await
                .unwrap();

            match &step.decision {
                Decision::Continue { .. } => println!("Continuing..."),
                Decision::Refine { refinement, .. } => {
                    println!("Refining: {}", refinement);
                }
                Decision::Complete { summary } => {
                    println!("Complete: {}", summary);
                    break;
                }
                _ => {}
            }
        }

        let summary = thinking_engine.get_thinking_summary();
        assert!(summary.total_steps >= 5);
        assert!(summary.avg_confidence > 0.5);
    }

    #[tokio::test]
    async fn test_confidence_degradation_and_pivot() {
        let mut thinking_engine = InterleavedThinkingEngine::new().with_config(10, 0.6);

        // Simulate repeated errors
        for i in 0..5 {
            let step = thinking_engine
                .process_observation(&format!("Error: Failed attempt #{}", i), "Backend")
                .await
                .unwrap();

            // After several errors, confidence should be lower
            if i >= 3 {
                assert!(step.confidence < 0.6);
            }
        }
    }

    #[tokio::test]
    async fn test_role_specific_thinking() {
        let mut thinking_engine = InterleavedThinkingEngine::new();

        // Test Frontend thinking
        let frontend_step = thinking_engine
            .process_observation("CSS styling issue with responsive layout", "Frontend")
            .await
            .unwrap();

        assert!(
            frontend_step.reflection.contains("responsive")
                || frontend_step.reflection.contains("styling")
        );

        // Test Backend thinking
        let backend_step = thinking_engine
            .process_observation("Database query performance degradation", "Backend")
            .await
            .unwrap();

        assert!(backend_step.reflection.contains("query optimization"));

        // Test DevOps thinking
        let devops_step = thinking_engine
            .process_observation("Docker container build failed", "DevOps")
            .await
            .unwrap();

        assert!(devops_step.reflection.contains("image optimization"));
    }
}
