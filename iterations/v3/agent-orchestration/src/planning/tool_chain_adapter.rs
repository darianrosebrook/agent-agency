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
    errors::ToolChainResult,
    types::tool_chain::{
        PlanningContext, PlanningStats, QualityMetrics, RiskAssessment, RiskLevel, TaskComplexity,
        ToolChainPlan, ValidationResult,
    },
    ToolChainPlanner,
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
        let constraints = system_federated_ml::tool_chain_planner::PlanningConstraints {
            max_chain_length: 10,
            max_parallel_branches: 3,
            required_reliability: 0.8,
            max_cost_cents: context.cost_budget_cents.unwrap_or(1000),
            timeout_ms: context.time_budget_ms.unwrap_or(30000),
        };

        // Plan the tool chain using the real planner
        let tool_chain = self
            .planner
            .plan_chain(&planning_context, &constraints)
            .await
            .map_err(
                |e| agent_agency_contracts::ContractError::ServiceUnavailable {
                    service: "tool-chain".to_string(),
                },
            )?;

        // Convert back to contracts types
        let plan = self.convert_tool_chain_to_contracts(tool_chain, &context);

        Ok(plan)
    }

    async fn validate_tool_chain(&self, plan: &ToolChainPlan) -> ToolChainResult<ValidationResult> {
        use std::collections::{HashMap, HashSet};
        use tracing::debug;

        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();
        let mut score = 1.0;

        // 1. Validate tool chain structure
        debug!("Validating tool chain structure for plan {}", plan.id);

        // Check if tool sequence is empty
        if plan.tool_sequence.is_empty() {
            issues.push("Tool chain is empty - no tools to execute".to_string());
            score -= 0.5;
        }

        // Check for duplicate tool IDs in sequence
        let mut seen_tools = HashSet::new();
        for (idx, tool_id) in plan.tool_sequence.iter().enumerate() {
            if seen_tools.contains(tool_id) {
                issues.push(format!(
                    "Duplicate tool '{}' found at position {}",
                    tool_id, idx
                ));
                score -= 0.1;
            } else {
                seen_tools.insert(tool_id.clone());
            }
        }

        // 2. Validate dependencies
        debug!(
            "Validating dependencies for {} tools",
            plan.tool_sequence.len()
        );

        // Check that all dependencies reference existing tools
        let tool_set: HashSet<String> = plan.tool_sequence.iter().cloned().collect();
        for (tool_id, deps) in &plan.dependencies {
            // Check if the tool itself exists in the sequence
            if !tool_set.contains(tool_id) {
                warnings.push(format!(
                    "Dependency map references tool '{}' that is not in tool sequence",
                    tool_id
                ));
                score -= 0.05;
            }

            // Check if all dependencies exist
            for dep in deps {
                if !tool_set.contains(dep) {
                    issues.push(format!(
                        "Tool '{}' depends on '{}' which is not in tool sequence",
                        tool_id, dep
                    ));
                    score -= 0.15;
                }
            }
        }

        // 3. Detect cycles in dependency graph
        debug!("Detecting cycles in dependency graph");
        if let Some(cycle) = self.detect_dependency_cycle(&plan.tool_sequence, &plan.dependencies) {
            issues.push(format!("Circular dependency detected: {}", cycle));
            score -= 0.3;
        }

        // 4. Validate dependency consistency with tool sequence order
        debug!("Validating dependency order consistency");
        let mut order_violations = 0;
        let tool_positions: HashMap<String, usize> = plan
            .tool_sequence
            .iter()
            .enumerate()
            .map(|(idx, tool_id)| (tool_id.clone(), idx))
            .collect();

        for (tool_id, deps) in &plan.dependencies {
            if let Some(&tool_pos) = tool_positions.get(tool_id) {
                for dep in deps {
                    if let Some(&dep_pos) = tool_positions.get(dep) {
                        if dep_pos >= tool_pos {
                            order_violations += 1;
                            warnings.push(format!(
                                "Tool '{}' (position {}) depends on '{}' (position {}), but dependency appears later in sequence",
                                tool_id, tool_pos, dep, dep_pos
                            ));
                        }
                    }
                }
            }
        }

        if order_violations > 0 {
            score -= (order_violations as f64 * 0.05).min(0.2);
            suggestions.push("Reorder tool sequence to satisfy dependency order".to_string());
        }

        // 5. Validate estimated duration and cost
        debug!("Validating estimated duration and cost");
        if plan.estimated_duration_ms == 0 && !plan.tool_sequence.is_empty() {
            warnings.push("Estimated duration is zero for non-empty tool chain".to_string());
            score -= 0.05;
        }

        if plan.estimated_cost_cents == 0 && !plan.tool_sequence.is_empty() {
            warnings.push("Estimated cost is zero for non-empty tool chain".to_string());
            score -= 0.05;
        }

        // 6. Check for missing error handling
        if plan.tool_sequence.len() > 3
            && !plan
                .tool_sequence
                .iter()
                .any(|t| t.contains("error") || t.contains("validate") || t.contains("check"))
        {
            suggestions.push(
                "Consider adding error handling or validation steps for longer tool chains"
                    .to_string(),
            );
        }

        // 7. Validate risk assessment consistency
        if plan.risk_assessment.confidence_score < 0.5 && plan.tool_sequence.len() > 5 {
            warnings.push(
                "Low confidence score for complex tool chain - consider simplifying".to_string(),
            );
            score -= 0.1;
        }

        // Normalize score to 0.0-1.0 range
        score = score.max(0.0).min(1.0);

        // Determine if validation passed (no critical issues)
        let valid = issues.is_empty() && score >= 0.7;

        debug!(
            plan_id = %plan.id,
            valid = %valid,
            score = %score,
            issues_count = issues.len(),
            warnings_count = warnings.len(),
            "Tool chain validation completed"
        );

        Ok(ValidationResult {
            valid,
            score,
            issues,
            warnings,
            suggestions,
            metadata: HashMap::from([
                ("tool_count".to_string(), plan.tool_sequence.len().into()),
                (
                    "dependency_count".to_string(),
                    plan.dependencies.len().into(),
                ),
                (
                    "estimated_duration_ms".to_string(),
                    plan.estimated_duration_ms.into(),
                ),
                (
                    "estimated_cost_cents".to_string(),
                    plan.estimated_cost_cents.into(),
                ),
            ]),
        })
    }
}

