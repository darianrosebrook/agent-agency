//! Unified Orchestrator Factory
//!
//! Factory for creating UnifiedOrchestrator instances with all dependencies initialized.
//! This factory is in agent-orchestration to avoid circular dependencies with data-interfaces-adapters.
//!
//! @author @darianrosebrook

use anyhow::{anyhow, Result};
use sqlx;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::council::{Council, CouncilConfig};
use crate::decision_making::AlgorithmicDecisionEngine;
use crate::decision_making::{ConsensusStrategy, RiskThresholds};
use crate::judge_backup::backup_types::JudgeType;
use crate::judge_backup::JudgeConfig;
use crate::judge_backup::{
    quality_judge::QualityAssuranceJudge, security_judge::SecurityJudge, EthicsJudge, Judge,
};
use crate::orchestration::task_state_persistence::{
    DatabaseTaskStatePersistence, TaskStatePersistence,
};
use crate::orchestration::unified_orchestrator::{UnifiedOrchestrator, UnifiedOrchestratorConfig};
use crate::planning::{
    caws_adjudication_cycle::CawsAdjudicationCycle,
    caws_debate_scorer::CawsDebateScorer,
    council_integration::{CouncilIntegration, CouncilIntegrationImpl},
    plan_executor::{
        WorkerHealth, WorkerInfo, WorkerPool, WorkerStatus,
    },
    reflexive_learner::{LearningConfig, ReflexiveLearner},
    worker_assignment::WorkerAssignmentStrategy,
    worker_evolution::{EvolutionConfig, WorkerEvolutionEngine},
    worker_lifecycle_manager::WorkerLifecycleManager,
    worktree_manager::{WorktreeManager, WorktreeManagerConfig},
};
use crate::planning::{DatabaseOperations, database_operations_bridge::DatabaseOperationsBridge};
use agent_agency_contracts::ports::DatabaseOperationsPort;
use crate::verdict_aggregation::{
    AggregationConfig, DissentHandling, RiskAggregationStrategy, VerdictAggregator,
};
use agent_workers::{MCPWorkerPool, TaskExecutor, WorkerPoolConfig};
use crate::workers::execution_bridge::WorkerExecutionBridge;
use crate::planning::plan_executor::{ExecutionConfig, PlanExecutor};
use crate::planning::plan_types::ExecutionPlan;
use crate::planning::factory::PlanningSystemFactory;
use async_trait::async_trait;
#[cfg(feature = "model-management")]
use agent_model_management::deployment::DeploymentOrchestrator;

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
    #[allow(unused_variables)] // Variables are used when memory feature is enabled
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

        // Create database client first (needed for adapter and other components)
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/agent_agency_v3".to_string());
        let db_config = data_infrastructure::DatabaseConfig {
            database_url: database_url.clone(),
            pool_max: Some(10),
            connection_timeout: Some(30),
            query_timeout: Some(60),
            ..Default::default()
        };
        // Create ApiDatabaseClient (complex client) for adapter - implements DatabaseOperations trait
        let api_db_client = Arc::new(
            data_infrastructure::ApiDatabaseClient::new(db_config.clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create database client: {}", e))?,
        );
        // Create simple DatabaseClient wrapper for other components that expect it
        let db_client = Arc::new(
            data_infrastructure::DatabaseClient::new(db_config)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create database client: {}", e))?,
        );

        // Create database operations adapter if not provided
        // Use real DatabaseOperationsAdapter implementation (inline to avoid circular dependency)
        let db_ops = if let Some(db_ops) = db_ops {
            db_ops
        } else {
            // Verify database schema before creating adapter
            // This helps catch schema issues early with better error messages
            info!("Verifying database schema before creating DatabaseOperationsAdapter...");
            let test_pool = api_db_client.pool();

            // Test query to planning_audit_events to verify description column exists
            let test_plan_id = uuid::Uuid::new_v4();
            match sqlx::query_scalar::<_, i64>(
                r#"
                SELECT COUNT(*)
                FROM planning_audit_events
                WHERE plan_id = $1
                "#,
            )
            .bind(test_plan_id)
            .fetch_one(test_pool)
            .await
            {
                Ok(_) => {
                    info!("Database schema verification passed for planning_audit_events");
                }
                Err(e) => {
                    error!("CRITICAL: Database schema verification failed during DatabaseOperationsAdapter creation");
                    error!("   Error: {}", e);
                    error!("   This may indicate the 'description' column is missing from 'planning_audit_events' table");
                    error!("   Please run migration 028 to fix the schema");
                    return Err(anyhow!("Database schema verification failed: {}. This may indicate the 'description' column is missing from 'planning_audit_events' table. Please run migration 028 to fix the schema.", e));
                }
            }

            // Create adapter that wraps ApiDatabaseClient to implement agent-orchestration DatabaseOperations
            Arc::new(DatabaseOperationsAdapter::new(api_db_client.clone()))
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
            use agent_memory::{MemoryConfig, MemorySystem};
            info!("Initializing MemorySystem...");
            match MemorySystem::init(MemoryConfig::default()).await {
                Ok(system) => {
                    info!("MemorySystem initialized successfully");
                    Arc::new(system)
                }
                Err(e) => {
                    error!("Failed to initialize MemorySystem: {}", e);
                    error!("   This may indicate a database schema issue");
                    return Err(anyhow!("Failed to initialize MemorySystem: {}. This may indicate a database schema issue.", e));
                }
            }
        };

        // Create UnifiedOrchestratorConfig (needed for worktree_manager)
        // Council review re-enabled after improving verdict aggregation to handle
        // weak consensus without dissent (common for plan reviews) more gracefully.
        // See EVALUATION_FIX_PLAN.md for details on the improvements.
        let config = UnifiedOrchestratorConfig {
            enable_council_review: true, // Re-enabled after fixing workspace creation hang
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

        // Scaffold standard workers if they don't exist
        // This ensures the orchestrator has workers available for task execution
        info!("Scaffolding standard workers...");
        if let Err(e) =
            crate::orchestration::worker_scaffolding::scaffold_standard_workers(db_client.clone())
                .await
        {
            error!("Failed to scaffold standard workers: {}", e);
            error!("   This may indicate a database schema issue");
            warn!("Continuing without worker scaffolding - workers may need to be registered manually");
        } else {
            info!("Standard workers scaffolded successfully");
        }

        // Clone db_client for TaskExecutor (it will be moved)
        let db_client_for_executor = db_client.clone();

        // Create ToolRegistry with real FileOperationsService for MCP tools
        // Use helper function from agent-workers that has access to both agent_mcp and data-infrastructure
        let repo_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let tool_registry = agent_workers::create_tool_registry_with_file_ops(repo_path.clone())
            .await
            .map_err(|e| {
                anyhow::anyhow!("Failed to create tool registry with file operations: {}", e)
            })?;

        // Verify tool registry initialization - ensure tools were actually registered
        let registered_tools = tool_registry.get_all_tools().await;
        let tool_names: Vec<&str> = registered_tools.iter().map(|t| t.name.as_str()).collect();
        info!(
            "Tool registry initialized with {} tools: {:?}",
            registered_tools.len(),
            tool_names
        );

        // Fail fast if critical tools are missing
        let required_tools = ["file_edit", "file_read"];
        for required_tool in &required_tools {
            if !tool_names.contains(required_tool) {
                return Err(anyhow::anyhow!(
                    "Critical tool '{}' not found in tool registry. Available tools: {:?}. \
                     Tool registry initialization may have failed.",
                    required_tool,
                    tool_names
                ));
            }
        }
        info!("Tool registry verification passed - all required tools available");

        // Use the existing memory_system for workers (already created above)
        #[cfg(not(feature = "memory"))]
        {
            return Err(anyhow::anyhow!(
                "Memory feature required for UnifiedOrchestrator initialization"
            ));
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
            languages: vec![
                "python".to_string(),
                "rust".to_string(),
                "typescript".to_string(),
            ],
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
        {
            use agent_workers::WorkerSpecialty;
            worker_pool
                .register_worker(WorkerSpecialty::General, default_capabilities)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to register default worker: {}", e))?;
        }

        let task_executor = Arc::new(TaskExecutor::new(db_client_for_executor));

        // Clone worker_pool before moving it into WorkerExecutionBridge (needed later for adapter)
        #[cfg(feature = "memory")]
        let worker_pool_for_bridge = worker_pool.clone();
        #[cfg(feature = "memory")]
        let worker_bridge = Arc::new(WorkerExecutionBridge::new(
            worker_pool_for_bridge,
            task_executor,
        ));

        // Create planning components - requires both research and memory features
        // Pass worker_bridge and worktree_manager so PlanExecutor has real execution capabilities
        #[cfg(all(feature = "research", feature = "memory"))]
        let planning_components = {
            // Verify database schema before creating planning components
            // This helps catch schema issues early with better error messages
            info!("Verifying database schema before creating planning components...");
            let test_pool = api_db_client.pool();
            let has_description: bool = sqlx::query_scalar(
                r#"
                SELECT EXISTS (
                    SELECT 1
                    FROM information_schema.columns
                    WHERE table_name = 'planning_audit_events'
                    AND column_name = 'description'
                    AND table_schema = 'public'
                )
                "#,
            )
            .fetch_one(test_pool)
            .await
            .unwrap_or(false);

            if !has_description {
                error!("CRITICAL: planning_audit_events table is missing 'description' column");
                error!("   This will cause PlanningSystemFactory initialization to fail");
                error!("   Please run migration 028 to fix the schema");
                return Err(anyhow!("planning_audit_events table is missing 'description' column. Please run migration 028 to fix the schema."));
            }

            info!("Database schema verification passed for planning_audit_events");
            info!("Creating planning system components...");
            match PlanningSystemFactory::create_planning_components(
                research_collector,
                memory_system.clone(),
                council.clone(),
                db_ops.clone(),
                Some(worker_bridge.clone()), // Pass WorkerExecutionBridge
                Some(worktree_manager.clone()), // Pass WorktreeManager
            )
            .await
            {
                Ok(components) => {
                    info!("Planning system components created successfully");
                    components
                }
                Err(e) => {
                    error!("Failed to create planning system components: {}", e);
                    error!("This may indicate a database schema issue (e.g., missing 'description' column)");
                    return Err(anyhow!("Failed to create planning system components: {}. Check database schema and migrations.", e));
                }
            }
        };

        #[cfg(not(all(feature = "research", feature = "memory")))]
        {
            return Err(anyhow::anyhow!(
                "Both research and memory features required for UnifiedOrchestrator initialization. \
                 Enable both features in Cargo.toml or use LegacyOrchestratorAdapter."
            ));
        }

        // Create CAWS adjudication cycle
        let council_integration: Arc<dyn CouncilIntegration> = Arc::new(
            CouncilIntegrationImpl::new(council.clone(), council_config.clone()),
        );
        let debate_scorer = Arc::new(CawsDebateScorer::new(council.clone()));
        let adjudication_cycle = Arc::new(CawsAdjudicationCycle::new(
            council.clone(),
            council_integration.clone(),
            debate_scorer,
        ));

        // Create worker lifecycle manager
        let worker_lifecycle_manager =
            Arc::new(WorkerLifecycleManager::new(council_integration.clone()));

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

        // Create worker evolution engine
        let evolution_config = EvolutionConfig::default();
        let evolution_engine =
            Arc::new(WorkerEvolutionEngine::new(db_ops.clone(), evolution_config));

        // Create curriculum learning engine
        // Note: CurriculumLearningEngine requires data_infrastructure::DatabaseOperations
        // which is a different trait than our local planning::DatabaseOperations.
        // For now, we create the reflexive learner without curriculum integration.
        // The curriculum engine can be added when data_infrastructure is properly integrated.
        let curriculum_engine: Option<Arc<crate::planning::curriculum_learning::CurriculumLearningEngine>> = None;

        // Create reflexive learner with evolution engine only (curriculum integration pending)
        let reflexive_learner = Arc::new(ReflexiveLearner::with_evolution_engine(
            worker_assignment_strategy.clone(),
            evolution_engine,
            LearningConfig::default(),
        ));

        // Clone for continuous learning loop (needed outside the cfg block)
        #[cfg(all(feature = "research", feature = "memory"))]
        let reflexive_learner_for_loop = Arc::clone(&reflexive_learner);

        // Create worker pool adapter for PlanExecutor using the real MCPWorkerPool
        #[cfg(feature = "memory")]
        let executor_worker_pool: Arc<dyn WorkerPool> = {
            use std::collections::HashMap;
            use tokio::sync::RwLock;

            // Create adapter that wraps the real worker_pool and tracks assignments
            struct MCPWorkerPoolAdapter {
                pool: Arc<MCPWorkerPool>,
                assignments: Arc<RwLock<HashMap<Uuid, String>>>, // worker_id -> milestone_id
            }

            #[async_trait]
            impl WorkerPool for MCPWorkerPoolAdapter {
                async fn available_workers(&self) -> Result<Vec<WorkerInfo>> {
                    let workers = self.pool.list_workers().await;
                    let assignments = self.assignments.read().await;

                    Ok(workers
                        .into_iter()
                        .map(|handle| {
                            let is_assigned = assignments.contains_key(&handle.id.0);
                            WorkerInfo {
                                id: handle.id.0,
                                capabilities: handle
                                    .capabilities
                                    .languages
                                    .iter()
                                    .chain(handle.capabilities.frameworks.iter())
                                    .chain(handle.capabilities.domains.iter())
                                    .map(|s| s.clone())
                                    .collect(),
                                load: if is_assigned { 1.0 } else { 0.0 },
                                health: WorkerHealth::Healthy,
                            }
                        })
                        .collect())
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

                async fn worker_status(&self, worker_id: Uuid) -> Result<WorkerStatus> {
                    let workers = self.pool.list_workers().await;
                    let assignments = self.assignments.read().await;

                    let worker = workers.iter().find(|w| w.id.0 == worker_id);
                    let current_assignment = assignments.get(&worker_id).cloned();

                    let stats = self.pool.get_stats().await;

                    Ok(WorkerStatus {
                        current_assignment,
                        health: if worker.is_some() {
                            WorkerHealth::Healthy
                        } else {
                            WorkerHealth::Unavailable
                        },
                        performance: crate::planning::plan_executor::WorkerPerformance {
                            tasks_completed: stats.total_tasks_completed as usize,
                            tasks_failed: stats.total_tasks_failed as usize,
                            avg_completion_time_ms: stats.average_execution_time_ms,
                            success_rate: if stats.total_tasks_completed + stats.total_tasks_failed
                                > 0
                            {
                                stats.total_tasks_completed as f64
                                    / (stats.total_tasks_completed + stats.total_tasks_failed)
                                        as f64
                            } else {
                                1.0
                            },
                        },
                    })
                }
            }

            Arc::new(MCPWorkerPoolAdapter {
                pool: worker_pool.clone(),
                assignments: Arc::new(RwLock::new(HashMap::new())),
            })
        };

        #[cfg(not(feature = "memory"))]
        let _executor_worker_pool: Arc<dyn WorkerPool> = {
            // Fallback when memory feature is not enabled
            struct FallbackWorkerPool;
            #[async_trait]
            impl WorkerPool for FallbackWorkerPool {
                async fn available_workers(&self) -> Result<Vec<WorkerInfo>> {
                    warn!("Memory feature not enabled - worker pool unavailable");
                    Ok(vec![])
                }
                async fn assign_worker(
                    &self,
                    _worker_id: Uuid,
                    _milestone_id: String,
                ) -> Result<()> {
                    warn!("Memory feature not enabled - worker assignment unavailable");
                    Ok(())
                }
                async fn release_worker(&self, _worker_id: Uuid) -> Result<()> {
                    warn!("Memory feature not enabled - worker release unavailable");
                    Ok(())
                }
                async fn worker_status(&self, _worker_id: Uuid) -> Result<WorkerStatus> {
                    warn!("Memory feature not enabled - worker status unavailable");
                    Ok(WorkerStatus {
                        current_assignment: None,
                        health: WorkerHealth::Unavailable,
                        performance: crate::planning::plan_executor::WorkerPerformance {
                            tasks_completed: 0,
                            tasks_failed: 0,
                            avg_completion_time_ms: 0.0,
                            success_rate: 0.0,
                        },
                    })
                }
            }
            Arc::new(FallbackWorkerPool)
        };

        // Create audit trail adapter
        use crate::audit_trail::{AuditConfig, AuditTrailManager};
        let audit_trail_manager = Arc::new(AuditTrailManager::new(AuditConfig::default()));
        struct AuditTrailAdapter {
            #[allow(dead_code)] // Reserved for future use
            manager: Arc<AuditTrailManager>,
        }
        #[async_trait]
        impl crate::planning::plan_executor::AuditTrail for AuditTrailAdapter {
            async fn log_event(
                &self,
                event: crate::planning::plan_executor::AuditEvent,
            ) -> Result<()> {
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
            None,                           // worker_lifecycle_manager - will be set separately
            Some(worker_bridge.clone()),    // Pass WorkerExecutionBridge for real execution
            Some(worktree_manager.clone()), // Pass WorktreeManager for worktree path resolution
            ExecutionConfig::default(),
        ));

        // Create state persistence for pause/resume/cancel support
        // Use database persistence when db_client is available, otherwise use in-memory
        // Database persistence provides crash recovery and task resumption capabilities
        let state_persistence: Arc<dyn TaskStatePersistence> =
            Arc::new(DatabaseTaskStatePersistence::new(db_client.clone()));

        // Create UnifiedOrchestrator
        #[cfg(all(feature = "research", feature = "memory"))]
        let orchestrator = {
            // Create ArbiterPipelineOptimizer if runtime-optimization feature is enabled
            #[cfg(feature = "runtime-optimization")]
            let arbiter_optimizer = {
                use system_federated_ml::{ArbiterPipelineOptimizer, DecisionPipelineConfig, StreamingPipelineConfig};
                use system_configuration::StreamingPipelineConfig as BaseStreamingConfig;

                // Configure for streaming execution with judge deliberations
                let mut config = DecisionPipelineConfig::default();
                config.enable_streaming = true;
                config.streaming = Some(StreamingPipelineConfig {
                    base: BaseStreamingConfig::default(),
                    buffer_size: 100, // Support concurrent decision streams
                    max_concurrent_streams: 50, // Allow parallel judge deliberations
                    enable_backpressure: true,
                    backpressure_threshold: 25,
                });

                match ArbiterPipelineOptimizer::new(config).await {
                    Ok(mut optimizer) => {
                        // Create continuous optimization service
                        use system_federated_ml::continuous_optimization::{ContinuousOptimizationService, ContinuousOptimizationConfig};

                        let opt_config = ContinuousOptimizationConfig::default();
                        let continuous_optimizer = Arc::new(ContinuousOptimizationService::new(
                            opt_config,
                            Arc::new(tokio::sync::RwLock::new(system_federated_ml::BayesianOptimizer::new(Default::default()).unwrap()))
                        ));

                        // Start the continuous optimizer
                        if let Err(e) = continuous_optimizer.start().await {
                            tracing::warn!("Failed to start continuous optimization service: {}", e);
                        } else {
                            // Set continuous optimizer on arbiter
                            optimizer.set_continuous_optimizer(Arc::clone(&continuous_optimizer));
                            info!("Continuous optimization service integrated with ArbiterPipelineOptimizer");
                        }

                        info!("ArbiterPipelineOptimizer created with streaming support and continuous optimization for judge deliberations");
                        Some(Arc::new(optimizer))
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create ArbiterPipelineOptimizer: {}", e);
                        None
                    }
                }
            };

            #[cfg(feature = "model-management")]
            let deployment_orchestrator: Option<Arc<DeploymentOrchestrator>> = match DeploymentOrchestrator::new().await {
                Ok(orchestrator) => {
                    info!("DeploymentOrchestrator created successfully");
                    Some(Arc::new(orchestrator))
                }
                Err(e) => {
                    warn!("Failed to create DeploymentOrchestrator: {}", e);
                    None
                }
            };

            // Create curriculum learning engine for skill progression
            let curriculum_engine = {
                use crate::planning::curriculum_learning::CurriculumLearningEngine;
                use data_infrastructure::DatabaseOperations;

                // Create database operations instance
                let db_config = data_infrastructure::database_config::DatabaseConfig::default();
                match data_infrastructure::create_database_operations(db_config).await {
                    Ok(db_ops) => {
                        info!("Curriculum learning engine initialized with database persistence");
                        Some(Arc::new(CurriculumLearningEngine::new(db_ops)))
                    }
                    Err(e) => {
                        warn!("Failed to create database operations for curriculum engine: {}", e);
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
                curriculum_engine, // Already Option<Arc<...>>
                #[cfg(feature = "memory")]
                Some(memory_system),
                #[cfg(not(feature = "memory"))]
                None,
                None,                    // turn_level_tracker - optional
                None,                    // session_manager - optional
                Some(state_persistence), // Enable state persistence for pause/resume/cancel
                None,                    // federated_learning - optional
                #[cfg(feature = "runtime-optimization")]
                arbiter_optimizer,
                #[cfg(feature = "model-management")]
                deployment_orchestrator,
            ))
        };

        #[cfg(all(feature = "research", feature = "memory"))]
        {
            info!("UnifiedOrchestrator created successfully");

            // Start continuous learning loop for ReflexiveLearner
            // Start continuous learning loop with 60 second interval
            // This will periodically analyze accumulated outcomes and update routing policies
            if let Err(e) = reflexive_learner_for_loop
                .start_continuous_learning(60)
                .await
            {
                warn!(
                    "Failed to start ReflexiveLearner continuous learning loop: {}",
                    e
                );
            } else {
                info!("ReflexiveLearner continuous learning loop started");
            }

            Ok(orchestrator)
        }
    }
}

/// Database operations adapter - bridges data-infrastructure DatabaseClient to agent-orchestration DatabaseOperations
///
/// This adapter is implemented inline to avoid circular dependency with data-interfaces-adapters.
/// It provides full database operations by wrapping DatabaseClient and mapping between type systems.
mod database_operations_adapter {
    use anyhow::{anyhow, Result};
    use async_trait::async_trait;
    use chrono::Utc;
    use sqlx::{self, Row};
    use std::sync::Arc;
    use tracing::{info, warn};
    use uuid::Uuid;

    use crate::planning::data_infrastructure_types::{
        models, CreateAuditTrailEntry, CreateCouncilSession, CreateExecutionPlan,
        CreateExecutionResult, CreateJudge, CreateJudgeEvaluation, CreatePlanningAuditEvent,
        CreatePlanningSession, CreatePlanningTelemetry, CreateWaiver, CreateWorker,
        DatabaseOperations, UpdateCouncilSession, UpdateExecutionPlan, UpdatePlanningSession,
        UpdateWaiver, UpdateWorker,
    };
    use data_infrastructure::{
        ApiDatabaseClient, DatabaseOperations as DataInfraDatabaseOperations,
    };

    /// Adapter that bridges data-infrastructure DatabaseClient to agent-orchestration DatabaseOperations
    pub struct DatabaseOperationsAdapter {
        db_client: Arc<ApiDatabaseClient>,
    }

    impl DatabaseOperationsAdapter {
        /// Create a new database operations adapter
        pub fn new(db_client: Arc<ApiDatabaseClient>) -> Self {
            Self { db_client }
        }
    }

    #[async_trait]
    impl DatabaseOperations for DatabaseOperationsAdapter {
        async fn get_workers(&self) -> Result<Vec<models::Worker>> {
            use data_infrastructure::models::Worker as DbWorker;

            let pool = self.db_client.pool();
            let rows = sqlx::query_as::<_, DbWorker>(
                r#"
                SELECT id, name, worker_type, specialty, model_name, endpoint,
                       capabilities, performance_history, is_active, created_at, updated_at
                FROM workers
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(pool)
            .await
            .map_err(|e| anyhow!("Failed to query workers from database: {}", e))?;

            let workers: Vec<models::Worker> = rows
                .into_iter()
                .map(|db_worker| {
                    let capabilities =
                        if let serde_json::Value::Object(caps_obj) = &db_worker.capabilities {
                            serde_json::json!(caps_obj)
                        } else {
                            db_worker.capabilities.clone()
                        };

                    let performance_history = if let serde_json::Value::Object(perf_obj) =
                        &db_worker.performance_history
                    {
                        serde_json::json!(perf_obj)
                    } else {
                        db_worker.performance_history.clone()
                    };

                    models::Worker {
                        id: db_worker.id,
                        name: db_worker.name,
                        worker_type: db_worker.worker_type,
                        specialty: db_worker.specialty,
                        model_name: db_worker.model_name,
                        endpoint: db_worker.endpoint,
                        capabilities,
                        performance_history,
                        is_active: db_worker.is_active,
                        metadata: std::collections::HashMap::new(),
                        created_at: db_worker.created_at,
                        updated_at: db_worker.updated_at,
                    }
                })
                .collect();

            info!("Queried {} workers from database", workers.len());
            Ok(workers)
        }

        async fn get_worker(&self, id: Uuid) -> Result<Option<models::Worker>> {
            use data_infrastructure::models::Worker as DbWorker;

            let pool = self.db_client.pool();
            let row = sqlx::query_as::<_, DbWorker>(
                r#"
                SELECT id, name, worker_type, specialty, model_name, endpoint,
                       capabilities, performance_history, is_active, created_at, updated_at
                FROM workers
                WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to query worker from database: {}", e))?;

            Ok(row.map(|db_worker| {
                let capabilities =
                    if let serde_json::Value::Object(caps_obj) = &db_worker.capabilities {
                        serde_json::json!(caps_obj)
                    } else {
                        db_worker.capabilities.clone()
                    };

                let performance_history =
                    if let serde_json::Value::Object(perf_obj) = &db_worker.performance_history {
                        serde_json::json!(perf_obj)
                    } else {
                        db_worker.performance_history.clone()
                    };

                models::Worker {
                    id: db_worker.id,
                    name: db_worker.name,
                    worker_type: db_worker.worker_type,
                    specialty: db_worker.specialty,
                    model_name: db_worker.model_name,
                    endpoint: db_worker.endpoint,
                    capabilities,
                    performance_history,
                    is_active: db_worker.is_active,
                    metadata: std::collections::HashMap::new(),
                    created_at: db_worker.created_at,
                    updated_at: db_worker.updated_at,
                }
            }))
        }

        async fn create_worker(&self, worker: CreateWorker) -> Result<models::Worker> {
            use data_infrastructure::database_operations::CreateWorker as DbCreateWorker;

            let db_worker = DbCreateWorker {
                name: worker.name,
                worker_type: worker.worker_type,
                specialty: worker.specialty,
                model_name: worker.model_name,
                endpoint: worker.endpoint,
                capabilities: worker.capabilities,
                performance_history: worker.performance_history,
                is_active: worker.is_active,
            };

            let created = self
                .db_client
                .create_worker(db_worker)
                .await
                .map_err(|e| anyhow!("Failed to create worker: {}", e))?;

            Ok(models::Worker {
                id: created.id,
                name: created.name,
                worker_type: created.worker_type,
                specialty: created.specialty,
                model_name: created.model_name,
                endpoint: created.endpoint,
                capabilities: created.capabilities,
                performance_history: created.performance_history,
                is_active: created.is_active,
                metadata: std::collections::HashMap::new(),
                created_at: created.created_at,
                updated_at: created.updated_at,
            })
        }

        async fn update_worker(&self, id: Uuid, update: UpdateWorker) -> Result<models::Worker> {
            use data_infrastructure::database_operations::UpdateWorker as DbUpdateWorker;

            let db_update = DbUpdateWorker {
                name: update.name,
                worker_type: update.worker_type,
                specialty: update.specialty,
                model_name: update.model_name,
                endpoint: update.endpoint,
                capabilities: update.capabilities,
                performance_history: update.performance_history,
                is_active: update.is_active,
            };

            let updated = self
                .db_client
                .update_worker(id, db_update)
                .await
                .map_err(|e| anyhow!("Failed to update worker: {}", e))?;

            Ok(models::Worker {
                id: updated.id,
                name: updated.name,
                worker_type: updated.worker_type,
                specialty: updated.specialty,
                model_name: updated.model_name,
                endpoint: updated.endpoint,
                capabilities: updated.capabilities,
                performance_history: updated.performance_history,
                is_active: updated.is_active,
                metadata: std::collections::HashMap::new(),
                created_at: updated.created_at,
                updated_at: updated.updated_at,
            })
        }

        async fn create_execution_plan(
            &self,
            plan: CreateExecutionPlan,
        ) -> Result<models::ExecutionPlan> {
            let pool = self.db_client.pool();
            let now = Utc::now();

            let working_spec_id = plan
                .working_spec_id
                .unwrap_or_else(|| format!("PLAN-{}", plan.id));

            sqlx::query(
                r#"
                INSERT INTO execution_plans (
                    id, session_id, working_spec_id, title, overview, state,
                    milestones, dependency_graph, change_budget, quality_gates,
                    evidence_requirements, active_waivers, metadata, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                "#,
            )
            .bind(plan.id)
            .bind(Uuid::new_v4())
            .bind(&working_spec_id)
            .bind(&plan.title)
            .bind(&plan.overview)
            .bind("draft")
            .bind(serde_json::json!([]))
            .bind(serde_json::json!({}))
            .bind(serde_json::json!({}))
            .bind(serde_json::json!({}))
            .bind(serde_json::json!([]))
            .bind(serde_json::json!([]))
            .bind(serde_json::json!({}))
            .bind(now)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to persist execution plan: {}", e))?;

            info!("Persisted execution plan {} to database", plan.id);

            let session_id = Uuid::new_v4();

            Ok(models::ExecutionPlan {
                id: plan.id,
                session_id,
                workspace_id: None,
                working_spec_id,
                title: plan.title,
                overview: Some(plan.overview),
                state: "draft".to_string(),
                milestones: serde_json::json!([]),
                dependency_graph: serde_json::json!({}),
                change_budget: serde_json::json!({}),
                quality_gates: serde_json::json!({}),
                evidence_requirements: serde_json::json!([]),
                active_waivers: serde_json::json!([]),
                metadata: serde_json::json!({}),
                created_at: now,
                updated_at: now,
                approved_at: None,
                completed_at: None,
            })
        }

        async fn get_execution_plan(&self, id: Uuid) -> Result<Option<models::ExecutionPlan>> {
            let pool = self.db_client.pool();

            let row = sqlx::query(
                r#"
                SELECT id, session_id, working_spec_id, title, overview, state,
                       milestones, dependency_graph, change_budget, quality_gates,
                       evidence_requirements, active_waivers, metadata,
                       created_at, updated_at, approved_at, completed_at
                FROM execution_plans
                WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to query execution plan: {}", e))?;

            Ok(row.map(|r: sqlx::postgres::PgRow| {
                let id: Uuid = r.get("id");
                let session_id: Uuid = r.get("session_id");
                let working_spec_id: String = r.get("working_spec_id");
                let state: String = r.get("state");

                models::ExecutionPlan {
                    id,
                    session_id,
                    workspace_id: r.try_get::<Option<String>, _>("workspace_id").ok().flatten(),
                    working_spec_id,
                    title: r.get("title"),
                    overview: r.try_get::<Option<String>, _>("overview").ok().flatten(),
                    state,
                    milestones: r
                        .try_get::<serde_json::Value, _>("milestones")
                        .unwrap_or_else(|_| serde_json::json!([])),
                    dependency_graph: r
                        .try_get::<serde_json::Value, _>("dependency_graph")
                        .unwrap_or_else(|_| serde_json::json!({})),
                    change_budget: r
                        .try_get::<serde_json::Value, _>("change_budget")
                        .unwrap_or_else(|_| serde_json::json!({})),
                    quality_gates: r
                        .try_get::<serde_json::Value, _>("quality_gates")
                        .unwrap_or_else(|_| serde_json::json!({})),
                    evidence_requirements: r
                        .try_get::<serde_json::Value, _>("evidence_requirements")
                        .unwrap_or_else(|_| serde_json::json!([])),
                    active_waivers: r
                        .try_get::<serde_json::Value, _>("active_waivers")
                        .unwrap_or_else(|_| serde_json::json!([])),
                    metadata: r
                        .try_get::<serde_json::Value, _>("metadata")
                        .unwrap_or_else(|_| serde_json::json!({})),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                    approved_at: r
                        .try_get::<Option<chrono::DateTime<Utc>>, _>("approved_at")
                        .ok()
                        .flatten(),
                    completed_at: r
                        .try_get::<Option<chrono::DateTime<Utc>>, _>("completed_at")
                        .ok()
                        .flatten(),
                }
            }))
        }

        async fn get_execution_plans(&self) -> Result<Vec<models::ExecutionPlan>> {
            let pool = self.db_client.pool();

            let rows = sqlx::query(
                r#"
                SELECT id, session_id, working_spec_id, title, overview, state,
                       milestones, dependency_graph, change_budget, quality_gates,
                       evidence_requirements, active_waivers, metadata,
                       created_at, updated_at, approved_at, completed_at
                FROM execution_plans
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(pool)
            .await
            .map_err(|e| anyhow!("Failed to query execution plans: {}", e))?;

            Ok(rows
                .into_iter()
                .map(|r: sqlx::postgres::PgRow| {
                    let id: Uuid = r.get("id");
                    let session_id: Uuid = r.get("session_id");
                    let working_spec_id: String = r.get("working_spec_id");
                    let state: String = r.get("state");

                    models::ExecutionPlan {
                        id,
                        session_id,
                        workspace_id: r.try_get::<Option<String>, _>("workspace_id").ok().flatten(),
                        working_spec_id,
                        title: r.get("title"),
                        overview: r.try_get::<Option<String>, _>("overview").ok().flatten(),
                        state,
                        milestones: r
                            .try_get::<serde_json::Value, _>("milestones")
                            .unwrap_or_else(|_| serde_json::json!([])),
                        dependency_graph: r
                            .try_get::<serde_json::Value, _>("dependency_graph")
                            .unwrap_or_else(|_| serde_json::json!({})),
                        change_budget: r
                            .try_get::<serde_json::Value, _>("change_budget")
                            .unwrap_or_else(|_| serde_json::json!({})),
                        quality_gates: r
                            .try_get::<serde_json::Value, _>("quality_gates")
                            .unwrap_or_else(|_| serde_json::json!({})),
                        evidence_requirements: r
                            .try_get::<serde_json::Value, _>("evidence_requirements")
                            .unwrap_or_else(|_| serde_json::json!([])),
                        active_waivers: r
                            .try_get::<serde_json::Value, _>("active_waivers")
                            .unwrap_or_else(|_| serde_json::json!([])),
                        metadata: r
                            .try_get::<serde_json::Value, _>("metadata")
                            .unwrap_or_else(|_| serde_json::json!({})),
                        created_at: r.get("created_at"),
                        updated_at: r.get("updated_at"),
                        approved_at: r
                            .try_get::<Option<chrono::DateTime<Utc>>, _>("approved_at")
                            .ok()
                            .flatten(),
                        completed_at: r
                            .try_get::<Option<chrono::DateTime<Utc>>, _>("completed_at")
                            .ok()
                            .flatten(),
                    }
                })
                .collect())
        }

        async fn update_execution_plan(
            &self,
            id: Uuid,
            update: UpdateExecutionPlan,
        ) -> Result<models::ExecutionPlan> {
            let pool = self.db_client.pool();

            let mut updates = Vec::new();
            let mut bind_index = 1;

            if let Some(ref _title) = update.title {
                updates.push(format!("title = ${}", bind_index));
                bind_index += 1;
            }
            if let Some(ref _overview) = update.overview {
                updates.push(format!("overview = ${}", bind_index));
                bind_index += 1;
            }
            if let Some(ref _status) = update.status {
                updates.push(format!("state = ${}", bind_index));
                bind_index += 1;
            }

            if updates.is_empty() {
                return self
                    .get_execution_plan(id)
                    .await?
                    .ok_or_else(|| anyhow!("Execution plan {} not found", id));
            }

            updates.push(format!("updated_at = ${}", bind_index));
            bind_index += 1;

            let query = format!(
                "UPDATE execution_plans SET {} WHERE id = ${}",
                updates.join(", "),
                bind_index
            );

            let mut query_builder = sqlx::query(&query);
            if let Some(ref title) = update.title {
                query_builder = query_builder.bind(title);
            }
            if let Some(ref overview) = update.overview {
                query_builder = query_builder.bind(overview);
            }
            if let Some(ref status) = update.status {
                query_builder = query_builder.bind(status);
            }
            query_builder = query_builder.bind(Utc::now());
            query_builder = query_builder.bind(id);

            query_builder
                .execute(pool)
                .await
                .map_err(|e| anyhow!("Failed to update execution plan: {}", e))?;

            self.get_execution_plan(id)
                .await?
                .ok_or_else(|| anyhow!("Execution plan {} not found after update", id))
        }

        async fn create_audit_trail_entry(
            &self,
            entry: CreateAuditTrailEntry,
        ) -> Result<models::AuditTrailEntry> {
            let task_id = entry
                .metadata
                .get("task_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(|| Uuid::new_v4());

            let pool = self.db_client.pool();
            let id = Uuid::new_v4();
            let timestamp = Utc::now();

            sqlx::query(
                r#"
                INSERT INTO audit_trail_entries (
                    id, entity_type, entity_id, action, details,
                    user_id, ip_address, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(id)
            .bind("task")
            .bind(task_id)
            .bind(&entry.event_type)
            .bind(&serde_json::json!({
                "description": entry.description,
                "metadata": entry.metadata,
            }))
            .bind(
                entry
                    .metadata
                    .get("user_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            )
            .bind(
                entry
                    .metadata
                    .get("ip_address")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            )
            .bind(timestamp)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to persist audit trail entry: {}", e))?;

            let db_result = sqlx::query_as::<_, data_infrastructure::models::AuditTrailEntry>(
                r#"
                SELECT id, entity_type, entity_id, action, details, user_id, ip_address, created_at
                FROM audit_trail_entries
                WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow!("Failed to retrieve persisted audit trail entry: {}", e))?;

            Ok(models::AuditTrailEntry {
                id: db_result.id,
                event_type: db_result.action,
                description: db_result
                    .details
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "".to_string()),
                timestamp: db_result.created_at,
                metadata: db_result
                    .details
                    .get("metadata")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
            })
        }

        async fn get_audit_trail_entries(
            &self,
            task_id: Uuid,
        ) -> Result<Vec<models::AuditTrailEntry>> {
            let pool = self.db_client.pool();
            let db_results = sqlx::query_as::<_, data_infrastructure::models::AuditTrailEntry>(
                r#"
                SELECT id, entity_type, entity_id, action, details, user_id, ip_address, created_at
                FROM audit_trail_entries
                WHERE entity_id = $1 AND entity_type = 'task'
                ORDER BY created_at DESC
                "#,
            )
            .bind(task_id)
            .fetch_all(pool)
            .await
            .map_err(|e| anyhow!("Failed to query audit trail entries: {}", e))?;

            Ok(db_results
                .into_iter()
                .map(|db_entry| models::AuditTrailEntry {
                    id: db_entry.id,
                    event_type: db_entry.action,
                    description: db_entry
                        .details
                        .get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "".to_string()),
                    timestamp: db_entry.created_at,
                    metadata: db_entry
                        .details
                        .get("metadata")
                        .and_then(|v| v.as_object())
                        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                        .unwrap_or_default(),
                })
                .collect())
        }

        async fn get_audit_trail_entry(&self, id: Uuid) -> Result<Option<models::AuditTrailEntry>> {
            let pool = self.db_client.pool();
            let db_result = sqlx::query_as::<_, data_infrastructure::models::AuditTrailEntry>(
                r#"
                SELECT id, entity_type, entity_id, action, details, user_id, ip_address, created_at
                FROM audit_trail_entries
                WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to query audit trail entry: {}", e))?;

            Ok(db_result.map(|db_entry| models::AuditTrailEntry {
                id: db_entry.id,
                event_type: db_entry.action,
                description: db_entry
                    .details
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "".to_string()),
                timestamp: db_entry.created_at,
                metadata: db_entry
                    .details
                    .get("metadata")
                    .and_then(|v| v.as_object())
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
            }))
        }

        async fn create_planning_session(
            &self,
            session: CreatePlanningSession,
        ) -> Result<models::PlanningSession> {
            let pool = self.db_client.pool();
            let session_id = Uuid::new_v4();
            let now = Utc::now();

            sqlx::query(
                r#"
                INSERT INTO planning_sessions (
                    id, plan_id, orchestrator_id, worker_pool_id, council_session_id,
                    audit_correlation_id, status, execution_state, started_at, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(session_id)
            .bind(session.plan_id)
            .bind("unified_orchestrator")
            .bind("mcp_worker_pool")
            .bind::<Option<Uuid>>(None)
            .bind(Uuid::new_v4())
            .bind("active")
            .bind(serde_json::json!({}))
            .bind(now)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to persist planning session: {}", e))?;

            info!("Persisted planning session {} to database", session_id);

            Ok(models::PlanningSession {
                id: session_id,
                plan_id: session.plan_id,
                status: "active".to_string(),
                created_at: now,
                updated_at: now,
                metadata: session.metadata,
            })
        }

        async fn get_planning_session(&self, id: Uuid) -> Result<Option<models::PlanningSession>> {
            let pool = self.db_client.pool();

            let row = sqlx::query(
                r#"
                SELECT id, plan_id, status, started_at, created_at
                FROM planning_sessions
                WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to query planning session: {}", e))?;

            Ok(row.map(|r: sqlx::postgres::PgRow| models::PlanningSession {
                id: r.get("id"),
                plan_id: r.get("plan_id"),
                status: r.get("status"),
                created_at: r.get("created_at"),
                updated_at: r
                    .try_get::<chrono::DateTime<Utc>, _>("started_at")
                    .unwrap_or_else(|_| r.get("created_at")),
                metadata: std::collections::HashMap::new(),
            }))
        }

        async fn update_planning_session(
            &self,
            id: Uuid,
            session: UpdatePlanningSession,
        ) -> Result<()> {
            let pool = self.db_client.pool();

            let mut updates = Vec::new();
            let mut bind_index = 1;

            if let Some(ref _status) = session.status {
                updates.push(format!("status = ${}", bind_index));
                bind_index += 1;
            }
            if let Some(ref _metadata) = session.metadata {
                updates.push(format!("execution_state = ${}", bind_index));
                bind_index += 1;
            }

            if updates.is_empty() {
                return Ok(());
            }

            let query = format!(
                "UPDATE planning_sessions SET {} WHERE id = ${}",
                updates.join(", "),
                bind_index
            );

            let mut query_builder = sqlx::query(&query);
            if let Some(ref status) = session.status {
                query_builder = query_builder.bind(status);
            }
            if let Some(ref metadata) = session.metadata {
                query_builder = query_builder.bind(serde_json::json!(metadata));
            }
            query_builder = query_builder.bind(id);

            query_builder
                .execute(pool)
                .await
                .map_err(|e| anyhow!("Failed to update planning session: {}", e))?;

            Ok(())
        }

        async fn create_planning_telemetry(
            &self,
            telemetry: CreatePlanningTelemetry,
        ) -> Result<models::PlanningTelemetry> {
            let pool = self.db_client.pool();
            let telemetry_id = Uuid::new_v4();
            let now = Utc::now();
            let plan_id = telemetry.session_id;

            sqlx::query(
                r#"
                INSERT INTO planning_telemetry (
                    id, plan_id, metric_type, metric_value, collected_at, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(telemetry_id)
            .bind(plan_id)
            .bind(&telemetry.metric_name)
            .bind(serde_json::json!(telemetry.metric_value))
            .bind(now)
            .bind(serde_json::to_value(&telemetry.metadata).unwrap_or(serde_json::json!({})))
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to persist planning telemetry: {}", e))?;

            info!("Persisted planning telemetry {} to database", telemetry_id);

            Ok(models::PlanningTelemetry {
                id: telemetry_id,
                session_id: telemetry.session_id,
                metric_name: telemetry.metric_name,
                metric_value: telemetry.metric_value,
                timestamp: now,
                metadata: telemetry.metadata,
            })
        }

        async fn get_planning_telemetry(
            &self,
            plan_id: Uuid,
            metric_type: Option<String>,
        ) -> Result<Vec<models::PlanningTelemetry>> {
            let pool = self.db_client.pool();

            let rows = if let Some(ref metric_type) = metric_type {
                sqlx::query(
                    r#"
                    SELECT id, plan_id, metric_type, metric_value, collected_at, metadata
                    FROM planning_telemetry
                    WHERE plan_id = $1 AND metric_type = $2
                    ORDER BY collected_at DESC
                    "#,
                )
                .bind(plan_id)
                .bind(metric_type)
                .fetch_all(pool)
                .await
            } else {
                sqlx::query(
                    r#"
                    SELECT id, plan_id, metric_type, metric_value, collected_at, metadata
                    FROM planning_telemetry
                    WHERE plan_id = $1
                    ORDER BY collected_at DESC
                    "#,
                )
                .bind(plan_id)
                .fetch_all(pool)
                .await
            }
            .map_err(|e| anyhow!("Failed to query planning telemetry: {}", e))?;

            Ok(rows
                .into_iter()
                .map(|r: sqlx::postgres::PgRow| {
                    let metric_value_json: serde_json::Value = r.get("metric_value");
                    let metric_value = metric_value_json
                        .as_f64()
                        .or_else(|| metric_value_json.as_i64().map(|v| v as f64))
                        .unwrap_or(0.0);

                    let metadata_json: serde_json::Value = r.get("metadata");
                    models::PlanningTelemetry {
                        id: r.get("id"),
                        session_id: r.get("plan_id"),
                        metric_name: r.get("metric_type"),
                        metric_value,
                        timestamp: r.get("collected_at"),
                        metadata: metadata_json
                            .as_object()
                            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                            .unwrap_or_default(),
                    }
                })
                .collect())
        }

        async fn create_planning_audit_event(&self, event: CreatePlanningAuditEvent) -> Result<()> {
            let pool = self.db_client.pool();
            let event_id = Uuid::new_v4();
            let now = Utc::now();

            let milestone_id = event
                .metadata
                .get("milestone_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let worker_id = event
                .metadata
                .get("worker_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok());

            sqlx::query(
                r#"
                INSERT INTO planning_audit_events (
                    id, plan_id, milestone_id, worker_id, event_type, description, metadata, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#
            )
            .bind(event_id)
            .bind(event.plan_id)
            .bind(milestone_id.as_deref())
            .bind(worker_id)
            .bind(&event.event_type)
            .bind(&event.description)
            .bind(serde_json::to_value(&event.metadata).unwrap_or(serde_json::json!({})))
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to persist planning audit event: {}", e))?;

            info!("Persisted planning audit event {} to database", event_id);

            Ok(())
        }

        async fn get_planning_audit_events(
            &self,
            plan_id: Uuid,
        ) -> Result<Vec<models::PlanningAuditEvent>> {
            let pool = self.db_client.pool();

            let rows = sqlx::query(
                r#"
                SELECT id, plan_id, milestone_id, worker_id, event_type, description, metadata, created_at
                FROM planning_audit_events
                WHERE plan_id = $1
                ORDER BY created_at DESC
                "#
            )
            .bind(plan_id)
            .fetch_all(pool)
            .await
            .map_err(|e| anyhow!("Failed to query planning audit events: {}", e))?;

            Ok(rows
                .into_iter()
                .map(|r: sqlx::postgres::PgRow| {
                    let id: Uuid = r.get("id");
                    let plan_id: Uuid = r.get("plan_id");
                    let event_type: String = r.get("event_type");
                    let description: String = r.get("description");
                    let created_at: chrono::DateTime<chrono::Utc> = r.get("created_at");
                    let metadata: serde_json::Value = r.get("metadata");

                    models::PlanningAuditEvent {
                        id,
                        session_id: plan_id,
                        event_type,
                        description,
                        timestamp: created_at,
                        metadata: metadata
                            .as_object()
                            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                            .unwrap_or_default(),
                    }
                })
                .collect())
        }

        async fn delete_execution_plan(&self, id: Uuid) -> Result<()> {
            let pool = self.db_client.pool();

            let rows_affected = sqlx::query(
                r#"
                DELETE FROM execution_plans
                WHERE id = $1
                "#,
            )
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to delete execution plan: {}", e))?
            .rows_affected();

            if rows_affected > 0 {
                info!("Deleted execution plan {} from database", id);
            } else {
                warn!("Execution plan {} not found for deletion", id);
            }

            Ok(())
        }

        async fn get_judges(&self) -> Result<Vec<models::Judge>> {
            use data_infrastructure::models::Judge as DbJudge;

            let pool = self.db_client.pool();
            let db_judges = sqlx::query_as::<_, DbJudge>(
                r#"
                SELECT id, name, model_name, endpoint, weight,
                       timeout_ms, optimization_target, is_active, created_at, updated_at
                FROM judges
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(pool)
            .await
            .map_err(|e| anyhow!("Failed to query judges from database: {}", e))?;

            let judges: Vec<models::Judge> = db_judges
                .into_iter()
                .map(|db_judge| {
                    let configuration = serde_json::json!({
                        "model_name": db_judge.model_name,
                        "endpoint": db_judge.endpoint,
                        "weight": db_judge.weight,
                        "timeout_ms": db_judge.timeout_ms,
                        "optimization_target": db_judge.optimization_target,
                    });

                    models::Judge {
                        id: db_judge.id,
                        name: db_judge.name,
                        judge_type: db_judge.optimization_target.clone(),
                        configuration,
                        is_active: db_judge.is_active,
                        metadata: std::collections::HashMap::new(),
                        created_at: db_judge.created_at,
                        updated_at: db_judge.updated_at,
                    }
                })
                .collect();

            info!("Queried {} judges from database", judges.len());
            Ok(judges)
        }

        async fn create_judge(&self, judge: CreateJudge) -> Result<models::Judge> {
            let pool = self.db_client.pool();
            let now = Utc::now();

            let model_name = judge
                .configuration
                .get("model_name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "default".to_string());
            let endpoint = judge
                .configuration
                .get("endpoint")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "http://localhost:8889".to_string());
            let weight = judge
                .configuration
                .get("weight")
                .and_then(|v| v.as_f64())
                .map(|v| v as f32)
                .unwrap_or(1.0);
            let timeout_ms = judge
                .configuration
                .get("timeout_ms")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32)
                .unwrap_or(5000);
            let optimization_target = judge
                .configuration
                .get("optimization_target")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| judge.judge_type.clone());

            sqlx::query(
                r#"
                INSERT INTO judges (
                    id, name, model_name, endpoint, weight,
                    timeout_ms, optimization_target, is_active, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(judge.id)
            .bind(&judge.name)
            .bind(&model_name)
            .bind(&endpoint)
            .bind(weight)
            .bind(timeout_ms)
            .bind(&optimization_target)
            .bind(true)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to persist judge to database: {}", e))?;

            info!("Persisted judge {} to database", judge.id);

            Ok(models::Judge {
                id: judge.id,
                name: judge.name,
                judge_type: judge.judge_type,
                configuration: judge.configuration,
                is_active: true,
                metadata: std::collections::HashMap::new(),
                created_at: now,
                updated_at: now,
            })
        }

        async fn get_judge(&self, id: Uuid) -> Result<Option<models::Judge>> {
            use data_infrastructure::models::Judge as DbJudge;

            let pool = self.db_client.pool();
            let db_judge = sqlx::query_as::<_, DbJudge>(
                r#"
                SELECT id, name, model_name, endpoint, weight,
                       timeout_ms, optimization_target, is_active, created_at, updated_at
                FROM judges
                WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to query judge from database: {}", e))?;

            Ok(db_judge.map(|db_judge| {
                let configuration = serde_json::json!({
                    "model_name": db_judge.model_name,
                    "endpoint": db_judge.endpoint,
                    "weight": db_judge.weight,
                    "timeout_ms": db_judge.timeout_ms,
                    "optimization_target": db_judge.optimization_target,
                });

                models::Judge {
                    id: db_judge.id,
                    name: db_judge.name,
                    judge_type: db_judge.optimization_target.clone(),
                    configuration,
                    is_active: db_judge.is_active,
                    metadata: std::collections::HashMap::new(),
                    created_at: db_judge.created_at,
                    updated_at: db_judge.updated_at,
                }
            }))
        }

        async fn create_judge_evaluation(
            &self,
            evaluation: CreateJudgeEvaluation,
        ) -> Result<models::JudgeEvaluation> {
            let pool = self.db_client.pool();
            let id = Uuid::new_v4();
            let now = Utc::now();

            let verdict_id = sqlx::query_scalar::<_, Option<Uuid>>(
                r#"
                SELECT id FROM council_verdicts WHERE task_id = $1 ORDER BY created_at DESC LIMIT 1
                "#,
            )
            .bind(evaluation.task_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to query verdict for task: {}", e))?
            .flatten()
            .unwrap_or_else(|| {
                warn!(
                    "No verdict found for task {}, using placeholder verdict_id",
                    evaluation.task_id
                );
                Uuid::new_v4()
            });

            let evaluation_score = evaluation
                .evaluation
                .get("score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            sqlx::query(
                r#"
                INSERT INTO judge_evaluations (
                    id, verdict_id, judge_id, judge_verdict, evaluation_time_ms,
                    tokens_used, confidence, created_at, evaluation_score,
                    confidence_score, reasoning, evidence_used, evaluation_metadata,
                    verdict_decision, risk_assessment, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                "#,
            )
            .bind(id)
            .bind(verdict_id)
            .bind(evaluation.judge_id)
            .bind(&evaluation.evaluation)
            .bind(0)
            .bind(None::<i32>)
            .bind(None::<f32>)
            .bind(now)
            .bind(Some(evaluation_score as f32))
            .bind(None::<f32>)
            .bind(
                evaluation
                    .evaluation
                    .get("reasoning")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            )
            .bind(Some(evaluation.evaluation.clone()))
            .bind(Some(evaluation.evaluation.clone()))
            .bind(
                evaluation
                    .evaluation
                    .get("verdict_decision")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            )
            .bind(evaluation.evaluation.get("risk_assessment").cloned())
            .bind(Some(now))
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to persist judge evaluation to database: {}", e))?;

            info!(
                "Persisted judge evaluation {} for judge {} and task {}",
                id, evaluation.judge_id, evaluation.task_id
            );

            let mut metadata = std::collections::HashMap::new();
            if let Some(obj) = evaluation.evaluation.as_object() {
                for (k, v) in obj {
                    metadata.insert(k.clone(), v.clone());
                }
            }

            Ok(models::JudgeEvaluation {
                id,
                judge_id: evaluation.judge_id,
                task_id: evaluation.task_id,
                evaluation: evaluation.evaluation,
                score: evaluation_score,
                metadata,
                created_at: now,
            })
        }

        async fn get_judge_evaluations(
            &self,
            task_id: Uuid,
        ) -> Result<Vec<models::JudgeEvaluation>> {
            use data_infrastructure::models::JudgeEvaluation as DbJudgeEvaluation;

            let pool = self.db_client.pool();

            let db_evaluations = sqlx::query_as::<_, DbJudgeEvaluation>(
                r#"
                SELECT id, verdict_id, judge_id, judge_verdict, evaluation_time_ms,
                       tokens_used, confidence, created_at, evaluation_score,
                       confidence_score, reasoning, evidence_used, evaluation_metadata,
                       verdict_decision, risk_assessment, updated_at
                FROM judge_evaluations
                WHERE verdict_id IN (
                    SELECT id FROM council_verdicts WHERE task_id = $1
                )
                ORDER BY created_at DESC
                "#,
            )
            .bind(task_id)
            .fetch_all(pool)
            .await
            .map_err(|e| anyhow!("Failed to query judge evaluations from database: {}", e))?;

            let evaluations: Vec<models::JudgeEvaluation> = db_evaluations
                .into_iter()
                .map(|db_eval| {
                    let mut evaluation_json = db_eval.judge_verdict.clone();

                    if let Some(reasoning) = &db_eval.reasoning {
                        evaluation_json["reasoning"] = serde_json::Value::String(reasoning.clone());
                    }
                    if let Some(verdict_decision) = &db_eval.verdict_decision {
                        evaluation_json["verdict_decision"] =
                            serde_json::Value::String(verdict_decision.clone());
                    }
                    if let Some(risk_assessment) = &db_eval.risk_assessment {
                        evaluation_json["risk_assessment"] = risk_assessment.clone();
                    }

                    let mut metadata = std::collections::HashMap::new();
                    if let Some(eval_metadata) = &db_eval.evaluation_metadata {
                        if let Some(obj) = eval_metadata.as_object() {
                            for (k, v) in obj {
                                metadata.insert(k.clone(), v.clone());
                            }
                        }
                    }
                    metadata.insert(
                        "verdict_id".to_string(),
                        serde_json::Value::String(db_eval.verdict_id.to_string()),
                    );
                    if let Some(tokens) = db_eval.tokens_used {
                        metadata.insert(
                            "tokens_used".to_string(),
                            serde_json::Value::Number(tokens.into()),
                        );
                    }
                    if let Some(confidence) = db_eval.confidence {
                        metadata.insert(
                            "confidence".to_string(),
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(confidence as f64)
                                    .unwrap_or(serde_json::Number::from(0)),
                            ),
                        );
                    }

                    models::JudgeEvaluation {
                        id: db_eval.id,
                        judge_id: db_eval.judge_id,
                        task_id,
                        evaluation: evaluation_json,
                        score: db_eval.evaluation_score.unwrap_or(0.0) as f64,
                        metadata,
                        created_at: db_eval.created_at,
                    }
                })
                .collect();

            info!(
                "Queried {} judge evaluations for task {} from database",
                evaluations.len(),
                task_id
            );
            Ok(evaluations)
        }

        async fn get_waivers(&self, status: Option<String>) -> Result<Vec<models::Waiver>> {
            let pool = self.db_client.pool();

            let query = if let Some(status_filter) = status {
                sqlx::query(
                    r#"
                    SELECT id, title, reason, description, gates, approved_by, impact_level,
                           mitigation_plan, expires_at, created_at, updated_at, status, metadata
                    FROM waivers
                    WHERE status = $1
                    ORDER BY created_at DESC
                    "#,
                )
                .bind(status_filter)
            } else {
                sqlx::query(
                    r#"
                    SELECT id, title, reason, description, gates, approved_by, impact_level,
                           mitigation_plan, expires_at, created_at, updated_at, status, metadata
                    FROM waivers
                    ORDER BY created_at DESC
                    "#,
                )
            };

            let rows = query
                .fetch_all(pool)
                .await
                .map_err(|e| anyhow!("Failed to query waivers: {}", e))?;

            let mut waivers = Vec::new();
            for row in rows {
                let db_waiver_id: Uuid = row.try_get("id")?;
                let db_waiver_title: String = row.try_get("title")?;
                let db_waiver_reason: String = row.try_get("reason")?;
                let db_waiver_description: String = row.try_get("description")?;
                let db_waiver_gates: serde_json::Value = row.try_get("gates")?;
                let db_waiver_approved_by: String = row.try_get("approved_by")?;
                let db_waiver_impact_level: String = row.try_get("impact_level")?;
                let db_waiver_mitigation_plan: Option<String> = row.try_get("mitigation_plan")?;
                let db_waiver_expires_at: Option<chrono::DateTime<Utc>> =
                    row.try_get("expires_at")?;
                let db_waiver_created_at: chrono::DateTime<Utc> = row.try_get("created_at")?;
                let db_waiver_updated_at: Option<chrono::DateTime<Utc>> =
                    row.try_get("updated_at")?;
                let db_waiver_status: String = row.try_get("status")?;
                let db_waiver_metadata: serde_json::Value = row.try_get("metadata")?;

                let gates = if let Some(gates_json) = db_waiver_gates.as_array() {
                    gates_json
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect()
                } else {
                    vec![]
                };

                let plan_id = db_waiver_metadata
                    .get("plan_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok())
                    .unwrap_or_else(|| Uuid::new_v4());

                let waiver_type = db_waiver_metadata
                    .get("waiver_type")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "general".to_string());

                let mut metadata = match db_waiver_metadata {
                    serde_json::Value::Object(ref map) => map.clone(),
                    _ => serde_json::Map::new(),
                };
                metadata.insert(
                    "title".to_string(),
                    serde_json::Value::String(db_waiver_title.clone()),
                );
                metadata.insert(
                    "description".to_string(),
                    serde_json::Value::String(db_waiver_description.clone()),
                );
                if let Some(updated_at) = db_waiver_updated_at {
                    metadata.insert(
                        "updated_at".to_string(),
                        serde_json::Value::String(updated_at.to_rfc3339()),
                    );
                }
                let metadata_map: std::collections::HashMap<String, serde_json::Value> =
                    metadata.into_iter().collect();

                waivers.push(models::Waiver {
                    id: db_waiver_id,
                    plan_id,
                    waiver_type,
                    reason: db_waiver_reason,
                    approved_by: db_waiver_approved_by,
                    status: db_waiver_status,
                    gates,
                    impact_level: db_waiver_impact_level,
                    mitigation_plan: db_waiver_mitigation_plan,
                    created_at: db_waiver_created_at,
                    expires_at: db_waiver_expires_at,
                    metadata: metadata_map,
                });
            }

            Ok(waivers)
        }

        async fn create_waiver(&self, waiver: CreateWaiver) -> Result<models::Waiver> {
            let pool = self.db_client.pool();
            let id = Uuid::new_v4();
            let now = Utc::now();
            let gates_json = serde_json::to_value(&waiver.waived_gates)
                .map_err(|e| anyhow!("Failed to serialize gates: {}", e))?;

            let mut metadata = serde_json::Map::new();
            metadata.insert(
                "plan_id".to_string(),
                serde_json::Value::String(waiver.plan_id.to_string()),
            );
            let metadata_value = serde_json::Value::Object(metadata);

            let title = format!("Waiver for plan {}", waiver.plan_id);
            let description = waiver.reason.clone();
            let approved_by = "system".to_string();
            let impact_level = "medium".to_string();
            let mitigation_plan = None::<String>;

            sqlx::query(
                r#"
                INSERT INTO waivers (
                    id, title, reason, description, gates, approved_by, impact_level,
                    mitigation_plan, expires_at, created_at, updated_at, status, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                "#,
            )
            .bind(id)
            .bind(&title)
            .bind(&waiver.reason)
            .bind(&description)
            .bind(&gates_json)
            .bind(&approved_by)
            .bind(&impact_level)
            .bind(mitigation_plan)
            .bind(None::<chrono::DateTime<Utc>>)
            .bind(now)
            .bind(now)
            .bind("active")
            .bind(&metadata_value)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to create waiver: {}", e))?;

            let row = sqlx::query(
                r#"
                SELECT id, title, reason, description, gates, approved_by, impact_level,
                       mitigation_plan, expires_at, created_at, updated_at, status, metadata
                FROM waivers
                WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch created waiver: {}", e))?;

            let db_waiver_id: Uuid = row.try_get("id")?;
            let db_waiver_title: String = row.try_get("title")?;
            let db_waiver_reason: String = row.try_get("reason")?;
            let db_waiver_description: String = row.try_get("description")?;
            let db_waiver_gates: serde_json::Value = row.try_get("gates")?;
            let db_waiver_approved_by: String = row.try_get("approved_by")?;
            let db_waiver_impact_level: String = row.try_get("impact_level")?;
            let db_waiver_mitigation_plan: Option<String> = row.try_get("mitigation_plan")?;
            let db_waiver_expires_at: Option<chrono::DateTime<Utc>> = row.try_get("expires_at")?;
            let db_waiver_created_at: chrono::DateTime<Utc> = row.try_get("created_at")?;
            let db_waiver_updated_at: Option<chrono::DateTime<Utc>> = row.try_get("updated_at")?;
            let db_waiver_status: String = row.try_get("status")?;
            let db_waiver_metadata: serde_json::Value = row.try_get("metadata")?;

            let gates = if let Some(gates_json) = db_waiver_gates.as_array() {
                gates_json
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            } else {
                vec![]
            };

            let plan_id = db_waiver_metadata
                .get("plan_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(|| Uuid::new_v4());

            let waiver_type = db_waiver_metadata
                .get("waiver_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "general".to_string());

            let mut metadata = match db_waiver_metadata {
                serde_json::Value::Object(ref map) => map.clone(),
                _ => serde_json::Map::new(),
            };
            metadata.insert(
                "title".to_string(),
                serde_json::Value::String(db_waiver_title.clone()),
            );
            metadata.insert(
                "description".to_string(),
                serde_json::Value::String(db_waiver_description.clone()),
            );
            if let Some(updated_at) = db_waiver_updated_at {
                metadata.insert(
                    "updated_at".to_string(),
                    serde_json::Value::String(updated_at.to_rfc3339()),
                );
            }
            let metadata_map: std::collections::HashMap<String, serde_json::Value> =
                metadata.into_iter().collect();

            Ok(models::Waiver {
                id: db_waiver_id,
                plan_id,
                waiver_type,
                reason: db_waiver_reason,
                approved_by: db_waiver_approved_by,
                status: db_waiver_status,
                gates,
                impact_level: db_waiver_impact_level,
                mitigation_plan: db_waiver_mitigation_plan,
                created_at: db_waiver_created_at,
                expires_at: db_waiver_expires_at,
                metadata: metadata_map,
            })
        }

        async fn update_waiver(&self, id: Uuid, update: UpdateWaiver) -> Result<models::Waiver> {
            let pool = self.db_client.pool();
            let now = Utc::now();

            if !update.status.is_empty() {
                sqlx::query("UPDATE waivers SET status = $1, updated_at = $2 WHERE id = $3")
                    .bind(&update.status)
                    .bind(now)
                    .bind(id)
                    .execute(pool)
                    .await
                    .map_err(|e| anyhow!("Failed to update waiver status: {}", e))?;
            } else {
                sqlx::query("UPDATE waivers SET updated_at = $1 WHERE id = $2")
                    .bind(now)
                    .bind(id)
                    .execute(pool)
                    .await
                    .map_err(|e| anyhow!("Failed to update waiver updated_at: {}", e))?;
            }

            let row = sqlx::query(
                r#"
                SELECT id, title, reason, description, gates, approved_by, impact_level,
                       mitigation_plan, expires_at, created_at, updated_at, status, metadata
                FROM waivers
                WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(|e| anyhow!("Failed to fetch updated waiver: {}", e))?;

            let db_waiver_id: Uuid = row.try_get("id")?;
            let db_waiver_title: String = row.try_get("title")?;
            let db_waiver_reason: String = row.try_get("reason")?;
            let db_waiver_description: String = row.try_get("description")?;
            let db_waiver_gates: serde_json::Value = row.try_get("gates")?;
            let db_waiver_approved_by: String = row.try_get("approved_by")?;
            let db_waiver_impact_level: String = row.try_get("impact_level")?;
            let db_waiver_mitigation_plan: Option<String> = row.try_get("mitigation_plan")?;
            let db_waiver_expires_at: Option<chrono::DateTime<Utc>> = row.try_get("expires_at")?;
            let db_waiver_created_at: chrono::DateTime<Utc> = row.try_get("created_at")?;
            let db_waiver_updated_at: Option<chrono::DateTime<Utc>> = row.try_get("updated_at")?;
            let db_waiver_status: String = row.try_get("status")?;
            let db_waiver_metadata: serde_json::Value = row.try_get("metadata")?;

            let gates = if let Some(gates_json) = db_waiver_gates.as_array() {
                gates_json
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            } else {
                vec![]
            };

            let plan_id = db_waiver_metadata
                .get("plan_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(|| Uuid::new_v4());

            let waiver_type = db_waiver_metadata
                .get("waiver_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "general".to_string());

            let mut metadata = match db_waiver_metadata {
                serde_json::Value::Object(ref map) => map.clone(),
                _ => serde_json::Map::new(),
            };
            metadata.insert(
                "title".to_string(),
                serde_json::Value::String(db_waiver_title.clone()),
            );
            metadata.insert(
                "description".to_string(),
                serde_json::Value::String(db_waiver_description.clone()),
            );
            if let Some(updated_at) = db_waiver_updated_at {
                metadata.insert(
                    "updated_at".to_string(),
                    serde_json::Value::String(updated_at.to_rfc3339()),
                );
            }
            let metadata_map: std::collections::HashMap<String, serde_json::Value> =
                metadata.into_iter().collect();

            Ok(models::Waiver {
                id: db_waiver_id,
                plan_id,
                waiver_type,
                reason: db_waiver_reason,
                approved_by: db_waiver_approved_by,
                status: db_waiver_status,
                gates,
                impact_level: db_waiver_impact_level,
                mitigation_plan: db_waiver_mitigation_plan,
                created_at: db_waiver_created_at,
                expires_at: db_waiver_expires_at,
                metadata: metadata_map,
            })
        }

        async fn create_execution_result(
            &self,
            result: CreateExecutionResult,
        ) -> Result<models::PlanExecutionResult> {
            let pool = self.db_client.pool();
            let now = Utc::now();

            sqlx::query(
                r#"
                INSERT INTO plan_execution_results (
                    plan_id, success, milestones_completed, total_duration_ms,
                    evidence, metrics, final_state, timeline, created_at, updated_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (plan_id) DO UPDATE SET
                    success = EXCLUDED.success,
                    milestones_completed = EXCLUDED.milestones_completed,
                    total_duration_ms = EXCLUDED.total_duration_ms,
                    evidence = EXCLUDED.evidence,
                    metrics = EXCLUDED.metrics,
                    final_state = EXCLUDED.final_state,
                    timeline = EXCLUDED.timeline,
                    updated_at = EXCLUDED.updated_at
                "#,
            )
            .bind(result.plan_id)
            .bind(result.success)
            .bind(result.milestones_completed as i32)
            .bind(result.total_duration_ms as i64)
            .bind(&result.evidence)
            .bind(&result.metrics)
            .bind(&result.final_state)
            .bind(&result.timeline)
            .bind(now)
            .bind(now)
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to persist execution result: {}", e))?;

            info!(
                "Persisted execution result for plan {} to database",
                result.plan_id
            );

            self.get_execution_result(result.plan_id)
                .await?
                .ok_or_else(|| anyhow!("Failed to retrieve persisted execution result"))
        }

        async fn get_execution_result(
            &self,
            plan_id: Uuid,
        ) -> Result<Option<models::PlanExecutionResult>> {
            let pool = self.db_client.pool();

            let row = sqlx::query(
                r#"
                SELECT plan_id, success, milestones_completed, total_duration_ms,
                       evidence, metrics, final_state, timeline, created_at, updated_at
                FROM plan_execution_results
                WHERE plan_id = $1
                "#,
            )
            .bind(plan_id)
            .fetch_optional(pool)
            .await
            .map_err(|e| anyhow!("Failed to query execution result: {}", e))?;

            Ok(
                row.map(|r: sqlx::postgres::PgRow| models::PlanExecutionResult {
                    plan_id: r.get("plan_id"),
                    success: r.get("success"),
                    milestones_completed: r.get("milestones_completed"),
                    total_duration_ms: r.get("total_duration_ms"),
                    evidence: r
                        .try_get::<serde_json::Value, _>("evidence")
                        .unwrap_or_else(|_| serde_json::json!({})),
                    metrics: r
                        .try_get::<serde_json::Value, _>("metrics")
                        .unwrap_or_else(|_| serde_json::json!({})),
                    final_state: r.get("final_state"),
                    timeline: r
                        .try_get::<serde_json::Value, _>("timeline")
                        .unwrap_or_else(|_| serde_json::json!([])),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                }),
            )
        }

        async fn create_council_session(
            &self,
            session: CreateCouncilSession,
        ) -> Result<models::CouncilSession> {
            use data_infrastructure::database_operations::CreateCouncilSession as DbCreateCouncilSession;

            let db_session = DbCreateCouncilSession {
                session_id: session.session_id,
                task_id: session.task_id,
                working_spec_id: session.working_spec_id,
                review_context: session.review_context,
                status: session.status,
                selected_judges: session.selected_judges,
                contributions: session.contributions,
                progress: session.progress,
                metadata: session.metadata,
            };

            let created = DataInfraDatabaseOperations::create_council_session(
                self.db_client.as_ref(),
                db_session,
            )
            .await
            .map_err(|e| anyhow!("Failed to create council session: {}", e))?;

            Ok(models::CouncilSession {
                id: created.id,
                session_id: created.session_id,
                task_id: created.task_id,
                working_spec_id: created.working_spec_id,
                review_context: created.review_context,
                status: created.status,
                selected_judges: created.selected_judges,
                contributions: created.contributions,
                aggregation_result: created.aggregation_result,
                final_decision: created.final_decision,
                progress: created.progress,
                started_at: created.started_at,
                completed_at: created.completed_at,
                created_at: created.created_at,
                updated_at: created.updated_at,
                metadata: created.metadata,
            })
        }

        async fn get_council_session(
            &self,
            session_id: Uuid,
        ) -> Result<Option<models::CouncilSession>> {
            let session = DataInfraDatabaseOperations::get_council_session(
                self.db_client.as_ref(),
                session_id,
            )
            .await
            .map_err(|e| anyhow!("Failed to get council session: {}", e))?;

            Ok(session.map(|s| models::CouncilSession {
                id: s.id,
                session_id: s.session_id,
                task_id: s.task_id,
                working_spec_id: s.working_spec_id,
                review_context: s.review_context,
                status: s.status,
                selected_judges: s.selected_judges,
                contributions: s.contributions,
                aggregation_result: s.aggregation_result,
                final_decision: s.final_decision,
                progress: s.progress,
                started_at: s.started_at,
                completed_at: s.completed_at,
                created_at: s.created_at,
                updated_at: s.updated_at,
                metadata: s.metadata,
            }))
        }

        async fn get_council_session_by_task(
            &self,
            task_id: Uuid,
        ) -> Result<Option<models::CouncilSession>> {
            let session = DataInfraDatabaseOperations::get_council_session_by_task(
                self.db_client.as_ref(),
                task_id,
            )
            .await
            .map_err(|e| anyhow!("Failed to get council session by task: {}", e))?;

            Ok(session.map(|s| models::CouncilSession {
                id: s.id,
                session_id: s.session_id,
                task_id: s.task_id,
                working_spec_id: s.working_spec_id,
                review_context: s.review_context,
                status: s.status,
                selected_judges: s.selected_judges,
                contributions: s.contributions,
                aggregation_result: s.aggregation_result,
                final_decision: s.final_decision,
                progress: s.progress,
                started_at: s.started_at,
                completed_at: s.completed_at,
                created_at: s.created_at,
                updated_at: s.updated_at,
                metadata: s.metadata,
            }))
        }

        async fn update_council_session(
            &self,
            session_id: Uuid,
            update: UpdateCouncilSession,
        ) -> Result<models::CouncilSession> {
            use data_infrastructure::database_operations::UpdateCouncilSession as DbUpdateCouncilSession;

            let db_update = DbUpdateCouncilSession {
                status: update.status,
                selected_judges: update.selected_judges,
                contributions: update.contributions,
                aggregation_result: update.aggregation_result,
                final_decision: update.final_decision,
                progress: update.progress,
                completed_at: update.completed_at,
                metadata: update.metadata,
            };

            let updated = DataInfraDatabaseOperations::update_council_session(
                self.db_client.as_ref(),
                session_id,
                db_update,
            )
            .await
            .map_err(|e| anyhow!("Failed to update council session: {}", e))?;

            Ok(models::CouncilSession {
                id: updated.id,
                session_id: updated.session_id,
                task_id: updated.task_id,
                working_spec_id: updated.working_spec_id,
                review_context: updated.review_context,
                status: updated.status,
                selected_judges: updated.selected_judges,
                contributions: updated.contributions,
                aggregation_result: updated.aggregation_result,
                final_decision: updated.final_decision,
                progress: updated.progress,
                started_at: updated.started_at,
                completed_at: updated.completed_at,
                created_at: updated.created_at,
                updated_at: updated.updated_at,
                metadata: updated.metadata,
            })
        }
    }
}

use database_operations_adapter::DatabaseOperationsAdapter;

/// Stub database operations implementation (kept for reference, no longer used)
#[allow(dead_code)]
struct StubDatabaseOperations;

#[async_trait]
impl DatabaseOperations for StubDatabaseOperations {
    async fn get_workers(
        &self,
    ) -> Result<Vec<crate::planning::data_infrastructure_types::models::Worker>> {
        Ok(vec![])
    }
    async fn create_execution_plan(
        &self,
        _plan: crate::planning::data_infrastructure_types::CreateExecutionPlan,
    ) -> Result<crate::planning::data_infrastructure_types::models::ExecutionPlan> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_execution_plan(
        &self,
        _id: Uuid,
    ) -> Result<Option<crate::planning::data_infrastructure_types::models::ExecutionPlan>> {
        Ok(None)
    }
    async fn get_execution_plans(
        &self,
    ) -> Result<Vec<crate::planning::data_infrastructure_types::models::ExecutionPlan>> {
        Ok(vec![])
    }
    async fn update_execution_plan(
        &self,
        _id: Uuid,
        _update: crate::planning::data_infrastructure_types::UpdateExecutionPlan,
    ) -> Result<crate::planning::data_infrastructure_types::models::ExecutionPlan> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn create_audit_trail_entry(
        &self,
        _entry: crate::planning::data_infrastructure_types::CreateAuditTrailEntry,
    ) -> Result<crate::planning::data_infrastructure_types::models::AuditTrailEntry> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_audit_trail_entries(
        &self,
        _task_id: Uuid,
    ) -> Result<Vec<crate::planning::data_infrastructure_types::models::AuditTrailEntry>> {
        Ok(vec![])
    }
    async fn get_audit_trail_entry(
        &self,
        _id: Uuid,
    ) -> Result<Option<crate::planning::data_infrastructure_types::models::AuditTrailEntry>> {
        Ok(None)
    }
    async fn create_planning_session(
        &self,
        _session: crate::planning::data_infrastructure_types::CreatePlanningSession,
    ) -> Result<crate::planning::data_infrastructure_types::models::PlanningSession> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_planning_session(
        &self,
        _id: Uuid,
    ) -> Result<Option<crate::planning::data_infrastructure_types::models::PlanningSession>> {
        Ok(None)
    }
    async fn update_planning_session(
        &self,
        _id: Uuid,
        _session: crate::planning::data_infrastructure_types::UpdatePlanningSession,
    ) -> Result<()> {
        Ok(())
    }
    async fn create_planning_telemetry(
        &self,
        _telemetry: crate::planning::data_infrastructure_types::CreatePlanningTelemetry,
    ) -> Result<crate::planning::data_infrastructure_types::models::PlanningTelemetry> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_planning_telemetry(
        &self,
        _plan_id: Uuid,
        _metric_type: Option<String>,
    ) -> Result<Vec<crate::planning::data_infrastructure_types::models::PlanningTelemetry>> {
        Ok(vec![])
    }
    async fn create_planning_audit_event(
        &self,
        _event: crate::planning::data_infrastructure_types::CreatePlanningAuditEvent,
    ) -> Result<()> {
        Ok(())
    }
    async fn get_planning_audit_events(
        &self,
        _plan_id: Uuid,
    ) -> Result<Vec<crate::planning::data_infrastructure_types::models::PlanningAuditEvent>> {
        Ok(vec![])
    }
    async fn delete_execution_plan(&self, _id: Uuid) -> Result<()> {
        Ok(())
    }
    async fn get_judges(
        &self,
    ) -> Result<Vec<crate::planning::data_infrastructure_types::models::Judge>> {
        Ok(vec![])
    }
    async fn create_judge(
        &self,
        _judge: crate::planning::data_infrastructure_types::CreateJudge,
    ) -> Result<crate::planning::data_infrastructure_types::models::Judge> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_judge(
        &self,
        _id: Uuid,
    ) -> Result<Option<crate::planning::data_infrastructure_types::models::Judge>> {
        Ok(None)
    }
    async fn create_judge_evaluation(
        &self,
        _evaluation: crate::planning::data_infrastructure_types::CreateJudgeEvaluation,
    ) -> Result<crate::planning::data_infrastructure_types::models::JudgeEvaluation> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_judge_evaluations(
        &self,
        _task_id: Uuid,
    ) -> Result<Vec<crate::planning::data_infrastructure_types::models::JudgeEvaluation>> {
        Ok(vec![])
    }
    async fn get_waivers(
        &self,
        _status: Option<String>,
    ) -> Result<Vec<crate::planning::data_infrastructure_types::models::Waiver>> {
        Ok(vec![])
    }
    async fn create_waiver(
        &self,
        _waiver: crate::planning::data_infrastructure_types::CreateWaiver,
    ) -> Result<crate::planning::data_infrastructure_types::models::Waiver> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn update_waiver(
        &self,
        _id: Uuid,
        _update: crate::planning::data_infrastructure_types::UpdateWaiver,
    ) -> Result<crate::planning::data_infrastructure_types::models::Waiver> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn create_execution_result(
        &self,
        _result: crate::planning::data_infrastructure_types::CreateExecutionResult,
    ) -> Result<crate::planning::data_infrastructure_types::models::PlanExecutionResult> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_execution_result(
        &self,
        _plan_id: Uuid,
    ) -> Result<Option<crate::planning::data_infrastructure_types::models::PlanExecutionResult>>
    {
        Ok(None)
    }
    async fn get_worker(
        &self,
        _id: Uuid,
    ) -> Result<Option<crate::planning::data_infrastructure_types::models::Worker>> {
        Ok(None)
    }
    async fn create_worker(
        &self,
        _worker: crate::planning::data_infrastructure_types::CreateWorker,
    ) -> Result<crate::planning::data_infrastructure_types::models::Worker> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn update_worker(
        &self,
        _id: Uuid,
        _update: crate::planning::data_infrastructure_types::UpdateWorker,
    ) -> Result<crate::planning::data_infrastructure_types::models::Worker> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn create_council_session(
        &self,
        _session: crate::planning::data_infrastructure_types::CreateCouncilSession,
    ) -> Result<crate::planning::data_infrastructure_types::models::CouncilSession> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
    async fn get_council_session(
        &self,
        _session_id: Uuid,
    ) -> Result<Option<crate::planning::data_infrastructure_types::models::CouncilSession>> {
        Ok(None)
    }
    async fn get_council_session_by_task(
        &self,
        _task_id: Uuid,
    ) -> Result<Option<crate::planning::data_infrastructure_types::models::CouncilSession>> {
        Ok(None)
    }
    async fn update_council_session(
        &self,
        _session_id: Uuid,
        _update: crate::planning::data_infrastructure_types::UpdateCouncilSession,
    ) -> Result<crate::planning::data_infrastructure_types::models::CouncilSession> {
        Err(anyhow::anyhow!("Stub implementation"))
    }
}
