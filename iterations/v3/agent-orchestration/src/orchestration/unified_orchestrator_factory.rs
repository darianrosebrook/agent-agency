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
use crate::council::{Council, CouncilConfig};
use crate::decision_making::{ConsensusStrategy, RiskThresholds};
use crate::verdict_aggregation::{VerdictAggregator, AggregationConfig, DissentHandling, RiskAggregationStrategy};
use crate::decision_making::AlgorithmicDecisionEngine;
use crate::judge_backup::{Judge, EthicsJudge, quality_judge::QualityAssuranceJudge, security_judge::SecurityJudge};
use crate::judge_backup::JudgeConfig;
use crate::judge_backup::backup_types::JudgeType;
use crate::planning::{
    worktree_manager::{WorktreeManager, WorktreeManagerConfig},
    caws_adjudication_cycle::CawsAdjudicationCycle,
    caws_debate_scorer::CawsDebateScorer,
    council_integration::{CouncilIntegration, CouncilIntegrationImpl},
    worker_lifecycle_manager::WorkerLifecycleManager,
    worker_assignment::WorkerAssignmentStrategy,
    reflexive_learner::{ReflexiveLearner, LearningConfig},
    plan_executor::{WorkerPool, WorkerInfo, WorkerStatus, WorkerHealth, PlanExecutor, ExecutionConfig},
    factory::PlanningSystemFactory,
};
use crate::orchestration::task_state_persistence::{InMemoryTaskStatePersistence, TaskStatePersistence, DatabaseTaskStatePersistence};
use crate::planning::{DatabaseOperations, plan_types::ExecutionPlan};
use crate::workers::execution_bridge::WorkerExecutionBridge;
use agent_workers::{TaskExecutor, MCPWorkerPool, WorkerPoolConfig, WorkerSpecialty};
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

        // Create UnifiedOrchestratorConfig (needed for worktree_manager)
        let config = UnifiedOrchestratorConfig {
            enable_council_review: true,
            enable_refinement: true,
            enable_worktree_isolation: true,
            worktree_base_path: PathBuf::from("/tmp/agent-agency-worktrees"),
            max_parallel_milestones: 5,
        };

        // Create WorktreeManager (needed for PlanExecutor)
        let worktree_config = WorktreeManagerConfig {
            worktree_base_path: config.worktree_base_path.clone(),
            main_repo_path: PathBuf::from("."),
            base_branch: "main".to_string(),
            auto_cleanup: true,
            max_concurrent_worktrees: 10,
        };
        let worktree_manager = Arc::new(WorktreeManager::new(worktree_config));

        // Create worker bridge (needed for PlanExecutor)
        // Create database client using DATABASE_URL from environment or default
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());
        let db_config = data_infrastructure::DatabaseConfig {
            database_url: database_url.clone(),
            pool_max: Some(10),
            connection_timeout: Some(30),
            query_timeout: Some(60),
            ..Default::default()
        };
        let db_client = Arc::new(data_infrastructure::DatabaseClient::new(db_config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create database client: {}", e))?);
        
        // Clone db_client for TaskExecutor (it will be moved)
        let db_client_for_executor = db_client.clone();
        
        // Create ToolRegistry with real FileOperationsService for MCP tools
        // Use helper function from agent-workers that has access to both agent_mcp and data-infrastructure
        let repo_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."));
        let tool_registry = agent_workers::create_tool_registry_with_file_ops(repo_path).await
            .map_err(|e| anyhow::anyhow!("Failed to create tool registry with file operations: {}", e))?;
        
        // Use the existing memory_system for workers (already created above)
        #[cfg(not(feature = "memory"))]
        {
            return Err(anyhow::anyhow!("Memory feature required for UnifiedOrchestrator initialization"));
        }
        
        #[cfg(feature = "memory")]
        let shared_memory = memory_system.clone();
        
        #[cfg(feature = "memory")]
        // Create worker pool with the initialized tool registry and shared memory
        let worker_pool = Arc::new(MCPWorkerPool::new_with_registry(
            WorkerPoolConfig::default(),
            tool_registry,
            shared_memory,
        ));
        
        // Register a default worker in the pool to handle tasks
        // This matches the worker in the database (Default MCP Worker)
        use agent_workers::WorkerCapabilities;
        let default_capabilities = WorkerCapabilities {
            languages: vec!["python".to_string(), "rust".to_string(), "typescript".to_string()],
            frameworks: vec![],
            domains: vec!["code_generation".to_string(), "file_operations".to_string()],
            max_context_length: 8192,
            max_output_length: 4096,
            supported_formats: vec!["text".to_string(), "json".to_string()],
            caws_awareness: 0.8,
            quality_score: 0.9,
            speed_score: 0.7,
        };
        
        #[cfg(feature = "memory")]
        worker_pool.register_worker(WorkerSpecialty::General, default_capabilities).await
            .map_err(|e| anyhow::anyhow!("Failed to register default worker: {}", e))?;
        
        let task_executor = Arc::new(TaskExecutor::new(db_client_for_executor));
        
        #[cfg(feature = "memory")]
        let worker_bridge = Arc::new(WorkerExecutionBridge::new(worker_pool, task_executor));

        // Create planning components - requires both research and memory features
        // Pass worker_bridge and worktree_manager so PlanExecutor has real execution capabilities
        #[cfg(all(feature = "research", feature = "memory"))]
        let planning_components = PlanningSystemFactory::create_planning_components(
            research_collector,
            memory_system.clone(),
            council.clone(),
            db_ops.clone(),
            Some(worker_bridge.clone()), // Pass WorkerExecutionBridge
            Some(worktree_manager.clone()), // Pass WorktreeManager
        ).await?;

        #[cfg(not(all(feature = "research", feature = "memory")))]
        {
            return Err(anyhow::anyhow!(
                "Both research and memory features required for UnifiedOrchestrator initialization. \
                 Enable both features in Cargo.toml or use LegacyOrchestratorAdapter."
            ));
        }

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

        // Create PerformanceTracker if research feature is enabled
        #[cfg(feature = "research")]
        let performance_tracker = {
            use agent_research::performance_tracker::PerformanceTracker;
            Some(Arc::new(PerformanceTracker::new()))
        };
        
        #[cfg(not(feature = "research"))]
        let performance_tracker = None;

        // Create worker assignment strategy with PerformanceTracker if available
        #[cfg(feature = "research")]
        let worker_assignment_strategy = {
            if let Some(ref tracker) = performance_tracker {
                Arc::new(WorkerAssignmentStrategy::with_performance_tracker(
                    db_ops.clone(),
                    crate::planning::worker_assignment::AssignmentConfig::default(),
                    tracker.clone(),
                ))
            } else {
                Arc::new(WorkerAssignmentStrategy::new(db_ops.clone()))
            }
        };
        
        #[cfg(not(feature = "research"))]
        let worker_assignment_strategy = Arc::new(WorkerAssignmentStrategy::new(db_ops.clone()));

        // Create reflexive learner
        let reflexive_learner = Arc::new(ReflexiveLearner::new(
            worker_assignment_strategy.clone(),
            LearningConfig::default(),
        ));
        
        // Clone for continuous learning loop (needed outside the cfg block)
        #[cfg(all(feature = "research", feature = "memory"))]
        let reflexive_learner_for_loop = Arc::clone(&reflexive_learner);

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
            #[allow(dead_code)] // Reserved for future use
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

        // Create PlanExecutor for UnifiedOrchestrator with WorkerExecutionBridge and WorktreeManager
        #[cfg(all(feature = "research", feature = "memory"))]
        let plan_executor = Arc::new(PlanExecutor::with_lifecycle_manager(
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
            None, // worker_lifecycle_manager - will be set separately
            Some(worker_bridge.clone()), // Pass WorkerExecutionBridge for real execution
            Some(worktree_manager.clone()), // Pass WorktreeManager for worktree path resolution
            ExecutionConfig::default(),
        ));

        // Create state persistence for pause/resume/cancel support
        // Use database persistence when db_client is available, otherwise use in-memory
        // Database persistence provides crash recovery and task resumption capabilities
        let state_persistence: Arc<dyn TaskStatePersistence> = Arc::new(DatabaseTaskStatePersistence::new(db_client.clone()));

        // Create UnifiedOrchestrator
        #[cfg(all(feature = "research", feature = "memory"))]
        let orchestrator = {
            // Create ArbiterPipelineOptimizer if runtime-optimization feature is enabled
            #[cfg(feature = "runtime-optimization")]
            let arbiter_optimizer = {
                use system_federated_ml::{ArbiterPipelineOptimizer, DecisionPipelineConfig};
                match ArbiterPipelineOptimizer::new(DecisionPipelineConfig::default()).await {
                    Ok(optimizer) => {
                        info!("ArbiterPipelineOptimizer created successfully");
                        Some(Arc::new(optimizer))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create ArbiterPipelineOptimizer: {}", e);
                        None
                    }
                }
            };
            
            Arc::new(UnifiedOrchestrator::new(
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
                #[cfg(feature = "runtime-optimization")]
                arbiter_optimizer,
            ))
        };

        #[cfg(all(feature = "research", feature = "memory"))]
        {
            info!("UnifiedOrchestrator created successfully");
            
            // Start continuous learning loop for ReflexiveLearner
            // Start continuous learning loop with 60 second interval
            // This will periodically analyze accumulated outcomes and update routing policies
            if let Err(e) = reflexive_learner_for_loop.start_continuous_learning(60).await {
                warn!("Failed to start ReflexiveLearner continuous learning loop: {}", e);
            } else {
                info!("ReflexiveLearner continuous learning loop started");
            }
            
            Ok(orchestrator)
        }
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
    async fn create_execution_result(&self, _result: crate::planning::data_infrastructure_types::CreateExecutionResult) -> Result<crate::planning::data_infrastructure_types::models::PlanExecutionResult> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_execution_result(&self, _plan_id: Uuid) -> Result<Option<crate::planning::data_infrastructure_types::models::PlanExecutionResult>> {
        Ok(None)
    }
}

