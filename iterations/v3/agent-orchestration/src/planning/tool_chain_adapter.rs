//! Tool Chain Planner Adapter
//!
//! Adapts the real system-federated-ml tool chain planner to implement the contracts::ToolChainPlanner trait.
//! This adapter enables dependency injection and breaks the direct dependency from orchestration to tool chain.
//!
//! @author @darianrosebrook

#[cfg(feature = "tool-chain")]
use async_trait::async_trait;
#[cfg(feature = "tool-chain")]
use std::sync::Arc;

#[cfg(feature = "tool-chain")]
use agent_agency_contracts::{
    ToolChainPlanner,
    types::tool_chain::{
        ToolChainPlan, PlanningContext, ValidationResult, PlanningStats,
        TaskComplexity, RiskLevel, RiskAssessment, QualityMetrics,
    },
    errors::ToolChainResult,
};

/// Adapter that wraps system-federated-ml::ToolChainPlanner to implement contracts::ToolChainPlanner
#[cfg(feature = "tool-chain")]
pub struct ToolChainPlannerAdapter {
    /// The underlying tool chain planner implementation
    planner: Arc<system_federated_ml::tool_chain_planner::ToolChainPlanner>,
}

#[cfg(feature = "tool-chain")]
impl ToolChainPlannerAdapter {
    /// Create a new tool chain planner adapter
    pub fn new(planner: Arc<system_federated_ml::tool_chain_planner::ToolChainPlanner>) -> Self {
        Self { planner }
    }
}

#[cfg(feature = "tool-chain")]
#[async_trait]
impl ToolChainPlanner for ToolChainPlannerAdapter {
    async fn plan_tool_chain(&self, context: PlanningContext) -> ToolChainResult<ToolChainPlan> {
        // Convert contracts PlanningContext to system-federated-ml types
        let planning_context = system_federated_ml::tool_chain_planner::PlanningContext {
            task_description: context.task_description.clone(),
            task_type: context.task_type.clone(),
            complexity: self.map_task_complexity(context.complexity),
            required_capabilities: context.required_capabilities.clone(),
            time_budget_ms: context.time_budget_ms,
            cost_budget_cents: context.cost_budget_cents,
            risk_tolerance: self.map_risk_level(context.risk_tolerance),
        };

        // TODO: Make planning constraints configurable
        // - [ ] Load constraints from configuration or context
        // - [ ] Support dynamic constraint adjustment based on task requirements
        // - [ ] Allow per-task constraint overrides
        // - [ ] Validate constraints against system capabilities
        // - [ ] Add unit tests with various constraint configurations
        // - [ ] Add integration tests with real constraint-based planning
        // Use default constraints for now - in a full implementation, these would be configurable
        let constraints = system_federated_ml::tool_chain_planner::PlanningConstraints {
            max_chain_length: 10,
            max_parallel_branches: 3,
            required_reliability: 0.8,
            max_cost_cents: context.cost_budget_cents.unwrap_or(1000),
            timeout_ms: context.time_budget_ms.unwrap_or(30000),
        };

        // Plan the tool chain using the real planner
        let tool_chain = self.planner.plan_chain(&planning_context, &constraints).await
            .map_err(|e| agent_agency_contracts::errors::ContractError::ServiceUnavailable {
                service: "tool-chain".to_string()
            })?;

        // Convert back to contracts types
        let plan = self.convert_tool_chain_to_contracts(tool_chain, &context);

        Ok(plan)
    }

    async fn validate_tool_chain(&self, plan: &ToolChainPlan) -> ToolChainResult<ValidationResult> {
        // TODO: Implement comprehensive tool chain validation
        // - [ ] Validate tool chain structure and dependencies
        // - [ ] Check for circular dependencies
        // - [ ] Validate tool availability and capabilities
        // - [ ] Verify resource constraints are met
        // - [ ] Check for compatibility between chain steps
        // - [ ] Add unit tests with various tool chain structures
        // - [ ] Add integration tests with real tool chain validation
        // For now, return a basic validation result
        // In a full implementation, this would validate the tool chain structure
        let validation_result = ValidationResult {
            is_valid: plan.tool_sequence.len() > 0,
            score: 0.8, // Basic score
            issues: if plan.tool_sequence.is_empty() {
                vec!["Tool chain is empty".to_string()]
            } else {
                vec![]
            },
            warnings: vec![],
            recommendations: vec!["Consider adding error handling steps".to_string()],
        };

        Ok(validation_result)
    }

    async fn optimize_tool_chain(
        &self,
        plan: &ToolChainPlan,
        optimization_criteria: Vec<String>,
    ) -> ToolChainResult<ToolChainPlan> {
        // TODO: Implement tool chain optimization
        // - [ ] Parse optimization criteria (performance, cost, reliability)
        // - [ ] Apply optimization algorithms based on criteria
        // - [ ] Reorder tool chain steps for better performance
        // - [ ] Merge or eliminate redundant steps
        // - [ ] Optimize resource usage and parallelization
        // - [ ] Add unit tests with various optimization criteria
        // - [ ] Add integration tests with real tool chain optimization
        // For now, return the plan as-is
        // In a full implementation, this would apply optimizations based on criteria
        warn!("optimize_tool_chain not fully implemented - returning original plan");
        Ok(plan.clone())
    }

