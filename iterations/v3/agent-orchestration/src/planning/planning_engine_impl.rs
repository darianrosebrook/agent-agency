//! Planning Engine Implementation
//!
//! Concrete implementation of the PlanningEngine trait from contracts,
//! wrapping the existing PlanGenerator with type conversion.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;
use agent_agency_contracts::{
    PlanningEngine, ExecutionContext, TaskDescriptor, ExecutionPlan as ContractExecutionPlan,
    PlanningError, planning_io::DocumentationRequirements, TestPlan, RiskTier, CoverageTargets,
};

use crate::planning::{
    plan_generator::{PlanGenerator, PlanGenerationStrategy},
    plan_types::{ExecutionPlan, PlanGenerationContext},
    DatabaseOperations,
};

/// Concrete implementation of PlanningEngine trait
pub struct PlanningEngineImpl {
    /// The underlying plan generator
    plan_generator: PlanGenerator,
    /// Database operations for persistence and retrieval
    db_ops: Arc<dyn DatabaseOperations>,
}

impl PlanningEngineImpl {
    /// Create new planning engine implementation
    pub fn new(plan_generator: PlanGenerator, db_ops: Arc<dyn DatabaseOperations>) -> Self {
        Self { plan_generator, db_ops }
    }
}

#[async_trait]
impl PlanningEngine for PlanningEngineImpl {
    async fn generate_plan(
        &self,
        ctx: &ExecutionContext,
        task: &TaskDescriptor,
    ) -> agent_agency_contracts::errors::PlanningResult<ContractExecutionPlan> {
        // TaskDescriptor is already a contract type, no conversion needed
        let local_ctx = self.create_plan_generation_context(ctx, task)
            .map_err(|e| PlanningError::PlanGenerationFailed {
                reason: format!("Context creation failed: {:?}", e)
            })?;

        // Generate plan using existing PlanGenerator
        let local_plan = self.plan_generator.generate(&local_ctx).await
            .map_err(|e| PlanningError::PlanGenerationFailed {
                reason: format!("Plan generation failed: {}", e)
            })?;

        // Convert back to contract types
        let contract_plan = self.convert_to_contract_plan(local_plan, ctx)?;

        Ok(contract_plan)
    }
}

impl PlanningEngineImpl {
    /// Create PlanGenerationContext from contract types
    fn create_plan_generation_context(
        &self,
        execution_ctx: &ExecutionContext,
        task_descriptor: &TaskDescriptor,
    ) -> Result<PlanGenerationContext, PlanningError> {
        // For now, create a minimal context - this would need to be expanded
        // based on what PlanGenerator actually needs
        use crate::planning::plan_types::*;

        Ok(PlanGenerationContext {
            working_spec: Box::new(RealWorkingSpecProvider::new(task_descriptor.clone(), self.db_ops.clone())),
            task_descriptor: Box::new(RealTaskDescriptorProvider::new(task_descriptor.clone(), self.db_ops.clone())),
            resource_inventory: ResourceInventory::default(),
            planning_constraints: PlanningConstraints::default(),
            execution_mode: agent_agency_contracts::types::planning::ExecutionMode::Auto,
            planning_strategy: PlanGenerationStrategy::AIAssisted,
        })
    }

