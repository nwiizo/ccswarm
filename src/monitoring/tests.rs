#[cfg(test)]
mod tests {
    use crate::monitoring::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tokio::time::{sleep, Duration as TokioDuration};

    /// Test output entry creation and filtering
    #[test]
    fn test_output_entry_creation() {
        let entry = OutputEntry::new(
            "agent-123".to_string(),
            "Frontend".to_string(),
            OutputType::Info,
            "Test message".to_string(),
            Some("task-456".to_string()),
            "session-789".to_string(),
        );

        assert_eq!(entry.agent_id, "agent-123");
        assert_eq!(entry.agent_type, "Frontend");
        assert_eq!(entry.output_type, OutputType::Info);
        assert_eq!(entry.content, "Test message");
        assert_eq!(entry.task_id, Some("task-456".to_string()));
        assert_eq!(entry.session_id, "session-789");
    }

    /// Test output entry filtering
    #[test]
    fn test_output_entry_filtering() {
        let entry = OutputEntry::new(
            "agent-123".to_string(),
            "Backend".to_string(),
            OutputType::Error,
            "Error occurred in API".to_string(),
            Some("task-456".to_string()),
            "session-789".to_string(),
        );

        // Test agent ID filter
        let filter = OutputFilter {
            agent_ids: Some(vec!["agent-123".to_string()]),
            ..Default::default()
        };
        assert!(entry.matches_filter(&filter));

        let filter = OutputFilter {
            agent_ids: Some(vec!["agent-999".to_string()]),
            ..Default::default()
        };
        assert!(!entry.matches_filter(&filter));

        // Test output type filter
        let filter = OutputFilter {
            output_types: Some(vec![OutputType::Error, OutputType::Warning]),
            ..Default::default()
        };
        assert!(entry.matches_filter(&filter));

        let filter = OutputFilter {
            output_types: Some(vec![OutputType::Info]),
            ..Default::default()
        };
        assert!(!entry.matches_filter(&filter));

        // Test content pattern filter
        let filter = OutputFilter {
            content_pattern: Some("API".to_string()),
            ..Default::default()
        };
        assert!(entry.matches_filter(&filter));

        let filter = OutputFilter {
            content_pattern: Some("database".to_string()),
            ..Default::default()
        };
        assert!(!entry.matches_filter(&filter));

        // Test task ID filter
        let filter = OutputFilter {
            task_id: Some("task-456".to_string()),
            ..Default::default()
        };
        assert!(entry.matches_filter(&filter));

        let filter = OutputFilter {
            task_id: Some("task-999".to_string()),
            ..Default::default()
        };
        assert!(!entry.matches_filter(&filter));

        // Test combined filters
        let filter = OutputFilter {
            agent_ids: Some(vec!["agent-123".to_string()]),
            output_types: Some(vec![OutputType::Error]),
            content_pattern: Some("api".to_string()), // Case insensitive
            ..Default::default()
        };
        assert!(entry.matches_filter(&filter));
    }