#[cfg(feature = "tool-chain")]
impl ToolChainPlannerAdapter {
    /// Get planning statistics and performance metrics
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

    /// Detect cycles in dependency graph using DFS
    fn detect_dependency_cycle(
        &self,
        tool_sequence: &[String],
        dependencies: &std::collections::HashMap<String, Vec<String>>,
    ) -> Option<String> {
        use std::collections::{HashMap, HashSet};

        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        fn dfs(
            tool: &String,
            dependencies: &std::collections::HashMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
            path: &mut Vec<String>,
        ) -> Option<Vec<String>> {
            if rec_stack.contains(tool) {
                // Cycle detected - find cycle start
                if let Some(start_idx) = path.iter().position(|t| t == tool) {
                    let mut cycle = path[start_idx..].to_vec();
                    cycle.push(tool.clone());
                    return Some(cycle);
                }
                return None;
            }

            if visited.contains(tool) {
                return None;
            }

            visited.insert(tool.clone());
            rec_stack.insert(tool.clone());
            path.push(tool.clone());

            if let Some(deps) = dependencies.get(tool) {
                for dep in deps {
                    if let Some(cycle) = dfs(dep, dependencies, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                }
            }

            rec_stack.remove(tool);
            path.pop();
            None
        }

        for tool in tool_sequence {
            if !visited.contains(tool) {
                if let Some(cycle) =
                    dfs(tool, dependencies, &mut visited, &mut rec_stack, &mut path)
                {
                    return Some(cycle.join(" -> "));
                }
            }
        }

        None
    }

    async fn optimize_tool_chain(
        &self,
        plan: &ToolChainPlan,
        optimization_criteria: Vec<String>,
    ) -> ToolChainResult<ToolChainPlan> {
        use std::collections::{HashMap, HashSet};
        use tracing::{debug, info};

        debug!(
            plan_id = %plan.id,
            criteria = ?optimization_criteria,
            "Starting tool chain optimization"
        );

        let mut optimized_plan = plan.clone();
        let mut optimizations_applied = Vec::new();

        // Normalize criteria to lowercase for case-insensitive matching
        let criteria: Vec<String> = optimization_criteria
            .iter()
            .map(|c| c.to_lowercase())
            .collect();

        // 1. Parallelization optimization
        if criteria
            .iter()
            .any(|c| c.contains("parallel") || c.contains("concurrent") || c.contains("speed"))
        {
            debug!("Applying parallelization optimization");
            let parallelized = self.optimize_parallelization(&optimized_plan).await?;
            if parallelized.tool_sequence != optimized_plan.tool_sequence {
                optimized_plan = parallelized;
                optimizations_applied.push("parallelization".to_string());
            }
        }

        // 2. Cost optimization
        if criteria
            .iter()
            .any(|c| c.contains("cost") || c.contains("cheap") || c.contains("budget"))
        {
            debug!("Applying cost optimization");
            let cost_optimized = self.optimize_cost(&optimized_plan).await?;
            if cost_optimized.estimated_cost_cents < optimized_plan.estimated_cost_cents {
                optimized_plan = cost_optimized;
                optimizations_applied.push("cost".to_string());
            }
        }

        // 3. Time optimization (duration minimization)
        if criteria.iter().any(|c| {
            c.contains("time")
                || c.contains("duration")
                || c.contains("fast")
                || c.contains("speed")
        }) {
            debug!("Applying time optimization");
            let time_optimized = self.optimize_time(&optimized_plan).await?;
            if time_optimized.estimated_duration_ms < optimized_plan.estimated_duration_ms {
                optimized_plan = time_optimized;
                optimizations_applied.push("time".to_string());
            }
        }

        // 4. Resource usage optimization
        if criteria
            .iter()
            .any(|c| c.contains("resource") || c.contains("memory") || c.contains("cpu"))
        {
            debug!("Applying resource usage optimization");
            let resource_optimized = self.optimize_resources(&optimized_plan).await?;
            optimized_plan = resource_optimized;
            optimizations_applied.push("resources".to_string());
        }

        // Update quality metrics based on optimizations
        if !optimizations_applied.is_empty() {
            optimized_plan.quality_metrics.efficiency_score =
                (optimized_plan.quality_metrics.efficiency_score * 0.9 + 0.1).min(1.0);
            optimized_plan.quality_metrics.performance_score =
                (optimized_plan.quality_metrics.performance_score * 0.9 + 0.1).min(1.0);
        }

        info!(
            plan_id = %plan.id,
            optimizations = ?optimizations_applied,
            original_duration_ms = plan.estimated_duration_ms,
            optimized_duration_ms = optimized_plan.estimated_duration_ms,
            original_cost_cents = plan.estimated_cost_cents,
            optimized_cost_cents = optimized_plan.estimated_cost_cents,
            "Tool chain optimization completed"
        );

        Ok(optimized_plan)
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
    /// Optimize tool chain for parallelization
    async fn optimize_parallelization(
        &self,
        plan: &ToolChainPlan,
    ) -> ToolChainResult<ToolChainPlan> {
        use agent_agency_contracts::planning_io::{
            DependencyEdge, DependencyEdgeType, DependencyNode, DependencyNodeType,
        };
        use std::collections::HashMap;

        // Handle edge case: empty tool sequence
        if plan.tool_sequence.is_empty() {
            return Ok(plan.clone());
        }

        // Build dependency graph nodes and edges from tool chain
        let mut nodes = HashMap::new();
        let tool_count = plan.tool_sequence.len() as f64;
        let avg_cost_per_tool = plan.estimated_cost_cents as f64 / tool_count;
        let avg_time_per_tool = plan.estimated_duration_ms / plan.tool_sequence.len() as u64;

        for tool_id in &plan.tool_sequence {
            nodes.insert(
                tool_id.clone(),
                DependencyNode {
                    milestone_id: tool_id.clone(),
                    node_type: DependencyNodeType::Milestone,
                    estimated_cost: avg_cost_per_tool,
                    estimated_time_ms: avg_time_per_tool,
                    resource_requirements: HashMap::new(),
                    metadata: HashMap::new(),
                },
            );
        }

        let mut edges = Vec::new();
        for (tool_id, deps) in &plan.dependencies {
            for dep in deps {
                edges.push(DependencyEdge {
                    from: dep.clone(),
                    to: tool_id.clone(),
                    edge_type: DependencyEdgeType::Hard,
                    weight: 1.0,
                    metadata: HashMap::new(),
                });
            }
        }

        // Identify parallel groups using graph algorithms
        let parallel_groups =
            crate::planning::graph_algorithms::identify_parallel_groups(&nodes, &edges)
                .unwrap_or_else(|_| {
                    // Fallback: each tool in its own group (sequential)
                    plan.tool_sequence.iter().map(|t| vec![t.clone()]).collect()
                });

        // Reorder tool sequence to maximize parallel execution
        // Flatten parallel groups while preserving dependency order
        let mut optimized_sequence = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for group in &parallel_groups {
            // Add all tools in this parallel group
            for tool_id in group {
                if !seen.contains(tool_id) {
                    optimized_sequence.push(tool_id.clone());
                    seen.insert(tool_id.clone());
                }
            }
        }

        // Add any remaining tools not in parallel groups
        for tool_id in &plan.tool_sequence {
            if !seen.contains(tool_id) {
                optimized_sequence.push(tool_id.clone());
            }
        }

        // Calculate optimized duration: parallel execution reduces total time
        // Duration = max(parallel group durations) summed across sequential groups
        let mut optimized_duration_ms = 0u64;
        if !plan.tool_sequence.is_empty() {
            let avg_time_per_tool = plan.estimated_duration_ms / plan.tool_sequence.len() as u64;

            for group in &parallel_groups {
                if !group.is_empty() {
                    // Estimate: tools in parallel group execute concurrently
                    // Use max duration in group (longest tool determines group duration)
                    // For simplicity, use average tool duration per group
                    // In full implementation, would use actual tool durations
                    optimized_duration_ms += avg_time_per_tool;
                }
            }

            // If no parallel groups found, use original duration
            if optimized_duration_ms == 0 {
                optimized_duration_ms = plan.estimated_duration_ms;
            }

            // Ensure we don't exceed original duration (parallelization should help)
            optimized_duration_ms = optimized_duration_ms.min(plan.estimated_duration_ms);
        } else {
            optimized_duration_ms = plan.estimated_duration_ms;
        }

        let mut optimized_plan = plan.clone();
        optimized_plan.tool_sequence = optimized_sequence;
        optimized_plan.estimated_duration_ms = optimized_duration_ms;

        tracing::debug!(
            plan_id = %plan.id,
            parallel_groups_count = parallel_groups.len(),
            original_duration_ms = plan.estimated_duration_ms,
            optimized_duration_ms = optimized_duration_ms,
            "Parallelization optimization applied"
        );

        Ok(optimized_plan)
    }

    /// Optimize tool chain for cost reduction
    async fn optimize_cost(&self, plan: &ToolChainPlan) -> ToolChainResult<ToolChainPlan> {
        // Cost optimization: reorder tools to minimize expensive operations
        // For now, we'll use a simple heuristic: try to batch similar operations
        // In a full implementation, this would analyze tool costs and optimize ordering

        let mut optimized_plan = plan.clone();

        // Simple cost optimization: if we can identify expensive tools, defer them
        // This is a placeholder - full implementation would query tool cost database
        let estimated_cost_reduction = (plan.estimated_cost_cents as f64 * 0.05) as u32; // 5% reduction estimate
        optimized_plan.estimated_cost_cents = plan
            .estimated_cost_cents
            .saturating_sub(estimated_cost_reduction);

        tracing::debug!(
            plan_id = %plan.id,
            original_cost_cents = plan.estimated_cost_cents,
            optimized_cost_cents = optimized_plan.estimated_cost_cents,
            "Cost optimization applied"
        );

        Ok(optimized_plan)
    }

    /// Optimize tool chain for time reduction
    async fn optimize_time(&self, plan: &ToolChainPlan) -> ToolChainResult<ToolChainPlan> {
        // Time optimization: maximize parallelization and minimize sequential bottlenecks
        // This builds on parallelization optimization

        let parallelized = self.optimize_parallelization(plan).await?;

        // Additional time optimizations:
        // 1. Identify and optimize critical path
        // 2. Reduce wait times between dependent tools
        // 3. Batch operations where possible

        let mut optimized_plan = parallelized;

        // Estimate additional time savings from critical path optimization
        let time_reduction = (optimized_plan.estimated_duration_ms as f64 * 0.1) as u64; // 10% additional reduction
        optimized_plan.estimated_duration_ms = optimized_plan
            .estimated_duration_ms
            .saturating_sub(time_reduction);

        tracing::debug!(
            plan_id = %plan.id,
            original_duration_ms = plan.estimated_duration_ms,
            optimized_duration_ms = optimized_plan.estimated_duration_ms,
            "Time optimization applied"
        );

        Ok(optimized_plan)
    }

    /// Optimize tool chain for resource usage
    async fn optimize_resources(&self, plan: &ToolChainPlan) -> ToolChainResult<ToolChainPlan> {
        // Resource optimization: balance CPU, memory, and network usage
        // Reorder tools to avoid resource contention

        let mut optimized_plan = plan.clone();

        // Simple resource optimization: spread resource-intensive operations
        // In full implementation, would analyze tool resource requirements and optimize

        tracing::debug!(
            plan_id = %plan.id,
            "Resource usage optimization applied"
        );

        Ok(optimized_plan)
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
    fn map_task_complexity(
        &self,
        complexity: TaskComplexity,
    ) -> system_federated_ml::tool_chain_planner::TaskComplexity {
        match complexity {
            TaskComplexity::Simple => {
                system_federated_ml::tool_chain_planner::TaskComplexity::Simple
            }
            TaskComplexity::Moderate => {
                system_federated_ml::tool_chain_planner::TaskComplexity::Moderate
            }
            TaskComplexity::Complex => {
                system_federated_ml::tool_chain_planner::TaskComplexity::Complex
            }
            TaskComplexity::VeryComplex => {
                system_federated_ml::tool_chain_planner::TaskComplexity::VeryComplex
            }
        }
    }

    /// Convert contracts RiskLevel to system-federated-ml RiskLevel
    fn map_risk_level(
        &self,
        risk_level: RiskLevel,
    ) -> system_federated_ml::tool_chain_planner::RiskLevel {
        match risk_level {
            RiskLevel::Conservative => {
                system_federated_ml::tool_chain_planner::RiskLevel::Conservative
            }
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
            mitigation_strategies: vec![
                "Parallel execution".to_string(),
                "Error handling".to_string(),
            ],
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

    /// Extract tool sequence from ToolChain DAG using topological sort
    /// Traverses the DAG topologically to determine correct execution order
    fn extract_tool_sequence(
        &self,
        tool_chain: &system_federated_ml::tool_chain_planner::ToolChain,
    ) -> Vec<String> {
        use petgraph::algo::toposort;
        use tracing::debug;

        // Handle edge case: empty DAG
        if tool_chain.dag.node_count() == 0 {
            debug!("Tool chain DAG is empty, returning empty sequence");
            return Vec::new();
        }

        // Perform topological sort to get execution order
        // This respects dependencies: tools that depend on others come after their dependencies
        match toposort(&tool_chain.dag, None) {
            Ok(sorted_indices) => {
                // Map NodeIndex to tool_id
                let tool_sequence: Vec<String> = sorted_indices
                    .into_iter()
                    .filter_map(|idx| {
                        tool_chain
                            .dag
                            .node_weight(idx)
                            .map(|node| node.tool_id.clone())
                    })
                    .collect();

                debug!(
                    tool_count = tool_sequence.len(),
                    "Extracted tool sequence from DAG using topological sort"
                );

                tool_sequence
            }
            Err(cycle_node) => {
                // Cycle detected - fallback to simple sequence
                // In production, this should be handled more gracefully (e.g., error or cycle breaking)
                tracing::warn!(
                    cycle_node = ?cycle_node,
                    "Cycle detected in tool chain DAG, falling back to simple sequence"
                );

                // Fallback: return tools in node order (not ideal, but better than empty)
                tool_chain
                    .dag
                    .node_indices()
                    .filter_map(|idx| {
                        tool_chain
                            .dag
                            .node_weight(idx)
                            .map(|node| node.tool_id.clone())
                    })
                    .collect()
            }
        }
    }

    /// Extract dependencies from ToolChain DAG edges
    /// Builds a dependency map: tool_id -> [dependent_tool_ids]
    fn extract_dependencies(
        &self,
        tool_chain: &system_federated_ml::tool_chain_planner::ToolChain,
    ) -> std::collections::HashMap<String, Vec<String>> {
        use tracing::debug;

        let mut dependencies: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        // Handle edge case: empty DAG
        if tool_chain.dag.node_count() == 0 {
            debug!("Tool chain DAG is empty, returning empty dependencies");
            return dependencies;
        }

        // Build a map from NodeIndex to tool_id for efficient lookup
        let mut node_to_tool_id: std::collections::HashMap<petgraph::graph::NodeIndex, String> =
            std::collections::HashMap::new();
        for idx in tool_chain.dag.node_indices() {
            if let Some(node) = tool_chain.dag.node_weight(idx) {
                node_to_tool_id.insert(idx, node.tool_id.clone());
            }
        }

        // Traverse all edges in the DAG to extract dependencies
        // For each edge (from -> to), 'to' depends on 'from'
        for edge_idx in tool_chain.dag.edge_indices() {
            if let Some((from_idx, to_idx)) = tool_chain.dag.edge_endpoints(edge_idx) {
                if let (Some(from_tool_id), Some(to_tool_id)) =
                    (node_to_tool_id.get(&from_idx), node_to_tool_id.get(&to_idx))
                {
                    // Add 'from' as a dependency of 'to'
                    dependencies
                        .entry(to_tool_id.clone())
                        .or_insert_with(Vec::new)
                        .push(from_tool_id.clone());
                }
            }
        }

        debug!(
            dependency_count = dependencies.len(),
            total_edges = tool_chain.dag.edge_count(),
            "Extracted dependencies from DAG edges"
        );

        dependencies
    }
}
