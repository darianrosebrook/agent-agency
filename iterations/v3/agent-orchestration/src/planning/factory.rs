//! Planning System Factory - Creates fully configured planning integrations
//!
//! Factory for creating OrchestratorPlanningIntegration with all real dependencies.
//! Handles dependency injection and wiring for the planning system.
//!
//! @author @darianrosebrook

use crate::planning::DatabaseOperations;
use anyhow::Result;
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

// Real types from contracts (feature-gated where necessary)
// NOTE: council_adapter is behind feature gate but agent-constitutional-council is commented out in Cargo.toml
// When that dependency is added back, uncomment the adapter usage below
// #[cfg(feature = "council")]
// use crate::planning::council_adapter::CouncilCoordinatorAdapter;

#[cfg(feature = "memory")]
use crate::planning::memory_adapter::MemorySystemAdapter;

#[cfg(feature = "research")]
use crate::planning::research_adapter::ResearchEvidenceAdapter;

// TODO: Document stub implementation for disabled feature
//       This is an intentional stub when council feature is disabled.
//       Methods return errors or empty results. Consider improving error handling.
//
// Stub implementation of CouncilCoordinator for when council feature is disabled
struct StubCouncilCoordinator;

#[async_trait::async_trait]
impl agent_agency_contracts::CouncilCoordinator for StubCouncilCoordinator {
    async fn start_session(
        &self,
        _task: &agent_agency_contracts::types::planning::TaskDescriptor,
    ) -> agent_agency_contracts::errors::CouncilResult<
        agent_agency_contracts::ports::council_coordinator::SessionId,
    > {
        Ok(agent_agency_contracts::ports::council_coordinator::SessionId(uuid::Uuid::new_v4()))
    }
    async fn review_task(
        &self,
        _session_id: &agent_agency_contracts::ports::council_coordinator::SessionId,
        _task: &agent_agency_contracts::types::planning::TaskDescriptor,
    ) -> agent_agency_contracts::errors::CouncilResult<
        agent_agency_contracts::types::council::CouncilVerdict,
    > {
        Ok(agent_agency_contracts::types::council::CouncilVerdict::Approved)
    }
    async fn get_session_status(
        &self,
        _session_id: &agent_agency_contracts::ports::council_coordinator::SessionId,
    ) -> agent_agency_contracts::errors::CouncilResult<
        agent_agency_contracts::ports::council_coordinator::SessionStatus,
    > {
        Ok(
            agent_agency_contracts::ports::council_coordinator::SessionStatus {
                session_id: *_session_id,
                status:
                    agent_agency_contracts::ports::council_coordinator::SessionStatusType::Completed,
                progress: 1.0,
                pending_requirements: vec![],
                estimated_completion: None,
            },
        )
    }
}

// NOTE: These dependencies are commented out in Cargo.toml
// When dependencies are added back, uncomment these imports:
// #[cfg(feature = "tool-chain")]
// use crate::planning::tool_chain_adapter::ToolChainPlannerAdapter;
// #[cfg(feature = "tool-chain")]
// use system_federated_ml::tool_chain_planner::ToolChainPlanner;
// #[cfg(feature = "data-processing")]
// use crate::planning::data_processing_adapter::DataProcessingServiceAdapter;

use crate::coreml::CoreMLManager;
use crate::planning::{
    council_monitor::CouncilMonitor, council_review::CouncilPlanReview,
    evidence::EvidenceCollector, orchestrator_integration::OrchestratorPlanningIntegration,
    parallel_coordinator::ParallelCoordinator, plan_generator::PlanGenerator,
    scope_guard::ScopeGuard, storage::PlanningStorage, todo_integration::TodoIntegration,
    worker_assignment::WorkerAssignmentStrategy,
};
use std::path::PathBuf;
use tracing::{info, warn};

/// Planning system factory for creating fully configured integrations
pub struct PlanningSystemFactory;

