//! OTLP Exporter
//!
//! Bridges the custom `Span`/`Trace` types to OpenTelemetry's export pipeline,
//! enabling export to any OTLP-compatible collector (Jaeger, Grafana Tempo, etc.).

use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry::KeyValue;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;

use crate::tracing as custom_tracing;

/// Errors from OTLP export operations.
#[derive(Debug, thiserror::Error)]
pub enum OtlpError {
    #[error("Failed to initialize OTLP exporter: {0}")]
    InitError(String),

    #[error("Export failed: {0}")]
    ExportError(String),
}

/// Configuration for the OTLP exporter.
#[derive(Debug, Clone)]
pub struct OtlpConfig {
    /// OTLP/gRPC endpoint (e.g., "http://localhost:4317")
    pub endpoint: String,
    /// Service name reported in traces
    pub service_name: String,
    /// Service version
    pub service_version: String,
}

impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4317".to_string(),
            service_name: "agent-agency-v4".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Initialize the global OTLP tracer provider.
///
/// After calling this, use `export_trace()` to send custom traces
/// through the OpenTelemetry pipeline.
pub fn init_otlp_provider(config: &OtlpConfig) -> Result<SdkTracerProvider, OtlpError> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&config.endpoint)
        .build()
        .map_err(|e| OtlpError::InitError(e.to_string()))?;

    let resource = Resource::builder()
        .with_service_name(config.service_name.clone())
        .with_attributes([KeyValue::new(
            "service.version",
            config.service_version.clone(),
        )])
        .build();

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    // Install as the global provider so tracers can be obtained anywhere
    let _ = opentelemetry::global::set_tracer_provider(provider.clone());

    Ok(provider)
}

/// Export a custom trace through the global OpenTelemetry pipeline.
///
/// Requires `init_otlp_provider()` to have been called first.
pub fn export_trace(trace: &custom_tracing::Trace) {
    let tracer = opentelemetry::global::tracer("agent-agency-v4");

    for span in &trace.spans {
        export_span(&tracer, span);
    }
}

/// Convert and export a single custom span through the OTel tracer.
fn export_span(tracer: &opentelemetry::global::BoxedTracer, span: &custom_tracing::Span) {
    // Clone data needed inside the 'static closure
    let name = span.name.clone();
    let attrs: Vec<KeyValue> = span
        .attributes
        .iter()
        .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
        .collect();
    let span_id = span.span_id.clone();
    let trace_id = span.trace_id.clone();
    let parent_id = span.parent_id.clone();
    let status = span.status;
    let error_message = span
        .attributes
        .get("error.message")
        .cloned()
        .unwrap_or_default();
    let events: Vec<_> = span
        .events
        .iter()
        .map(|e| {
            let event_attrs: Vec<KeyValue> = e
                .attributes
                .iter()
                .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
                .collect();
            (e.name.clone(), e.timestamp, event_attrs)
        })
        .collect();

    tracer.in_span(name, |cx| {
        let otel_span = cx.span();

        otel_span.set_attributes(attrs);
        otel_span.set_attribute(KeyValue::new("custom.span_id", span_id));
        otel_span.set_attribute(KeyValue::new("custom.trace_id", trace_id));

        if let Some(parent_id) = parent_id {
            otel_span.set_attribute(KeyValue::new("custom.parent_id", parent_id));
        }

        match status {
            custom_tracing::SpanStatus::Ok => {
                otel_span.set_status(opentelemetry::trace::Status::Ok);
            }
            custom_tracing::SpanStatus::Error => {
                otel_span.set_status(opentelemetry::trace::Status::error(error_message));
            }
            custom_tracing::SpanStatus::Unset => {}
        }

        for (event_name, timestamp, event_attrs) in events {
            otel_span.add_event_with_timestamp(event_name, timestamp.into(), event_attrs);
        }
    });
}

/// Shutdown the global tracer provider, flushing pending spans.
pub fn shutdown_otlp(provider: SdkTracerProvider) -> Result<(), OtlpError> {
    provider
        .shutdown()
        .map_err(|e| OtlpError::ExportError(e.to_string()))
}

/// Convert a custom `SpanStatus` to a descriptive string.
pub fn status_name(status: custom_tracing::SpanStatus) -> &'static str {
    match status {
        custom_tracing::SpanStatus::Ok => "ok",
        custom_tracing::SpanStatus::Error => "error",
        custom_tracing::SpanStatus::Unset => "unset",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tracing::{Span, SpanStatus, Trace};

    #[test]
    fn test_otlp_config_defaults() {
        let config = OtlpConfig::default();
        assert_eq!(config.endpoint, "http://localhost:4317");
        assert_eq!(config.service_name, "agent-agency-v4");
    }

    #[test]
    fn test_status_name_conversion() {
        assert_eq!(status_name(SpanStatus::Ok), "ok");
        assert_eq!(status_name(SpanStatus::Error), "error");
        assert_eq!(status_name(SpanStatus::Unset), "unset");
    }

    #[test]
    fn test_span_to_otel_attributes() {
        let mut span = Span::new("trace-1".to_string(), "tool_execution");
        span.set_attribute("tool_id", "builtin:file-read");
        span.set_attribute("operator_class", "S");
        span.set_attribute("task_id", "task-123");
        span.set_ok();
        span.end();

        let attrs: Vec<KeyValue> = span
            .attributes
            .iter()
            .map(|(k, v)| KeyValue::new(k.clone(), v.clone()))
            .collect();

        assert_eq!(attrs.len(), 3);
        assert!(attrs.iter().any(|kv| kv.key.as_str() == "tool_id"));
        assert!(attrs.iter().any(|kv| kv.key.as_str() == "task_id"));
    }

    #[test]
    fn test_trace_with_spans_for_export() {
        let mut trace = Trace::new();

        let mut root = Span::new(trace.trace_id.clone(), "agent_loop_iteration");
        root.set_attribute("iteration", "1");
        root.set_attribute("task_id", "task-42");
        let root_id = root.span_id.clone();
        root.set_ok();
        root.end();
        trace.add_span(root);

        let mut child = Span::new(trace.trace_id.clone(), "tool_execution")
            .with_parent(root_id);
        child.set_attribute("tool_id", "builtin:file-read");
        child.set_attribute("duration_ms", "15");
        child.set_attribute("success", "true");
        child.set_ok();
        child.end();
        trace.add_span(child);

        assert_eq!(trace.span_count(), 2);
        assert!(!trace.has_errors());
        assert!(trace.total_duration_ms().is_some());
    }

    #[test]
    fn test_export_trace_without_collector() {
        // export_trace should work even without a collector — spans just won't go anywhere
        let mut trace = Trace::new();
        let mut span = Span::new(trace.trace_id.clone(), "test_span");
        span.set_ok();
        span.end();
        trace.add_span(span);

        // This should not panic even without init_otlp_provider
        export_trace(&trace);
    }
}
