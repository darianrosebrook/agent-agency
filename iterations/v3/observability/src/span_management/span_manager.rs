//! Span Management Module
//!
//! Handles the lifecycle of individual spans within traces,
//! including creation, completion, attribute management, and event tracking.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use opentelemetry::trace::SpanBuilder;

use crate::trace_types::*;

/// Manager for individual span lifecycle operations
#[derive(Debug)]
pub struct SpanManager {
    /// Configuration for span management
    config: TraceConfig,
    /// Active spans storage
    active_spans: Arc<RwLock<HashMap<String, SpanInfo>>>,
    /// OpenTelemetry tracer if available
    tracer: Option<opentelemetry::trace::TracerProvider>,
}

impl SpanManager {
    /// Create a new span manager
    pub fn new(
        config: TraceConfig,
        active_spans: Arc<RwLock<HashMap<String, SpanInfo>>>,
        tracer: Option<opentelemetry::trace::TracerProvider>,
    ) -> Self {
        Self {
            config,
            active_spans,
            tracer,
        }
    }

    /// Start a new trace span
    pub async fn start_span(
        &self,
        operation: &str,
        parent_trace_id: Option<&str>,
        attributes: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
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
            let mut otel_attributes = Vec::new();

            // Set trace and span IDs
            otel_attributes.push(opentelemetry::KeyValue::new("trace.id", trace_id.clone()));
            otel_attributes.push(opentelemetry::KeyValue::new("span.id", span_id.clone()));

            if let Some(parent_id) = parent_trace_id {
                otel_attributes.push(opentelemetry::KeyValue::new("parent.span.id", parent_id.to_string()));
            }

            let _otel_span = tracer.build_with_context(span_builder, &opentelemetry::Context::new());
            // Store span in context for later use
        }

        Ok(span_id)
    }

    /// End a trace span
    pub async fn end_span(&self, span_id: &str, status: SpanStatus) -> Result<()> {
        let mut active_spans = self.active_spans.write().await;

        if let Some(span_info) = active_spans.get_mut(span_id) {
            span_info.end_time = Some(chrono::Utc::now());
            span_info.status = status.clone();

            if let Some(end) = span_info.end_time {
                let start = span_info.start_time;
                span_info.duration_ms = Some((end - start).num_milliseconds() as u64);
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
    ) -> Result<()> {
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
    ) -> Result<()> {
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

    /// Get information about a span
    pub async fn get_span_info(&self, span_id: &str) -> Result<Option<SpanInfo>> {
        let active_spans = self.active_spans.read().await;
        Ok(active_spans.get(span_id).cloned())
    }

    /// Check if a span is currently active
    pub async fn is_span_active(&self, span_id: &str) -> bool {
        let active_spans = self.active_spans.read().await;
        active_spans.contains_key(span_id)
    }

    /// Get all active span IDs
    pub async fn get_active_span_ids(&self) -> Vec<String> {
        let active_spans = self.active_spans.read().await;
        active_spans.keys().cloned().collect()
    }

    /// Get span error information
    pub fn get_span_error_info(span_info: &SpanInfo) -> SpanErrorInfo {
        let is_error = matches!(span_info.status, SpanStatus::Error);

        let error_message = if is_error {
            span_info.attributes.get("error.message")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        };

        let error_type = if is_error {
            span_info.attributes.get("error.type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        };

        let stack_trace = if is_error {
            span_info.attributes.get("error.stack")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        };

        let http_status = span_info.attributes.get("http.status_code")
            .and_then(|v| v.as_u64())
            .map(|n| n as u16);

        SpanErrorInfo {
            is_error,
            error_message,
            error_type,
            stack_trace,
            http_status,
        }
    }

    /// Validate span attributes against configuration
    pub fn validate_span_attributes(&self, attributes: &HashMap<String, serde_json::Value>) -> Result<()> {
        if attributes.len() > self.config.max_attributes {
            return Err(anyhow::anyhow!(
                "Too many span attributes: {} (max: {})",
                attributes.len(),
                self.config.max_attributes
            ));
        }

        // Check for reserved attribute names
        let reserved_names = ["trace.id", "span.id", "parent.span.id"];
        for key in attributes.keys() {
            if reserved_names.contains(&key.as_str()) {
                return Err(anyhow::anyhow!("Reserved attribute name: {}", key));
            }
        }

        Ok(())
    }
}

// Configuration constants
impl TraceConfig {
    /// Maximum number of attributes per span
    pub const MAX_ATTRIBUTES: usize = 100;
    /// Maximum number of events per span
    pub const MAX_EVENTS: usize = 50;
}

impl TraceConfig {
    /// Get maximum attributes per span
    pub fn max_attributes(&self) -> usize {
        Self::MAX_ATTRIBUTES
    }

    /// Get maximum events per span
    pub fn max_events(&self) -> usize {
        Self::MAX_EVENTS
    }
}
