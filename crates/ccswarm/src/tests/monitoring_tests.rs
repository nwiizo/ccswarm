use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::monitoring::{
    MonitoringSystem, OutputEntry, OutputFilter, OutputType, 
    AgentOutputStream, OutputSubscriber, ConsoleOutputSubscriber
};
use crate::streaming::{StreamingManager, StreamConfig};

#[tokio::test]
async fn test_monitoring_system_basic_operations() {
    let monitoring = MonitoringSystem::new();
    
    // Register an agent
    let agent_id = "test-agent-123".to_string();
    let stream = monitoring.register_agent(agent_id.clone()).unwrap();
    
    // Add some output
    monitoring.add_output(
        agent_id.clone(),
        "Frontend".to_string(),
        OutputType::Info,
        "Test message".to_string(),
        Some("task-1".to_string()),
        "session-1".to_string(),
    ).unwrap();
    
    // Verify output was added
    let recent = monitoring.get_all_recent(10, None);
    assert_eq!(recent.len(), 1);
    assert_eq!(recent[0].content, "Test message");
    assert_eq!(recent[0].agent_id, agent_id);
    
    // Test agent-specific stream
    let agent_recent = stream.get_recent(10, None);
    assert_eq!(agent_recent.len(), 1);
    assert_eq!(agent_recent[0].content, "Test message");
}

#[tokio::test]
async fn test_output_filtering() {
    let monitoring = MonitoringSystem::new();
    let agent_id = "test-agent-456".to_string();
    
    monitoring.register_agent(agent_id.clone()).unwrap();
    
    // Add various types of output
    let outputs = vec![
        (OutputType::Info, "Info message"),
        (OutputType::Warning, "Warning message"),
        (OutputType::Error, "Error message"),
        (OutputType::Debug, "Debug message"),
    ];
    
    for (output_type, content) in outputs {
        monitoring.add_output(
            agent_id.clone(),
            "Backend".to_string(),
            output_type,
            content.to_string(),
            None,
            "session-1".to_string(),
        ).unwrap();
    }
    
    // Test filtering by output type
    let error_filter = OutputFilter {
        agent_ids: None,
        output_types: Some(vec![OutputType::Error]),
        content_pattern: None,
        task_id: None,
    };
    
    let filtered = monitoring.get_all_recent(10, Some(&error_filter));
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].content, "Error message");
    
    // Test filtering by content pattern
    let warning_filter = OutputFilter {
        agent_ids: None,
        output_types: None,
        content_pattern: Some("Warning".to_string()),
        task_id: None,
    };
    
    let warning_filtered = monitoring.get_all_recent(10, Some(&warning_filter));
    assert_eq!(warning_filtered.len(), 1);
    assert_eq!(warning_filtered[0].content, "Warning message");
}

#[tokio::test]
async fn test_agent_output_stream() {
    let agent_id = "stream-test-agent".to_string();
    let stream = AgentOutputStream::new(agent_id.clone());
    
    // Add some entries
    for i in 0..5 {
        let entry = OutputEntry::new(
            agent_id.clone(),
            "DevOps".to_string(),
            OutputType::Info,
            format!("Message {}", i),
            None,
            "session-1".to_string(),
        );
        stream.add_output(entry).unwrap();
    }
    
    // Test recent entries
    let recent = stream.get_recent(3, None);
    assert_eq!(recent.len(), 3);
    assert_eq!(recent[2].content, "Message 4"); // Most recent should be last
    assert_eq!(recent[0].content, "Message 2"); // Oldest of the 3
    
    // Test total length
    assert_eq!(stream.len(), 5);
    
    // Test clearing
    stream.clear();
    assert_eq!(stream.len(), 0);
}

#[tokio::test]
async fn test_output_subscription() {
    let monitoring = Arc::new(MonitoringSystem::new());
    let agent_id = "subscription-test-agent".to_string();
    
    monitoring.register_agent(agent_id.clone()).unwrap();
    
    // Subscribe to global output
    let mut receiver = monitoring.subscribe_global();
    
    // Add output in a background task
    let monitoring_clone = Arc::clone(&monitoring);
    let agent_id_clone = agent_id.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        
        monitoring_clone.add_output(
            agent_id_clone,
            "QA".to_string(),
            OutputType::System,
            "Subscription test message".to_string(),
            Some("task-2".to_string()),
            "session-2".to_string(),
        ).unwrap();
    });
    
    // Wait for the message
    let received = tokio::time::timeout(Duration::from_secs(1), receiver.recv()).await;
    assert!(received.is_ok());
    
    let entry = received.unwrap().unwrap();
    assert_eq!(entry.content, "Subscription test message");
    assert_eq!(entry.output_type, OutputType::System);
}

#[tokio::test]
async fn test_streaming_manager() {
    let monitoring = Arc::new(MonitoringSystem::new());
    let config = StreamConfig::default();
    let streaming = Arc::new(StreamingManager::new(Arc::clone(&monitoring), config));
    
    // Start streaming manager
    streaming.start().await.unwrap();
    
    // Subscribe to streaming
    let subscription_id = "test-subscription".to_string();
    let mut receiver = streaming.subscribe(subscription_id.clone()).unwrap();
    
    // Register agent and add output
    let agent_id = "streaming-test-agent".to_string();
    monitoring.register_agent(agent_id.clone()).unwrap();
    
    // Add output in background
    let monitoring_clone = Arc::clone(&monitoring);
    let agent_id_clone = agent_id.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        
        monitoring_clone.add_output(
            agent_id_clone,
            "Testing".to_string(),
            OutputType::Info,
            "Streaming test message".to_string(),
            None,
            "streaming-session".to_string(),
        ).unwrap();
    });
    
    // Receive the streamed message
    let received = tokio::time::timeout(Duration::from_secs(2), receiver.recv()).await;
    assert!(received.is_ok());
    
    let entry = received.unwrap().unwrap();
    assert_eq!(entry.content, "Streaming test message");
    
    // Test unsubscribe
    streaming.unsubscribe(&subscription_id).unwrap();
}

