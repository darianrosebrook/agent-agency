//! Plan Generator - AI-Assisted Execution Planning
//!
//! Generates execution plans from working specs using AI assistance
//! and bridges to existing planning infrastructure.
//!
//! @author @darianrosebrook

use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use chrono::Utc;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use agent_agency_contracts::{
    *,
    planning::PlanningEngine,
    planning_io::{ExecutionPlan as ContractExecutionPlan, Milestone as ContractMilestone, PlanState, MilestoneState},
    WorkingSpec,
};

use crate::planning::{
    plan_types::{ExecutionPlan, ExecutionContext, PlanGenerationContext, PlanningConstraints},
    caws_integration::CawsPlanBridge,
    tool_chain_bridge::ToolChainBridge,
    legacy_plan_adapter::LegacyPlanAdapter,
};

/// AI-assisted plan generator
pub struct PlanGenerator {
    /// CAWS integration bridge
    caws_bridge: CawsPlanBridge,

    /// Tool chain bridge for existing planning
    tool_chain_bridge: Option<ToolChainBridge>,

    /// Legacy plan adapter for backward compatibility
    legacy_adapter: Option<LegacyPlanAdapter>,

    /// Planning constraints
    constraints: PlanningConstraints,
}

/// Plan generation strategy
#[derive(Debug, Clone)]
pub enum PlanGenerationStrategy {
    /// Pure AI-assisted generation
    AIAssisted,

    /// Bridge to existing tool chain planner
    ToolChainBridge,

    /// Use legacy planning agent
    LegacyAdapter,

    /// Hybrid approach combining strategies
    Hybrid,
}

impl PlanGenerator {
    /// Create new plan generator
    pub fn new(
        constraints: PlanningConstraints,
        tool_chain_bridge: Option<ToolChainBridge>,
        legacy_adapter: Option<LegacyPlanAdapter>,
    ) -> Self {
        Self {
            caws_bridge: CawsPlanBridge::new(),
            tool_chain_bridge,
            legacy_adapter,
            constraints,
        }
    }

    /// Generate execution plan from context
    pub async fn generate(&self, context: &PlanGenerationContext) -> Result<ExecutionPlan> {
        // Get working spec and task descriptor
        let working_spec = context.working_spec.get_working_spec().await?;
        let task_descriptor = context.task_descriptor.get_task_descriptor().await?;

        // Choose generation strategy based on available bridges and context
        let strategy = self.select_generation_strategy(&working_spec)?;

        match strategy {
            PlanGenerationStrategy::AIAssisted => {
                self.generate_ai_assisted(&working_spec, &task_descriptor, context).await
            }
            PlanGenerationStrategy::ToolChainBridge => {
                self.generate_tool_chain_bridge(&working_spec, &task_descriptor, context).await
            }
            PlanGenerationStrategy::LegacyAdapter => {
                self.generate_legacy_adapter(&working_spec, &task_descriptor, context).await
            }
            PlanGenerationStrategy::Hybrid => {
                self.generate_hybrid(&working_spec, &task_descriptor, context).await
            }
        }
    }

    /// Select appropriate generation strategy
    fn select_generation_strategy(&self, working_spec: &WorkingSpec) -> Result<PlanGenerationStrategy> {
        // Prefer tool chain bridge if available and compatible
        if self.tool_chain_bridge.is_some() && self.is_tool_chain_compatible(working_spec) {
            return Ok(PlanGenerationStrategy::ToolChainBridge);
        }

        // Fall back to legacy adapter if available
        if self.legacy_adapter.is_some() {
            return Ok(PlanGenerationStrategy::LegacyAdapter);
        }

        // Default to AI-assisted generation
        Ok(PlanGenerationStrategy::AIAssisted)
    }

    /// Check if tool chain bridge is compatible with working spec
    fn is_tool_chain_compatible(&self, working_spec: &WorkingSpec) -> bool {
        // Check if working spec has compatible acceptance criteria
        // and can be decomposed into tool chains
        working_spec.acceptance_criteria.len() > 0 &&
        working_spec.acceptance_criteria.iter().all(|criterion|
            !criterion.given.is_empty() &&
            !criterion.when.is_empty() &&
            !criterion.then.is_empty()
        )
    }

