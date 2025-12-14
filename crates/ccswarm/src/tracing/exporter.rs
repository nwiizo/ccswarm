//! Trace exporter for various formats

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::span::SpanStatus;
use super::trace::Trace;

/// Export format for traces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ExportFormat {
    /// JSON format
    #[default]
    Json,
    /// Pretty-printed JSON
    JsonPretty,
    /// OpenTelemetry-compatible JSON
    OpenTelemetry,
    /// Langfuse-compatible format
    Langfuse,
    /// CSV format (flattened)
    Csv,
}

/// Exporter for traces
pub struct TraceExporter {
    /// Custom attributes to add to all exports
    global_attributes: HashMap<String, String>,
}

impl TraceExporter {
    /// Create a new trace exporter
    pub fn new() -> Self {
        Self {
            global_attributes: HashMap::new(),
        }
    }

    /// Add a global attribute
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.global_attributes.insert(key.into(), value.into());
        self
    }

    /// Export traces to the specified format
    pub fn export(&self, traces: &[Trace], format: ExportFormat) -> Result<String, String> {
        match format {
            ExportFormat::Json => self.export_json(traces, false),
            ExportFormat::JsonPretty => self.export_json(traces, true),
            ExportFormat::OpenTelemetry => self.export_opentelemetry(traces),
            ExportFormat::Langfuse => self.export_langfuse(traces),
            ExportFormat::Csv => self.export_csv(traces),
        }
    }

    /// Export as JSON
    fn export_json(&self, traces: &[Trace], pretty: bool) -> Result<String, String> {
        let export_data = JsonExport {
            traces: traces.to_vec(),
            metadata: ExportMetadata {
                exported_at: chrono::Utc::now(),
                format_version: "1.0".to_string(),
                exporter: "ccswarm".to_string(),
                global_attributes: self.global_attributes.clone(),
            },
        };

        if pretty {
            serde_json::to_string_pretty(&export_data)
                .map_err(|e| format!("JSON export failed: {}", e))
        } else {
            serde_json::to_string(&export_data).map_err(|e| format!("JSON export failed: {}", e))
        }
    }

    /// Export as OpenTelemetry-compatible format
    fn export_opentelemetry(&self, traces: &[Trace]) -> Result<String, String> {
        let resource_spans: Vec<OtelResourceSpans> = traces
            .iter()
            .map(|trace| self.trace_to_otel(trace))
            .collect();

        let export = OtelExport { resource_spans };

        serde_json::to_string_pretty(&export)
            .map_err(|e| format!("OpenTelemetry export failed: {}", e))
    }

    /// Convert a trace to OpenTelemetry format
    fn trace_to_otel(&self, trace: &Trace) -> OtelResourceSpans {
        let spans: Vec<OtelSpan> = trace
            .spans
            .iter()
            .map(|span| OtelSpan {
                trace_id: trace.id.as_str().to_string(),
                span_id: span.span_id.clone(),
                parent_span_id: span.parent_span_id.clone().unwrap_or_default(),
                name: span.name.clone(),
                kind: match span.kind {
                    super::span::SpanKind::Internal => 1,
                    super::span::SpanKind::Server => 2,
                    super::span::SpanKind::Client => 3,
                    super::span::SpanKind::Producer => 4,
                    super::span::SpanKind::Consumer => 5,
                    _ => 1,
                },
                start_time_unix_nano: span.start_time.timestamp_nanos_opt().unwrap_or(0) as u64,
                end_time_unix_nano: span
                    .end_time
                    .and_then(|t| t.timestamp_nanos_opt())
                    .unwrap_or(0) as u64,
                attributes: span
                    .metadata
                    .attributes
                    .iter()
                    .map(|(k, v)| OtelAttribute {
                        key: k.clone(),
                        value: OtelValue {
                            string_value: Some(v.to_string()),
                        },
                    })
                    .collect(),
                status: OtelStatus {
                    code: match &span.status {
                        SpanStatus::Ok => 1,
                        SpanStatus::Error { .. } => 2,
                        _ => 0,
                    },
                    message: span.status.error_message().unwrap_or("").to_string(),
                },
                events: span
                    .events
                    .iter()
                    .map(|e| OtelEvent {
                        name: e.name.clone(),
                        time_unix_nano: e.timestamp.timestamp_nanos_opt().unwrap_or(0) as u64,
                        attributes: e
                            .attributes
                            .iter()
                            .map(|(k, v)| OtelAttribute {
                                key: k.clone(),
                                value: OtelValue {
                                    string_value: Some(v.to_string()),
                                },
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect();

        OtelResourceSpans {
            resource: OtelResource {
                attributes: vec![
                    OtelAttribute {
                        key: "service.name".to_string(),
                        value: OtelValue {
                            string_value: Some("ccswarm".to_string()),
                        },
                    },
                    OtelAttribute {
                        key: "service.version".to_string(),
                        value: OtelValue {
                            string_value: Some(env!("CARGO_PKG_VERSION").to_string()),
                        },
                    },
                ],
            },
            scope_spans: vec![OtelScopeSpans {
                scope: OtelScope {
                    name: "ccswarm.tracing".to_string(),
                    version: "1.0.0".to_string(),
                },
                spans,
            }],
        }
    }

    /// Export as Langfuse-compatible format
    fn export_langfuse(&self, traces: &[Trace]) -> Result<String, String> {
        let langfuse_traces: Vec<LangfuseTrace> = traces
            .iter()
            .map(|trace| self.trace_to_langfuse(trace))
            .collect();

        let export = LangfuseExport {
            traces: langfuse_traces,
        };

        serde_json::to_string_pretty(&export).map_err(|e| format!("Langfuse export failed: {}", e))
    }

    /// Convert a trace to Langfuse format
    fn trace_to_langfuse(&self, trace: &Trace) -> LangfuseTrace {
        let observations: Vec<LangfuseObservation> = trace
            .spans
            .iter()
            .map(|span| {
                let observation_type = match span.kind {
                    super::span::SpanKind::LlmCall => "generation",
                    _ => "span",
                };

                LangfuseObservation {
                    id: span.span_id.clone(),
                    trace_id: trace.id.as_str().to_string(),
                    observation_type: observation_type.to_string(),
                    name: span.name.clone(),
                    start_time: span.start_time.to_rfc3339(),
                    end_time: span.end_time.map(|t| t.to_rfc3339()),
                    model: span.metadata.model.clone(),
                    input: None,  // Would need to store prompts
                    output: None, // Would need to store completions
                    usage: span.metadata.tokens_used.as_ref().map(|t| LangfuseUsage {
                        input: t.input_tokens as i64,
                        output: t.output_tokens as i64,
                        total: t.total_tokens as i64,
                    }),
                    level: match &span.status {
                        SpanStatus::Ok => "DEFAULT".to_string(),
                        SpanStatus::Error { .. } => "ERROR".to_string(),
                        SpanStatus::Cancelled => "WARNING".to_string(),
                        SpanStatus::InProgress => "DEBUG".to_string(),
                    },
                    status_message: span.status.error_message().map(|s| s.to_string()),
                    parent_observation_id: span.parent_span_id.clone(),
                    metadata: span.metadata.attributes.clone(),
                }
            })
            .collect();

        LangfuseTrace {
            id: trace.id.as_str().to_string(),
            name: trace.name.clone(),
            user_id: trace.user_id.clone(),
            session_id: trace.session_id.clone(),
            timestamp: trace.start_time.to_rfc3339(),
            tags: trace.tags.keys().cloned().collect(),
            metadata: trace.metadata.attributes.clone(),
            observations,
        }
    }

    /// Export as CSV
    fn export_csv(&self, traces: &[Trace]) -> Result<String, String> {
        let mut csv = String::new();

        // Header
        csv.push_str("trace_id,trace_name,span_id,span_name,parent_span_id,start_time,end_time,duration_ms,status,agent_id,model,input_tokens,output_tokens,cost_usd\n");

        for trace in traces {
            for span in &trace.spans {
                let line = format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                    trace.id.as_str(),
                    escape_csv(&trace.name),
                    &span.span_id,
                    escape_csv(&span.name),
                    span.parent_span_id.as_deref().unwrap_or(""),
                    span.start_time.to_rfc3339(),
                    span.end_time.map(|t| t.to_rfc3339()).unwrap_or_default(),
                    span.duration_ms().unwrap_or(0),
                    match &span.status {
                        SpanStatus::Ok => "ok",
                        SpanStatus::Error { .. } => "error",
                        SpanStatus::Cancelled => "cancelled",
                        SpanStatus::InProgress => "in_progress",
                    },
                    span.metadata.agent_id.as_deref().unwrap_or(""),
                    span.metadata.model.as_deref().unwrap_or(""),
                    span.metadata
                        .tokens_used
                        .as_ref()
                        .map(|t| t.input_tokens)
                        .unwrap_or(0),
                    span.metadata
                        .tokens_used
                        .as_ref()
                        .map(|t| t.output_tokens)
                        .unwrap_or(0),
                    span.metadata.cost_usd.unwrap_or(0.0),
                );
                csv.push_str(&line);
            }
        }

        Ok(csv)
    }
}

impl Default for TraceExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape a string for CSV
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// JSON export structures
#[derive(Debug, Serialize, Deserialize)]
struct JsonExport {
    traces: Vec<Trace>,
    metadata: ExportMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportMetadata {
    exported_at: chrono::DateTime<chrono::Utc>,
    format_version: String,
    exporter: String,
    global_attributes: HashMap<String, String>,
}

// OpenTelemetry structures
#[derive(Debug, Serialize, Deserialize)]
struct OtelExport {
    #[serde(rename = "resourceSpans")]
    resource_spans: Vec<OtelResourceSpans>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtelResourceSpans {
    resource: OtelResource,
    #[serde(rename = "scopeSpans")]
    scope_spans: Vec<OtelScopeSpans>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtelResource {
    attributes: Vec<OtelAttribute>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtelScopeSpans {
    scope: OtelScope,
    spans: Vec<OtelSpan>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtelScope {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtelSpan {
    #[serde(rename = "traceId")]
    trace_id: String,
    #[serde(rename = "spanId")]
    span_id: String,
    #[serde(rename = "parentSpanId")]
    parent_span_id: String,
    name: String,
    kind: u8,
    #[serde(rename = "startTimeUnixNano")]
    start_time_unix_nano: u64,
    #[serde(rename = "endTimeUnixNano")]
    end_time_unix_nano: u64,
    attributes: Vec<OtelAttribute>,
    status: OtelStatus,
    events: Vec<OtelEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtelAttribute {
    key: String,
    value: OtelValue,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtelValue {
    #[serde(rename = "stringValue")]
    string_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtelStatus {
    code: u8,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OtelEvent {
    name: String,
    #[serde(rename = "timeUnixNano")]
    time_unix_nano: u64,
    attributes: Vec<OtelAttribute>,
}

// Langfuse structures
#[derive(Debug, Serialize, Deserialize)]
struct LangfuseExport {
    traces: Vec<LangfuseTrace>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LangfuseTrace {
    id: String,
    name: String,
    user_id: Option<String>,
    session_id: Option<String>,
    timestamp: String,
    tags: Vec<String>,
    metadata: HashMap<String, serde_json::Value>,
    observations: Vec<LangfuseObservation>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LangfuseObservation {
    id: String,
    trace_id: String,
    #[serde(rename = "type")]
    observation_type: String,
    name: String,
    start_time: String,
    end_time: Option<String>,
    model: Option<String>,
    input: Option<serde_json::Value>,
    output: Option<serde_json::Value>,
    usage: Option<LangfuseUsage>,
    level: String,
    status_message: Option<String>,
    parent_observation_id: Option<String>,
    metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LangfuseUsage {
    input: i64,
    output: i64,
    total: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::{SpanEvent, TokenUsage, TraceMetadata};

    fn create_test_trace() -> Trace {
        let mut trace = Trace::new("test_operation");
        trace.user_id = Some("user-123".to_string());
        trace.tags.insert("env".to_string(), "test".to_string());

        let span_id = trace.start_span("llm_call", None);
        let metadata = TraceMetadata::new()
            .with_tokens(TokenUsage::new(100, 50))
            .with_cost(0.01)
            .with_model("claude-3-opus")
            .with_agent("frontend");

        trace.add_event(&span_id, SpanEvent::new("prompt_sent"));
        trace.end_span(&span_id, SpanStatus::Ok, Some(metadata));
        trace.end();

        trace
    }

    #[test]
    fn test_export_json() {
        let exporter = TraceExporter::new();
        let trace = create_test_trace();
        let result = exporter.export(&[trace], ExportFormat::Json);

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("test_operation"));
        assert!(json.contains("llm_call"));
    }

    #[test]
    fn test_export_json_pretty() {
        let exporter = TraceExporter::new();
        let trace = create_test_trace();
        let result = exporter.export(&[trace], ExportFormat::JsonPretty);

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains('\n')); // Pretty printed has newlines
    }

    #[test]
    fn test_export_opentelemetry() {
        let exporter = TraceExporter::new();
        let trace = create_test_trace();
        let result = exporter.export(&[trace], ExportFormat::OpenTelemetry);

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("resourceSpans"));
        assert!(json.contains("scopeSpans"));
        assert!(json.contains("traceId"));
    }

    #[test]
    fn test_export_langfuse() {
        let exporter = TraceExporter::new();
        let trace = create_test_trace();
        let result = exporter.export(&[trace], ExportFormat::Langfuse);

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("observations"));
        assert!(json.contains("user-123"));
    }

    #[test]
    fn test_export_csv() {
        let exporter = TraceExporter::new();
        let trace = create_test_trace();
        let result = exporter.export(&[trace], ExportFormat::Csv);

        assert!(result.is_ok());
        let csv = result.unwrap();
        assert!(csv.contains("trace_id,trace_name"));
        assert!(csv.contains("llm_call"));
        assert!(csv.contains("claude-3-opus"));
    }

    #[test]
    fn test_exporter_with_global_attributes() {
        let exporter = TraceExporter::new()
            .with_attribute("environment", "production")
            .with_attribute("version", "1.0.0");

        let trace = create_test_trace();
        let result = exporter.export(&[trace], ExportFormat::Json);

        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains("production"));
    }

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("simple"), "simple");
        assert_eq!(escape_csv("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv("with\"quote"), "\"with\"\"quote\"");
    }
}