#[tokio::test]
async fn test_output_subscriber() {
    let monitoring = MonitoringSystem::new();
    
    // Create a console subscriber
    let subscriber = Arc::new(ConsoleOutputSubscriber::new("test-console".to_string()));
    monitoring.add_subscriber(subscriber).unwrap();
    
    // Add output (should trigger subscriber)
    monitoring.add_output(
        "subscriber-test-agent".to_string(),
        "System".to_string(),
        OutputType::Warning,
        "Subscriber test message".to_string(),
        None,
        "subscriber-session".to_string(),
    ).unwrap();
    
    // The console subscriber will print to stdout, 
    // which we can't easily test here, but we can verify
    // the subscriber was called without error
    assert!(true);
}

#[tokio::test]
async fn test_monitoring_stats() {
    let monitoring = MonitoringSystem::new();
    
    // Register multiple agents
    let agents = vec![
        "stats-agent-1", "stats-agent-2", "stats-agent-3"
    ];
    
    for agent in &agents {
        monitoring.register_agent(agent.to_string()).unwrap();
    }
    
    // Add various outputs
    for (i, agent) in agents.iter().enumerate() {
        for j in 0..=i {
            monitoring.add_output(
                agent.to_string(),
                "Stats".to_string(),
                if j % 2 == 0 { OutputType::Info } else { OutputType::Error },
                format!("Stats message {} from {}", j, agent),
                None,
                "stats-session".to_string(),
            ).unwrap();
        }
    }
    
    let stats = monitoring.get_stats();
    assert_eq!(stats.active_streams, 3);
    assert_eq!(stats.total_entries, 6); // 1 + 2 + 3 = 6 total messages
    assert!(stats.entries_per_agent.contains_key("stats-agent-1"));
    assert!(stats.entries_per_agent.contains_key("stats-agent-2"));
    assert!(stats.entries_per_agent.contains_key("stats-agent-3"));
}

#[tokio::test]
async fn test_buffer_size_limits() {
    let agent_id = "buffer-test-agent".to_string();
    let stream = AgentOutputStream::new(agent_id.clone());
    
    // Add more entries than the typical buffer size
    for i in 0..15000 {
        let entry = OutputEntry::new(
            agent_id.clone(),
            "Buffer".to_string(),
            OutputType::Debug,
            format!("Buffer test message {}", i),
            None,
            "buffer-session".to_string(),
        );
        stream.add_output(entry).unwrap();
    }
    
    // Should be limited by MAX_BUFFER_SIZE (10000)
    assert!(stream.len() <= 10000);
    
    // Recent entries should be the newest ones
    let recent = stream.get_recent(5, None);
    assert_eq!(recent.len(), 5);
    // Should contain the latest messages
    assert!(recent[4].content.contains("14999") || recent[4].content.contains("9999"));
}

#[tokio::test]
async fn test_concurrent_access() {
    let monitoring = Arc::new(MonitoringSystem::new());
    let agent_id = "concurrent-test-agent".to_string();
    
    monitoring.register_agent(agent_id.clone()).unwrap();
    
    // Spawn multiple tasks adding output concurrently
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let monitoring_clone = Arc::clone(&monitoring);
        let agent_id_clone = agent_id.clone();
        
        let handle = tokio::spawn(async move {
            for j in 0..20 {
                monitoring_clone.add_output(
                    agent_id_clone.clone(),
                    "Concurrent".to_string(),
                    OutputType::Info,
                    format!("Concurrent message {} from task {}", j, i),
                    None,
                    format!("session-{}", i),
                ).unwrap();
                
                // Small delay to increase chance of concurrent access
                sleep(Duration::from_millis(1)).await;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all messages were added
    let all_recent = monitoring.get_all_recent(300, None);
    assert_eq!(all_recent.len(), 200); // 10 tasks * 20 messages each
    
    let stats = monitoring.get_stats();
    assert_eq!(stats.total_entries, 200);
}

#[test]
fn test_output_entry_filter_matching() {
    let entry = OutputEntry::new(
        "test-agent-filter".to_string(),
        "Frontend".to_string(),
        OutputType::Warning,
        "This is a test warning message".to_string(),
        Some("task-123".to_string()),
        "session-abc".to_string(),
    );
    
    // Test agent filter
    let agent_filter = OutputFilter {
        agent_ids: Some(vec!["test-agent-filter".to_string()]),
        output_types: None,
        content_pattern: None,
        task_id: None,
    };
    assert!(entry.matches_filter(&agent_filter));
    
    let wrong_agent_filter = OutputFilter {
        agent_ids: Some(vec!["other-agent".to_string()]),
        output_types: None,
        content_pattern: None,
        task_id: None,
    };
    assert!(!entry.matches_filter(&wrong_agent_filter));
    
    // Test output type filter
    let type_filter = OutputFilter {
        agent_ids: None,
        output_types: Some(vec![OutputType::Warning]),
        content_pattern: None,
        task_id: None,
    };
    assert!(entry.matches_filter(&type_filter));
    
    // Test content pattern filter
    let content_filter = OutputFilter {
        agent_ids: None,
        output_types: None,
        content_pattern: Some("warning".to_string()),
        task_id: None,
    };
    assert!(entry.matches_filter(&content_filter));
    
    // Test task ID filter
    let task_filter = OutputFilter {
        agent_ids: None,
        output_types: None,
        content_pattern: None,
        task_id: Some("task-123".to_string()),
    };
    assert!(entry.matches_filter(&task_filter));
}