    async fn get_planning_stats(&self) -> ToolChainResult<PlanningStats> {
        // Return basic stats - in a full implementation, this would query the planner
        Ok(PlanningStats {
            total_plans_generated: 0,
            average_planning_time_ms: 0.0,
            plan_success_rate: 0.0,
            average_optimization_improvement: 0.0,
            cache_hit_rate: 0.0,
            last_planning_time: None,
        })
    }
}

#[cfg(feature = "tool-chain")]
impl ToolChainPlannerAdapter {
    /// Convert contracts TaskComplexity to system-federated-ml TaskComplexity
    fn map_task_complexity(&self, complexity: TaskComplexity) -> system_federated_ml::tool_chain_planner::TaskComplexity {
        match complexity {
            TaskComplexity::Simple => system_federated_ml::tool_chain_planner::TaskComplexity::Simple,
            TaskComplexity::Moderate => system_federated_ml::tool_chain_planner::TaskComplexity::Moderate,
            TaskComplexity::Complex => system_federated_ml::tool_chain_planner::TaskComplexity::Complex,
            TaskComplexity::VeryComplex => system_federated_ml::tool_chain_planner::TaskComplexity::VeryComplex,
        }
    }

    /// Convert contracts RiskLevel to system-federated-ml RiskLevel
    fn map_risk_level(&self, risk_level: RiskLevel) -> system_federated_ml::tool_chain_planner::RiskLevel {
        match risk_level {
            RiskLevel::Conservative => system_federated_ml::tool_chain_planner::RiskLevel::Conservative,
            RiskLevel::Balanced => system_federated_ml::tool_chain_planner::RiskLevel::Balanced,
            RiskLevel::Aggressive => system_federated_ml::tool_chain_planner::RiskLevel::Aggressive,
        }
    }

    /// Convert system-federated-ml ToolChain to contracts ToolChainPlan
    fn convert_tool_chain_to_contracts(
        &self,
        tool_chain: system_federated_ml::tool_chain_planner::ToolChain,
        context: &PlanningContext,
    ) -> ToolChainPlan {
        // Extract tool sequence from the DAG
        let tool_sequence = self.extract_tool_sequence(&tool_chain);

        // Build dependencies map
        let dependencies = self.extract_dependencies(&tool_chain);

        // Create risk assessment
        let risk_assessment = RiskAssessment {
            risk_level: context.risk_tolerance.clone(),
            risk_factors: vec!["Tool chain complexity".to_string()],
            mitigation_strategies: vec!["Parallel execution".to_string(), "Error handling".to_string()],
            confidence_score: 0.85,
        };

        // Create quality metrics
        let quality_metrics = QualityMetrics {
            efficiency_score: 0.8,
            reliability_score: 0.9,
            cost_effectiveness_score: 0.75,
            performance_score: 0.85,
        };

        ToolChainPlan {
            id: uuid::Uuid::new_v4().to_string(),
            description: format!("Tool chain for: {}", context.task_description),
            tool_sequence,
            dependencies,
            estimated_duration_ms: context.time_budget_ms.unwrap_or(30000),
            estimated_cost_cents: context.cost_budget_cents.unwrap_or(100),
            risk_assessment,
            quality_metrics,
        }
    }

    /// Extract tool sequence from ToolChain DAG
    fn extract_tool_sequence(&self, tool_chain: &system_federated_ml::tool_chain_planner::ToolChain) -> Vec<String> {
        // TODO: Implement proper DAG topological traversal
        // - [ ] Implement topological sort algorithm for DAG
        // - [ ] Extract tool sequence respecting dependencies
        // - [ ] Handle cycles and circular dependencies
        // - [ ] Preserve execution order requirements
        // - [ ] Add unit tests with various DAG structures
        // - [ ] Add integration tests with real tool chain DAGs
        // For now, return a simple sequence
        // In a full implementation, this would traverse the DAG topologically
        tool_chain.nodes.iter()
            .map(|node| node.tool_id.clone())
            .collect()
    }

    /// Extract dependencies from ToolChain DAG
    fn extract_dependencies(&self, tool_chain: &system_federated_ml::tool_chain_planner::ToolChain) -> std::collections::HashMap<String, Vec<String>> {
        // TODO: Extract dependencies from DAG edges
        // - [ ] Parse DAG edge structure to extract dependencies
        // - [ ] Map tool IDs to their dependency lists
        // - [ ] Handle transitive dependencies if needed
        // - [ ] Validate dependency structure for cycles
        // - [ ] Add unit tests with various dependency patterns
        // - [ ] Add integration tests with real tool chain dependencies
        // For now, return empty dependencies
        // In a full implementation, this would extract from the DAG edges
        std::collections::HashMap::new()
    }
}
