//! Trace collector for gathering and managing traces

use std::collections::HashMap;

use super::span::SpanStatus;
use super::trace::{Trace, TraceId};
use super::{SpanEvent, TraceMetadata, TracingStats};

/// Configuration for the trace collector
#[derive(Debug, Clone)]
pub struct TraceCollectorConfig {
    /// Maximum number of traces to keep
    pub max_traces: usize,
    /// Whether to auto-cleanup old traces
    pub auto_cleanup: bool,
    /// Cleanup interval in seconds
    pub cleanup_interval_secs: u64,
}

impl Default for TraceCollectorConfig {
    fn default() -> Self {
        Self {
            max_traces: 1000,
            auto_cleanup: true,
            cleanup_interval_secs: 3600,
        }
    }
}

/// Collects and manages traces
pub struct TraceCollector {
    /// All traces indexed by ID
    traces: HashMap<TraceId, Trace>,
    /// Configuration
    config: TraceCollectorConfig,
    /// Traces ordered by creation time for LRU eviction
    trace_order: Vec<TraceId>,
}

impl TraceCollector {
    /// Create a new trace collector
    pub fn new(config: TraceCollectorConfig) -> Self {
        Self {
            traces: HashMap::new(),
            config,
            trace_order: Vec::new(),
        }
    }

    /// Start a new trace
    pub fn start_trace(&mut self, name: impl Into<String>) -> TraceId {
        self.enforce_max_traces();

        let trace = Trace::new(name);
        let id = trace.id.clone();
        self.trace_order.push(id.clone());
        self.traces.insert(id.clone(), trace);
        id
    }

    /// Start a new trace with a specific ID
    pub fn start_trace_with_id(&mut self, id: TraceId, name: impl Into<String>) -> TraceId {
        self.enforce_max_traces();

        let trace = Trace::with_id(id.clone(), name);
        self.trace_order.push(id.clone());
        self.traces.insert(id.clone(), trace);
        id
    }

    /// Start a span within a trace
    pub fn start_span(
        &mut self,
        trace_id: &TraceId,
        name: impl Into<String>,
        parent_span_id: Option<String>,
    ) -> Option<String> {
        let trace = self.traces.get_mut(trace_id)?;
        Some(trace.start_span(name, parent_span_id))
    }

    /// End a span
    pub fn end_span(
        &mut self,
        trace_id: &TraceId,
        span_id: &str,
        status: SpanStatus,
        metadata: Option<TraceMetadata>,
    ) {
        if let Some(trace) = self.traces.get_mut(trace_id) {
            trace.end_span(span_id, status, metadata);
        }
    }

    /// Add an event to a span
    pub fn add_event(&mut self, trace_id: &TraceId, span_id: &str, event: SpanEvent) {
        if let Some(trace) = self.traces.get_mut(trace_id) {
            trace.add_event(span_id, event);
        }
    }

    /// End a trace
    pub fn end_trace(&mut self, trace_id: &TraceId) {
        if let Some(trace) = self.traces.get_mut(trace_id) {
            trace.end();
        }
    }

    /// Get a trace by ID
    pub fn get_trace(&self, trace_id: &TraceId) -> Option<Trace> {
        self.traces.get(trace_id).cloned()
    }

    /// Get a mutable reference to a trace
    pub fn get_trace_mut(&mut self, trace_id: &TraceId) -> Option<&mut Trace> {
        self.traces.get_mut(trace_id)
    }

    /// Get all traces
    pub fn get_all_traces(&self) -> Vec<Trace> {
        self.traces.values().cloned().collect()
    }

    /// Get active traces
    pub fn get_active_traces(&self) -> Vec<Trace> {
        self.traces
            .values()
            .filter(|t| t.is_active())
            .cloned()
            .collect()
    }

    /// Get completed traces
    pub fn get_completed_traces(&self) -> Vec<Trace> {
        self.traces
            .values()
            .filter(|t| !t.is_active())
            .cloned()
            .collect()
    }

    /// Get traces for a specific agent
    pub fn get_traces_by_agent(&self, agent_id: &str) -> Vec<Trace> {
        self.traces
            .values()
            .filter(|t| t.involved_agents().contains(&agent_id.to_string()))
            .cloned()
            .collect()
    }

