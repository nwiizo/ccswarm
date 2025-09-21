/// Resource monitoring and management for ccswarm agents
///
/// This module provides CPU and memory tracking for each agent,
/// automatic suspension of idle agents, and resource limit enforcement
/// to optimize system performance and efficiency.
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use sysinfo::{Pid, System};
use tokio::sync::broadcast;
use tokio::time::{interval, Duration as TokioDuration};

/// Resource usage snapshot for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage (0-100)
    pub cpu_percent: f32,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Memory usage percentage (0-100)
    pub memory_percent: f32,
    /// Number of threads
    pub thread_count: usize,
    /// Timestamp of this measurement
    pub timestamp: DateTime<Utc>,
}

/// Resource limits configuration for agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum CPU usage percentage before throttling
    pub max_cpu_percent: f32,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: u64,
    /// Maximum memory usage percentage
    pub max_memory_percent: f32,
    /// Duration of idle time before suspension
    pub idle_timeout: Duration,
    /// CPU usage threshold to consider agent as idle
    pub idle_cpu_threshold: f32,
    /// Whether to automatically suspend idle agents
    pub auto_suspend_enabled: bool,
    /// Whether to enforce hard resource limits
    pub enforce_limits: bool,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_cpu_percent: 80.0,
            max_memory_bytes: 2 * 1024 * 1024 * 1024, // 2GB
            max_memory_percent: 50.0,
            idle_timeout: Duration::minutes(15),
            idle_cpu_threshold: 5.0,
            auto_suspend_enabled: true,
            enforce_limits: false,
        }
    }
}

/// Agent resource state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResourceState {
    /// Agent ID
    pub agent_id: String,
    /// Current resource usage
    pub current_usage: ResourceUsage,
    /// Resource usage history (last 100 measurements)
    pub usage_history: Vec<ResourceUsage>,
    /// Whether the agent is currently suspended
    pub is_suspended: bool,
    /// Last activity timestamp
    pub last_active: DateTime<Utc>,
    /// Process ID if available
    pub pid: Option<u32>,
    /// Number of times resource limits were exceeded
    pub limit_violations: u32,
    /// Resource limits for this agent
    pub limits: ResourceLimits,
}

impl AgentResourceState {
    /// Create a new agent resource state
    pub fn new(agent_id: String, pid: Option<u32>, limits: ResourceLimits) -> Self {
        Self {
            agent_id,
            current_usage: ResourceUsage {
                cpu_percent: 0.0,
                memory_bytes: 0,
                memory_percent: 0.0,
                thread_count: 0,
                timestamp: Utc::now(),
            },
            usage_history: Vec::with_capacity(100),
            is_suspended: false,
            last_active: Utc::now(),
            pid,
            limit_violations: 0,
            limits,
        }
    }

    /// Update resource usage
    pub fn update_usage(&mut self, usage: ResourceUsage) {
        // Check if agent is active (not idle)
        if usage.cpu_percent > self.limits.idle_cpu_threshold {
            self.last_active = Utc::now();
        }

        // Check for limit violations
        if self.limits.enforce_limits
            && (usage.cpu_percent > self.limits.max_cpu_percent
                || usage.memory_bytes > self.limits.max_memory_bytes
                || usage.memory_percent > self.limits.max_memory_percent)
        {
            self.limit_violations += 1;
        }

        // Update current usage
        self.current_usage = usage.clone();

        // Add to history
        self.usage_history.push(usage);
        if self.usage_history.len() > 100 {
            self.usage_history.remove(0);
        }
    }

    /// Check if agent should be suspended due to idle timeout
    pub fn should_suspend(&self) -> bool {
        if !self.limits.auto_suspend_enabled || self.is_suspended {
            return false;
        }

        let idle_duration = Utc::now() - self.last_active;
        idle_duration > self.limits.idle_timeout
    }

    /// Check if agent is exceeding resource limits
    pub fn is_exceeding_limits(&self) -> bool {
        self.current_usage.cpu_percent > self.limits.max_cpu_percent
            || self.current_usage.memory_bytes > self.limits.max_memory_bytes
            || self.current_usage.memory_percent > self.limits.max_memory_percent
    }

