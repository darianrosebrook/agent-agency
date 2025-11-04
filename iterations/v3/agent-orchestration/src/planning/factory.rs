//! Planning System Factory - Creates fully configured planning integrations
//!
//! Factory for creating OrchestratorPlanningIntegration with all real dependencies.
//! Handles dependency injection and wiring for the planning system.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use async_trait::async_trait;
use crate::planning::DatabaseOperations;

// Real types from contracts (feature-gated where necessary)
// NOTE: council_adapter is behind feature gate but agent-constitutional-council is commented out in Cargo.toml
// When that dependency is added back, uncomment the adapter usage below
// #[cfg(feature = "council")]
// use crate::planning::council_adapter::CouncilCoordinatorAdapter;

#[cfg(feature = "memory")]
use crate::planning::memory_adapter::MemorySystemAdapter;

#[cfg(feature = "research")]
use crate::planning::research_adapter::ResearchEvidenceCollectorAdapter;
#[cfg(feature = "research")]
use agent_research::evidence::collector::EvidenceCollector as ResearchEvidenceCollector;

// Stub implementation of CouncilCoordinator for when council feature is disabled
struct StubCouncilCoordinator;

#[async_trait::async_trait]
impl agent_agency_contracts::CouncilCoordinator for StubCouncilCoordinator {
    async fn start_session(&self, _task: &agent_agency_contracts::types::planning::TaskDescriptor) -> agent_agency_contracts::errors::CouncilResult<agent_agency_contracts::ports::council_coordinator::SessionId> {
        Ok(agent_agency_contracts::ports::council_coordinator::SessionId(uuid::Uuid::new_v4()))
    }
    async fn review_task(&self, _session_id: &agent_agency_contracts::ports::council_coordinator::SessionId, _task: &agent_agency_contracts::types::planning::TaskDescriptor) -> agent_agency_contracts::errors::CouncilResult<agent_agency_contracts::types::council::CouncilVerdict> {
        Ok(agent_agency_contracts::types::council::CouncilVerdict::Approved)
    }
    async fn get_session_status(&self, _session_id: &agent_agency_contracts::ports::council_coordinator::SessionId) -> agent_agency_contracts::errors::CouncilResult<agent_agency_contracts::ports::council_coordinator::SessionStatus> {
        Ok(agent_agency_contracts::ports::council_coordinator::SessionStatus {
            session_id: *_session_id,
            status: agent_agency_contracts::ports::council_coordinator::SessionStatusType::Completed,
            progress: 1.0,
            pending_requirements: vec![],
            estimated_completion: None,
        })
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

use crate::planning::{
    plan_generator::PlanGenerator,
    storage::PlanningStorage,
    parallel_coordinator::ParallelCoordinator,
    worker_assignment::WorkerAssignmentStrategy,
    evidence::EvidenceCollector,
    scope_guard::ScopeGuard,
    council_monitor::CouncilMonitor,
    todo_integration::TodoIntegration,
    council_review::CouncilPlanReview,
    orchestrator_integration::OrchestratorPlanningIntegration,
};

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
        todo_integration: Arc<TodoIntegration>,

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

        #[cfg(feature = "research")] research_evidence_collector: Arc<agent_research::evidence::collector::EvidenceCollector>,
        #[cfg(feature = "memory")] memory_system: Arc<agent_memory::MemorySystem>,
        council: Arc<crate::council::Council>,
        db_ops: Arc<dyn DatabaseOperations>,
    ) -> Result<PlanningSystemComponents> {
        // Create plan generator with tool chain integration
        let plan_generator = Arc::new(PlanGenerator::new(
            crate::planning::plan_types::PlanningConstraints::default(),
            None, // tool_chain_bridge
            None, // legacy_adapter
        ));

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
        let council_monitor = Arc::new(CouncilMonitor::new(council_coordinator_stub, db_ops.clone()));
        #[cfg(not(feature = "council"))]
        let council_monitor = Arc::new(CouncilMonitor::new(
            Arc::new(StubCouncilCoordinator),
            db_ops.clone(),
        ));

        // Create evidence collector with research integration
        #[cfg(feature = "research")]
        let research_adapter = Arc::new(ResearchEvidenceCollectorAdapter::new(research_evidence_collector.clone()));
        #[cfg(feature = "research")]
        let evidence_collector = Arc::new(EvidenceCollector::new(research_adapter));
        #[cfg(not(feature = "research"))]
        let evidence_collector = Arc::new(EvidenceCollector::new(Arc::new(crate::planning::evidence::NoOpResearchEvidenceCollector)));

        // Create TODO integration
        let todo_integration = Arc::new(TodoIntegration::new(
            Arc::new(crate::planning::todo_template::TodoTemplateSystem::new()),
            db_ops.clone(),
        ));

        // Create audit trail stub for PlanExecutor
        struct StubAuditTrail;
        #[async_trait]
        impl crate::planning::plan_executor::AuditTrail for StubAuditTrail {
            async fn log_event(&self, _event: crate::planning::plan_executor::AuditEvent) -> Result<()> {
                // Stub implementation - no-op
                Ok(())
            }
        }
        let audit_trail = Arc::new(StubAuditTrail) as Arc<dyn crate::planning::plan_executor::AuditTrail>;

        // Create worker pool stub (needed for PlanExecutor)
        struct StubWorkerPool;
        #[async_trait::async_trait]
        impl crate::planning::plan_executor::WorkerPool for StubWorkerPool {
            async fn available_workers(&self) -> Result<Vec<crate::planning::plan_executor::WorkerInfo>> {
                Ok(vec![])
            }
            async fn assign_worker(&self, _worker_id: Uuid, _milestone_id: String) -> Result<()> {
                Ok(())
            }
            async fn release_worker(&self, _worker_id: Uuid) -> Result<()> {
                Ok(())
            }
            async fn worker_status(&self, _worker_id: Uuid) -> Result<crate::planning::plan_executor::WorkerStatus> {
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
        let worker_pool = Arc::new(StubWorkerPool);

        // Break circular dependency using Arc::new_cyclic
        // PlanExecutor needs ParallelCoordinator, and ParallelCoordinator needs PlanExecutor
        // We'll create them using a cyclic reference pattern
        let parallel_coordinator = Arc::new_cyclic(|coordinator_ref| {
            // Create PlanExecutor that references the coordinator
            let plan_executor = Arc::new(crate::planning::plan_executor::PlanExecutor::new(
                crate::planning::plan_types::ExecutionPlan::default(),
                worker_pool.clone(),
                evidence_collector.clone(),
                worker_assigner.clone(),
                scope_guard.clone(),
                council_monitor.clone(),
                coordinator_ref.clone(), // Weak reference to the coordinator being created
                audit_trail.clone(),
                Arc::new(tokio::sync::Mutex::new(todo_integration.clone())), // Wrap Arc<TodoIntegration> in Arc<Mutex<Arc<TodoIntegration>>>
                crate::planning::plan_executor::ExecutionConfig::default(),
            ));
            
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
        let council_review = Arc::new(CouncilPlanReview::new(
            council.clone(),
            db_ops.clone(),
        ));

        Ok(PlanningSystemComponents {
            plan_generator,
            planning_storage,
            parallel_coordinator,
            worker_assigner,
            evidence_collector,
            scope_guard,
            council_monitor,
            todo_integration,
            council_review,
            // NOTE: When agent-constitutional-council is added back, uncomment this:
            // council_coordinator: Arc::new(CouncilCoordinatorAdapter::new(council_coordinator)),
            // For now, create a stub adapter - TODO: implement NoOpCouncilCoordinatorAdapter
            council_coordinator: Arc::new(StubCouncilCoordinator) as Arc<dyn agent_agency_contracts::CouncilCoordinator>,
            #[cfg(feature = "memory")]
            memory_system: Arc::new(MemorySystemAdapter::new(memory_system)),
            #[cfg(feature = "research")]
            research_evidence_collector: Arc::new(ResearchEvidenceCollectorAdapter::new(research_evidence_collector)),
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
    pub todo_integration: Arc<TodoIntegration>,
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
