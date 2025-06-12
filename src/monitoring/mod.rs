use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

/// Maximum number of output lines to buffer per agent
const MAX_BUFFER_SIZE: usize = 10000;

/// Maximum age of buffered output before automatic cleanup
const MAX_OUTPUT_AGE: Duration = Duration::from_secs(3600); // 1 hour

/// Type of output message
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputType {
    Stdout,
    Stderr,
    Info,
    Warning,
    Error,
    Debug,
    System,
}

/// A single output entry from an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputEntry {
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub agent_type: String,
    pub output_type: OutputType,
    pub content: String,
    pub task_id: Option<String>,
    pub session_id: String,
}

impl OutputEntry {
    /// Create a new output entry
    pub fn new(
        agent_id: String,
        agent_type: String,
        output_type: OutputType,
        content: String,
        task_id: Option<String>,
        session_id: String,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            agent_id,
            agent_type,
            output_type,
            content,
            task_id,
            session_id,
        }
    }

    /// Check if this entry matches a filter pattern
    pub fn matches_filter(&self, filter: &OutputFilter) -> bool {
        // Check agent filter
        if let Some(ref agent_ids) = filter.agent_ids {
            if !agent_ids.contains(&self.agent_id) {
                return false;
            }
        }

        // Check output type filter
        if let Some(ref output_types) = filter.output_types {
            if !output_types.contains(&self.output_type) {
                return false;
            }
        }

        // Check content filter
        if let Some(ref pattern) = filter.content_pattern {
            if !self
                .content
                .to_lowercase()
                .contains(&pattern.to_lowercase())
            {
                return false;
            }
        }

        // Check task filter
        if let Some(ref task_id) = filter.task_id {
            if self.task_id.as_ref() != Some(task_id) {
                return false;
            }
        }

        true
    }
}

/// Filter for output entries
#[derive(Debug, Clone, Default)]
pub struct OutputFilter {
    pub agent_ids: Option<Vec<String>>,
    pub output_types: Option<Vec<OutputType>>,
    pub content_pattern: Option<String>,
    pub task_id: Option<String>,
}

/// Output stream for a single agent
pub struct AgentOutputStream {
    buffer: Arc<Mutex<VecDeque<OutputEntry>>>,
    last_cleanup: Arc<Mutex<Instant>>,
    #[allow(dead_code)] // Used in matches_filter method via OutputEntry
    agent_id: String,
    broadcast_tx: broadcast::Sender<OutputEntry>,
}

impl AgentOutputStream {
    /// Create a new agent output stream
    pub fn new(agent_id: String) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1024);
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_BUFFER_SIZE))),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
            agent_id,
            broadcast_tx,
        }
    }

    /// Add an output entry to the stream
    pub fn add_output(&self, entry: OutputEntry) -> Result<(), String> {
        // Broadcast to live subscribers
        let _ = self.broadcast_tx.send(entry.clone());

        let mut buffer = self
            .buffer
            .lock()
            .map_err(|e| format!("Failed to lock buffer: {}", e))?;

        // Add new entry
        buffer.push_back(entry);

        // Enforce buffer size limit
        while buffer.len() > MAX_BUFFER_SIZE {
            buffer.pop_front();
        }

        // Periodic cleanup of old entries
        let mut last_cleanup = self
            .last_cleanup
            .lock()
            .map_err(|e| format!("Failed to lock cleanup time: {}", e))?;
        if last_cleanup.elapsed() > Duration::from_secs(300) {
            // Check every 5 minutes
            let cutoff_time = Utc::now() - chrono::Duration::from_std(MAX_OUTPUT_AGE).unwrap();
            buffer.retain(|entry| entry.timestamp > cutoff_time);
            *last_cleanup = Instant::now();
        }

        Ok(())
    }

    /// Get recent output entries with optional filter
    pub fn get_recent(&self, count: usize, filter: Option<&OutputFilter>) -> Vec<OutputEntry> {
        let buffer = match self.buffer.lock() {
            Ok(b) => b,
            Err(_) => return vec![],
        };

        buffer
            .iter()
            .rev()
            .filter(|entry| filter.map_or(true, |f| entry.matches_filter(f)))
            .take(count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Subscribe to live output updates
    pub fn subscribe(&self) -> broadcast::Receiver<OutputEntry> {
        self.broadcast_tx.subscribe()
    }

    /// Clear all buffered output
    pub fn clear(&self) {
        if let Ok(mut buffer) = self.buffer.lock() {
            buffer.clear();
        }
    }

    /// Get the total number of buffered entries
    pub fn len(&self) -> usize {
        self.buffer.lock().map(|b| b.len()).unwrap_or(0)
    }
}

/// Trait for output subscribers
pub trait OutputSubscriber: Send + Sync {
    /// Called when a new output entry is available
    fn on_output(&self, entry: &OutputEntry);

    /// Get the subscriber ID
    fn id(&self) -> &str;

    /// Check if this subscriber is interested in the given entry
    fn accepts(&self, _entry: &OutputEntry) -> bool {
        true // Default implementation accepts all entries
    }
}

/// Statistics for monitoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringStats {
    pub total_entries: usize,
    pub entries_per_agent: std::collections::HashMap<String, usize>,
    pub entries_per_type: std::collections::HashMap<String, usize>,
    pub active_streams: usize,
    pub total_subscribers: usize,
}

