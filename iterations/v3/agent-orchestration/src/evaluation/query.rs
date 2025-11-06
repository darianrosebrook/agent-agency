//! Query API for Event Traces
//!
//! Provides efficient querying of event traces with indexing and windowing support.

use crate::evaluation::trace::EventEnvelope;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Query structure for filtering events
#[derive(Debug, Clone)]
pub struct Query {
    /// Filter by plan ID
    pub plan_id: Option<Uuid>,
    
    /// Filter by correlation ID
    pub correlation_id: Option<Uuid>,
    
    /// Filter by event kinds (None = all kinds)
    pub kinds: Option<Vec<&'static str>>,
    
    /// Filter events after this time
    pub since: Option<DateTime<Utc>>,
    
    /// Filter events before this time
    pub until: Option<DateTime<Utc>>,
    
    /// Maximum number of results
    pub limit: Option<usize>,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            plan_id: None,
            correlation_id: None,
            kinds: None,
            since: None,
            until: None,
            limit: None,
        }
    }
}

impl Query {
    /// Create a new query
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Filter by plan ID
    pub fn with_plan_id(mut self, plan_id: Uuid) -> Self {
        self.plan_id = Some(plan_id);
        self
    }
    
    /// Filter by correlation ID
    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
    
    /// Filter by event kinds
    pub fn with_kinds(mut self, kinds: Vec<&'static str>) -> Self {
        self.kinds = Some(kinds);
        self
    }
    
    /// Filter by time window
    pub fn with_time_window(mut self, since: DateTime<Utc>, until: DateTime<Utc>) -> Self {
        self.since = Some(since);
        self.until = Some(until);
        self
    }
    
    /// Limit results
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Apply query to a slice of events
    pub fn apply<'a>(&self, events: &'a [EventEnvelope]) -> Vec<&'a EventEnvelope> {
        let mut results: Vec<&EventEnvelope> = events.iter()
            .filter(|e| {
                // Filter by plan_id
                if let Some(plan_id) = self.plan_id {
                    if e.plan_id != plan_id {
                        return false;
                    }
                }
                
                // Filter by correlation_id
                if let Some(correlation_id) = self.correlation_id {
                    if e.correlation_id != correlation_id {
                        return false;
                    }
                }
                
                // Filter by time window
                if let Some(since) = self.since {
                    if e.timestamp < since {
                        return false;
                    }
                }
                
                if let Some(until) = self.until {
                    if e.timestamp > until {
                        return false;
                    }
                }
                
                // Filter by kind
                if let Some(ref kinds) = self.kinds {
                    let event_kind_name = match &e.kind {
                        crate::evaluation::trace::EventKind::Decision(_) => "Decision",
                        crate::evaluation::trace::EventKind::WorkerAssigned { .. } => "WorkerAssigned",
                        crate::evaluation::trace::EventKind::WorkerReleased { .. } => "WorkerReleased",
                        crate::evaluation::trace::EventKind::Coordination(_) => "Coordination",
                        crate::evaluation::trace::EventKind::Observation { .. } => "Observation",
                        crate::evaluation::trace::EventKind::Failure { .. } => "Failure",
                        crate::evaluation::trace::EventKind::Recovery { .. } => "Recovery",
                        crate::evaluation::trace::EventKind::Metric { .. } => "Metric",
                        crate::evaluation::trace::EventKind::Audit(_) => "Audit",
                        crate::evaluation::trace::EventKind::Custom { event_type, .. } => event_type.as_str(),
                    };
                    
                    if !kinds.contains(&event_kind_name) {
                        return false;
                    }
                }
                
                true
            })
            .collect();
        
        // Apply limit
        if let Some(limit) = self.limit {
            results.truncate(limit);
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::trace::{EventEnvelope, EventKind};
    use std::collections::HashMap;
    
    #[test]
    fn test_query_by_plan_id() {
        let plan_id1 = Uuid::new_v4();
        let plan_id2 = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        
        let events = vec![
            EventEnvelope {
                trace_version: 1,
                plan_id: plan_id1,
                correlation_id,
                timestamp: Utc::now(),
                kind: EventKind::Observation {
                    component: "test".to_string(),
                    observation: "test".to_string(),
                    context: HashMap::new(),
                },
                metadata: HashMap::new(),
            },
            EventEnvelope {
                trace_version: 1,
                plan_id: plan_id2,
                correlation_id,
                timestamp: Utc::now(),
                kind: EventKind::Observation {
                    component: "test".to_string(),
                    observation: "test".to_string(),
                    context: HashMap::new(),
                },
                metadata: HashMap::new(),
            },
        ];
        
        let query = Query::new().with_plan_id(plan_id1);
        let results = query.apply(&events);
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].plan_id, plan_id1);
    }
    
    #[test]
    fn test_query_with_limit() {
        let plan_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        
        let events: Vec<EventEnvelope> = (0..10).map(|_| EventEnvelope {
            trace_version: 1,
            plan_id,
            correlation_id,
            timestamp: Utc::now(),
            kind: EventKind::Observation {
                component: "test".to_string(),
                observation: "test".to_string(),
                context: HashMap::new(),
            },
            metadata: HashMap::new(),
        }).collect();
        
        let query = Query::new().with_limit(5);
        let results = query.apply(&events);
        
        assert_eq!(results.len(), 5);
    }
}
