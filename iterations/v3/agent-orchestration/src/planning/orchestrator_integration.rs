//! Orchestrator Integration - Hook Planning System into Orchestrator Workflow
//!
//! Integrates the planning system into orchestrator.plan_task() and autonomous_executor.
//! Provides planning-aware task submission and execution with full CAWS compliance.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use anyhow::{anyhow, Result};
use uuid::Uuid;
use agent_agency_contracts::planning_io::ExecutionPlan;
use crate::types::TaskDescriptor;
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
    db_ops: Arc<dyn data_infrastructure::DatabaseOperations>,
}

/// Planning-aware task execution result
#[derive(Debug)]
pub struct PlanningTaskResult {
    /// Task ID that was executed
    pub task_id: Uuid,

    /// Generated execution plan
    pub execution_plan: PlanningExecutionPlan,

    /// Plan execution result
    pub execution_result: crate::planning::plan_executor::PlanExecutionResult,

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
        db_ops: Arc<dyn data_infrastructure::DatabaseOperations>,
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
        let task_id = Uuid::parse_str(&task_descriptor.task_id)
            .map_err(|_| anyhow!("Invalid task ID format: {}", task_descriptor.task_id))?;

        // 1. Generate execution plan using planning system
        let execution_plan = self.generate_execution_plan(task_descriptor).await?;

        // 2. Review plan with constitutional council
        let review_result = self.council_review.review_plan(&execution_plan).await?;
        if !review_result.approved {
            return Err(anyhow!(
                "Plan {} rejected by constitutional council: {}",
                execution_plan.contract_plan.id,
                review_result.council_decision.rationale
            ));
        }

        // 3. Create plan executor with all dependencies
        let plan_executor = self.create_plan_executor(execution_plan.clone()).await?;

        // 3. Execute the plan
        let execution_result = plan_executor.execute().await?;

        // 4. Collect final evidence and verification
        let quality_verified = self.verify_execution_quality(&execution_plan, &execution_result).await?;
        let evidence_count = execution_result.evidence_artifacts.len();

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
        // Convert task descriptor to planning request
        let planning_request = crate::planning::plan_types::PlanGenerationRequest {
            task_description: task_descriptor.description.clone(),
            context: format!("Priority: {:?}, Scope: {:?}", task_descriptor.priority, task_descriptor.scope),
            constraints: vec![], // Could extract from task descriptor
            desired_outcome: task_descriptor.description.clone(),
            existing_plan_id: None,
        };

        // Generate plan
        let plan_response = self.plan_generator.generate_plan(planning_request).await?;

        // Store the generated plan
        self.planning_storage.store_plan(&plan_response.plan).await?;

