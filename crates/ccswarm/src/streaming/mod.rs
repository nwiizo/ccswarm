use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{interval, Duration};

use crate::monitoring::{MonitoringSystem, OutputEntry, OutputFilter, OutputType};

/// Stream configuration for different output types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    pub buffer_size: usize,
    pub max_line_length: usize,
    pub enable_filtering: bool,
    pub enable_highlighting: bool,
    pub refresh_rate_ms: u64,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            max_line_length: 2000,
            enable_filtering: true,
            enable_highlighting: true,
            refresh_rate_ms: 100,
        }
    }
}

/// Stream subscription handle
#[derive(Debug, Clone)]
pub struct StreamSubscription {
    pub id: String,
    pub agent_id: Option<String>,
    pub filter: Option<OutputFilter>,
    pub created_at: DateTime<Utc>,
    tx: mpsc::UnboundedSender<OutputEntry>,
}

impl StreamSubscription {
    pub fn new(id: String, tx: mpsc::UnboundedSender<OutputEntry>) -> Self {
        Self {
            id,
            agent_id: None,
            filter: None,
            created_at: Utc::now(),
            tx,
        }
    }

    pub fn with_agent(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    pub fn with_filter(mut self, filter: OutputFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn send(&self, entry: OutputEntry) -> Result<(), String> {
        self.tx
            .send(entry)
            .map_err(|e| format!("Failed to send to subscriber: {}", e))
    }

    pub fn should_receive(&self, entry: &OutputEntry) -> bool {
        // Check agent filter
        if let Some(ref agent_id) = self.agent_id {
            if entry.agent_id != *agent_id {
                return false;
            }
        }

        // Check output filter
        if let Some(ref filter) = self.filter {
            if !entry.matches_filter(filter) {
                return false;
            }
        }

        true
    }
}

/// Formatted output entry for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedOutputEntry {
    pub original: OutputEntry,
    pub formatted_content: String,
    pub highlight_spans: Vec<HighlightSpan>,
    pub display_prefix: String,
}

/// Highlight span for syntax highlighting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub style: HighlightStyle,
}

/// Highlight styles for different types of content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HighlightStyle {
    Error,
    Warning,
    Success,
    Command,
    Path,
    Timestamp,
    AgentName,
    TaskId,
    Json,
    Code,
}

/// Output formatter for different display contexts
pub struct OutputFormatter {
    config: StreamConfig,
}

impl OutputFormatter {
    pub fn new(config: StreamConfig) -> Self {
        Self { config }
    }

    /// Format an output entry for display
    pub fn format_entry(&self, entry: &OutputEntry) -> FormattedOutputEntry {
        let display_prefix = self.create_display_prefix(entry);
        let formatted_content = self.format_content(&entry.content);
        let highlight_spans = if self.config.enable_highlighting {
            self.generate_highlights(entry, &formatted_content)
        } else {
            vec![]
        };

        FormattedOutputEntry {
            original: entry.clone(),
            formatted_content,
            highlight_spans,
            display_prefix,
        }
    }

    fn create_display_prefix(&self, entry: &OutputEntry) -> String {
        let time_str = entry.timestamp.format("%H:%M:%S").to_string();
        let type_icon = match entry.output_type {
            OutputType::Error => "❌",
            OutputType::Warning => "⚠️",
            OutputType::Info => "ℹ️",
            OutputType::Debug => "🔍",
            OutputType::System => "⚙️",
            OutputType::Stdout => "📤",
            OutputType::Stderr => "📥",
        };

        let task_suffix = entry
            .task_id
            .as_ref()
            .map(|t| format!(" [{}]", &t[..8.min(t.len())]))
            .unwrap_or_default();

        format!(
            "[{}] {} {} {}{}",
            time_str,
            type_icon,
            entry.agent_type,
            &entry.agent_id[..8.min(entry.agent_id.len())],
            task_suffix
        )
    }

    fn format_content(&self, content: &str) -> String {
        let mut formatted = content.to_string();

        // Truncate if too long
        if formatted.len() > self.config.max_line_length {
            formatted.truncate(self.config.max_line_length - 3);
            formatted.push_str("...");
        }

        // Replace tabs with spaces for consistent display
        formatted = formatted.replace('\t', "    ");

        // Handle multi-line content
        if formatted.contains('\n') {
            formatted = formatted
                .lines()
                .map(|line| format!("  {}", line))
                .collect::<Vec<_>>()
                .join("\n");
        }

        formatted
    }

