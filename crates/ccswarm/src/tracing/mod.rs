//! Agent Tracing Module
//!
//! Provides observability and tracing capabilities for multi-agent systems.
//! Compatible with OpenTelemetry and Langfuse for external integration.

mod collector;
mod exporter;
mod span;
mod trace;

pub use collector::{TraceCollector, TraceCollectorConfig};
pub use exporter::{ExportFormat, TraceExporter};
pub use span::{Span, SpanBuilder, SpanContext, SpanStatus};
pub use trace::{Trace, TraceId};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

/// Metadata attached to spans and traces
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TraceMetadata {
    /// Custom key-value attributes
    pub attributes: HashMap<String, serde_json::Value>,
    /// Token count for LLM calls
    pub tokens_used: Option<TokenUsage>,
    /// Estimated cost in USD
    pub cost_usd: Option<f64>,
    /// Model name if LLM call
    pub model: Option<String>,
    /// Agent that executed this operation
    pub agent_id: Option<String>,
    /// Task associated with this trace
    pub task_id: Option<String>,
}

impl TraceMetadata {
    /// Create new empty metadata
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a custom attribute
    pub fn with_attribute(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Set token usage
    pub fn with_tokens(mut self, usage: TokenUsage) -> Self {
        self.tokens_used = Some(usage);
        self
    }

    /// Set cost
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost_usd = Some(cost);
        self
    }

    /// Set model name
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set agent ID
    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    /// Set task ID
    pub fn with_task(mut self, task_id: impl Into<String>) -> Self {
        self.task_id = Some(task_id.into());
        self
    }
}

/// Token usage for LLM calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Input/prompt tokens
    pub input_tokens: u32,
    /// Output/completion tokens
    pub output_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

impl TokenUsage {
    /// Create new token usage
    pub fn new(input: u32, output: u32) -> Self {
        Self {
            input_tokens: input,
            output_tokens: output,
            total_tokens: input + output,
        }
    }
}

/// Event that occurred during a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    /// Event name
    pub name: String,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Event attributes
    pub attributes: HashMap<String, serde_json::Value>,
}

impl SpanEvent {
    /// Create a new event
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            timestamp: Utc::now(),
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute to the event
    pub fn with_attribute(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// Messages for the tracing actor (channel-based pattern)
#[derive(Debug)]
pub enum TracingMsg {
    /// Start a new trace
    StartTrace {
        name: String,
        reply: oneshot::Sender<Option<TraceId>>,
    },
    /// Start a span within a trace
    StartSpan {
        trace_id: TraceId,
        name: String,
        parent_span_id: Option<String>,
        reply: oneshot::Sender<Option<String>>,
    },
    /// End a span
    EndSpan {
        trace_id: TraceId,
        span_id: String,
        status: SpanStatus,
        metadata: Option<TraceMetadata>,
    },
    /// Add event to span
    AddEvent {
        trace_id: TraceId,
        span_id: String,
        event: SpanEvent,
    },
    /// End a trace
    EndTrace { trace_id: TraceId },
    /// Get a trace by ID
    GetTrace {
        trace_id: TraceId,
        reply: oneshot::Sender<Option<Trace>>,
    },
    /// Get all traces
    GetAllTraces { reply: oneshot::Sender<Vec<Trace>> },
    /// Get traces by agent
    GetTracesByAgent {
        agent_id: String,
        reply: oneshot::Sender<Vec<Trace>>,
    },
    /// Export traces
    Export {
        format: ExportFormat,
        trace_ids: Option<Vec<TraceId>>,
        reply: oneshot::Sender<Result<String, String>>,
    },
    /// Get statistics
    GetStats {
        reply: oneshot::Sender<TracingStats>,
    },
    /// Clear all traces
    Clear,
    /// Shutdown the actor
    Shutdown,
}

/// Handle to communicate with the tracing actor (channel-based pattern)
#[derive(Clone)]
pub struct TracingSystem {
    tx: mpsc::Sender<TracingMsg>,
    config: TracingConfig,
}

/// Configuration for the tracing system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Whether tracing is enabled
    pub enabled: bool,
    /// Maximum traces to keep in memory
    pub max_traces: usize,
    /// Auto-export interval in seconds (0 = disabled)
    pub auto_export_interval_secs: u64,
    /// Default export format
    pub default_format: ExportFormat,
    /// Export path for file exports
    pub export_path: Option<String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_traces: 1000,
            auto_export_interval_secs: 0,
            default_format: ExportFormat::Json,
            export_path: None,
        }
    }
}

