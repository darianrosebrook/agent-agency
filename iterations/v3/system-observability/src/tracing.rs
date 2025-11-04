//! Distributed tracing functionality
//!
//! Request tracing, span management, and distributed tracing support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
/// Trace context for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub sampled: bool,
    pub baggage: HashMap<String, String>,
}

/// Span represents a single operation within a trace
#[derive(Debug, Clone)]
pub struct Span {
    pub id: String,
    pub trace_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<LogEntry>,
    pub status: SpanStatus,
}

/// Log entry within a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
}

/// Log level for tracing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Span status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}

/// Tracer for creating and managing spans
pub struct Tracer {
    service_name: String,
    active_spans: Arc<RwLock<HashMap<String, Span>>>,
    finished_spans: Arc<RwLock<Vec<Span>>>,
}

impl Tracer {
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            active_spans: Arc::new(RwLock::new(HashMap::new())),
            finished_spans: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start a new root span
    pub async fn start_span(&self, name: &str) -> SpanHandle {
        let span_id = format!("span_{}", chrono::Utc::now().timestamp_millis());
        let trace_id = format!("trace_{}", chrono::Utc::now().timestamp_millis());

        let span = Span {
            id: span_id.clone(),
            trace_id: trace_id.clone(),
            parent_id: None,
            name: name.to_string(),
            start_time: chrono::Utc::now(),
            end_time: None,
            tags: HashMap::new(),
            logs: Vec::new(),
            status: SpanStatus::Unset,
        };

        let mut active_spans = self.active_spans.write().await;
        active_spans.insert(span_id.clone(), span);

        SpanHandle {
            tracer: Arc::new(self.clone()),
            span_id,
        }
    }

    /// Start a child span
    pub async fn start_child_span(&self, parent_span_id: &str, name: &str) -> Result<SpanHandle, TracingError> {
        let active_spans = self.active_spans.read().await;
        let parent_span = active_spans.get(parent_span_id)
            .ok_or_else(|| TracingError::SpanNotFound(parent_span_id.to_string()))?;

        let span_id = format!("span_{}", chrono::Utc::now().timestamp_millis());

        let span = Span {
            id: span_id.clone(),
            trace_id: parent_span.trace_id.clone(),
            parent_id: Some(parent_span_id.to_string()),
            name: name.to_string(),
            start_time: chrono::Utc::now(),
            end_time: None,
            tags: HashMap::new(),
            logs: Vec::new(),
            status: SpanStatus::Unset,
        };

        drop(active_spans); // Release read lock

        let mut active_spans = self.active_spans.write().await;
        active_spans.insert(span_id.clone(), span);

        Ok(SpanHandle {
            tracer: Arc::new(self.clone()),
            span_id,
        })
    }

    /// Get active span
    pub async fn get_active_span(&self, span_id: &str) -> Option<Span> {
        let active_spans = self.active_spans.read().await;
        active_spans.get(span_id).cloned()
    }

    /// Finish a span
    async fn finish_span(&self, span_id: &str, status: SpanStatus) -> Result<(), TracingError> {
        let mut active_spans = self.active_spans.write().await;
        let mut finished_spans = self.finished_spans.write().await;

        if let Some(mut span) = active_spans.remove(span_id) {
            span.end_time = Some(chrono::Utc::now());
            span.status = status;
            finished_spans.push(span);
            Ok(())
        } else {
            Err(TracingError::SpanNotFound(span_id.to_string()))
        }
    }

    /// Get finished spans
    pub async fn get_finished_spans(&self) -> Vec<Span> {
        let finished_spans = self.finished_spans.read().await;
        finished_spans.clone()
    }

    /// Clear finished spans (for memory management)
    pub async fn clear_finished_spans(&self) {
        let mut finished_spans = self.finished_spans.write().await;
        finished_spans.clear();
    }

    /// Extract trace context from headers
    pub fn extract_context(headers: &HashMap<String, String>) -> Option<TraceContext> {
        let trace_id = headers.get("x-trace-id")?;
        let span_id = headers.get("x-span-id")?;
        let parent_span_id = headers.get("x-parent-span-id").cloned();
        let sampled = headers.get("x-sampled")
            .map(|s| s == "true")
            .unwrap_or(true);

        let mut baggage = HashMap::new();
        for (key, value) in headers {
            if key.starts_with("x-baggage-") {
                let baggage_key = key.strip_prefix("x-baggage-").unwrap_or(key);
                baggage.insert(baggage_key.to_string(), value.clone());
            }
        }

        Some(TraceContext {
            trace_id: trace_id.clone(),
            span_id: span_id.clone(),
            parent_span_id,
            sampled,
            baggage,
        })
    }

    /// Inject trace context into headers
    pub fn inject_context(context: &TraceContext) -> HashMap<String, String> {
        let mut headers = HashMap::new();

        headers.insert("x-trace-id".to_string(), context.trace_id.clone());
        headers.insert("x-span-id".to_string(), context.span_id.clone());
        headers.insert("x-sampled".to_string(), context.sampled.to_string());

        if let Some(parent_id) = &context.parent_span_id {
            headers.insert("x-parent-span-id".to_string(), parent_id.clone());
        }

        for (key, value) in &context.baggage {
            headers.insert(format!("x-baggage-{}", key), value.clone());
        }

        headers
    }
}

impl Clone for Tracer {
    fn clone(&self) -> Self {
        Self {
            service_name: self.service_name.clone(),
            active_spans: Arc::clone(&self.active_spans),
            finished_spans: Arc::clone(&self.finished_spans),
        }
    }
}

/// Span handle for safe span management
pub struct SpanHandle {
    tracer: Arc<Tracer>,
    span_id: String,
}

impl SpanHandle {
    /// Set a tag on the span
    pub async fn set_tag(&self, key: &str, value: &str) {
        if let Some(span) = self.tracer.active_spans.write().await.get_mut(&self.span_id) {
            span.tags.insert(key.to_string(), value.to_string());
        }
    }

    /// Log an event on the span
    pub async fn log(&self, level: LogLevel, message: &str, fields: HashMap<String, serde_json::Value>) {
        if let Some(span) = self.tracer.active_spans.write().await.get_mut(&self.span_id) {
            span.logs.push(LogEntry {
                timestamp: chrono::Utc::now(),
                level,
                message: message.to_string(),
                fields,
            });
        }
    }

    /// Finish the span with success
    pub async fn finish_ok(self) {
        let _ = self.tracer.finish_span(&self.span_id, SpanStatus::Ok).await;
    }

    /// Finish the span with error
    pub async fn finish_error(self) {
        let _ = self.tracer.finish_span(&self.span_id, SpanStatus::Error).await;
    }

    /// Get span ID
    pub fn span_id(&self) -> &str {
        &self.span_id
    }
}

impl Drop for SpanHandle {
    fn drop(&mut self) {
        // If span wasn't explicitly finished, finish it as unset
        // Note: This is a simplified implementation. In practice, you'd want
        // to handle this more carefully to avoid blocking in drop.
        let tracer = Arc::clone(&self.tracer);
        let span_id = self.span_id.clone();

        tokio::spawn(async move {
            let _ = tracer.finish_span(&span_id, SpanStatus::Unset).await;
        });
    }
}

/// Tracing errors
#[derive(Debug, thiserror::Error)]
pub enum TracingError {
    #[error("Span not found: {0}")]
    SpanNotFound(String),

    #[error("Invalid trace context: {message}")]
    InvalidContext { message: String },

    #[error("Tracing system error: {message}")]
    SystemError { message: String },
}
