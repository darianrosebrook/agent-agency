//! OpenTelemetry Integration Module
//!
//! Handles integration with OpenTelemetry for distributed tracing,
//! including OTLP export configuration and tracer management.

use anyhow::Result;
use opentelemetry::{global, trace::SpanBuilder};
use opentelemetry_sdk::trace;
use opentelemetry_otlp::WithExportConfig;

use crate::trace_types::*;

/// OpenTelemetry integrator for distributed tracing
#[derive(Debug)]
pub struct OtelIntegrator {
    /// Configuration for OTLP export
    config: TraceConfig,
    /// OpenTelemetry tracer provider
    tracer_provider: Option<opentelemetry::trace::TracerProvider>,
}

impl OtelIntegrator {
    /// Create a new OpenTelemetry integrator
    pub fn new(config: TraceConfig) -> Result<Self> {
        let tracer_provider = if config.enable_otlp {
            Self::setup_otlp_tracer(&config)?
        } else {
            None
        };

        Ok(Self {
            config,
            tracer_provider,
        })
    }

    /// Get the OpenTelemetry tracer if available
    pub fn get_tracer(&self) -> Option<opentelemetry::trace::TracerProvider> {
        self.tracer_provider.clone()
    }

    /// Create an OpenTelemetry span
    pub fn create_otel_span(
        &self,
        operation: &str,
        trace_id: &str,
        span_id: &str,
        parent_span_id: Option<&str>,
    ) -> Option<opentelemetry::trace::Span> {
        if let Some(tracer) = &self.tracer_provider {
            let mut span_builder = SpanBuilder::from_name(operation.to_string());
            let mut attributes = Vec::new();

            // Set trace and span IDs
            attributes.push(opentelemetry::KeyValue::new("trace.id", trace_id.to_string()));
            attributes.push(opentelemetry::KeyValue::new("span.id", span_id.to_string()));

            if let Some(parent_id) = parent_span_id {
                attributes.push(opentelemetry::KeyValue::new("parent.span.id", parent_id.to_string()));
            }

            Some(tracer.build_with_context(span_builder, &opentelemetry::Context::new()))
        } else {
            None
        }
    }

    /// Add attributes to an OpenTelemetry span
    pub fn add_otel_span_attributes(
        &self,
        span: &mut opentelemetry::trace::Span,
        attributes: &std::collections::HashMap<String, serde_json::Value>,
    ) {
        for (key, value) in attributes {
            match value {
                serde_json::Value::String(s) => {
                    span.set_attribute(opentelemetry::KeyValue::new(key.clone(), s.clone()));
                },
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        span.set_attribute(opentelemetry::KeyValue::new(key.clone(), i));
                    } else if let Some(f) = n.as_f64() {
                        span.set_attribute(opentelemetry::KeyValue::new(key.clone(), f));
                    }
                },
                serde_json::Value::Bool(b) => {
                    span.set_attribute(opentelemetry::KeyValue::new(key.clone(), *b));
                },
                _ => {
                    // Convert other types to string
                    span.set_attribute(opentelemetry::KeyValue::new(key.clone(), value.to_string()));
                }
            }
        }
    }

    /// Record an event on an OpenTelemetry span
    pub fn record_otel_event(
        &self,
        span: &mut opentelemetry::trace::Span,
        name: &str,
        attributes: &std::collections::HashMap<String, serde_json::Value>,
    ) {
        let mut otel_attributes = Vec::new();

        for (key, value) in attributes {
            match value {
                serde_json::Value::String(s) => {
                    otel_attributes.push(opentelemetry::KeyValue::new(key.clone(), s.clone()));
                },
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        otel_attributes.push(opentelemetry::KeyValue::new(key.clone(), i));
                    } else if let Some(f) = n.as_f64() {
                        otel_attributes.push(opentelemetry::KeyValue::new(key.clone(), f));
                    }
                },
                serde_json::Value::Bool(b) => {
                    otel_attributes.push(opentelemetry::KeyValue::new(key.clone(), *b));
                },
                _ => {
                    otel_attributes.push(opentelemetry::KeyValue::new(key.clone(), value.to_string()));
                }
            }
        }

        span.add_event(name.to_string(), otel_attributes);
    }

    /// End an OpenTelemetry span
    pub fn end_otel_span(&self, span: opentelemetry::trace::Span) {
        drop(span); // This will end the span
    }

    /// Check if OpenTelemetry is enabled and configured
    pub fn is_enabled(&self) -> bool {
        self.config.enable_otlp && self.tracer_provider.is_some()
    }

    /// Get OTLP endpoint if configured
    pub fn get_otlp_endpoint(&self) -> Option<&str> {
        self.config.otlp_endpoint.as_deref()
    }

    /// Setup OTLP tracer with the given configuration
    fn setup_otlp_tracer(config: &TraceConfig) -> Result<Option<opentelemetry::trace::TracerProvider>> {
        if !config.enable_otlp {
            return Ok(None);
        }

        if let Some(endpoint) = &config.otlp_endpoint {
            // Setup OTLP exporter
            let exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint);

            // Create tracer provider with batch processor
            let tracer_provider = opentelemetry_otlp::new_pipeline()
                .tracing()
                .with_exporter(exporter)
                .with_trace_config(
                    trace::config().with_sampler(
                        trace::Sampler::TraceIdRatioBased(config.sample_rate),
                    )
                )
                .install_batch(opentelemetry_sdk::runtime::Tokio)?;

            // Set as global tracer provider
            global::set_tracer_provider(tracer_provider.clone());

            Ok(Some(tracer_provider))
        } else {
            Ok(None)
        }
    }

    /// Shutdown OpenTelemetry integration
    pub fn shutdown(&self) -> Result<()> {
        if let Some(provider) = &self.tracer_provider {
            // Force flush any pending spans
            opentelemetry::global::shutdown_tracer_provider();

            // Shutdown the provider
            // Note: In a real implementation, you might need to call shutdown methods
            // on the specific provider type
        }

        Ok(())
    }

    /// Get tracer statistics
    pub fn get_tracer_stats(&self) -> std::collections::HashMap<String, serde_json::Value> {
        let mut stats = std::collections::HashMap::new();

        stats.insert("otel_enabled".to_string(), serde_json::json!(self.is_enabled()));
        stats.insert("sample_rate".to_string(), serde_json::json!(self.config.sample_rate));
        stats.insert("otlp_endpoint".to_string(), serde_json::json!(self.config.otlp_endpoint));

        stats
    }
}

/// Extension trait for OpenTelemetry spans
pub trait OtelSpanExt {
    /// Set status on the span
    fn set_status(&mut self, status: SpanStatus);
}

impl OtelSpanExt for opentelemetry::trace::Span {
    fn set_status(&mut self, status: SpanStatus) {
        match status {
            SpanStatus::Ok => {
                self.set_status(opentelemetry::trace::Status::Ok);
            },
            SpanStatus::Error => {
                self.set_status(opentelemetry::trace::Status::error("Span failed"));
            },
            SpanStatus::Unset => {
                // Leave status unset
            }
        }
    }
}