/// Central monitoring system for all agents
pub struct MonitoringSystem {
    streams: Arc<RwLock<std::collections::HashMap<String, Arc<AgentOutputStream>>>>,
    subscribers: Arc<RwLock<Vec<Arc<dyn OutputSubscriber>>>>,
    global_broadcast: broadcast::Sender<OutputEntry>,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub fn new() -> Self {
        let (global_broadcast, _) = broadcast::channel(2048);
        Self {
            streams: Arc::new(RwLock::new(std::collections::HashMap::new())),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            global_broadcast,
        }
    }

    /// Register a new agent stream
    pub fn register_agent(&self, agent_id: String) -> Result<Arc<AgentOutputStream>, String> {
        let mut streams = self
            .streams
            .write()
            .map_err(|e| format!("Failed to lock streams: {}", e))?;

        let stream = Arc::new(AgentOutputStream::new(agent_id.clone()));
        streams.insert(agent_id, stream.clone());

        Ok(stream)
    }

    /// Unregister an agent stream
    pub fn unregister_agent(&self, agent_id: &str) -> Result<(), String> {
        let mut streams = self
            .streams
            .write()
            .map_err(|e| format!("Failed to lock streams: {}", e))?;
        streams.remove(agent_id);
        Ok(())
    }

    /// Get an agent's output stream
    pub fn get_agent_stream(&self, agent_id: &str) -> Option<Arc<AgentOutputStream>> {
        self.streams.read().ok()?.get(agent_id).cloned()
    }

    /// Add an output entry for an agent
    pub fn add_output(
        &self,
        agent_id: String,
        agent_type: String,
        output_type: OutputType,
        content: String,
        task_id: Option<String>,
        session_id: String,
    ) -> Result<(), String> {
        let entry = OutputEntry::new(
            agent_id.clone(),
            agent_type,
            output_type,
            content,
            task_id,
            session_id,
        );

        // Send to global broadcast
        let _ = self.global_broadcast.send(entry.clone());

        // Send to agent-specific stream
        if let Some(stream) = self.get_agent_stream(&agent_id) {
            stream.add_output(entry.clone())?;
        } else {
            // Auto-register agent if not exists
            let stream = self.register_agent(agent_id)?;
            stream.add_output(entry.clone())?;
        }

        // Notify subscribers
        if let Ok(subscribers) = self.subscribers.read() {
            for subscriber in subscribers.iter() {
                if subscriber.accepts(&entry) {
                    subscriber.on_output(&entry);
                }
            }
        }

        Ok(())
    }

    /// Subscribe to global output updates
    pub fn subscribe_global(&self) -> broadcast::Receiver<OutputEntry> {
        self.global_broadcast.subscribe()
    }

