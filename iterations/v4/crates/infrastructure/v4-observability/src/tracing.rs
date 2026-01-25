//! Distributed Tracing
//!
//! Lightweight tracing for tracking request flows through the system.

use std::collections::HashMap;
use std::sync::RwLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Trace identifier
pub type TraceId = String;

/// Span identifier
pub type SpanId = String;

/// Span status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    Unset,
    Ok,
    Error,
}

/// A span in a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_id: Option<SpanId>,
    pub name: String,
    pub status: SpanStatus,
    pub attributes: HashMap<String, String>,
    pub events: Vec<SpanEvent>,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
}

impl Span {
    pub fn new(trace_id: TraceId, name: impl Into<String>) -> Self {
        Self {
            trace_id,
            span_id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            name: name.into(),
            status: SpanStatus::Unset,
            attributes: HashMap::new(),
            events: Vec::new(),
            start_time: Utc::now(),
            end_time: None,
        }
    }

    pub fn with_parent(mut self, parent_id: SpanId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    pub fn add_event(&mut self, name: impl Into<String>) {
        self.events.push(SpanEvent {
            name: name.into(),
            timestamp: Utc::now(),
            attributes: HashMap::new(),
        });
    }

    pub fn add_event_with_attrs(
        &mut self,
        name: impl Into<String>,
        attributes: HashMap<String, String>,
    ) {
        self.events.push(SpanEvent {
            name: name.into(),
            timestamp: Utc::now(),
            attributes,
        });
    }

    pub fn set_ok(&mut self) {
        self.status = SpanStatus::Ok;
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.status = SpanStatus::Error;
        self.set_attribute("error.message", message);
    }

    pub fn end(&mut self) {
        self.end_time = Some(Utc::now());
    }

    pub fn duration_ms(&self) -> Option<i64> {
        self.end_time.map(|end| {
            (end - self.start_time).num_milliseconds()
        })
    }

    pub fn is_ended(&self) -> bool {
        self.end_time.is_some()
    }
}

/// An event within a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub attributes: HashMap<String, String>,
}

/// A complete trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub trace_id: TraceId,
    pub root_span_id: Option<SpanId>,
    pub spans: Vec<Span>,
    pub created_at: DateTime<Utc>,
}

impl Trace {
    pub fn new() -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            root_span_id: None,
            spans: Vec::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_id(trace_id: TraceId) -> Self {
        Self {
            trace_id,
            root_span_id: None,
            spans: Vec::new(),
            created_at: Utc::now(),
        }
    }

    pub fn add_span(&mut self, span: Span) {
        if self.root_span_id.is_none() && span.parent_id.is_none() {
            self.root_span_id = Some(span.span_id.clone());
        }
        self.spans.push(span);
    }

    pub fn total_duration_ms(&self) -> Option<i64> {
        if let Some(root_id) = &self.root_span_id {
            self.spans
                .iter()
                .find(|s| &s.span_id == root_id)
                .and_then(|s| s.duration_ms())
        } else {
            None
        }
    }

    pub fn span_count(&self) -> usize {
        self.spans.len()
    }

    pub fn has_errors(&self) -> bool {
        self.spans.iter().any(|s| s.status == SpanStatus::Error)
    }
}

impl Default for Trace {
    fn default() -> Self {
        Self::new()
    }
}

/// Trace collector for storing and querying traces
pub struct TraceCollector {
    traces: RwLock<HashMap<TraceId, Trace>>,
    max_traces: usize,
}

impl TraceCollector {
    pub fn new() -> Self {
        Self {
            traces: RwLock::new(HashMap::new()),
            max_traces: 1000,
        }
    }

    pub fn with_max_traces(max: usize) -> Self {
        Self {
            traces: RwLock::new(HashMap::new()),
            max_traces: max,
        }
    }

    /// Start a new trace
    pub fn start_trace(&self) -> Trace {
        let trace = Trace::new();
        self.traces
            .write()
            .unwrap()
            .insert(trace.trace_id.clone(), trace.clone());
        trace
    }

    /// Record a span
    pub fn record_span(&self, span: Span) {
        let mut traces = self.traces.write().unwrap();
        if let Some(trace) = traces.get_mut(&span.trace_id) {
            trace.add_span(span);
        } else {
            // Create new trace if it doesn't exist
            let mut trace = Trace::with_id(span.trace_id.clone());
            trace.add_span(span);
            traces.insert(trace.trace_id.clone(), trace);
        }

        // Clean up old traces if at limit
        if traces.len() > self.max_traces {
            self.cleanup_old(&mut traces);
        }
    }