    /// Get traces with a specific tag
    pub fn get_traces_by_tag(&self, key: &str, value: &str) -> Vec<Trace> {
        self.traces
            .values()
            .filter(|t| t.tags.get(key).is_some_and(|v| v == value))
            .cloned()
            .collect()
    }

    /// Get traces within a time range
    pub fn get_traces_in_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Vec<Trace> {
        self.traces
            .values()
            .filter(|t| t.start_time >= start && t.start_time <= end)
            .cloned()
            .collect()
    }

    /// Remove a trace
    pub fn remove_trace(&mut self, trace_id: &TraceId) -> Option<Trace> {
        self.trace_order.retain(|id| id != trace_id);
        self.traces.remove(trace_id)
    }

    /// Clear all traces
    pub fn clear(&mut self) {
        self.traces.clear();
        self.trace_order.clear();
    }

    /// Get statistics
    pub fn get_stats(&self) -> TracingStats {
        let mut total_tokens: u64 = 0;
        let mut total_cost: f64 = 0.0;
        let mut total_spans: usize = 0;
        let mut total_duration: i64 = 0;
        let mut span_count: usize = 0;
        let mut traces_per_agent: HashMap<String, usize> = HashMap::new();

        for trace in self.traces.values() {
            total_tokens += trace.total_tokens();
            total_cost += trace.total_cost();
            total_spans += trace.spans.len();

            for span in &trace.spans {
                if let Some(duration) = span.duration_ms() {
                    total_duration += duration;
                    span_count += 1;
                }
                if let Some(ref agent) = span.metadata.agent_id {
                    *traces_per_agent.entry(agent.clone()).or_insert(0) += 1;
                }
            }
        }

        let avg_duration = if span_count > 0 {
            total_duration as f64 / span_count as f64
        } else {
            0.0
        };

        TracingStats {
            total_traces: self.traces.len(),
            active_traces: self.traces.values().filter(|t| t.is_active()).count(),
            completed_traces: self.traces.values().filter(|t| !t.is_active()).count(),
            total_spans,
            total_tokens,
            total_cost_usd: total_cost,
            traces_per_agent,
            avg_span_duration_ms: avg_duration,
        }
    }

    /// Enforce maximum trace limit using LRU eviction
    fn enforce_max_traces(&mut self) {
        while self.traces.len() >= self.config.max_traces {
            // Remove oldest completed trace first
            if let Some(pos) = self
                .trace_order
                .iter()
                .position(|id| self.traces.get(id).is_some_and(|t| !t.is_active()))
            {
                let id = self.trace_order.remove(pos);
                self.traces.remove(&id);
            } else if let Some(id) = self.trace_order.first().cloned() {
                // If no completed traces, remove oldest active trace
                self.trace_order.remove(0);
                self.traces.remove(&id);
            } else {
                break;
            }
        }
    }

    /// Cleanup old completed traces
    pub fn cleanup_old_traces(&mut self, max_age_secs: u64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(max_age_secs as i64);

        let to_remove: Vec<TraceId> = self
            .traces
            .iter()
            .filter(|(_, t)| !t.is_active() && t.start_time < cutoff)
            .map(|(id, _)| id.clone())
            .collect();

        for id in to_remove {
            self.remove_trace(&id);
        }
    }

    /// Get the number of traces
    pub fn len(&self) -> usize {
        self.traces.len()
    }

    /// Check if collector is empty
    pub fn is_empty(&self) -> bool {
        self.traces.is_empty()
    }
}