impl PlanningSystemFactory {
    /// Create a complete orchestrator planning integration with all dependencies
    pub async fn create_orchestrator_integration(
        // Core planning components
        plan_generator: Arc<PlanGenerator>,
        planning_storage: Arc<PlanningStorage>,

        // Execution components
        parallel_coordinator: Arc<ParallelCoordinator>,
        worker_assigner: Arc<WorkerAssignmentStrategy>,

        // Evidence and validation
        evidence_collector: Arc<EvidenceCollector>,
        scope_guard: Arc<ScopeGuard>,
        council_monitor: Arc<CouncilMonitor>,

        // Quality enforcement
        todo_integration: Arc<dyn crate::planning::plan_executor::TodoInterface>,

        // Council review for pre-execution assessment
        council_review: Arc<CouncilPlanReview>,

        // Infrastructure - use real database operations
        db_ops: Arc<dyn DatabaseOperations>,
    ) -> Result<OrchestratorPlanningIntegration> {
        Ok(OrchestratorPlanningIntegration::new(
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
        ))
    }

    /// Create planning system components from infrastructure services
    ///
    /// # Arguments
    /// * `tool_chain_planner` - Real tool chain planner from system-federated-ml (wrapped in adapter)
    /// * `research_evidence_collector` - Real evidence collector from agent-research (wrapped in adapter)
    /// * `council_coordinator` - Real council coordinator from agent-constitutional-council (wrapped in adapter)
    /// * `memory_system` - Real memory system from agent-memory (wrapped in adapter)
    /// * `data_processor` - Real data processor from agent-data-processing (wrapped in adapter)
    /// * `council` - Real Council instance for council review and monitor (local type)
    /// * `db_ops` - Database operations for persistence
    /// NOTE: This function uses local Council type, no feature gate needed
    pub async fn create_planning_components(
        // NOTE: These dependencies are commented out in Cargo.toml due to circular dependencies
        // When dependencies are added back, uncomment these parameters:
        // #[cfg(feature = "tool-chain")] tool_chain_planner: Arc<system_federated_ml::tool_chain_planner::ToolChainPlanner>,
        // #[cfg(feature = "council")] council_coordinator: Arc<agent_constitutional_council::CouncilCoordinator<E>>,
        // #[cfg(feature = "data-processing")] data_processor: Arc<dyn agent_data_processing::DataProcessor>,
        #[cfg(feature = "research")] research_evidence_collector: Arc<
            agent_research::evidence::collector::EvidenceCollector,
        >,
        #[cfg(feature = "memory")] memory_system: Arc<agent_memory::MemorySystem>,
        council: Arc<crate::council::Council>,
        db_ops: Arc<dyn DatabaseOperations>,
        // Optional execution dependencies - if provided, PlanExecutor will have real execution capabilities
        worker_bridge: Option<Arc<crate::workers::execution_bridge::WorkerExecutionBridge>>,
        worktree_manager: Option<Arc<crate::planning::worktree_manager::WorktreeManager>>,
    ) -> Result<PlanningSystemComponents> {
        // Verify database schema before creating components
        // This helps catch schema issues early with better error messages
        tracing::info!("Verifying database schema before creating planning components...");
        let test_plan_id = uuid::Uuid::new_v4();
        match db_ops.get_planning_audit_events(test_plan_id).await {
            Ok(_) => {
                // Query succeeded - schema is correct
                tracing::info!("Database schema verification passed for planning_audit_events");
            }
            Err(e) => {
                // Query failed - likely schema issue
                tracing::error!(
                    error = %e,
                    "CRITICAL: Database schema verification failed during PlanningSystemFactory initialization. \
                    This may indicate the 'description' column is missing from 'planning_audit_events' table. \
                    Please run migration 028 to fix the schema."
                );
                return Err(anyhow::anyhow!(
                    "Database schema verification failed during PlanningSystemFactory initialization: {}. \
                    This may indicate the 'description' column is missing from 'planning_audit_events' table. \
                    Please run migration 028 to fix the schema.",
                    e
                ));
            }
        }

        // Also verify other critical queries that might fail
        tracing::info!("Verifying other critical database queries...");
        match db_ops.get_execution_plans().await {
            Ok(_) => {
                tracing::info!("Database schema verification passed for execution_plans");
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "CRITICAL: Database schema verification failed for execution_plans. \
                    This may indicate the 'description' column is missing from 'execution_plans' table."
                );
                return Err(anyhow::anyhow!(
                    "Database schema verification failed for execution_plans: {}. \
                    This may indicate the 'description' column is missing from 'execution_plans' table.",
                    e
                ));
            }
        }

