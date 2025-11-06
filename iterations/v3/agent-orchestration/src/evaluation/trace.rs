//! Versioned Execution Trace Model
//!
//! This module provides a unified, versioned event sourcing model for all orchestration
//! events, enabling stable schemas that evolve over time while maintaining backward compatibility.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::chain_of_thought::{DecisionPoint, CoordinationEvent};
use crate::audit_trail::AuditEvent;

/// Current trace version
pub const CURRENT_TRACE_VERSION: u16 = 1;

/// Versioned event envelope for all orchestration events
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EventEnvelope {
    /// Trace schema version for forward compatibility
    pub trace_version: u16,
    
    /// Plan identifier this event belongs to
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    
    /// Correlation ID tying decision → action → outcome
    #[schemars(with = "String")]
    pub correlation_id: Uuid,
    
    /// Event timestamp
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    
    /// Event kind (non-exhaustive for forward compatibility)
    pub kind: EventKind,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Event kinds (non-exhaustive enum for forward compatibility)
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum EventKind {
    /// Decision point from chain-of-thought
    Decision(DecisionPoint),
    
    /// Worker assignment event
    WorkerAssigned {
        #[schemars(with = "String")]
        worker_id: Uuid,
        milestone_id: String,
        capability_score: f64,
        load_factor: f64,
    },
    
    /// Worker released event
    WorkerReleased {
        #[schemars(with = "String")]
        worker_id: Uuid,
        milestone_id: String,
    },
    
    /// Coordination event
    Coordination(CoordinationEvent),
    
    /// Observation event
    Observation {
        component: String,
        observation: String,
        context: HashMap<String, serde_json::Value>,
    },
    
    /// Failure event
    Failure {
        failure_type: String,
        component: String,
        error_message: String,
        recoverable: bool,
        context: HashMap<String, serde_json::Value>,
    },
    
    /// Recovery event
    Recovery {
        recovery_strategy: String,
        recovery_duration_ms: u64,
        success: bool,
        fallback_used: bool,
        lessons_learned: Vec<String>,
    },
    
    /// Resource metric sample
    Metric {
        resource_type: String,
        cpu_utilization: f64,
        memory_utilization: f64,
        network_utilization: f64,
        disk_utilization: f64,
    },
    
    /// Audit trail event
    Audit(AuditEvent),
    
    /// Custom event type for extensibility
    Custom {
        event_type: String,
        data: serde_json::Value,
    },
}

/// Complete execution trace for a plan
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Trace {
    /// Plan identifier
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    
    /// Trace version
    pub trace_version: u16,
    
    /// All events in chronological order
    pub events: Vec<EventEnvelope>,
    
    /// Trace metadata
    pub metadata: TraceMetadata,
}

/// Trace metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TraceMetadata {
    /// When trace collection started
    #[schemars(with = "String")]
    pub started_at: DateTime<Utc>,
    
    /// When trace collection ended (None if still in progress)
    #[schemars(with = "Option<String>")]
    pub ended_at: Option<DateTime<Utc>>,
    
    /// Total number of events
    pub event_count: usize,
    
    /// Event type distribution
    pub event_type_distribution: HashMap<String, usize>,
    
    /// Correlation IDs present in trace
    #[schemars(with = "Vec<String>")]
    pub correlation_ids: Vec<Uuid>,
}

impl Trace {
    /// Create a new trace
    pub fn new(plan_id: Uuid) -> Self {
        Self {
            plan_id,
            trace_version: CURRENT_TRACE_VERSION,
            events: Vec::new(),
            metadata: TraceMetadata {
                started_at: Utc::now(),
                ended_at: None,
                event_count: 0,
                event_type_distribution: HashMap::new(),
                correlation_ids: Vec::new(),
            },
        }
    }
    