    /// Test agent output stream
    #[test]
    fn test_agent_output_stream() {
        let stream = AgentOutputStream::new("test-agent".to_string());

        // Test adding entries
        for i in 0..5 {
            let entry = OutputEntry::new(
                "test-agent".to_string(),
                "Test".to_string(),
                OutputType::Info,
                format!("Message {}", i),
                None,
                "session-1".to_string(),
            );
            stream.add_output(entry).unwrap();
        }

        assert_eq!(stream.len(), 5);
        assert!(!stream.is_empty());

        // Test getting recent entries
        let recent = stream.get_recent(3, None);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].content, "Message 2");
        assert_eq!(recent[1].content, "Message 3");
        assert_eq!(recent[2].content, "Message 4");

        // Test with filter
        let filter = OutputFilter {
            content_pattern: Some("3".to_string()),
            ..Default::default()
        };
        let filtered = stream.get_recent(10, Some(&filter));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].content, "Message 3");

        // Test clearing
        stream.clear();
        assert_eq!(stream.len(), 0);
        assert!(stream.is_empty());
    }

    /// Test buffer size limit
    #[test]
    fn test_buffer_size_limit() {
        let stream = AgentOutputStream::new("test-agent".to_string());

        // Add more entries than MAX_BUFFER_SIZE
        for i in 0..MAX_BUFFER_SIZE + 100 {
            let entry = OutputEntry::new(
                "test-agent".to_string(),
                "Test".to_string(),
                OutputType::Info,
                format!("Message {}", i),
                None,
                "session-1".to_string(),
            );
            stream.add_output(entry).unwrap();
        }

        // Verify buffer doesn't exceed max size
        assert_eq!(stream.len(), MAX_BUFFER_SIZE);
    }

    /// Test monitoring system
    #[test]
    fn test_monitoring_system() {
        let system = MonitoringSystem::new();

        // Register agents
        let stream1 = system.register_agent("agent-1".to_string()).unwrap();
        let stream2 = system.register_agent("agent-2".to_string()).unwrap();

        // Add outputs through system
        system
            .add_output(
                "agent-1".to_string(),
                "Frontend".to_string(),
                OutputType::Info,
                "Frontend message".to_string(),
                None,
                "session-1".to_string(),
            )
            .unwrap();

        system
            .add_output(
                "agent-2".to_string(),
                "Backend".to_string(),
                OutputType::Error,
                "Backend error".to_string(),
                Some("task-123".to_string()),
                "session-2".to_string(),
            )
            .unwrap();

        // Verify streams received outputs
        assert_eq!(stream1.len(), 1);
        assert_eq!(stream2.len(), 1);

        // Test getting all recent
        let all_recent = system.get_all_recent(10, None);
        assert_eq!(all_recent.len(), 2);

        // Test with filter
        let filter = OutputFilter {
            output_types: Some(vec![OutputType::Error]),
            ..Default::default()
        };
        let filtered = system.get_all_recent(10, Some(&filter));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].agent_id, "agent-2");

        // Test stats
        let stats = system.get_stats();
        assert_eq!(stats.total_entries, 2);
        assert_eq!(stats.active_streams, 2);
        assert_eq!(stats.entries_per_agent.get("agent-1"), Some(&1));
        assert_eq!(stats.entries_per_agent.get("agent-2"), Some(&1));

        // Test clearing
        system.clear_agent_output("agent-1").unwrap();
        assert_eq!(stream1.len(), 0);
        assert_eq!(stream2.len(), 1);

        system.clear_all_output().unwrap();
        assert_eq!(stream1.len(), 0);
        assert_eq!(stream2.len(), 0);

        // Test unregistering
        system.unregister_agent("agent-1").unwrap();
        let stats = system.get_stats();
        assert_eq!(stats.active_streams, 1);
    }

    /// Test custom output subscriber
    struct TestSubscriber {
        id: String,
        received: Arc<AtomicUsize>,
        filter_type: Option<OutputType>,
    }

    impl OutputSubscriber for TestSubscriber {
        fn on_output(&self, entry: &OutputEntry) {
            if self.filter_type.is_none() || self.filter_type == Some(entry.output_type.clone()) {
                self.received.fetch_add(1, Ordering::SeqCst);
            }
        }

        fn id(&self) -> &str {
            &self.id
        }

        fn accepts(&self, entry: &OutputEntry) -> bool {
            self.filter_type.is_none() || self.filter_type == Some(entry.output_type.clone())
        }
    }

    #[test]
    fn test_output_subscribers() {
        let system = MonitoringSystem::new();

        // Create test subscribers
        let subscriber1 = Arc::new(TestSubscriber {
            id: "sub-1".to_string(),
            received: Arc::new(AtomicUsize::new(0)),
            filter_type: None,
        });

        let subscriber2 = Arc::new(TestSubscriber {
            id: "sub-2".to_string(),
            received: Arc::new(AtomicUsize::new(0)),
            filter_type: Some(OutputType::Error),
        });

        // Add subscribers
        system.add_subscriber(subscriber1.clone()).unwrap();
        system.add_subscriber(subscriber2.clone()).unwrap();

        // Send outputs
        system
            .add_output(
                "agent-1".to_string(),
                "Test".to_string(),
                OutputType::Info,
                "Info message".to_string(),
                None,
                "session-1".to_string(),
            )
            .unwrap();

        system
            .add_output(
                "agent-1".to_string(),
                "Test".to_string(),
                OutputType::Error,
                "Error message".to_string(),
                None,
                "session-1".to_string(),
            )
            .unwrap();

        // Verify subscribers received correct outputs
        assert_eq!(subscriber1.received.load(Ordering::SeqCst), 2);
        assert_eq!(subscriber2.received.load(Ordering::SeqCst), 1);

        // Remove subscriber
        system.remove_subscriber("sub-1").unwrap();

        // Send another output
        system
            .add_output(
                "agent-1".to_string(),
                "Test".to_string(),
                OutputType::Error,
                "Another error".to_string(),
                None,
                "session-1".to_string(),
            )
            .unwrap();

        // Verify only subscriber2 received it
        assert_eq!(subscriber1.received.load(Ordering::SeqCst), 2);
        assert_eq!(subscriber2.received.load(Ordering::SeqCst), 2);
    }

    /// Test broadcast subscriptions
    #[tokio::test]
    async fn test_broadcast_subscriptions() {
        let system = Arc::new(MonitoringSystem::new());

        // Subscribe to global broadcasts
        let mut global_rx = system.subscribe_global();

        // Register agent and subscribe to its stream
        let stream = system.register_agent("test-agent".to_string()).unwrap();
        let mut agent_rx = stream.subscribe();

        // Clone system for the async task
        let system_clone = system.clone();

        // Spawn task to send outputs
        let sender = tokio::spawn(async move {
            sleep(TokioDuration::from_millis(5)).await;

            system_clone
                .add_output(
                    "test-agent".to_string(),
                    "Test".to_string(),
                    OutputType::Info,
                    "Broadcast test".to_string(),
                    None,
                    "session-1".to_string(),
                )
                .unwrap();
        });

        // Wait for outputs on both channels
        let global_result =
            tokio::time::timeout(TokioDuration::from_secs(1), global_rx.recv()).await;

        let agent_result = tokio::time::timeout(TokioDuration::from_secs(1), agent_rx.recv()).await;

        // Verify both received the output
        assert!(global_result.is_ok());
        assert!(agent_result.is_ok());

        let global_entry = global_result.unwrap().unwrap();
        let agent_entry = agent_result.unwrap().unwrap();

        assert_eq!(global_entry.content, "Broadcast test");
        assert_eq!(agent_entry.content, "Broadcast test");

        sender.await.unwrap();
    }

    /// Test console output subscriber
    #[test]
    fn test_console_output_subscriber() {
        let subscriber = ConsoleOutputSubscriber::new("console-1".to_string());
        assert_eq!(subscriber.id(), "console-1");

        let entry = OutputEntry::new(
            "agent-1".to_string(),
            "Test".to_string(),
            OutputType::Info,
            "Test message".to_string(),
            None,
            "session-1".to_string(),
        );

        // Without filter, accepts all
        assert!(subscriber.accepts(&entry));

        // With filter
        let filter = OutputFilter {
            output_types: Some(vec![OutputType::Error]),
            ..Default::default()
        };
        let subscriber = ConsoleOutputSubscriber::new("console-2".to_string()).with_filter(filter);

        assert!(!subscriber.accepts(&entry));

        let error_entry = OutputEntry::new(
            "agent-1".to_string(),
            "Test".to_string(),
            OutputType::Error,
            "Error message".to_string(),
            None,
            "session-1".to_string(),
        );

        assert!(subscriber.accepts(&error_entry));
    }

    /// Test auto-registration of agents
    #[test]
    fn test_auto_registration() {
        let system = MonitoringSystem::new();

        // Add output for non-existent agent
        system
            .add_output(
                "new-agent".to_string(),
                "Test".to_string(),
                OutputType::Info,
                "Auto-registered".to_string(),
                None,
                "session-1".to_string(),
            )
            .unwrap();

        // Verify agent was auto-registered
        let stream = system.get_agent_stream("new-agent");
        assert!(stream.is_some());
        assert_eq!(stream.unwrap().len(), 1);
    }

    /// Test thread safety
    #[test]
    fn test_thread_safety() {
        let system = Arc::new(MonitoringSystem::new());
        let mut handles = vec![];

        // Spawn multiple threads adding outputs
        for i in 0..10 {
            let system_clone = system.clone();
            let handle = std::thread::spawn(move || {
                for j in 0..100 {
                    system_clone
                        .add_output(
                            format!("agent-{}", i),
                            "Test".to_string(),
                            OutputType::Info,
                            format!("Message {}", j),
                            None,
                            "session-1".to_string(),
                        )
                        .unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all outputs were recorded
        let stats = system.get_stats();
        assert_eq!(stats.total_entries, 1000);
        assert_eq!(stats.active_streams, 10);
    }

    /// Test empty state
    #[test]
    fn test_empty_state() {
        let system = MonitoringSystem::new();

        // Test operations on empty system
        let stats = system.get_stats();
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.active_streams, 0);

        let recent = system.get_all_recent(10, None);
        assert!(recent.is_empty());

        // Clear operations should not fail
        assert!(system.clear_agent_output("non-existent").is_err());
        assert!(system.clear_all_output().is_ok());
    }
}