    fn generate_highlights(&self, _entry: &OutputEntry, content: &str) -> Vec<HighlightSpan> {
        let mut spans = Vec::new();

        // Highlight error patterns
        if let Some(error_match) = self.find_pattern(
            content,
            &[
                "error",
                "ERROR",
                "Error",
                "failed",
                "FAILED",
                "Failed",
                "panic",
                "PANIC",
                "Panic",
                "exception",
                "Exception",
            ],
        ) {
            spans.push(HighlightSpan {
                start: error_match.0,
                end: error_match.1,
                style: HighlightStyle::Error,
            });
        }

        // Highlight warning patterns
        if let Some(warning_match) = self.find_pattern(
            content,
            &["warning", "WARNING", "Warning", "warn", "WARN", "Warn"],
        ) {
            spans.push(HighlightSpan {
                start: warning_match.0,
                end: warning_match.1,
                style: HighlightStyle::Warning,
            });
        }

        // Highlight success patterns
        if let Some(success_match) = self.find_pattern(
            content,
            &[
                "success",
                "SUCCESS",
                "Success",
                "completed",
                "COMPLETED",
                "Completed",
                "done",
                "DONE",
                "Done",
                "finished",
                "FINISHED",
                "Finished",
            ],
        ) {
            spans.push(HighlightSpan {
                start: success_match.0,
                end: success_match.1,
                style: HighlightStyle::Success,
            });
        }

        // Highlight file paths
        for path_match in self.find_all_paths(content) {
            spans.push(HighlightSpan {
                start: path_match.0,
                end: path_match.1,
                style: HighlightStyle::Path,
            });
        }

        // Highlight JSON content
        if content.trim().starts_with('{') || content.trim().starts_with('[') {
            spans.push(HighlightSpan {
                start: 0,
                end: content.len(),
                style: HighlightStyle::Json,
            });
        }

        spans
    }

    fn find_pattern(&self, content: &str, patterns: &[&str]) -> Option<(usize, usize)> {
        for pattern in patterns {
            if let Some(pos) = content.find(pattern) {
                return Some((pos, pos + pattern.len()));
            }
        }
        None
    }

    fn find_all_paths(&self, content: &str) -> Vec<(usize, usize)> {
        let mut paths = Vec::new();
        let path_regex =
            regex::Regex::new(r"[/\\]?[\w.-]+(?:[/\\][\w.-]+)*\.[a-zA-Z0-9]+").unwrap();

        for mat in path_regex.find_iter(content) {
            paths.push((mat.start(), mat.end()));
        }

        paths
    }
}

