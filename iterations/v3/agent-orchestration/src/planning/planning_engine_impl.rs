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
    PlanningError,
    types::planning::RiskTier,
    working_spec::{TestPlan, CoverageTargets, UnitTestSpec, IntegrationTestSpec, E2eScenario},
};

use crate::planning::{
    plan_generator::PlanGenerator,
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
        // TODO: Build comprehensive plan generation context
        //       Currently creates minimal context; should build comprehensive context based on PlanGenerator requirements and available data.
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
        // - Context includes all required data for PlanGenerator
        // - Context is built from available sources
        // - Missing data is handled gracefully
        // - Context is comprehensive and accurate
        //
        // DEPENDENCIES:
        // - PlanGenerator requirements (Required)
        // - Context data sources (Required)
        // - Context building utilities (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (planning context feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Planning engine expertise
        use crate::planning::plan_types::*; // Temporary: minimal context until comprehensive context building

        Ok(PlanGenerationContext {
            working_spec_provider: Box::new(RealWorkingSpecProvider::new(task_descriptor.clone(), self.db_ops.clone())),
            task_descriptor: Box::new(RealTaskDescriptorProvider::new(task_descriptor.clone(), self.db_ops.clone())),
            resource_inventory: ResourceInventory::default(),
            constraints: PlanningConstraints::default(),
            historical_data: None,
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
        // TODO: Implement comprehensive plan type conversion
        //       Currently uses basic conversion; should implement comprehensive conversion between local plan types and contract types.
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
        // - Plan types are converted comprehensively
        // - All fields are mapped correctly
        // - Type conversions are accurate
        // - Conversion handles edge cases
        //
        // DEPENDENCIES:
        // - Local plan types (Required)
        // - Contract types (Required)
        // - Type conversion utilities (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (type conversion feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Type system expertise
        use agent_agency_contracts::*; // Temporary: basic conversion until comprehensive implementation

        let milestones: Vec<Milestone> = local_plan.contract_plan.milestones.iter().map(|milestone| {
            // Convert local milestone to contract milestone
            Milestone {
                id: milestone.id.clone(),
                objective: milestone.objective.clone(),
                scope: MilestoneScope {
                    files: milestone.scope.files.clone(),
                    directories: milestone.scope.directories.clone(),
                    included_paths: milestone.scope.included_paths.clone(),
                    excluded_paths: milestone.scope.excluded_paths.clone(),
                    will_modify: milestone.scope.will_modify,
                    allowed_operations: milestone.scope.allowed_operations.clone(),
                    parallelism: milestone.scope.parallelism,
                    resource_requirements: milestone.scope.resource_requirements.clone(),
                },
                interfaces: milestone.interfaces.clone(),
                tests: milestone.tests.clone(),
                evidence_gate: milestone.evidence_gate.clone(),
                quality_gates: milestone.quality_gates.clone(),
                dependencies: milestone.dependencies.clone(),
                estimated_duration: milestone.estimated_duration,
                rollback_plan: milestone.rollback_plan.clone(),
                state: milestone.state.clone(),
                assigned_workers: milestone.assigned_workers.clone(),
                estimated_effort: milestone.estimated_effort,
                priority: milestone.priority.clone(),
                risk_tier: milestone.risk_tier,
                is_blocking: milestone.is_blocking,
                blocking_reason: milestone.blocking_reason.clone(),
                metrics: milestone.metrics.clone(),
            }
        }).collect();

        // Simply clone and return the contract plan from the local plan
        Ok(local_plan.contract_plan.clone())
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
        // TODO: Query database for existing working specs
        //       Currently creates new spec; should query database for existing working specs before creating new ones.
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
        // - Existing specs are queried from database
        // - Specs are retrieved correctly
        // - Database errors are handled gracefully
        // - Query performance is acceptable
        //
        // DEPENDENCIES:
        // - Database connection (Required)
        // - Working spec table schema (Required)
        // - Database query utilities (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (database integration feature)
        // - Change Budget: ~80 LOC
        // - Reviewer Requirements: Database expertise
        self.create_working_spec_from_task().await // Temporary: create new until database query is implemented
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
                priority: Some(working_spec::CriterionPriority::Must),
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
                    max_files: Some(self.task_descriptor.change_budget.max_files as u32),
                    max_loc: Some(self.task_descriptor.change_budget.max_loc as u32),
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
                    min_coverage: 0.0,
                    quality_checks: vec![],
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
                        included_paths: vec![],
                        excluded_paths: vec![],
                        will_modify: false,
                        allowed_operations: vec!["read".to_string()],
                        parallelism: Some(1),
                        resource_requirements: std::collections::HashMap::new(),
                    },
                    interfaces: vec![],
                    tests: vec![],
                    evidence_gate: EvidenceGate {
                        min_coverage: 0.8,
                        min_branch_coverage: 0.75,
                        min_mutation_score: 0.5,
                        security_scan_required: true,
                        performance_budget: None,
                        required_artifacts: vec!["requirements_doc".to_string()],
                        custom_validations: vec![],
                    },
                    quality_gates: vec![],
                    dependencies: vec![],
                    estimated_duration: None,
                    rollback_plan: "Revert analysis changes".to_string(),
                    state: agent_agency_contracts::planning_io::MilestoneState::Pending,
                    assigned_workers: vec![],
                    estimated_effort: 0.5,
                    priority: agent_agency_contracts::planning_io::MilestonePriority::Normal,
                    risk_tier: 2,
                    is_blocking: false,
                    blocking_reason: None,
                    metrics: Some(MilestoneMetrics {
                        worker_performance: std::collections::HashMap::new(),
                        execution_time_ms: 0,
                        resources_used: std::collections::HashMap::new(),
                        quality_metrics: std::collections::HashMap::new(),
                        evidence_results: vec![],
                        execution_events: vec![],
                    }),
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
        let risk_tier = self.task_descriptor.risk_tier.clone()
            .unwrap_or(RiskTier::Tier2);

        match risk_tier {
            RiskTier::Tier1 => TestPlan {
                unit_tests: vec![
                    UnitTestSpec {
                        description: "critical-path-tests".to_string(),
                        target_function: None,
                        test_cases: vec![],
                    },
                    UnitTestSpec {
                        description: "error-handling-tests".to_string(),
                        target_function: None,
                        test_cases: vec![],
                    },
                ],
                integration_tests: vec![
                    IntegrationTestSpec {
                        description: "end-to-end-workflow".to_string(),
                        components: vec![],
                        test_cases: vec![],
                    },
                    IntegrationTestSpec {
                        description: "external-service-integration".to_string(),
                        components: vec![],
                        test_cases: vec![],
                    },
                ],
                e2e_scenarios: vec![
                    E2eScenario {
                        description: "complete-user-journey".to_string(),
                        user_journey: "Complete user journey from registration to task completion".to_string(),
                        expected_outcomes: vec![],
                    },
                    E2eScenario {
                        description: "failure-recovery".to_string(),
                        user_journey: "User journey with failure scenarios and recovery".to_string(),
                        expected_outcomes: vec![],
                    },
                ],
                coverage_targets: Some(CoverageTargets {
                    line_coverage: Some(0.9),
                    branch_coverage: Some(0.95),
                    mutation_score: Some(0.7),
                }),
            },
            RiskTier::Tier2 => TestPlan {
                unit_tests: vec![
                    UnitTestSpec {
                        description: "core-logic-tests".to_string(),
                        target_function: None,
                        test_cases: vec![],
                    },
                    UnitTestSpec {
                        description: "validation-tests".to_string(),
                        target_function: None,
                        test_cases: vec![],
                    },
                ],
                integration_tests: vec![
                    IntegrationTestSpec {
                        description: "api-integration-tests".to_string(),
                        components: vec![],
                        test_cases: vec![],
                    },
                ],
                e2e_scenarios: vec![
                    E2eScenario {
                        description: "happy-path-scenario".to_string(),
                        user_journey: "Happy path user journey".to_string(),
                        expected_outcomes: vec![],
                    },
                ],
                coverage_targets: Some(CoverageTargets {
                    line_coverage: Some(0.8),
                    branch_coverage: Some(0.9),
                    mutation_score: Some(0.5),
                }),
            },
            RiskTier::Tier3 => TestPlan {
                unit_tests: vec![
                    UnitTestSpec {
                        description: "basic-functionality-tests".to_string(),
                        target_function: None,
                        test_cases: vec![],
                    },
                ],
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
        // TODO: Enhance task descriptor with database and external data
        //       Currently returns existing descriptor; should enhance with additional context from database, validation, and historical data.
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
        // - Descriptor is enhanced with database context
        // - Task requirements are validated
        // - Historical data enriches descriptor
        // - Enhancement improves task quality
        //
        // DEPENDENCIES:
        // - Database connection (Required)
        // - External data sources (Optional)
        // - Historical data infrastructure (Required)
        //
        // ESTIMATED EFFORT: 5-6 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (task enhancement feature)
        // - Change Budget: ~120 LOC
        // - Reviewer Requirements: Task management expertise
        // Temporary: return existing until enhancement is implemented
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
