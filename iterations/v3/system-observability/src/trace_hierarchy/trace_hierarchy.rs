//! Trace Hierarchy Management
//!
//! Manages the hierarchical relationships between spans within traces,
//! including parent-child relationships, depth calculations, and trace structure.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::trace_types::*;

/// Hierarchical representation of a complete trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceHierarchy {
    /// Unique trace identifier
    pub trace_id: String,
    /// Root span identifier
    pub root_span_id: String,
    /// All spans in the hierarchy
    pub spans: HashMap<String, SpanHierarchyInfo>,
    /// Maximum depth of the trace hierarchy
    pub max_depth: u32,
    /// Total number of spans in the trace
    pub total_spans: u32,
    /// When the hierarchy was created
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Hierarchical information for a single span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanHierarchyInfo {
    /// Unique span identifier
    pub span_id: String,
    /// Parent span identifier (None for root spans)
    pub parent_span_id: Option<String>,
    /// Child span identifiers
    pub children: Vec<String>,
    /// Depth in the hierarchy (0 for root)
    pub depth: u32,
    /// Service that created the span
    pub service_name: String,
    /// Operation name
    pub operation: String,
    /// When the span started
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// When the span ended
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Span duration in milliseconds
    pub duration_ms: Option<u64>,
}

/// Analytics information about trace hierarchies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceAnalytics {
    /// Total number of traces analyzed
    pub total_traces: u64,
    /// Average trace depth
    pub avg_depth: f64,
    /// Average number of spans per trace
    pub avg_spans_per_trace: f64,
    /// Maximum trace depth seen
    pub max_depth_seen: u32,
    /// Most common service interactions
    pub service_interactions: HashMap<String, HashMap<String, u64>>,
    /// Performance bottlenecks identified
    pub bottlenecks: Vec<String>,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Manager for trace hierarchy operations
#[derive(Debug)]
pub struct TraceHierarchyManager {
    /// Configuration for hierarchy management
    config: TraceConfig,
    /// Hierarchy storage
    trace_hierarchies: Arc<RwLock<HashMap<String, TraceHierarchy>>>,
    /// Span relationship storage
    span_relationships: Arc<RwLock<HashMap<String, SpanHierarchyInfo>>>,
}

impl TraceHierarchyManager {
    /// Create a new trace hierarchy manager
    pub fn new(
        config: TraceConfig,
        trace_hierarchies: Arc<RwLock<HashMap<String, TraceHierarchy>>>,
        span_relationships: Arc<RwLock<HashMap<String, SpanHierarchyInfo>>>,
    ) -> Self {
        Self {
            config,
            trace_hierarchies,
            span_relationships,
        }
    }

    /// Track the start of a span in the hierarchy
    pub async fn track_span_start(
        &self,
        span_id: &str,
        parent_trace_id: Option<&str>,
        service_name: &str,
        operation: &str,
    ) -> Result<()> {
        let trace_id = parent_trace_id.unwrap_or(span_id);
        let hierarchy_info = SpanHierarchyInfo {
            span_id: span_id.to_string(),
            parent_span_id: parent_trace_id.map(|s| s.to_string()),
            children: Vec::new(),
            depth: 0, // Will be calculated when building hierarchy
            service_name: service_name.to_string(),
            operation: operation.to_string(),
            start_time: chrono::Utc::now(),
            end_time: None,
            duration_ms: None,
        };

        let mut span_relationships = self.span_relationships.write().await;
        span_relationships.insert(span_id.to_string(), hierarchy_info);

        // Update parent span's children list
        if let Some(parent_id) = parent_trace_id {
            if let Some(parent_info) = span_relationships.get_mut(parent_id) {
                if !parent_info.children.contains(&span_id.to_string()) {
                    parent_info.children.push(span_id.to_string());
                }
            }
        }

        Ok(())
    }

    /// Track the end of a span in the hierarchy
    pub async fn track_span_end(&self, span_id: &str) -> Result<()> {
        let mut span_relationships = self.span_relationships.write().await;

        if let Some(span_info) = span_relationships.get_mut(span_id) {
            span_info.end_time = Some(chrono::Utc::now());

            if let (Some(start), Some(end)) = (span_info.start_time, span_info.end_time) {
                span_info.duration_ms = Some((end - start).num_milliseconds() as u64);
            }
        }

        Ok(())
    }

    /// Build complete trace hierarchy from span relationships
    pub async fn build_trace_hierarchy(&self, trace_id: &str) -> Result<TraceHierarchy> {
        let span_relationships = self.span_relationships.read().await;

        // Find root span (one with no parent or parent == trace_id)
        let root_span_id = span_relationships
            .iter()
            .find(|(_, info)| {
                info.parent_span_id.as_ref().map_or(true, |p| p == trace_id)
            })
            .map(|(id, _)| id.clone())
            .ok_or_else(|| anyhow::anyhow!("No root span found for trace {}", trace_id))?;

        // Calculate depths for all spans
        let mut spans = HashMap::new();
        self.calculate_span_depths(&span_relationships, &root_span_id, &mut spans, 0);

        let max_depth = spans.values().map(|info| info.depth).max().unwrap_or(0);
        let total_spans = spans.len() as u32;

        let hierarchy = TraceHierarchy {
            trace_id: trace_id.to_string(),
            root_span_id,
            spans,
            max_depth,
            total_spans,
            created_at: chrono::Utc::now(),
        };

        // Store the hierarchy
        let mut trace_hierarchies = self.trace_hierarchies.write().await;
        trace_hierarchies.insert(trace_id.to_string(), hierarchy.clone());

        Ok(hierarchy)
    }

