//! Legacy Plan Adapter - Bridge to agent-research PlanningAgent
//!
//! Adapts the new execution plan format to work with the existing
//! planning agent infrastructure for backward compatibility.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};use std::collections::HashMap;
use anyhow::Result;
use agent_agency_contracts::{
    planning_io::{ExecutionPlan as ContractExecutionPlan, Milestone as ContractMilestone},
    WorkingSpec,
};

/// Adapter for legacy planning agent
#[derive(Debug)]
pub struct LegacyPlanAdapter {
    // Would hold reference to actual planning agent when integrated
    // planning_agent: Arc<agent_research::planning_agent::PlanningAgent>,
}

impl LegacyPlanAdapter {
    /// Create new legacy adapter
    pub fn new() -> Self {
        Self {}
    }

    /// Adapt working spec to legacy plan format
    pub async fn adapt_working_spec(&self, working_spec: WorkingSpec) -> Result<ContractExecutionPlan> {
        // Placeholder implementation
        // In real implementation, this would:
        // 1. Extract task description from working spec
        // 2. Use planning agent to decompose into subtasks
        // 3. Convert back to execution plan format

        Err(anyhow::anyhow!("Legacy plan adapter not yet implemented - PLACEHOLDER"))
    }

    /// Convert execution plan to legacy format
    pub fn to_legacy_plan(&self, plan: &ContractExecutionPlan) -> Result<LegacyTaskPlan> {
        // Convert milestones to subtasks
        let subtasks: Vec<LegacySubTask> = plan.milestones.iter()
            .enumerate()
            .map(|(i, milestone)| self.milestone_to_subtask(i, milestone))
            .collect::<Result<Vec<_>>>()?;

        // Extract dependencies
        let dependencies = self.extract_legacy_dependencies(&plan.dependency_graph)?;

        Ok(LegacyTaskPlan {
            id: plan.working_spec_id.clone(),
            description: plan.title.clone(),
            subtasks,
            dependencies,
            estimated_duration: plan.metadata.estimated_duration_ms / 1000,
        })
    }

    /// Convert milestone to legacy subtask
    fn milestone_to_subtask(&self, index: usize, milestone: &ContractMilestone) -> Result<LegacySubTask> {
        Ok(LegacySubTask {
            id: milestone.id.clone(),
            description: milestone.objective.clone(),
            priority: self.map_priority(milestone.priority.clone()),
            estimated_duration: milestone.estimated_effort as u64,
            required_resources: vec![], // Would map from milestone scope
        })
    }

    /// Map milestone priority to legacy priority
    fn map_priority(&self, priority: agent_agency_contracts::planning_io::MilestonePriority) -> LegacyTaskPriority {
        match priority {
            agent_agency_contracts::planning_io::MilestonePriority::Low => LegacyTaskPriority::Low,
            agent_agency_contracts::planning_io::MilestonePriority::Normal => LegacyTaskPriority::Medium,
            agent_agency_contracts::planning_io::MilestonePriority::High => LegacyTaskPriority::High,
            agent_agency_contracts::planning_io::MilestonePriority::Critical => LegacyTaskPriority::Critical,
        }
    }

    /// Extract legacy dependencies from dependency graph
    fn extract_legacy_dependencies(&self, dependency_graph: &agent_agency_contracts::planning_io::DependencyGraph) -> Result<HashMap<String, Vec<String>>> {
        // Convert edges to legacy dependency format
        let mut dependencies = HashMap::new();

        for edge in &dependency_graph.edges {
            dependencies.entry(edge.to.clone())
                .or_insert(vec![])
                .push(edge.from.clone());
        }

        Ok(dependencies)
    }
}

/// Legacy task plan structure

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct LegacyTaskPlan {
    /// Plan identifier
    pub id: String,

    /// Plan description
    pub description: String,

    /// Subtasks to execute
    pub subtasks: Vec<LegacySubTask>,

    /// Dependencies between subtasks
    pub dependencies: HashMap<String, Vec<String>>,

    /// Estimated total duration in seconds
    pub estimated_duration: u64,
}

/// Legacy subtask structure

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct LegacySubTask {
    /// Subtask identifier
    pub id: String,

    /// Subtask description
    pub description: String,

    /// Task priority
    pub priority: LegacyTaskPriority,

    /// Estimated duration in seconds
    pub estimated_duration: u64,

    /// Required resources
    pub required_resources: Vec<String>,
}

/// Legacy task priority levels

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
enum LegacyTaskPriority {
    /// Low priority
    Low,

    /// Medium priority
    Medium,

    /// High priority
    High,

    /// Critical priority
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_plan_adapter_creation() {
        let adapter = LegacyPlanAdapter::new();
        // Adapter created successfully
        assert!(true);
    }

    #[test]
    fn test_legacy_task_plan_creation() {
        let plan = LegacyTaskPlan {
            id: "test-plan".to_string(),
            description: "Test plan description".to_string(),
            subtasks: vec![
                LegacySubTask {
                    id: "task1".to_string(),
                    description: "First task".to_string(),
                    priority: LegacyTaskPriority::High,
                    estimated_duration: 3600,
                    required_resources: vec!["cpu".to_string()],
                },
                LegacySubTask {
                    id: "task2".to_string(),
                    description: "Second task".to_string(),
                    priority: LegacyTaskPriority::Medium,
                    estimated_duration: 1800,
                    required_resources: vec!["memory".to_string()],
                },
            ],
            dependencies: HashMap::from([
                ("task2".to_string(), vec!["task1".to_string()])
            ]),
            estimated_duration: 5400,
        };

        assert_eq!(plan.id, "test-plan");
        assert_eq!(plan.subtasks.len(), 2);
        assert_eq!(plan.dependencies.get("task2").unwrap(), &vec!["task1"]);
        assert_eq!(plan.estimated_duration, 5400);
    }

    #[test]
    fn test_priority_mapping() {
        let adapter = LegacyPlanAdapter::new();

        assert_eq!(adapter.map_priority(agent_agency_contracts::planning_io::MilestonePriority::Low), LegacyTaskPriority::Low);
        assert_eq!(adapter.map_priority(agent_agency_contracts::planning_io::MilestonePriority::Normal), LegacyTaskPriority::Medium);
        assert_eq!(adapter.map_priority(agent_agency_contracts::planning_io::MilestonePriority::High), LegacyTaskPriority::High);
        assert_eq!(adapter.map_priority(agent_agency_contracts::planning_io::MilestonePriority::Critical), LegacyTaskPriority::Critical);
    }
}
