//! Council types - Re-export types from contracts for backward compatibility

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export types for backward compatibility
pub use agent_agency_contracts::final_verdict::FinalVerdictContract;
pub use agent_agency_contracts::planning_io::ChangeBudget;
pub use agent_agency_contracts::refinement_decision::{CouncilDecision, JudgeType};
#[allow(ambiguous_glob_reexports)]
pub use agent_agency_contracts::types::prelude::*; // Includes TaskPriority and RiskTier
pub use agent_agency_contracts::working_spec::{
    RollbackPlan, TestPlan, WorkingSpec, WorkingSpecConstraints, WorkingSpecContext,
}; // Use contracts ChangeBudget

/// Consensus result from council decision making
///
/// This type represents the result of a council consensus process,
/// indicating whether a task was approved and the confidence level.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConsensusResult {
    /// Whether the task was approved
    pub approved: bool,
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    /// Reason for the decision
    pub reason: String,
}

/// Final verdict from council decision making

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FinalVerdict {
    pub decision: String,
    pub confidence: f64,
    pub summary: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task structure for planning

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub scope: Vec<String>,
    pub change_budget: ChangeBudget, // Now uses contracts ChangeBudget
    pub blast_radius: BlastRadius,
    pub priority: u32,
}

// ChangeBudget and BlastRadius are now imported from agent_agency_contracts
// No duplicate definitions needed
