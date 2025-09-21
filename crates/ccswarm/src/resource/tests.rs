/// Tests for resource monitoring module

#[cfg(test)]
mod tests {
    use super::super::*;
    use chrono::Duration as ChronoDuration;
    use std::sync::Arc;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_cpu_percent, 80.0);
        assert_eq!(limits.max_memory_bytes, 2 * 1024 * 1024 * 1024); // 2GB
        assert_eq!(limits.max_memory_percent, 50.0);
        assert_eq!(limits.idle_timeout, ChronoDuration::minutes(15));
        assert_eq!(limits.idle_cpu_threshold, 5.0);
        assert!(limits.auto_suspend_enabled);
        assert!(!limits.enforce_limits);
    }

    #[test]
    fn test_agent_resource_state_new() {
        let limits = ResourceLimits::default();
        let state = AgentResourceState::new("test-agent".to_string(), Some(1234), limits);
        
        assert_eq!(state.agent_id, "test-agent");
        assert_eq!(state.pid, Some(1234));
        assert!(!state.is_suspended);
        assert_eq!(state.limit_violations, 0);
        assert_eq!(state.current_usage.cpu_percent, 0.0);
        assert_eq!(state.current_usage.memory_bytes, 0);
    }

    #[test]
    fn test_update_usage() {
        let limits = ResourceLimits::default();
        let mut state = AgentResourceState::new("test-agent".to_string(), None, limits);
        
        let usage = ResourceUsage {
            cpu_percent: 25.0,
            memory_bytes: 512 * 1024 * 1024, // 512MB
            memory_percent: 12.5,
            thread_count: 8,
            timestamp: Utc::now(),
        };
        
        state.update_usage(usage.clone());
        
        assert_eq!(state.current_usage.cpu_percent, 25.0);
        assert_eq!(state.current_usage.memory_bytes, 512 * 1024 * 1024);
        assert_eq!(state.usage_history.len(), 1);
        assert_eq!(state.limit_violations, 0); // Not enforcing limits
    }

    #[test]
    fn test_usage_history_limit() {
        let limits = ResourceLimits::default();
        let mut state = AgentResourceState::new("test-agent".to_string(), None, limits);
        
        // Add 150 usage entries
        for i in 0..150 {
            let usage = ResourceUsage {
                cpu_percent: i as f32,
                memory_bytes: 1024 * 1024 * i as u64,
                memory_percent: (i as f32) / 10.0,
                thread_count: 1,
                timestamp: Utc::now(),
            };
            state.update_usage(usage);
        }
        
        // Should only keep last 100
        assert_eq!(state.usage_history.len(), 100);
        assert_eq!(state.usage_history[0].cpu_percent, 50.0); // First entry should be #50
    }

    #[test]
    fn test_should_suspend_idle() {
        let mut limits = ResourceLimits::default();
        limits.idle_timeout = ChronoDuration::seconds(10); // Short timeout for testing
        
        let mut state = AgentResourceState::new("test-agent".to_string(), None, limits);
        
        // Initially should not suspend
        assert!(!state.should_suspend());
        
        // Make agent idle by setting last_active to past
        state.last_active = Utc::now() - ChronoDuration::seconds(20);
        assert!(state.should_suspend());
        
        // Suspended agents should not be suspended again
        state.is_suspended = true;
        assert!(!state.should_suspend());
    }

    #[test]
    fn test_should_suspend_disabled() {
        let mut limits = ResourceLimits::default();
        limits.auto_suspend_enabled = false;
        limits.idle_timeout = ChronoDuration::seconds(1);
        
        let mut state = AgentResourceState::new("test-agent".to_string(), None, limits);
        state.last_active = Utc::now() - ChronoDuration::seconds(10);
        
        // Should not suspend when disabled
        assert!(!state.should_suspend());
    }

    #[test]
    fn test_is_exceeding_limits() {
        let mut limits = ResourceLimits::default();
        limits.max_cpu_percent = 50.0;
        limits.max_memory_bytes = 1024 * 1024 * 1024; // 1GB
        limits.max_memory_percent = 25.0;
        
        let mut state = AgentResourceState::new("test-agent".to_string(), None, limits);
        
        // Not exceeding
        state.current_usage = ResourceUsage {
            cpu_percent: 30.0,
            memory_bytes: 512 * 1024 * 1024,
            memory_percent: 12.0,
            thread_count: 4,
            timestamp: Utc::now(),
        };
        assert!(!state.is_exceeding_limits());
        
        // Exceeding CPU
        state.current_usage.cpu_percent = 60.0;
        assert!(state.is_exceeding_limits());
        
        // Exceeding memory bytes
        state.current_usage.cpu_percent = 30.0;
        state.current_usage.memory_bytes = 2 * 1024 * 1024 * 1024;
        assert!(state.is_exceeding_limits());
        
        // Exceeding memory percent
        state.current_usage.memory_bytes = 512 * 1024 * 1024;
        state.current_usage.memory_percent = 30.0;
        assert!(state.is_exceeding_limits());
    }

    #[test]
    fn test_get_average_usage() {
        let limits = ResourceLimits::default();
        let mut state = AgentResourceState::new("test-agent".to_string(), None, limits);
        
        // Add some usage entries
        for i in 1..=5 {
            let usage = ResourceUsage {
                cpu_percent: i as f32 * 10.0,
                memory_bytes: i as u64 * 100 * 1024 * 1024,
                memory_percent: i as f32 * 5.0,
                thread_count: i,
                timestamp: Utc::now(),
            };
            state.update_usage(usage);
        }
        
        let avg = state.get_average_usage();
        assert_eq!(avg.cpu_percent, 30.0); // (10+20+30+40+50)/5
        assert_eq!(avg.memory_bytes, 300 * 1024 * 1024); // Average of 100-500MB
        assert_eq!(avg.memory_percent, 15.0); // (5+10+15+20+25)/5
        assert_eq!(avg.thread_count, 3); // (1+2+3+4+5)/5
    }

    #[test]
    fn test_limit_violations() {
        let mut limits = ResourceLimits::default();
        limits.max_cpu_percent = 50.0;
        limits.enforce_limits = true;
        
        let mut state = AgentResourceState::new("test-agent".to_string(), None, limits);
        
        // Normal usage
        let normal_usage = ResourceUsage {
            cpu_percent: 30.0,
            memory_bytes: 512 * 1024 * 1024,
            memory_percent: 12.5,
            thread_count: 4,
            timestamp: Utc::now(),
        };
        state.update_usage(normal_usage);
        assert_eq!(state.limit_violations, 0);
        
        // Exceeding usage
        let high_usage = ResourceUsage {
            cpu_percent: 75.0,
            memory_bytes: 512 * 1024 * 1024,
            memory_percent: 12.5,
            thread_count: 4,
            timestamp: Utc::now(),
        };
        state.update_usage(high_usage);
        assert_eq!(state.limit_violations, 1);
    }

    #[tokio::test]
    async fn test_resource_monitor_basic() {
        let limits = ResourceLimits::default();
        let monitor = Arc::new(ResourceMonitor::new(limits));
        
        // Start monitoring an agent
        monitor.start_monitoring("test-agent".to_string(), Some(1234), None).unwrap();
        
        // Check agent state exists
        let state = monitor.get_agent_state("test-agent");
        assert!(state.is_some());
        assert_eq!(state.unwrap().agent_id, "test-agent");
        
        // Stop monitoring
        monitor.stop_monitoring("test-agent").unwrap();
        let state = monitor.get_agent_state("test-agent");
        assert!(state.is_none());
    }

    #[tokio::test]
    async fn test_resource_monitor_events() {
        let limits = ResourceLimits::default();
        let monitor = Arc::new(ResourceMonitor::new(limits));
        
        // Subscribe to events
        let mut event_rx = monitor.subscribe();
        
        // Start monitoring should emit event
        monitor.start_monitoring("test-agent".to_string(), Some(1234), None).unwrap();
        
        // Check event
        let event = tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            event_rx.recv()
        ).await;
        
        assert!(event.is_ok());
        if let Ok(Ok(ResourceEvent::MonitoringStarted { agent_id, pid })) = event {
            assert_eq!(agent_id, "test-agent");
            assert_eq!(pid, Some(1234));
        } else {
            panic!("Expected MonitoringStarted event");
        }
    }

    #[test]
    fn test_efficiency_stats_empty() {
        let limits = ResourceLimits::default();
        let monitor = ResourceMonitor::new(limits);
        
        let stats = monitor.get_efficiency_stats();
        assert_eq!(stats.total_agents, 0);
        assert_eq!(stats.active_agents, 0);
        assert_eq!(stats.suspended_agents, 0);
        assert_eq!(stats.average_cpu_usage, 0.0);
        assert_eq!(stats.suspension_rate, 0.0);
    }

    #[test]
    fn test_efficiency_stats_with_agents() {
        let limits = ResourceLimits::default();
        let monitor = ResourceMonitor::new(limits);
        
        // Add some agents
        monitor.start_monitoring("agent1".to_string(), Some(1001), None).unwrap();
        monitor.start_monitoring("agent2".to_string(), Some(1002), None).unwrap();
        
        let stats = monitor.get_efficiency_stats();
        assert_eq!(stats.total_agents, 2);
        assert_eq!(stats.active_agents, 2);
        assert_eq!(stats.suspended_agents, 0);
    }
}