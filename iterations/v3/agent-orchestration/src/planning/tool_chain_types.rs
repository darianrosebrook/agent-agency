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
use anyhow::Result;

/// Tool chain planner interface (local implementation)

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[cfg(not(feature = "tool-chain"))]
impl ToolChainPlanner {
    /// Plan a tool chain (basic implementation without ML dependencies)
    /// 
    /// This implementation creates a functional tool chain based on:
    /// - Available tools from the planning context
    /// - Planning constraints (max length, prohibited tools, etc.)
    /// - Required capabilities
    /// - Sequential execution order respecting dependencies
    pub async fn plan_chain(
        &self,
        context: &PlanningContext,
        constraints: &PlanningConstraints,
    ) -> Result<ToolChain, anyhow::Error> {
        // Filter available tools based on constraints
        let mut candidate_tools: Vec<String> = context.available_tools
            .iter()
            .filter(|tool| {
                // Exclude prohibited tools
                !constraints.prohibited_tools.contains(tool)
            })
            .cloned()
            .collect();

        // Filter by required capabilities if specified
        if !constraints.required_capabilities.is_empty() {
            // Simple capability matching - tools that contain capability keywords
            candidate_tools.retain(|tool| {
                constraints.required_capabilities.iter().any(|cap| {
                    tool.to_lowercase().contains(&cap.to_lowercase())
                })
            });
        }

        // Apply max chain length constraint
        let max_length = constraints.max_chain_length
            .unwrap_or(10)
            .min(candidate_tools.len());
        
        let selected_tools: Vec<String> = candidate_tools
            .into_iter()
            .take(max_length)
            .collect();

        // Build tool nodes with sequential dependencies
        let mut nodes = Vec::new();
        let mut roots = Vec::new();
        let mut sinks = Vec::new();
        
        for (index, tool_name) in selected_tools.iter().enumerate() {
            let node_id = format!("node-{}", index);
            
            // First node has no dependencies (root)
            if index == 0 {
                roots.push(index);
            }
            
            // Last node is a sink
            if index == selected_tools.len() - 1 {
                sinks.push(index);
            }
            
            // Dependencies: previous node (for sequential execution)
            let dependencies = if index > 0 {
                vec![format!("node-{}", index - 1)]
            } else {
                vec![]
            };
            
            nodes.push(ToolNode {
                id: node_id.clone(),
                tool_name: tool_name.clone(),
                tool_version: "1.0.0".to_string(), // Default version
                inputs: HashMap::new(), // Inputs would be populated by caller
                output_schema: None, // Schema would be determined by tool registry
                dependencies,
            });
        }

        // Estimate duration: base time per tool + overhead
        // Simple heuristic: 5 seconds per tool + 2 seconds overhead per dependency
        let base_time_per_tool = 5u64;
        let overhead_per_dependency = 2u64;
        let total_dependencies: usize = nodes.iter()
            .map(|n| n.dependencies.len())
            .sum();
        let estimated_duration_secs = (nodes.len() as u64 * base_time_per_tool)
            + (total_dependencies as u64 * overhead_per_dependency);

        // Apply max execution time constraint if specified
        let final_duration = if let Some(max_time) = constraints.max_execution_time_secs {
            estimated_duration_secs.min(max_time)
        } else {
            estimated_duration_secs
        };

        // Check node properties before moving nodes into ToolChain
        let nodes_is_empty = nodes.is_empty();
        let nodes_len = nodes.len();
        
        Ok(ToolChain {
            id: format!("local-chain-{}", uuid::Uuid::new_v4()),
            nodes,
            estimated_duration_secs: final_duration,
            roots: if roots.is_empty() && !nodes_is_empty {
                vec![0] // Default: first node is root
            } else {
                roots
            },
            sinks: if sinks.is_empty() && !nodes_is_empty {
                vec![nodes_len - 1] // Default: last node is sink
            } else {
                sinks
            },
        })
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
    /// Root node indices (for DAG structure)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub roots: Vec<usize>,
    /// Sink node indices (for DAG structure)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sinks: Vec<usize>,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum RiskLevel {
    /// Conservative - prefer safe, proven tools
    Conservative,
    /// Balanced - mix of safe and experimental tools
    Balanced,
    /// Aggressive - allow experimental and high-risk tools
    Aggressive,
}

/// Schema registry interface (local implementation)

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