    /// Get average resource usage over the history
    pub fn get_average_usage(&self) -> ResourceUsage {
        if self.usage_history.is_empty() {
            return self.current_usage.clone();
        }

        let count = self.usage_history.len() as f32;
        let (cpu_sum, mem_sum, mem_pct_sum, thread_sum) = self.usage_history.iter().fold(
            (0.0, 0u64, 0.0, 0usize),
            |(cpu, mem, mem_pct, threads), usage| {
                (
                    cpu + usage.cpu_percent,
                    mem + usage.memory_bytes,
                    mem_pct + usage.memory_percent,
                    threads + usage.thread_count,
                )
            },
        );

        ResourceUsage {
            cpu_percent: cpu_sum / count,
            memory_bytes: (mem_sum as f32 / count) as u64,
            memory_percent: mem_pct_sum / count,
            thread_count: (thread_sum as f32 / count) as usize,
            timestamp: Utc::now(),
        }
    }
}

/// Resource monitoring event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceEvent {
    /// Agent suspended due to idle timeout
    AgentSuspended { agent_id: String, reason: String },
    /// Agent resumed from suspension
    AgentResumed { agent_id: String },
    /// Resource limit exceeded
    LimitExceeded {
        agent_id: String,
        resource_type: String,
        current_value: f64,
        limit_value: f64,
    },
    /// Resource monitoring started for agent
    MonitoringStarted { agent_id: String, pid: Option<u32> },
    /// Resource monitoring stopped for agent
    MonitoringStopped { agent_id: String },
}

