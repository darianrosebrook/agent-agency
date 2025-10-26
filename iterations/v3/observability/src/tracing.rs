//! Distributed tracing implementation

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde_json;
use tracing::{info, debug, warn};

use crate::trace_types::*;
use crate::span_management::*;
use crate::trace_hierarchy::*;
use crate::health_monitoring::*;
use crate::otel_integration::*;

/// Main trace collector that orchestrates all tracing components
#[derive(Debug)]
pub struct TraceCollector {
    /// Configuration for the tracing system
    config: TraceConfig,
    /// Span lifecycle manager
    span_manager: SpanManager,
    /// Trace hierarchy manager
    hierarchy_manager: TraceHierarchyManager,
    /// Health monitoring system
    health_monitor: HealthMonitor,
    /// OpenTelemetry integration
    otel_integrator: OtelIntegrator,
    /// Completed traces storage
    completed_traces: Arc<RwLock<Vec<TraceInfo>>>,
}

impl TraceCollector {
    /// Create a new trace collector with all component managers
    pub async fn new(config: TraceConfig) -> Result<Self> {
        // Initialize shared storage
        let active_spans = Arc::new(RwLock::new(HashMap::new()));
        let completed_traces = Arc::new(RwLock::new(Vec::new()));
        let trace_hierarchies = Arc::new(RwLock::new(HashMap::new()));
        let span_relationships = Arc::new(RwLock::new(HashMap::new()));
        let health_checks = Arc::new(RwLock::new(HashMap::new()));
        let circuit_breakers = Arc::new(RwLock::new(HashMap::new()));
        let system_health = Arc::new(RwLock::new(SystemHealthSnapshot {
            overall_healthy: true,
            component_health: HashMap::new(),
            circuit_breakers: HashMap::new(),
            metrics: HashMap::new(),
            timestamp: chrono::Utc::now(),
        }));

        // Initialize OpenTelemetry integrator
        let otel_integrator = OtelIntegrator::new(config.clone())?;

        // Create component managers
        let span_manager = SpanManager::new(
            config.clone(),
            active_spans.clone(),
            otel_integrator.get_tracer(),
        );

        let hierarchy_manager = TraceHierarchyManager::new(
            config.clone(),
            trace_hierarchies.clone(),
            span_relationships.clone(),
        );

        let health_monitor = HealthMonitor::new(
            config.clone(),
            health_checks.clone(),
            circuit_breakers.clone(),
            system_health.clone(),
        );

        Ok(Self {
            config,
            span_manager,
            hierarchy_manager,
            health_monitor,
            otel_integrator,
            completed_traces,
        })
    }

    /// Start a new trace span
    pub async fn start_span(
        &self,
        operation: &str,
        parent_trace_id: Option<&str>,
        attributes: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        // Start span using span manager
        let span_id = self.span_manager.start_span(operation, parent_trace_id, attributes).await?;

        // Track in hierarchy
        self.hierarchy_manager.track_span_start(
            &span_id,
            parent_trace_id,
            &self.config.service_name,
            operation,
        ).await?;

        Ok(span_id)
    }

    /// End a trace span
    pub async fn end_span(&self, span_id: &str, status: SpanStatus) -> Result<()> {
        // End span using span manager
        self.span_manager.end_span(span_id, status).await?;

        // Track completion in hierarchy
        self.hierarchy_manager.track_span_end(span_id).await?;

        // Check if this completes a trace
        if let Some(trace_id) = self.hierarchy_manager.extract_trace_id(span_id).await {
            if trace_id == span_id {
                // This is a root span, complete the trace
                self.complete_trace(&trace_id).await?;
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
        self.span_manager.add_span_event(span_id, event_name, attributes).await
    }

    /// Add attributes to a span
    pub async fn add_span_attributes(
        &self,
        span_id: &str,
        attributes: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        self.span_manager.add_span_attributes(span_id, attributes).await
    }

    /// Get span information
    pub async fn get_span_info(&self, span_id: &str) -> Result<Option<SpanInfo>> {
        self.span_manager.get_span_info(span_id).await
    }

    /// Perform a health check
    pub async fn perform_health_check(&self, component: &str) -> Result<HealthCheckResult> {
        self.health_monitor.perform_health_check(component).await
    }

    /// Get system health snapshot
    pub async fn get_system_health(&self) -> Result<SystemHealthSnapshot> {
        self.health_monitor.generate_system_health_snapshot().await
    }

    /// Get trace hierarchy
    pub async fn get_trace_hierarchy(&self, trace_id: &str) -> Option<TraceHierarchy> {
        self.hierarchy_manager.get_trace_hierarchy(trace_id).await
    }

    /// Complete a trace and store it
    async fn complete_trace(&self, trace_id: &str) -> Result<()> {
        // Build the complete trace hierarchy
        let hierarchy = self.hierarchy_manager.build_trace_hierarchy(trace_id).await?;

        // Create trace info
        let root_span = hierarchy.spans.get(&hierarchy.root_span_id)
            .ok_or_else(|| anyhow::anyhow!("Root span not found in hierarchy"))?;

        let child_spans: Vec<SpanInfo> = hierarchy.spans
            .iter()
            .filter(|(id, _)| *id != &hierarchy.root_span_id)
            .map(|(_, info)| SpanInfo {
                name: info.operation.clone(),
                start_time: info.start_time,
                end_time: info.end_time,
                duration_ms: info.duration_ms,
                attributes: serde_json::json!({
                    "service": info.service_name,
                    "depth": info.depth
                }).as_object().unwrap().clone(),
                events: Vec::new(), // Events not stored in hierarchy
                status: SpanStatus::Unset, // Status not stored in hierarchy
            })
            .collect();

        let trace_info = TraceInfo {
            trace_id: hierarchy.trace_id,
            root_span: SpanInfo {
                name: root_span.operation.clone(),
                start_time: root_span.start_time,
                end_time: root_span.end_time,
                duration_ms: root_span.duration_ms,
                attributes: serde_json::json!({
                    "service": root_span.service_name
                }).as_object().unwrap().clone(),
                events: Vec::new(),
                status: SpanStatus::Unset,
            },
            child_spans,
            duration_ms: root_span.duration_ms.unwrap_or(0),
            status: TraceStatus::Success, // Assume success for now
        };

        // Store completed trace
        let mut completed_traces = self.completed_traces.write().await;
        completed_traces.push(trace_info);

        info!("Completed trace: {}", trace_id);
        Ok(())
    }

    /// Get completed traces
    pub async fn get_completed_traces(&self) -> Vec<TraceInfo> {
        let completed_traces = self.completed_traces.read().await;
        completed_traces.clone()
    }

    /// Get tracing configuration
    pub fn config(&self) -> &TraceConfig {
        &self.config
    }
}