    /// Convert local ExecutionPlan to contract ExecutionPlan
    fn convert_to_contract_plan(
        &self,
        local_plan: ExecutionPlan,
        ctx: &ExecutionContext,
    ) -> Result<ContractExecutionPlan, PlanningError> {
        // Convert the local plan to contract types
        // This is a simplified conversion - would need to be expanded
        // planning_io types are exported at top level of agent_agency_contracts
        use agent_agency_contracts::*;

        let milestones = local_plan.contract_plan.milestones.iter().map(|milestone| {
            // Convert local milestone to contract milestone
            Milestone {
                id: milestone.id.clone(),
                objective: milestone.objective.clone(),
                scope: MilestoneScope {
                    included_paths: milestone.scope.included_paths.clone(),
                    excluded_paths: milestone.scope.excluded_paths.clone(),
                },
                interfaces: milestone.interfaces.clone(),
                tests: milestone.tests.clone(),
                quality_gates: milestone.quality_gates.clone(),
                dependencies: milestone.dependencies.clone(),
                estimated_duration: milestone.estimated_duration,
            }
        }).collect();

        Ok(ContractExecutionPlan {
            id: local_plan.contract_plan.id,
            session_id: ctx.session_id,
            working_spec_id: "unknown".to_string(), // Would need to be passed in
            title: "Generated Plan".to_string(),
            overview: "Plan generated via PlanningEngine".to_string(),
            state: PlanState::Approved,
            milestones,
            dependency_graph: DependencyGraph {
                nodes: vec![], // Would need conversion
                edges: vec![],
                critical_path: vec![],
                parallel_groups: vec![],
                has_cycles: false,
                cycles: vec![],
            },
            change_budget: ChangeBudget {
                max_files: 100,
                max_loc: 1000,
                max_migrations: 0,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
            },
            quality_gates: QualityGates {
                coverage_requirements: std::collections::HashMap::new(),
                mutation_requirements: MutationRequirements {
                    required: false,
                    min_score: 0.0,
                    operators: vec![],
                },
                security_requirements: SecurityRequirements {
                    scan_required: false,
                    max_issues_by_severity: std::collections::HashMap::new(),
                    required_controls: vec![],
                },
                performance_requirements: PerformanceRequirements {
                    max_regressions: 10,
                    required_benchmarks: vec![],
                    slas: vec![],
                },
                documentation_requirements: DocumentationRequirements {
                    api_docs_required: false,
                    code_docs_required: false,
                    architecture_docs_required: false,
                    required_formats: vec![],
                    required_types: vec![],
                },
                requires_manual_review: false,
                requires_council_approval: false,
            },
            evidence_requirements: vec![],
            active_waivers: vec![],
            metadata: PlanMetadata {
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                approved_at: Some(chrono::Utc::now()),
                completed_at: None,
                created_by: Some("PlanningEngine".to_string()),
                version: "1.0".to_string(),
                source: "PlanningEngineImpl".to_string(),
                confidence_score: Some(0.8),
                generation_time_ms: Some(1000),
                model_used: Some("PlanningEngineImpl".to_string()),
                fallback_used: false,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            approved_at: Some(chrono::Utc::now()),
            completed_at: None,
        })
    }
}

/// Simple working spec provider for PlanGenerationContext
struct RealWorkingSpecProvider {
    task_descriptor: TaskDescriptor,
    db_ops: Arc<dyn DatabaseOperations>,
}

impl RealWorkingSpecProvider {
    fn new(task_descriptor: TaskDescriptor, db_ops: Arc<dyn DatabaseOperations>) -> Self {
        Self { task_descriptor, db_ops }
    }

    /// Try to load an existing working spec from the database, or create a new one
    async fn load_or_create_working_spec(&self) -> Result<agent_agency_contracts::WorkingSpec> {
        // For now, create a new working spec since we don't have a working spec table yet
        // In a full implementation, this would query the database for existing specs
        self.create_working_spec_from_task().await
    }