/// Resource monitor for tracking agent resource usage
pub struct ResourceMonitor {
    /// System info collector
    system: Arc<Mutex<System>>,
    /// Agent resource states
    agents: Arc<RwLock<HashMap<String, AgentResourceState>>>,
    /// Event broadcaster
    event_tx: broadcast::Sender<ResourceEvent>,
    /// Global resource limits (can be overridden per agent)
    global_limits: ResourceLimits,
    /// Monitoring interval
    monitor_interval: TokioDuration,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(global_limits: ResourceLimits) -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            system: Arc::new(Mutex::new(System::new_all())),
            agents: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            global_limits,
            monitor_interval: TokioDuration::from_secs(5),
        }
    }

    /// Start monitoring an agent
    pub fn start_monitoring(
        &self,
        agent_id: String,
        pid: Option<u32>,
        custom_limits: Option<ResourceLimits>,
    ) -> Result<()> {
        let limits = custom_limits.unwrap_or_else(|| self.global_limits.clone());
        let state = AgentResourceState::new(agent_id.clone(), pid, limits);

        let mut agents = self
            .agents
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
        agents.insert(agent_id.clone(), state);

        let _ = self
            .event_tx
            .send(ResourceEvent::MonitoringStarted { agent_id, pid });

        Ok(())
    }

    /// Stop monitoring an agent
    pub fn stop_monitoring(&self, agent_id: &str) -> Result<()> {
        let mut agents = self
            .agents
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
        agents.remove(agent_id);

        let _ = self.event_tx.send(ResourceEvent::MonitoringStopped {
            agent_id: agent_id.to_string(),
        });

        Ok(())
    }

    /// Update resource limits for an agent
    pub fn update_limits(&self, agent_id: &str, limits: ResourceLimits) -> Result<()> {
        let mut agents = self
            .agents
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
        if let Some(state) = agents.get_mut(agent_id) {
            state.limits = limits;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Agent not found: {}", agent_id))
        }
    }

    /// Get current resource usage for an agent
    pub fn get_agent_usage(&self, agent_id: &str) -> Option<ResourceUsage> {
        let agents = self
            .agents
            .read()
            .map_err(|e| {
                tracing::error!("Failed to acquire read lock: {}", e);
                e
            })
            .ok()?;
        agents
            .get(agent_id)
            .map(|state| state.current_usage.clone())
    }

    /// Get resource state for an agent
    pub fn get_agent_state(&self, agent_id: &str) -> Option<AgentResourceState> {
        let agents = self
            .agents
            .read()
            .map_err(|e| {
                tracing::error!("Failed to acquire read lock: {}", e);
                e
            })
            .ok()?;
        agents.get(agent_id).cloned()
    }

    /// Get all agent states
    pub fn get_all_states(&self) -> Vec<AgentResourceState> {
        self.agents
            .read()
            .map_err(|e| {
                tracing::error!("Failed to acquire read lock: {}", e);
                e
            })
            .ok()
            .map(|agents| agents.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Subscribe to resource events
    pub fn subscribe(&self) -> broadcast::Receiver<ResourceEvent> {
        self.event_tx.subscribe()
    }

    /// Start the monitoring loop
    pub async fn start_monitoring_loop(self: Arc<Self>) {
        let mut interval = interval(self.monitor_interval);

        loop {
            interval.tick().await;
            if let Err(e) = self.update_all_agents().await {
                tracing::error!("Error updating agent resources: {}", e);
            }
        }
    }

    /// Update resource usage for all agents
    async fn update_all_agents(&self) -> Result<()> {
        // Refresh system information
        {
            let mut system = self
                .system
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire system lock: {}", e))?;
            system.refresh_all();
        }

        // Get current agent states
        let agent_ids: Vec<String> = {
            let agents = self
                .agents
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))?;
            agents.keys().cloned().collect()
        };

        // Update each agent
        for agent_id in agent_ids {
            if let Err(e) = self.update_agent_resources(&agent_id).await {
                tracing::warn!("Failed to update resources for agent {}: {}", agent_id, e);
            }
        }

        Ok(())
    }

    /// Update resource usage for a specific agent
    async fn update_agent_resources(&self, agent_id: &str) -> Result<()> {
        let (pid, limits) = {
            let agents = self
                .agents
                .read()
                .map_err(|e| anyhow::anyhow!("Failed to acquire read lock: {}", e))?;
            let state = agents.get(agent_id).context("Agent not found")?;
            (state.pid, state.limits.clone())
        };

        // Get resource usage
        let usage = if let Some(pid) = pid {
            self.get_process_usage(pid)?
        } else {
            // If no PID, try to find it by agent name pattern
            self.find_agent_process_usage(agent_id)?
        };

        // Update agent state
        let mut should_suspend = false;
        let mut should_emit_limit_event = false;
        {
            let mut agents = self
                .agents
                .write()
                .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
            if let Some(state) = agents.get_mut(agent_id) {
                state.update_usage(usage.clone());

                // Check for suspension
                if state.should_suspend() {
                    should_suspend = true;
                    state.is_suspended = true;
                }

                // Check for limit violations
                if limits.enforce_limits && state.is_exceeding_limits() {
                    should_emit_limit_event = true;
                }
            }
        }

        // Emit events outside of lock
        if should_suspend {
            let _ = self.event_tx.send(ResourceEvent::AgentSuspended {
                agent_id: agent_id.to_string(),
                reason: "Idle timeout exceeded".to_string(),
            });
        }

        if should_emit_limit_event && usage.cpu_percent > limits.max_cpu_percent {
            let _ = self.event_tx.send(ResourceEvent::LimitExceeded {
                agent_id: agent_id.to_string(),
                resource_type: "CPU".to_string(),
                current_value: usage.cpu_percent as f64,
                limit_value: limits.max_cpu_percent as f64,
            });
        }
        if should_emit_limit_event && usage.memory_bytes > limits.max_memory_bytes {
            let _ = self.event_tx.send(ResourceEvent::LimitExceeded {
                agent_id: agent_id.to_string(),
                resource_type: "Memory".to_string(),
                current_value: usage.memory_bytes as f64,
                limit_value: limits.max_memory_bytes as f64,
            });
        }

        Ok(())
    }

    /// Get resource usage for a process by PID
    fn get_process_usage(&self, pid: u32) -> Result<ResourceUsage> {
        let system = self
            .system
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire system lock: {}", e))?;
        let pid = Pid::from(pid as usize);

        if let Some(process) = system.process(pid) {
            let total_memory = system.total_memory();
            let memory_bytes = process.memory();
            let memory_percent = if total_memory > 0 {
                (memory_bytes as f32 / total_memory as f32) * 100.0
            } else {
                0.0
            };

            Ok(ResourceUsage {
                cpu_percent: process.cpu_usage(),
                memory_bytes,
                memory_percent,
                thread_count: 1, // sysinfo doesn't provide thread count directly
                timestamp: Utc::now(),
            })
        } else {
            Err(anyhow::anyhow!("Process not found: {}", pid))
        }
    }

    /// Find agent process by name pattern and get its usage
    fn find_agent_process_usage(&self, _agent_id: &str) -> Result<ResourceUsage> {
        let system = self
            .system
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire system lock: {}", e))?;

        // Look for processes that might be the agent
        // Pattern: ccswarm processes with agent ID in command line
        for process in system.processes().values() {
            // Try to get process name and cmd
            if let Some(name) = process.name().to_str() {
                if name.contains("ccswarm")
                    || process
                        .exe()
                        .map(|exe| exe.to_string_lossy().contains("ccswarm"))
                        .unwrap_or(false)
                {
                    // Check if this might be our agent by looking at the process
                    let total_memory = system.total_memory();
                    let memory_bytes = process.memory();
                    let memory_percent = if total_memory > 0 {
                        (memory_bytes as f32 / total_memory as f32) * 100.0
                    } else {
                        0.0
                    };

                    return Ok(ResourceUsage {
                        cpu_percent: process.cpu_usage(),
                        memory_bytes,
                        memory_percent,
                        thread_count: 1,
                        timestamp: Utc::now(),
                    });
                }
            }
        }

        // If no process found, return minimal usage
        Ok(ResourceUsage {
            cpu_percent: 0.0,
            memory_bytes: 0,
            memory_percent: 0.0,
            thread_count: 0,
            timestamp: Utc::now(),
        })
    }

    /// Resume a suspended agent
    pub async fn resume_agent(&self, agent_id: &str) -> Result<()> {
        let mut agents = self
            .agents
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire write lock: {}", e))?;
        if let Some(state) = agents.get_mut(agent_id) {
            if state.is_suspended {
                state.is_suspended = false;
                state.last_active = Utc::now();

                let _ = self.event_tx.send(ResourceEvent::AgentResumed {
                    agent_id: agent_id.to_string(),
                });
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Agent not found: {}", agent_id))
        }
    }

    /// Get resource efficiency statistics
    pub fn get_efficiency_stats(&self) -> ResourceEfficiencyStats {
        let agents_guard = self.agents.read().map_err(|e| {
            tracing::error!("Failed to acquire read lock: {}", e);
            e
        });

        if let Ok(agents) = agents_guard {
            let total_agents = agents.len();
            let suspended_agents = agents.values().filter(|s| s.is_suspended).count();
            let active_agents = total_agents - suspended_agents;

            let (total_cpu, total_memory, total_memory_pct) = agents
                .values()
                .filter(|s| !s.is_suspended)
                .fold((0.0, 0u64, 0.0), |(cpu, mem, mem_pct), state| {
                    (
                        cpu + state.current_usage.cpu_percent,
                        mem + state.current_usage.memory_bytes,
                        mem_pct + state.current_usage.memory_percent,
                    )
                });

            let avg_cpu = if active_agents > 0 {
                total_cpu / active_agents as f32
            } else {
                0.0
            };

            let avg_memory = if active_agents > 0 {
                total_memory / active_agents as u64
            } else {
                0
            };

            let avg_memory_pct = if active_agents > 0 {
                total_memory_pct / active_agents as f32
            } else {
                0.0
            };

            ResourceEfficiencyStats {
                total_agents,
                active_agents,
                suspended_agents,
                average_cpu_usage: avg_cpu,
                average_memory_usage: avg_memory,
                average_memory_percent: avg_memory_pct,
                total_memory_usage: total_memory,
                suspension_rate: if total_agents > 0 {
                    (suspended_agents as f32 / total_agents as f32) * 100.0
                } else {
                    0.0
                },
            }
        } else {
            // Return default stats if lock acquisition failed
            ResourceEfficiencyStats {
                total_agents: 0,
                active_agents: 0,
                suspended_agents: 0,
                average_cpu_usage: 0.0,
                average_memory_usage: 0,
                average_memory_percent: 0.0,
                total_memory_usage: 0,
                suspension_rate: 0.0,
            }
        }
    }
}