    /// Add an output subscriber
    pub fn add_subscriber(&self, subscriber: Arc<dyn OutputSubscriber>) -> Result<(), String> {
        let mut subscribers = self
            .subscribers
            .write()
            .map_err(|e| format!("Failed to lock subscribers: {}", e))?;
        subscribers.push(subscriber);
        Ok(())
    }

    /// Remove an output subscriber
    pub fn remove_subscriber(&self, subscriber_id: &str) -> Result<(), String> {
        let mut subscribers = self
            .subscribers
            .write()
            .map_err(|e| format!("Failed to lock subscribers: {}", e))?;
        subscribers.retain(|s| s.id() != subscriber_id);
        Ok(())
    }

    /// Get recent output from all agents
    pub fn get_all_recent(&self, count: usize, filter: Option<&OutputFilter>) -> Vec<OutputEntry> {
        let streams = match self.streams.read() {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        let mut all_entries: Vec<OutputEntry> = streams
            .values()
            .flat_map(|stream| stream.get_recent(count, filter))
            .collect();

        // Sort by timestamp
        all_entries.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Take the most recent entries
        all_entries.into_iter().rev().take(count).rev().collect()
    }

    /// Clear output for a specific agent
    pub fn clear_agent_output(&self, agent_id: &str) -> Result<(), String> {
        if let Some(stream) = self.get_agent_stream(agent_id) {
            stream.clear();
            Ok(())
        } else {
            Err(format!("Agent stream not found: {}", agent_id))
        }
    }

    /// Clear all output
    pub fn clear_all_output(&self) -> Result<(), String> {
        let streams = self
            .streams
            .read()
            .map_err(|e| format!("Failed to lock streams: {}", e))?;
        for stream in streams.values() {
            stream.clear();
        }
        Ok(())
    }

    /// Get monitoring statistics
    pub fn get_stats(&self) -> MonitoringStats {
        let streams = match self.streams.read() {
            Ok(s) => s,
            Err(_) => {
                return MonitoringStats {
                    total_entries: 0,
                    entries_per_agent: std::collections::HashMap::new(),
                    entries_per_type: std::collections::HashMap::new(),
                    active_streams: 0,
                    total_subscribers: 0,
                }
            }
        };

        let mut total_entries = 0;
        let mut entries_per_agent = std::collections::HashMap::new();
        let mut entries_per_type: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for (agent_id, stream) in streams.iter() {
            let count = stream.len();
            total_entries += count;
            entries_per_agent.insert(agent_id.clone(), count);

            // Count by type (sampling last 100 entries)
            for entry in stream.get_recent(100, None) {
                let type_name = format!("{:?}", entry.output_type);
                *entries_per_type.entry(type_name).or_insert(0) += 1;
            }
        }

        let total_subscribers = self.subscribers.read().map(|s| s.len()).unwrap_or(0);

        MonitoringStats {
            total_entries,
            entries_per_agent,
            entries_per_type,
            active_streams: streams.len(),
            total_subscribers,
        }
    }
}

impl Default for MonitoringSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple console output subscriber for debugging
pub struct ConsoleOutputSubscriber {
    id: String,
    filter: Option<OutputFilter>,
}

impl ConsoleOutputSubscriber {
    pub fn new(id: String) -> Self {
        Self { id, filter: None }
    }

    pub fn with_filter(mut self, filter: OutputFilter) -> Self {
        self.filter = Some(filter);
        self
    }
}

impl OutputSubscriber for ConsoleOutputSubscriber {
    fn on_output(&self, entry: &OutputEntry) {
        let type_indicator = match entry.output_type {
            OutputType::Error => "❌",
            OutputType::Warning => "⚠️",
            OutputType::Info => "ℹ️",
            OutputType::Debug => "🔍",
            OutputType::System => "⚙️",
            _ => "📝",
        };

        println!(
            "[{}] {} {} [{}]: {}",
            entry.timestamp.format("%H:%M:%S"),
            type_indicator,
            entry.agent_type,
            entry.agent_id,
            entry.content
        );
    }

    fn id(&self) -> &str {
        &self.id
    }

    fn accepts(&self, entry: &OutputEntry) -> bool {
        self.filter
            .as_ref()
            .map_or(true, |f| entry.matches_filter(f))
    }
}