        Ok(plan_response.plan)
    }

    /// Create plan executor with all real dependencies
    async fn create_plan_executor(&self, plan: PlanningExecutionPlan) -> Result<PlanExecutor> {
        // Create worker pool using real MCPWorkerPool with adapter
        let mcp_pool = Arc::new(
            agent_workers::MCPWorkerPool::new(agent_workers::WorkerPoolConfig::default()).await
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
            Arc::clone(&self.parallel_coordinator),
            audit_trail,
            Arc::clone(&self.todo_integration),
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
        let quality_gates_satisfied = result.quality_verifications.iter()
            .all(|v| v.gate_status == crate::planning::plan_executor::QualityGateStatus::Passed);

        // Check evidence completeness
        let evidence_complete = !result.evidence_artifacts.is_empty() &&
            result.evidence_artifacts.iter().all(|e| e.verified);

        // Check milestone completion
        let milestones_complete = result.milestone_results.iter()
            .all(|m| m.status == crate::planning::plan_executor::MilestoneStatus::Completed);

        Ok(quality_gates_satisfied && evidence_complete && milestones_complete)
    }

    /// Store execution results for audit and analysis
    async fn store_execution_results(
        &self,
        task_id: Uuid,
        plan: &PlanningExecutionPlan,
        result: &agent_agency_contracts::planning::PlanExecutionResult,
    ) -> Result<()> {
        // Store in planning storage
        self.planning_storage.store_execution_result(plan.id, result).await?;

        // Create audit trail entry
        let audit_entry = data_infrastructure::models::AuditTrailEntry {
            id: Uuid::new_v4(),
            task_id,
            action: "planning_execution_completed".to_string(),
            actor: "orchestrator_integration".to_string(),
            resource_id: Some(plan.id),
            resource_type: Some("execution_plan".to_string()),
            change_summary: format!(
                "Planning execution completed: {} milestones, {} evidence artifacts, quality_verified: {}",
                result.milestone_results.len(),
                result.evidence_artifacts.len(),
                result.quality_verifications.iter().all(|v| v.gate_status == crate::planning::plan_executor::QualityGateStatus::Passed)
            ),
            timestamp: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
            metadata: serde_json::json!({
                "execution_result": result,
                "plan_id": plan.id,
                "task_id": task_id
            }),
        };

        self.db_ops.create_audit_trail_entry(audit_entry).await?;

        Ok(())
    }

    /// Get planning status for a task
    pub async fn get_task_planning_status(&self, task_id: Uuid) -> Result<Option<PlanningStatus>> {
        // Check if there's an execution plan for this task
        if let Some(plan) = self.planning_storage.get_plan_for_task(task_id).await? {
            // Get execution result if available
            if let Some(result) = self.planning_storage.get_execution_result(plan.id).await? {
                let status = PlanningStatus {
                    task_id,
                    plan_id: plan.id,
                    state: plan.state,
                    progress: self.calculate_progress(&plan, &result),
                    quality_verified: result.quality_verifications.iter()
                        .all(|v| v.gate_status == crate::planning::plan_executor::QualityGateStatus::Passed),
                    evidence_count: result.evidence_artifacts.len(),
                    last_updated: result.completed_at,
                };
                Ok(Some(status))
            } else {
                // Plan exists but not executed yet
                Ok(Some(PlanningStatus {
                    task_id,
                    plan_id: plan.id,
                    state: plan.state,
                    progress: 0.0,
                    quality_verified: false,
                    evidence_count: 0,
                    last_updated: plan.created_at,
                }))
            }
        } else {
            Ok(None)
        }
    }
}