/// Statistics for streaming system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingStats {
    pub active_subscriptions: usize,
    pub total_messages_sent: usize,
    pub messages_per_second: f64,
    pub average_latency_ms: f64,
    pub subscription_details: Vec<SubscriptionStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionStats {
    pub id: String,
    pub agent_id: Option<String>,
    pub messages_received: usize,
    pub last_message_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Main streaming management system
pub struct StreamingManager {
    monitoring: Arc<MonitoringSystem>,
    subscriptions: Arc<RwLock<HashMap<String, StreamSubscription>>>,
    formatter: OutputFormatter,
    config: StreamConfig,
    stats: Arc<RwLock<StreamingStats>>,
    global_receiver: Arc<RwLock<Option<broadcast::Receiver<OutputEntry>>>>,
}

impl StreamingManager {
    /// Create a new streaming manager
    pub fn new(monitoring: Arc<MonitoringSystem>, config: StreamConfig) -> Self {
        let formatter = OutputFormatter::new(config.clone());
        let stats = Arc::new(RwLock::new(StreamingStats {
            active_subscriptions: 0,
            total_messages_sent: 0,
            messages_per_second: 0.0,
            average_latency_ms: 0.0,
            subscription_details: Vec::new(),
        }));

        Self {
            monitoring,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            formatter,
            config,
            stats,
            global_receiver: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the streaming manager
    pub async fn start(&self) -> Result<(), String> {
        // Initialize global receiver
        {
            let mut receiver = self
                .global_receiver
                .write()
                .map_err(|e| format!("Failed to lock global receiver: {}", e))?;
            *receiver = Some(self.monitoring.subscribe_global());
        }

        // Start the main streaming loop
        let streaming_manager = self.clone();
        tokio::spawn(async move {
            streaming_manager.run_streaming_loop().await;
        });

        // Start statistics updater
        let stats_manager = self.clone();
        tokio::spawn(async move {
            stats_manager.run_stats_updater().await;
        });

        Ok(())
    }

    /// Subscribe to output stream
    pub fn subscribe(&self, id: String) -> Result<mpsc::UnboundedReceiver<OutputEntry>, String> {
        let (tx, rx) = mpsc::unbounded_channel();
        let subscription = StreamSubscription::new(id.clone(), tx);

        let mut subscriptions = self
            .subscriptions
            .write()
            .map_err(|e| format!("Failed to lock subscriptions: {}", e))?;
        subscriptions.insert(id, subscription);

        Ok(rx)
    }

    /// Subscribe to a specific agent's output
    pub fn subscribe_agent(
        &self,
        id: String,
        agent_id: String,
    ) -> Result<mpsc::UnboundedReceiver<OutputEntry>, String> {
        let (tx, rx) = mpsc::unbounded_channel();
        let subscription = StreamSubscription::new(id.clone(), tx).with_agent(agent_id);

        let mut subscriptions = self
            .subscriptions
            .write()
            .map_err(|e| format!("Failed to lock subscriptions: {}", e))?;
        subscriptions.insert(id, subscription);

        Ok(rx)
    }

    /// Subscribe with a filter
    pub fn subscribe_filtered(
        &self,
        id: String,
        filter: OutputFilter,
    ) -> Result<mpsc::UnboundedReceiver<OutputEntry>, String> {
        let (tx, rx) = mpsc::unbounded_channel();
        let subscription = StreamSubscription::new(id.clone(), tx).with_filter(filter);

        let mut subscriptions = self
            .subscriptions
            .write()
            .map_err(|e| format!("Failed to lock subscriptions: {}", e))?;
        subscriptions.insert(id, subscription);

        Ok(rx)
    }

    /// Unsubscribe from output stream
    pub fn unsubscribe(&self, id: &str) -> Result<(), String> {
        let mut subscriptions = self
            .subscriptions
            .write()
            .map_err(|e| format!("Failed to lock subscriptions: {}", e))?;
        subscriptions.remove(id);
        Ok(())
    }

    /// Format an output entry for display
    pub fn format_entry(&self, entry: &OutputEntry) -> FormattedOutputEntry {
        self.formatter.format_entry(entry)
    }

    /// Get current streaming statistics
    pub fn get_stats(&self) -> Result<StreamingStats, String> {
        self.stats
            .read()
            .map_err(|e| format!("Failed to lock stats: {}", e))
            .map(|stats| stats.clone())
    }

    /// Clear all subscriptions
    pub fn clear_subscriptions(&self) -> Result<(), String> {
        let mut subscriptions = self
            .subscriptions
            .write()
            .map_err(|e| format!("Failed to lock subscriptions: {}", e))?;
        subscriptions.clear();
        Ok(())
    }

    /// Main streaming loop
    async fn run_streaming_loop(&self) {
        // Get the receiver once at startup
        let mut receiver = {
            let global_receiver = match self.global_receiver.read() {
                Ok(gr) => gr,
                Err(_) => return,
            };

            match global_receiver.as_ref() {
                Some(r) => r.resubscribe(),
                None => return,
            }
        };

        // Main loop to process messages
        while let Ok(entry) = receiver.recv().await {
            self.distribute_entry(entry).await;
        }
    }

    /// Distribute an entry to all matching subscriptions
    async fn distribute_entry(&self, entry: OutputEntry) {
        let subscriptions = match self.subscriptions.read() {
            Ok(s) => s,
            Err(_) => return,
        };

        let mut sent_count = 0;
        for subscription in subscriptions.values() {
            if subscription.should_receive(&entry) && subscription.send(entry.clone()).is_ok() {
                sent_count += 1;
            }
        }

        // Update statistics
        if let Ok(mut stats) = self.stats.write() {
            stats.total_messages_sent += sent_count;
        }
    }

    /// Update streaming statistics
    async fn run_stats_updater(&self) {
        let mut interval = interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            self.update_stats();
        }
    }

    fn update_stats(&self) {
        let subscriptions = match self.subscriptions.read() {
            Ok(s) => s,
            Err(_) => return,
        };

        let subscription_details: Vec<SubscriptionStats> = subscriptions
            .values()
            .map(|sub| SubscriptionStats {
                id: sub.id.clone(),
                agent_id: sub.agent_id.clone(),
                messages_received: 0, // This would need to be tracked separately
                last_message_at: None, // This would need to be tracked separately
                created_at: sub.created_at,
            })
            .collect();

        if let Ok(mut stats) = self.stats.write() {
            stats.active_subscriptions = subscriptions.len();
            stats.subscription_details = subscription_details;
            // messages_per_second and average_latency_ms would be calculated
            // based on additional tracking that would be implemented
        }
    }
}

impl Clone for StreamingManager {
    fn clone(&self) -> Self {
        Self {
            monitoring: Arc::clone(&self.monitoring),
            subscriptions: Arc::clone(&self.subscriptions),
            formatter: OutputFormatter::new(self.config.clone()),
            config: self.config.clone(),
            stats: Arc::clone(&self.stats),
            global_receiver: Arc::clone(&self.global_receiver),
        }
    }
}

