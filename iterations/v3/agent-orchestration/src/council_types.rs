//! Council types - Re-export types from contracts for backward compatibility

use std::collections::HashMap;

// Re-export types for backward compatibility
pub use agent_agency_contracts::task_request::RiskTier;
pub use agent_agency_contracts::working_spec::{
    WorkingSpec, WorkingSpecConstraints, WorkingSpecContext, TestPlan, RollbackPlan
};
pub use agent_agency_contracts::refinement_decision::{CouncilDecision, JudgeType};
pub use agent_agency_contracts::final_verdict::FinalVerdictContract;

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Final verdict from council decision making
#[derive(Debug, Clone)]
pub struct FinalVerdict {
    pub decision: String,
    pub confidence: f64,
    pub summary: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task structure for planning
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub scope: Vec<String>,
    pub change_budget: ChangeBudget,
    pub blast_radius: BlastRadius,
    pub priority: u32,
}

/// Change budget for task execution
#[derive(Debug, Clone)]
pub struct ChangeBudget {
    pub max_files: u32,
    pub max_loc: u32,
}

/// Blast radius for task impact
#[derive(Debug, Clone)]
pub struct BlastRadius {
    pub modules: Vec<String>,
    pub data_migration: bool,
}
