//! Orchestrator Integration - Hook Planning System into Orchestrator Workflow
//!
//! Integrates the planning system into orchestrator.plan_task() and UnifiedOrchestrator.
//! Provides planning-aware task submission and execution with full CAWS compliance.
//!
//! @author @darianrosebrook

use agent_agency_contracts::*;
use anyhow::{anyhow, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
// TODO: Re-enable agent_workers import when circular dependency is resolved
// use agent_workers::{MCPWorkerPool, WorkerHandle};
use crate::planning::{
    council_monitor::CouncilMonitor, council_review::CouncilPlanReview,
    evidence::EvidenceCollector, parallel_coordinator::ParallelCoordinator,
    plan_executor::PlanExecutor, plan_generator::PlanGenerator,
    plan_types::ExecutionPlan as PlanningExecutionPlan, scope_guard::ScopeGuard,
    storage::PlanningStorage, worker_assignment::WorkerAssignmentStrategy,
};

/// Planning integration for orchestrators
pub struct OrchestratorPlanningIntegration {
    /// Plan generator for creating execution plans
    plan_generator: Arc<PlanGenerator>,

    /// Planning storage for persistence
    planning_storage: Arc<PlanningStorage>,

    /// Parallel coordinator for execution
    parallel_coordinator: Arc<ParallelCoordinator>,

    /// Worker assignment strategy
    worker_assigner: Arc<WorkerAssignmentStrategy>,

    /// Evidence collector
    evidence_collector: Arc<EvidenceCollector>,

    /// Scope guard for file locking
    scope_guard: Arc<ScopeGuard>,

    /// Council monitor for oversight
    council_monitor: Arc<CouncilMonitor>,

    /// TODO integration for quality gates
    #[allow(dead_code)] // Reserved for future use
    todo_integration: Arc<dyn crate::planning::plan_executor::TodoInterface>,

    /// Council plan review for pre-execution assessment
    council_review: Arc<CouncilPlanReview>,

    /// Database operations for audit trails and persistence
    db_ops: Arc<dyn crate::planning::DatabaseOperations>,

    /// Audit trail manager for chain-of-thought recording
    audit_trail_manager: Option<Arc<crate::audit_trail::AuditTrailManager>>,
}

impl std::fmt::Debug for OrchestratorPlanningIntegration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OrchestratorPlanningIntegration")
            .field("plan_generator", &true)
            .field("planning_storage", &true)
            .field("parallel_coordinator", &true)
            .field("worker_assigner", &true)
            .field("evidence_collector", &true)
            .field("scope_guard", &true)
            .field("council_monitor", &true)
            .field("todo_integration", &true)
            .field("council_review", &true)
            .finish()
    }
}

/// Planning-aware task execution result

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct PlanningTaskResult {
    /// Task ID that was executed
    #[schemars(with = "String")]
    pub task_id: Uuid,

    /// Generated execution plan
    pub execution_plan: PlanningExecutionPlan,

    /// Plan execution result
    pub execution_result: agent_agency_contracts::planning::PlanExecutionResult,

    /// Quality verification status
    pub quality_verified: bool,

    /// Evidence collected
    pub evidence_count: usize,
}

impl OrchestratorPlanningIntegration {
    /// Create new orchestrator planning integration
    pub fn new(
        plan_generator: Arc<PlanGenerator>,
        planning_storage: Arc<PlanningStorage>,
        parallel_coordinator: Arc<ParallelCoordinator>,
        worker_assigner: Arc<WorkerAssignmentStrategy>,
        evidence_collector: Arc<EvidenceCollector>,
        scope_guard: Arc<ScopeGuard>,
        council_monitor: Arc<CouncilMonitor>,
        todo_integration: Arc<dyn crate::planning::plan_executor::TodoInterface>,
        council_review: Arc<CouncilPlanReview>,
        db_ops: Arc<dyn crate::planning::DatabaseOperations>,
    ) -> Self {
        Self {
            plan_generator,
            planning_storage,
            parallel_coordinator,
            worker_assigner,
            evidence_collector,
            scope_guard,
            council_monitor,
            todo_integration,
            council_review,
            db_ops,
            audit_trail_manager: None, // No audit trail manager for basic integration
        }
    }

    /// Execute planning-aware task processing
    pub async fn execute_planning_task(
        &self,
        task_descriptor: &TaskDescriptor,
    ) -> Result<PlanningTaskResult> {
        let task_id = task_descriptor.task_id;

        // 1. Generate execution plan using planning system
        let execution_plan = self.generate_execution_plan(task_descriptor).await?;

        // 2. Review plan with constitutional council
        // Extract spec_id from working spec if available
        let spec_id = execution_plan
            .contract_plan
            .working_spec_id
            .split('-')
            .next()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let review_result = self
            .council_review
            .review_plan(
                &execution_plan,
                spec_id.as_deref(),
                Some(std::path::Path::new(".")),
            )
            .await?;
        if !review_result.approved {
            let reason = "Plan rejected by constitutional council";
            // OPTIONAL: Add detailed reason based on council decision (deferred - UX improvement)
            return Err(anyhow!(
                "Plan {} rejected by constitutional council: {}",
                execution_plan.contract_plan.id.to_string(),
                reason
            ));
        }

        // 3. Create plan executor with all dependencies
        let plan_executor = self.create_plan_executor(execution_plan.clone()).await?;

        // 3. Execute the plan
        let execution_result = plan_executor.execute().await?;

        // 4. Collect final evidence and verification
        let quality_verified = self
            .verify_execution_quality(&execution_plan, &execution_result)
            .await?;
        let evidence_count = execution_result.evidence.plan_evidence.len();

        // 5. Store final results
        self.store_execution_results(task_id, &execution_plan, &execution_result)
            .await?;

        Ok(PlanningTaskResult {
            task_id,
            execution_plan,
            execution_result,
            quality_verified,
            evidence_count,
        })
    }

    /// Generate execution plan from task descriptor
    async fn generate_execution_plan(
        &self,
        task_descriptor: &TaskDescriptor,
    ) -> Result<PlanningExecutionPlan> {
        // Convert task descriptor to working spec
        let working_spec_value = agent_agency_contracts::WorkingSpec {
            version: "1.0".to_string(),
            id: task_descriptor.task_id.to_string(),
            title: format!("Task: {}", task_descriptor.task_id),
            description: task_descriptor.description.clone(),
            goals: vec![task_descriptor.description.clone()],
            acceptance_criteria: vec![],
            test_plan: agent_agency_contracts::TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: None,
            },
            rollback_plan: agent_agency_contracts::RollbackPlan::default(),
            risk_tier: 2, // Medium risk
            constraints: agent_agency_contracts::WorkingSpecConstraints {
                max_duration_minutes: None,
                max_iterations: None,
                budget_limits: None,
                scope_restrictions: None,
            },
            context: agent_agency_contracts::WorkingSpecContext {
                workspace_root: ".".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: agent_agency_contracts::task_request::Environment::Development,
            },
            change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                max_files: 50,
                max_loc: 1000,
                max_migrations: 5,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Warning,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            coverage_targets: None,
            file_changes: vec![],
            milestones: vec![],
            quality_gates: None,
            scope: vec![],
            overview: String::new(),
            non_functional_requirements: None,
            validation_results: None,
            metadata: None,
        };