    /// Generate plan using AI assistance
    async fn generate_ai_assisted(
        &self,
        working_spec: &WorkingSpec,
        task_descriptor: &TaskDescriptor,
        context: &PlanGenerationContext,
    ) -> Result<ExecutionPlan> {
        // Use CAWS bridge to convert working spec to plan structure
        let contract_plan = self.caws_bridge.spec_to_plan(working_spec.clone())?;

        // Enhance with AI-assisted milestone decomposition
        let enhanced_plan = self.enhance_with_ai_assistance(contract_plan, task_descriptor, context).await?;

        // Wrap in orchestration plan
        self.wrap_in_orchestration_plan(enhanced_plan, context)
    }

    /// Generate plan using tool chain bridge
    async fn generate_tool_chain_bridge(
        &self,
        working_spec: &WorkingSpec,
        task_descriptor: &TaskDescriptor,
        context: &PlanGenerationContext,
    ) -> Result<ExecutionPlan> {
        if let Some(bridge) = &self.tool_chain_bridge {
            // Use tool chain bridge to generate plan
            let contract_plan = bridge.generate_from_working_spec(working_spec.clone()).await?;
            self.wrap_in_orchestration_plan(contract_plan, context)
        } else {
            Err(anyhow!("Tool chain bridge not available"))
        }
    }

    /// Generate plan using legacy adapter
    async fn generate_legacy_adapter(
        &self,
        working_spec: &WorkingSpec,
        task_descriptor: &TaskDescriptor,
        context: &PlanGenerationContext,
    ) -> Result<ExecutionPlan> {
        if let Some(adapter) = &self.legacy_adapter {
            // Use legacy adapter to generate plan
            let contract_plan = adapter.adapt_working_spec(working_spec.clone()).await?;
            self.wrap_in_orchestration_plan(contract_plan, context)
        } else {
            Err(anyhow!("Legacy adapter not available"))
        }
    }

    /// Generate hybrid plan combining multiple strategies
    async fn generate_hybrid(
        &self,
        working_spec: &WorkingSpec,
        task_descriptor: &TaskDescriptor,
        context: &PlanGenerationContext,
    ) -> Result<ExecutionPlan> {
        // Try tool chain bridge first
        if let Ok(plan) = self.generate_tool_chain_bridge(working_spec, task_descriptor, context).await {
            return Ok(plan);
        }

        // Fall back to AI-assisted generation
        self.generate_ai_assisted(working_spec, task_descriptor, context).await
    }

    /// Enhance plan with AI assistance
    async fn enhance_with_ai_assistance(
        &self,
        mut contract_plan: ContractExecutionPlan,
        task_descriptor: &TaskDescriptor,
        context: &PlanGenerationContext,
    ) -> Result<ContractExecutionPlan> {
        // Analyze task complexity and dependencies
        let complexity = self.analyze_task_complexity(task_descriptor)?;
        let dependencies = self.analyze_dependencies(&contract_plan)?;

        // Decompose into optimal milestones
        let milestones = self.decompose_into_milestones(
            &contract_plan,
            &complexity,
            &dependencies,
            context
        ).await?;

        // Build dependency graph
        let dependency_graph = self.build_dependency_graph(&milestones, &dependencies)?;

        // Optimize for parallel execution
        let optimized_milestones = self.optimize_for_parallelism(milestones, &context.available_resources)?;

        // Update plan with enhanced data
        contract_plan.milestones = optimized_milestones;
        contract_plan.dependency_graph = dependency_graph;

        Ok(contract_plan)
    }

    /// Analyze task complexity
    fn analyze_task_complexity(&self, task_descriptor: &TaskDescriptor) -> Result<TaskComplexity> {
        let mut complexity = TaskComplexity::Simple;
        let description_len = task_descriptor.description.len();
        let has_dependencies = false; // TaskDescriptor doesn't have dependencies field

        if description_len > 500 || has_dependencies {
            complexity = TaskComplexity::Moderate;
        }

        if description_len > 1000 {
            complexity = TaskComplexity::Complex;
        }

        Ok(complexity)
    }