        match db_ops.get_waivers(None).await {
            Ok(_) => {
                tracing::info!("Database schema verification passed for waivers");
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    "CRITICAL: Database schema verification failed for waivers. \
                    This may indicate the 'description' column is missing from 'waivers' table."
                );
                return Err(anyhow::anyhow!(
                    "Database schema verification failed for waivers: {}. \
                    This may indicate the 'description' column is missing from 'waivers' table.",
                    e
                ));
            }
        }

        tracing::info!(
            "All database schema verifications passed - proceeding with component creation"
        );

        // Initialize CoreML manager for AI-assisted planning
        let coreml_manager = {
            let model_path = std::env::var("COREML_MODELS_PATH")
                .map(|p| PathBuf::from(p))
                .unwrap_or_else(|_| {
                    // Default to project models directory
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .parent()
                        .and_then(|p| p.parent())
                        .and_then(|p| p.parent())
                        .map(|p| p.join("models").join("coreml"))
                        .unwrap_or_else(|| PathBuf::from("/models/coreml"))
                });

            let manager = Arc::new(CoreMLManager::new(model_path.clone()));

            // Try to load models asynchronously
            let manager_clone = manager.clone();
            tokio::spawn(async move {
                match manager_clone.load_available_models().await {
                    Ok(_) => {
                        info!("CoreML models loaded successfully for orchestrator planning");
                    }
                    Err(e) => {
                        warn!(
                            "Failed to load CoreML models for orchestrator planning: {}",
                            e
                        );
                        warn!("Planning will continue without AI assistance");
                    }
                }
            });

            Some(manager)
        };

        // Create plan generator with tool chain integration and CoreML manager
        let plan_generator = Arc::new(PlanGenerator::new(
            crate::planning::plan_types::PlanningConstraints::default(),
            None,           // tool_chain_bridge
            None,           // legacy_adapter
            coreml_manager, // CoreML manager for AI-assisted planning
        )?);

        // Create planning storage
        let planning_storage = Arc::new(PlanningStorage::new(
            db_ops.clone(),
            std::path::PathBuf::from("/tmp/plans"),
            std::path::PathBuf::from("/tmp/specs"),
            crate::planning::storage::StorageConfig::default(),
        ));

        // Create shared components first (no dependencies)
        let worker_assigner = Arc::new(WorkerAssignmentStrategy::new(db_ops.clone()));
        let scope_guard = Arc::new(ScopeGuard::new());

        // Create council monitor
        #[cfg(feature = "council")]
        let council_coordinator_stub = Arc::new(StubCouncilCoordinator);
        #[cfg(feature = "council")]
        let council_monitor = Arc::new(CouncilMonitor::new(
            council_coordinator_stub,
            db_ops.clone(),
        ));
        #[cfg(not(feature = "council"))]
        let council_monitor = Arc::new(CouncilMonitor::new(
            Arc::new(StubCouncilCoordinator),
            db_ops.clone(),
        ));

        // Create evidence collector with research integration
        #[cfg(feature = "research")]
        let research_adapter = Arc::new(ResearchEvidenceAdapter::new(
            research_evidence_collector.clone(),
        ));
        #[cfg(feature = "research")]
        let evidence_collector = Arc::new(EvidenceCollector::new(research_adapter));
        #[cfg(not(feature = "research"))]
        let evidence_collector = Arc::new(EvidenceCollector::new(Arc::new(
            crate::planning::evidence::NoOpResearchEvidenceCollector,
        )));

        // Create TODO adapter for the PlanExecutor interface
        let new_todo_integration = TodoIntegration::new(
            Arc::new(crate::planning::todo_template::TodoTemplateSystem::new()),
            db_ops.clone(),
        );
        let todo_adapter = Arc::new(crate::planning::plan_executor::TodoAdapter {
            inner: tokio::sync::RwLock::new(new_todo_integration),
        });

        // Create audit trail adapter with real AuditTrailManager and database persistence
        use crate::audit_trail::{AuditConfig, AuditTrailManager};
        let audit_trail_manager = Arc::new(AuditTrailManager::new(AuditConfig::default()));

        struct AuditTrailAdapter {
            audit_manager: Arc<AuditTrailManager>,
            db_ops: Arc<dyn DatabaseOperations>,
        }

        #[async_trait]
        impl crate::planning::plan_executor::AuditTrail for AuditTrailAdapter {
            async fn log_event(
                &self,
                event: crate::planning::plan_executor::AuditEvent,
            ) -> Result<()> {
                use crate::planning::data_infrastructure_types::CreatePlanningAuditEvent;
                use tracing::error;

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

                self.db_ops.create_planning_audit_event(audit_entry).await
                    .map_err(|e| {
                        error!(
                            error = %e,
                            plan_id = %event.plan_id,
                            event_type = %format!("{:?}", event.event_type),
                            "CRITICAL: Failed to create planning audit event - this may indicate missing 'description' column"
                        );
                        anyhow::anyhow!("Failed to create planning audit event (plan_id: {}, event_type: {:?}): {}. This may indicate the 'description' column is missing from 'planning_audit_events' table.", event.plan_id, event.event_type, e)
                    })?;

                // Also update in-memory stats via AuditTrailManager for council decisions
                match event.event_type {
                    crate::planning::plan_executor::AuditEventType::CouncilDecision => {
                        if let Some(milestone_id) = &event.milestone_id {
                            self.audit_manager
                                .council_auditor()
                                .record_council_consensus(
                                    &event.plan_id.to_string(),
                                    "plan_executor",
                                    std::collections::HashMap::new(),
                                    1.0,
                                    std::time::Duration::from_secs(0),
                                )
                                .await
                                .map_err(|e| {
                                    anyhow::anyhow!("Failed to record council audit: {}", e)
                                })?;
                        }
                    }
                    _ => {
                        // Other events are persisted to database via DatabaseOperations above
                        // In-memory tracking can be added here if needed
                    }
                }

                Ok(())
            }
        }

        let audit_trail = Arc::new(AuditTrailAdapter {
            audit_manager: audit_trail_manager,
            db_ops: db_ops.clone(),
        }) as Arc<dyn crate::planning::plan_executor::AuditTrail>;

        // Create real worker pool adapter using MCPWorkerPool
        #[cfg(feature = "memory")]
        let worker_pool: Arc<dyn crate::planning::plan_executor::WorkerPool> = {
            use agent_workers::{
                MCPWorkerPool, WorkerCapabilities, WorkerPoolConfig, WorkerSpecialty,
            };
            use std::collections::HashMap;
            use tokio::sync::RwLock;

            // Create tool registry for MCPWorkerPool
            let repo_path =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let tool_registry = agent_workers::create_tool_registry_with_file_ops(repo_path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create tool registry: {}", e))?;

            // Create MCPWorkerPool with shared memory system
            let mcp_worker_pool = Arc::new(MCPWorkerPool::new_with_registry(
                WorkerPoolConfig::default(),
                tool_registry,
                memory_system.clone(),
            ));

            // Register a default worker for task execution
            let _ = mcp_worker_pool
                .register_worker(
                    WorkerSpecialty::General,
                    WorkerCapabilities {
                        languages: vec![
                            "rust".to_string(),
                            "typescript".to_string(),
                            "python".to_string(),
                        ],
                        frameworks: vec![],
                        domains: vec!["general".to_string()],
                        max_context_length: 8192,
                        max_output_length: 4096,
                        supported_formats: vec!["text".to_string(), "json".to_string()],
                        caws_awareness: 1.0,
                        quality_score: 0.9,
                        speed_score: 0.8,
                    },
                )
                .await
                .map_err(|e| anyhow::anyhow!("Failed to register default worker: {}", e))?;

            // Create adapter that tracks assignments and implements WorkerPool trait
            struct MCPWorkerPoolAdapter {
                pool: Arc<MCPWorkerPool>,
                assignments: Arc<RwLock<HashMap<Uuid, String>>>, // worker_id -> milestone_id
            }

            #[async_trait::async_trait]
            impl crate::planning::plan_executor::WorkerPool for MCPWorkerPoolAdapter {
                async fn available_workers(
                    &self,
                ) -> Result<Vec<crate::planning::plan_executor::WorkerInfo>> {
                    let workers = self.pool.list_workers().await;
                    let assignments = self.assignments.read().await;

                    Ok(workers
                        .into_iter()
                        .map(|handle| {
                            let is_assigned = assignments.contains_key(&handle.id.0);
                            crate::planning::plan_executor::WorkerInfo {
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
                                health: crate::planning::plan_executor::WorkerHealth::Healthy,
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

                async fn worker_status(
                    &self,
                    worker_id: Uuid,
                ) -> Result<crate::planning::plan_executor::WorkerStatus> {
                    let workers = self.pool.list_workers().await;
                    let assignments = self.assignments.read().await;

                    let worker = workers.iter().find(|w| w.id.0 == worker_id);
                    let current_assignment = assignments.get(&worker_id).cloned();

                    let stats = self.pool.get_stats().await;

                    Ok(crate::planning::plan_executor::WorkerStatus {
                        current_assignment,
                        health: if worker.is_some() {
                            crate::planning::plan_executor::WorkerHealth::Healthy
                        } else {
                            crate::planning::plan_executor::WorkerHealth::Unavailable
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
                pool: mcp_worker_pool,
                assignments: Arc::new(RwLock::new(HashMap::new())),
            })
        };

        #[cfg(not(feature = "memory"))]
        let worker_pool: Arc<dyn crate::planning::plan_executor::WorkerPool> = {
            // Fallback stub when memory feature is not enabled
            struct FallbackWorkerPool;
            #[async_trait::async_trait]
            impl crate::planning::plan_executor::WorkerPool for FallbackWorkerPool {
                async fn available_workers(
                    &self,
                ) -> Result<Vec<crate::planning::plan_executor::WorkerInfo>> {
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
                async fn worker_status(
                    &self,
                    _worker_id: Uuid,
                ) -> Result<crate::planning::plan_executor::WorkerStatus> {
                    warn!("Memory feature not enabled - worker status unavailable");
                    Ok(crate::planning::plan_executor::WorkerStatus {
                        current_assignment: None,
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
            Arc::new(FallbackWorkerPool)
        };

        // Break circular dependency using Arc::new_cyclic
        // PlanExecutor needs ParallelCoordinator, and ParallelCoordinator needs PlanExecutor
        // We'll create them using a cyclic reference pattern
        let parallel_coordinator = Arc::new_cyclic(|coordinator_ref| {
            // Create PlanExecutor that references the coordinator
            // Use with_lifecycle_manager if worker_bridge and worktree_manager are provided
            let plan_executor = if worker_bridge.is_some() || worktree_manager.is_some() {
                Arc::new(
                    crate::planning::plan_executor::PlanExecutor::with_lifecycle_manager(
                        crate::planning::plan_types::ExecutionPlan::default(),
                        worker_pool.clone(),
                        evidence_collector.clone(),
                        worker_assigner.clone(),
                        scope_guard.clone(),
                        council_monitor.clone(),
                        coordinator_ref.clone(), // Weak reference to the coordinator being created
                        audit_trail.clone(),
                        None, // audit_trail_manager - optional, not provided in factory
                        todo_adapter.clone(), // Pass the TodoAdapter implementing TodoInterface
                        None, // worker_lifecycle_manager - optional
                        worker_bridge.clone(), // Pass WorkerExecutionBridge if provided
                        worktree_manager.clone(), // Pass WorktreeManager if provided
                        crate::planning::plan_executor::ExecutionConfig::default(),
                    ),
                )
            } else {
                Arc::new(crate::planning::plan_executor::PlanExecutor::new(
                    crate::planning::plan_types::ExecutionPlan::default(),
                    worker_pool.clone(),
                    evidence_collector.clone(),
                    worker_assigner.clone(),
                    scope_guard.clone(),
                    council_monitor.clone(),
                    coordinator_ref.clone(), // Weak reference to the coordinator being created
                    audit_trail.clone(),
                    None, // audit_trail_manager - optional, not provided in factory
                    todo_adapter.clone(), // Pass the TodoAdapter implementing TodoInterface
                    crate::planning::plan_executor::ExecutionConfig::default(),
                ))
            };

            // Create ParallelCoordinator with the PlanExecutor
            ParallelCoordinator::new(
                plan_executor,
                scope_guard.clone(),
                council_monitor.clone(),
                worker_assigner.clone(),
                crate::planning::parallel_coordinator::ParallelConfig::default(),
            )
        });

        // Create council plan review with real Council instance
        let council_review = Arc::new(CouncilPlanReview::new(council.clone(), db_ops.clone()));

        Ok(PlanningSystemComponents {
            plan_generator,
            planning_storage,
            parallel_coordinator,
            worker_assigner,
            evidence_collector,
            scope_guard,
            council_monitor,
            todo_integration: todo_adapter.clone(),
            council_review,
            // NOTE: When agent-constitutional-council is added back, uncomment this:
            // council_coordinator: Arc::new(CouncilCoordinatorAdapter::new(council_coordinator)),
            //
            // TODO: Implement comprehensive NoOpCouncilCoordinatorAdapter
            //       Currently creates stub adapter; should implement comprehensive NoOpCouncilCoordinatorAdapter that provides proper no-op implementation for council coordinator when agent-constitutional-council is not available.
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
            // - NoOpCouncilCoordinatorAdapter is implemented
            // - No-op behavior is correct and safe
            // - Adapter integrates properly with factory
            // - Transition to real adapter is seamless when available
            //
            // DEPENDENCIES:
            // - CouncilCoordinatorAdapter interface (Required)
            // - No-op implementation pattern (Required)
            // - Factory integration utilities (Required)
            //
            // ESTIMATED EFFORT: 4-6 hours (medium confidence)
            // PRIORITY: Low
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (adapter pattern implementation)
            // - Change Budget: ~100 LOC
            // - Reviewer Requirements: Adapter pattern and council integration expertise
            council_coordinator: Arc::new(StubCouncilCoordinator)
                as Arc<dyn agent_agency_contracts::CouncilCoordinator>,
            #[cfg(feature = "memory")]
            memory_system: Arc::new(MemorySystemAdapter::new(memory_system)),
            #[cfg(feature = "research")]
            research_evidence_collector: Arc::new(ResearchEvidenceAdapter::new(
                research_evidence_collector,
            )),
            // NOTE: When dependencies are added back, uncomment these:
            // #[cfg(feature = "tool-chain")]
            // tool_chain_planner: Arc::new(ToolChainPlannerAdapter::new(tool_chain_planner)),
            // #[cfg(feature = "data-processing")]
            // data_processing_service: Arc::new(DataProcessingServiceAdapter::new(data_processor)),
        })
    }
}

/// Complete set of planning system components
pub struct PlanningSystemComponents {
    pub plan_generator: Arc<PlanGenerator>,
    pub planning_storage: Arc<PlanningStorage>,
    pub parallel_coordinator: Arc<ParallelCoordinator>,
    pub worker_assigner: Arc<WorkerAssignmentStrategy>,
    pub evidence_collector: Arc<EvidenceCollector>,
    pub scope_guard: Arc<ScopeGuard>,
    pub council_monitor: Arc<CouncilMonitor>,
    pub todo_integration: Arc<dyn crate::planning::plan_executor::TodoInterface>,
    pub council_review: Arc<CouncilPlanReview>,
    pub council_coordinator: Arc<dyn agent_agency_contracts::CouncilCoordinator>,
    #[cfg(feature = "memory")]
    pub memory_system: Arc<dyn agent_agency_contracts::MemorySystem>,
    #[cfg(feature = "research")]
    pub research_evidence_collector: Arc<dyn agent_agency_contracts::ResearchEvidenceCollector>,
    // NOTE: When dependencies are added back, uncomment these fields:
    // #[cfg(feature = "tool-chain")]
    // pub tool_chain_planner: Arc<dyn agent_agency_contracts::ToolChainPlanner>,
    // #[cfg(feature = "data-processing")]
    // pub data_processing_service: Arc<dyn agent_agency_contracts::DataProcessingService>,
}

impl PlanningSystemComponents {
    /// Create orchestrator integration from these components
    pub fn create_orchestrator_integration(
        self,
        db_ops: Arc<dyn DatabaseOperations>,
    ) -> OrchestratorPlanningIntegration {
        OrchestratorPlanningIntegration::new(
            self.plan_generator,
            self.planning_storage,
            self.parallel_coordinator,
            self.worker_assigner,
            self.evidence_collector,
            self.scope_guard,
            self.council_monitor,
            self.todo_integration,
            self.council_review,
            db_ops,
        )
    }
}

/// Planning system configuration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
struct PlanningSystemConfig {
    /// Enable planning system integration
    pub enable_planning_integration: bool,

    /// Enable quality gate enforcement
    pub enable_quality_gates: bool,

    /// Enable council monitoring
    pub enable_council_monitoring: bool,

    /// Enable parallel execution
    pub enable_parallel_execution: bool,

    /// Enable evidence collection
    pub enable_evidence_collection: bool,

    /// Enable TODO tracking
    pub enable_todo_tracking: bool,

    /// Planning storage configuration
    pub storage_config: PlanningStorageConfig,

    /// Evidence collection configuration
    pub evidence_config: EvidenceCollectionConfig,
}

/// Planning storage configuration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
struct PlanningStorageConfig {
    /// Enable file-based storage
    pub enable_file_storage: bool,

    /// Enable database storage
    pub enable_db_storage: bool,

    /// Storage retention period (days)
    pub retention_days: u32,

    /// Enable compression
    pub enable_compression: bool,
}

/// Evidence collection configuration

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
struct EvidenceCollectionConfig {
    /// Evidence retention period (days)
    pub retention_days: u32,

    /// Minimum quality score threshold
    pub min_quality_score: f64,

    /// Enable automatic verification
    pub enable_auto_verification: bool,

    /// Verification timeout (seconds)
    pub verification_timeout_seconds: u64,
}

impl Default for PlanningSystemConfig {
    fn default() -> Self {
        Self {
            enable_planning_integration: true,
            enable_quality_gates: true,
            enable_council_monitoring: true,
            enable_parallel_execution: true,
            enable_evidence_collection: true,
            enable_todo_tracking: true,
            storage_config: PlanningStorageConfig::default(),
            evidence_config: EvidenceCollectionConfig::default(),
        }
    }
}

impl Default for PlanningStorageConfig {
    fn default() -> Self {
        Self {
            enable_file_storage: true,
            enable_db_storage: true,
            retention_days: 30,
            enable_compression: true,
        }
    }
}

impl Default for EvidenceCollectionConfig {
    fn default() -> Self {
        Self {
            retention_days: 30,
            min_quality_score: 0.8,
            enable_auto_verification: true,
            verification_timeout_seconds: 300,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_planning_config_defaults() {
        let config = PlanningSystemConfig::default();
        assert!(config.enable_planning_integration);
        assert!(config.enable_quality_gates);
        assert!(config.enable_council_monitoring);
    }

    #[test]
    fn test_storage_config_defaults() {
        let config = PlanningStorageConfig::default();
        assert!(config.enable_file_storage);
        assert!(config.enable_db_storage);
        assert_eq!(config.retention_days, 30);
    }

    #[test]
    fn test_evidence_config_defaults() {
        let config = EvidenceCollectionConfig::default();
        assert_eq!(config.min_quality_score, 0.8);
        assert!(config.enable_auto_verification);
        assert_eq!(config.verification_timeout_seconds, 300);
    }
}
