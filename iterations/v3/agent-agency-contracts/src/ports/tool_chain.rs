//! Tool Chain Planner Port
//!
//! Defines the interface for planning and optimizing tool execution chains.
//! This port enables dependency injection and testing for tool chain planning.
//!
//! @author @darianrosebrook

use crate::errors::ToolChainResult;
use crate::types::tool_chain::{PlanningContext, PlanningStats, ToolChainPlan, ValidationResult};

/// Core tool chain planner interface
/// Implementations provide tool chain planning and optimization capabilities
#[async_trait::async_trait]
pub trait ToolChainPlanner: Send + Sync {
    /// Plan an optimal tool chain for task execution
    ///
    /// # Arguments
    /// * `context` - Planning context with task requirements
    ///
    /// # Returns
    /// Optimized tool chain plan, or an error if planning fails
    async fn plan_tool_chain(&self, context: PlanningContext) -> ToolChainResult<ToolChainPlan>;

    /// Validate a tool chain plan for correctness and feasibility
    ///
    /// # Arguments
    /// * `plan` - The tool chain plan to validate
    ///
    /// # Returns
    /// Validation result indicating issues and feasibility
    async fn validate_tool_chain(&self, plan: &ToolChainPlan) -> ToolChainResult<ValidationResult>;

    /// Optimize an existing tool chain plan
    ///
    /// # Arguments
    /// * `plan` - The tool chain plan to optimize
    /// * `optimization_criteria` - What to optimize for (cost, time, reliability)
    ///
    /// # Returns
    /// Optimized tool chain plan, or an error if optimization fails
    async fn optimize_tool_chain(
        &self,
        plan: &ToolChainPlan,
        optimization_criteria: Vec<String>,
    ) -> ToolChainResult<ToolChainPlan>;

    /// Get planning statistics and performance metrics
    ///
    /// # Returns
    /// Statistics about tool chain planning system
    async fn get_planning_stats(&self) -> ToolChainResult<PlanningStats>;
}
