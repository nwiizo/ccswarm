# Resource Monitoring and Efficiency

## Overview

ccswarm now includes comprehensive resource monitoring and efficiency features to optimize system performance and reduce resource consumption. This system automatically tracks CPU and memory usage for each agent, suspends idle agents, and provides detailed efficiency statistics.

## Key Features

### 1. Real-time Resource Tracking
- **CPU Usage**: Monitor CPU percentage for each agent
- **Memory Usage**: Track memory consumption in bytes and percentage
- **Thread Count**: Monitor number of threads per agent
- **Historical Data**: Maintain last 100 measurements for trend analysis

### 2. Automatic Idle Agent Suspension
- **Idle Detection**: Agents with CPU usage below threshold are marked as idle
- **Configurable Timeout**: Default 15 minutes, customizable per deployment
- **Automatic Suspension**: Idle agents are paused to free resources
- **Quick Resume**: Suspended agents can be resumed instantly when needed

### 3. Resource Limits
- **CPU Limits**: Set maximum CPU percentage per agent (default: 80%)
- **Memory Limits**: Configure memory caps in bytes or percentage
- **Soft/Hard Limits**: Choose between warning or enforcement modes
- **Violation Tracking**: Monitor how often limits are exceeded

### 4. Efficiency Statistics
- **Suspension Rate**: Percentage of agents currently suspended
- **Resource Savings**: Calculate memory and CPU saved by suspensions
- **Average Usage**: Track average resource consumption across agents
- **Active vs Idle**: Monitor the ratio of working to idle agents

## Configuration

### Resource Limits Configuration

```rust
use ccswarm::resource::ResourceLimits;
use chrono::Duration;

let mut limits = ResourceLimits {
    max_cpu_percent: 80.0,                      // Max 80% CPU
    max_memory_bytes: 2 * 1024 * 1024 * 1024,   // 2GB max memory
    max_memory_percent: 50.0,                   // Max 50% of system memory
    idle_timeout: Duration::minutes(15),        // Suspend after 15 min idle
    idle_cpu_threshold: 5.0,                    // Below 5% CPU = idle
    auto_suspend_enabled: true,                 // Enable auto-suspension
    enforce_limits: false,                      // Soft limits (warn only)
};
```

### Creating Session Manager with Monitoring

```rust
use ccswarm::session::SessionManager;

// Create with default resource monitoring
let session_manager = SessionManager::with_resource_monitoring(
    ResourceLimits::default()
).await?;

// Create with custom limits
let session_manager = SessionManager::with_resource_monitoring(limits).await?;
```

## CLI Commands

### Resource Status
Show current resource usage for all agents:
```bash
ccswarm resource status
```

Output:
```
Resource Usage Status
================================================================================
Agent ID                       CPU %      Memory MB  Status          Idle Time
--------------------------------------------------------------------------------
frontend-agent-a1b2c3d4        12.5       245        Active          2m
backend-agent-e5f6g7h8         3.2        189        Paused          18m
devops-agent-i9j0k1l2          45.8       512        Active          0s
```

### Resource Statistics
View efficiency statistics:
```bash
ccswarm resource stats
```

Output:
```
Resource Efficiency Statistics
==================================================
Total Agents:      4
Active Agents:     2 (50.0%)
Suspended Agents:  2 (50.0%)

Average CPU Usage:    29.0%
Average Memory Usage: 378 MB (18.5%)
Total Memory Usage:   756 MB

💡 Efficiency Tip: 50.0% of agents are suspended, saving resources
```

### Configure Limits
Set resource limits:
```bash
# Set maximum CPU to 60%
ccswarm resource limits --max-cpu 60

# Set maximum memory to 1.5GB
ccswarm resource limits --max-memory-gb 1.5

# Set idle timeout to 30 minutes
ccswarm resource limits --idle-timeout-min 30

# Disable auto-suspension
ccswarm resource limits --auto-suspend false
```

### Manual Suspension Management
```bash
# Check and suspend idle agents manually
ccswarm resource check-idle

# Resume a specific agent
ccswarm resource resume frontend-agent-a1b2c3d4
```

## Programmatic API

### Monitor Resource Events

