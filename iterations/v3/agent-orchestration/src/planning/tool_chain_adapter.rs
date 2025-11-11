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
        let constraints = system_federated_ml::tool_chain_planner::PlanningConstraints {
            max_chain_length: 10,
            max_parallel_branches: 3,
            required_reliability: 0.8,
            max_cost_cents: context.cost_budget_cents.unwrap_or(1000),
            timeout_ms: context.time_budget_ms.unwrap_or(30000),
        };

        // Plan the tool chain using the real planner
        let tool_chain = self.planner.plan_chain(&planning_context, &constraints).await
            .map_err(|e| agent_agency_contracts::ContractError::ServiceUnavailable {
                service: "tool-chain".to_string()
            })?;

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
                issues.push(format!("Duplicate tool '{}' found at position {}", tool_id, idx));
                score -= 0.1;
            } else {
                seen_tools.insert(tool_id.clone());
            }
        }

        // 2. Validate dependencies
        debug!("Validating dependencies for {} tools", plan.tool_sequence.len());

        // Check that all dependencies reference existing tools
        let tool_set: HashSet<String> = plan.tool_sequence.iter().cloned().collect();
        for (tool_id, deps) in &plan.dependencies {
            // Check if the tool itself exists in the sequence
            if !tool_set.contains(tool_id) {
                warnings.push(format!("Dependency map references tool '{}' that is not in tool sequence", tool_id));
                score -= 0.05;
            }

            // Check if all dependencies exist
            for dep in deps {
                if !tool_set.contains(dep) {
                    issues.push(format!("Tool '{}' depends on '{}' which is not in tool sequence", tool_id, dep));
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
        let tool_positions: HashMap<String, usize> = plan.tool_sequence.iter()
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
        if plan.tool_sequence.len() > 3 && !plan.tool_sequence.iter().any(|t| t.contains("error") || t.contains("validate") || t.contains("check")) {
            suggestions.push("Consider adding error handling or validation steps for longer tool chains".to_string());
        }

        // 7. Validate risk assessment consistency
        if plan.risk_assessment.confidence_score < 0.5 && plan.tool_sequence.len() > 5 {
            warnings.push("Low confidence score for complex tool chain - consider simplifying".to_string());
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
                ("dependency_count".to_string(), plan.dependencies.len().into()),
                ("estimated_duration_ms".to_string(), plan.estimated_duration_ms.into()),
                ("estimated_cost_cents".to_string(), plan.estimated_cost_cents.into()),
            ]),
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
                if let Some(cycle) = dfs(tool, dependencies, &mut visited, &mut rec_stack, &mut path) {
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
        // TODO: Implement tool chain optimization based on criteria
        //       Currently returns plan as-is; should apply optimizations based on criteria (resource usage, parallelization, etc.).
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Optimizations are applied based on criteria
        // - Resource usage is optimized
        // - Parallelization opportunities are identified
        // - Optimized plan improves performance
        //
        // DEPENDENCIES:
        // - Optimization algorithms (Required)
        // - Resource analysis utilities (Required)
        // - Parallelization analysis utilities (Required)
        //
        // ESTIMATED EFFORT: 5-6 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (optimization feature)
        // - Change Budget: ~120 LOC
        // - Reviewer Requirements: Optimization algorithms expertise
        warn!("optimize_tool_chain not fully implemented - returning original plan"); // Temporary: return as-is until optimization is implemented
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
        // TODO: Traverse DAG topologically to determine execution order
        //       Currently returns simple sequence; should traverse DAG topologically to preserve execution order requirements.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - DAG is traversed topologically
        // - Execution order requirements are preserved
        // - Dependencies are respected
        // - Traversal handles cycles correctly
        //
        // DEPENDENCIES:
        // - Graph algorithms library (Required)
        // - Topological sort utilities (Required)
        // - DAG structure (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (graph traversal feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Graph algorithms expertise
        tool_chain.nodes.iter() // Temporary: simple sequence until topological traversal is implemented
            .map(|node| node.tool_id.clone())
            .collect()
    }

    /// Extract dependencies from ToolChain DAG
    fn extract_dependencies(&self, tool_chain: &system_federated_ml::tool_chain_planner::ToolChain) -> std::collections::HashMap<String, Vec<String>> {
        // TODO: Extract dependencies from DAG edges
        //       Currently returns empty dependencies; should extract dependencies from DAG edges to build dependency map.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Dependencies are extracted from DAG edges
        // - Dependency map is accurate
        // - Cycles are detected if present
        // - Dependency structure is validated
        //
        // DEPENDENCIES:
        // - DAG structure with edges (Required)
        // - Dependency extraction utilities (Required)
        // - Cycle detection utilities (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (dependency extraction feature)
        // - Change Budget: ~80 LOC
        // - Reviewer Requirements: Graph algorithms expertise
        std::collections::HashMap::new() // Temporary: empty until DAG edge extraction is implemented
    }
}