    /// Add an event to the trace
    pub fn add_event(&mut self, event: EventEnvelope) {
        // Update event type distribution
        let event_type = match &event.kind {
            EventKind::Decision(_) => "Decision",
            EventKind::WorkerAssigned { .. } => "WorkerAssigned",
            EventKind::WorkerReleased { .. } => "WorkerReleased",
            EventKind::Coordination(_) => "Coordination",
            EventKind::Observation { .. } => "Observation",
            EventKind::Failure { .. } => "Failure",
            EventKind::Recovery { .. } => "Recovery",
            EventKind::Metric { .. } => "Metric",
            EventKind::Audit(_) => "Audit",
            EventKind::Custom { event_type, .. } => event_type.as_str(),
        };
        
        *self.metadata.event_type_distribution.entry(event_type.to_string()).or_insert(0) += 1;
        
        // Track correlation ID
        if !self.metadata.correlation_ids.contains(&event.correlation_id) {
            self.metadata.correlation_ids.push(event.correlation_id);
        }
        
        self.events.push(event);
        self.metadata.event_count += 1;
    }
    
    /// Finalize the trace (mark as complete)
    pub fn finalize(&mut self) {
        self.metadata.ended_at = Some(Utc::now());
    }
    
    /// Get events filtered by correlation ID
    pub fn events_by_correlation(&self, correlation_id: Uuid) -> Vec<&EventEnvelope> {
        self.events.iter()
            .filter(|e| e.correlation_id == correlation_id)
            .collect()
    }
    
    /// Get events filtered by kind
    pub fn events_by_kind(&self, kind_name: &str) -> Vec<&EventEnvelope> {
        self.events.iter()
            .filter(|e| {
                match &e.kind {
                    EventKind::Decision(_) => kind_name == "Decision",
                    EventKind::WorkerAssigned { .. } => kind_name == "WorkerAssigned",
                    EventKind::WorkerReleased { .. } => kind_name == "WorkerReleased",
                    EventKind::Coordination(_) => kind_name == "Coordination",
                    EventKind::Observation { .. } => kind_name == "Observation",
                    EventKind::Failure { .. } => kind_name == "Failure",
                    EventKind::Recovery { .. } => kind_name == "Recovery",
                    EventKind::Metric { .. } => kind_name == "Metric",
                    EventKind::Audit(_) => kind_name == "Audit",
                    EventKind::Custom { event_type, .. } => event_type == kind_name,
                }
            })
            .collect()
    }
    
    /// Get events within time window
    pub fn events_in_window(&self, since: DateTime<Utc>, until: DateTime<Utc>) -> Vec<&EventEnvelope> {
        self.events.iter()
            .filter(|e| e.timestamp >= since && e.timestamp <= until)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trace_creation() {
        let plan_id = Uuid::new_v4();
        let trace = Trace::new(plan_id);
        
        assert_eq!(trace.plan_id, plan_id);
        assert_eq!(trace.trace_version, CURRENT_TRACE_VERSION);
        assert_eq!(trace.events.len(), 0);
        assert_eq!(trace.metadata.event_count, 0);
    }
    
    #[test]
    fn test_trace_add_event() {
        let plan_id = Uuid::new_v4();
        let mut trace = Trace::new(plan_id);
        let correlation_id = Uuid::new_v4();
        
        let event = EventEnvelope {
            trace_version: CURRENT_TRACE_VERSION,
            plan_id,
            correlation_id,
            timestamp: Utc::now(),
            kind: EventKind::Observation {
                component: "test".to_string(),
                observation: "test observation".to_string(),
                context: HashMap::new(),
            },
            metadata: HashMap::new(),
        };
        
        trace.add_event(event);
        
        assert_eq!(trace.events.len(), 1);
        assert_eq!(trace.metadata.event_count, 1);
        assert_eq!(trace.metadata.event_type_distribution.get("Observation"), Some(&1));
    }
    
    #[test]
    fn test_trace_finalize() {
        let plan_id = Uuid::new_v4();
        let mut trace = Trace::new(plan_id);
        
        trace.finalize();
        
        assert!(trace.metadata.ended_at.is_some());
    }
}