    /// Analyze dependencies from acceptance criteria
    fn analyze_dependencies(&self, plan: &ContractExecutionPlan) -> Result<DependencyAnalysis> {
        let mut dependencies = HashMap::new();
        let mut blocking_items = HashSet::new();

        // Analyze acceptance criteria for dependencies
        for criterion in &plan.contract_plan.acceptance_criteria {
            let deps = self.extract_dependencies_from_criterion(criterion)?;
            for dep in deps {
                dependencies.entry(dep.clone()).or_insert(vec![]).push(criterion.id.clone());
                if self.is_blocking_dependency(&dep) {
                    blocking_items.insert(dep);
                }
            }
        }

        Ok(DependencyAnalysis {
            dependencies,
            blocking_items,
            dependency_graph: self.build_dependency_graph_from_analysis(&dependencies)?,
        })
    }

    /// Extract dependencies from acceptance criterion
    fn extract_dependencies_from_criterion(&self, criterion: &agent_agency_contracts::AcceptanceCriterion) -> Result<Vec<String>> {
        let mut deps = vec![];

        // Look for dependency keywords in the criterion text
        let text = format!("{} {} {}", criterion.given, criterion.when, criterion.then);

        if text.contains("after") || text.contains("requires") || text.contains("depends on") {
            // Extract dependency references (simplified - would use NLP in real implementation)
            // For now, return empty - real implementation would analyze text
        }

        Ok(deps)
    }

    /// Check if dependency is blocking
    fn is_blocking_dependency(&self, dependency: &str) -> bool {
        // Check if dependency blocks other milestones
        // Simplified logic - real implementation would analyze impact
        dependency.contains("infrastructure") ||
        dependency.contains("database") ||
        dependency.contains("security")
    }

    /// Decompose into optimal milestones
    async fn decompose_into_milestones(
        &self,
        plan: &ContractExecutionPlan,
        complexity: &TaskComplexity,
        dependencies: &DependencyAnalysis,
        context: &PlanGenerationContext,
    ) -> Result<Vec<ContractMilestone>> {
        let mut milestones = vec![];

        // Create milestones based on acceptance criteria
        for criterion in &plan.acceptance {
            let milestone = self.create_milestone_from_criterion(
                criterion,
                complexity,
                dependencies,
                context,
            ).await?;
            milestones.push(milestone);
        }

        // Add infrastructure/setup milestones if needed
        if self.needs_infrastructure_milestone(plan, complexity)? {
            milestones.insert(0, self.create_infrastructure_milestone(plan)?);
        }

        Ok(milestones)
    }

    /// Create milestone from acceptance criterion
    async fn create_milestone_from_criterion(
        &self,
        criterion: &agent_agency_contracts::AcceptanceCriterion,
        complexity: &TaskComplexity,
        dependencies: &DependencyAnalysis,
        context: &PlanGenerationContext,
    ) -> Result<ContractMilestone> {
        let milestone_id = criterion.id.clone();
        let objective = format!("{} → {} → {}", criterion.given, criterion.when, criterion.then);

        // Determine scope based on objective
        let scope = self.determine_milestone_scope(&objective, context)?;

        // Generate evidence gate based on risk tier
        let task_descriptor = context.task_descriptor.get_task_descriptor().await?;
        let evidence_gate = self.generate_evidence_gate(task_descriptor.risk_tier, complexity)?;

        // Estimate effort based on complexity and dependencies
        let estimated_effort = self.estimate_milestone_effort(complexity, &scope, dependencies)?;

        Ok(ContractMilestone {
            id: milestone_id,
            objective,
            scope,
            interfaces: vec![], // Would be populated based on analysis
            tests: vec![], // Would be populated based on requirements
            evidence_gate,
            rollback_plan: self.generate_rollback_plan(&objective),
            dependencies: dependencies.get_dependencies_for_criterion(&criterion.id),
            state: MilestoneState::Pending,
            assigned_workers: vec![],
            estimated_effort,
            priority: self.determine_priority(complexity, dependencies),
            risk_tier: task_descriptor.risk_tier,
            is_blocking: dependencies.is_blocking_criterion(&criterion.id),
            blocking_reason: dependencies.get_blocking_reason(&criterion.id),
            metrics: None,
        })
    }