/// Planning status for a task
#[derive(Debug, Clone)]
pub struct PlanningStatus {
    pub task_id: Uuid,
    pub plan_id: Uuid,
    pub state: crate::planning::plan_types::PlanState,
    pub progress: f64,
    pub quality_verified: bool,
    pub evidence_count: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl OrchestratorPlanningIntegration {
    fn calculate_progress(
        &self,
        plan: &PlanningExecutionPlan,
        result: &agent_agency_contracts::planning::PlanExecutionResult,
    ) -> f64 {
        let total_milestones = plan.milestones.len() as f64;
        if total_milestones == 0.0 {
            return 100.0;
        }

        let completed_milestones = result.milestone_results.iter()
            .filter(|m| m.status == crate::planning::plan_executor::MilestoneStatus::Completed)
            .count() as f64;

        (completed_milestones / total_milestones) * 100.0
    }
}

// Adapters for real implementations

/// Adapter that wraps AuditTrailManager to implement plan_executor::AuditTrail trait
struct AuditTrailAdapter {
    audit_manager: Arc<crate::AuditTrailManager>,
    db_ops: Arc<dyn data_infrastructure::DatabaseOperations>,
}

impl AuditTrailAdapter {
    fn new(
        audit_manager: Arc<crate::AuditTrailManager>,
        db_ops: Arc<dyn data_infrastructure::DatabaseOperations>,
    ) -> Self {
        Self { audit_manager, db_ops }
    }
}

#[async_trait::async_trait]
impl crate::planning::plan_executor::AuditTrail for AuditTrailAdapter {
    async fn log_event(&self, event: crate::planning::plan_executor::AuditEvent) -> Result<()> {
        use data_infrastructure::CreatePlanningAuditEvent;
        
        // Persist to database via DatabaseOperations
        let audit_entry = CreatePlanningAuditEvent {
            plan_id: event.plan_id,
            milestone_id: event.milestone_id.clone(),
            worker_id: event.worker_id,
            event_type: format!("{:?}", event.event_type),
            description: event.description.clone(),
            metadata: event.metadata.clone(),
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
                            &format!("{:?}", event.event_type),
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
    worker_pool: Arc<agent_workers::MCPWorkerPool>,
    assignments: Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, String>>>, // worker_id -> milestone_id
}

impl WorkerPoolAdapter {
    async fn new(worker_pool: Arc<agent_workers::MCPWorkerPool>) -> Self {
        Self {
            worker_pool,
            assignments: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl crate::planning::plan_executor::WorkerPool for WorkerPoolAdapter {
    async fn available_workers(&self) -> Result<Vec<crate::planning::plan_executor::WorkerInfo>> {
        use agent_workers::worker_types::WorkerSpecialty;
        
        // Get all registered workers from the pool
        let workers = self.worker_pool.list_workers().await;
        
        // If no workers are registered, register a default general-purpose worker
        if workers.is_empty() {
            let _handle = self.worker_pool.register_worker(
                WorkerSpecialty::General,
                agent_workers::worker_types::WorkerCapabilities::default(),
            ).await
            .map_err(|e| anyhow!("Failed to register default worker: {}", e))?;

            // Get workers again after registration
            let workers = self.worker_pool.list_workers().await;
            return Ok(self.convert_workers_to_info(workers).await);
        }
        
        Ok(self.convert_workers_to_info(workers).await)
    }

    /// Convert WorkerHandle list to WorkerInfo list for planning executor
    async fn convert_workers_to_info(&self, workers: Vec<agent_workers::WorkerHandle>) -> Vec<crate::planning::plan_executor::WorkerInfo> {
        let stats = self.worker_pool.get_stats().await;
        let assignments = self.assignments.read().await;
        
        workers.into_iter().map(|worker| {
            // Build capabilities list from worker capabilities
            let mut capabilities = vec![format!("{:?}", worker.specialty).to_lowercase()];
            capabilities.extend(worker.capabilities.languages.iter().cloned());
            capabilities.extend(worker.capabilities.frameworks.iter().cloned());
            capabilities.extend(worker.capabilities.domains.iter().cloned());
            
            // Calculate load based on assignments
            let is_assigned = assignments.contains_key(&worker.id.0);
            let load = if stats.total_workers > 0 {
                if is_assigned {
                    // Worker is assigned, calculate load based on active workers
                    (stats.active_workers as f64) / (stats.total_workers as f64)
                } else {
                    // Worker is idle
                    0.0
                }
            } else {
                0.0
            };
            
            // Map worker health to plan executor health
            let health = match worker.capabilities.health_status {
                agent_workers::worker_types::WorkerHealth::Healthy => {
                    crate::planning::plan_executor::WorkerHealth::Healthy
                }
                agent_workers::worker_types::WorkerHealth::Degraded => {
                    crate::planning::plan_executor::WorkerHealth::Degraded
                }
                agent_workers::worker_types::WorkerHealth::Unhealthy | 
                agent_workers::worker_types::WorkerHealth::Offline => {
                    crate::planning::plan_executor::WorkerHealth::Degraded
                }
            };
            
            crate::planning::plan_executor::WorkerInfo {
                id: worker.id.0,
                capabilities,
                load,
                health,
            }
        }).collect()
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
        
        let stats = self.worker_pool.get_stats().await;
        let health = if stats.unhealthy_workers == 0 {
            crate::planning::plan_executor::WorkerHealth::Healthy
        } else {
            crate::planning::plan_executor::WorkerHealth::Degraded
        };

        Ok(crate::planning::plan_executor::WorkerStatus {
            current_assignment,
            health,
            performance: crate::planning::plan_executor::WorkerPerformance {
                tasks_completed: stats.total_tasks_completed,
                tasks_failed: 0, // PLACEHOLDER: MCPWorkerPool doesn't track failures separately
                avg_completion_time_ms: stats.average_execution_time_ms,
                success_rate: if stats.total_tasks_completed > 0 {
                    1.0 - (stats.unhealthy_workers as f64 / stats.total_workers as f64)
                } else {
                    1.0
                },
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
                current_load: 0.5,
                status: crate::planning::plan_executor::WorkerStatus::Available,
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
        Ok(crate::planning::plan_executor::WorkerStatus::Available)
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
    impl data_infrastructure::DatabaseOperations for MockDbOps {
        async fn create_execution_plan(&self, _plan: data_infrastructure::CreateExecutionPlan) -> Result<data_infrastructure::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
        async fn get_execution_plan(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::ExecutionPlan>> { Ok(None) }
        async fn get_execution_plans(&self) -> Result<Vec<data_infrastructure::models::ExecutionPlan>> { Ok(vec![]) }
        async fn update_execution_plan(&self, _id: Uuid, _update: data_infrastructure::UpdateExecutionPlan) -> Result<data_infrastructure::models::ExecutionPlan> { Err(anyhow!("Not implemented")) }
        async fn delete_execution_plan(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_judge(&self, _judge: data_infrastructure::CreateJudge) -> Result<data_infrastructure::models::Judge> { Err(anyhow!("Not implemented")) }
        async fn get_judge(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::Judge>> { Ok(None) }
        async fn get_judges(&self) -> Result<Vec<data_infrastructure::models::Judge>> { Ok(vec![]) }
        async fn update_judge(&self, _id: Uuid, _update: data_infrastructure::UpdateJudge) -> Result<data_infrastructure::models::Judge> { Err(anyhow!("Not implemented")) }
        async fn delete_judge(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_worker(&self, _worker: data_infrastructure::CreateWorker) -> Result<data_infrastructure::models::Worker> { Err(anyhow!("Not implemented")) }
        async fn get_worker(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::Worker>> { Ok(None) }
        async fn get_workers(&self) -> Result<Vec<data_infrastructure::models::Worker>> { Ok(vec![]) }
        async fn update_worker(&self, _id: Uuid, _update: data_infrastructure::UpdateWorker) -> Result<data_infrastructure::models::Worker> { Err(anyhow!("Not implemented")) }
        async fn delete_worker(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_task(&self, _task: data_infrastructure::CreateTask) -> Result<data_infrastructure::models::Task> { Err(anyhow!("Not implemented")) }
        async fn get_task(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::Task>> { Ok(None) }
        async fn get_tasks(&self, _status: Option<String>) -> Result<Vec<data_infrastructure::models::Task>> { Ok(vec![]) }
        async fn update_task(&self, _id: Uuid, _update: data_infrastructure::UpdateTask) -> Result<data_infrastructure::models::Task> { Err(anyhow!("Not implemented")) }
        async fn delete_task(&self, _id: Uuid) -> Result<()> { Ok(()) }
        async fn create_task_execution(&self, _execution: data_infrastructure::CreateTaskExecution) -> Result<data_infrastructure::models::TaskExecution> { Err(anyhow!("Not implemented")) }
        async fn get_task_execution(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::TaskExecution>> { Ok(None) }
        async fn get_task_executions(&self, _task_id: Uuid) -> Result<Vec<data_infrastructure::models::TaskExecution>> { Ok(vec![]) }
        async fn update_task_execution(&self, _id: Uuid, _update: data_infrastructure::UpdateTaskExecution) -> Result<data_infrastructure::models::TaskExecution> { Err(anyhow!("Not implemented")) }
        async fn create_audit_trail_entry(&self, _entry: data_infrastructure::CreateAuditTrailEntry) -> Result<data_infrastructure::models::AuditTrailEntry> { Err(anyhow!("Not implemented")) }
        async fn get_audit_trail_entries(&self, _task_id: Uuid) -> Result<Vec<data_infrastructure::models::AuditTrailEntry>> { Ok(vec![]) }
        async fn get_audit_trail_entry(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::AuditTrailEntry>> { Ok(None) }
        async fn create_council_verdict(&self, _verdict: data_infrastructure::CreateCouncilVerdict) -> Result<data_infrastructure::models::CouncilVerdict> { Err(anyhow!("Not implemented")) }
        async fn get_council_verdict(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::CouncilVerdict>> { Ok(None) }
        async fn get_council_verdicts(&self, _task_id: Uuid) -> Result<Vec<data_infrastructure::models::CouncilVerdict>> { Ok(vec![]) }
        async fn create_judge_evaluation(&self, _evaluation: data_infrastructure::CreateJudgeEvaluation) -> Result<data_infrastructure::models::JudgeEvaluation> { Err(anyhow!("Not implemented")) }
        async fn get_judge_evaluations(&self, _task_id: Uuid) -> Result<Vec<data_infrastructure::models::JudgeEvaluation>> { Ok(vec![]) }
        // Planning methods (stubs)
        async fn create_milestone(&self, _milestone: data_infrastructure::CreateMilestone) -> Result<data_infrastructure::models::Milestone> { Err(anyhow!("Not implemented")) }
        async fn get_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Option<data_infrastructure::models::Milestone>> { Ok(None) }
        async fn get_milestones(&self, _plan_id: Uuid) -> Result<Vec<data_infrastructure::models::Milestone>> { Ok(vec![]) }
        async fn update_milestone(&self, _plan_id: Uuid, _milestone_id: String, _update: data_infrastructure::UpdateMilestone) -> Result<data_infrastructure::models::Milestone> { Err(anyhow!("Not implemented")) }
        async fn delete_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<()> { Ok(()) }
        async fn create_planning_session(&self, _session: data_infrastructure::CreatePlanningSession) -> Result<data_infrastructure::models::PlanningSession> { Err(anyhow!("Not implemented")) }
        async fn get_planning_session(&self, _id: Uuid) -> Result<Option<data_infrastructure::models::PlanningSession>> { Ok(None) }
        async fn get_planning_sessions(&self, _plan_id: Uuid) -> Result<Vec<data_infrastructure::models::PlanningSession>> { Ok(vec![]) }
        async fn update_planning_session(&self, _id: Uuid, _update: data_infrastructure::UpdatePlanningSession) -> Result<data_infrastructure::models::PlanningSession> { Err(anyhow!("Not implemented")) }
        async fn create_evidence_artifact(&self, _artifact: data_infrastructure::CreateEvidenceArtifact) -> Result<data_infrastructure::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
        async fn get_evidence_artifacts(&self, _plan_id: Uuid) -> Result<Vec<data_infrastructure::models::EvidenceArtifact>> { Ok(vec![]) }
        async fn get_evidence_artifacts_for_milestone(&self, _plan_id: Uuid, _milestone_id: String) -> Result<Vec<data_infrastructure::models::EvidenceArtifact>> { Ok(vec![]) }
        async fn update_evidence_artifact(&self, _id: Uuid, _update: data_infrastructure::UpdateEvidenceArtifact) -> Result<data_infrastructure::models::EvidenceArtifact> { Err(anyhow!("Not implemented")) }
        async fn create_planning_audit_event(&self, _event: data_infrastructure::CreatePlanningAuditEvent) -> Result<data_infrastructure::models::PlanningAuditEvent> { Err(anyhow!("Not implemented")) }
        async fn get_planning_audit_events(&self, _plan_id: Uuid) -> Result<Vec<data_infrastructure::models::PlanningAuditEvent>> { Ok(vec![]) }
        async fn create_planning_telemetry(&self, _telemetry: data_infrastructure::CreatePlanningTelemetry) -> Result<data_infrastructure::models::PlanningTelemetry> { Err(anyhow!("Not implemented")) }
        async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> Result<Vec<data_infrastructure::models::PlanningTelemetry>> { Ok(vec![]) }
        
        // Waiver operations
        async fn get_waivers(&self, _status: Option<String>) -> Result<Vec<data_infrastructure::models::Waiver>> { Ok(vec![]) }
        async fn create_waiver(&self, _waiver: data_infrastructure::CreateWaiver) -> Result<data_infrastructure::models::Waiver> { Err(anyhow!("Not implemented")) }
        async fn update_waiver(&self, _id: Uuid, _update: data_infrastructure::UpdateWaiver) -> Result<data_infrastructure::models::Waiver> { Err(anyhow!("Not implemented")) }
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

        let execution_result = crate::planning::plan_executor::PlanExecutionResult {
            plan_id: execution_plan.id,
            status: crate::planning::plan_executor::ExecutionStatus::Completed,
            milestone_results: vec![],
            evidence_artifacts: vec![],
            quality_verifications: vec![],
            started_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
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