/// The tracing actor that owns state and processes messages
struct TracingActor {
    rx: mpsc::Receiver<TracingMsg>,
    collector: TraceCollector,
    exporter: TraceExporter,
    config: TracingConfig,
}

impl TracingActor {
    /// Run the actor event loop
    async fn run(mut self) {
        while let Some(msg) = self.rx.recv().await {
            match msg {
                TracingMsg::StartTrace { name, reply } => {
                    let result = if self.config.enabled {
                        Some(self.collector.start_trace(name))
                    } else {
                        None
                    };
                    let _ = reply.send(result);
                }
                TracingMsg::StartSpan {
                    trace_id,
                    name,
                    parent_span_id,
                    reply,
                } => {
                    let result = if self.config.enabled {
                        self.collector.start_span(&trace_id, name, parent_span_id)
                    } else {
                        None
                    };
                    let _ = reply.send(result);
                }
                TracingMsg::EndSpan {
                    trace_id,
                    span_id,
                    status,
                    metadata,
                } => {
                    if self.config.enabled {
                        self.collector
                            .end_span(&trace_id, &span_id, status, metadata);
                    }
                }
                TracingMsg::AddEvent {
                    trace_id,
                    span_id,
                    event,
                } => {
                    if self.config.enabled {
                        self.collector.add_event(&trace_id, &span_id, event);
                    }
                }
                TracingMsg::EndTrace { trace_id } => {
                    if self.config.enabled {
                        self.collector.end_trace(&trace_id);
                    }
                }
                TracingMsg::GetTrace { trace_id, reply } => {
                    let result = self.collector.get_trace(&trace_id);
                    let _ = reply.send(result);
                }
                TracingMsg::GetAllTraces { reply } => {
                    let result = self.collector.get_all_traces();
                    let _ = reply.send(result);
                }
                TracingMsg::GetTracesByAgent { agent_id, reply } => {
                    let result = self.collector.get_traces_by_agent(&agent_id);
                    let _ = reply.send(result);
                }
                TracingMsg::Export {
                    format,
                    trace_ids,
                    reply,
                } => {
                    let traces: Vec<Trace> = if let Some(ids) = trace_ids {
                        ids.iter()
                            .filter_map(|id| self.collector.get_trace(id))
                            .collect()
                    } else {
                        self.collector.get_all_traces()
                    };
                    let result = self.exporter.export(&traces, format);
                    let _ = reply.send(result);
                }
                TracingMsg::GetStats { reply } => {
                    let result = self.collector.get_stats();
                    let _ = reply.send(result);
                }
                TracingMsg::Clear => {
                    self.collector.clear();
                }
                TracingMsg::Shutdown => {
                    break;
                }
            }
        }
    }
}

impl TracingSystem {
    /// Create a new tracing system with default config
    pub fn new() -> Self {
        Self::with_config(TracingConfig::default())
    }

    /// Create a new tracing system with custom config (spawns background actor)
    pub fn with_config(config: TracingConfig) -> Self {
        let (tx, rx) = mpsc::channel(100);

        let collector_config = TraceCollectorConfig {
            max_traces: config.max_traces,
            auto_cleanup: true,
            cleanup_interval_secs: 3600,
        };

        let actor = TracingActor {
            rx,
            collector: TraceCollector::new(collector_config),
            exporter: TraceExporter::new(),
            config: config.clone(),
        };

        // Spawn the actor as a background task
        tokio::spawn(async move {
            actor.run().await;
        });

        Self { tx, config }
    }

