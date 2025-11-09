//! Orchestration Service Adapter
//!
//! Adapts `agent-orchestration` implementations to `data-interfaces` service traits.

use async_trait::async_trait;
use data_interfaces::service_contracts::{
    OrchestrationService, ServiceError, TaskStatus, TaskStatusEnum,
};
use agent_agency_contracts::{
    WorkingSpec, TaskExecutionResult, TaskContext,
};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use agent_orchestration::{
    types::OrchestratorConfig,
    adapter::LegacyOrchestratorAdapter,
    orchestration::unified_orchestrator::UnifiedOrchestrator,
    orchestration::task_state_persistence::ExecutionStateStatus,
};
use chrono::Utc;
use anyhow::Result;
use tracing::warn;
use tokio::sync::RwLock;

/// Adapter for orchestration service using UnifiedOrchestrator
pub struct UnifiedOrchestratorAdapter {
    orchestrator: Arc<UnifiedOrchestrator>,
    /// Mapping of task_id -> plan_id for status queries
    task_to_plan: Arc<RwLock<HashMap<Uuid, Uuid>>>,
}

impl UnifiedOrchestratorAdapter {
    /// Create a new unified orchestrator adapter
    pub fn new(orchestrator: Arc<UnifiedOrchestrator>) -> Self {
        Self {
            orchestrator,
            task_to_plan: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get access to the underlying UnifiedOrchestrator
    /// This is needed to create UnifiedOrchestratorTaskExecutor
    pub fn orchestrator(&self) -> Arc<UnifiedOrchestrator> {
        Arc::clone(&self.orchestrator)
    }

    /// Create UnifiedOrchestratorAdapter from an existing UnifiedOrchestrator
    /// 
    /// This adapter wraps an existing UnifiedOrchestrator instance.
    /// To create a UnifiedOrchestrator, use UnifiedOrchestratorFactory from agent-orchestration crate.
    /// 
    /// # Arguments
    /// * `orchestrator` - Pre-created UnifiedOrchestrator instance
    pub fn from_orchestrator(orchestrator: Arc<UnifiedOrchestrator>) -> Self {
        Self {
            orchestrator,
            task_to_plan: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create UnifiedOrchestratorAdapter with all dependencies initialized
    /// 
    /// DEPRECATED: Use UnifiedOrchestratorFactory::create() from agent-orchestration crate instead,
    /// then call UnifiedOrchestratorAdapter::from_orchestrator().
    /// 
    /// This method is kept for backward compatibility but will be removed in a future version.
    /// 
    /// # Arguments
    /// * `db_client` - Optional database client from data-infrastructure. If None, uses stub implementation.
    #[deprecated(note = "Use UnifiedOrchestratorFactory::create() from agent-orchestration crate instead")]
    pub async fn create_with_dependencies(
        db_client: Option<Arc<data_infrastructure::simple_client::DatabaseClient>>,
    ) -> Result<Self, ServiceError> {
        use std::path::PathBuf;
        use agent_orchestration::{
            council::Council,
            council::CouncilConfig,
            decision_making::{ConsensusStrategy, RiskThresholds},
            verdict_aggregation::{VerdictAggregator, AggregationConfig, DissentHandling, RiskAggregationStrategy},
            decision_making::AlgorithmicDecisionEngine,
            judge_backup::{Judge, EthicsJudge, quality_judge::QualityAssuranceJudge, security_judge::SecurityJudge},
            judge_backup::JudgeConfig,
            judge_backup::backup_types::JudgeType,
            planning::{
                factory::PlanningSystemFactory,
                worktree_manager::{WorktreeManager, WorktreeManagerConfig},
                council_integration::{CouncilIntegration, CouncilIntegrationImpl},
                caws_adjudication_cycle::CawsAdjudicationCycle,
                caws_debate_scorer::CawsDebateScorer,
                worker_lifecycle_manager::WorkerLifecycleManager,
                worker_assignment::WorkerAssignmentStrategy,
                reflexive_learner::{ReflexiveLearner, LearningConfig},
            },
            orchestration::unified_orchestrator::UnifiedOrchestratorConfig,
            orchestration::task_state_persistence::InMemoryTaskStatePersistence,
            workers::execution_bridge::WorkerExecutionBridge,
        };
        use agent_workers::{MCPWorkerPool, TaskExecutor, WorkerPoolConfig};
        
        
        
        
        

        // Create Council (reusing pattern from LegacyOrchestratorAdapter)
        let council_config = CouncilConfig {
            session_timeout_seconds: 300,
            min_judges_required: 3,
            max_judges_per_session: 10,
            judge_selection_strategy: agent_orchestration::council::JudgeSelectionStrategy::AllAvailable,
            consensus_strategy: ConsensusStrategy::Majority,
            risk_thresholds: RiskThresholds::default(),
            enable_parallel_reviews: true,
            judge_timeout_seconds: 60,
            enable_circuit_breakers: true,
            enable_graceful_degradation: true,
            enable_error_recovery: true,
        };

        let judges: Vec<Arc<dyn Judge>> = vec![
            Arc::new(EthicsJudge::new(JudgeConfig {
                judge_id: "ethics-001".to_string(),
                name: "Ethics Judge".to_string(),
                judge_type: JudgeType::Ethics,
                specialization: "moral reasoning".to_string(),
                max_response_time_ms: 5000,
                health_check_interval_ms: 30000,
            })),
            Arc::new(QualityAssuranceJudge::new(JudgeConfig {
                judge_id: "qa-001".to_string(),
                name: "Quality Assurance Judge".to_string(),
                judge_type: JudgeType::Quality,
                specialization: "code quality".to_string(),
                max_response_time_ms: 3000,
                health_check_interval_ms: 30000,
            })),
            Arc::new(SecurityJudge::new(JudgeConfig {
                judge_id: "security-001".to_string(),
                name: "Security Judge".to_string(),
                judge_type: JudgeType::Security,
                specialization: "security analysis".to_string(),
                max_response_time_ms: 3000,
                health_check_interval_ms: 30000,
            })),
        ];

        let verdict_aggregator = Arc::new(VerdictAggregator::new(AggregationConfig {
            consensus_threshold: 0.7,
            weight_by_specialization: true,
            min_judges_required: 3,
            dissent_handling: DissentHandling::Strict,
            risk_aggregation: RiskAggregationStrategy::WeightedAverage,
        }));

        let decision_engine = Box::new(AlgorithmicDecisionEngine::new(ConsensusStrategy::Majority));
        // Clone council_config before moving it into Council::new (needed later in cfg block)
        let council_config_clone = council_config.clone();
        let council = Arc::new(Council::new(
            council_config,
            judges,
            verdict_aggregator,
            decision_engine,
        ));

        // Create database operations adapter
        // If db_client is provided, create an adapter; otherwise use stub
        // Clone db_client before moving it (needed later in cfg block)
        let db_client_clone = db_client.as_ref().map(|db| db.clone());
        let db_ops: Arc<dyn agent_orchestration::planning::DatabaseOperations> = if let Some(db_client) = db_client {
            // Use DatabaseOperationsAdapter - partial implementation with placeholders
            Arc::new(crate::database_operations_adapter::DatabaseOperationsAdapter::new(db_client))
        } else {
            Arc::new(StubDatabaseOperations)
        };

        struct StubDatabaseOperations;
        #[async_trait::async_trait]
        impl agent_orchestration::planning::DatabaseOperations for StubDatabaseOperations {
            async fn get_workers(&self) -> Result<Vec<agent_orchestration::planning::data_infrastructure_types::models::Worker>, anyhow::Error> {
                Ok(vec![])
            }
            // Implement other required methods with stubs
            async fn create_execution_plan(&self, _plan: agent_orchestration::planning::data_infrastructure_types::CreateExecutionPlan) -> Result<agent_orchestration::planning::data_infrastructure_types::models::ExecutionPlan, anyhow::Error> {
                Err(anyhow::anyhow!("Stub implementation"))
            }
            async fn get_execution_plan(&self, _id: Uuid) -> Result<Option<agent_orchestration::planning::data_infrastructure_types::models::ExecutionPlan>, anyhow::Error> {
                Ok(None)
            }
            async fn get_execution_plans(&self) -> Result<Vec<agent_orchestration::planning::data_infrastructure_types::models::ExecutionPlan>, anyhow::Error> {
                Ok(vec![])
            }
            async fn update_execution_plan(&self, _id: Uuid, _update: agent_orchestration::planning::data_infrastructure_types::UpdateExecutionPlan) -> Result<agent_orchestration::planning::data_infrastructure_types::models::ExecutionPlan, anyhow::Error> {
                Err(anyhow::anyhow!("Stub implementation"))
            }
            async fn create_audit_trail_entry(&self, _entry: agent_orchestration::planning::data_infrastructure_types::CreateAuditTrailEntry) -> Result<agent_orchestration::planning::data_infrastructure_types::models::AuditTrailEntry, anyhow::Error> {
                Err(anyhow::anyhow!("Stub implementation"))
            }
            async fn get_audit_trail_entries(&self, _task_id: Uuid) -> Result<Vec<agent_orchestration::planning::data_infrastructure_types::models::AuditTrailEntry>, anyhow::Error> {
                Ok(vec![])
            }
            async fn get_audit_trail_entry(&self, _id: Uuid) -> Result<Option<agent_orchestration::planning::data_infrastructure_types::models::AuditTrailEntry>, anyhow::Error> {
                Ok(None)
            }
            async fn create_planning_session(&self, _session: agent_orchestration::planning::data_infrastructure_types::CreatePlanningSession) -> Result<agent_orchestration::planning::data_infrastructure_types::models::PlanningSession, anyhow::Error> {
                Err(anyhow::anyhow!("Stub implementation"))
            }
            async fn get_planning_session(&self, _id: Uuid) -> Result<Option<agent_orchestration::planning::data_infrastructure_types::models::PlanningSession>, anyhow::Error> {
                Ok(None)
            }
            async fn update_planning_session(&self, _id: Uuid, _session: agent_orchestration::planning::data_infrastructure_types::UpdatePlanningSession) -> Result<(), anyhow::Error> {
                Ok(())
            }
            async fn create_planning_telemetry(&self, _telemetry: agent_orchestration::planning::data_infrastructure_types::CreatePlanningTelemetry) -> Result<agent_orchestration::planning::data_infrastructure_types::models::PlanningTelemetry, anyhow::Error> {
                Err(anyhow::anyhow!("Stub implementation"))
            }
            async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> Result<Vec<agent_orchestration::planning::data_infrastructure_types::models::PlanningTelemetry>, anyhow::Error> {
                Ok(vec![])
            }
            async fn create_planning_audit_event(&self, _event: agent_orchestration::planning::data_infrastructure_types::CreatePlanningAuditEvent) -> Result<(), anyhow::Error> {
                Ok(())
            }
            async fn get_planning_audit_events(&self, _plan_id: Uuid) -> Result<Vec<agent_orchestration::planning::data_infrastructure_types::models::PlanningAuditEvent>, anyhow::Error> {
                Ok(vec![])
            }
            async fn delete_execution_plan(&self, _id: Uuid) -> Result<(), anyhow::Error> {
                Ok(())
            }
            async fn get_judges(&self) -> Result<Vec<agent_orchestration::planning::data_infrastructure_types::models::Judge>, anyhow::Error> {
                Ok(vec![])
            }
            async fn create_judge(&self, _judge: agent_orchestration::planning::data_infrastructure_types::CreateJudge) -> Result<agent_orchestration::planning::data_infrastructure_types::models::Judge, anyhow::Error> {
                Err(anyhow::anyhow!("Stub implementation"))
            }
            async fn get_judge(&self, _id: Uuid) -> Result<Option<agent_orchestration::planning::data_infrastructure_types::models::Judge>, anyhow::Error> {
                Ok(None)
            }
            async fn create_judge_evaluation(&self, _evaluation: agent_orchestration::planning::data_infrastructure_types::CreateJudgeEvaluation) -> Result<agent_orchestration::planning::data_infrastructure_types::models::JudgeEvaluation, anyhow::Error> {
                Err(anyhow::anyhow!("Stub implementation"))
            }
            async fn get_judge_evaluations(&self, _task_id: Uuid) -> Result<Vec<agent_orchestration::planning::data_infrastructure_types::models::JudgeEvaluation>, anyhow::Error> {
                Ok(vec![])
            }
            async fn get_waivers(&self, _status: Option<String>) -> Result<Vec<agent_orchestration::planning::data_infrastructure_types::models::Waiver>, anyhow::Error> {
                Ok(vec![])
            }
            async fn create_waiver(&self, _waiver: agent_orchestration::planning::data_infrastructure_types::CreateWaiver) -> Result<agent_orchestration::planning::data_infrastructure_types::models::Waiver, anyhow::Error> {
                Err(anyhow::anyhow!("Stub implementation"))
            }
            async fn update_waiver(&self, _id: Uuid, _update: agent_orchestration::planning::data_infrastructure_types::UpdateWaiver) -> Result<agent_orchestration::planning::data_infrastructure_types::models::Waiver, anyhow::Error> {
                Err(anyhow::anyhow!("Stub implementation"))
            }
        }


        // Create planning components using PlanningSystemFactory
        // Research feature is in default features, so it's always available
        // Memory feature is optional
        #[cfg(feature = "research")]
        let research_collector = {
            use agent_research::evidence::collector::EvidenceCollector;
            Arc::new(EvidenceCollector::new())
        };
        
        #[cfg(feature = "memory")]
        let memory_system = {
            use agent_memory::MemorySystem;
            use agent_memory::MemoryConfig;
            Arc::new(MemorySystem::init(MemoryConfig::default()).await
                .map_err(|e| ServiceError::Internal(format!("Failed to initialize memory system: {}", e)))?)
        };

        // Call create_planning_components with conditional feature gates
        // Research is in default features, so it's always available
        // Memory is optional - function signature changes based on feature
        // Note: We require both research and memory features for UnifiedOrchestrator
        #[cfg(not(all(feature = "research", feature = "memory")))]
        {
            return Err(ServiceError::Internal(
                "Both research and memory features required for UnifiedOrchestrator initialization. \
                 Enable both features in Cargo.toml or use LegacyOrchestratorAdapter.".to_string()
            ));
        }
        
        #[cfg(all(feature = "research", feature = "memory"))]
        {
            let planning_components = PlanningSystemFactory::create_planning_components(
                research_collector,
                memory_system.clone(),
                council.clone(),
                db_ops.clone(),
                None, // worker_bridge - not available in deprecated method
                None, // worktree_manager - not available in deprecated method
            ).await
            .map_err(|e| ServiceError::Internal(format!("Failed to create planning components: {}", e)))?;

            // Create UnifiedOrchestratorConfig
            let config = UnifiedOrchestratorConfig {
                enable_council_review: true,
                enable_refinement: true,
                enable_worktree_isolation: true,
                worktree_base_path: PathBuf::from("/tmp/agent-agency-worktrees"),
                max_parallel_milestones: 5,
            };

            // Create WorktreeManager
            let worktree_config = WorktreeManagerConfig {
                worktree_base_path: config.worktree_base_path.clone(),
                main_repo_path: PathBuf::from("."),
                base_branch: "main".to_string(),
                auto_cleanup: true,
                max_concurrent_worktrees: 10,
            };
            let worktree_manager = Arc::new(WorktreeManager::new(worktree_config));

            // Create CAWS adjudication cycle
            let council_integration: Arc<dyn CouncilIntegration> = Arc::new(CouncilIntegrationImpl::new(
                council.clone(),
                council_config_clone.clone(),
            ));
            let debate_scorer = Arc::new(CawsDebateScorer::new(council.clone()));
            let adjudication_cycle = Arc::new(CawsAdjudicationCycle::new(
                council.clone(),
                council_integration.clone(),
                debate_scorer,
            ));

            // Create worker lifecycle manager
            let worker_lifecycle_manager = Arc::new(WorkerLifecycleManager::new(council_integration.clone()));

            // Create worker assignment strategy
            let worker_assignment_strategy = Arc::new(WorkerAssignmentStrategy::new(db_ops.clone()));

            // Create reflexive learner
            let reflexive_learner = Arc::new(ReflexiveLearner::new(
                worker_assignment_strategy.clone(),
                LearningConfig::default(),
            ));

            // Create worker bridge
            let worker_pool = Arc::new(MCPWorkerPool::new(WorkerPoolConfig::default()).await);
            // TaskExecutor requires a database client - use provided one or create stub
            let db_client_for_executor = if let Some(db) = db_client_clone {
                db
            } else {
                // TODO: Implement async database client creation
                //       Currently requires pre-provided client; should implement async database client creation handling DatabaseClient::new async requirement.
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
                // - Database client is created asynchronously
                // - Async creation is handled correctly
                // - Error handling works for creation failures
                // - Client is properly initialized
                //
                // DEPENDENCIES:
                // - DatabaseClient::new async API (Required)
                // - Async initialization utilities (Required)
                // - Error handling infrastructure (Required)
                //
                // ESTIMATED EFFORT: 3-4 hours (medium confidence)
                // PRIORITY: Medium
                // BLOCKING: No
                //
                // GOVERNANCE:
                // - CAWS Tier: 2 (database integration feature)
                // - Change Budget: ~80 LOC
                // - Reviewer Requirements: Async Rust and database expertise
                return Err(ServiceError::Internal( // Temporary: error until async client creation
                    "Database client required for TaskExecutor. Please provide a database client when creating UnifiedOrchestratorAdapter.".to_string()
                ));
            };
            let task_executor = Arc::new(TaskExecutor::new(db_client_for_executor));
            let worker_bridge = Arc::new(WorkerExecutionBridge::new(worker_pool, task_executor));

            // Extract PlanExecutor from parallel_coordinator
            // Note: ParallelCoordinator contains PlanExecutor, but UnifiedOrchestrator needs both
            // We'll create a separate PlanExecutor for UnifiedOrchestrator
            // The one inside parallel_coordinator is used by ParallelCoordinator itself
            use agent_orchestration::planning::plan_executor::{PlanExecutor, ExecutionConfig, WorkerPool, WorkerInfo, WorkerStatus, WorkerHealth};
            use agent_orchestration::audit_trail::{AuditTrailManager, AuditConfig};
            use agent_orchestration::planning::plan_types::ExecutionPlan;
            
            // Create stub worker pool (similar to factory.rs pattern)
            struct StubWorkerPool;
            #[async_trait::async_trait]
            impl WorkerPool for StubWorkerPool {
                async fn available_workers(&self) -> Result<Vec<WorkerInfo>> {
                    Ok(vec![])
                }
                async fn assign_worker(&self, _worker_id: Uuid, _milestone_id: String) -> Result<()> {
                    Ok(())
                }
                async fn release_worker(&self, _worker_id: Uuid) -> Result<()> {
                    Ok(())
                }
                async fn worker_status(&self, _worker_id: Uuid) -> Result<WorkerStatus> {
                    Ok(WorkerStatus {
                        current_assignment: None,
                        health: WorkerHealth::Healthy,
                        performance: agent_orchestration::planning::plan_executor::WorkerPerformance {
                            tasks_completed: 0,
                            tasks_failed: 0,
                            avg_completion_time_ms: 0.0,
                            success_rate: 1.0,
                        },
                    })
                }
            }
            let worker_pool = Arc::new(StubWorkerPool);
            
            // Create audit trail
            let audit_trail_manager = Arc::new(AuditTrailManager::new(AuditConfig::default()));
            struct AuditTrailAdapter {
                #[allow(dead_code)] // Reserved for future use
                manager: Arc<AuditTrailManager>,
            }
            #[async_trait::async_trait]
            impl agent_orchestration::planning::plan_executor::AuditTrail for AuditTrailAdapter {
                async fn log_event(&self, event: agent_orchestration::planning::plan_executor::AuditEvent) -> Result<()> {
                    // TODO: Convert and persist audit trail entry
                    //       Currently only logs via tracing; should convert audit event to audit trail entry and persist to database.
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
                    // - Audit events are converted correctly
                    // - Entries are persisted to database
                    // - Persistence works reliably
                    // - Error handling works for persistence failures
                    //
                    // DEPENDENCIES:
                    // - Database connection (Required)
                    // - Audit trail entry conversion utilities (Required)
                    // - Database persistence infrastructure (Required)
                    //
                    // ESTIMATED EFFORT: 3-4 hours (medium confidence)
                    // PRIORITY: Medium
                    // BLOCKING: No
                    //
                    // GOVERNANCE:
                    // - CAWS Tier: 2 (audit trail feature)
                    // - Change Budget: ~80 LOC
                    // - Reviewer Requirements: Database and audit trail expertise
                    tracing::info!("Audit event: {:?}", event); // Temporary: tracing log until database persistence
                    Ok(())
                }
            }
            let audit_trail = Arc::new(AuditTrailAdapter {
                manager: audit_trail_manager.clone(),
            }) as Arc<dyn agent_orchestration::planning::plan_executor::AuditTrail>;
            
            // Create PlanExecutor for UnifiedOrchestrator
            let plan_executor = Arc::new(PlanExecutor::new(
                ExecutionPlan::default(),
                worker_pool,
                planning_components.evidence_collector.clone(),
                planning_components.worker_assigner.clone(),
                planning_components.scope_guard.clone(),
                planning_components.council_monitor.clone(),
                Arc::downgrade(&planning_components.parallel_coordinator),
                audit_trail,
                Some(audit_trail_manager),
                planning_components.todo_integration.clone(),
                ExecutionConfig::default(),
            ));

            // Create state persistence for pause/resume/cancel support
            let state_persistence = Arc::new(InMemoryTaskStatePersistence::new());

            // Create UnifiedOrchestrator
            let orchestrator = Arc::new(UnifiedOrchestrator::new(
                config,
                planning_components.plan_generator,
                plan_executor,
                planning_components.parallel_coordinator,
                council,
                worker_bridge,
                None, // refinement_coordinator - optional
                worktree_manager,
                Some(adjudication_cycle),
                worker_lifecycle_manager,
                Some(worker_assignment_strategy),
                Some(reflexive_learner),
                #[cfg(feature = "memory")]
                Some(memory_system),
                None, // turn_level_tracker - optional
                None, // session_manager - optional
                Some(state_persistence), // Enable state persistence for pause/resume/cancel
                None, // federated_learning - optional
            ));

            Ok(Self {
                orchestrator,
                task_to_plan: Arc::new(RwLock::new(HashMap::new())),
            })
        }
    }
}

#[async_trait]
impl OrchestrationService for UnifiedOrchestratorAdapter {
    async fn orchestrate_task(
        &self,
        spec: WorkingSpec,
        context: TaskContext,
    ) -> Result<TaskExecutionResult, ServiceError> {
        // Execute plan using UnifiedOrchestrator
        let execution_result = self.orchestrator
            .execute_plan(spec.clone())
            .await
            .map_err(|e| ServiceError::Internal(format!("Orchestration failed: {}", e)))?;

        // Convert ExecutionResult to TaskExecutionResult
        let success = execution_result.final_verdict
            .as_ref()
            .map(|v| matches!(v.decision, agent_agency_contracts::final_verdict::FinalDecision::Accept))
            .unwrap_or(false);

        let errors = if success {
            Vec::new()
        } else {
            execution_result.final_verdict
                .as_ref()
                .map(|v| {
                    if v.dissent.is_empty() {
                        "Execution completed with warnings".to_string()
                    } else {
                        v.dissent.clone()
                    }
                })
                .map(|r| vec![r])
                .unwrap_or_else(|| vec!["Execution completed with warnings".to_string()])
        };

        let output = serde_json::to_string(&execution_result.artifacts)
            .unwrap_or_else(|_| "Failed to serialize artifacts".to_string());

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("plan_id".to_string(), serde_json::json!(execution_result.plan_id.to_string()));
        metadata.insert("iterations".to_string(), serde_json::json!(execution_result.iterations));
        metadata.insert("quality_scores".to_string(), serde_json::json!(execution_result.quality_scores));
        if let Some(ref verdict) = execution_result.final_verdict {
            metadata.insert("verdict_status".to_string(), serde_json::json!(format!("{:?}", verdict.decision)));
        }

        // Generate task_id from working spec ID
        // Working spec ID format: TASK-<UUID>
        let task_id = if spec.id.starts_with("TASK-") {
            spec.id.strip_prefix("TASK-")
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(|| execution_result.plan_id)
        } else {
            // Fallback: use plan_id as task_id
            execution_result.plan_id
        };

        // Store task_id -> plan_id mapping for status queries
        {
            let mut mapping = self.task_to_plan.write().await;
            mapping.insert(task_id, execution_result.plan_id);
        }

        Ok(TaskExecutionResult {
            execution_id: execution_result.plan_id,
            task_id,
            success,
            output,
            errors,
            metadata,
            started_at: Utc::now(), // TODO: Track actual start time
            completed_at: Utc::now(),
            duration_ms: 0, // TODO: Calculate actual duration
            worker_id: None, // UnifiedOrchestrator uses multiple workers
        })
    }
    
    async fn get_task_status(&self, task_id: &Uuid) -> Result<TaskStatus, ServiceError> {
        // Look up plan_id from task_id mapping
        let plan_id = {
            let mapping = self.task_to_plan.read().await;
            mapping.get(task_id).copied()
        };

        if let Some(plan_id) = plan_id {
            // Query UnifiedOrchestrator for execution status
            match self.orchestrator.get_execution_status(plan_id).await {
                Ok(Some(state)) => {
                    // Convert ExecutionStateStatus to TaskStatusEnum
                    let status = match state.status {
                        ExecutionStateStatus::Pending => TaskStatusEnum::Pending,
                        ExecutionStateStatus::Running => TaskStatusEnum::Running,
                        ExecutionStateStatus::Paused => TaskStatusEnum::Paused,
                        ExecutionStateStatus::Completed => TaskStatusEnum::Completed,
                        ExecutionStateStatus::Failed => TaskStatusEnum::Failed,
                        ExecutionStateStatus::Cancelled => TaskStatusEnum::Cancelled,
                        ExecutionStateStatus::Crashed => TaskStatusEnum::Failed, // Treat crashed as failed
                    };

                    Ok(TaskStatus {
                        task_id: *task_id,
                        status,
                        progress_percent: Some(state.progress_percentage as u8),
                        error_message: state.error,
                        created_at: state.created_at,
                        updated_at: state.last_updated,
                    })
                }
                Ok(None) => {
                    // State not found, return pending status
                    Ok(TaskStatus {
                        task_id: *task_id,
                        status: TaskStatusEnum::Pending,
                        progress_percent: None,
                        error_message: Some("Task state not found".to_string()),
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    })
                }
                Err(e) => {
                    warn!("Failed to get execution status for plan {}: {}", plan_id, e);
                    Err(ServiceError::Internal(format!("Failed to get status: {}", e)))
                }
            }
        } else {
            // Task ID not found in mapping, return pending status
            Ok(TaskStatus {
                task_id: *task_id,
                status: TaskStatusEnum::Pending,
                progress_percent: None,
                error_message: Some("Task not found".to_string()),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }
    }
    
    async fn pause_task(&self, task_id: &Uuid) -> Result<(), ServiceError> {
        // Look up plan_id from task_id mapping
        let plan_id = {
            let mapping = self.task_to_plan.read().await;
            mapping.get(task_id).copied()
        };

        if let Some(plan_id) = plan_id {
            self.orchestrator.pause_execution(plan_id).await
                .map_err(|e| ServiceError::Internal(format!("Failed to pause task: {}", e)))
        } else {
            Err(ServiceError::Internal(format!("Task {} not found", task_id)))
        }
    }
    
    async fn resume_task(&self, task_id: &Uuid) -> Result<(), ServiceError> {
        // Look up plan_id from task_id mapping
        let plan_id = {
            let mapping = self.task_to_plan.read().await;
            mapping.get(task_id).copied()
        };

        if let Some(plan_id) = plan_id {
            self.orchestrator.resume_execution(plan_id).await
                .map_err(|e| ServiceError::Internal(format!("Failed to resume task: {}", e)))
        } else {
            Err(ServiceError::Internal(format!("Task {} not found", task_id)))
        }
    }
    
    async fn cancel_task(&self, task_id: &Uuid) -> Result<(), ServiceError> {
        // Look up plan_id from task_id mapping
        let plan_id = {
            let mapping = self.task_to_plan.read().await;
            mapping.get(task_id).copied()
        };

        if let Some(plan_id) = plan_id {
            self.orchestrator.cancel_execution(plan_id).await
                .map_err(|e| ServiceError::Internal(format!("Failed to cancel task: {}", e)))
        } else {
            Err(ServiceError::Internal(format!("Task {} not found", task_id)))
        }
    }
}

/// Legacy adapter for orchestration service (kept for backward compatibility)
pub struct OrchestrationServiceAdapter {
    adapter: Arc<LegacyOrchestratorAdapter>,
}

impl OrchestrationServiceAdapter {
    /// Create a new orchestration service adapter
    pub async fn new(config: OrchestratorConfig) -> Result<Self, ServiceError> {
        let adapter = LegacyOrchestratorAdapter::new(config)
            .await
            .map_err(|e| ServiceError::Internal(format!("Failed to create adapter: {}", e)))?;
        Ok(Self {
            adapter: Arc::new(adapter),
        })
    }
    
    /// Create with default configuration
    pub async fn with_defaults() -> Result<Self, ServiceError> {
        Self::new(OrchestratorConfig::default()).await
    }
}

#[async_trait]
impl OrchestrationService for OrchestrationServiceAdapter {
    async fn orchestrate_task(
        &self,
        spec: WorkingSpec,
        context: TaskContext,
    ) -> Result<TaskExecutionResult, ServiceError> {
        // Convert TaskContext to TaskDescriptor
        use agent_agency_contracts::types::planning::{TaskDescriptor, BlastRadius, RiskTier};
        use agent_agency_contracts::planning_io::ChangeBudget;
        use agent_agency_contracts::task_request::ScopeRestrictions;
        
        let task_descriptor = TaskDescriptor {
            task_id: context.task_id,
            description: format!("Orchestrate task {}", context.task_id),
            change_budget: ChangeBudget {
                max_files: 25,
                max_loc: 1000,
                max_migrations: 0,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
            },
            priority: agent_agency_contracts::types::planning::TaskPriority::Normal,
            execution_mode: agent_agency_contracts::types::planning::ExecutionMode::Auto,
            risk_tier: Some(RiskTier::Tier2),
            blast_radius: BlastRadius {
                modules: vec![],
                data_migration: false,
                external_deps: vec![],
            },
            scope_in: ScopeRestrictions {
                allowed_paths: vec![],
                blocked_paths: vec![],
            },
            scope_out: None,
            acceptance: None,
        };
        
        // Create diff stats placeholder
        use agent_orchestration::types::DiffStats;
        let diff_stats = DiffStats {
            files_changed: 0,
            lines_added: 0,
            lines_removed: 0,
            lines_modified: 0,
            files_added: 0,
            files_modified: 0,
            files_deleted: 0,
            lines_deleted: 0,
            binary_files_changed: 0,
        };
        
        // Call orchestration adapter
        self.adapter.orchestrate_task(
            &spec,
            &task_descriptor,
            &diff_stats,
            false, // tests_added
            true,  // deterministic
        ).await
        .map_err(|e| ServiceError::Internal(format!("Orchestration failed: {}", e)))
    }
    
    async fn get_task_status(&self, task_id: &Uuid) -> Result<TaskStatus, ServiceError> {
        // TODO: Retrieve actual task status from database
        //       Currently returns placeholder; should retrieve actual task status from database for the given task ID.
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
        // - Task status is retrieved from database correctly
        // - Status information is accurate
        // - Missing tasks are handled gracefully
        // - Error handling works for query failures
        //
        // DEPENDENCIES:
        // - Database connection (Required)
        // - Task status table schema (Required)
        // - Status query utilities (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (database query feature)
        // - Change Budget: ~80 LOC
        // - Reviewer Requirements: Database and task management expertise
        Ok(TaskStatus { // Temporary: placeholder until database retrieval
            task_id: *task_id,
            status: TaskStatusEnum::Running,
            progress_percent: None,
            error_message: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
    
    async fn pause_task(&self, _task_id: &Uuid) -> Result<(), ServiceError> {
        // TODO: Implement actual pause logic
        Err(ServiceError::Internal("Pause not yet implemented".to_string()))
    }
    
    async fn resume_task(&self, _task_id: &Uuid) -> Result<(), ServiceError> {
        // TODO: Implement actual resume logic
        Err(ServiceError::Internal("Resume not yet implemented".to_string()))
    }
    
    async fn cancel_task(&self, _task_id: &Uuid) -> Result<(), ServiceError> {
        // TODO: Implement actual cancel logic
        Err(ServiceError::Internal("Cancel not yet implemented".to_string()))
    }
}