/// Resource efficiency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEfficiencyStats {
    pub total_agents: usize,
    pub active_agents: usize,
    pub suspended_agents: usize,
    pub average_cpu_usage: f32,
    pub average_memory_usage: u64,
    pub average_memory_percent: f32,
    pub total_memory_usage: u64,
    pub suspension_rate: f32,
}

/// Integration with session management
pub struct SessionResourceIntegration {
    monitor: Arc<ResourceMonitor>,
}

impl SessionResourceIntegration {
    /// Create a new session resource integration
    pub fn new(monitor: Arc<ResourceMonitor>) -> Self {
        Self { monitor }
    }

    /// Handle session creation - start monitoring
    pub async fn on_session_created(
        &self,
        session_id: &str,
        agent_id: &str,
        pid: Option<u32>,
    ) -> Result<()> {
        self.monitor.start_monitoring(
            agent_id.to_string(),
            pid,
            None, // Use global limits
        )?;

        tracing::info!(
            "Started resource monitoring for agent {} in session {}",
            agent_id,
            session_id
        );

        Ok(())
    }

    /// Handle session termination - stop monitoring
    pub async fn on_session_terminated(&self, session_id: &str, agent_id: &str) -> Result<()> {
        self.monitor.stop_monitoring(agent_id)?;

        tracing::info!(
            "Stopped resource monitoring for agent {} in session {}",
            agent_id,
            session_id
        );

        Ok(())
    }

