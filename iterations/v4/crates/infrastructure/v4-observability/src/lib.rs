//! V4 Observability
//!
//! Metrics, tracing, and health checks for the Agent Agency system.
//!
//! # Architecture
//!
//! This crate provides three pillars of observability:
//!
//! 1. **Metrics** - Counters, gauges, and histograms for quantitative monitoring
//! 2. **Tracing** - Distributed tracing for request flow tracking
//! 3. **Health** - Liveness and readiness checks for system health
//!
//! # Usage
//!
//! ```rust
//! use v4_observability::{
//!     metrics::MetricsRegistry,
//!     tracing::{TraceCollector, Span},
//!     health::{HealthMonitor, HealthCheck},
//! };
//!
//! // Create a metrics registry
//! let registry = MetricsRegistry::new();
//! let request_counter = registry.counter("requests_total", "Total HTTP requests");
//! request_counter.inc();
//!
//! // Create a trace collector
//! let collector = TraceCollector::new();
//! let trace = collector.start_trace();
//! let mut span = Span::new(trace.trace_id.clone(), "handle_request");
//! span.set_attribute("method", "POST");
//! span.set_ok();
//! span.end();
//! collector.record_span(span);
//!
//! // Create a health monitor
//! let monitor = HealthMonitor::new("1.0.0");
//! monitor.register_simple("database", || true);
//! let report = monitor.check();
//! assert!(report.is_healthy());
//! ```

pub mod health;
pub mod metrics;
pub mod otlp;
pub mod tracing;

// Re-exports for convenience
pub use health::{
    liveness_check, HealthCheck, HealthChecker, HealthMonitor, HealthReport, HealthStatus,
    ReadinessCheck, SimpleHealthCheck,
};
pub use metrics::{Counter, Gauge, Histogram, HistogramValue, Metric, MetricType, MetricValue, MetricsRegistry};
pub use tracing::{Span, SpanBuilder, SpanEvent, SpanId, SpanStatus, Trace, TraceCollector, TraceId};
pub use otlp::{OtlpConfig, OtlpError};

/// Initialize observability with default settings
pub fn init() -> ObservabilityContext {
    ObservabilityContext::new("unknown")
}

/// Initialize observability with a version string
pub fn init_with_version(version: impl Into<String>) -> ObservabilityContext {
    ObservabilityContext::new(version)
}

/// Unified observability context
pub struct ObservabilityContext {
    pub metrics: MetricsRegistry,
    pub traces: TraceCollector,
    pub health: HealthMonitor,
}

impl ObservabilityContext {
    pub fn new(version: impl Into<String>) -> Self {
        let version = version.into();
        Self {
            metrics: MetricsRegistry::new(),
            traces: TraceCollector::new(),
            health: HealthMonitor::new(&version),
        }
    }

    /// Register standard system metrics
    pub fn register_standard_metrics(&self) {
        // Request metrics
        self.metrics.counter("requests_total", "Total requests processed");
        self.metrics.counter("requests_failed", "Total failed requests");
        self.metrics.histogram("request_duration_seconds", "Request duration in seconds");

        // Task metrics
        self.metrics.counter("tasks_created", "Total tasks created");
        self.metrics.counter("tasks_completed", "Total tasks completed");
        self.metrics.counter("tasks_failed", "Total tasks failed");
        self.metrics.gauge("tasks_active", "Currently active tasks");

        // Council metrics
        self.metrics.counter("council_reviews", "Total council reviews");
        self.metrics.counter("council_approvals", "Total council approvals");
        self.metrics.counter("council_rejections", "Total council rejections");
        self.metrics.histogram("council_review_duration_seconds", "Council review duration");

        // Gate metrics
        self.metrics.counter("gates_evaluated", "Total gate evaluations");
        self.metrics.counter("gates_passed", "Total gates passed");
        self.metrics.counter("gates_failed", "Total gates failed");
    }