    /// Determine milestone scope
    fn determine_milestone_scope(&self, objective: &str, context: &PlanGenerationContext) -> Result<MilestoneScope> {
        // Analyze objective to determine affected files and operations
        // Simplified - real implementation would use NLP and project analysis
        Ok(MilestoneScope {
            files: vec![], // Would be populated by analysis
            directories: vec![],
            will_modify: objective.contains("create") || objective.contains("modify"),
            allowed_operations: vec!["read".to_string(), "write".to_string()],
            parallelism: Some(1),
            resource_requirements: HashMap::new(),
        })
    }

    /// Generate evidence gate
    fn generate_evidence_gate(&self, risk_tier: u8, complexity: &TaskComplexity) -> Result<EvidenceGate> {
        let (min_coverage, min_mutation) = match risk_tier {
            1 => (0.90, 0.70),
            2 => (0.80, 0.50),
            _ => (0.70, 0.30),
        };

        let security_required = risk_tier == 1 || matches!(complexity, TaskComplexity::Complex);

        Ok(EvidenceGate {
            min_coverage,
            min_branch_coverage: min_coverage * 0.9,
            min_mutation_score: min_mutation,
            security_scan_required: security_required,
            performance_budget: None,
            required_artifacts: vec!["test_results".to_string(), "coverage".to_string()],
            custom_validations: vec![],
        })
    }

    /// Estimate milestone effort
    fn estimate_milestone_effort(&self, complexity: &TaskComplexity, scope: &MilestoneScope, dependencies: &DependencyAnalysis) -> Result<f64> {
        let base_effort = match complexity {
            TaskComplexity::Simple => 2.0,
            TaskComplexity::Moderate => 4.0,
            TaskComplexity::Complex => 8.0,
        };

        let scope_multiplier = scope.files.len() as f64 * 0.5 + 1.0;
        let dependency_multiplier = dependencies.dependency_count() as f64 * 0.2 + 1.0;

        Ok(base_effort * scope_multiplier * dependency_multiplier)
    }

    /// Determine milestone priority
    fn determine_priority(&self, complexity: &TaskComplexity, dependencies: &DependencyAnalysis) -> MilestonePriority {
        if dependencies.has_blocking_dependencies() {
            MilestonePriority::Critical
        } else if matches!(complexity, TaskComplexity::Complex) {
            MilestonePriority::High
        } else {
            MilestonePriority::Normal
        }
    }

    /// Build dependency graph
    fn build_dependency_graph(&self, milestones: &[ContractMilestone], dependencies: &DependencyAnalysis) -> Result<DependencyGraph> {
        let mut nodes = HashMap::new();
        let mut edges = vec![];

        // Create nodes
        for milestone in milestones {
            nodes.insert(milestone.id.clone(), DependencyNode {
                milestone_id: milestone.id.clone(),
                node_type: DependencyNodeType::Milestone,
                estimated_cost: milestone.estimated_effort,
                estimated_time_ms: (milestone.estimated_effort * 3600.0 * 1000.0) as u64,
                resource_requirements: HashMap::new(),
                metadata: HashMap::new(),
            });
        }

        // Create edges based on dependencies
        for milestone in milestones {
            for dep in &milestone.dependencies {
                edges.push(DependencyEdge {
                    from: dep.clone(),
                    to: milestone.id.clone(),
                    edge_type: if dependencies.is_blocking_dependency(dep) {
                        DependencyEdgeType::Hard
                    } else {
                        DependencyEdgeType::Soft
                    },
                    weight: 1.0,
                    metadata: HashMap::new(),
                });
            }
        }

        // Calculate critical path and parallel groups
        let critical_path = self.calculate_critical_path(&nodes, &edges)?;
        let parallel_groups = self.identify_parallel_groups(&nodes, &edges)?;

        Ok(DependencyGraph {
            nodes,
            edges,
            critical_path,
            parallel_groups,
            has_cycles: false, // Assume no cycles for now
            cycles: vec![],
        })
    }