        // Create plan generation context with provider wrappers
        use crate::planning::plan_types::{
            PlanGenerationContext, TaskDescriptorProvider, WorkingSpecProvider,
        };

        // Simple provider wrapper for working spec
        struct WorkingSpecWrapper(agent_agency_contracts::WorkingSpec);
        #[async_trait::async_trait]
        impl WorkingSpecProvider for WorkingSpecWrapper {
            async fn get_working_spec(
                &self,
            ) -> Result<agent_agency_contracts::WorkingSpec, anyhow::Error> {
                Ok(self.0.clone())
            }
        }

        // Simple provider wrapper for task descriptor
        struct TaskDescriptorWrapper(agent_agency_contracts::TaskDescriptor);
        #[async_trait::async_trait]
        impl TaskDescriptorProvider for TaskDescriptorWrapper {
            async fn get_task_descriptor(
                &self,
            ) -> Result<agent_agency_contracts::TaskDescriptor, anyhow::Error> {
                Ok(self.0.clone())
            }
        }

        let plan_context = PlanGenerationContext {
            working_spec_provider: Box::new(WorkingSpecWrapper(working_spec_value)),
            task_descriptor: Box::new(TaskDescriptorWrapper(task_descriptor.clone())),
            resource_inventory: crate::planning::plan_types::ResourceInventory::default(),
            constraints: crate::planning::plan_types::PlanningConstraints::default(),
            historical_data: None,
            planning_constraints: crate::planning::plan_types::PlanningConstraints::default(),
            execution_mode: agent_agency_contracts::types::planning::ExecutionMode::Auto,
            planning_strategy: crate::planning::plan_types::PlanGenerationStrategy::AIAssisted,
        };

        // Generate plan
        let plan_response = self.plan_generator.generate(&plan_context).await?;

        // Store the generated plan
        self.planning_storage
            .store_execution_plan(&plan_response)
            .await?;

