//! Span definitions for distributed tracing

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{SpanEvent, TraceMetadata};

/// Unique identifier for a span
pub type SpanId = String;

/// Status of a span
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpanStatus {
    /// Operation completed successfully
    Ok,
    /// Operation failed with an error
    Error { message: String },
    /// Operation was cancelled
    Cancelled,
    /// Operation is still in progress
    InProgress,
}

impl SpanStatus {
    /// Check if the span completed successfully
    pub fn is_ok(&self) -> bool {
        matches!(self, SpanStatus::Ok)
    }

    /// Check if the span failed
    pub fn is_error(&self) -> bool {
        matches!(self, SpanStatus::Error { .. })
    }

    /// Get error message if present
    pub fn error_message(&self) -> Option<&str> {
        match self {
            SpanStatus::Error { message } => Some(message),
            _ => None,
        }
    }
}

/// Context for span propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanContext {
    /// Trace ID this span belongs to
    pub trace_id: String,
    /// This span's ID
    pub span_id: SpanId,
    /// Parent span ID (if any)
    pub parent_span_id: Option<SpanId>,
    /// Baggage items for cross-service propagation
    pub baggage: HashMap<String, String>,
}

impl SpanContext {
    /// Create a new span context
    pub fn new(trace_id: impl Into<String>, span_id: impl Into<String>) -> Self {
        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            parent_span_id: None,
            baggage: HashMap::new(),
        }
    }

    /// Set parent span
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_span_id = Some(parent_id.into());
        self
    }

    /// Add baggage item
    pub fn with_baggage(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.baggage.insert(key.into(), value.into());
        self
    }
}

/// A span represents a single operation within a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    /// Unique span ID
    pub span_id: SpanId,
    /// Parent span ID (for nested spans)
    pub parent_span_id: Option<SpanId>,
    /// Operation name
    pub name: String,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time (None if still active)
    pub end_time: Option<DateTime<Utc>>,
    /// Span status
    pub status: SpanStatus,
    /// Events that occurred during this span
    pub events: Vec<SpanEvent>,
    /// Metadata and attributes
    pub metadata: TraceMetadata,
    /// Span kind (client, server, producer, consumer, internal)
    pub kind: SpanKind,
}

/// The kind of span
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum SpanKind {
    /// Internal operation
    #[default]
    Internal,
    /// Client making a request
    Client,
    /// Server handling a request
    Server,
    /// Producer sending a message
    Producer,
    /// Consumer receiving a message
    Consumer,
    /// LLM API call
    LlmCall,
    /// Tool execution
    ToolExecution,
    /// Agent operation
    AgentOperation,
}

impl Span {
    /// Create a new span
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            name: name.into(),
            start_time: Utc::now(),
            end_time: None,
            status: SpanStatus::InProgress,
            events: Vec::new(),
            metadata: TraceMetadata::default(),
            kind: SpanKind::Internal,
        }
    }

    /// Set parent span
    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_span_id = Some(parent_id.into());
        self
    }

    /// Set span kind
    pub fn with_kind(mut self, kind: SpanKind) -> Self {
        self.kind = kind;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: TraceMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add an event
    pub fn add_event(&mut self, event: SpanEvent) {
        self.events.push(event);
    }

    /// End the span with a status
    pub fn end(&mut self, status: SpanStatus) {
        self.end_time = Some(Utc::now());
        self.status = status;
    }

    /// End the span successfully
    pub fn end_ok(&mut self) {
        self.end(SpanStatus::Ok);
    }

    /// End the span with an error
    pub fn end_error(&mut self, message: impl Into<String>) {
        self.end(SpanStatus::Error {
            message: message.into(),
        });
    }

    /// Get span duration in milliseconds
    pub fn duration_ms(&self) -> Option<i64> {
        self.end_time
            .map(|end| (end - self.start_time).num_milliseconds())
    }

    /// Check if span is still active
    pub fn is_active(&self) -> bool {
        self.end_time.is_none()
    }

    /// Get the span context
    pub fn context(&self, trace_id: &str) -> SpanContext {
        SpanContext {
            trace_id: trace_id.to_string(),
            span_id: self.span_id.clone(),
            parent_span_id: self.parent_span_id.clone(),
            baggage: HashMap::new(),
        }
    }

    /// Update metadata
    pub fn set_metadata(&mut self, metadata: TraceMetadata) {
        self.metadata = metadata;
    }

    /// Merge additional metadata
    pub fn merge_metadata(&mut self, additional: TraceMetadata) {
        // Merge attributes
        for (k, v) in additional.attributes {
            self.metadata.attributes.insert(k, v);
        }
        // Update other fields if present
        if additional.tokens_used.is_some() {
            self.metadata.tokens_used = additional.tokens_used;
        }
        if additional.cost_usd.is_some() {
            self.metadata.cost_usd = additional.cost_usd;
        }
        if additional.model.is_some() {
            self.metadata.model = additional.model;
        }
        if additional.agent_id.is_some() {
            self.metadata.agent_id = additional.agent_id;
        }
        if additional.task_id.is_some() {
            self.metadata.task_id = additional.task_id;
        }
    }
}