    /// Wrap contract plan in orchestration plan
    fn wrap_in_orchestration_plan(&self, contract_plan: ContractExecutionPlan, context: &PlanGenerationContext) -> Result<ExecutionPlan> {
        Ok(ExecutionPlan {
            contract_plan,
            orchestration_meta: OrchestrationMetadata {
                orchestrator_id: "plan-generator".to_string(),
                worker_pool_id: "default-pool".to_string(),
                council_session_id: None,
                audit_correlation_id: Uuid::new_v4(),
                planning_engine: "ai-assisted".to_string(),
                planning_version: "1.0.0".to_string(),
            },
            execution_context: ExecutionContext {
                session_start: Utc::now(),
                working_directory: std::env::current_dir()?.to_string_lossy().to_string(),
                environment: std::env::vars().collect(),
                available_resources: context.resource_inventory.clone(),
                worker_assignments: HashMap::new(),
                parallel_batches: vec![],
            },
            execution_state: None,
        })
    }

    // Placeholder implementations for complex methods
    fn build_dependency_graph_from_analysis(&self, dependencies: &HashMap<String, Vec<String>>) -> Result<DependencyGraph> {
        Ok(DependencyGraph {
            nodes: HashMap::new(),
            edges: vec![],
            critical_path: vec![],
            parallel_groups: vec![],
            has_cycles: false,
            cycles: vec![],
        })
    }

    fn needs_infrastructure_milestone(&self, plan: &ContractExecutionPlan, complexity: &TaskComplexity) -> Result<bool> {
        Ok(matches!(complexity, TaskComplexity::Complex) && plan.milestones.len() > 3)
    }

    fn create_infrastructure_milestone(&self, plan: &ContractExecutionPlan) -> Result<ContractMilestone> {
        Ok(ContractMilestone {
            id: "M0".to_string(),
            objective: "Set up infrastructure and dependencies".to_string(),
            scope: MilestoneScope {
                files: vec![],
                directories: vec![],
                will_modify: false,
                allowed_operations: vec!["read".to_string()],
                parallelism: Some(1),
                resource_requirements: HashMap::new(),
            },
            interfaces: vec![],
            tests: vec![],
            quality_gates: vec!["infrastructure_ready".to_string()],
            dependencies: vec![],
            estimated_duration: None, // Contracts Milestone doesn't have this field
        })
    }

    fn optimize_for_parallelism(&self, milestones: Vec<ContractMilestone>, resources: &ResourceInventory) -> Result<Vec<ContractMilestone>> {
        // Basic optimization - real implementation would analyze dependencies and resources
        Ok(milestones)
    }

    fn generate_rollback_plan(&self, objective: &str) -> String {
        format!("Revert changes made for: {}", objective)
    }

    fn calculate_critical_path(&self, nodes: &HashMap<String, DependencyNode>, edges: &[DependencyEdge]) -> Result<Vec<String>> {
        // Simplified critical path calculation
        Ok(vec![])
    }

    fn identify_parallel_groups(&self, nodes: &HashMap<String, DependencyNode>, edges: &[DependencyEdge]) -> Result<Vec<Vec<String>>> {
        // Simplified parallel group identification
        Ok(vec![])
    }
}

// Supporting types and implementations
#[derive(Debug, Clone)]
pub enum TaskComplexity {
    Simple,
    Moderate,
    Complex,
}

#[derive(Debug, Clone)]
pub struct DependencyAnalysis {
    pub dependencies: HashMap<String, Vec<String>>,
    pub blocking_items: HashSet<String>,
    pub dependency_graph: DependencyGraph,
}

impl DependencyAnalysis {
    fn get_dependencies_for_criterion(&self, criterion_id: &str) -> Vec<String> {
        self.dependencies.get(criterion_id).cloned().unwrap_or_default()
    }

    fn is_blocking_criterion(&self, criterion_id: &str) -> bool {
        // Simplified - real implementation would check if criterion blocks others
        false
    }

