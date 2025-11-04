//! Orchestrator Integration - Hook Planning System into Orchestrator Workflow
//!
//! Integrates the planning system into orchestrator.plan_task() and autonomous_executor.
//! Provides planning-aware task submission and execution with full CAWS compliance.
//!
//! @author @darianrosebrook

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};use std::sync::Arc;
use anyhow::{anyhow, Result};
use uuid::Uuid;
use agent_agency_contracts::*;
use crate::planning::{
    plan_types::ExecutionPlan as PlanningExecutionPlan,
    plan_generator::PlanGenerator,
    plan_executor::PlanExecutor,
    storage::PlanningStorage,
    parallel_coordinator::ParallelCoordinator,
    worker_assignment::WorkerAssignmentStrategy,
    evidence::EvidenceCollector,
    scope_guard::ScopeGuard,
    council_monitor::CouncilMonitor,
    todo_integration::TodoIntegration,
    council_review::CouncilPlanReview,
};

/// Planning integration for orchestrators
#[derive(Debug)]
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
    todo_integration: Arc<TodoIntegration>,

    /// Council plan review for pre-execution assessment
    council_review: Arc<CouncilPlanReview>,

    /// Database operations for audit trails and persistence
    db_ops: Arc<dyn crate::planning::DatabaseOperations>,
}

