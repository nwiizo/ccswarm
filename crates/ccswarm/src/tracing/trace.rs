//! Trace definitions for distributed tracing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::span::{Span, SpanId, SpanStatus};
use super::{SpanEvent, TraceMetadata};

/// Unique identifier for a trace
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(String);

impl TraceId {
    /// Create a new random trace ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from an existing string
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A trace represents a complete operation, potentially spanning multiple agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    /// Unique trace ID
    pub id: TraceId,
    /// Trace name/operation
    pub name: String,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time (None if still active)
    pub end_time: Option<DateTime<Utc>>,
    /// All spans within this trace
    pub spans: Vec<Span>,
    /// Root-level metadata
    pub metadata: TraceMetadata,
    /// Tags for filtering
    pub tags: HashMap<String, String>,
    /// User or session that initiated the trace
    pub user_id: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
}

impl Trace {
    /// Create a new trace
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: TraceId::new(),
            name: name.into(),
            start_time: Utc::now(),
            end_time: None,
            spans: Vec::new(),
            metadata: TraceMetadata::default(),
            tags: HashMap::new(),
            user_id: None,
            session_id: None,
        }
    }

    /// Create a trace with a specific ID
    pub fn with_id(id: TraceId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            start_time: Utc::now(),
            end_time: None,
            spans: Vec::new(),
            metadata: TraceMetadata::default(),
            tags: HashMap::new(),
            user_id: None,
            session_id: None,
        }
    }

    /// Set user ID
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: TraceMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add a new span to this trace
    pub fn add_span(&mut self, span: Span) {
        self.spans.push(span);
    }

    /// Start a new span
    pub fn start_span(&mut self, name: impl Into<String>, parent_id: Option<SpanId>) -> SpanId {
        let mut span = Span::new(name);
        if let Some(parent) = parent_id {
            span = span.with_parent(parent);
        }
        let span_id = span.span_id.clone();
        self.spans.push(span);
        span_id
    }

    /// End a span by ID
    pub fn end_span(&mut self, span_id: &str, status: SpanStatus, metadata: Option<TraceMetadata>) {
        if let Some(span) = self.spans.iter_mut().find(|s| s.span_id == span_id) {
            span.end(status);
            if let Some(meta) = metadata {
                span.merge_metadata(meta);
            }
        }
    }

    /// Add an event to a span
    pub fn add_event(&mut self, span_id: &str, event: SpanEvent) {
        if let Some(span) = self.spans.iter_mut().find(|s| s.span_id == span_id) {
            span.add_event(event);
        }
    }

    /// Get a span by ID
    pub fn get_span(&self, span_id: &str) -> Option<&Span> {
        self.spans.iter().find(|s| s.span_id == span_id)
    }

    /// Get a mutable span by ID
    pub fn get_span_mut(&mut self, span_id: &str) -> Option<&mut Span> {
        self.spans.iter_mut().find(|s| s.span_id == span_id)
    }

    /// End the trace
    pub fn end(&mut self) {
        self.end_time = Some(Utc::now());
        // End any active spans
        for span in &mut self.spans {
            if span.is_active() {
                span.end(SpanStatus::Cancelled);
            }
        }
    }

    /// Check if trace is active
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }

    /// Get trace duration in milliseconds
    pub fn duration_ms(&self) -> Option<i64> {
        self.end_time
            .map(|end| (end - self.start_time).num_milliseconds())
    }

    /// Get total tokens used across all spans
    pub fn total_tokens(&self) -> u64 {
        self.spans
            .iter()
            .filter_map(|s| s.metadata.tokens_used.as_ref())
            .map(|t| t.total_tokens as u64)
            .sum()
    }

    /// Get total cost across all spans
    pub fn total_cost(&self) -> f64 {
        self.spans.iter().filter_map(|s| s.metadata.cost_usd).sum()
    }

    /// Get all agent IDs involved in this trace
    pub fn involved_agents(&self) -> Vec<String> {
        let mut agents: Vec<String> = self
            .spans
            .iter()
            .filter_map(|s| s.metadata.agent_id.clone())
            .collect();
        agents.sort();
        agents.dedup();
        agents
    }

    /// Get the root spans (spans without parents)
    pub fn root_spans(&self) -> Vec<&Span> {
        self.spans
            .iter()
            .filter(|s| s.parent_span_id.is_none())
            .collect()
    }

    /// Get child spans of a given span
    pub fn child_spans(&self, parent_id: &str) -> Vec<&Span> {
        self.spans
            .iter()
            .filter(|s| s.parent_span_id.as_deref() == Some(parent_id))
            .collect()
    }

    /// Build a tree representation of spans
    pub fn span_tree(&self) -> Vec<SpanTreeNode> {
        let roots = self.root_spans();
        roots
            .iter()
            .map(|root| self.build_tree_node(root))
            .collect()
    }

    fn build_tree_node(&self, span: &Span) -> SpanTreeNode {
        let children = self
            .child_spans(&span.span_id)
            .iter()
            .map(|child| self.build_tree_node(child))
            .collect();

        SpanTreeNode {
            span: span.clone(),
            children,
        }
    }

    /// Get summary statistics
    pub fn summary(&self) -> TraceSummary {
        let successful_spans = self.spans.iter().filter(|s| s.status.is_ok()).count();
        let failed_spans = self.spans.iter().filter(|s| s.status.is_error()).count();
        let total_duration: i64 = self.spans.iter().filter_map(|s| s.duration_ms()).sum();

        TraceSummary {
            trace_id: self.id.clone(),
            name: self.name.clone(),
            total_spans: self.spans.len(),
            successful_spans,
            failed_spans,
            total_duration_ms: total_duration,
            total_tokens: self.total_tokens(),
            total_cost_usd: self.total_cost(),
            agents_involved: self.involved_agents(),
            is_complete: self.end_time.is_some(),
        }
    }
}

