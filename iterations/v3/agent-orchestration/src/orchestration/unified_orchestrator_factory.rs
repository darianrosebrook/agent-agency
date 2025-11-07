//! Unified Orchestrator Factory
//!
//! Factory for creating UnifiedOrchestrator instances with all dependencies initialized.
//! This factory is in agent-orchestration to avoid circular dependencies with data-interfaces-adapters.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use std::path::PathBuf;
use anyhow::Result;
use uuid::Uuid;
use tracing::info;

use crate::orchestration::unified_orchestrator::{UnifiedOrchestrator, UnifiedOrchestratorConfig};
use crate::planning::factory::PlanningSystemFactory;
use crate::council::{Council, CouncilConfig};
use crate::decision_making::{ConsensusStrategy, RiskThresholds};
use crate::verdict_aggregation::{VerdictAggregator, AggregationConfig, DissentHandling, RiskAggregationStrategy};
use crate::decision_making::AlgorithmicDecisionEngine;
use crate::judge_backup::{Judge, EthicsJudge, quality_judge::QualityAssuranceJudge, security_judge::SecurityJudge};
use crate::judge_backup::JudgeConfig;
use crate::judge_backup::backup_types::JudgeType;
use crate::planning::{
    plan_generator::PlanGenerator,
    worktree_manager::{WorktreeManager, WorktreeManagerConfig},
    caws_adjudication_cycle::{CawsAdjudicationCycle, CawsDebateScorer},
    council_integration::{CouncilIntegration, CouncilIntegrationImpl},
    worker_lifecycle_manager::WorkerLifecycleManager,
    worker_assignment::WorkerAssignmentStrategy,
    reflexive_learner::{ReflexiveLearner, LearningConfig},
    plan_executor::{PlanExecutor, ExecutionConfig, WorkerPool, WorkerInfo, WorkerStatus, WorkerHealth},
};
use crate::workers::execution_bridge::WorkerExecutionBridge;
use crate::orchestration::task_state_persistence::InMemoryTaskStatePersistence;
use crate::planning::plan_types::ExecutionPlan;
use crate::planning::DatabaseOperations;
use agent_workers::{MCPWorkerPool, TaskExecutor, WorkerPoolConfig};
use async_trait::async_trait;

/// Factory for creating UnifiedOrchestrator instances
pub struct UnifiedOrchestratorFactory;

impl UnifiedOrchestratorFactory {
    /// Create a UnifiedOrchestrator with all dependencies initialized
    ///
    /// # Arguments
    /// * `db_ops` - Optional database operations adapter. If None, uses stub implementation.
    ///
    /// # Returns
    /// * `Arc<UnifiedOrchestrator>` - Fully configured orchestrator instance
    pub async fn create(
        db_ops: Option<Arc<dyn DatabaseOperations>>,
    ) -> Result<Arc<UnifiedOrchestrator>> {
        info!("Creating UnifiedOrchestrator with all dependencies...");

        // Create Council
        let council_config = CouncilConfig {
            session_timeout_seconds: 300,
            min_judges_required: 3,
            max_judges_per_session: 10,
            judge_selection_strategy: crate::council::JudgeSelectionStrategy::AllAvailable,
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
        let council = Arc::new(Council::new(
            council_config.clone(),
            judges,
            verdict_aggregator,
            decision_engine,
        ));

        // Create database operations stub if not provided
        let db_ops = if let Some(db_ops) = db_ops {
            db_ops
        } else {
            Arc::new(StubDatabaseOperations)
        };

        // Create planning components using PlanningSystemFactory
        // Research is in default features, so it's always available
        #[cfg(feature = "research")]
        let research_collector = {
            use agent_research::evidence::collector::EvidenceCollector;
            Arc::new(EvidenceCollector::new())
        };

        #[cfg(not(feature = "research"))]
        {
            return Err(anyhow::anyhow!(
                "Research feature required for UnifiedOrchestrator initialization. \
                 This should be enabled by default."
            ));
        }

        // Memory is optional - create if feature enabled
        #[cfg(feature = "memory")]
        let memory_system = {
            use agent_memory::{MemorySystem, MemoryConfig};
            Arc::new(MemorySystem::init(MemoryConfig::default()).await?)
        };

        // Create planning components - requires both research and memory features
        #[cfg(all(feature = "research", feature = "memory"))]
        let planning_components = PlanningSystemFactory::create_planning_components(
            research_collector,
            memory_system,
            council.clone(),
            db_ops.clone(),
        ).await?;

        #[cfg(all(feature = "research", not(feature = "memory")))]
        {
            return Err(anyhow::anyhow!(
                "Memory feature required for UnifiedOrchestrator initialization. \
                 Enable memory feature in Cargo.toml or use LegacyOrchestratorAdapter."
            ));
        }

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
            council_config.clone(),
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
        let worker_pool = Arc::new(MCPWorkerPool::new(WorkerPoolConfig::default()).await?);
        let task_executor = Arc::new(TaskExecutor::new().await?);
        let worker_bridge = Arc::new(WorkerExecutionBridge::new(worker_pool, task_executor));

        // Create stub worker pool for PlanExecutor
        struct StubWorkerPool;
        #[async_trait]
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
                    performance: crate::planning::plan_executor::WorkerPerformance {
                        tasks_completed: 0,
                        tasks_failed: 0,
                        avg_completion_time_ms: 0.0,
                        success_rate: 1.0,
                    },
                })
            }
        }
        let executor_worker_pool = Arc::new(StubWorkerPool);

        // Create audit trail adapter
        use crate::audit_trail::{AuditTrailManager, AuditConfig};
        let audit_trail_manager = Arc::new(AuditTrailManager::new(AuditConfig::default()));
        struct AuditTrailAdapter {
            manager: Arc<AuditTrailManager>,
        }
        #[async_trait]
        impl crate::planning::plan_executor::AuditTrail for AuditTrailAdapter {
            async fn log_event(&self, event: crate::planning::plan_executor::AuditEvent) -> Result<()> {
                tracing::info!("Audit event: {:?}", event);
                Ok(())
            }
        }
        let audit_trail = Arc::new(AuditTrailAdapter {
            manager: audit_trail_manager.clone(),
        }) as Arc<dyn crate::planning::plan_executor::AuditTrail>;

        // Create PlanExecutor for UnifiedOrchestrator
        let plan_executor = Arc::new(PlanExecutor::new(
            ExecutionPlan::default(),
            executor_worker_pool,
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
            #[cfg(not(feature = "memory"))]
            None,
            None, // turn_level_tracker - optional
            None, // session_manager - optional
            Some(state_persistence), // Enable state persistence for pause/resume/cancel
            None, // federated_learning - optional
        ));

        info!("UnifiedOrchestrator created successfully");
        Ok(orchestrator)
    }
}

