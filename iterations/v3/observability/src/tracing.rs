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
pub struct TraceHierarchy {
    pub trace_id: String,
    pub root_span_id: String,
    pub spans: HashMap<String, SpanHierarchyInfo>,
    pub max_depth: u32,
    pub total_spans: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanHierarchyInfo {
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub children: Vec<String>,
    pub depth: u32,
    pub service_name: String,
    pub operation: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAnalytics {
    pub average_span_duration_ms: f64,
    pub max_span_depth: u32,
    pub total_spans: u32,
    pub service_breakdown: HashMap<String, u32>,
    pub error_rate: f64,
    pub slowest_operations: Vec<(String, u64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub service_name: String,
    pub component: String,
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub last_checked: chrono::DateTime<chrono::Utc>,
    pub error_message: Option<String>,
    pub metrics: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerState {
    pub service_name: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub last_failure_time: Option<chrono::DateTime<chrono::Utc>>,
    pub next_retry_time: Option<chrono::DateTime<chrono::Utc>>,
    pub success_count: u32,
    pub total_requests: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Circuit is open, failing fast
    HalfOpen,    // Testing if service recovered
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub overall_status: HealthStatus,
    pub service_health: HashMap<String, HealthCheckResult>,
    pub circuit_breakers: HashMap<String, CircuitBreakerState>,
    pub system_metrics: SystemMetrics,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub memory_usage_percent: f32,
    pub cpu_usage_percent: f32,
    pub active_connections: u32,
    pub queue_depth: u32,
    pub error_rate_percent: f32,
    pub average_response_time_ms: f64,
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
    trace_hierarchies: Arc<RwLock<HashMap<String, TraceHierarchy>>>,
    span_relationships: Arc<RwLock<HashMap<String, SpanHierarchyInfo>>>,
    health_checks: Arc<RwLock<HashMap<String, HealthCheckResult>>>,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreakerState>>>,
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
            trace_hierarchies: Arc::new(RwLock::new(HashMap::new())),
            span_relationships: Arc::new(RwLock::new(HashMap::new())),
            health_checks: Arc::new(RwLock::new(HashMap::new())),
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
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

        // Track span hierarchy relationships
        self.track_span_start(&span_id, parent_trace_id, &self.config.service_name, operation).await;

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

            // Track span completion in hierarchy
            self.track_span_end(span_id).await;

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

    /// Track span relationship in hierarchy when span starts
    async fn track_span_start(&self, span_id: &str, parent_span_id: Option<&str>, service_name: &str, operation: &str) {
        let mut span_relationships = self.span_relationships.write().await;

        let hierarchy_info = SpanHierarchyInfo {
            span_id: span_id.to_string(),
            parent_span_id: parent_span_id.map(|s| s.to_string()),
            children: Vec::new(),
            depth: 0, // Will be calculated
            service_name: service_name.to_string(),
            operation: operation.to_string(),
            start_time: chrono::Utc::now(),
            end_time: None,
            duration_ms: None,
        };

        // Calculate depth by walking up parent chain
        let mut depth = 0;
        let mut current_parent = parent_span_id;
        let mut visited = std::collections::HashSet::new();

        while let Some(parent_id) = current_parent {
            if visited.contains(parent_id) || depth > 50 {
                break; // Prevent cycles and excessive depth
            }
            visited.insert(parent_id);

            if let Some(parent_info) = span_relationships.get(parent_id) {
                depth = parent_info.depth + 1;
                current_parent = parent_info.parent_span_id.as_deref();
            } else {
                break;
            }
        }

        // Update depth
        let mut hierarchy_info = hierarchy_info;
        hierarchy_info.depth = depth;

        // Add to parent as child if parent exists
        if let Some(parent_id) = parent_span_id {
            if let Some(parent_info) = span_relationships.get_mut(parent_id) {
                if !parent_info.children.contains(&span_id.to_string()) {
                    parent_info.children.push(span_id.to_string());
                }
            }
        }

        span_relationships.insert(span_id.to_string(), hierarchy_info);
    }

    /// Track span completion and update hierarchy
    async fn track_span_end(&self, span_id: &str) {
        let mut span_relationships = self.span_relationships.write().await;

        if let Some(span_info) = span_relationships.get_mut(span_id) {
            let end_time = chrono::Utc::now();
            span_info.end_time = Some(end_time);

            if let Some(start_time) = Some(span_info.start_time) {
                span_info.duration_ms = Some(
                    (end_time - start_time).num_milliseconds() as u64
                );
            }
        }
    }

    /// Build complete trace hierarchy from span relationships
    async fn build_trace_hierarchy(&self, trace_id: &str) -> Option<TraceHierarchy> {
        let span_relationships = self.span_relationships.read().await;

        // Find all spans that belong to this trace
        let trace_spans: HashMap<String, SpanHierarchyInfo> = span_relationships
            .iter()
            .filter_map(|(span_id, info)| {
                // Check if this span belongs to the trace by walking up hierarchy
                if self.span_belongs_to_trace(span_id, trace_id) {
                    Some((span_id.clone(), info.clone()))
                } else {
                    None
                }
            })
            .collect();

        if trace_spans.is_empty() {
            return None;
        }

        // Find root span (one with no parent or parent not in this trace)
        let root_span_id = trace_spans
            .iter()
            .find_map(|(span_id, info)| {
                if info.parent_span_id.is_none() ||
                   !trace_spans.contains_key(info.parent_span_id.as_ref().unwrap()) {
                    Some(span_id.clone())
                } else {
                    None
                }
            })?;

        // Calculate max depth
        let max_depth = trace_spans.values().map(|info| info.depth).max().unwrap_or(0);

        Some(TraceHierarchy {
            trace_id: trace_id.to_string(),
            root_span_id,
            spans: trace_spans,
            max_depth,
            total_spans: trace_spans.len() as u32,
            created_at: chrono::Utc::now(),
        })
    }

    /// Check if a span belongs to a specific trace
    fn span_belongs_to_trace(&self, span_id: &str, trace_id: &str) -> bool {
        // For now, we use a simple heuristic: span belongs to trace if
        // the trace_id is a prefix of the span_id or vice versa
        span_id.starts_with(trace_id) || trace_id.starts_with(span_id)
    }

    /// Generate analytics for a trace hierarchy
    async fn generate_trace_analytics(&self, hierarchy: &TraceHierarchy) -> TraceAnalytics {
        let mut total_duration = 0u64;
        let mut completed_spans = 0;
        let mut service_breakdown = HashMap::new();
        let mut error_count = 0;
        let mut operation_durations = Vec::new();

        for span_info in hierarchy.spans.values() {
            if let Some(duration) = span_info.duration_ms {
                total_duration += duration;
                completed_spans += 1;
                operation_durations.push((span_info.operation.clone(), duration));
            }

            *service_breakdown.entry(span_info.service_name.clone()).or_insert(0) += 1;

            // Check for errors (simplified - would need actual span status)
            if span_info.operation.contains("error") || span_info.operation.contains("fail") {
                error_count += 1;
            }
        }

        let average_duration = if completed_spans > 0 {
            total_duration as f64 / completed_spans as f64
        } else {
            0.0
        };

        let error_rate = if hierarchy.total_spans > 0 {
            error_count as f64 / hierarchy.total_spans as f64
        } else {
            0.0
        };

        // Sort by duration descending and take top 5
        operation_durations.sort_by(|a, b| b.1.cmp(&a.1));
        let slowest_operations = operation_durations.into_iter().take(5).collect();

        TraceAnalytics {
            average_span_duration_ms: average_duration,
            max_span_depth: hierarchy.max_depth,
            total_spans: hierarchy.total_spans,
            service_breakdown,
            error_rate,
            slowest_operations,
        }
    }

    /// Extract trace ID from span hierarchy with comprehensive tracking
    async fn extract_trace_id(&self, span_id: &str) -> Option<String> {
        let span_relationships = self.span_relationships.read().await;

        // Find the span in our relationship tracking
        if let Some(span_info) = span_relationships.get(span_id) {
            // Walk up the hierarchy to find the root span
            let mut current_span_id = span_id;
            let mut visited = std::collections::HashSet::new();

            // Prevent infinite loops in case of cycles
            while visited.len() < 100 && !visited.contains(current_span_id) {
                visited.insert(current_span_id);

                if let Some(current_info) = span_relationships.get(current_span_id) {
                    if current_info.parent_span_id.is_none() {
                        // This is a root span, use its span_id as trace_id
                        return Some(current_span_id.to_string());
                    }
                    current_span_id = current_info.parent_span_id.as_ref().unwrap();
                } else {
                    break;
                }
            }

            // Fallback: if we can't find a root, use the original span_id
            Some(span_id.to_string())
        } else {
            // Span not found in relationships, check active spans
            let active_spans = self.active_spans.read().await;
            if active_spans.contains_key(span_id) {
                Some(span_id.to_string())
            } else {
                None
            }
        }
    }

    /// Get trace hierarchy for a given trace ID
    pub async fn get_trace_hierarchy(&self, trace_id: &str) -> Option<TraceHierarchy> {
        self.build_trace_hierarchy(trace_id).await
    }

    /// Get analytics for a trace hierarchy
    pub async fn get_trace_analytics(&self, trace_id: &str) -> Option<TraceAnalytics> {
        if let Some(hierarchy) = self.build_trace_hierarchy(trace_id).await {
            Some(self.generate_trace_analytics(&hierarchy).await)
        } else {
            None
        }
    }

    /// Get all trace hierarchies (for debugging/admin)
    pub async fn get_all_trace_hierarchies(&self) -> HashMap<String, TraceHierarchy> {
        let trace_hierarchies = self.trace_hierarchies.read().await;
        trace_hierarchies.clone()
    }

    /// Clean up old trace hierarchies (keep last N)
    pub async fn cleanup_old_hierarchies(&self, keep_last: usize) {
        let mut trace_hierarchies = self.trace_hierarchies.write().await;
        let mut hierarchies: Vec<_> = trace_hierarchies.iter().collect();

        // Sort by creation time (newest first)
        hierarchies.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

        // Remove old hierarchies beyond the keep limit
        if hierarchies.len() > keep_last {
            let to_remove: Vec<String> = hierarchies[keep_last..]
                .iter()
                .map(|(trace_id, _)| (*trace_id).clone())
                .collect();

            for trace_id in to_remove {
                trace_hierarchies.remove(&trace_id);
            }
        }
    }

    /// Perform health check on a service
    pub async fn perform_health_check(&self, service_name: &str, component: &str) -> Result<HealthCheckResult> {
        let start_time = std::time::Instant::now();

        // Simulate health check logic (would integrate with actual service health endpoints)
        let (status, error_message, metrics) = self.check_service_health(service_name, component).await;

        let response_time = start_time.elapsed().as_millis() as u64;

        let result = HealthCheckResult {
            service_name: service_name.to_string(),
            component: component.to_string(),
            status,
            response_time_ms: response_time,
            last_checked: chrono::Utc::now(),
            error_message,
            metrics,
        };

        // Store health check result
        let mut health_checks = self.health_checks.write().await;
        health_checks.insert(format!("{}:{}", service_name, component), result.clone());

        Ok(result)
    }

    /// Check circuit breaker state for a service
    pub async fn check_circuit_breaker(&self, service_name: &str) -> CircuitState {
        let circuit_breakers = self.circuit_breakers.read().await;

        if let Some(circuit) = circuit_breakers.get(service_name) {
            // Check if circuit should transition from HalfOpen to Closed on success
            if circuit.state == CircuitState::HalfOpen {
                let now = chrono::Utc::now();
                if let Some(next_retry) = circuit.next_retry_time {
                    if now >= next_retry {
                        // Time to test the service again
                        return CircuitState::HalfOpen;
                    }
                }
            }
            circuit.state.clone()
        } else {
            // No circuit breaker exists, assume closed (healthy)
            CircuitState::Closed
        }
    }

    /// Record service call success/failure for circuit breaker
    pub async fn record_service_call(&self, service_name: &str, success: bool) {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        let now = chrono::Utc::now();

        let circuit = circuit_breakers.entry(service_name.to_string()).or_insert(CircuitBreakerState {
            service_name: service_name.to_string(),
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure_time: None,
            next_retry_time: None,
            success_count: 0,
            total_requests: 0,
        });

        circuit.total_requests += 1;

        if success {
            circuit.success_count += 1;
            circuit.failure_count = 0; // Reset failure count on success

            // Transition from HalfOpen to Closed on success
            if circuit.state == CircuitState::HalfOpen {
                circuit.state = CircuitState::Closed;
                circuit.next_retry_time = None;
                tracing::info!("Circuit breaker for {} transitioned to CLOSED (service recovered)", service_name);
            }
        } else {
            circuit.failure_count += 1;
            circuit.last_failure_time = Some(now);

            // Check if circuit should open
            if circuit.state == CircuitState::Closed && circuit.failure_count >= 5 {
                circuit.state = CircuitState::Open;
                circuit.next_retry_time = Some(now + chrono::Duration::seconds(60)); // 1 minute timeout
                tracing::warn!("Circuit breaker for {} opened due to {} consecutive failures", service_name, circuit.failure_count);
            } else if circuit.state == CircuitState::HalfOpen {
                // Failed during half-open test, go back to open
                circuit.state = CircuitState::Open;
                circuit.next_retry_time = Some(now + chrono::Duration::seconds(60));
                tracing::warn!("Circuit breaker for {} remained OPEN (half-open test failed)", service_name);
            }
        }
    }

    /// Get comprehensive system health snapshot
    pub async fn get_system_health_snapshot(&self) -> SystemHealthSnapshot {
        let health_checks = self.health_checks.read().await;
        let circuit_breakers = self.circuit_breakers.read().await;

        // Calculate overall system status
        let overall_status = self.calculate_overall_health_status(&health_checks, &circuit_breakers).await;

        // Gather system metrics
        let system_metrics = self.collect_system_metrics().await;

        // Generate recommendations
        let recommendations = self.generate_health_recommendations(&health_checks, &circuit_breakers, &system_metrics).await;

        SystemHealthSnapshot {
            timestamp: chrono::Utc::now(),
            overall_status,
            service_health: health_checks.clone(),
            circuit_breakers: circuit_breakers.clone(),
            system_metrics,
            recommendations,
        }
    }

    /// Internal method to check service health
    async fn check_service_health(&self, service_name: &str, component: &str) -> (HealthStatus, Option<String>, HashMap<String, serde_json::Value>) {
        // Simulate health checks - in production this would make actual HTTP calls,
        // check database connections, verify service dependencies, etc.

        let mut metrics = HashMap::new();

        match (service_name, component) {
            ("database", "postgres") => {
                // Simulate database health check
                metrics.insert("connection_pool_size".to_string(), serde_json::json!(10));
                metrics.insert("active_connections".to_string(), serde_json::json!(5));
                metrics.insert("query_latency_ms".to_string(), serde_json::json!(15.5));
                (HealthStatus::Healthy, None, metrics)
            },
            ("cache", "redis") => {
                // Simulate cache health check
                metrics.insert("hit_rate".to_string(), serde_json::json!(0.95));
                metrics.insert("memory_usage_mb".to_string(), serde_json::json!(256));
                (HealthStatus::Healthy, None, metrics)
            },
            ("api", "gateway") => {
                // Simulate API gateway health check
                metrics.insert("requests_per_second".to_string(), serde_json::json!(150.0));
                metrics.insert("error_rate".to_string(), serde_json::json!(0.02));
                (HealthStatus::Degraded, Some("High error rate detected".to_string()), metrics)
            },
            _ => {
                // Unknown service/component
                metrics.insert("status".to_string(), serde_json::json!("unknown"));
                (HealthStatus::Unknown, Some("Service not recognized".to_string()), metrics)
            }
        }
    }

    /// Calculate overall system health status
    async fn calculate_overall_health_status(
        &self,
        health_checks: &HashMap<String, HealthCheckResult>,
        circuit_breakers: &HashMap<String, CircuitBreakerState>,
    ) -> HealthStatus {
        if health_checks.is_empty() && circuit_breakers.is_empty() {
            return HealthStatus::Unknown;
        }

        let mut has_unhealthy = false;
        let mut has_degraded = false;
        let mut has_healthy = false;

        // Check health statuses
        for health_check in health_checks.values() {
            match health_check.status {
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
                HealthStatus::Healthy => has_healthy = true,
                HealthStatus::Unknown => {}
            }
        }

        // Check circuit breaker states
        for circuit in circuit_breakers.values() {
            if circuit.state == CircuitState::Open {
                has_unhealthy = true;
            }
        }

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else if has_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }

    /// Collect system-wide metrics
    async fn collect_system_metrics(&self) -> SystemMetrics {
        // In a real implementation, this would collect actual system metrics
        // For now, simulate realistic values

        SystemMetrics {
            memory_usage_percent: 65.0 + (chrono::Utc::now().timestamp_millis() % 20) as f32,
            cpu_usage_percent: 45.0 + (chrono::Utc::now().timestamp_millis() % 30) as f32,
            active_connections: 150 + (chrono::Utc::now().timestamp_millis() % 50) as u32,
            queue_depth: 25 + (chrono::Utc::now().timestamp_millis() % 25) as u32,
            error_rate_percent: 1.2 + (chrono::Utc::now().timestamp_millis() % 10) as f32,
            average_response_time_ms: 125.0 + (chrono::Utc::now().timestamp_millis() % 50) as f64,
        }
    }

    /// Generate health recommendations based on current state
    async fn generate_health_recommendations(
        &self,
        health_checks: &HashMap<String, HealthCheckResult>,
        circuit_breakers: &HashMap<String, CircuitBreakerState>,
        system_metrics: &SystemMetrics,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check system metrics thresholds
        if system_metrics.memory_usage_percent > 85.0 {
            recommendations.push("High memory usage detected. Consider scaling or memory optimization.".to_string());
        }

        if system_metrics.cpu_usage_percent > 80.0 {
            recommendations.push("High CPU usage detected. Consider scaling or performance optimization.".to_string());
        }

        if system_metrics.error_rate_percent > 5.0 {
            recommendations.push("High error rate detected. Investigate service failures and implement circuit breakers.".to_string());
        }

        if system_metrics.queue_depth > 100 {
            recommendations.push("High queue depth detected. Consider scaling workers or optimizing processing.".to_string());
        }

        // Check circuit breakers
        for circuit in circuit_breakers.values() {
            if circuit.state == CircuitState::Open {
                recommendations.push(format!("Circuit breaker for {} is OPEN. Service may be down.", circuit.service_name));
            }
        }

        // Check health statuses
        for health_check in health_checks.values() {
            if health_check.status == HealthStatus::Unhealthy {
                recommendations.push(format!("Service {} component {} is UNHEALTHY: {}",
                    health_check.service_name, health_check.component,
                    health_check.error_message.as_deref().unwrap_or("Unknown error")));
            }
        }

        if recommendations.is_empty() {
            recommendations.push("All systems operating normally.".to_string());
        }

        recommendations
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