    /// Calculate depths for all spans in the hierarchy
    fn calculate_span_depths(
        &self,
        relationships: &HashMap<String, SpanHierarchyInfo>,
        span_id: &str,
        spans: &mut HashMap<String, SpanHierarchyInfo>,
        depth: u32,
    ) {
        if let Some(info) = relationships.get(span_id) {
            let mut info_with_depth = info.clone();
            info_with_depth.depth = depth;
            spans.insert(span_id.to_string(), info_with_depth);

            // Recursively calculate depths for children
            for child_id in &info.children {
                self.calculate_span_depths(relationships, child_id, spans, depth + 1);
            }
        }
    }

    /// Get trace hierarchy for a given trace ID
    pub async fn get_trace_hierarchy(&self, trace_id: &str) -> Option<TraceHierarchy> {
        let trace_hierarchies = self.trace_hierarchies.read().await;
        trace_hierarchies.get(trace_id).cloned()
    }

    /// Get span hierarchy information
    pub async fn get_span_hierarchy_info(&self, span_id: &str) -> Option<SpanHierarchyInfo> {
        let span_relationships = self.span_relationships.read().await;
        span_relationships.get(span_id).cloned()
    }

    /// Get all spans at a specific depth in a trace
    pub async fn get_spans_at_depth(&self, trace_id: &str, depth: u32) -> Vec<String> {
        if let Some(hierarchy) = self.get_trace_hierarchy(trace_id).await {
            hierarchy.spans
                .iter()
                .filter(|(_, info)| info.depth == depth)
                .map(|(id, _)| id.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Extract trace ID from span ID
    pub async fn extract_trace_id(&self, span_id: &str) -> Option<String> {
        let span_relationships = self.span_relationships.read().await;

        // Walk up the hierarchy to find the root
        let mut current_id = span_id.to_string();
        let mut visited = std::collections::HashSet::new();

        while visited.insert(current_id.clone()) {
            if let Some(info) = span_relationships.get(&current_id) {
                if let Some(parent_id) = &info.parent_span_id {
                    current_id = parent_id.clone();
                } else {
                    // This is the root span, its ID is the trace ID
                    return Some(current_id);
                }
            } else {
                break;
            }
        }

        None
    }

    /// Analyze trace hierarchies for patterns and bottlenecks
    pub async fn analyze_hierarchies(&self) -> TraceAnalytics {
        let trace_hierarchies = self.trace_hierarchies.read().await;

        let total_traces = trace_hierarchies.len() as u64;
        if total_traces == 0 {
            return TraceAnalytics {
                total_traces: 0,
                avg_depth: 0.0,
                avg_spans_per_trace: 0.0,
                max_depth_seen: 0,
                service_interactions: HashMap::new(),
                bottlenecks: Vec::new(),
                last_updated: chrono::Utc::now(),
            };
        }

        let mut total_depth = 0u64;
        let mut total_spans = 0u64;
        let mut max_depth = 0u32;
        let mut service_interactions = HashMap::new();

        for hierarchy in trace_hierarchies.values() {
            total_depth += hierarchy.max_depth as u64;
            total_spans += hierarchy.total_spans as u64;
            max_depth = max_depth.max(hierarchy.max_depth);

            // Analyze service interactions
            for span_info in hierarchy.spans.values() {
                let service = span_info.service_name.clone();
                let operation = span_info.operation.clone();

                service_interactions
                    .entry(service)
                    .or_insert_with(HashMap::new)
                    .entry(operation)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }

        let avg_depth = total_depth as f64 / total_traces as f64;
        let avg_spans_per_trace = total_spans as f64 / total_traces as f64;

        // Identify potential bottlenecks (spans with high duration)
        let bottlenecks = self.identify_bottlenecks().await;

        TraceAnalytics {
            total_traces,
            avg_depth,
            avg_spans_per_trace,
            max_depth_seen: max_depth,
            service_interactions,
            bottlenecks,
            last_updated: chrono::Utc::now(),
        }
    }

    /// Identify potential performance bottlenecks
    async fn identify_bottlenecks(&self) -> Vec<String> {
        let mut bottlenecks = Vec::new();
        let span_relationships = self.span_relationships.read().await;

        for (span_id, info) in span_relationships.iter() {
            if let Some(duration) = info.duration_ms {
                if duration > 5000 { // 5 seconds
                    bottlenecks.push(format!(
                        "Span {} ({}) took {}ms",
                        span_id, info.operation, duration
                    ));
                }
            }
        }

        bottlenecks
    }
}
