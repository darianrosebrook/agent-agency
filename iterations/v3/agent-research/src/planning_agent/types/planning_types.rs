//! Core types for the Planning Agent
//!
//! This module contains all the core data structures used by the planning agent
//! including configuration, requests, responses, and validation types.

use schemars::JsonSchema;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Configuration for the planning agent

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningConfig {
    /// Maximum time allowed for planning (in seconds)
    pub max_planning_time_seconds: u64,

    /// Maximum refinement iterations
    pub max_refinement_iterations: u32,

    /// Whether to enable automatic refinement
    pub enable_auto_refinement: bool,

    /// Risk tier escalation thresholds
    pub risk_escalation_thresholds: RiskEscalationThresholds,
}

impl Default for PlanningConfig {
    fn default() -> Self {
        Self {
            max_planning_time_seconds: 300, // 5 minutes
            max_refinement_iterations: 3,
            enable_auto_refinement: true,
            risk_escalation_thresholds: RiskEscalationThresholds::default(),
        }
    }
}

/// Risk escalation thresholds

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RiskEscalationThresholds {
    /// Maximum files for T1 tasks before escalation
    pub t1_max_files: u32,

    /// Maximum LOC for T1 tasks before escalation
    pub t1_max_loc: u32,

    /// Maximum duration for T1 tasks before escalation
    pub t1_max_duration_hours: u32,
}

impl Default for RiskEscalationThresholds {
    fn default() -> Self {
        Self {
            t1_max_files: 25,
            t1_max_loc: 1000,
            t1_max_duration_hours: 8,
        }
    }
}

/// Request to the planning agent

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningRequest {
    /// The task request to plan
    pub task_request: agent_agency_contracts::task_request::TaskRequest,

    /// Planning configuration override (optional)
    pub config_override: Option<PlanningConfig>,
}

/// Response from the planning agent

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningResponse {
    /// Generated working specification
    pub working_spec: agent_agency_contracts::working_spec::WorkingSpec,

    /// Planning metadata
    pub metadata: PlanningMetadata,

    /// Validation results
    pub validation_results: ValidationResults,

    /// Refinement history (if any refinements were applied)
    pub refinement_history: Vec<RefinementRecord>,
}

/// Planning operation metadata

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningMetadata {
    /// Total planning time
    pub planning_duration: Duration,

    /// Number of refinement iterations performed
    pub refinement_iterations: u32,

    /// Whether human intervention was required
    pub human_intervention_required: bool,

    /// Risk assessment result
    pub risk_assessment: RiskAssessment,
}

/// Risk assessment result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RiskAssessment {
    /// Assessed risk tier
    pub assessed_tier: agent_agency_contracts::task_request::RiskTier,

    /// Risk factors identified
    pub risk_factors: Vec<String>,

    /// Whether escalation is recommended
    pub escalation_recommended: bool,
}

/// Validation results summary

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationResults {
    /// Overall validation status
    pub overall_status: ValidationStatus,

    /// CAWS compliance score (0.0-1.0)
    pub caws_compliance_score: f64,

    /// Individual validation issues
    pub issues: Vec<ValidationIssue>,

    /// Applied refinements
    pub applied_refinements: Vec<String>,
}

/// Validation status

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum ValidationStatus {
    Passed,
    PassedWithRefinements,
    Failed,
    EscalationRequired,
}

/// Individual validation issue

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationIssue {
    /// Issue severity
    pub severity: IssueSeverity,

    /// Issue category
    pub category: String,

    /// Human-readable description
    pub description: String,

    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Issue severity levels

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Refinement record

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RefinementRecord {
    /// Refinement iteration number
    pub iteration: u32,

    /// Issues that triggered refinement
    pub triggering_issues: Vec<String>,

    /// Applied refinement actions
    pub applied_actions: Vec<String>,

    /// Whether refinement was successful
    pub successful: bool,
}
