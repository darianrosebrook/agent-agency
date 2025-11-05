//! Tool Chain Types - DTOs for tool chain operations
//!
//! Defines the data transfer objects used by the tool chain planner.
//! These types enable clean communication between orchestration and tool chain services.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Tool chain plan containing ordered tool execution sequence
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolChainPlan {
    /// Unique identifier for this tool chain
    pub id: String,
    /// Human-readable description of the tool chain
    pub description: String,
    /// Ordered sequence of tool IDs to execute
    pub tool_sequence: Vec<String>,
    /// Dependencies between tools (tool_id -> [dependent_tool_ids])
    pub dependencies: std::collections::HashMap<String, Vec<String>>,
    /// Estimated execution time in milliseconds
    pub estimated_duration_ms: u64,
    /// Estimated cost in cents
    pub estimated_cost_cents: u32,
    /// Risk assessment for this tool chain
    pub risk_assessment: RiskAssessment,
    /// Quality metrics for the planned chain
    pub quality_metrics: QualityMetrics,
}

/// Risk assessment for tool chain execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RiskAssessment {
    /// Overall risk level
    pub risk_level: RiskLevel,
    /// Risk factors identified
    pub risk_factors: Vec<String>,
    /// Mitigation strategies
    pub mitigation_strategies: Vec<String>,
    /// Confidence in successful execution (0.0 to 1.0)
    pub confidence_score: f64,
}

/// Quality metrics for tool chain evaluation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityMetrics {
    /// Efficiency score (0.0 to 1.0)
    pub efficiency_score: f64,
    /// Reliability score (0.0 to 1.0)
    pub reliability_score: f64,
    /// Cost-effectiveness score (0.0 to 1.0)
    pub cost_effectiveness_score: f64,
    /// Performance score (0.0 to 1.0)
    pub performance_score: f64,
}

/// Task complexity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskComplexity {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Risk tolerance levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Conservative,
    Balanced,
    Aggressive,
}

/// Planning context for tool chain generation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningContext {
    /// Description of the task to plan for
    pub task_description: String,
    /// Type/category of the task
    pub task_type: String,
    /// Complexity level of the task
    pub complexity: TaskComplexity,
    /// Required capabilities/tools
    pub required_capabilities: Vec<String>,
    /// Time budget in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_budget_ms: Option<u64>,
    /// Cost budget in cents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_budget_cents: Option<u32>,
    /// Risk tolerance for planning
    pub risk_tolerance: RiskLevel,
}

/// Validation result for tool chain plans - uses string issues with warnings
pub type ValidationResult = super::validation::ValidationResult<String>;

/// Statistics about tool chain planning system
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningStats {
    /// Total number of plans generated
    pub total_plans_generated: u64,
    /// Average planning time in milliseconds
    pub average_planning_time_ms: f64,
    /// Plan success rate (0.0 to 1.0)
    pub plan_success_rate: f64,
    /// Average optimization improvement (0.0 to 1.0)
    pub average_optimization_improvement: f64,
    /// Cache hit rate for plan reuse (0.0 to 1.0)
    pub cache_hit_rate: f64,
    /// Last planning operation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub last_planning_time: Option<chrono::DateTime<chrono::Utc>>,
}
