//! Local type definitions for data infrastructure to avoid circular dependencies

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditTrailEntry {
    pub id: Uuid,
    pub event_type: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Database operations trait
#[async_trait]
pub trait DatabaseOperations: Send + Sync {
    async fn create_execution_plan(&self, plan: CreateExecutionPlan) -> Result<models::ExecutionPlan, anyhow::Error>;
    async fn get_execution_plan(&self, id: Uuid) -> Result<Option<models::ExecutionPlan>, anyhow::Error>;
    async fn create_audit_trail_entry(&self, entry: AuditTrailEntry) -> Result<(), anyhow::Error>;
    async fn create_planning_session(&self, session: CreatePlanningSession) -> Result<models::PlanningSession, anyhow::Error>;
    async fn get_planning_session(&self, id: Uuid) -> Result<Option<models::PlanningSession>, anyhow::Error>;
    async fn update_planning_session(&self, id: Uuid, session: UpdatePlanningSession) -> Result<(), anyhow::Error>;
    async fn create_planning_telemetry(&self, telemetry: CreatePlanningTelemetry) -> Result<(), anyhow::Error>;
    async fn create_planning_audit_event(&self, event: CreatePlanningAuditEvent) -> Result<(), anyhow::Error>;
    // Add other methods as needed
}

/// Create execution plan request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateExecutionPlan {
    pub id: Uuid,
    pub title: String,
    pub overview: String,
    // Add other fields as needed
}

/// Update execution plan request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateExecutionPlan {
    pub id: Uuid,
    pub title: Option<String>,
    pub overview: Option<String>,
    pub status: Option<String>,
    // Add other fields as needed
}

/// Models namespace
pub mod models {
    use super::*;

    /// Execution plan model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct ExecutionPlan {
        pub id: Uuid,
        pub title: String,
        pub overview: String,
        // Add other fields as needed
    }

    /// Planning session model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PlanningSession {
        pub id: Uuid,
        pub plan_id: Uuid,
        pub status: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    /// Milestone model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Milestone {
        pub id: Uuid,
        pub plan_id: Uuid,
        pub title: String,
        pub description: String,
        pub status: String,
        pub created_at: DateTime<Utc>,
        pub updated_at: DateTime<Utc>,
    }

    /// Planning audit event model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PlanningAuditEvent {
        pub id: Uuid,
        pub session_id: Uuid,
        pub event_type: String,
        pub description: String,
        pub timestamp: DateTime<Utc>,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    /// Planning telemetry model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct PlanningTelemetry {
        pub id: Uuid,
        pub session_id: Uuid,
        pub metric_name: String,
        pub metric_value: f64,
        pub timestamp: DateTime<Utc>,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    /// Worker model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Worker {
        pub id: Uuid,
        pub worker_type: String,
        pub status: String,
        pub capabilities: Vec<String>,
        pub last_seen: DateTime<Utc>,
        pub metadata: HashMap<String, serde_json::Value>,
    }

    /// Waiver model
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Waiver {
        pub id: Uuid,
        pub plan_id: Uuid,
        pub waiver_type: String,
        pub reason: String,
        pub approved_by: String,
        pub created_at: DateTime<Utc>,
        pub expires_at: Option<DateTime<Utc>>,
    }
}

/// Create audit trail entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateAuditTrailEntry {
    pub event_type: String,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Create planning audit event
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePlanningAuditEvent {
    pub plan_id: Uuid,
    pub event_type: String,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Create planning session
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePlanningSession {
    pub plan_id: Uuid,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Update planning session
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdatePlanningSession {
    pub id: Uuid,
    pub status: Option<String>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Create planning telemetry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePlanningTelemetry {
    pub session_id: Uuid,
    pub metric_name: String,
    pub metric_value: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Create waiver
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateWaiver {
    pub plan_id: Uuid,
    pub reason: String,
    pub waived_gates: Vec<String>,
}

/// Update waiver
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateWaiver {
    pub id: Uuid,
    pub status: String,
    // Add other fields as needed
}

/// Cost limits
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CostLimits {
    pub max_cost: f64,
    pub currency: String,
}