impl Default for TraceCollector {
    fn default() -> Self {
        Self::new(TraceCollectorConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::TokenUsage;

    #[test]
    fn test_collector_creation() {
        let collector = TraceCollector::default();
        assert!(collector.is_empty());
    }

    #[test]
    fn test_start_trace() {
        let mut collector = TraceCollector::default();
        let trace_id = collector.start_trace("test");

        assert_eq!(collector.len(), 1);
        assert!(collector.get_trace(&trace_id).is_some());
    }

    #[test]
    fn test_start_span() {
        let mut collector = TraceCollector::default();
        let trace_id = collector.start_trace("test");
        let span_id = collector.start_span(&trace_id, "span1", None);

        assert!(span_id.is_some());

        let trace = collector.get_trace(&trace_id).unwrap();
        assert_eq!(trace.spans.len(), 1);
    }

    #[test]
    fn test_end_span() {
        let mut collector = TraceCollector::default();
        let trace_id = collector.start_trace("test");
        let span_id = collector.start_span(&trace_id, "span1", None).unwrap();

        let metadata = TraceMetadata::new()
            .with_tokens(TokenUsage::new(100, 50))
            .with_cost(0.01);

        collector.end_span(&trace_id, &span_id, SpanStatus::Ok, Some(metadata));

        let trace = collector.get_trace(&trace_id).unwrap();
        let span = trace.get_span(&span_id).unwrap();
        assert!(span.status.is_ok());
        assert!(span.metadata.tokens_used.is_some());
    }

    #[test]
    fn test_end_trace() {
        let mut collector = TraceCollector::default();
        let trace_id = collector.start_trace("test");
        collector.end_trace(&trace_id);

        let trace = collector.get_trace(&trace_id).unwrap();
        assert!(!trace.is_active());
    }

    #[test]
    fn test_get_active_traces() {
        let mut collector = TraceCollector::default();

        let trace1 = collector.start_trace("active");
        let trace2 = collector.start_trace("completed");
        collector.end_trace(&trace2);

        let active = collector.get_active_traces();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, trace1);
    }

    #[test]
    fn test_traces_by_agent() {
        let mut collector = TraceCollector::default();

        let trace_id = collector.start_trace("test");
        let span_id = collector.start_span(&trace_id, "span", None).unwrap();
        let metadata = TraceMetadata::new().with_agent("frontend");
        collector.end_span(&trace_id, &span_id, SpanStatus::Ok, Some(metadata));

        let traces = collector.get_traces_by_agent("frontend");
        assert_eq!(traces.len(), 1);

        let traces = collector.get_traces_by_agent("backend");
        assert!(traces.is_empty());
    }

    #[test]
    fn test_max_traces_eviction() {
        let config = TraceCollectorConfig {
            max_traces: 3,
            ..Default::default()
        };
        let mut collector = TraceCollector::new(config);

        // Create 3 traces and complete them
        let id1 = collector.start_trace("trace1");
        let id2 = collector.start_trace("trace2");
        let id3 = collector.start_trace("trace3");

        collector.end_trace(&id1);
        collector.end_trace(&id2);

        // Create a 4th trace - should evict oldest completed trace
        let _id4 = collector.start_trace("trace4");

        assert_eq!(collector.len(), 3);
        assert!(collector.get_trace(&id1).is_none()); // Oldest completed should be evicted
        assert!(collector.get_trace(&id2).is_some());
        assert!(collector.get_trace(&id3).is_some());
    }

    #[test]
    fn test_stats() {
        let mut collector = TraceCollector::default();

        let trace_id = collector.start_trace("test");
        let span_id = collector.start_span(&trace_id, "span", None).unwrap();
        let metadata = TraceMetadata::new()
            .with_tokens(TokenUsage::new(100, 50))
            .with_cost(0.01)
            .with_agent("frontend");
        collector.end_span(&trace_id, &span_id, SpanStatus::Ok, Some(metadata));
        collector.end_trace(&trace_id);

        let stats = collector.get_stats();
        assert_eq!(stats.total_traces, 1);
        assert_eq!(stats.completed_traces, 1);
        assert_eq!(stats.total_spans, 1);
        assert_eq!(stats.total_tokens, 150);
        assert!(stats.traces_per_agent.contains_key("frontend"));
    }

    #[test]
    fn test_clear() {
        let mut collector = TraceCollector::default();
        collector.start_trace("test1");
        collector.start_trace("test2");

        assert_eq!(collector.len(), 2);

        collector.clear();
        assert!(collector.is_empty());
    }

    #[test]
    fn test_add_event() {
        let mut collector = TraceCollector::default();
        let trace_id = collector.start_trace("test");
        let span_id = collector.start_span(&trace_id, "span", None).unwrap();

        let event = SpanEvent::new("test_event").with_attribute("key", "value");
        collector.add_event(&trace_id, &span_id, event);

        let trace = collector.get_trace(&trace_id).unwrap();
        let span = trace.get_span(&span_id).unwrap();
        assert_eq!(span.events.len(), 1);
        assert_eq!(span.events[0].name, "test_event");
    }
}
