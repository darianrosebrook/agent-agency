//! Council models and data structures
//!
//! Contains the core model definitions for the council system,
//! including task specifications, judge evaluations, and consensus results.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Task specification for council evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub risk_tier: RiskTier,
    pub scope: TaskScope,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub context: TaskContext,
    pub worker_output: WorkerOutput,
    pub caws_spec: Option<CawsSpec>,
}

/// Risk tier for task classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskTier {
    Tier1, // Critical systems (auth, billing, migrations)
    Tier2, // Standard features (APIs, data writes)
    Tier3, // Low-risk changes (UI, docs, internal tools)
}

/// Task scope definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScope {
    pub files_affected: Vec<String>,
    pub max_files: Option<u32>,
    pub max_loc: Option<u32>,
    pub domains: Vec<String>,
}

/// Acceptance criterion for task validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub description: String,
}

/// Task execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub workspace_root: String,
    pub git_branch: String,
    pub recent_changes: Vec<String>,
    pub dependencies: std::collections::HashMap<String, serde_json::Value>,
    pub environment: Environment,
}

/// Environment types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

/// Worker output for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerOutput {
    pub content: String,
    pub files_modified: Vec<FileModification>,
    pub rationale: String,
    pub self_assessment: SelfAssessment,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// File modification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModification {
    pub path: String,
    pub operation: FileOperation,
    pub content: Option<String>,
    pub diff: Option<String>,
    pub size_bytes: u64,
}

/// File operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileOperation {
    Create,
    Modify,
    Delete,
    Move { from: String, to: String },
}

/// Worker self-assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAssessment {
    pub caws_compliance: f32,
    pub quality_score: f32,
    pub confidence: f32,
    pub concerns: Vec<String>,
    pub improvements: Vec<String>,
    pub estimated_effort: Option<String>,
}

/// CAWS specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsSpec {
    pub rules: Vec<String>,
    pub waivers: Vec<CawsWaiver>,
}

/// CAWS waiver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsWaiver {
    pub id: String,
    pub reason: String,
    pub justification: String,
    pub time_bounded: bool,
    pub expires_at: Option<DateTime<Utc>>,
}
