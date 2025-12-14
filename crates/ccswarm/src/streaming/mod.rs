use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{Duration, interval};

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

/// Options for creating a subscription
#[derive(Debug, Default, Clone)]
pub struct SubscriptionOptions {
    pub agent_id: Option<String>,
    pub filter: Option<OutputFilter>,
}

impl SubscriptionOptions {
    /// Create options for a specific agent
    pub fn for_agent(agent_id: String) -> Self {
        Self {
            agent_id: Some(agent_id),
            filter: None,
        }
    }

    /// Create options with a filter
    pub fn with_filter(filter: OutputFilter) -> Self {
        Self {
            agent_id: None,
            filter: Some(filter),
        }
    }

    /// Add agent filter to existing options
    pub fn and_agent(mut self, agent_id: String) -> Self {
        self.agent_id = Some(agent_id);
        self
    }

    /// Add output filter to existing options
    pub fn and_filter(mut self, filter: OutputFilter) -> Self {
        self.filter = Some(filter);
        self
    }
}

/// Stream subscription handle
#[derive(Debug, Clone)]
pub struct StreamSubscription {
    pub id: String,
    pub options: SubscriptionOptions,
    pub created_at: DateTime<Utc>,
    tx: mpsc::UnboundedSender<OutputEntry>,
}

impl StreamSubscription {
    /// Create a new subscription with options
    pub fn new(
        id: String,
        tx: mpsc::UnboundedSender<OutputEntry>,
        options: SubscriptionOptions,
    ) -> Self {
        Self {
            id,
            options,
            created_at: Utc::now(),
            tx,
        }
    }

    /// Send an entry to this subscriber
    pub fn send(&self, entry: &OutputEntry) -> Result<(), String> {
        self.tx
            .send(entry.clone())
            .map_err(|e| format!("Failed to send to subscriber: {}", e))
    }

    /// Check if this subscription should receive an entry
    pub fn should_receive(&self, entry: &OutputEntry) -> bool {
        // Use a single filtering function for all checks
        Self::entry_matches_options(entry, &self.options)
    }

