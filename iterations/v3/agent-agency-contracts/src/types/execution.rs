//! Execution-related types and DTOs
//!
//! Data structures for execution plans, contexts, and milestones.
//! These types define the execution domain and are shared across planning and execution components.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Execution context for planning operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionContext {
    /// Session identifier
    #[schemars(with = "String")]
    pub session_id: Uuid,
    /// Planning engine name
    pub planning_engine: String,
    /// Engine version
    pub engine_version: String,
    /// Planning metadata
    pub planning_metadata: std::collections::HashMap<String, serde_json::Value>,
}

// Milestone and MilestoneScope definitions moved to planning_io.rs for consolidation

/// Acceptance criterion in Given-When-Then format
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AcceptanceCriterion {
    /// Unique identifier
    pub id: String,
    /// Given condition
    pub given: String,
    /// When action
    pub when: String,
    /// Then expected outcome
    pub then: String,
}

/// Evidence gate for milestone completion
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceGate {
    /// Gate identifier
    pub id: String,
    /// Evidence type required
    pub evidence_type: String,
    /// Description of required evidence
    pub description: String,
    /// Whether this gate is required
    pub required: bool,
}
