//! Planning Engine Implementation
//!
//! Concrete implementation of the PlanningEngine trait from contracts,
//! wrapping the existing PlanGenerator with type conversion.
//!
//! @author @darianrosebrook

use agent_agency_contracts::{
    types::planning::RiskTier,
    working_spec::{CoverageTargets, E2eScenario, IntegrationTestSpec, TestPlan, UnitTestSpec},
    ExecutionContext, ExecutionPlan as ContractExecutionPlan, PlanningEngine, PlanningError,
    TaskDescriptor,
};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

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
        Self {
            plan_generator,
            db_ops,
        }
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
        let local_ctx = self
            .create_plan_generation_context(ctx, task)
            .map_err(|e| PlanningError::PlanGenerationFailed {
                reason: format!("Context creation failed: {:?}", e),
            })?;

        // Generate plan using existing PlanGenerator
        let local_plan = self
            .plan_generator
            .generate(&local_ctx)
            .await
            .map_err(|e| PlanningError::PlanGenerationFailed {
                reason: format!("Plan generation failed: {}", e),
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
        use crate::planning::plan_types::*;
        use std::collections::HashMap;

        // Build resource inventory from execution context planning_metadata
        // Extract resource information from metadata or use defaults
        let available_cpu_cores = execution_ctx
            .planning_metadata
            .get("available_cpu_cores")
            .and_then(|v| v.as_u64())
            .unwrap_or(4) as usize;
        let available_memory_mb = execution_ctx
            .planning_metadata
            .get("available_memory_mb")
            .and_then(|v| v.as_u64())
            .unwrap_or(8192) as usize;
        let available_disk_mb = execution_ctx
            .planning_metadata
            .get("available_disk_mb")
            .and_then(|v| v.as_u64())
            .unwrap_or(102400) as usize;
        let available_network_mbps = execution_ctx
            .planning_metadata
            .get("available_network_mbps")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0);

        // Count workers from worker_assignments in metadata if available
        let mut worker_counts: HashMap<String, usize> = HashMap::new();
        if let Some(assignments_json) = execution_ctx.planning_metadata.get("worker_assignments") {
            if let Some(assignments_array) = assignments_json.as_array() {
                for assignment in assignments_array {
                    if let Some(worker_id) = assignment.get("worker_id").and_then(|v| v.as_str()) {
                        *worker_counts.entry(worker_id.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        let resource_inventory = ResourceInventory {
            available_cpu_cores,
            available_memory_mb,
            available_disk_mb,
            available_network_mbps,
            available_workers: worker_counts,
        };

        // Build planning constraints from task descriptor
        let risk_tolerance = match task_descriptor.risk_tier {
            Some(RiskTier::Tier1) => RiskTolerance::Conservative,
            Some(RiskTier::Tier2) => RiskTolerance::Balanced,
            Some(RiskTier::Tier3) | None => RiskTolerance::Aggressive,
        };

        // Determine quality requirements based on risk tier
        let quality_requirements = match task_descriptor.risk_tier {
            Some(RiskTier::Tier1) => QualityRequirements {
                min_coverage: 0.9,       // 90% for Tier 1
                min_mutation_score: 0.7, // 70% for Tier 1
                security_scan_required: true,
                manual_review_required: true,
                council_approval_required: true,
            },
            Some(RiskTier::Tier2) => QualityRequirements {
                min_coverage: 0.8,       // 80% for Tier 2
                min_mutation_score: 0.5, // 50% for Tier 2
                security_scan_required: true,
                manual_review_required: false,
                council_approval_required: false,
            },
            Some(RiskTier::Tier3) | None => QualityRequirements {
                min_coverage: 0.7,       // 70% for Tier 3
                min_mutation_score: 0.3, // 30% for Tier 3
                security_scan_required: false,
                manual_review_required: false,
                council_approval_required: false,
            },
        };

        // Build cost limits from change budget if available
        let cost_limits = if task_descriptor.change_budget.max_files > 0 {
            // Estimate cost based on change budget
            // Rough estimate: 1 cent per file changed, 0.1 cents per LOC
            let estimated_cost_cents = (task_descriptor.change_budget.max_files as u32 * 1)
                + ((task_descriptor.change_budget.max_loc as f64 / 10.0) as u32);
            Some(CostLimits {
                max_cost_cents: estimated_cost_cents,
                cost_per_ms_budget: 0.001, // 0.001 cents per millisecond
                optimization_priority: CostOptimizationPriority::Balanced,
            })
        } else {
            None
        };

        // Determine max complexity based on change budget
        let max_complexity = if task_descriptor.change_budget.max_files > 0 {
            // Complexity roughly correlates with number of files and LOC
            (task_descriptor.change_budget.max_files as usize * 10).min(1000) // Cap at 1000
        } else {
            100 // Default complexity
        };

        // Determine parallel preferences based on task priority and blast radius
        let worker_count = execution_ctx
            .planning_metadata
            .get("worker_assignments")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(3);
        let max_parallelism = if task_descriptor.blast_radius.modules.is_empty() {
            // No blast radius restrictions - allow more parallelism
            worker_count.max(3)
        } else {
            // Limited blast radius - reduce parallelism
            task_descriptor.blast_radius.modules.len().min(2)
        };

        let prefer_parallel = matches!(
            task_descriptor.priority,
            agent_agency_contracts::types::planning::TaskPriority::Low
                | agent_agency_contracts::types::planning::TaskPriority::Normal
        );

        let parallel_preferences = ParallelPreferences {
            max_parallelism,
            prefer_parallel,
            allow_resource_contention: false, // Default to no resource contention
            load_balancing: LoadBalancingStrategy::Even, // Default to even distribution
        };

        // Build planning constraints
        let constraints = PlanningConstraints {
            max_planning_time_ms: 300000, // 5 minutes default
            max_complexity,
            risk_tolerance,
            cost_limits,
            quality_requirements,
            parallel_preferences,
        };

        // Determine execution mode from task descriptor
        let execution_mode = task_descriptor.execution_mode.clone();

        // Determine planning strategy based on risk tier and priority
        let planning_strategy = match task_descriptor.risk_tier {
            Some(RiskTier::Tier1) => PlanGenerationStrategy::HumanGuided, // Most conservative for Tier1
            Some(RiskTier::Tier2) => PlanGenerationStrategy::AIAssisted,
            Some(RiskTier::Tier3) | None => PlanGenerationStrategy::AIAssisted,
        };

        Ok(PlanGenerationContext {
            working_spec_provider: Box::new(RealWorkingSpecProvider::new(
                task_descriptor.clone(),
                self.db_ops.clone(),
            )),
            task_descriptor: Box::new(RealTaskDescriptorProvider::new(
                task_descriptor.clone(),
                self.db_ops.clone(),
            )),
            resource_inventory,
            constraints: constraints.clone(),
            historical_data: None, // Historical data would require database queries - can be enhanced later
            planning_constraints: constraints,
            execution_mode,
            planning_strategy,
        })
    }

    /// Convert local ExecutionPlan to contract ExecutionPlan
    ///
    /// Comprehensive conversion that maps:
    /// - All contract plan fields (milestones, dependencies, etc.)
    /// - Orchestration metadata into plan metadata
    /// - Execution context into contract execution context
    /// - Execution state information into plan state
    fn convert_to_contract_plan(
        &self,
        local_plan: ExecutionPlan,
        _ctx: &ExecutionContext,
    ) -> Result<ContractExecutionPlan, PlanningError> {
        use agent_agency_contracts::planning_io::PlanMetadata;
        use tracing::debug;

        // Start with base contract plan
        let mut contract_plan = local_plan.contract_plan.clone();

        // Enhance metadata with orchestration metadata
        contract_plan.metadata = PlanMetadata {
            created_at: contract_plan.metadata.created_at,
            updated_at: contract_plan.metadata.updated_at,
            approved_at: contract_plan.metadata.approved_at,
            completed_at: contract_plan.metadata.completed_at,
            created_by: contract_plan.metadata.created_by.clone(),
            version: contract_plan.metadata.version.clone(),
            source: local_plan.orchestration_meta.planning_engine.clone(),
            confidence_score: contract_plan.metadata.confidence_score,
            generation_time_ms: contract_plan.metadata.generation_time_ms,
            model_used: contract_plan.metadata.model_used.clone(),
            fallback_used: contract_plan.metadata.fallback_used,
            strategy: contract_plan.metadata.strategy.clone(),
            confidence: contract_plan.metadata.confidence,
            estimated_duration_ms: contract_plan.metadata.estimated_duration_ms,
            estimated_cost_cents: contract_plan.metadata.estimated_cost_cents,
            adaptive: contract_plan.metadata.adaptive,
            engine_version: local_plan.orchestration_meta.planning_version.clone(),
            additional_metadata: {
                let mut additional = contract_plan.metadata.additional_metadata.clone();
                // Add orchestration metadata to additional_metadata
                additional.insert(
                    "orchestrator_id".to_string(),
                    serde_json::json!(local_plan.orchestration_meta.orchestrator_id),
                );
                additional.insert(
                    "worker_pool_id".to_string(),
                    serde_json::json!(local_plan.orchestration_meta.worker_pool_id),
                );
                if let Some(ref council_session_id) =
                    local_plan.orchestration_meta.council_session_id
                {
                    additional.insert(
                        "council_session_id".to_string(),
                        serde_json::json!(council_session_id),
                    );
                }
                additional.insert(
                    "audit_correlation_id".to_string(),
                    serde_json::json!(local_plan
                        .orchestration_meta
                        .audit_correlation_id
                        .to_string()),
                );
                additional.insert(
                    "planning_engine".to_string(),
                    serde_json::json!(local_plan.orchestration_meta.planning_engine),
                );
                additional.insert(
                    "planning_version".to_string(),
                    serde_json::json!(local_plan.orchestration_meta.planning_version),
                );
                additional
            },
        };

        // Map execution context to contract execution context
        // Note: ContractExecutionPlan has an optional execution_context field of type ExecutionContext
        // Convert local ExecutionContext to contract ExecutionContext format
        contract_plan.execution_context =
            Some(agent_agency_contracts::types::execution::ExecutionContext {
                session_id: contract_plan.session_id,
                planning_engine: local_plan.orchestration_meta.planning_engine.clone(),
                engine_version: local_plan.orchestration_meta.planning_version.clone(),
                planning_metadata: {
                    let mut metadata = std::collections::HashMap::new();
                    metadata.insert(
                        "working_directory".to_string(),
                        serde_json::json!(local_plan.execution_context.working_directory),
                    );
                    metadata.insert(
                        "session_start".to_string(),
                        serde_json::json!(local_plan.execution_context.session_start.to_rfc3339()),
                    );
                    metadata.insert(
                        "available_cpu_cores".to_string(),
                        serde_json::json!(
                            local_plan
                                .execution_context
                                .available_resources
                                .available_cpu_cores
                        ),
                    );
                    metadata.insert(
                        "available_memory_mb".to_string(),
                        serde_json::json!(
                            local_plan
                                .execution_context
                                .available_resources
                                .available_memory_mb
                        ),
                    );
                    metadata.insert(
                        "available_disk_mb".to_string(),
                        serde_json::json!(
                            local_plan
                                .execution_context
                                .available_resources
                                .available_disk_mb
                        ),
                    );
                    metadata.insert(
                        "available_network_mbps".to_string(),
                        serde_json::json!(
                            local_plan
                                .execution_context
                                .available_resources
                                .available_network_mbps
                        ),
                    );
                    metadata.insert(
                        "orchestrator_id".to_string(),
                        serde_json::json!(local_plan.orchestration_meta.orchestrator_id),
                    );
                    metadata.insert(
                        "worker_pool_id".to_string(),
                        serde_json::json!(local_plan.orchestration_meta.worker_pool_id),
                    );
                    if let Some(ref council_session_id) =
                        local_plan.orchestration_meta.council_session_id
                    {
                        metadata.insert(
                            "council_session_id".to_string(),
                            serde_json::json!(council_session_id),
                        );
                    }
                    metadata.insert(
                        "audit_correlation_id".to_string(),
                        serde_json::json!(local_plan
                            .orchestration_meta
                            .audit_correlation_id
                            .to_string()),
                    );
                    // Add worker assignments as JSON
                    let worker_assignments_json: Vec<_> = local_plan
                        .execution_context
                        .worker_assignments
                        .iter()
                        .map(|(k, v)| {
                            serde_json::json!({
                                "milestone_id": k,
                                "worker_id": v.worker_id.to_string(),
                                "assigned_at": v.assigned_at.to_rfc3339(),
                                "status": format!("{:?}", v.status),
                            })
                        })
                        .collect();
                    metadata.insert(
                        "worker_assignments".to_string(),
                        serde_json::json!(worker_assignments_json),
                    );
                    metadata
                },
            });

        // Update plan state based on execution state if available
        if let Some(ref execution_state) = local_plan.execution_state {
            // Map execution state to contract plan state
            let has_failures = !execution_state.failed_milestones.is_empty();
            let all_completed =
                execution_state.completed_milestones.len() == contract_plan.milestones.len();
            let has_executing = !execution_state.executing_milestones.is_empty();

            contract_plan.state = if has_failures {
                agent_agency_contracts::planning_io::PlanState::Failed {
                    reason: "One or more milestones failed".to_string(),
                }
            } else if all_completed {
                agent_agency_contracts::planning_io::PlanState::Completed
            } else if has_executing {
                agent_agency_contracts::planning_io::PlanState::InProgress
            } else {
                contract_plan.state // Keep existing state if no clear mapping
            };
        }

        // Session ID is already set correctly from contract_plan.session_id
        // The execution context's session_id matches the plan's session_id

        debug!(
            plan_id = %contract_plan.id,
            orchestrator_id = %local_plan.orchestration_meta.orchestrator_id,
            planning_engine = %local_plan.orchestration_meta.planning_engine,
            "Converted local ExecutionPlan to ContractExecutionPlan with enhanced metadata"
        );

        Ok(contract_plan)
    }
}

/// Simple working spec provider for PlanGenerationContext
struct RealWorkingSpecProvider {
    task_descriptor: TaskDescriptor,
    #[allow(dead_code)] // Reserved for future use
    db_ops: Arc<dyn DatabaseOperations>,
}

impl RealWorkingSpecProvider {
    fn new(task_descriptor: TaskDescriptor, db_ops: Arc<dyn DatabaseOperations>) -> Self {
        Self {
            task_descriptor,
            db_ops,
        }
    }

    /// Try to load an existing working spec from the database, or create a new one
    async fn load_or_create_working_spec(&self) -> Result<agent_agency_contracts::WorkingSpec> {
        use tracing::debug;

        // Generate expected working_spec_id from task descriptor
        let expected_working_spec_id = format!("ws-{}", self.task_descriptor.task_id);

        // Query execution plans to find existing plan with matching working_spec_id
        match self.db_ops.get_execution_plans().await {
            Ok(plans) => {
                // Find plan with matching working_spec_id
                if let Some(existing_plan) = plans
                    .iter()
                    .find(|plan| plan.working_spec_id == expected_working_spec_id)
                    .cloned()
                {
                    debug!(
                        "Found existing execution plan for working_spec_id: {}",
                        expected_working_spec_id
                    );

                    // Reconstruct working spec from execution plan and task descriptor
                    // The execution plan contains derived information (milestones, quality_gates, etc.)
                    // but we reconstruct the working spec from the task descriptor to ensure consistency
                    let mut working_spec = self.create_working_spec_from_task().await?;

                    // Enhance working spec with data from execution plan if available
                    // Try to extract quality gates from plan metadata
                    if let Some(quality_gates_json) = existing_plan.metadata.get("quality_gates") {
                        if let Ok(quality_gates) =
                            serde_json::from_value::<
                                agent_agency_contracts::planning_io::QualityGates,
                            >(quality_gates_json.clone())
                        {
                            working_spec.quality_gates = Some(quality_gates);
                        }
                    }

                    // Also try to extract from quality_gates field directly
                    if let Ok(quality_gates) =
                        serde_json::from_value::<agent_agency_contracts::planning_io::QualityGates>(
                            existing_plan.quality_gates.clone(),
                        )
                    {
                        working_spec.quality_gates = Some(quality_gates);
                    }

                    // Use the existing working_spec_id to maintain consistency
                    working_spec.id = existing_plan.working_spec_id.clone();

                    debug!(
                        "Reconstructed working spec from existing execution plan: {}",
                        working_spec.id
                    );

                    return Ok(working_spec);
                }
            }
            Err(e) => {
                // Log error but continue to create new spec (graceful degradation)
                tracing::warn!(
                    "Failed to query execution plans for working spec lookup: {}. Creating new spec.",
                    e
                );
            }
        }

        // No existing plan found, create new working spec
        debug!(
            "No existing execution plan found for working_spec_id: {}. Creating new working spec.",
            expected_working_spec_id
        );

        self.create_working_spec_from_task().await
    }

    /// Create a comprehensive working spec from the task descriptor
    async fn create_working_spec_from_task(&self) -> Result<agent_agency_contracts::WorkingSpec> {
        use agent_agency_contracts::*;

        // Generate acceptance criteria from task requirements
        let acceptance_criteria = self
            .task_descriptor
            .acceptance
            .clone()
            .map(|acceptance| {
                vec![working_spec::AcceptanceCriterion {
                    id: "A1".to_string(),
                    given: "Task is submitted and validated".to_string(),
                    when: "Planning engine processes the task".to_string(),
                    then: acceptance,
                    priority: Some(working_spec::CriterionPriority::Must),
                }]
            })
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
                performance_requirements:
                    agent_agency_contracts::planning_io::PerformanceRequirements {
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
            milestones: vec![Milestone {
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
                metadata: std::collections::HashMap::new(),
            }],
            change_budget: self.task_descriptor.change_budget.clone(),
            file_changes: vec![],
            coverage_targets: Some(CoverageTargets {
                line_coverage: Some(0.8),
                branch_coverage: Some(0.9),
                mutation_score: Some(0.5),
            }),
            overview: format!(
                "Working spec for task: {}",
                self.task_descriptor.description
            ),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    /// Generate a test plan based on task characteristics
    fn generate_test_plan(&self) -> TestPlan {
        let risk_tier = self
            .task_descriptor
            .risk_tier
            .clone()
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
                        user_journey: "Complete user journey from registration to task completion"
                            .to_string(),
                        expected_outcomes: vec![],
                    },
                    E2eScenario {
                        description: "failure-recovery".to_string(),
                        user_journey: "User journey with failure scenarios and recovery"
                            .to_string(),
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
                integration_tests: vec![IntegrationTestSpec {
                    description: "api-integration-tests".to_string(),
                    components: vec![],
                    test_cases: vec![],
                }],
                e2e_scenarios: vec![E2eScenario {
                    description: "happy-path-scenario".to_string(),
                    user_journey: "Happy path user journey".to_string(),
                    expected_outcomes: vec![],
                }],
                coverage_targets: Some(CoverageTargets {
                    line_coverage: Some(0.8),
                    branch_coverage: Some(0.9),
                    mutation_score: Some(0.5),
                }),
            },
            RiskTier::Tier3 => TestPlan {
                unit_tests: vec![UnitTestSpec {
                    description: "basic-functionality-tests".to_string(),
                    target_function: None,
                    test_cases: vec![],
                }],
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
    #[allow(dead_code)] // Reserved for future use
    db_ops: Arc<dyn DatabaseOperations>,
}

impl RealTaskDescriptorProvider {
    fn new(task_descriptor: TaskDescriptor, db_ops: Arc<dyn DatabaseOperations>) -> Self {
        Self {
            task_descriptor,
            db_ops,
        }
    }

    /// Enhance the task descriptor with additional information from database or external sources
    async fn enhance_task_descriptor(
        &self,
    ) -> Result<agent_agency_contracts::types::planning::TaskDescriptor> {
        let mut enhanced = self.task_descriptor.clone();

        // Query execution plans for similar tasks (by description similarity or scope overlap)
        let all_plans = match self.db_ops.get_execution_plans().await {
            Ok(plans) => plans,
            Err(e) => {
                tracing::warn!(
                    task_id = %enhanced.task_id,
                    error = %e,
                    "Failed to query execution plans for enhancement, continuing without historical data"
                );
                return Ok(enhanced); // Return unenhanced descriptor on error
            }
        };

        // Find similar plans based on description keywords or scope overlap
        let similar_plans: Vec<_> = all_plans
            .iter()
            .filter(|plan| {
                // Check if plan title or overview contains keywords from task description
                let desc_lower = enhanced.description.to_lowercase();
                let title_lower = plan.title.to_lowercase();
                let overview_lower = plan
                    .overview
                    .as_ref()
                    .map(|o| o.to_lowercase())
                    .unwrap_or_default();

                // Simple keyword matching (could be enhanced with more sophisticated similarity)
                desc_lower.split_whitespace().any(|word| {
                    word.len() > 3 && (title_lower.contains(word) || overview_lower.contains(word))
                })
            })
            .take(5) // Limit to 5 most similar plans
            .collect();

        // Query execution results for historical performance data
        let mut historical_avg_duration_ms = 0u64;
        let mut historical_success_rate = 0.0f64;
        let mut similar_results_count = 0usize;

        for plan in &similar_plans {
            if let Ok(Some(result)) = self.db_ops.get_execution_result(plan.id).await {
                historical_avg_duration_ms += result.total_duration_ms as u64;
                if result.success {
                    similar_results_count += 1;
                }
            }
        }

        if !similar_plans.is_empty() {
            historical_avg_duration_ms /= similar_plans.len() as u64;
            historical_success_rate = similar_results_count as f64 / similar_plans.len() as f64;

            tracing::debug!(
                task_id = %enhanced.task_id,
                similar_plans_count = similar_plans.len(),
                historical_avg_duration_ms = historical_avg_duration_ms,
                historical_success_rate = %historical_success_rate,
                "Found similar historical plans for task descriptor enhancement"
            );
        }

        // Validate task requirements against available resources
        // Check if change budget is reasonable based on historical data
        if historical_avg_duration_ms > 0 {
            // Estimate if change budget is reasonable (heuristic: if historical avg is much higher than budget, warn)
            let _budget_max_files = enhanced.change_budget.max_files;
            let _budget_max_loc = enhanced.change_budget.max_loc;

            // Simple validation: if we have historical data suggesting longer execution, adjust expectations
            // This is a heuristic - actual validation would require more sophisticated analysis
            if historical_avg_duration_ms > 300_000 {
                // 5 minutes
                tracing::debug!(
                    task_id = %enhanced.task_id,
                    historical_avg_duration_ms = historical_avg_duration_ms,
                    "Historical data suggests longer execution time for similar tasks"
                );
            }
        }

        // Enrich descriptor with historical metadata
        // Note: TaskDescriptor doesn't have a metadata field, so we can't directly add metadata
        // Instead, we can enhance the description or use the acceptance criteria field

        // Enhance acceptance criteria with historical insights if available
        if historical_success_rate > 0.0 && !enhanced.acceptance.is_some() {
            let mut acceptance_criteria = format!(
                "Task completion based on historical success rate: {:.1}%",
                historical_success_rate * 100.0
            );

            if historical_avg_duration_ms > 0 {
                acceptance_criteria.push_str(&format!(
                    "\nEstimated duration based on similar tasks: {}ms",
                    historical_avg_duration_ms
                ));
            }

            enhanced.acceptance = Some(acceptance_criteria);
        }

        // Validate risk tier if not set, infer from historical data
        if enhanced.risk_tier.is_none() {
            // Infer risk tier from historical success rate
            // Lower success rate suggests higher risk (Tier1 = highest risk)
            let inferred_risk_tier =
                if historical_success_rate > 0.0 && historical_success_rate < 0.7 {
                    agent_agency_contracts::types::planning::RiskTier::Tier1 // High risk
                } else if historical_success_rate >= 0.7 && historical_success_rate < 0.9 {
                    agent_agency_contracts::types::planning::RiskTier::Tier2 // Medium risk
                } else {
                    agent_agency_contracts::types::planning::RiskTier::Tier3 // Low risk
                };

            enhanced.risk_tier = Some(inferred_risk_tier.clone());

            tracing::debug!(
                task_id = %enhanced.task_id,
                inferred_risk_tier = ?inferred_risk_tier,
                historical_success_rate = %historical_success_rate,
                "Inferred risk tier from historical data"
            );
        }

        // Check for conflicts with other tasks (tasks with overlapping scope)
        let conflicting_plans: Vec<_> = all_plans
            .iter()
            .filter(|plan| {
                // Check if plan is in progress and has overlapping scope
                plan.state == "InProgress" || plan.state == "Approved"
            })
            .filter(|plan| {
                // Simple overlap check: if titles share significant keywords
                let plan_title_lower = plan.title.to_lowercase();
                let task_desc_lower = enhanced.description.to_lowercase();

                // Count shared significant words (length > 3)
                let task_words: std::collections::HashSet<&str> = task_desc_lower
                    .split_whitespace()
                    .filter(|w| w.len() > 3)
                    .collect();

                let plan_words: std::collections::HashSet<&str> = plan_title_lower
                    .split_whitespace()
                    .filter(|w| w.len() > 3)
                    .collect();

                let overlap = task_words.intersection(&plan_words).count();
                overlap >= 2 // At least 2 shared significant words suggests potential conflict
            })
            .collect();

        if !conflicting_plans.is_empty() {
            tracing::warn!(
                task_id = %enhanced.task_id,
                conflicting_plans_count = conflicting_plans.len(),
                "Found potentially conflicting tasks in progress"
            );
        }

        tracing::debug!(
            task_id = %enhanced.task_id,
            similar_plans_found = similar_plans.len(),
            conflicting_plans_found = conflicting_plans.len(),
            historical_avg_duration_ms = historical_avg_duration_ms,
            historical_success_rate = %historical_success_rate,
            "Enhanced task descriptor with database context and historical data"
        );

        Ok(enhanced)
    }
}

#[async_trait::async_trait]
impl crate::planning::plan_types::TaskDescriptorProvider for RealTaskDescriptorProvider {
    async fn get_task_descriptor(
        &self,
    ) -> Result<agent_agency_contracts::types::planning::TaskDescriptor> {
        self.enhance_task_descriptor().await
    }
}
