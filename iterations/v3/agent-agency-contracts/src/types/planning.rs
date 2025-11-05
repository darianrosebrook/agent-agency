//! Planning-related types and DTOs
//!
//! Core data structures for task planning, execution modes, and impact analysis.
//! These types are shared across multiple crates and define the planning domain.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;

/// Execution mode for task orchestration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ExecutionMode {
    /// Dry run - validate without execution
    DryRun,
    /// Auto - execute automatically
    Auto,
    /// Strict - execute with full validation
    Strict,
}

/// Task scope definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TaskScope {
    /// Files and directories that are in scope
    pub in_scope: Vec<String>,
    /// Files and directories that are explicitly out of scope
    pub out_scope: Vec<String>,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TaskPriority {
    Low,
    Normal,
    Medium,
    High,
    Urgent,
    Critical,
}

/// Risk tier assessment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    /// Tier 1: Critical systems (auth, billing, migrations)
    Tier1 = 1,
    /// Tier 2: Standard features (APIs, data writes)
    Tier2 = 2,
    /// Tier 3: Low risk (UI, internal tools)
    Tier3 = 3,
}

/// Planning strategy for execution plan generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PlanningStrategy {
    /// Top-down decomposition from requirements
    TopDown,

    /// Bottom-up composition from tool chains
    BottomUp,

    /// Dependency-driven critical path analysis
    DependencyDriven,

    /// Risk-based milestone prioritization
    RiskBased,

    /// Hybrid strategy combining approaches
    Hybrid,

    /// AI-assisted planning with human oversight
    AIAssisted,

    /// Template-based planning from patterns
    TemplateBased,
}

/// Blast radius for change impact analysis
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlastRadius {
    /// Affected modules
    pub modules: Vec<String>,
    /// Whether data migration is required
    pub data_migration: bool,
    /// External dependencies affected
    pub external_deps: Vec<String>,
}

/// Task descriptor for orchestration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskDescriptor {
    /// Unique task identifier
    #[schemars(with = "String")]
    pub task_id: Uuid,
    /// Human-readable task description
    pub description: String,
    /// Change budget constraints
    pub change_budget: super::super::planning_io::ChangeBudget,
    /// Task execution priority
    pub priority: TaskPriority,
    /// Execution mode
    pub execution_mode: ExecutionMode,
    /// Risk tier assessment
    pub risk_tier: Option<RiskTier>,
    /// Blast radius analysis
    pub blast_radius: BlastRadius,
    /// Scope restrictions (in)
    pub scope_in: super::super::task_request::ScopeRestrictions,
    /// Scope restrictions (out)
    pub scope_out: Option<super::super::task_request::ScopeRestrictions>,
    /// Acceptance criteria
    pub acceptance: Option<String>,
}
