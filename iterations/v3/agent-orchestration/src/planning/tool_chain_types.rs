//! Local tool chain types for feature-gated system-federated-ml dependency
//!
//! These are lightweight, local implementations of types from system-federated-ml.
//! They allow the orchestration crate to compile without the heavy ML dependencies.
//!
//! When the "tool-chain" feature is enabled, these are replaced with the real types
//! from system-federated-ml. When disabled, they provide minimal viable implementations.
//!
//! @author @darianrosebrook

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::collections::HashMap;

/// Tool chain planner interface (local implementation)
#[derive(Debug, Clone)]
pub struct ToolChainPlanner {
    /// Placeholder for planner state
    pub name: String,
}

impl Default for ToolChainPlanner {
    fn default() -> Self {
        Self {
            name: "local-planner".to_string(),
        }
    }
}

/// Planning context for tool chain operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningContext {
    /// Working spec identifier
    pub working_spec_id: String,
    /// Available tools
    pub available_tools: Vec<String>,
    /// Planning constraints
    pub constraints: PlanningConstraints,
    /// Risk tolerance level
    pub risk_tolerance: RiskLevel,

    /// Task description (added for compatibility)
    pub task_description: Option<String>,
    /// Task type classification
    pub task_type: Option<String>,
    /// Task complexity assessment
    pub complexity: Option<TaskComplexity>,
    /// Required tool capabilities
    pub required_capabilities: Vec<String>,
    /// Time budget in milliseconds
    pub time_budget_ms: Option<u64>,
    /// Cost budget in cents
    pub cost_budget_cents: Option<u64>,
}

/// Planning constraints for tool selection
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanningConstraints {
    /// Maximum execution time in seconds
    pub max_execution_time_secs: Option<u64>,
    /// Maximum tool chain length
    pub max_chain_length: Option<usize>,
    /// Required tool capabilities
    pub required_capabilities: Vec<String>,
    /// Prohibited tool types
    pub prohibited_tools: Vec<String>,

    /// Maximum parallelism level
    pub max_parallelism: Option<usize>,
    /// Maximum cost in cents
    pub max_cost_cents: Option<u64>,
    /// Maximum time in milliseconds
    pub max_time_ms: Option<u64>,
    /// Whether to require fallback mechanisms
    pub require_fallbacks: bool,
}

impl Default for PlanningConstraints {
    fn default() -> Self {
        Self {
            max_execution_time_secs: None,
            max_chain_length: Some(5),
            required_capabilities: Vec::new(),
            prohibited_tools: Vec::new(),
            max_parallelism: Some(3),
            max_cost_cents: None,
            max_time_ms: None,
            require_fallbacks: false,
        }
    }
}

/// Tool chain definition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolChain {
    /// Chain identifier
    pub id: String,
    /// Sequence of tool nodes
    pub nodes: Vec<ToolNode>,
    /// Expected total execution time
    pub estimated_duration_secs: u64,
}

/// Individual tool node in a chain
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolNode {
    /// Node identifier
    pub id: String,
    /// Tool name
    pub tool_name: String,
    /// Tool version
    pub tool_version: String,
    /// Input parameters
    pub inputs: HashMap<String, serde_json::Value>,
    /// Expected output schema
    pub output_schema: Option<serde_json::Value>,
    /// Dependencies on other nodes
    pub dependencies: Vec<String>,
}

/// Task complexity assessment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskComplexity {
    /// Very simple task
    VerySimple,
    /// Simple task
    Simple,
    /// Moderate complexity
    Moderate,
    /// Complex task
    Complex,
    /// Very complex task
    VeryComplex,
}

/// Risk tolerance levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Conservative - prefer safe, proven tools
    Conservative,
    /// Balanced - mix of safe and experimental tools
    Balanced,
    /// Aggressive - allow experimental and high-risk tools
    Aggressive,
}

/// Schema registry interface (local implementation)
#[derive(Debug, Clone)]
pub struct SchemaRegistry {
    /// Placeholder for registry state
    pub name: String,
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        Self {
            name: "local-schema-registry".to_string(),
        }
    }
}

/// Tool registry interface (local implementation)
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    /// Placeholder for registry state
    pub name: String,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self {
            name: "local-tool-registry".to_string(),
        }
    }
}

// Feature-gated re-exports for compatibility
#[cfg(feature = "tool-chain")]
pub use system_federated_ml::{
    tool_chain_planner::{
        ToolChainPlanner as ExternalToolChainPlanner,
        PlanningContext as ExternalPlanningContext,
        PlanningConstraints as ExternalPlanningConstraints,
        ToolChain as ExternalToolChain,
        ToolNode as ExternalToolNode,
        TaskComplexity as ExternalTaskComplexity,
        RiskLevel as ExternalRiskLevel,
    },
    tool_chain_planner::SchemaRegistry as ExternalSchemaRegistry,
    tool_registry::ToolRegistry as ExternalToolRegistry,
};

#[cfg(not(feature = "tool-chain"))]
pub use self::{
    ToolChainPlanner as ExternalToolChainPlanner,
    PlanningContext as ExternalPlanningContext,
    PlanningConstraints as ExternalPlanningConstraints,
    ToolChain as ExternalToolChain,
    ToolNode as ExternalToolNode,
    TaskComplexity as ExternalTaskComplexity,
    RiskLevel as ExternalRiskLevel,
    SchemaRegistry as ExternalSchemaRegistry,
    ToolRegistry as ExternalToolRegistry,
};