    /// Get a trace by ID
    pub fn get_trace(&self, trace_id: &str) -> Option<Trace> {
        self.traces.read().unwrap().get(trace_id).cloned()
    }

    /// Get recent traces
    pub fn recent_traces(&self, limit: usize) -> Vec<Trace> {
        let traces = self.traces.read().unwrap();
        let mut all: Vec<_> = traces.values().cloned().collect();
        all.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        all.truncate(limit);
        all
    }

    /// Get traces with errors
    pub fn error_traces(&self, limit: usize) -> Vec<Trace> {
        let traces = self.traces.read().unwrap();
        let mut errors: Vec<_> = traces
            .values()
            .filter(|t| t.has_errors())
            .cloned()
            .collect();
        errors.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        errors.truncate(limit);
        errors
    }

    /// Get trace count
    pub fn trace_count(&self) -> usize {
        self.traces.read().unwrap().len()
    }

    fn cleanup_old(&self, traces: &mut HashMap<TraceId, Trace>) {
        // Remove oldest traces
        let mut by_time: Vec<_> = traces.iter().map(|(k, v)| (k.clone(), v.created_at)).collect();
        by_time.sort_by(|a, b| a.1.cmp(&b.1));

        let to_remove = by_time.len().saturating_sub(self.max_traces / 2);
        for (key, _) in by_time.into_iter().take(to_remove) {
            traces.remove(&key);
        }
    }
}

impl Default for TraceCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Span builder for fluent API
pub struct SpanBuilder {
    span: Span,
}

impl SpanBuilder {
    pub fn new(trace_id: TraceId, name: impl Into<String>) -> Self {
        Self {
            span: Span::new(trace_id, name),
        }
    }

    pub fn parent(mut self, parent_id: SpanId) -> Self {
        self.span.parent_id = Some(parent_id);
        self
    }

    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.span.set_attribute(key, value);
        self
    }

    pub fn build(self) -> Span {
        self.span
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_creation() {
        let span = Span::new("trace-1".to_string(), "test-span");
        assert_eq!(span.name, "test-span");
        assert_eq!(span.status, SpanStatus::Unset);
        assert!(!span.is_ended());
    }

    #[test]
    fn test_span_lifecycle() {
        let mut span = Span::new("trace-1".to_string(), "test-span");
        span.set_attribute("key", "value");
        span.add_event("started");
        span.set_ok();
        span.end();

        assert!(span.is_ended());
        assert_eq!(span.status, SpanStatus::Ok);
        assert!(span.duration_ms().is_some());
    }

    #[test]
    fn test_span_error() {
        let mut span = Span::new("trace-1".to_string(), "test-span");
        span.set_error("Something went wrong");

        assert_eq!(span.status, SpanStatus::Error);
        assert!(span.attributes.contains_key("error.message"));
    }

    #[test]
    fn test_trace() {
        let mut trace = Trace::new();

        let mut root = Span::new(trace.trace_id.clone(), "root");
        let root_id = root.span_id.clone();
        root.end();
        trace.add_span(root);

        let mut child = Span::new(trace.trace_id.clone(), "child")
            .with_parent(root_id);
        child.end();
        trace.add_span(child);

        assert_eq!(trace.span_count(), 2);
        assert!(!trace.has_errors());
    }

    #[test]
    fn test_trace_collector() {
        let collector = TraceCollector::new();

        let trace = collector.start_trace();
        let trace_id = trace.trace_id.clone();

        let mut span = Span::new(trace_id.clone(), "test");
        span.set_ok();
        span.end();
        collector.record_span(span);

        let retrieved = collector.get_trace(&trace_id).unwrap();
        assert_eq!(retrieved.span_count(), 1);
    }

    #[test]
    fn test_error_traces() {
        let collector = TraceCollector::new();

        // Good trace
        let trace1 = collector.start_trace();
        let mut span1 = Span::new(trace1.trace_id.clone(), "good");
        span1.set_ok();
        span1.end();
        collector.record_span(span1);

        // Error trace
        let trace2 = collector.start_trace();
        let mut span2 = Span::new(trace2.trace_id.clone(), "bad");
        span2.set_error("Failed");
        span2.end();
        collector.record_span(span2);

        let errors = collector.error_traces(10);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_span_builder() {
        let span = SpanBuilder::new("trace-1".to_string(), "test")
            .parent("parent-1".to_string())
            .attribute("key", "value")
            .build();

        assert_eq!(span.parent_id, Some("parent-1".to_string()));
        assert_eq!(span.attributes.get("key"), Some(&"value".to_string()));
    }
}