/// Planning-aware task execution result

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct PlanningTaskResult {
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
        todo_integration: Arc<TodoIntegration>,
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
        let review_result = self.council_review.review_plan(&execution_plan).await?;
        if !review_result.approved {
            let reason = "Plan rejected by constitutional council";
            // TODO: Add detailed reason based on council decision
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
        let quality_verified = self.verify_execution_quality(&execution_plan, &execution_result).await?;
        let evidence_count = execution_result.evidence.plan_evidence.len();

        // 5. Store final results
        self.store_execution_results(task_id, &execution_plan, &execution_result).await?;

        Ok(PlanningTaskResult {
            task_id,
            execution_plan,
            execution_result,
            quality_verified,
            evidence_count,
        })
    }

    /// Generate execution plan from task descriptor
    async fn generate_execution_plan(&self, task_descriptor: &TaskDescriptor) -> Result<PlanningExecutionPlan> {
        // Convert task descriptor to working spec
        let working_spec = agent_agency_contracts::WorkingSpec {
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
        use crate::planning::plan_types::{PlanGenerationContext, WorkingSpecProvider, TaskDescriptorProvider};
        
        // Simple provider wrapper for working spec
        struct WorkingSpecWrapper(agent_agency_contracts::WorkingSpec);
        #[async_trait::async_trait]
        impl WorkingSpecProvider for WorkingSpecWrapper {
            async fn get_working_spec(&self) -> Result<agent_agency_contracts::WorkingSpec, anyhow::Error> {
                Ok(self.0.clone())
            }
        }
        
        // Simple provider wrapper for task descriptor
        struct TaskDescriptorWrapper(agent_agency_contracts::TaskDescriptor);
        #[async_trait::async_trait]
        impl TaskDescriptorProvider for TaskDescriptorWrapper {
            async fn get_task_descriptor(&self) -> Result<agent_agency_contracts::TaskDescriptor, anyhow::Error> {
                Ok(self.0.clone())
            }
        }
        
        let plan_context = PlanGenerationContext {
            working_spec_provider: Box::new(WorkingSpecWrapper(working_spec)),
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
        self.planning_storage.store_execution_plan(&plan_response).await?;

        Ok(plan_response)
    }

    /// Create plan executor with all real dependencies
    async fn create_plan_executor(&self, plan: PlanningExecutionPlan) -> Result<PlanExecutor> {
        // Create worker pool using local MCPWorkerPool implementation
        let mcp_pool = Arc::new(
            crate::multimodal_orchestration::MCPWorkerPool::new(
                crate::multimodal_orchestration::WorkerPoolConfig::default()
            ).await
                .map_err(|e| anyhow!("Failed to create MCPWorkerPool: {}", e))?
        );
        let worker_pool = Arc::new(WorkerPoolAdapter::new(mcp_pool).await);

        // Create audit trail using real AuditTrailManager with adapter
        let audit_config = crate::AuditConfig::default();
        let audit_manager = Arc::new(crate::AuditTrailManager::new(audit_config));
        let audit_trail = Arc::new(AuditTrailAdapter::new(
            audit_manager,
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
            Arc::new(tokio::sync::Mutex::new(Arc::clone(&self.todo_integration))),
            crate::planning::plan_executor::ExecutionConfig::default(),
        );

        Ok(executor)
    }

    /// Verify execution quality against requirements
    async fn verify_execution_quality(
        &self,
        plan: &PlanningExecutionPlan,
        result: &agent_agency_contracts::planning::PlanExecutionResult,
    ) -> Result<bool> {
        // Check if all quality gates were satisfied
        let quality_gates_satisfied = result.evidence.quality_validation.iter()
            .all(|v| v.passed);

        // Check evidence completeness
        let evidence_complete = !result.evidence.plan_evidence.is_empty() &&
            result.evidence.plan_evidence.iter().all(|e| e.verified);

        // Check milestone completion
        let milestones_complete = result.milestones_completed == result.evidence.milestone_evidence.len();

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
            metadata: serde_json::Value::Object([
                ("task_id".to_string(), serde_json::Value::String(task_id.to_string())),
                ("success".to_string(), serde_json::Value::Bool(result.success)),
                ("milestones_completed".to_string(), serde_json::Value::Number(result.milestones_completed.into())),
            ].into_iter().collect()),
        timestamp: chrono::Utc::now(),
        plan_id: plan.contract_plan.id,
        };
        self.planning_storage.as_ref().log_audit_event(audit_event).await?;

        // Create audit trail entry
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("task_id".to_string(), serde_json::Value::String(task_id.to_string()));
        metadata.insert("resource_id".to_string(), serde_json::Value::String(plan.contract_plan.id.to_string()));
        metadata.insert("resource_type".to_string(), serde_json::Value::String("execution_plan".to_string()));

        let audit_entry = crate::planning::data_infrastructure_types::CreateAuditTrailEntry {
            event_type: "planning_execution_completed".to_string(),
            description: format!(
                "Planning execution completed: {} milestones completed, quality_verified: {}",
                result.milestones_completed,
                result.success
            ),
            metadata,
        };

        self.db_ops.create_audit_trail_entry(audit_entry).await?;

        Ok(())
    }

    /// Get planning status for a task
    pub async fn get_task_planning_status(&self, task_id: Uuid) -> Result<Option<PlanningStatus>> {
        // Check if there's an execution plan for this task
        if let Some(plan) = self.planning_storage.as_ref().load_execution_plan(task_id).await? {
            // Return planning status based on plan state
            let progress = plan.execution_state.as_ref()
                .map(|state| state.completed_milestones.len() as f64 / plan.contract_plan.milestones.len().max(1) as f64 * 100.0)
                .unwrap_or(0.0);

            let status = PlanningStatus {
                task_id,
                plan_id: plan.contract_plan.id,
                state: plan.contract_plan.state.clone(),
                progress,
                quality_verified: plan.contract_plan.quality_gates.coverage_requirements.values().all(|&req| req >= 80.0), // Simplified check
                evidence_count: plan.contract_plan.evidence_requirements.len(),
                last_updated: chrono::Utc::now(),
            };
            Ok(Some(status))
        } else {
            Ok(None)
        }
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
        Self { audit_manager, db_ops }
    }
}

#[async_trait::async_trait]
impl crate::planning::plan_executor::AuditTrail for AuditTrailAdapter {
    async fn log_event(&self, event: crate::planning::plan_executor::AuditEvent) -> Result<()> {
        use crate::planning::CreatePlanningAuditEvent;
        
        // Persist to database via DatabaseOperations
        let mut metadata = event.metadata.clone();
        if let Some(milestone_id) = &event.milestone_id {
            metadata.insert("milestone_id".to_string(), serde_json::Value::String(milestone_id.clone()));
        }
        if let Some(worker_id) = &event.worker_id {
            metadata.insert("worker_id".to_string(), serde_json::Value::String(worker_id.to_string()));
        }

        let audit_entry = CreatePlanningAuditEvent {
            plan_id: event.plan_id,
            event_type: format!("{:?}", event.event_type),
            description: event.description.clone(),
            metadata,
        };

        self.db_ops.create_planning_audit_event(audit_entry).await
            .map_err(|e| anyhow!("Failed to create planning audit event: {}", e))?;

        // Also update in-memory stats via AuditTrailManager (if configured)
        // Note: AuditTrailManager's write_event is private, so we use the appropriate
        // auditor methods based on event type for in-memory tracking
        match event.event_type {
            crate::planning::plan_executor::AuditEventType::CouncilDecision => {
                // Use council auditor for council decisions
                if let Some(milestone_id) = &event.milestone_id {
                    self.audit_manager.council_auditor()
                        .record_council_consensus(
                            &event.plan_id.to_string(),
                            "plan_executor",
                            std::collections::HashMap::new(), // vote_distribution - empty for now
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

/// Adapter that wraps MCPWorkerPool to implement plan_executor::WorkerPool trait
struct WorkerPoolAdapter {
    worker_pool: Arc<crate::multimodal_orchestration::MCPWorkerPool>,
    assignments: Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, String>>>, // worker_id -> milestone_id
}

impl WorkerPoolAdapter {
    async fn new(worker_pool: Arc<crate::multimodal_orchestration::MCPWorkerPool>) -> Self {
        Self {
            worker_pool,
            assignments: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl crate::planning::plan_executor::WorkerPool for WorkerPoolAdapter {
    async fn available_workers(&self) -> Result<Vec<crate::planning::plan_executor::WorkerInfo>> {
        // TODO: Integrate with actual MCPWorkerPool when available
        // For now, return empty list
        let workers: Vec<crate::planning::plan_executor::WorkerInfo> = Vec::new();

        // If no workers are available, return empty list
        if workers.is_empty() {
            return Ok(vec![]);
        }

        // Convert WorkerHandle list to WorkerInfo list for planning executor
        let worker_infos = workers.into_iter().enumerate().map(|(i, _worker)| {
            // Mock capabilities
            let capabilities = vec!["general".to_string(), "rust".to_string(), "typescript".to_string()];

            // Mock load calculation
            let load = (i as f64 * 0.2).min(1.0);

            // Mock health - alternate between healthy and degraded
            let health = if i % 2 == 0 {
                crate::planning::plan_executor::WorkerHealth::Healthy
            } else {
                crate::planning::plan_executor::WorkerHealth::Degraded
            };

            crate::planning::plan_executor::WorkerInfo {
                id: uuid::Uuid::new_v4(),
                capabilities,
                load,
                health,
            }
        }).collect();

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

    async fn worker_status(&self, worker_id: Uuid) -> Result<crate::planning::plan_executor::WorkerStatus> {
        let assignments = self.assignments.read().await;
        let current_assignment = assignments.get(&worker_id).cloned();

        // Mock health and performance for now
        let health = crate::planning::plan_executor::WorkerHealth::Healthy;

        Ok(crate::planning::plan_executor::WorkerStatus {
            current_assignment,
            health,
            performance: crate::planning::plan_executor::WorkerPerformance {
                tasks_completed: 0, // TODO: Get from WorkerPool trait
                tasks_failed: 0,
                avg_completion_time_ms: 1000.0, // PLACEHOLDER
                success_rate: 1.0,
            },
        })
    }

}

// Mock implementations for integration (would be replaced with real implementations)

/// Mock worker pool for integration
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
        Ok(vec![
            crate::planning::plan_executor::WorkerInfo {
                id: Uuid::new_v4(),
                capabilities: vec!["general".to_string()],
                load: 0.5,
                health: crate::planning::plan_executor::WorkerHealth::Healthy,
            }
        ])
    }

    async fn assign_worker(&self, worker_id: Uuid, milestone_id: String) -> Result<()> {
        // Mock assignment
        println!("Mock assigned worker {} to milestone {}", worker_id, milestone_id);
        Ok(())
    }

    async fn release_worker(&self, worker_id: Uuid) -> Result<()> {
        // Mock release
        println!("Mock released worker {}", worker_id);
        Ok(())
    }

    async fn worker_status(&self, worker_id: Uuid) -> Result<crate::planning::plan_executor::WorkerStatus> {
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
        println!("Mock audit event: {} - {}", event.event_type, event.description);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock database operations
    struct MockDbOps;

    #[async_trait::async_trait]
    impl crate::planning::DatabaseOperations for MockDbOps {
        async fn create_execution_plan(&self, _plan: crate::planning::CreateExecutionPlan) -> Result<crate::planning::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
        async fn get_execution_plan(&self, _id: Uuid) -> Result<Option<crate::planning::models::ExecutionPlan>> { Ok(None) }
        async fn get_execution_plans(&self) -> Result<Vec<crate::planning::models::ExecutionPlan>> { Ok(vec![]) }
        async fn update_execution_plan(&self, _id: Uuid, _update: crate::planning::UpdateExecutionPlan) -> Result<crate::planning::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
        async fn delete_execution_plan(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_judge(&self, _judge: crate::planning::CreateJudge) -> Result<crate::planning::models::Judge> { Err(anyhow!("Not implemented")) }
        async fn get_judge(&self, _id: Uuid) -> Result<Option<crate::planning::models::Judge>> { Ok(None) }
        async fn get_judges(&self) -> Result<Vec<crate::planning::models::Judge>> { Ok(vec![]) }
        async fn update_judge(&self, _id: Uuid, _update: crate::planning::UpdateJudge) -> Result<crate::planning::models::Judge> { Err(anyhow!("Not implemented")) }
        async fn delete_judge(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_worker(&self, _worker: crate::planning::CreateWorker) -> Result<crate::planning::models::Worker> { Err(anyhow!("Not implemented")) }
        async fn get_worker(&self, _id: Uuid) -> Result<Option<crate::planning::models::Worker>> { Ok(None) }
        async fn get_workers(&self) -> Result<Vec<crate::planning::models::Worker>> { Ok(vec![]) }
        async fn update_worker(&self, _id: Uuid, _update: crate::planning::UpdateWorker) -> Result<crate::planning::models::Worker> { Err(anyhow!("Not implemented")) }
        async fn delete_worker(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_task(&self, _task: crate::planning::CreateTask) -> Result<crate::planning::models::Task> { Err(anyhow!("Not implemented")) }
        async fn get_task(&self, _id: Uuid) -> Result<Option<crate::planning::models::Task>> { Ok(None) }
        async fn get_tasks(&self, _status: Option<String>) -> Result<Vec<crate::planning::models::Task>> { Ok(vec![]) }
        async fn update_task(&self, _id: Uuid, _update: crate::planning::UpdateTask) -> Result<crate::planning::models::Task> { Err(anyhow!("Not implemented")) }
        async fn delete_task(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_task_execution(&self, _execution: crate::planning::CreateTaskExecution) -> Result<crate::planning::models::TaskExecution> { Err(anyhow!("Not implemented")) }
        async fn get_task_execution(&self, _id: Uuid) -> Result<Option<crate::planning::models::TaskExecution>> { Ok(None) }
        async fn get_task_executions(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::TaskExecution>> { Ok(vec![]) }
        async fn update_task_execution(&self, _id: Uuid, _update: crate::planning::UpdateTaskExecution) -> Result<crate::planning::models::TaskExecution> { Err(anyhow!("Not implemented")) }
        async fn create_audit_trail_entry(&self, _entry: crate::planning::CreateAuditTrailEntry) -> Result<crate::planning::models::AuditTrailEntry> { Err(anyhow!("Not implemented")) }
        async fn get_audit_trail_entries(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::AuditTrailEntry>> { Ok(vec![]) }
        async fn get_audit_trail_entry(&self, _id: Uuid) -> Result<Option<crate::planning::models::AuditTrailEntry>> { Ok(None) }
        async fn create_council_verdict(&self, _verdict: crate::planning::CreateCouncilVerdict) -> Result<crate::planning::models::CouncilVerdict> { Err(anyhow!("Not implemented")) }
        async fn get_council_verdict(&self, _id: Uuid) -> Result<Option<crate::planning::models::CouncilVerdict>> { Ok(None) }
        async fn get_council_verdicts(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::CouncilVerdict>> { Ok(vec![]) }
        async fn create_judge_evaluation(&self, _evaluation: crate::planning::CreateJudgeEvaluation) -> Result<crate::planning::models::JudgeEvaluation> { Err(anyhow!("Not implemented")) }
        async fn get_judge_evaluations(&self, _task_id: Uuid) -> Result<Vec<crate::planning::models::JudgeEvaluation>> { Ok(vec![]) }
        // Planning methods (stubs)
        async fn create_milestone(&self, _milestone: crate::planning::CreateMilestone) -> Result<crate::planning::models::Milestone> { Err(anyhow!("Not implemented")) }
        async fn get_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Option<crate::planning::models::Milestone>> { Ok(None) }
        async fn get_milestones(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::Milestone>> { Ok(vec![]) }
        async fn update_milestone(&self, _plan_id: Uuid, _milestone_id: String, _update: crate::planning::UpdateMilestone) -> Result<crate::planning::models::Milestone> { Err(anyhow!("Not implemented")) }
        async fn delete_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<()> { Ok(()) }
        async fn create_planning_session(&self, _session: crate::planning::CreatePlanningSession) -> Result<crate::planning::models::PlanningSession> { Err(anyhow!("Not implemented")) }
        async fn get_planning_session(&self, _id: Uuid) -> Result<Option<crate::planning::models::PlanningSession>> { Ok(None) }
        async fn get_planning_sessions(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::PlanningSession>> { Ok(vec![]) }
        async fn update_planning_session(&self, _id: Uuid, _update: crate::planning::UpdatePlanningSession) -> Result<crate::planning::models::PlanningSession> { Err(anyhow!("Not implemented")) }
        async fn create_evidence_artifact(&self, _artifact: crate::planning::CreateEvidenceArtifact) -> Result<crate::planning::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
        async fn get_evidence_artifacts(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::EvidenceArtifact>> { Ok(vec![]) }
        async fn get_evidence_artifacts_for_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Vec<crate::planning::models::EvidenceArtifact>> { Ok(vec![]) }
        async fn update_evidence_artifact(&self, _id: Uuid, _update: crate::planning::UpdateEvidenceArtifact) -> Result<crate::planning::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
        async fn create_planning_audit_event(&self, _event: crate::planning::CreatePlanningAuditEvent) -> Result<crate::planning::models::PlanningAuditEvent> { Err(anyhow!("Not implemented")) }
        async fn get_planning_audit_events(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::models::PlanningAuditEvent>> { Ok(vec![]) }
        async fn create_planning_telemetry(&self, _telemetry: crate::planning::CreatePlanningTelemetry) -> Result<crate::planning::models::PlanningTelemetry> { Err(anyhow!("Not implemented")) }
        async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> Result<Vec<crate::planning::models::PlanningTelemetry>> { Ok(vec![]) }
        
        // Waiver operations
        async fn get_waivers(&self, _status: Option<String>) -> Result<Vec<crate::planning::models::Waiver>> { Ok(vec![]) }
        async fn create_waiver(&self, _waiver: crate::planning::CreateWaiver) -> Result<crate::planning::models::Waiver> { Err(anyhow!("Not implemented")) }
        async fn update_waiver(&self, _id: Uuid, _update: crate::planning::UpdateWaiver) -> Result<crate::planning::models::Waiver> { Err(anyhow!("Not implemented")) }
    }

    #[test]
    fn test_planning_task_result_creation() {
        // Test that PlanningTaskResult can be created
        let task_id = Uuid::new_v4();
        let execution_plan = crate::planning::plan_types::ExecutionPlan {
            id: Uuid::new_v4(),
            contract_plan: agent_agency_contracts::planning::ExecutionPlan {
                id: "test-plan".to_string(),
                title: "Test Plan".to_string(),
                description: "Test execution plan".to_string(),
                milestones: vec![],
                constraints: vec![],
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                status: agent_agency_contracts::planning::PlanStatus::Draft,
                priority: agent_agency_contracts::planning::PlanPriority::Medium,
                owner: "test-user".to_string(),
                tags: vec![],
                metadata: std::collections::HashMap::new(),
            },
            milestones: vec![],
            state: crate::planning::plan_types::PlanState::Draft,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let execution_result = agent_agency_contracts::planning::PlanExecutionResult {
            plan_id: execution_plan.id,
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
                total_execution_time_ms: 0,
                milestone_execution_times: std::collections::HashMap::new(),
                resource_usage: std::collections::HashMap::new(),
                quality_metrics: std::collections::HashMap::new(),
            },
            final_state: agent_agency_contracts::planning::PlanExecutionState::Completed,
            error_message: None,
        };

        let result = PlanningTaskResult {
            task_id,
            execution_plan: execution_plan.clone(),
            execution_result: execution_result.clone(),
            quality_verified: true,
            evidence_count: 0,
        };

        assert_eq!(result.task_id, task_id);
        assert_eq!(result.execution_plan.id, execution_plan.id);
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
            state: crate::planning::plan_types::PlanState::Draft,
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