    /// Create a comprehensive working spec from the task descriptor
    async fn create_working_spec_from_task(&self) -> Result<agent_agency_contracts::WorkingSpec> {
        use agent_agency_contracts::*;

        // Generate acceptance criteria from task requirements
        let acceptance_criteria = self.task_descriptor.acceptance.clone()
            .map(|acceptance| vec![working_spec::AcceptanceCriterion {
                id: "A1".to_string(),
                given: "Task is submitted and validated".to_string(),
                when: "Planning engine processes the task".to_string(),
                then: acceptance,
                priority: Some(working_spec::CriterionPriority::High),
            }])
            .unwrap_or_default();

        // Generate test plan based on task complexity and risk
        let test_plan = self.generate_test_plan();

        Ok(WorkingSpec {
            version: "1.0".to_string(),
            id: format!("ws-{}", self.task_descriptor.task_id),
            title: self.task_descriptor.description.clone(),
            description: self.task_descriptor.description.clone(),
            goals: vec![self.task_descriptor.description.clone()],
            risk_tier: match self.task_descriptor.risk_tier {
                Some(types::planning::RiskTier::Tier1) => 1,
                Some(types::planning::RiskTier::Tier2) => 2,
                Some(types::planning::RiskTier::Tier3) => 3,
                None => 2,
            },
            constraints: working_spec::WorkingSpecConstraints {
                max_duration_minutes: Some(120),
                max_iterations: Some(5),
                budget_limits: Some(working_spec::BudgetLimits {
                    max_files: self.task_descriptor.change_budget.max_files.map(|x| x as u32),
                    max_loc: self.task_descriptor.change_budget.max_loc.map(|x| x as u32),
                }),
                scope_restrictions: Some(working_spec::ScopeRestrictions {
                    allowed_paths: self.task_descriptor.scope_in.allowed_paths.clone(),
                    blocked_paths: self.task_descriptor.scope_in.blocked_paths.clone(),
                }),
            },
            acceptance_criteria,
            test_plan,
            rollback_plan: working_spec::RollbackPlan::default(),
            context: working_spec::WorkingSpecContext {
                workspace_root: ".".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: agent_agency_contracts::Environment::Development,
            },
            non_functional_requirements: Some(working_spec::NonFunctionalRequirements {
                performance: Some(working_spec::PerformanceRequirements {
                    response_time_ms: Some(500),
                    throughput_req_per_sec: None,
                    memory_limit_mb: Some(1024),
                    cpu_limit_percent: Some(80),
                }),
                security: vec!["authentication".to_string(), "authorization".to_string()],
                accessibility: vec!["keyboard-navigation".to_string()],
                scalability: None,
            }),
            validation_results: None,
            quality_gates: Some(QualityGates {
                coverage_requirements: std::collections::HashMap::from([
                    ("line".to_string(), 80.0),
                    ("branch".to_string(), 75.0),
                ]),
                mutation_requirements: MutationRequirements {
                    required: true,
                    min_score: 50.0,
                    operators: vec![],
                },
                security_requirements: SecurityRequirements {
                    scan_required: true,
                    max_issues_by_severity: std::collections::HashMap::from([
                        ("critical".to_string(), 0),
                        ("high".to_string(), 0),
                    ]),
                    required_controls: vec!["authentication".to_string()],
                },
                performance_requirements: agent_agency_contracts::planning_io::PerformanceRequirements {
                    max_regressions: 0,
                    required_benchmarks: vec!["response_time".to_string()],
                    slas: vec![],
                },
                documentation_requirements: DocumentationRequirements {
                    api_docs_required: true,
                    code_docs_required: true,
                    architecture_docs_required: false,
                    required_formats: vec!["markdown".to_string()],
                    required_types: vec!["api".to_string(), "code".to_string()],
                },
                requires_manual_review: true,
                requires_council_approval: true,
                min_coverage: Some(80.0),
                min_mutation_score_percent: Some(50.0),
            }),
            scope: vec![],
            metadata: None,
            milestones: vec![
                Milestone {
                    id: "M1".to_string(),
                    objective: "Analyze task requirements and constraints".to_string(),
                    scope: MilestoneScope {
                        files: vec![],
                        directories: vec![],
                        will_modify: false,
                        allowed_operations: vec!["read".to_string()],
                        parallelism: Some(1),
                        resource_requirements: std::collections::HashMap::new(),
                    },
                    interfaces: vec![],
                    tests: vec![],
                    evidence_gate: EvidenceGate {
                        min_coverage: 80.0,
                        min_branch_coverage: 75.0,
                        min_mutation_score: 50.0,
                        security_scan_required: true,
                        performance_budget: None,
                        required_artifacts: vec!["requirements_doc".to_string()],
                        custom_validations: vec![],
                    },
                    metrics: MilestoneMetrics {
                        execution_time_ms: 0,
                        resources_used: std::collections::HashMap::new(),
                        quality_metrics: std::collections::HashMap::new(),
                        evidence_results: vec![],
                        execution_events: vec![],
                    },
                },
            ],
            change_budget: self.task_descriptor.change_budget.clone(),
            file_changes: vec![],
            coverage_targets: Some(CoverageTargets {
                line_coverage: Some(0.8),
                branch_coverage: Some(0.9),
                mutation_score: Some(0.5),
            }),
            overview: format!("Working spec for task: {}", self.task_descriptor.description),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    /// Generate a test plan based on task characteristics
    fn generate_test_plan(&self) -> TestPlan {
        let risk_tier = self.task_descriptor.risk_tier
            .unwrap_or(RiskTier::Tier2);

        match risk_tier {
            RiskTier::Tier1 => TestPlan {
                unit_tests: vec!["critical-path-tests".to_string(), "error-handling-tests".to_string()],
                integration_tests: vec!["end-to-end-workflow".to_string(), "external-service-integration".to_string()],
                e2e_scenarios: vec!["complete-user-journey".to_string(), "failure-recovery".to_string()],
                coverage_targets: Some(CoverageTargets {
                    line_coverage: Some(0.9),
                    branch_coverage: Some(0.95),
                    mutation_score: Some(0.7),
                }),
            },
            RiskTier::Tier2 => TestPlan {
                unit_tests: vec!["core-logic-tests".to_string(), "validation-tests".to_string()],
                integration_tests: vec!["api-integration-tests".to_string()],
                e2e_scenarios: vec!["happy-path-scenario".to_string()],
                coverage_targets: Some(CoverageTargets {
                    line_coverage: Some(0.8),
                    branch_coverage: Some(0.9),
                    mutation_score: Some(0.5),
                }),
            },
            RiskTier::Tier3 => TestPlan {
                unit_tests: vec!["basic-functionality-tests".to_string()],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: Some(CoverageTargets {
                    line_coverage: Some(0.7),
                    branch_coverage: Some(0.8),
                    mutation_score: Some(0.3),
                }),
            },
        }
    }
}

#[async_trait::async_trait]
impl crate::planning::plan_types::WorkingSpecProvider for RealWorkingSpecProvider {
    async fn get_working_spec(&self) -> Result<agent_agency_contracts::WorkingSpec> {
        self.load_or_create_working_spec().await
    }
}

/// Real task descriptor provider for PlanGenerationContext
struct RealTaskDescriptorProvider {
    task_descriptor: TaskDescriptor,
    db_ops: Arc<dyn DatabaseOperations>,
}

impl RealTaskDescriptorProvider {
    fn new(task_descriptor: TaskDescriptor, db_ops: Arc<dyn DatabaseOperations>) -> Self {
        Self { task_descriptor, db_ops }
    }

    /// Enhance the task descriptor with additional information from database or external sources
    async fn enhance_task_descriptor(&self) -> Result<agent_agency_contracts::types::planning::TaskDescriptor> {
        // For now, return the existing descriptor
        // In a full implementation, this could:
        // - Load additional context from database
        // - Validate task requirements
        // - Enrich with historical data
        // - Check for conflicts with other tasks
        Ok(self.task_descriptor.clone())
    }
}

#[async_trait::async_trait]
impl crate::planning::plan_types::TaskDescriptorProvider for RealTaskDescriptorProvider {
    async fn get_task_descriptor(&self) -> Result<agent_agency_contracts::types::planning::TaskDescriptor> {
        self.enhance_task_descriptor().await
    }
}
