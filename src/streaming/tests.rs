#[cfg(test)]
mod tests {
    use crate::monitoring::{MonitoringSystem, OutputEntry, OutputFilter, OutputType};
    use crate::streaming::*;
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tokio::time::{sleep, Duration};

    /// Test stream configuration
    #[test]
    fn test_stream_config_default() {
        let config = StreamConfig::default();
        assert_eq!(config.buffer_size, 1000);
        assert_eq!(config.max_line_length, 2000);
        assert!(config.enable_filtering);
        assert!(config.enable_highlighting);
        assert_eq!(config.refresh_rate_ms, 100);
    }

    /// Test stream subscription creation
    #[test]
    fn test_stream_subscription() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let sub = StreamSubscription::new("test-sub".to_string(), tx);

        assert_eq!(sub.id, "test-sub");
        assert!(sub.agent_id.is_none());
        assert!(sub.filter.is_none());

        // Test with agent
        let sub = sub.with_agent("agent-123".to_string());
        assert_eq!(sub.agent_id, Some("agent-123".to_string()));

        // Test with filter
        let filter = OutputFilter {
            output_types: Some(vec![OutputType::Error]),
            ..Default::default()
        };
        let sub = sub.with_filter(filter);
        assert!(sub.filter.is_some());
    }

    /// Test subscription filtering
    #[test]
    fn test_subscription_filtering() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let sub =
            StreamSubscription::new("test-sub".to_string(), tx).with_agent("agent-123".to_string());

        // Create test entries
        let matching_entry = OutputEntry::new(
            "agent-123".to_string(),
            "Test".to_string(),
            OutputType::Info,
            "Test message".to_string(),
            None,
            "session-1".to_string(),
        );

        let non_matching_entry = OutputEntry::new(
            "agent-456".to_string(),
            "Test".to_string(),
            OutputType::Info,
            "Test message".to_string(),
            None,
            "session-1".to_string(),
        );

        assert!(sub.should_receive(&matching_entry));
        assert!(!sub.should_receive(&non_matching_entry));

        // Test with filter
        let filter = OutputFilter {
            output_types: Some(vec![OutputType::Error]),
            ..Default::default()
        };
        let sub = sub.with_filter(filter);

        assert!(!sub.should_receive(&matching_entry)); // Wrong type

        let error_entry = OutputEntry::new(
            "agent-123".to_string(),
            "Test".to_string(),
            OutputType::Error,
            "Error message".to_string(),
            None,
            "session-1".to_string(),
        );

        assert!(sub.should_receive(&error_entry));
    }

    /// Test output formatter
    #[test]
    fn test_output_formatter() {
        let config = StreamConfig::default();
        let formatter = OutputFormatter::new(config);

        let entry = OutputEntry::new(
            "agent-123-very-long-id".to_string(),
            "Frontend".to_string(),
            OutputType::Error,
            "Error: Failed to compile".to_string(),
            Some("task-456".to_string()),
            "session-789".to_string(),
        );

        let formatted = formatter.format_entry(&entry);

        // Check prefix formatting
        assert!(formatted.display_prefix.contains("❌"));
        assert!(formatted.display_prefix.contains("Frontend"));
        assert!(formatted.display_prefix.contains("agent-12")); // Truncated
        assert!(formatted.display_prefix.contains("[task-456]"));

        // Check content formatting
        assert_eq!(formatted.formatted_content, "Error: Failed to compile");

        // Check highlighting
        assert!(!formatted.highlight_spans.is_empty());
        let error_highlight = formatted
            .highlight_spans
            .iter()
            .find(|span| matches!(span.style, HighlightStyle::Error));
        assert!(error_highlight.is_some());
    }

    /// Test content truncation
    #[test]
    fn test_content_truncation() {
        let mut config = StreamConfig::default();
        config.max_line_length = 20;
        let formatter = OutputFormatter::new(config);

        let entry = OutputEntry::new(
            "agent-123".to_string(),
            "Test".to_string(),
            OutputType::Info,
            "This is a very long message that should be truncated".to_string(),
            None,
            "session-1".to_string(),
        );

        let formatted = formatter.format_entry(&entry);
        assert_eq!(formatted.formatted_content, "This is a very lo...");
        assert_eq!(formatted.formatted_content.len(), 20);
    }

    /// Test multi-line content formatting
    #[test]
    fn test_multiline_formatting() {
        let config = StreamConfig::default();
        let formatter = OutputFormatter::new(config);

        let entry = OutputEntry::new(
            "agent-123".to_string(),
            "Test".to_string(),
            OutputType::Info,
            "Line 1\nLine 2\nLine 3".to_string(),
            None,
            "session-1".to_string(),
        );

        let formatted = formatter.format_entry(&entry);
        assert_eq!(formatted.formatted_content, "  Line 1\n  Line 2\n  Line 3");
    }

    /// Test highlight patterns
    #[test]
    fn test_highlight_patterns() {
        let config = StreamConfig::default();
        let formatter = OutputFormatter::new(config);

        // Test error highlighting
        let entry = OutputEntry::new(
            "agent-123".to_string(),
            "Test".to_string(),
            OutputType::Info,
            "An error occurred in the system".to_string(),
            None,
            "session-1".to_string(),
        );

        let formatted = formatter.format_entry(&entry);
        let error_span = formatted
            .highlight_spans
            .iter()
            .find(|span| matches!(span.style, HighlightStyle::Error));
        assert!(error_span.is_some());

        // Test warning highlighting
        let entry = OutputEntry::new(
            "agent-123".to_string(),
            "Test".to_string(),
            OutputType::Info,
            "Warning: deprecated function".to_string(),
            None,
            "session-1".to_string(),
        );

        let formatted = formatter.format_entry(&entry);
        let warning_span = formatted
            .highlight_spans
            .iter()
            .find(|span| matches!(span.style, HighlightStyle::Warning));
        assert!(warning_span.is_some());

        // Test success highlighting
        let entry = OutputEntry::new(
            "agent-123".to_string(),
            "Test".to_string(),
            OutputType::Info,
            "Task completed successfully".to_string(),
            None,
            "session-1".to_string(),
        );

        let formatted = formatter.format_entry(&entry);
        let success_span = formatted
            .highlight_spans
            .iter()
            .find(|span| matches!(span.style, HighlightStyle::Success));
        assert!(success_span.is_some());
    }

    /// Test JSON highlighting
    #[test]
    fn test_json_highlighting() {
        let config = StreamConfig::default();
        let formatter = OutputFormatter::new(config);

        let entry = OutputEntry::new(
            "agent-123".to_string(),
            "Test".to_string(),
            OutputType::Info,
            r#"{"status": "ok", "count": 42}"#.to_string(),
            None,
            "session-1".to_string(),
        );

        let formatted = formatter.format_entry(&entry);
        let json_span = formatted
            .highlight_spans
            .iter()
            .find(|span| matches!(span.style, HighlightStyle::Json));
        assert!(json_span.is_some());
    }

    /// Test streaming manager creation
    #[tokio::test]
    async fn test_streaming_manager() {
        let monitoring = Arc::new(MonitoringSystem::new());
        let config = StreamConfig::default();
        let manager = StreamingManager::new(monitoring.clone(), config);

        // Subscribe to stream first
        let mut rx = manager.subscribe("test-sub".to_string()).unwrap();

        // Start the manager
        manager.start().await.unwrap();

        // Give the streaming loop time to start
        sleep(Duration::from_millis(10)).await;

        // Add output to monitoring system
        monitoring
            .add_output(
                "agent-123".to_string(),
                "Test".to_string(),
                OutputType::Info,
                "Test message".to_string(),
                None,
                "session-1".to_string(),
            )
            .unwrap();

        // Wait for message with longer timeout
        let timeout = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;

        assert!(timeout.is_ok());
        let entry = timeout.unwrap();
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().content, "Test message");

        // Unsubscribe
        manager.unsubscribe("test-sub").unwrap();
    }

    /// Test agent-specific subscription
    #[tokio::test]
    async fn test_agent_subscription() {
        let monitoring = Arc::new(MonitoringSystem::new());
        let config = StreamConfig::default();
        let manager = StreamingManager::new(monitoring.clone(), config);

        // Subscribe to specific agent first
        let mut rx = manager
            .subscribe_agent("test-sub".to_string(), "agent-123".to_string())
            .unwrap();

        manager.start().await.unwrap();

        // Give the streaming loop time to start
        sleep(Duration::from_millis(10)).await;

        // Add outputs
        monitoring
            .add_output(
                "agent-123".to_string(),
                "Test".to_string(),
                OutputType::Info,
                "Message from agent-123".to_string(),
                None,
                "session-1".to_string(),
            )
            .unwrap();

        monitoring
            .add_output(
                "agent-456".to_string(),
                "Test".to_string(),
                OutputType::Info,
                "Message from agent-456".to_string(),
                None,
                "session-1".to_string(),
            )
            .unwrap();

        // Should only receive from agent-123
        let timeout = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;

        assert!(timeout.is_ok());
        let entry = timeout.unwrap();
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.agent_id, "agent-123");
        assert_eq!(entry.content, "Message from agent-123");

        // Should not receive any more messages
        let timeout = tokio::time::timeout(Duration::from_millis(50), rx.recv()).await;

        assert!(timeout.is_err()); // Timeout
    }

    /// Test filtered subscription
    #[tokio::test]
    async fn test_filtered_subscription() {
        let monitoring = Arc::new(MonitoringSystem::new());
        let config = StreamConfig::default();
        let manager = StreamingManager::new(monitoring.clone(), config);

        // Subscribe with error filter first
        let filter = OutputFilter {
            output_types: Some(vec![OutputType::Error]),
            ..Default::default()
        };
        let mut rx = manager
            .subscribe_filtered("test-sub".to_string(), filter)
            .unwrap();

        manager.start().await.unwrap();

        // Give the streaming loop time to start
        sleep(Duration::from_millis(10)).await;

        // Add various outputs
        monitoring
            .add_output(
                "agent-123".to_string(),
                "Test".to_string(),
                OutputType::Info,
                "Info message".to_string(),
                None,
                "session-1".to_string(),
            )
            .unwrap();

        monitoring
            .add_output(
                "agent-123".to_string(),
                "Test".to_string(),
                OutputType::Error,
                "Error message".to_string(),
                None,
                "session-1".to_string(),
            )
            .unwrap();

        // Should only receive error
        let timeout = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;

        assert!(timeout.is_ok());
        let entry = timeout.unwrap();
        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.output_type, OutputType::Error);
        assert_eq!(entry.content, "Error message");
    }

    /// Test statistics tracking
    #[tokio::test]
    async fn test_streaming_stats() {
        let monitoring = Arc::new(MonitoringSystem::new());
        let config = StreamConfig::default();
        let manager = StreamingManager::new(monitoring.clone(), config);

        manager.start().await.unwrap();

        // Create subscriptions
        let _rx1 = manager.subscribe("sub-1".to_string()).unwrap();
        let _rx2 = manager.subscribe("sub-2".to_string()).unwrap();

        // Wait for stats updater to run (runs every second)
        sleep(Duration::from_millis(1100)).await;

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.active_subscriptions, 2);
        assert_eq!(stats.subscription_details.len(), 2);

        // Unsubscribe one
        manager.unsubscribe("sub-1").unwrap();

        // Wait for stats updater to run again
        sleep(Duration::from_millis(1100)).await;

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.active_subscriptions, 1);
    }

    /// Test clear subscriptions
    #[tokio::test]
    async fn test_clear_subscriptions() {
        let monitoring = Arc::new(MonitoringSystem::new());
        let config = StreamConfig::default();
        let manager = StreamingManager::new(monitoring.clone(), config);

        manager.start().await.unwrap();

        // Create multiple subscriptions
        let _rx1 = manager.subscribe("sub-1".to_string()).unwrap();
        let _rx2 = manager.subscribe("sub-2".to_string()).unwrap();
        let _rx3 = manager.subscribe("sub-3".to_string()).unwrap();

        // Wait for stats updater to run
        sleep(Duration::from_millis(1100)).await;

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.active_subscriptions, 3);

        // Clear all
        manager.clear_subscriptions().unwrap();

        // Wait for stats updater to run again
        sleep(Duration::from_millis(1100)).await;

        let stats = manager.get_stats().unwrap();
        assert_eq!(stats.active_subscriptions, 0);
    }

    /// Test multiple subscribers receiving same message
    #[tokio::test]
    async fn test_multiple_subscribers() {
        let monitoring = Arc::new(MonitoringSystem::new());
        let config = StreamConfig::default();
        let manager = StreamingManager::new(monitoring.clone(), config);

        // Create multiple subscribers first
        let mut rx1 = manager.subscribe("sub-1".to_string()).unwrap();
        let mut rx2 = manager.subscribe("sub-2".to_string()).unwrap();

        manager.start().await.unwrap();

        // Give the streaming loop time to start
        sleep(Duration::from_millis(10)).await;

        // Send a message
        monitoring
            .add_output(
                "agent-123".to_string(),
                "Test".to_string(),
                OutputType::Info,
                "Broadcast message".to_string(),
                None,
                "session-1".to_string(),
            )
            .unwrap();

        // Both should receive it
        let timeout1 = tokio::time::timeout(Duration::from_millis(100), rx1.recv()).await;

        let timeout2 = tokio::time::timeout(Duration::from_millis(100), rx2.recv()).await;

        assert!(timeout1.is_ok());
        assert!(timeout2.is_ok());

        let entry1 = timeout1.unwrap();
        let entry2 = timeout2.unwrap();

        assert!(entry1.is_some());
        assert!(entry2.is_some());

        assert_eq!(entry1.unwrap().content, "Broadcast message");
        assert_eq!(entry2.unwrap().content, "Broadcast message");
    }

    /// Test formatting edge cases
    #[test]
    fn test_formatting_edge_cases() {
        let config = StreamConfig::default();
        let formatter = OutputFormatter::new(config);

        // Test empty content
        let entry = OutputEntry::new(
            "agent-123".to_string(),
            "Test".to_string(),
            OutputType::Info,
            "".to_string(),
            None,
            "session-1".to_string(),
        );

        let formatted = formatter.format_entry(&entry);
        assert_eq!(formatted.formatted_content, "");

        // Test content with tabs
        let entry = OutputEntry::new(
            "agent-123".to_string(),
            "Test".to_string(),
            OutputType::Info,
            "Line\twith\ttabs".to_string(),
            None,
            "session-1".to_string(),
        );

        let formatted = formatter.format_entry(&entry);
        assert_eq!(formatted.formatted_content, "Line    with    tabs");

        // Test very short agent ID
        let entry = OutputEntry::new(
            "a".to_string(),
            "Test".to_string(),
            OutputType::Info,
            "Test".to_string(),
            None,
            "session-1".to_string(),
        );

        let formatted = formatter.format_entry(&entry);
        assert!(formatted.display_prefix.contains(" a"));
    }
}