/// Builder for creating spans with fluent API
pub struct SpanBuilder {
    name: String,
    parent_id: Option<String>,
    kind: SpanKind,
    metadata: TraceMetadata,
}

impl SpanBuilder {
    /// Create a new span builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent_id: None,
            kind: SpanKind::Internal,
            metadata: TraceMetadata::default(),
        }
    }

    /// Set parent span
    pub fn parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Set span kind
    pub fn kind(mut self, kind: SpanKind) -> Self {
        self.kind = kind;
        self
    }

    /// Set as LLM call
    pub fn llm_call(self) -> Self {
        self.kind(SpanKind::LlmCall)
    }

    /// Set as tool execution
    pub fn tool_execution(self) -> Self {
        self.kind(SpanKind::ToolExecution)
    }

    /// Set as agent operation
    pub fn agent_operation(self) -> Self {
        self.kind(SpanKind::AgentOperation)
    }

    /// Add metadata
    pub fn metadata(mut self, metadata: TraceMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add agent ID
    pub fn agent(mut self, agent_id: impl Into<String>) -> Self {
        self.metadata.agent_id = Some(agent_id.into());
        self
    }

    /// Add task ID
    pub fn task(mut self, task_id: impl Into<String>) -> Self {
        self.metadata.task_id = Some(task_id.into());
        self
    }

    /// Add model name
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.metadata.model = Some(model.into());
        self
    }

    /// Build the span
    pub fn build(self) -> Span {
        let mut span = Span::new(self.name)
            .with_kind(self.kind)
            .with_metadata(self.metadata);

        if let Some(parent) = self.parent_id {
            span = span.with_parent(parent);
        }

        span
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new("test_operation");
        assert_eq!(span.name, "test_operation");
        assert!(span.is_active());
        assert_eq!(span.status, SpanStatus::InProgress);
    }

    #[test]
    fn test_span_end() {
        let mut span = Span::new("test");
        span.end_ok();

        assert!(!span.is_active());
        assert!(span.status.is_ok());
        assert!(span.end_time.is_some());
    }

    #[test]
    fn test_span_error() {
        let mut span = Span::new("test");
        span.end_error("Something went wrong");

        assert!(span.status.is_error());
        assert_eq!(span.status.error_message(), Some("Something went wrong"));
    }

    #[test]
    fn test_span_duration() {
        let mut span = Span::new("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        span.end_ok();

        let duration = span.duration_ms().unwrap();
        assert!(duration >= 10);
    }

    #[test]
    fn test_span_builder() {
        let span = SpanBuilder::new("llm_call")
            .llm_call()
            .agent("frontend")
            .model("claude-3-opus")
            .build();

        assert_eq!(span.name, "llm_call");
        assert_eq!(span.kind, SpanKind::LlmCall);
        assert_eq!(span.metadata.agent_id, Some("frontend".to_string()));
        assert_eq!(span.metadata.model, Some("claude-3-opus".to_string()));
    }

    #[test]
    fn test_span_context() {
        let span = Span::new("test");
        let context = span.context("trace-123");

        assert_eq!(context.trace_id, "trace-123");
        assert_eq!(context.span_id, span.span_id);
    }

    #[test]
    fn test_nested_spans() {
        let parent = Span::new("parent");
        let child = SpanBuilder::new("child").parent(&parent.span_id).build();

        assert_eq!(child.parent_span_id, Some(parent.span_id.clone()));
    }
}