    /// Check if tracing is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Start a new trace
    pub async fn start_trace(&self, name: impl Into<String>) -> Option<TraceId> {
        if !self.config.enabled {
            return None;
        }

        let (reply, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(TracingMsg::StartTrace {
                name: name.into(),
                reply,
            })
            .await;
        rx.await.ok().flatten()
    }

    /// Start a new span within a trace
    pub async fn start_span(
        &self,
        trace_id: &TraceId,
        name: impl Into<String>,
        parent_span_id: Option<String>,
    ) -> Option<String> {
        if !self.config.enabled {
            return None;
        }

        let (reply, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(TracingMsg::StartSpan {
                trace_id: trace_id.clone(),
                name: name.into(),
                parent_span_id,
                reply,
            })
            .await;
        rx.await.ok().flatten()
    }

    /// End a span
    pub async fn end_span(
        &self,
        trace_id: &TraceId,
        span_id: &str,
        status: SpanStatus,
        metadata: Option<TraceMetadata>,
    ) {
        if !self.config.enabled {
            return;
        }

        let _ = self
            .tx
            .send(TracingMsg::EndSpan {
                trace_id: trace_id.clone(),
                span_id: span_id.to_string(),
                status,
                metadata,
            })
            .await;
    }

    /// Add an event to a span
    pub async fn add_event(&self, trace_id: &TraceId, span_id: &str, event: SpanEvent) {
        if !self.config.enabled {
            return;
        }

        let _ = self
            .tx
            .send(TracingMsg::AddEvent {
                trace_id: trace_id.clone(),
                span_id: span_id.to_string(),
                event,
            })
            .await;
    }

    /// End a trace
    pub async fn end_trace(&self, trace_id: &TraceId) {
        if !self.config.enabled {
            return;
        }

        let _ = self
            .tx
            .send(TracingMsg::EndTrace {
                trace_id: trace_id.clone(),
            })
            .await;
    }

    /// Get a trace by ID
    pub async fn get_trace(&self, trace_id: &TraceId) -> Option<Trace> {
        let (reply, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(TracingMsg::GetTrace {
                trace_id: trace_id.clone(),
                reply,
            })
            .await;
        rx.await.ok().flatten()
    }

    /// Get all traces
    pub async fn get_all_traces(&self) -> Vec<Trace> {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(TracingMsg::GetAllTraces { reply }).await;
        rx.await.unwrap_or_default()
    }

    /// Get traces for a specific agent
    pub async fn get_traces_by_agent(&self, agent_id: &str) -> Vec<Trace> {
        let (reply, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(TracingMsg::GetTracesByAgent {
                agent_id: agent_id.to_string(),
                reply,
            })
            .await;
        rx.await.unwrap_or_default()
    }

    /// Export traces to a specific format
    pub async fn export(
        &self,
        format: ExportFormat,
        trace_ids: Option<Vec<TraceId>>,
    ) -> Result<String, String> {
        let (reply, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(TracingMsg::Export {
                format,
                trace_ids,
                reply,
            })
            .await;
        rx.await.map_err(|_| "Channel closed".to_string())?
    }

    /// Export traces to file
    pub async fn export_to_file(&self, path: &str, format: ExportFormat) -> Result<(), String> {
        let content = self.export(format, None).await?;
        std::fs::write(path, content).map_err(|e| format!("Failed to write file: {}", e))
    }

    /// Get tracing statistics
    pub async fn get_stats(&self) -> TracingStats {
        let (reply, rx) = oneshot::channel();
        let _ = self.tx.send(TracingMsg::GetStats { reply }).await;
        rx.await.unwrap_or(TracingStats {
            total_traces: 0,
            active_traces: 0,
            completed_traces: 0,
            total_spans: 0,
            total_tokens: 0,
            total_cost_usd: 0.0,
            traces_per_agent: HashMap::new(),
            avg_span_duration_ms: 0.0,
        })
    }

    /// Clear all traces
    pub async fn clear(&self) {
        let _ = self.tx.send(TracingMsg::Clear).await;
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) {
        let _ = self.tx.send(TracingMsg::Shutdown).await;
    }
}

impl Default for TracingSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for the tracing system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingStats {
    /// Total number of traces
    pub total_traces: usize,
    /// Active (ongoing) traces
    pub active_traces: usize,
    /// Completed traces
    pub completed_traces: usize,
    /// Total spans across all traces
    pub total_spans: usize,
    /// Total tokens used
    pub total_tokens: u64,
    /// Total cost in USD
    pub total_cost_usd: f64,
    /// Traces per agent
    pub traces_per_agent: HashMap<String, usize>,
    /// Average span duration in milliseconds
    pub avg_span_duration_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tracing_system_basic() {
        let system = TracingSystem::new();

        // Start a trace
        let trace_id = system.start_trace("test_operation").await.unwrap();

        // Start a span
        let span_id = system
            .start_span(&trace_id, "llm_call", None)
            .await
            .unwrap();

        // Add an event
        let event = SpanEvent::new("prompt_sent").with_attribute("prompt_length", 100);
        system.add_event(&trace_id, &span_id, event).await;

        // End the span
        let metadata = TraceMetadata::new()
            .with_tokens(TokenUsage::new(100, 50))
            .with_cost(0.01)
            .with_model("claude-3-opus");
        system
            .end_span(&trace_id, &span_id, SpanStatus::Ok, Some(metadata))
            .await;

        // End the trace
        system.end_trace(&trace_id).await;

        // Verify trace was recorded
        let trace = system.get_trace(&trace_id).await.unwrap();
        assert_eq!(trace.name, "test_operation");
        assert_eq!(trace.spans.len(), 1);
        assert!(trace.end_time.is_some());
    }

    #[tokio::test]
    async fn test_tracing_disabled() {
        let config = TracingConfig {
            enabled: false,
            ..Default::default()
        };
        let system = TracingSystem::with_config(config);

        // Should return None when disabled
        let trace_id = system.start_trace("test").await;
        assert!(trace_id.is_none());
    }

    #[tokio::test]
    async fn test_nested_spans() {
        let system = TracingSystem::new();
        let trace_id = system.start_trace("parent_operation").await.unwrap();

        // Parent span
        let parent_span = system.start_span(&trace_id, "parent", None).await.unwrap();

        // Child span
        let child_span = system
            .start_span(&trace_id, "child", Some(parent_span.clone()))
            .await
            .unwrap();

        // End child first
        system
            .end_span(&trace_id, &child_span, SpanStatus::Ok, None)
            .await;

        // End parent
        system
            .end_span(&trace_id, &parent_span, SpanStatus::Ok, None)
            .await;

        system.end_trace(&trace_id).await;

        let trace = system.get_trace(&trace_id).await.unwrap();
        assert_eq!(trace.spans.len(), 2);

        // Verify parent-child relationship
        let child = trace.spans.iter().find(|s| s.name == "child").unwrap();
        assert_eq!(child.parent_span_id, Some(parent_span));
    }

    #[tokio::test]
    async fn test_export_json() {
        let system = TracingSystem::new();
        let trace_id = system.start_trace("export_test").await.unwrap();
        let span_id = system
            .start_span(&trace_id, "test_span", None)
            .await
            .unwrap();
        system
            .end_span(&trace_id, &span_id, SpanStatus::Ok, None)
            .await;
        system.end_trace(&trace_id).await;

        let json = system.export(ExportFormat::Json, None).await.unwrap();
        assert!(json.contains("export_test"));
        assert!(json.contains("test_span"));
    }

    #[tokio::test]
    async fn test_stats() {
        let system = TracingSystem::new();

        // Create some traces
        for i in 0..3 {
            let trace_id = system.start_trace(format!("trace_{}", i)).await.unwrap();
            let span_id = system.start_span(&trace_id, "span", None).await.unwrap();

            let metadata = TraceMetadata::new()
                .with_tokens(TokenUsage::new(100, 50))
                .with_cost(0.01)
                .with_agent("agent-1");

            system
                .end_span(&trace_id, &span_id, SpanStatus::Ok, Some(metadata))
                .await;
            system.end_trace(&trace_id).await;
        }

        let stats = system.get_stats().await;
        assert_eq!(stats.total_traces, 3);
        assert_eq!(stats.completed_traces, 3);
        assert_eq!(stats.total_spans, 3);
        assert_eq!(stats.total_tokens, 450); // 150 * 3
        assert!((stats.total_cost_usd - 0.03).abs() < 0.001);
    }

    #[test]
    fn test_trace_metadata() {
        let metadata = TraceMetadata::new()
            .with_attribute("key", "value")
            .with_tokens(TokenUsage::new(100, 50))
            .with_cost(0.01)
            .with_model("gpt-4")
            .with_agent("frontend")
            .with_task("task-123");

        assert_eq!(metadata.tokens_used.unwrap().total_tokens, 150);
        assert_eq!(metadata.cost_usd, Some(0.01));
        assert_eq!(metadata.model, Some("gpt-4".to_string()));
        assert_eq!(metadata.agent_id, Some("frontend".to_string()));
        assert_eq!(metadata.task_id, Some("task-123".to_string()));
    }
}