```rust
use ccswarm::resource::{ResourceMonitor, ResourceEvent};
use std::sync::Arc;

let monitor = Arc::new(ResourceMonitor::new(limits));
let mut event_rx = monitor.subscribe();

// Listen for resource events
tokio::spawn(async move {
    while let Ok(event) = event_rx.recv().await {
        match event {
            ResourceEvent::AgentSuspended { agent_id, reason } => {
                println!("Agent {} suspended: {}", agent_id, reason);
            }
            ResourceEvent::LimitExceeded { agent_id, resource_type, current_value, limit_value } => {
                println!("Agent {} exceeded {} limit: {:.1} > {:.1}", 
                    agent_id, resource_type, current_value, limit_value);
            }
            _ => {}
        }
    }
});
```

### Custom Resource Limits per Agent

```rust
// Start monitoring with custom limits for specific agent
let custom_limits = ResourceLimits {
    max_cpu_percent: 90.0,  // Higher limit for compute-intensive agent
    ..ResourceLimits::default()
};

monitor.start_monitoring(
    "compute-agent".to_string(),
    Some(process_id),
    Some(custom_limits)
)?;
```

### Query Resource Usage

```rust
// Get current usage for an agent
if let Some(usage) = monitor.get_agent_usage("frontend-agent") {
    println!("CPU: {:.1}%, Memory: {} MB", 
        usage.cpu_percent, 
        usage.memory_bytes / (1024 * 1024)
    );
}

// Get average usage over history
if let Some(state) = monitor.get_agent_state("frontend-agent") {
    let avg = state.get_average_usage();
    println!("Average CPU: {:.1}%", avg.cpu_percent);
}
```

## Best Practices

### 1. Idle Timeout Configuration
- **Development**: Use shorter timeouts (5-10 minutes) to save resources
- **Production**: Use longer timeouts (30-60 minutes) for better responsiveness
- **CI/CD**: Use very short timeouts (1-2 minutes) for ephemeral agents

### 2. Resource Limit Guidelines
- **CPU**: Set limits 10-20% below maximum to leave headroom
- **Memory**: Account for peak usage + 25% buffer
- **Enforcement**: Start with soft limits, enable hard limits after tuning

### 3. Monitoring Integration
- Subscribe to resource events for alerting
- Track suspension rates to optimize timeout settings
- Monitor limit violations to adjust thresholds

### 4. Performance Optimization
- Suspended agents consume zero CPU and minimal memory
- Resume latency is typically <100ms
- Resource monitoring adds <1% overhead

## Architecture

### Components

1. **ResourceMonitor**: Core monitoring engine
   - Tracks all agent resources
   - Manages suspension/resume
   - Emits resource events

2. **AgentResourceState**: Per-agent resource tracking
   - Current and historical usage
   - Idle detection logic
   - Limit violation tracking

3. **SessionResourceIntegration**: Session manager integration
   - Automatic monitoring start/stop
   - Session-based suspension
   - Unified resource queries

### Data Flow

```
Agent Process
    ↓
System Monitor (sysinfo)
    ↓
ResourceMonitor
    ↓
AgentResourceState (update)
    ↓
Idle Detection → Suspension Event
    ↓
SessionManager → Pause Session
```

## Troubleshooting

### Agent Not Suspending
1. Check idle timeout setting: `ccswarm resource limits`
2. Verify auto-suspend is enabled
3. Check CPU threshold - agent may not be truly idle
4. Look for periodic tasks keeping agent active

### High Resource Usage
1. Check for resource leaks in agent code
2. Review limit violations in monitoring
3. Consider increasing limits for legitimate usage
4. Enable hard limits to prevent runaway processes

### Monitoring Not Working
1. Ensure session manager created with monitoring enabled
2. Check process permissions for system monitoring
3. Verify sysinfo can access process information
4. Check logs for monitoring errors

## Future Enhancements

1. **Predictive Suspension**: Use ML to predict idle periods
2. **Dynamic Limits**: Adjust limits based on system load
3. **Resource Quotas**: Daily/weekly resource budgets
4. **Priority-based Scheduling**: Important agents get more resources
5. **Cluster-wide Monitoring**: Aggregate stats across multiple hosts