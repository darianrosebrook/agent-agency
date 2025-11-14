//! Planning Engine Port
//!
//! Defines the interface for planning engines that can generate execution plans
//! from task descriptors. This port enables dependency injection and testing.
//!
//! @author @darianrosebrook

use crate::errors::PlanningResult;
use crate::planning_io::ExecutionPlan;
use crate::types::execution::ExecutionContext;
use crate::types::planning::TaskDescriptor;

/// Core planning engine interface
/// Implementations provide the logic to generate execution plans from task descriptors
#[async_trait::async_trait]
pub trait PlanningEngine: Send + Sync {
    /// Generate an execution plan for the given task descriptor
    ///
    /// # Arguments
    /// * `ctx` - Execution context with session and engine information
    /// * `task` - Task descriptor containing requirements and constraints
    ///
    /// # Returns
    /// Execution plan with milestones and dependencies, or an error if planning fails
    async fn generate_plan(
        &self,
        ctx: &ExecutionContext,
        task: &TaskDescriptor,
    ) -> PlanningResult<ExecutionPlan>;
}