/// A node in the span tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanTreeNode {
    /// The span at this node
    pub span: Span,
    /// Child spans
    pub children: Vec<SpanTreeNode>,
}

/// Summary of a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSummary {
    /// Trace ID
    pub trace_id: TraceId,
    /// Trace name
    pub name: String,
    /// Total number of spans
    pub total_spans: usize,
    /// Successful spans
    pub successful_spans: usize,
    /// Failed spans
    pub failed_spans: usize,
    /// Total duration in milliseconds
    pub total_duration_ms: i64,
    /// Total tokens used
    pub total_tokens: u64,
    /// Total cost
    pub total_cost_usd: f64,
    /// Agents involved
    pub agents_involved: Vec<String>,
    /// Whether trace is complete
    pub is_complete: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::TokenUsage;

    #[test]
    fn test_trace_creation() {
        let trace = Trace::new("test_operation");
        assert_eq!(trace.name, "test_operation");
        assert!(trace.is_active());
        assert!(trace.spans.is_empty());
    }

    #[test]
    fn test_trace_with_spans() {
        let mut trace = Trace::new("test");

        let span1_id = trace.start_span("span1", None);
        let span2_id = trace.start_span("span2", Some(span1_id.clone()));

        assert_eq!(trace.spans.len(), 2);

        let span2 = trace.get_span(&span2_id).unwrap();
        assert_eq!(span2.parent_span_id, Some(span1_id));
    }

    #[test]
    fn test_trace_end() {
        let mut trace = Trace::new("test");
        trace.start_span("span1", None);
        trace.end();

        assert!(!trace.is_active());
        assert!(trace.duration_ms().is_some());
        // Active spans should be cancelled
        assert!(trace.spans.iter().all(|s| !s.is_active()));
    }

    #[test]
    fn test_trace_tokens_and_cost() {
        let mut trace = Trace::new("test");

        let span1_id = trace.start_span("span1", None);
        let meta1 = TraceMetadata::new()
            .with_tokens(TokenUsage::new(100, 50))
            .with_cost(0.01);
        trace.end_span(&span1_id, SpanStatus::Ok, Some(meta1));

        let span2_id = trace.start_span("span2", None);
        let meta2 = TraceMetadata::new()
            .with_tokens(TokenUsage::new(200, 100))
            .with_cost(0.02);
        trace.end_span(&span2_id, SpanStatus::Ok, Some(meta2));

        assert_eq!(trace.total_tokens(), 450);
        assert!((trace.total_cost() - 0.03).abs() < 0.001);
    }

    #[test]
    fn test_trace_involved_agents() {
        let mut trace = Trace::new("test");

        let span1_id = trace.start_span("span1", None);
        let meta1 = TraceMetadata::new().with_agent("frontend");
        trace.end_span(&span1_id, SpanStatus::Ok, Some(meta1));

        let span2_id = trace.start_span("span2", None);
        let meta2 = TraceMetadata::new().with_agent("backend");
        trace.end_span(&span2_id, SpanStatus::Ok, Some(meta2));

        let agents = trace.involved_agents();
        assert_eq!(agents.len(), 2);
        assert!(agents.contains(&"frontend".to_string()));
        assert!(agents.contains(&"backend".to_string()));
    }

    #[test]
    fn test_trace_span_tree() {
        let mut trace = Trace::new("test");

        let root_id = trace.start_span("root", None);
        let _child1_id = trace.start_span("child1", Some(root_id.clone()));
        let _child2_id = trace.start_span("child2", Some(root_id.clone()));

        let tree = trace.span_tree();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].children.len(), 2);
    }

    #[test]
    fn test_trace_summary() {
        let mut trace = Trace::new("test");

        let span1_id = trace.start_span("span1", None);
        trace.end_span(&span1_id, SpanStatus::Ok, None);

        let span2_id = trace.start_span("span2", None);
        trace.end_span(
            &span2_id,
            SpanStatus::Error {
                message: "failed".to_string(),
            },
            None,
        );

        trace.end();

        let summary = trace.summary();
        assert_eq!(summary.total_spans, 2);
        assert_eq!(summary.successful_spans, 1);
        assert_eq!(summary.failed_spans, 1);
        assert!(summary.is_complete);
    }

    #[test]
    fn test_trace_id() {
        let id1 = TraceId::new();
        let id2 = TraceId::new();
        assert_ne!(id1, id2);

        let id3 = TraceId::from_string("custom-id");
        assert_eq!(id3.as_str(), "custom-id");
    }
}