    /// Unified filtering logic
    fn entry_matches_options(entry: &OutputEntry, options: &SubscriptionOptions) -> bool {
        // Check agent filter
        if let Some(ref agent_id) = options.agent_id {
            if &entry.agent_id != agent_id {
                return false;
            }
        }

        // Check output filter
        if let Some(ref filter) = options.filter {
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

/// Pattern matcher for highlighting
struct PatternMatcher {
    patterns: Vec<(&'static str, HighlightStyle)>,
}

impl PatternMatcher {
    fn new() -> Self {
        Self {
            patterns: vec![
                ("error", HighlightStyle::Error),
                ("ERROR", HighlightStyle::Error),
                ("Error", HighlightStyle::Error),
                ("failed", HighlightStyle::Error),
                ("FAILED", HighlightStyle::Error),
                ("Failed", HighlightStyle::Error),
                ("panic", HighlightStyle::Error),
                ("PANIC", HighlightStyle::Error),
                ("Panic", HighlightStyle::Error),
                ("exception", HighlightStyle::Error),
                ("Exception", HighlightStyle::Error),
                ("warning", HighlightStyle::Warning),
                ("WARNING", HighlightStyle::Warning),
                ("Warning", HighlightStyle::Warning),
                ("warn", HighlightStyle::Warning),
                ("WARN", HighlightStyle::Warning),
                ("Warn", HighlightStyle::Warning),
                ("success", HighlightStyle::Success),
                ("SUCCESS", HighlightStyle::Success),
                ("Success", HighlightStyle::Success),
                ("completed", HighlightStyle::Success),
                ("COMPLETED", HighlightStyle::Success),
                ("Completed", HighlightStyle::Success),
                ("done", HighlightStyle::Success),
                ("DONE", HighlightStyle::Success),
                ("Done", HighlightStyle::Success),
                ("finished", HighlightStyle::Success),
                ("FINISHED", HighlightStyle::Success),
                ("Finished", HighlightStyle::Success),
            ],
        }
    }

    fn find_matches(&self, content: &str) -> Vec<HighlightSpan> {
        let mut spans = Vec::new();

        // Group patterns by style and find first match for each style
        let mut matched_styles = Vec::new();

        for (pattern, style) in &self.patterns {
            // Skip if we already have a match for this style
            if matched_styles
                .iter()
                .any(|(_, s)| std::mem::discriminant(s) == std::mem::discriminant(style))
            {
                continue;
            }

            if let Some(pos) = content.find(pattern) {
                spans.push(HighlightSpan {
                    start: pos,
                    end: pos + pattern.len(),
                    style: style.clone(),
                });
                matched_styles.push((pos, style.clone()));
            }
        }

        spans
    }
}

/// Output formatter for different display contexts
pub struct OutputFormatter {
    config: StreamConfig,
    pattern_matcher: PatternMatcher,
    path_regex: regex::Regex,
}

impl OutputFormatter {
    pub fn new(config: StreamConfig) -> Self {
        Self {
            config,
            pattern_matcher: PatternMatcher::new(),
            path_regex: regex::Regex::new(r"[/\\]?[\w.-]+(?:[/\\][\w.-]+)*\.[a-zA-Z0-9]+")
                .expect("Failed to compile path regex"),
        }
    }

    /// Format an output entry for display
    pub fn format_entry(&self, entry: &OutputEntry) -> FormattedOutputEntry {
        let display_prefix = self.create_display_prefix(entry);
        let formatted_content = self.format_content(&entry.content);
        let highlight_spans = if self.config.enable_highlighting {
            self.generate_highlights(&formatted_content)
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
        let time_str = entry.timestamp.format("%H:%M:%S");
        let type_icon = Self::output_type_icon(&entry.output_type);
        let agent_id_short = Self::truncate_string(&entry.agent_id, 8);
        let task_suffix = entry
            .task_id
            .as_ref()
            .map(|t| format!(" [{}]", Self::truncate_string(t, 8)))
            .unwrap_or_default();

        format!(
            "[{}] {} {} {}{}",
            time_str, type_icon, entry.agent_type, agent_id_short, task_suffix
        )
    }

    fn format_content(&self, content: &str) -> String {
        let mut formatted = if content.len() > self.config.max_line_length {
            format!("{}...", &content[..self.config.max_line_length - 3])
        } else {
            content.to_string()
        };

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

    fn generate_highlights(&self, content: &str) -> Vec<HighlightSpan> {
        let mut spans = self.pattern_matcher.find_matches(content);

        // Highlight file paths
        for mat in self.path_regex.find_iter(content) {
            spans.push(HighlightSpan {
                start: mat.start(),
                end: mat.end(),
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

    /// Get icon for output type
    fn output_type_icon(output_type: &OutputType) -> &'static str {
        match output_type {
            OutputType::Error => "❌",
            OutputType::Warning => "⚠️",
            OutputType::Info => "ℹ️",
            OutputType::Debug => "🔍",
            OutputType::System => "⚙️",
            OutputType::Stdout => "📤",
            OutputType::Stderr => "📥",
        }
    }

    /// Truncate string to specified length
    fn truncate_string(s: &str, max_len: usize) -> &str {
        if s.len() <= max_len {
            s
        } else {
            &s[..max_len.min(s.len())]
        }
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
        self.init_global_receiver()?;

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

    /// Initialize the global receiver
    fn init_global_receiver(&self) -> Result<(), String> {
        let mut receiver = self
            .global_receiver
            .write()
            .map_err(|e| format!("Failed to lock global receiver: {}", e))?;
        *receiver = Some(self.monitoring.subscribe_global());
        Ok(())
    }

    /// Subscribe to output stream with options
    pub fn subscribe_with_options(
        &self,
        id: String,
        options: SubscriptionOptions,
    ) -> Result<mpsc::UnboundedReceiver<OutputEntry>, String> {
        let (tx, rx) = mpsc::unbounded_channel();
        let subscription = StreamSubscription::new(id.clone(), tx, options);

        self.add_subscription(id, subscription)?;
        Ok(rx)
    }

    /// Subscribe to all output
    pub fn subscribe(&self, id: String) -> Result<mpsc::UnboundedReceiver<OutputEntry>, String> {
        self.subscribe_with_options(id, SubscriptionOptions::default())
    }

    /// Subscribe to a specific agent's output
    pub fn subscribe_agent(
        &self,
        id: String,
        agent_id: String,
    ) -> Result<mpsc::UnboundedReceiver<OutputEntry>, String> {
        self.subscribe_with_options(id, SubscriptionOptions::for_agent(agent_id))
    }

    /// Subscribe with a filter
    pub fn subscribe_filtered(
        &self,
        id: String,
        filter: OutputFilter,
    ) -> Result<mpsc::UnboundedReceiver<OutputEntry>, String> {
        self.subscribe_with_options(id, SubscriptionOptions::with_filter(filter))
    }

    /// Add a subscription to the manager
    fn add_subscription(&self, id: String, subscription: StreamSubscription) -> Result<(), String> {
        let mut subscriptions = self
            .subscriptions
            .write()
            .map_err(|e| format!("Failed to lock subscriptions: {}", e))?;
        subscriptions.insert(id, subscription);
        Ok(())
    }

    /// Remove a subscription
    pub fn unsubscribe(&self, id: &str) -> Result<(), String> {
        self.modify_subscriptions(|subs| {
            subs.remove(id);
        })
    }

    /// Clear all subscriptions
    pub fn clear_subscriptions(&self) -> Result<(), String> {
        self.modify_subscriptions(|subs| {
            subs.clear();
        })
    }

    /// Modify subscriptions with a closure
    fn modify_subscriptions<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut HashMap<String, StreamSubscription>),
    {
        let mut subscriptions = self
            .subscriptions
            .write()
            .map_err(|e| format!("Failed to lock subscriptions: {}", e))?;
        f(&mut subscriptions);
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

    /// Main streaming loop
    async fn run_streaming_loop(&self) {
        // Get the receiver once at startup
        let mut receiver = match self.create_receiver() {
            Some(r) => r,
            None => return,
        };

        // Main loop to process messages
        while let Ok(entry) = receiver.recv().await {
            self.distribute_entry(&entry).await;
        }
    }

    /// Create a receiver from the global receiver
    fn create_receiver(&self) -> Option<broadcast::Receiver<OutputEntry>> {
        self.global_receiver
            .read()
            .ok()?
            .as_ref()
            .map(|r| r.resubscribe())
    }

    /// Distribute an entry to all matching subscriptions
    async fn distribute_entry(&self, entry: &OutputEntry) {
        let subscriptions = match self.subscriptions.read() {
            Ok(s) => s,
            Err(_) => return,
        };

        let sent_count = subscriptions
            .values()
            .filter(|sub| sub.should_receive(entry))
            .filter_map(|sub| sub.send(entry).ok())
            .count();

        // Update statistics
        self.update_sent_count(sent_count);
    }

    /// Update the sent count in statistics
    fn update_sent_count(&self, count: usize) {
        if let Ok(mut stats) = self.stats.write() {
            stats.total_messages_sent += count;
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

    /// Update statistics
    fn update_stats(&self) {
        let subscriptions = match self.subscriptions.read() {
            Ok(s) => s,
            Err(_) => return,
        };

        let subscription_details: Vec<SubscriptionStats> = subscriptions
            .values()
            .map(|sub| SubscriptionStats {
                id: sub.id.clone(),
                agent_id: sub.options.agent_id.clone(),
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
