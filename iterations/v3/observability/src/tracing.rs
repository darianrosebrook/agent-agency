//! Distributed tracing implementation

use opentelemetry::{global, trace::SpanBuilder};
use opentelemetry_sdk::trace;
use opentelemetry_otlp::WithExportConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub service_name: String,
    pub operation: String,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanInfo {
    pub name: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<u64>,
    pub attributes: HashMap<String, serde_json::Value>,
    pub events: Vec<SpanEvent>,
    pub status: SpanStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub attributes: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceConfig {
    pub service_name: String,
    pub service_version: String,
    pub enable_otlp: bool,
    pub otlp_endpoint: Option<String>,
    pub sample_rate: f64,
    pub max_attributes: usize,
    pub max_events: usize,
}

impl Default for TraceConfig {
    fn default() -> Self {
        Self {
            service_name: "agent-agency".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            enable_otlp: false,
            otlp_endpoint: None,
            sample_rate: 1.0,
            max_attributes: 100,
            max_events: 100,
        }
    }
}

#[derive(Debug)]
pub struct TraceCollector {
    config: TraceConfig,
    active_spans: Arc<RwLock<HashMap<String, SpanInfo>>>,
    completed_traces: Arc<RwLock<Vec<TraceInfo>>>,
    tracer: Option<opentelemetry::trace::TracerProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceInfo {
    pub trace_id: String,
    pub root_span: SpanInfo,
    pub child_spans: Vec<SpanInfo>,
    pub duration_ms: u64,
    pub status: TraceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceStatus {
    Completed,
    Error,
    Timeout,
}

impl TraceCollector {
    pub fn new(config: TraceConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let tracer = if config.enable_otlp {
            if let Some(endpoint) = &config.otlp_endpoint {
                let exporter = opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint);

                let tracer_provider = opentelemetry_otlp::new_pipeline()
                    .tracing()
                    .with_exporter(exporter)
                    .with_trace_config(trace::config().with_sampler(
                        trace::Sampler::TraceIdRatioBased(config.sample_rate),
                    ))
                    .install_batch(opentelemetry_sdk::runtime::Tokio)?;

                global::set_tracer_provider(tracer_provider);
                Some(global::tracer("agent-agency"))
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            config,
            active_spans: Arc::new(RwLock::new(HashMap::new())),
            completed_traces: Arc::new(RwLock::new(Vec::new())),
            tracer,
        })
    }

    /// Start a new trace span
    pub async fn start_span(
        &self,
        operation: &str,
        parent_trace_id: Option<&str>,
        attributes: HashMap<String, serde_json::Value>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let span_id = uuid::Uuid::new_v4().to_string();
        let trace_id = parent_trace_id.unwrap_or(&span_id).to_string();

        let span_info = SpanInfo {
            name: operation.to_string(),
            start_time: chrono::Utc::now(),
            end_time: None,
            duration_ms: None,
            attributes: attributes.into_iter().take(self.config.max_attributes).collect(),
            events: Vec::new(),
            status: SpanStatus::Unset,
        };

        let mut active_spans = self.active_spans.write().await;
        active_spans.insert(span_id.clone(), span_info);

        // Create OpenTelemetry span if enabled
        if let Some(tracer) = &self.tracer {
            let mut span_builder = SpanBuilder::from_name(operation.to_string());
            let mut attributes = Vec::new();

            // Set trace and span IDs
            attributes.push(opentelemetry::KeyValue::new("trace.id", trace_id.clone()));
            attributes.push(opentelemetry::KeyValue::new("span.id", span_id.clone()));

            if let Some(parent_id) = parent_trace_id {
                attributes.push(opentelemetry::KeyValue::new("parent.span.id", parent_id.to_string()));
            }

            let _otel_span = tracer.build_with_context(span_builder, &opentelemetry::Context::new());
            // Store span in context for later use
        }

        Ok(span_id)
    }

    /// End a trace span
    pub async fn end_span(&self, span_id: &str, status: SpanStatus) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut active_spans = self.active_spans.write().await;

        if let Some(span_info) = active_spans.get_mut(span_id) {
            span_info.end_time = Some(chrono::Utc::now());
            span_info.status = status.clone();

            if let Some(end) = span_info.end_time {
                let start = span_info.start_time;
                span_info.duration_ms = Some((end - start).num_milliseconds() as u64);
            }

            // Check if this is a root span (no parent trace ID different from span ID)
            let is_root = span_id == self.extract_trace_id(span_id).await.unwrap_or_default();

            if is_root {
                // Complete the trace
                self.complete_trace(span_id).await?;
            }
        }

        Ok(())
    }

    /// Add an event to a span
    pub async fn add_span_event(
        &self,
        span_id: &str,
        event_name: &str,
        attributes: HashMap<String, serde_json::Value>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut active_spans = self.active_spans.write().await;

        if let Some(span_info) = active_spans.get_mut(span_id) {
            if span_info.events.len() < self.config.max_events {
                span_info.events.push(SpanEvent {
                    name: event_name.to_string(),
                    timestamp: chrono::Utc::now(),
                    attributes,
                });
            }
        }

        Ok(())
    }

    /// Add attributes to a span
    pub async fn add_span_attributes(
        &self,
        span_id: &str,
        attributes: HashMap<String, serde_json::Value>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut active_spans = self.active_spans.write().await;

        if let Some(span_info) = active_spans.get_mut(span_id) {
            for (key, value) in attributes {
                if span_info.attributes.len() < self.config.max_attributes {
                    span_info.attributes.insert(key, value);
                }
            }
        }

        Ok(())
    }

    /// Get span information
    pub async fn get_span_info(&self, span_id: &str) -> Option<SpanInfo> {
        let active_spans = self.active_spans.read().await;
        active_spans.get(span_id).cloned()
    }

    /// Get all active spans
    pub async fn get_active_spans(&self) -> Vec<(String, SpanInfo)> {
        let active_spans = self.active_spans.read().await;
        active_spans.iter()
            .map(|(id, info)| (id.clone(), info.clone()))
            .collect()
    }

    /// Get completed traces
    pub async fn get_completed_traces(&self, limit: usize) -> Vec<TraceInfo> {
        let traces = self.completed_traces.read().await;
        traces.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// Create a child span
    pub async fn create_child_span(
        &self,
        parent_span_id: &str,
        operation: &str,
        attributes: HashMap<String, serde_json::Value>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let trace_id = self.extract_trace_id(parent_span_id).await
            .unwrap_or_else(|| parent_span_id.to_string());

        self.start_span(operation, Some(&trace_id), attributes).await
    }

    /// Extract trace context for propagation
    pub async fn extract_trace_context(&self, span_id: &str) -> Option<TraceContext> {
        let active_spans = self.active_spans.read().await;

        if let Some(span_info) = active_spans.get(span_id) {
            let trace_id = self.extract_trace_id(span_id).await?;
            let parent_span_id = if trace_id != *span_id {
                Some(span_id.to_string())
            } else {
                None
            };

            Some(TraceContext {
                trace_id,
                span_id: span_id.to_string(),
                parent_span_id,
                service_name: self.config.service_name.clone(),
                operation: span_info.name.clone(),
                tags: HashMap::new(), // Could extract from attributes
            })
        } else {
            None
        }
    }

    /// Inject trace context into headers for propagation
    pub async fn inject_trace_context(&self, span_id: &str) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        if let Some(context) = self.extract_trace_context(span_id).await {
            headers.insert("x-trace-id".to_string(), context.trace_id);
            headers.insert("x-span-id".to_string(), context.span_id);
            if let Some(parent_id) = context.parent_span_id {
                headers.insert("x-parent-span-id".to_string(), parent_id);
            }
        }

        headers
    }

    /// Extract trace context from headers
    pub async fn extract_from_headers(&self, headers: &HashMap<String, String>) -> Option<TraceContext> {
        let trace_id = headers.get("x-trace-id")?.clone();
        let span_id = headers.get("x-span-id")?.clone();
        let parent_span_id = headers.get("x-parent-span-id").cloned();

        Some(TraceContext {
            trace_id,
            span_id,
            parent_span_id,
            service_name: self.config.service_name.clone(),
            operation: "unknown".to_string(),
            tags: HashMap::new(),
        })
    }

    async fn complete_trace(&self, root_span_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut active_spans = self.active_spans.write().await;
        let mut completed_traces = self.completed_traces.write().await;

        if let Some(root_span) = active_spans.remove(root_span_id) {
            let trace_id = root_span_id.to_string();

            // Find all child spans for this trace
            let child_spans: Vec<SpanInfo> = active_spans
                .iter()
                .filter_map(|(span_id, span_info)| {
                    if self.extract_trace_id(span_id) == Some(trace_id.clone()) && span_id != root_span_id {
                        Some(span_info.clone())
                    } else {
                        None
                    }
                })
                .collect();

            let duration_ms = root_span.duration_ms.unwrap_or(0);
            let status = match root_span.status {
                SpanStatus::Error => TraceStatus::Error,
                _ => TraceStatus::Completed,
            };

            let trace_info = TraceInfo {
                trace_id,
                root_span,
                child_spans,
                duration_ms,
                status,
            };

            completed_traces.push(trace_info);

            // Clean up old traces (keep last 1000)
            while completed_traces.len() > 1000 {
                completed_traces.remove(0);
            }
        }

        Ok(())
    }

    async fn extract_trace_id(&self, span_id: &str) -> Option<String> {
        // TODO: Implement trace hierarchy tracking with the following requirements:
        // 1. Trace hierarchy management: Track and manage trace hierarchy relationships
        //    - Track trace hierarchy and parent-child span relationships
        //    - Handle trace hierarchy validation and integrity
        //    - Implement trace hierarchy optimization and performance
        // 2. Span relationship tracking: Track span relationships and dependencies
        //    - Track span parent-child relationships and dependencies
        //    - Handle span relationship validation and quality assurance
        //    - Implement span relationship optimization and caching
        // 3. Trace ID extraction: Implement proper trace ID extraction algorithms
        //    - Extract trace IDs from span hierarchies and relationships
        //    - Handle trace ID extraction optimization and performance
        //    - Implement trace ID extraction validation and quality assurance
        // 4. Trace analytics: Analyze trace hierarchies and relationships
        //    - Generate trace hierarchy analytics and insights
        //    - Handle trace analytics optimization and reporting
        //    - Ensure trace hierarchy tracking meets performance and accuracy standards
        Some(span_id.to_string())
    }
}

// Helper functions for common tracing patterns
pub async fn trace_operation<F, T>(
    collector: &TraceCollector,
    operation: &str,
    attributes: HashMap<String, serde_json::Value>,
    f: F,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    F: FnOnce() -> Result<T, Box<dyn std::error::Error + Send + Sync>>,
{
    let span_id = collector.start_span(operation, None, attributes).await?;

    let start = std::time::Instant::now();
    let result = f();
    let duration = start.elapsed();

    let status = if result.is_ok() {
        SpanStatus::Ok
    } else {
        SpanStatus::Error
    };

    collector.end_span(&span_id, status).await?;

    result
}

pub async fn trace_async_operation<F, Fut, T>(
    collector: &TraceCollector,
    operation: &str,
    attributes: HashMap<String, serde_json::Value>,
    f: F,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
{
    let span_id = collector.start_span(operation, None, attributes).await?;

    let start = std::time::Instant::now();
    let result = f().await;
    let duration = start.elapsed();

    let status = if result.is_ok() {
        SpanStatus::Ok
    } else {
        SpanStatus::Error
    };

    collector.end_span(&span_id, status).await?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_span_creation_and_completion() {
        let config = TraceConfig {
            enable_otlp: false,
            ..Default::default()
        };

        let collector = TraceCollector::new(config).unwrap();

        // Start a span
        let span_id = collector.start_span(
            "test_operation",
            None,
            HashMap::from([("key".to_string(), serde_json::json!("value"))]),
        ).await.unwrap();

        // Add an event
        collector.add_span_event(
            &span_id,
            "test_event",
            HashMap::from([("event_key".to_string(), serde_json::json!("event_value"))]),
        ).await.unwrap();

        // End the span
        collector.end_span(&span_id, SpanStatus::Ok).await.unwrap();

        // Check that it's completed
        let completed = collector.get_completed_traces(10).await;
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].trace_id, span_id);
        assert_eq!(completed[0].status, TraceStatus::Completed);
    }

    #[tokio::test]
    async fn test_child_span_creation() {
        let config = TraceConfig {
            enable_otlp: false,
            ..Default::default()
        };

        let collector = TraceCollector::new(config).unwrap();

        // Start parent span
        let parent_span_id = collector.start_span(
            "parent_operation",
            None,
            HashMap::new(),
        ).await.unwrap();

        // Start child span
        let child_span_id = collector.create_child_span(
            &parent_span_id,
            "child_operation",
            HashMap::new(),
        ).await.unwrap();

        // End child span
        collector.end_span(&child_span_id, SpanStatus::Ok).await.unwrap();

        // End parent span
        collector.end_span(&parent_span_id, SpanStatus::Ok).await.unwrap();

        // Check completed traces
        let completed = collector.get_completed_traces(10).await;
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].child_spans.len(), 1);
    }
}