    /// Check if an agent should be suspended
    pub async fn check_agent_suspension(&self, agent_id: &str) -> bool {
        if let Some(state) = self.monitor.get_agent_state(agent_id) {
            state.should_suspend()
        } else {
            false
        }
    }

    /// Resume a suspended agent
    pub async fn resume_agent(&self, agent_id: &str) -> Result<()> {
        self.monitor.resume_agent(agent_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_cpu_percent, 80.0);
        assert_eq!(limits.max_memory_bytes, 2 * 1024 * 1024 * 1024);
        assert_eq!(limits.idle_cpu_threshold, 5.0);
        assert!(limits.auto_suspend_enabled);
    }

    #[test]
    fn test_agent_resource_state() {
        let limits = ResourceLimits::default();
        let mut state = AgentResourceState::new("test-agent".to_string(), Some(1234), limits);

        // Test initial state
        assert_eq!(state.agent_id, "test-agent");
        assert!(!state.is_suspended);
        assert_eq!(state.limit_violations, 0);

        // Test usage update
        let usage = ResourceUsage {
            cpu_percent: 10.0,
            memory_bytes: 1024 * 1024 * 100, // 100MB
            memory_percent: 5.0,
            thread_count: 4,
            timestamp: Utc::now(),
        };

        state.update_usage(usage.clone());
        assert_eq!(state.current_usage.cpu_percent, 10.0);
        assert_eq!(state.usage_history.len(), 1);
    }

    #[test]
    fn test_should_suspend() {
        let mut limits = ResourceLimits::default();
        limits.idle_timeout = Duration::seconds(10); // Short timeout for testing

        let mut state = AgentResourceState::new("test-agent".to_string(), None, limits);

        // Initially should not suspend
        assert!(!state.should_suspend());

        // Set last active to past
        state.last_active = Utc::now() - Duration::seconds(20);
        assert!(state.should_suspend());

        // Suspended agents should not be suspended again
        state.is_suspended = true;
        assert!(!state.should_suspend());
    }

    #[test]
    fn test_resource_efficiency_stats() {
        let limits = ResourceLimits::default();
        let monitor = ResourceMonitor::new(limits);

        // Add some test agents
        monitor
            .start_monitoring("agent1".to_string(), Some(1001), None)
            .unwrap();
        monitor
            .start_monitoring("agent2".to_string(), Some(1002), None)
            .unwrap();

        let stats = monitor.get_efficiency_stats();
        assert_eq!(stats.total_agents, 2);
        assert_eq!(stats.suspended_agents, 0);
        assert_eq!(stats.active_agents, 2);
    }
}

// Tests module is already defined above