/// Stub database operations implementation
struct StubDatabaseOperations;

#[async_trait]
impl DatabaseOperations for StubDatabaseOperations {
    async fn get_workers(&self) -> Result<Vec<crate::planning::data_infrastructure_types::models::Worker>> {
        Ok(vec![])
    }
    async fn create_execution_plan(&self, _plan: crate::planning::data_infrastructure_types::CreateExecutionPlan) -> Result<crate::planning::data_infrastructure_types::models::ExecutionPlan> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_execution_plan(&self, _id: Uuid) -> Result<Option<crate::planning::data_infrastructure_types::models::ExecutionPlan>> {
        Ok(None)
    }
    async fn get_execution_plans(&self) -> Result<Vec<crate::planning::data_infrastructure_types::models::ExecutionPlan>> {
        Ok(vec![])
    }
    async fn update_execution_plan(&self, _id: Uuid, _update: crate::planning::data_infrastructure_types::UpdateExecutionPlan) -> Result<crate::planning::data_infrastructure_types::models::ExecutionPlan> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn create_audit_trail_entry(&self, _entry: crate::planning::data_infrastructure_types::CreateAuditTrailEntry) -> Result<crate::planning::data_infrastructure_types::models::AuditTrailEntry> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_audit_trail_entries(&self, _task_id: Uuid) -> Result<Vec<crate::planning::data_infrastructure_types::models::AuditTrailEntry>> {
        Ok(vec![])
    }
    async fn get_audit_trail_entry(&self, _id: Uuid) -> Result<Option<crate::planning::data_infrastructure_types::models::AuditTrailEntry>> {
        Ok(None)
    }
    async fn create_planning_session(&self, _session: crate::planning::data_infrastructure_types::CreatePlanningSession) -> Result<crate::planning::data_infrastructure_types::models::PlanningSession> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_planning_session(&self, _id: Uuid) -> Result<Option<crate::planning::data_infrastructure_types::models::PlanningSession>> {
        Ok(None)
    }
    async fn update_planning_session(&self, _id: Uuid, _session: crate::planning::data_infrastructure_types::UpdatePlanningSession) -> Result<()> {
        Ok(())
    }
    async fn create_planning_telemetry(&self, _telemetry: crate::planning::data_infrastructure_types::CreatePlanningTelemetry) -> Result<crate::planning::data_infrastructure_types::models::PlanningTelemetry> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_planning_telemetry(&self, _plan_id: Uuid, _metric_type: Option<String>) -> Result<Vec<crate::planning::data_infrastructure_types::models::PlanningTelemetry>> {
        Ok(vec![])
    }
    async fn create_planning_audit_event(&self, _event: crate::planning::data_infrastructure_types::CreatePlanningAuditEvent) -> Result<()> {
        Ok(())
    }
    async fn get_planning_audit_events(&self, _plan_id: Uuid) -> Result<Vec<crate::planning::data_infrastructure_types::models::PlanningAuditEvent>> {
        Ok(vec![])
    }
    async fn delete_execution_plan(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }
    async fn get_judges(&self) -> Result<Vec<crate::planning::data_infrastructure_types::models::Judge>> {
        Ok(vec![])
    }
    async fn create_judge(&self, _judge: crate::planning::data_infrastructure_types::CreateJudge) -> Result<crate::planning::data_infrastructure_types::models::Judge> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_judge(&self, _id: Uuid) -> Result<Option<crate::planning::data_infrastructure_types::models::Judge>> {
        Ok(None)
    }
    async fn create_judge_evaluation(&self, _evaluation: crate::planning::data_infrastructure_types::CreateJudgeEvaluation) -> Result<crate::planning::data_infrastructure_types::models::JudgeEvaluation> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_judge_evaluations(&self, _task_id: Uuid) -> Result<Vec<crate::planning::data_infrastructure_types::models::JudgeEvaluation>> {
        Ok(vec![])
    }
    async fn get_waivers(&self, _status: Option<String>) -> Result<Vec<crate::planning::data_infrastructure_types::models::Waiver>> {
        Ok(vec![])
    }
    async fn create_waiver(&self, _waiver: crate::planning::data_infrastructure_types::CreateWaiver) -> Result<crate::planning::data_infrastructure_types::models::Waiver> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn update_waiver(&self, _id: Uuid, _update: crate::planning::data_infrastructure_types::UpdateWaiver) -> Result<crate::planning::data_infrastructure_types::models::Waiver> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
}