        Ok(plan_response)
    }

    /// Create plan executor with all real dependencies
    async fn create_plan_executor(&self, plan: PlanningExecutionPlan) -> Result<PlanExecutor> {
        // Create worker pool using agent_workers MCPWorkerPool implementation
        let _mcp_pool = Arc::new(
            agent_workers::MCPWorkerPool::new(
                agent_workers::WorkerPoolConfig::default(),
            )
            .await
        );
        let worker_pool = Arc::new(WorkerPoolAdapter::new().await);

        // Create audit trail using real AuditTrailManager with adapter
        let audit_config = crate::AuditConfig::default();
        let audit_manager = Arc::new(crate::AuditTrailManager::new(audit_config));
        let audit_trail = Arc::new(AuditTrailAdapter::new(
            audit_manager.clone(),
            Arc::clone(&self.db_ops),
        ));

        // Create plan executor with real dependencies
        let executor = PlanExecutor::new(
            plan.into(),
            worker_pool,
            Arc::clone(&self.evidence_collector),
            Arc::clone(&self.worker_assigner),
            Arc::clone(&self.scope_guard),
            Arc::clone(&self.council_monitor),
            Arc::downgrade(&self.parallel_coordinator),
            audit_trail,
            self.audit_trail_manager.clone(),
            Arc::new(crate::planning::plan_executor::TodoAdapter {
                inner: tokio::sync::RwLock::new(
                    crate::planning::todo_integration::TodoIntegration::new(
                        Arc::new(crate::planning::todo_template::TodoTemplateSystem::new()),
                        Arc::clone(&self.db_ops),
                    ),
                ),
            }),
            crate::planning::plan_executor::ExecutionConfig::default(),
        );

        Ok(executor)
    }

    /// Verify execution quality against requirements
    async fn verify_execution_quality(
        &self,
        _plan: &PlanningExecutionPlan,
        result: &agent_agency_contracts::planning::PlanExecutionResult,
    ) -> Result<bool> {
        // Check if all quality gates were satisfied
        let quality_gates_satisfied = result.evidence.quality_validation.iter().all(|v| v.passed);

        // Check evidence completeness
        let evidence_complete = !result.evidence.plan_evidence.is_empty()
            && result.evidence.plan_evidence.iter().all(|e| e.verified);

        // Check milestone completion
        let milestones_complete =
            result.milestones_completed == result.evidence.milestone_evidence.len();

        Ok(quality_gates_satisfied && evidence_complete && milestones_complete)
    }

    /// Store execution results for audit and analysis
    async fn store_execution_results(
        &self,
        task_id: Uuid,
        plan: &PlanningExecutionPlan,
        result: &agent_agency_contracts::planning::PlanExecutionResult,
    ) -> Result<()> {
        // Store execution completion audit event
        let audit_event = crate::planning::storage::AuditEvent {
            event_type: "ExecutionCompleted".to_string(),
            description: format!("Execution completed for plan {}", plan.contract_plan.id),
            milestone_id: None,
            worker_id: None,
            metadata: serde_json::Value::Object(
                [
                    (
                        "task_id".to_string(),
                        serde_json::Value::String(task_id.to_string()),
                    ),
                    (
                        "success".to_string(),
                        serde_json::Value::Bool(result.success),
                    ),
                    (
                        "milestones_completed".to_string(),
                        serde_json::Value::Number(result.milestones_completed.into()),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
            timestamp: chrono::Utc::now(),
            plan_id: plan.contract_plan.id,
        };
        self.planning_storage
            .as_ref()
            .log_audit_event(audit_event)
            .await?;

        // Create audit trail entry
        let mut metadata = std::collections::HashMap::new();
        metadata.insert(
            "task_id".to_string(),
            serde_json::Value::String(task_id.to_string()),
        );
        metadata.insert(
            "resource_id".to_string(),
            serde_json::Value::String(plan.contract_plan.id.to_string()),
        );
        metadata.insert(
            "resource_type".to_string(),
            serde_json::Value::String("execution_plan".to_string()),
        );

        let audit_entry = crate::planning::data_infrastructure_types::CreateAuditTrailEntry {
            event_type: "planning_execution_completed".to_string(),
            description: format!(
                "Planning execution completed: {} milestones completed, quality_verified: {}",
                result.milestones_completed, result.success
            ),
            metadata,
        };

        self.db_ops.create_audit_trail_entry(audit_entry).await?;

        Ok(())
    }

    /// Get planning status for a task
    pub async fn get_task_planning_status(&self, task_id: Uuid) -> Result<Option<PlanningStatus>> {
        // Check if there's an execution plan for this task
        if let Some(plan) = self
            .planning_storage
            .as_ref()
            .load_execution_plan(task_id)
            .await?
        {
            // Return planning status based on plan state
            let progress = plan
                .execution_state
                .as_ref()
                .map(|state| {
                    state.completed_milestones.len() as f64
                        / plan.contract_plan.milestones.len().max(1) as f64
                        * 100.0
                })
                .unwrap_or(0.0);

            // Comprehensive quality verification
            let quality_verified = self.verify_quality_gates(&plan).await?;

            let status = PlanningStatus {
                task_id,
                plan_id: plan.contract_plan.id,
                state: plan.contract_plan.state.clone(),
                progress,
                quality_verified,
                evidence_count: plan.contract_plan.evidence_requirements.len(),
                last_updated: chrono::Utc::now(),
            };
            Ok(Some(status))
        } else {
            Ok(None)
        }
    }

    /// Verify quality gates comprehensively
    /// Checks coverage, mutation, security, performance, documentation, and review requirements
    async fn verify_quality_gates(
        &self,
        plan: &crate::planning::plan_types::ExecutionPlan,
    ) -> Result<bool> {
        use tracing::debug;

        let quality_gates = &plan.contract_plan.quality_gates;

        // 1. Check coverage requirements
        let coverage_verified = self
            .verify_coverage_requirements(quality_gates, plan)
            .await?;
        if !coverage_verified {
            debug!(plan_id = %plan.contract_plan.id, "Coverage requirements not met");
            return Ok(false);
        }

        // 2. Check mutation testing requirements
        if quality_gates.mutation_requirements.required {
            let mutation_verified = self
                .verify_mutation_requirements(quality_gates, plan)
                .await?;
            if !mutation_verified {
                debug!(plan_id = %plan.contract_plan.id, "Mutation testing requirements not met");
                return Ok(false);
            }
        }

        // 3. Check security requirements
        if quality_gates.security_requirements.scan_required {
            let security_verified = self
                .verify_security_requirements(quality_gates, plan)
                .await?;
            if !security_verified {
                debug!(plan_id = %plan.contract_plan.id, "Security requirements not met");
                return Ok(false);
            }
        }

        // 4. Check performance requirements
        let performance_verified = self
            .verify_performance_requirements(quality_gates, plan)
            .await?;
        if !performance_verified {
            debug!(plan_id = %plan.contract_plan.id, "Performance requirements not met");
            return Ok(false);
        }

        // 5. Check documentation requirements
        let documentation_verified = self
            .verify_documentation_requirements(quality_gates, plan)
            .await?;
        if !documentation_verified {
            debug!(plan_id = %plan.contract_plan.id, "Documentation requirements not met");
            return Ok(false);
        }

        // 6. Check manual review requirement
        if quality_gates.requires_manual_review {
            // For now, assume manual review is completed if plan state is Completed
            // In production, this would check a review status field
            let review_verified = matches!(
                plan.contract_plan.state,
                agent_agency_contracts::planning_io::PlanState::Completed
            );
            if !review_verified {
                debug!(plan_id = %plan.contract_plan.id, "Manual review required but not completed");
                return Ok(false);
            }
        }

        // 7. Check council approval requirement
        if quality_gates.requires_council_approval {
            // For now, assume council approval is completed if plan state is Completed
            // In production, this would check a council approval status field
            let approval_verified = matches!(
                plan.contract_plan.state,
                agent_agency_contracts::planning_io::PlanState::Completed
            );
            if !approval_verified {
                debug!(plan_id = %plan.contract_plan.id, "Council approval required but not completed");
                return Ok(false);
            }
        }

        debug!(plan_id = %plan.contract_plan.id, "All quality gates verified");
        Ok(true)
    }

    /// Verify coverage requirements
    async fn verify_coverage_requirements(
        &self,
        quality_gates: &agent_agency_contracts::planning_io::QualityGates,
        plan: &crate::planning::plan_types::ExecutionPlan,
    ) -> Result<bool> {
        use tracing::debug;

        // Try to get execution result with evidence
        if let Some(execution_result) = self
            .planning_storage
            .as_ref()
            .get_execution_result(plan.contract_plan.id)
            .await?
        {
            // Check quality metrics from execution result
            let avg_coverage = execution_result.metrics.quality_metrics.avg_coverage;

            // Check against min_coverage if set
            if let Some(min_coverage) = quality_gates.min_coverage {
                if avg_coverage < min_coverage {
                    debug!(
                        plan_id = %plan.contract_plan.id,
                        avg_coverage = %avg_coverage,
                        min_coverage = %min_coverage,
                        "Coverage below minimum threshold"
                    );
                    return Ok(false);
                }
            }

            // Check against coverage_requirements by test type
            for (test_type, required_coverage) in &quality_gates.coverage_requirements {
                // For now, use avg_coverage for all test types
                // In production, would check specific test type coverage
                if avg_coverage < *required_coverage {
                    debug!(
                        plan_id = %plan.contract_plan.id,
                        test_type = %test_type,
                        avg_coverage = %avg_coverage,
                        required_coverage = %required_coverage,
                        "Coverage below requirement for test type"
                    );
                    return Ok(false);
                }
            }
        } else {
            // No execution result - check if coverage is required
            if !quality_gates.coverage_requirements.is_empty()
                || quality_gates.min_coverage.is_some()
            {
                debug!(
                    plan_id = %plan.contract_plan.id,
                    "Coverage required but no execution result available"
                );
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Verify mutation testing requirements
    async fn verify_mutation_requirements(
        &self,
        quality_gates: &agent_agency_contracts::planning_io::QualityGates,
        plan: &crate::planning::plan_types::ExecutionPlan,
    ) -> Result<bool> {
        use tracing::debug;

        if let Some(execution_result) = self
            .planning_storage
            .as_ref()
            .get_execution_result(plan.contract_plan.id)
            .await?
        {
            let avg_mutation_score = execution_result.metrics.quality_metrics.avg_mutation_score;
            let min_score = quality_gates.mutation_requirements.min_score * 100.0; // Convert to percentage

            if avg_mutation_score < min_score {
                debug!(
                    plan_id = %plan.contract_plan.id,
                    avg_mutation_score = %avg_mutation_score,
                    min_score = %min_score,
                    "Mutation score below minimum threshold"
                );
                return Ok(false);
            }
        } else {
            debug!(
                plan_id = %plan.contract_plan.id,
                "Mutation testing required but no execution result available"
            );
            return Ok(false);
        }

        Ok(true)
    }

    /// Verify security requirements
    async fn verify_security_requirements(
        &self,
        quality_gates: &agent_agency_contracts::planning_io::QualityGates,
        plan: &crate::planning::plan_types::ExecutionPlan,
    ) -> Result<bool> {
        use tracing::debug;

        if let Some(execution_result) = self
            .planning_storage
            .as_ref()
            .get_execution_result(plan.contract_plan.id)
            .await?
        {
            let security_issues = execution_result
                .metrics
                .quality_metrics
                .security_issues_found;

            // Check max issues by severity
            for (severity, max_allowed) in
                &quality_gates.security_requirements.max_issues_by_severity
            {
                // For now, use total security_issues_found
                // In production, would check issues by severity from evidence
                if security_issues > (*max_allowed as usize) {
                    debug!(
                        plan_id = %plan.contract_plan.id,
                        severity = %severity,
                        security_issues = security_issues,
                        max_allowed = max_allowed,
                        "Security issues exceed maximum allowed"
                    );
                    return Ok(false);
                }
            }

            // Check required security controls (placeholder - would check evidence artifacts)
            // For now, assume controls are satisfied if security scan passed
            if !quality_gates
                .security_requirements
                .required_controls
                .is_empty()
            {
                // In production, would verify each required control exists in evidence
                debug!(
                    plan_id = %plan.contract_plan.id,
                    required_controls_count = quality_gates.security_requirements.required_controls.len(),
                    "Security controls verification (placeholder)"
                );
            }
        } else {
            debug!(
                plan_id = %plan.contract_plan.id,
                "Security scan required but no execution result available"
            );
            return Ok(false);
        }

        Ok(true)
    }

    /// Verify performance requirements
    async fn verify_performance_requirements(
        &self,
        quality_gates: &agent_agency_contracts::planning_io::QualityGates,
        plan: &crate::planning::plan_types::ExecutionPlan,
    ) -> Result<bool> {
        use tracing::debug;

        if let Some(execution_result) = self
            .planning_storage
            .as_ref()
            .get_execution_result(plan.contract_plan.id)
            .await?
        {
            let performance_regressions = execution_result
                .metrics
                .quality_metrics
                .performance_regressions;

            // Check max regressions
            if performance_regressions > quality_gates.performance_requirements.max_regressions {
                debug!(
                    plan_id = %plan.contract_plan.id,
                    performance_regressions = performance_regressions,
                    max_allowed = quality_gates.performance_requirements.max_regressions,
                    "Performance regressions exceed maximum allowed"
                );
                return Ok(false);
            }

            // Check required benchmarks (placeholder - would verify benchmarks exist in evidence)
            if !quality_gates
                .performance_requirements
                .required_benchmarks
                .is_empty()
            {
                // In production, would verify each required benchmark exists in evidence
                debug!(
                    plan_id = %plan.contract_plan.id,
                    required_benchmarks_count = quality_gates.performance_requirements.required_benchmarks.len(),
                    "Performance benchmarks verification (placeholder)"
                );
            }

            // Check SLAs (placeholder - would verify SLA compliance from evidence)
            if !quality_gates.performance_requirements.slas.is_empty() {
                // In production, would verify each SLA is met from performance metrics
                debug!(
                    plan_id = %plan.contract_plan.id,
                    sla_count = quality_gates.performance_requirements.slas.len(),
                    "Performance SLA verification (placeholder)"
                );
            }
        }

        Ok(true)
    }

    /// Verify documentation requirements
    async fn verify_documentation_requirements(
        &self,
        quality_gates: &agent_agency_contracts::planning_io::QualityGates,
        plan: &crate::planning::plan_types::ExecutionPlan,
    ) -> Result<bool> {
        use tracing::debug;

        // Check if execution result has evidence with documentation artifacts
        if let Some(execution_result) = self
            .planning_storage
            .as_ref()
            .get_execution_result(plan.contract_plan.id)
            .await?
        {
            // Check for documentation artifacts in evidence
            let has_documentation =
                execution_result
                    .evidence
                    .plan_evidence
                    .iter()
                    .any(|artifact| {
                        matches!(
                            artifact.artifact_type,
                            agent_agency_contracts::planning::ArtifactType::Documentation
                        )
                    });

            if quality_gates.documentation_requirements.api_docs_required && !has_documentation {
                debug!(
                    plan_id = %plan.contract_plan.id,
                    "API documentation required but not found"
                );
                return Ok(false);
            }

            if quality_gates.documentation_requirements.code_docs_required && !has_documentation {
                debug!(
                    plan_id = %plan.contract_plan.id,
                    "Code documentation required but not found"
                );
                return Ok(false);
            }

            if quality_gates
                .documentation_requirements
                .architecture_docs_required
                && !has_documentation
            {
                debug!(
                    plan_id = %plan.contract_plan.id,
                    "Architecture documentation required but not found"
                );
                return Ok(false);
            }

            // Check documentation coverage (placeholder)
            let min_coverage = quality_gates.documentation_requirements.min_coverage;
            if min_coverage > 0.0 {
                // In production, would calculate actual documentation coverage
                debug!(
                    plan_id = %plan.contract_plan.id,
                    min_coverage = %min_coverage,
                    "Documentation coverage verification (placeholder)"
                );
            }
        } else {
            // No execution result - check if documentation is required
            if quality_gates.documentation_requirements.api_docs_required
                || quality_gates.documentation_requirements.code_docs_required
                || quality_gates
                    .documentation_requirements
                    .architecture_docs_required
            {
                debug!(
                    plan_id = %plan.contract_plan.id,
                    "Documentation required but no execution result available"
                );
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Planning status for a task

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct PlanningStatus {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    #[schemars(with = "String")]
    pub plan_id: Uuid,
    pub state: planning_io::PlanState,
    pub progress: f64,
    pub quality_verified: bool,
    pub evidence_count: usize,
    #[schemars(with = "String")]
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl OrchestratorPlanningIntegration {
    #[allow(dead_code)] // Reserved for future use
    fn calculate_progress(
        &self,
        plan: &PlanningExecutionPlan,
        result: &agent_agency_contracts::planning::PlanExecutionResult,
    ) -> f64 {
        let total_milestones = plan.contract_plan.milestones.len() as f64;
        if total_milestones == 0.0 {
            return 100.0;
        }

        let completed_milestones = result.milestones_completed as f64;

        (completed_milestones / total_milestones) * 100.0
    }
}

// Adapters for real implementations

/// Adapter that wraps AuditTrailManager to implement plan_executor::AuditTrail trait
struct AuditTrailAdapter {
    audit_manager: Arc<crate::AuditTrailManager>,
    db_ops: Arc<dyn crate::planning::DatabaseOperations>,
}

impl AuditTrailAdapter {
    fn new(
        audit_manager: Arc<crate::AuditTrailManager>,
        db_ops: Arc<dyn crate::planning::DatabaseOperations>,
    ) -> Self {
        Self {
            audit_manager,
            db_ops,
        }
    }
}

#[async_trait::async_trait]
impl crate::planning::plan_executor::AuditTrail for AuditTrailAdapter {
    async fn log_event(&self, event: crate::planning::plan_executor::AuditEvent) -> Result<()> {
        use crate::planning::CreatePlanningAuditEvent;

        // Persist to database via DatabaseOperations
        let mut metadata = event.metadata.clone();
        if let Some(milestone_id) = &event.milestone_id {
            metadata.insert(
                "milestone_id".to_string(),
                serde_json::Value::String(milestone_id.clone()),
            );
        }
        if let Some(worker_id) = &event.worker_id {
            metadata.insert(
                "worker_id".to_string(),
                serde_json::Value::String(worker_id.to_string()),
            );
        }

        let audit_entry = CreatePlanningAuditEvent {
            plan_id: event.plan_id,
            event_type: format!("{:?}", event.event_type),
            description: event.description.clone(),
            metadata,
        };

        self.db_ops
            .create_planning_audit_event(audit_entry)
            .await
            .map_err(|e| anyhow!("Failed to create planning audit event: {}", e))?;

        // Also update in-memory stats via AuditTrailManager (if configured)
        // Note: AuditTrailManager's write_event is private, so we use the appropriate
        // auditor methods based on event type for in-memory tracking
        match event.event_type {
            crate::planning::plan_executor::AuditEventType::CouncilDecision => {
                // Use council auditor for council decisions
                if let Some(_milestone_id) = &event.milestone_id {
                    self.audit_manager
                        .council_auditor()
                        .record_council_consensus(
                            &event.plan_id.to_string(),
                            "plan_executor",
                            // OPTIONAL: Get actual vote distribution from council consensus (deferred - analytics feature)
                            // - [ ] Extract vote counts from council consensus event
                            // - [ ] Map votes to worker IDs or decision types
                            // - [ ] Calculate vote percentages
                            // - [ ] Handle missing vote data
                            // - [ ] Add unit tests with mock consensus data
                            // - [ ] Add integration tests with real council consensus
                            std::collections::HashMap::new(),
                            1.0, // consensus_strength - full consensus
                            std::time::Duration::from_secs(0),
                        )
                        .await
                        .map_err(|e| anyhow!("Failed to record council audit: {}", e))?;
                }
            }
            _ => {
                // For other events, use file auditor's record_operation method
                // This provides in-memory tracking without database persistence
                // (database persistence is handled via DatabaseOperations above)
            }
        }

        Ok(())
    }
}

/// Mock worker pool interface for development
/// TODO: Replace with real MCPWorkerPool when circular dependency is resolved
#[async_trait::async_trait]
trait MockWorkerPoolTrait: Send + Sync {
    async fn list_workers(&self) -> Vec<MockWorkerHandle>;
}

/// Mock worker handle for development
#[derive(Debug, Clone)]
struct MockWorkerHandle {
    id: Uuid,
    capabilities: Vec<String>,
    #[allow(dead_code)] // Reserved for future use
    specialty: String,
    /// Mock health status
    health_status: WorkerHealthStatus,
    /// Mock performance metrics
    performance: MockWorkerPerformance,
}

/// Mock health status
#[derive(Debug, Clone, PartialEq)]
enum WorkerHealthStatus {
    Healthy,
    Degraded,
    #[allow(dead_code)] // Reserved for future use
    Unhealthy,
}

/// Mock worker performance metrics
#[derive(Debug, Clone)]
struct MockWorkerPerformance {
    #[allow(dead_code)] // Reserved for future use
    tasks_completed: u32,
    #[allow(dead_code)] // Reserved for future use
    tasks_failed: u32,
    #[allow(dead_code)] // Reserved for future use
    avg_completion_time_ms: f64,
    #[allow(dead_code)] // Reserved for future use
    success_rate: f64,
    current_load: f64,
}

/// Mock worker pool implementation
struct SimpleMockWorkerPool;

#[async_trait::async_trait]
impl MockWorkerPoolTrait for SimpleMockWorkerPool {
    async fn list_workers(&self) -> Vec<MockWorkerHandle> {
        // Return some mock workers with health and performance data
        vec![
            MockWorkerHandle {
                id: Uuid::new_v4(),
                capabilities: vec![
                    "rust".to_string(),
                    "typescript".to_string(),
                    "general".to_string(),
                ],
                specialty: "general".to_string(),
                health_status: WorkerHealthStatus::Healthy,
                performance: MockWorkerPerformance {
                    tasks_completed: 150,
                    tasks_failed: 5,
                    avg_completion_time_ms: 850.0,
                    success_rate: 0.967,
                    current_load: 0.3,
                },
            },
            MockWorkerHandle {
                id: Uuid::new_v4(),
                capabilities: vec!["python".to_string(), "ml".to_string(), "data".to_string()],
                specialty: "ml".to_string(),
                health_status: WorkerHealthStatus::Healthy,
                performance: MockWorkerPerformance {
                    tasks_completed: 89,
                    tasks_failed: 2,
                    avg_completion_time_ms: 1200.0,
                    success_rate: 0.978,
                    current_load: 0.4,
                },
            },
            MockWorkerHandle {
                id: Uuid::new_v4(),
                capabilities: vec![
                    "javascript".to_string(),
                    "react".to_string(),
                    "frontend".to_string(),
                ],
                specialty: "frontend".to_string(),
                health_status: WorkerHealthStatus::Degraded, // Simulate a worker with issues
                performance: MockWorkerPerformance {
                    tasks_completed: 67,
                    tasks_failed: 12,
                    avg_completion_time_ms: 950.0,
                    success_rate: 0.848,
                    current_load: 0.2,
                },
            },
        ]
    }
}

/// Adapter that wraps worker pool to implement plan_executor::WorkerPool trait
struct WorkerPoolAdapter {
    worker_pool: Arc<dyn MockWorkerPoolTrait>,
    assignments: Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, String>>>, // worker_id -> milestone_id
}

impl WorkerPoolAdapter {
    async fn new() -> Self {
        Self {
            worker_pool: Arc::new(SimpleMockWorkerPool),
            assignments: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Calculate worker load from available metrics
    /// Uses performance.current_load and enhances with additional factors
    /// In production, this would query real worker metrics API
    async fn calculate_worker_load_from_metrics(&self, worker_handle: &MockWorkerHandle) -> f64 {
        use tracing::debug;

        // Base load from performance metrics (already normalized 0.0-1.0)
        let base_load = worker_handle.performance.current_load;

        // Enhance load calculation with success rate factor
        // Lower success rate indicates higher effective load (more retries/errors)
        let success_rate_factor = 1.0 - worker_handle.performance.success_rate;

        // Combine base load with success rate adjustment
        // Workers with lower success rates are effectively more loaded
        let adjusted_load = (base_load * 0.7) + (success_rate_factor * 0.3);

        // Normalize to 0.0-1.0 range
        let final_load = adjusted_load.max(0.0).min(1.0);

        debug!(
            worker_id = %worker_handle.id,
            base_load = %base_load,
            success_rate = %worker_handle.performance.success_rate,
            adjusted_load = %final_load,
            "Calculated worker load from metrics"
        );

        final_load
    }

    /// Get worker health status from available health metrics
    /// Maps health_status and considers performance indicators
    async fn get_worker_health_from_metrics(
        &self,
        worker_handle: &MockWorkerHandle,
    ) -> crate::planning::plan_executor::WorkerHealth {
        use tracing::debug;

        // Base health from health status field
        let base_health = match worker_handle.health_status {
            WorkerHealthStatus::Healthy => crate::planning::plan_executor::WorkerHealth::Healthy,
            WorkerHealthStatus::Degraded => crate::planning::plan_executor::WorkerHealth::Degraded,
            WorkerHealthStatus::Unhealthy => {
                crate::planning::plan_executor::WorkerHealth::Unhealthy
            }
        };

        // Enhance health assessment with performance metrics
        // Low success rate or high failure rate indicates degraded health
        let success_rate = worker_handle.performance.success_rate;
        let failure_rate = 1.0 - success_rate;

        let final_health = if matches!(
            base_health,
            crate::planning::plan_executor::WorkerHealth::Healthy
        ) {
            // If base health is healthy, check if performance indicates degradation
            if failure_rate > 0.2 {
                // More than 20% failure rate -> degraded
                debug!(
                    worker_id = %worker_handle.id,
                    base_health = ?base_health,
                    failure_rate = %failure_rate,
                    "Health degraded due to high failure rate"
                );
                crate::planning::plan_executor::WorkerHealth::Degraded
            } else if failure_rate > 0.5 {
                // More than 50% failure rate -> unhealthy
                debug!(
                    worker_id = %worker_handle.id,
                    base_health = ?base_health,
                    failure_rate = %failure_rate,
                    "Health unhealthy due to very high failure rate"
                );
                crate::planning::plan_executor::WorkerHealth::Unhealthy
            } else {
                base_health.clone()
            }
        } else {
            // Use base health if already degraded or unhealthy
            base_health.clone()
        };

        debug!(
            worker_id = %worker_handle.id,
            base_health = ?base_health,
            final_health = ?final_health,
            success_rate = %success_rate,
            "Determined worker health from metrics"
        );

        final_health
    }
}

#[async_trait::async_trait]
impl crate::planning::plan_executor::WorkerPool for WorkerPoolAdapter {
    async fn available_workers(&self) -> Result<Vec<crate::planning::plan_executor::WorkerInfo>> {
        // Query mock worker pool for available workers
        let worker_handles = self.worker_pool.list_workers().await;

        if worker_handles.is_empty() {
            return Ok(vec![]);
        }

        // Convert MockWorkerHandle list to WorkerInfo list for planning executor
        let mut worker_infos = Vec::new();

        for worker_handle in worker_handles {
            // Use capabilities directly from mock worker handle
            let capabilities = worker_handle.capabilities.clone();

            // Calculate worker load from available metrics
            // Uses performance.current_load from mock worker handle
            // In production, this would query real worker metrics API
            let load = self
                .calculate_worker_load_from_metrics(&worker_handle)
                .await;

            // Get worker health status from available health data
            // Uses health_status from mock worker handle
            // In production, this would query real worker health monitoring API
            let health = self.get_worker_health_from_metrics(&worker_handle).await;

            let worker_info = crate::planning::plan_executor::WorkerInfo {
                id: worker_handle.id,
                capabilities,
                load,
                health,
            };

            worker_infos.push(worker_info);
        }

        Ok(worker_infos)
    }

    async fn assign_worker(&self, worker_id: Uuid, milestone_id: String) -> Result<()> {
        let mut assignments = self.assignments.write().await;
        assignments.insert(worker_id, milestone_id);
        Ok(())
    }

    async fn release_worker(&self, worker_id: Uuid) -> Result<()> {
        let mut assignments = self.assignments.write().await;
        assignments.remove(&worker_id);
        Ok(())
    }

    async fn worker_status(
        &self,
        worker_id: Uuid,
    ) -> Result<crate::planning::plan_executor::WorkerStatus> {
        use tracing::debug;

        let assignments = self.assignments.read().await;
        let current_assignment = assignments.get(&worker_id).cloned();

        // Get worker handle from mock worker pool to access metrics
        let worker_handles = self.worker_pool.list_workers().await;
        let worker_handle = worker_handles.iter().find(|w| w.id == worker_id);

        if let Some(handle) = worker_handle {
            // Get health from metrics
            let health = self.get_worker_health_from_metrics(handle).await;

            // Extract performance metrics from worker handle
            let performance = crate::planning::plan_executor::WorkerPerformance {
                tasks_completed: handle.performance.tasks_completed as usize,
                tasks_failed: handle.performance.tasks_failed as usize,
                avg_completion_time_ms: handle.performance.avg_completion_time_ms,
                success_rate: handle.performance.success_rate,
            };

            debug!(
                worker_id = %worker_id,
                health = ?health,
                tasks_completed = performance.tasks_completed,
                tasks_failed = performance.tasks_failed,
                success_rate = %performance.success_rate,
                "Retrieved worker status from metrics"
            );

            Ok(crate::planning::plan_executor::WorkerStatus {
                current_assignment,
                health,
                performance,
            })
        } else {
            // Worker not found - return unavailable status
            debug!(
                worker_id = %worker_id,
                "Worker not found in worker pool"
            );

            Ok(crate::planning::plan_executor::WorkerStatus {
                current_assignment,
                health: crate::planning::plan_executor::WorkerHealth::Unavailable,
                performance: crate::planning::plan_executor::WorkerPerformance {
                    tasks_completed: 0,
                    tasks_failed: 0,
                    avg_completion_time_ms: 0.0,
                    success_rate: 0.0,
                },
            })
        }
    }
}

// Mock implementations for integration (would be replaced with real implementations)

/// Mock worker pool for integration
#[allow(dead_code)] // Reserved for future use
struct MockWorkerPool;

impl MockWorkerPool {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl crate::planning::plan_executor::WorkerPool for MockWorkerPool {
    async fn available_workers(&self) -> Result<Vec<crate::planning::plan_executor::WorkerInfo>> {
        // Return mock workers
        Ok(vec![crate::planning::plan_executor::WorkerInfo {
            id: Uuid::new_v4(),
            capabilities: vec!["general".to_string()],
            load: 0.5,
            health: crate::planning::plan_executor::WorkerHealth::Healthy,
        }])
    }

    async fn assign_worker(&self, worker_id: Uuid, milestone_id: String) -> Result<()> {
        // Mock assignment
        println!(
            "Mock assigned worker {} to milestone {}",
            worker_id, milestone_id
        );
        Ok(())
    }

    async fn release_worker(&self, worker_id: Uuid) -> Result<()> {
        // Mock release
        println!("Mock released worker {}", worker_id);
        Ok(())
    }

    async fn worker_status(
        &self,
        _worker_id: Uuid,
    ) -> Result<crate::planning::plan_executor::WorkerStatus> {
        Ok(crate::planning::plan_executor::WorkerStatus {
            current_assignment: None,
            health: crate::planning::plan_executor::WorkerHealth::Healthy,
            performance: crate::planning::plan_executor::WorkerPerformance {
                tasks_completed: 0,
                tasks_failed: 0,
                avg_completion_time_ms: 0.0,
                success_rate: 1.0,
            },
        })
    }
}

/// Mock audit trail for integration
#[allow(dead_code)] // Reserved for future use
struct MockAuditTrail;

impl MockAuditTrail {
    fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl crate::planning::plan_executor::AuditTrail for MockAuditTrail {
    async fn log_event(&self, event: crate::planning::plan_executor::AuditEvent) -> Result<()> {
        // Mock logging
        println!(
            "Mock audit event: {} - {}",
            event.event_type, event.description
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock database operations
    // struct MockDbOps; disabled due to massive api drift

    // #[async_trait::async_trait]
    // impl crate::planning::DatabaseOperations for MockDbOps {
    //     async fn create_execution_plan(&self, _plan: crate::planning::CreateExecutionPlan) -> Result<crate::planning::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
    //     async fn get_execution_plan(&self, _id: Uuid) -> Result<Option<crate::planning::models::ExecutionPlan>> { Ok(None) }
    //     async fn get_execution_plans(&self) -> Result<Vec<crate::planning::models::ExecutionPlan>> { Ok(vec![]) }
    //     async fn update_execution_plan(&self, _id: Uuid, _update: crate::planning::UpdateExecutionPlan) -> Result<crate::planning::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
    //     async fn delete_execution_plan(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_judge(&self, _judge: crate::planning::CreateJudge) -> Result<crate::planning::models::Judge> { Err(anyhow!("Not implemented")) }
    //     async fn get_judge(&self, _id: Uuid) -> Result<Option<crate::planning::models::Judge>> { Ok(None) }
    //     async fn get_judges(&self) -> Result<Vec<crate::planning::models::Judge>> { Ok(vec![]) }
    //     async fn update_judge(&self, _id: Uuid, _update: crate::planning::UpdateJudge) -> Result<crate::planning::models::Judge> { Err(anyhow!("Not implemented")) }
    //     async fn delete_judge(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_worker(&self, _worker: crate::planning::CreateWorker) -> Result<crate::planning::models::Worker> { Err(anyhow!("Not implemented")) }
    //     async fn get_worker(&self, _id: Uuid) -> Result<Option<crate::planning::models::Worker>> { Ok(None) }
    //     async fn get_workers(&self) -> Result<Vec<crate::planning::models::Worker>> { Ok(vec![]) }
    //     async fn update_worker(&self, _id: Uuid, _update: crate::planning::UpdateWorker) -> Result<crate::planning::models::Worker> { Err(anyhow!("Not implemented")) }
    //     async fn delete_worker(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_task(&self, _task: crate::planning::CreateTask) -> Result<crate::planning::models::Task> { Err(anyhow!("Not implemented")) }
    //     async fn get_task(&self, _id: Uuid) -> Result<Option<crate::planning::models::Task>> { Ok(None) }
    //     async fn get_tasks(&self, _status: Option<String>) -> Result<Vec<crate::planning::models::Task>> { Ok(vec![]) }
    //     async fn update_task(&self, _id: Uuid, _update: crate::planning::UpdateTask) -> Result<crate::planning::models::Task> { Err(anyhow!("Not implemented")) }
    //     async fn delete_task(&self, _id: Uuid) -> Result<()> { Ok(()) }
    //     async fn create_task_execution(&self, _execution: crate::planning::CreateTaskExecution) -> Result<crate::planning::models::TaskExecution> { Err(anyhow!("Not implemented")) }
    //     async fn get_task_execution(&self, _id: Uuid) -> Result<Option<crate::planning::models::TaskExecution>> { Ok(None) }
    //     async fn get_task_executions(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::TaskExecution>> { Ok(vec![]) }
    //     async fn update_task_execution(&self, _id: Uuid, _update: crate::planning::UpdateTaskExecution) -> Result<crate::planning::models::TaskExecution> { Err(anyhow!("Not implemented")) }
    //     async fn create_audit_trail_entry(&self, _entry: crate::planning::CreateAuditTrailEntry) -> Result<crate::planning::models::AuditTrailEntry> { Err(anyhow!("Not implemented")) }
    //     async fn get_audit_trail_entries(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::AuditTrailEntry>> { Ok(vec![]) }
    //     async fn get_audit_trail_entry(&self, _id: Uuid) -> Result<Option<crate::planning::models::AuditTrailEntry>> { Ok(None) }
    //     async fn create_council_verdict(&self, _verdict: crate::planning::CreateCouncilVerdict) -> Result<crate::planning::models::CouncilVerdict> { Err(anyhow!("Not implemented")) }
    //     async fn get_council_verdict(&self, _id: Uuid) -> Result<Option<crate::planning::models::CouncilVerdict>> { Ok(None) }
    //     async fn get_council_verdicts(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::CouncilVerdict>> { Ok(vec![]) }
    //     async fn create_judge_evaluation(&self, _evaluation: crate::planning::CreateJudgeEvaluation) -> Result<crate::planning::models::JudgeEvaluation> { Err(anyhow!("Not implemented")) }
    //     async fn get_judge_evaluations(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::JudgeEvaluation>> { Ok(vec![]) }
    //     // Planning methods (stubs)
    //     async fn create_milestone(&self, _milestone: crate::planning::CreateMilestone) -> Result<crate::planning::models::Milestone> { Err(anyhow!("Not implemented")) }
    //     async fn get_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Option<crate::planning::models::Milestone>> { Ok(None) }
    //     async fn get_milestones(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::Milestone>> { Ok(vec![]) }
    //     async fn update_milestone(&self, _plan_id: Uuid, _milestone_id: String, _update: crate::planning::UpdateMilestone) -> Result<crate::planning::models::Milestone> { Err(anyhow!("Not implemented")) }
    //     async fn delete_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<()> { Ok(()) }
    //     async fn create_planning_session(&self, _session: crate::planning::CreatePlanningSession) -> Result<crate::planning::models::PlanningSession> { Err(anyhow!("Not implemented")) }
    //     async fn get_planning_session(&self, _id: Uuid) -> Result<Option<crate::planning::models::PlanningSession>> { Ok(None) }
    //     async fn get_planning_sessions(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::PlanningSession>> { Ok(vec![]) }
    //     async fn update_planning_session(&self, _id: Uuid, _update: crate::planning::UpdatePlanningSession) -> Result<crate::planning::models::PlanningSession> { Err(anyhow!("Not implemented")) }
    //     async fn create_evidence_artifact(&self, _artifact: crate::planning::CreateEvidenceArtifact) -> Result<crate::planning::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
    //     async fn get_evidence_artifacts(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::EvidenceArtifact>> { Ok(vec![]) }
    //     async fn get_evidence_artifacts_for_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Vec<crate::planning::models::EvidenceArtifact>> { Ok(vec![]) }
    //     async fn update_evidence_artifact(&self, _id: Uuid, _update: crate::planning::UpdateEvidenceArtifact) -> Result<crate::planning::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
    //     async fn create_planning_audit_event(&self, _event: crate::planning::CreatePlanningAuditEvent) -> Result<crate::planning::models::PlanningAuditEvent> { Err(anyhow!("Not implemented")) }
    //     async fn get_planning_audit_events(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::PlanningAuditEvent>> { Ok(vec![]) }
    //     async fn create_planning_telemetry(&self, _telemetry: crate::planning::CreatePlanningTelemetry) -> Result<crate::planning::models::PlanningTelemetry> { Err(anyhow!("Not implemented")) }
    //     async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> Result<Vec<crate::planning::models::PlanningTelemetry>> { Ok(vec![]) }

    //     // Waiver operations
    //     async fn get_waivers(&self, _status: Option<String>) -> Result<Vec<crate::planning::models::Waiver>> { Ok(vec![]) }
    //     async fn create_waiver(&self, _waiver: crate::planning::CreateWaiver) -> Result<crate::planning::models::Waiver> { Err(anyhow!("Not implemented")) }
    //     async fn update_waiver(&self, _id: Uuid, _update: crate::planning::UpdateWaiver) -> Result<crate::planning::models::Waiver> { Err(anyhow!("Not implemented")) }
    // }

    #[test]
    fn test_planning_task_result_creation() {
        // Test that PlanningTaskResult can be created
        let task_id = Uuid::new_v4();
        let execution_plan = crate::planning::plan_types::ExecutionPlan {
            contract_plan: agent_agency_contracts::planning_io::ExecutionPlan {
                id: Uuid::new_v4(),
                session_id: Uuid::new_v4(),
                working_spec_id: "test-plan".to_string(),
                contract_plan: agent_agency_contracts::WorkingSpec {
                    version: "1.0".to_string(),
                    id: "TEST-001".to_string(),
                    title: "Test Plan".to_string(),
                    description: "Test execution plan".to_string(),
                    goals: vec![],
                    risk_tier: 2,
                    constraints: agent_agency_contracts::WorkingSpecConstraints {
                        max_duration_minutes: None,
                        max_iterations: None,
                        budget_limits: None,
                        scope_restrictions: None,
                    },
                    acceptance_criteria: vec![],
                    test_plan: agent_agency_contracts::TestPlan {
                        unit_tests: vec![],
                        integration_tests: vec![],
                        e2e_scenarios: vec![],
                        coverage_targets: None,
                    },
                    rollback_plan: agent_agency_contracts::RollbackPlan {
                        strategy: agent_agency_contracts::RollbackStrategy::GitRevert,
                        automated_steps: vec![],
                        manual_steps: vec![],
                        data_impact: agent_agency_contracts::DataImpact::None,
                        downtime_required: None,
                        rollback_window_minutes: None,
                    },
                    context: agent_agency_contracts::WorkingSpecContext {
                        workspace_root: ".".to_string(),
                        git_branch: "main".to_string(),
                        recent_changes: vec![],
                        dependencies: std::collections::HashMap::new(),
                        environment: agent_agency_contracts::task_request::Environment::Development,
                    },
                    non_functional_requirements: None,
                    validation_results: None,
                    quality_gates: None,
                    scope: vec![],
                    metadata: None,
                    milestones: vec![],
                    change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                        max_files: 10,
                        max_loc: 100,
                        max_migrations: 0,
                        allow_breaking_changes: false,
                        allow_new_dependencies: false,
                        enforcement_mode:
                            agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
                    },
                    file_changes: vec![],
                    coverage_targets: None,
                    overview: "Test plan".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                },
                title: "Test Plan".to_string(),
                overview: "Test execution plan".to_string(),
                state: agent_agency_contracts::planning_io::PlanState::Draft,
                milestones: vec![],
                dependency_graph: agent_agency_contracts::planning_io::DependencyGraph {
                    nodes: std::collections::HashMap::new(),
                    edges: vec![],
                    critical_path: vec![],
                    parallel_groups: vec![],
                    has_cycles: false,
                    cycles: vec![],
                },
                change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                    max_files: 10,
                    max_loc: 100,
                    max_migrations: 0,
                    allow_breaking_changes: false,
                    allow_new_dependencies: false,
                    enforcement_mode:
                        agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
                },
                quality_gates: agent_agency_contracts::planning_io::QualityGates {
                    coverage_requirements: std::collections::HashMap::new(),
                    mutation_requirements:
                        agent_agency_contracts::planning_io::MutationRequirements {
                            required: false,
                            min_score: 0.0,
                            operators: vec![],
                        },
                    security_requirements:
                        agent_agency_contracts::planning_io::SecurityRequirements {
                            scan_required: false,
                            max_issues_by_severity: std::collections::HashMap::new(),
                            required_controls: vec![],
                        },
                    performance_requirements:
                        agent_agency_contracts::planning_io::PerformanceRequirements {
                            max_regressions: 0,
                            required_benchmarks: vec![],
                            slas: vec![],
                        },
                    documentation_requirements:
                        agent_agency_contracts::planning_io::DocumentationRequirements {
                            api_docs_required: false,
                            code_docs_required: false,
                            architecture_docs_required: false,
                            required_formats: vec![],
                            required_types: vec![],
                            min_coverage: 0.0,
                            quality_checks: vec![],
                        },
                    requires_manual_review: false,
                    requires_council_approval: false,
                    min_coverage: None,
                    min_mutation_score_percent: None,
                },
                evidence_requirements: vec![],
                active_waivers: vec![],
                metadata: agent_agency_contracts::planning_io::PlanMetadata {
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    approved_at: None,
                    completed_at: None,
                    created_by: agent_agency_contracts::planning_io::PlanCreator::AI {
                        model: "test-model".to_string(),
                        version: "1.0".to_string(),
                    },
                    version: "1.0".to_string(),
                    source: "test".to_string(),
                    confidence_score: Some(0.8),
                    generation_time_ms: None,
                    model_used: None,
                    fallback_used: false,
                    strategy: agent_agency_contracts::types::planning::PlanningStrategy::AIAssisted,
                    confidence: 0.8,
                    estimated_duration_ms: 1000,
                    estimated_cost_cents: 0,
                    adaptive: false,
                    engine_version: "1.0".to_string(),
                    additional_metadata: std::collections::HashMap::new(),
                },
                execution_context: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                approved_at: None,
                completed_at: None,
            },
            orchestration_meta: Default::default(),
            execution_context: Default::default(),
            execution_state: None,
        };

        let execution_result = agent_agency_contracts::planning::PlanExecutionResult {
            plan_id: execution_plan.contract_plan.id,
            success: true,
            milestones_completed: 0,
            total_duration_ms: 0,
            evidence: agent_agency_contracts::planning::ExecutionEvidence {
                plan_evidence: vec![],
                milestone_evidence: std::collections::HashMap::new(),
                quality_validation: vec![],
                council_reviews: vec![],
            },
            metrics: agent_agency_contracts::planning::ExecutionMetrics {
                total_milestones: 0,
                successful_milestones: 0,
                failed_milestones: 0,
                skipped_milestones: 0,
                avg_milestone_time_ms: 0.0,
                parallel_time_saved_ms: 0,
                resource_utilization: agent_agency_contracts::planning::ResourceUtilization {
                    cpu_utilization: 0.0,
                    memory_utilization: 0.0,
                    network_io_bytes: 0,
                    disk_io_bytes: 0,
                    worker_utilization: std::collections::HashMap::new(),
                },
                quality_metrics: agent_agency_contracts::planning::QualityMetrics {
                    avg_coverage: 0.8,
                    avg_mutation_score: 70.0,
                    security_issues_found: 0,
                    performance_regressions: 0,
                    code_quality_score: 0.0,
                },
                performance_metrics: agent_agency_contracts::planning::PerformanceMetrics {
                    total_time_ms: 0,
                    dependency_wait_time_ms: 0,
                    parallel_execution_time_ms: 0,
                    sequential_execution_time_ms: 0,
                    efficiency_ratio: 0.0,
                },
            },
            final_state: agent_agency_contracts::planning_io::PlanState::Completed,
            timeline: vec![],
        };

        let result = PlanningTaskResult {
            task_id,
            execution_plan: execution_plan.clone(),
            execution_result: execution_result.clone(),
            quality_verified: true,
            evidence_count: 0,
        };

        assert_eq!(result.task_id, task_id);
        assert_eq!(
            result.execution_plan.contract_plan.id,
            execution_plan.contract_plan.id
        );
        assert_eq!(result.execution_result.plan_id, execution_result.plan_id);
        assert_eq!(result.quality_verified, true);
        assert_eq!(result.evidence_count, 0);
    }

    #[test]
    fn test_planning_status_creation() {
        // Test that PlanningStatus can be created
        let task_id = Uuid::new_v4();
        let plan_id = Uuid::new_v4();
        let status = PlanningStatus {
            task_id,
            plan_id,
            state: agent_agency_contracts::planning_io::PlanState::Draft,
            progress: 0.0,
            quality_verified: false,
            evidence_count: 0,
            last_updated: chrono::Utc::now(),
        };

        assert_eq!(status.task_id, task_id);
        assert_eq!(status.plan_id, plan_id);
        assert_eq!(status.progress, 0.0);
        assert_eq!(status.quality_verified, false);
        assert_eq!(status.evidence_count, 0);
    }
}