    fn get_blocking_reason(&self, criterion_id: &str) -> Option<String> {
        if self.is_blocking_criterion(criterion_id) {
            Some("Blocks dependent milestones".to_string())
        } else {
            None
        }
    }

    fn has_blocking_dependencies(&self) -> bool {
        !self.blocking_items.is_empty()
    }

    fn dependency_count(&self) -> usize {
        self.dependencies.len()
    }

    fn is_blocking_dependency(&self, dep: &str) -> bool {
        self.blocking_items.contains(dep)
    }
}

// Import missing types
use agent_agency_contracts::planning_io::{
    MilestoneScope, EvidenceGate, DependencyGraph, DependencyNode, DependencyEdge,
    DependencyNodeType, DependencyEdgeType, MilestonePriority,
};
use crate::planning::plan_types::{OrchestrationMetadata, ResourceInventory};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_generation_context() {
        let context = PlanGenerationContext {
            working_spec: Box::new(MockWorkingSpecProvider),
            task_descriptor: Box::new(MockTaskDescriptorProvider),
            resource_inventory: ResourceInventory {
                available_cpu_cores: 4,
                available_memory_mb: 8192,
                available_disk_mb: 102400,
                available_network_mbps: 100.0,
                available_workers: HashMap::new(),
            },
            constraints: PlanningConstraints {
                max_planning_time_ms: 30000,
                max_complexity: 10,
                risk_tolerance: RiskTolerance::Balanced,
                cost_limits: None,
                quality_requirements: QualityRequirements {
                    min_coverage: 0.8,
                    min_mutation_score: 0.5,
                    security_scan_required: true,
                    manual_review_required: false,
                    council_approval_required: true,
                },
                parallel_preferences: ParallelPreferences {
                    max_parallelism: 3,
                    prefer_parallel: true,
                    allow_resource_contention: false,
                    load_balancing: LoadBalancingStrategy::Even,
                },
            },
            historical_data: None,
        };

        assert_eq!(context.resource_inventory.available_cpu_cores, 4);
        assert_eq!(context.constraints.max_complexity, 10);
    }

    #[test]
    fn test_dependency_analysis() {
        let mut dependencies = HashMap::new();
        dependencies.insert("M1".to_string(), vec!["M0".to_string()]);
        dependencies.insert("M2".to_string(), vec!["M0".to_string()]);

        let analysis = DependencyAnalysis {
            dependencies,
            blocking_items: HashSet::from(["M0".to_string()]),
            dependency_graph: DependencyGraph {
                nodes: HashMap::new(),
                edges: vec![],
                critical_path: vec![],
                parallel_groups: vec![],
                has_cycles: false,
                cycles: vec![],
            },
        };

        assert_eq!(analysis.get_dependencies_for_criterion("M1"), vec!["M0"]);
        assert!(analysis.has_blocking_dependencies());
        assert_eq!(analysis.dependency_count(), 2);
    }

    // Mock implementations for testing
    struct MockWorkingSpecProvider;
    struct MockTaskDescriptorProvider;

    #[async_trait]
    impl WorkingSpecProvider for MockWorkingSpecProvider {
        async fn get_working_spec(&self) -> Result<WorkingSpec, anyhow::Error> {
            Ok(WorkingSpec {
                id: "test-spec".to_string(),
                title: "Test Spec".to_string(),
                risk_tier: 2,
                acceptance: vec![],
                scope: Default::default(),
                constraints: Default::default(),
                context: Default::default(),
                metadata: Default::default(),
            })
        }
    }

    #[async_trait]
    impl TaskDescriptorProvider for MockTaskDescriptorProvider {
        async fn get_task_descriptor(&self) -> Result<TaskDescriptor, anyhow::Error> {
            Ok(TaskDescriptor {
                task_id: "test-task".to_string(),
                description: "Test task".to_string(),
                priority: Default::default(),
                dependencies: vec![],
                metadata: HashMap::new(),
            })
        }
    }

    // Import missing types
    use agent_agency_contracts::{WorkingSpec, TaskDescriptor};
    use crate::planning::plan_types::{PlanningConstraints, RiskTolerance, QualityRequirements, ParallelPreferences, LoadBalancingStrategy};
}