    /// Create a new traced span
    pub fn start_span(&self, name: impl Into<String>) -> TracedSpan<'_> {
        let trace = self.traces.start_trace();
        let span = Span::new(trace.trace_id.clone(), name);
        TracedSpan {
            span,
            collector: &self.traces,
        }
    }

    /// Check system health
    pub fn check_health(&self) -> HealthReport {
        self.health.check()
    }
}

/// A span that automatically records itself when dropped
pub struct TracedSpan<'a> {
    span: Span,
    collector: &'a TraceCollector,
}

impl<'a> TracedSpan<'a> {
    pub fn set_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.span.set_attribute(key, value);
    }

    pub fn add_event(&mut self, name: impl Into<String>) {
        self.span.add_event(name);
    }

    pub fn set_ok(&mut self) {
        self.span.set_ok();
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.span.set_error(message);
    }

    pub fn span_id(&self) -> &SpanId {
        &self.span.span_id
    }

    pub fn trace_id(&self) -> &TraceId {
        &self.span.trace_id
    }
}

impl<'a> Drop for TracedSpan<'a> {
    fn drop(&mut self) {
        self.span.end();
        self.collector.record_span(self.span.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let ctx = init();
        assert_eq!(ctx.metrics.metric_count(), 0);
        assert_eq!(ctx.traces.trace_count(), 0);
    }

    #[test]
    fn test_init_with_version() {
        let ctx = init_with_version("1.0.0");
        let report = ctx.check_health();
        assert_eq!(report.version, "1.0.0");
    }

    #[test]
    fn test_standard_metrics() {
        let ctx = init_with_version("1.0.0");
        ctx.register_standard_metrics();

        // Should have registered multiple metrics
        assert!(ctx.metrics.metric_count() > 0);

        // Verify we can access them
        let counter = ctx.metrics.counter("requests_total", "");
        counter.inc();
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_traced_span() {
        let ctx = init_with_version("1.0.0");

        {
            let mut span = ctx.start_span("test_operation");
            span.set_attribute("key", "value");
            span.add_event("started");
            span.set_ok();
            // span automatically recorded on drop
        }

        // Trace should be recorded
        let traces = ctx.traces.recent_traces(10);
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].span_count(), 1);
    }

    #[test]
    fn test_traced_span_error() {
        let ctx = init_with_version("1.0.0");

        {
            let mut span = ctx.start_span("failing_operation");
            span.set_error("Something went wrong");
        }

        let errors = ctx.traces.error_traces(10);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_health_integration() {
        let ctx = init_with_version("1.0.0");

        ctx.health.register_simple("service_a", || true);
        ctx.health.register_simple("service_b", || true);

        let report = ctx.check_health();
        assert!(report.is_healthy());
        assert_eq!(report.checks.len(), 2);
    }

    #[test]
    fn test_full_observability_workflow() {
        let ctx = init_with_version("2.0.0");
        ctx.register_standard_metrics();

        // Register health checks
        ctx.health.register_simple("database", || true);
        ctx.health.register_simple("cache", || true);

        // Simulate a request
        let request_counter = ctx.metrics.counter("requests_total", "");
        let duration_histogram = ctx.metrics.histogram("request_duration_seconds", "");

        {
            let mut span = ctx.start_span("handle_request");
            span.set_attribute("method", "POST");
            span.set_attribute("path", "/api/tasks");

            // Simulate work
            request_counter.inc();
            duration_histogram.observe(0.125);

            span.add_event("validated_input");
            span.add_event("processed_request");
            span.set_ok();
        }

        // Verify metrics
        assert_eq!(request_counter.get(), 1);
        let hist = duration_histogram.get();
        assert_eq!(hist.count, 1);

        // Verify traces
        let traces = ctx.traces.recent_traces(10);
        assert_eq!(traces.len(), 1);
        assert!(!traces[0].has_errors());

        // Verify health
        let health = ctx.check_health();
        assert!(health.is_healthy());
    }
}
