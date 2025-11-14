//! Database audit logging
//!
//! Comprehensive audit trail for database operations, security events,
//! and compliance monitoring with structured logging and retention.

use anyhow::Result;
use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, warn};
use uuid::Uuid;

/// Task audit event structure (matches task_audit_logs table)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, JsonSchema)]
pub struct TaskAuditEvent {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
    pub task_id: Uuid,
    #[schemars(with = "String")]
    pub ts: DateTime<Utc>,
    pub category: String,
    pub actor: String,
    pub action: String,
    pub payload: Value,
    pub idx: i64,
}

/// Database audit event
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseAuditEvent {
    #[schemars(with = "String")]
    pub id: Uuid,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub event_type: AuditEventType,
    pub actor: String,
    pub resource: String,
    pub action: String,
    pub details: Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub execution_time_ms: Option<u64>,
}

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AuditEventType {
    Query,
    Connection,
    Migration,
    HealthCheck,
    SecurityEvent,
    PerformanceIssue,
    ConfigurationChange,
}

/// Audit logger for database operations
#[derive(Debug)]
pub struct DatabaseAuditLogger {
    events: std::sync::RwLock<Vec<DatabaseAuditEvent>>,
    max_events: usize,
    retention_days: i64,
}

impl DatabaseAuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        Self {
            events: std::sync::RwLock::new(Vec::new()),
            max_events: 10000,  // Keep last 10k events in memory
            retention_days: 90, // Retain for 90 days
        }
    }

    /// Log a database operation
    pub async fn log_operation(&self, event: DatabaseAuditEvent) {
        debug!(
            "Logging audit event: {:?} on {}",
            event.event_type, event.resource
        );

        let mut events = self.events.write().unwrap();

        // Add new event
        events.push(event);

        // Maintain size limit
        if events.len() > self.max_events {
            let excess = events.len() - self.max_events;
            events.drain(0..excess);
        }

        // Clean up old events
        self.cleanup_old_events(&mut events);
    }

    /// Log a successful query operation
    pub async fn log_query_success(&self, actor: &str, query_type: &str, execution_time_ms: u64) {
        let event = DatabaseAuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::Query,
            actor: actor.to_string(),
            resource: "database".to_string(),
            action: query_type.to_string(),
            details: serde_json::json!({
                "query_type": query_type,
                "execution_time_ms": execution_time_ms
            }),
            ip_address: None,
            user_agent: None,
            success: true,
            execution_time_ms: Some(execution_time_ms),
        };

        self.log_operation(event).await;
    }

    /// Log a failed query operation
    pub async fn log_query_failure(&self, actor: &str, query_type: &str, error: &str) {
        let event = DatabaseAuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::Query,
            actor: actor.to_string(),
            resource: "database".to_string(),
            action: query_type.to_string(),
            details: serde_json::json!({
                "query_type": query_type,
                "error": error
            }),
            ip_address: None,
            user_agent: None,
            success: false,
            execution_time_ms: None,
        };

        self.log_operation(event).await;
    }

    /// Log a connection event
    pub async fn log_connection(&self, actor: &str, success: bool, details: Value) {
        let event = DatabaseAuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::Connection,
            actor: actor.to_string(),
            resource: "connection_pool".to_string(),
            action: if success { "connect" } else { "connect_failed" }.to_string(),
            details,
            ip_address: None,
            user_agent: None,
            success,
            execution_time_ms: None,
        };

        self.log_operation(event).await;
    }

    /// Log a security event
    pub async fn log_security_event(&self, actor: &str, event_type: &str, details: Value) {
        warn!("Security event: {} by {}", event_type, actor);

        let event = DatabaseAuditEvent {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: AuditEventType::SecurityEvent,
            actor: actor.to_string(),
            resource: "security".to_string(),
            action: event_type.to_string(),
            details,
            ip_address: None,
            user_agent: None,
            success: false, // Security events are typically failures/anomalies
            execution_time_ms: None,
        };

        self.log_operation(event).await;
    }

    /// Get audit events within a time range
    pub async fn get_events(
        &self,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
        event_type: Option<AuditEventType>,
        actor: Option<String>,
        limit: usize,
    ) -> Vec<DatabaseAuditEvent> {
        let events = self.events.read().unwrap();
        let end_time = end_time.unwrap_or_else(Utc::now);

        events
            .iter()
            .filter(|event| {
                event.timestamp >= start_time
                    && event.timestamp <= end_time
                    && event_type.as_ref().map_or(true, |et| {
                        std::mem::discriminant(et) == std::mem::discriminant(&event.event_type)
                    })
                    && actor.as_ref().map_or(true, |a| event.actor.contains(a))
            })
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get audit statistics
    pub async fn get_statistics(&self) -> AuditStatistics {
        let events = self.events.read().unwrap();

        let mut event_counts = HashMap::new();
        let mut success_count = 0;
        let mut failure_count = 0;
        let mut total_execution_time = 0u64;
        let mut execution_time_count = 0u64;

        for event in events.iter() {
            *event_counts
                .entry(format!("{:?}", event.event_type))
                .or_insert(0) += 1;

            if event.success {
                success_count += 1;
            } else {
                failure_count += 1;
            }

            if let Some(time) = event.execution_time_ms {
                total_execution_time += time;
                execution_time_count += 1;
            }
        }

        let avg_execution_time_ms = if execution_time_count > 0 {
            total_execution_time / execution_time_count
        } else {
            0
        };

        AuditStatistics {
            total_events: events.len(),
            event_counts,
            success_count,
            failure_count,
            avg_execution_time_ms,
            success_rate: if events.is_empty() {
                0.0
            } else {
                success_count as f64 / events.len() as f64
            },
        }
    }

    /// Export audit events to JSON
    pub async fn export_to_json(&self) -> Result<String> {
        let events = self.events.read().unwrap();
        serde_json::to_string_pretty(&*events).map_err(Into::into)
    }

    /// Clear old events based on retention policy
    fn cleanup_old_events(&self, events: &mut Vec<DatabaseAuditEvent>) {
        let cutoff = Utc::now() - chrono::Duration::days(self.retention_days);

        events.retain(|event| event.timestamp > cutoff);

        if events.len() < self.events.read().unwrap().len() {
            debug!(
                "Cleaned up {} old audit events",
                self.events.read().unwrap().len() - events.len()
            );
        }
    }
}

/// Audit statistics summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditStatistics {
    pub total_events: usize,
    pub event_counts: HashMap<String, usize>,
    pub success_count: usize,
    pub failure_count: usize,
    pub avg_execution_time_ms: u64,
    pub success_rate: f64,
}
