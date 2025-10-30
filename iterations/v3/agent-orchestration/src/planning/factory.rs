//! Planning System Factory - Creates fully configured planning integrations
//!
//! Factory for creating OrchestratorPlanningIntegration with all real dependencies.
//! Handles dependency injection and wiring for the planning system.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use anyhow::Result;
// TODO: Use real external dependencies
// use agent_constitutional_council::CouncilCoordinator;
// use system_federated_ml::tool_chain_planner::ToolChainPlanner;
// use agent_research::evidence::collector::EvidenceCollector as ResearchEvidenceCollector;
// use data_infrastructure::DatabaseOperations;

// Stub types for missing dependencies
#[derive(Debug)]
pub struct CouncilCoordinator<E> {
    _phantom: std::marker::PhantomData<E>,
}

#[derive(Debug)]
pub struct ToolChainPlanner;

#[derive(Debug)]
pub struct ResearchEvidenceCollector;

pub trait DatabaseOperations {
    fn create_execution_plan(&self, _plan: serde_json::Value) -> Result<serde_json::Value, String> { Ok(serde_json::Value::Null) }
    fn get_execution_plan(&self, _id: uuid::Uuid) -> Result<Option<serde_json::Value>, String> { Ok(None) }
    fn get_execution_plans(&self) -> Result<Vec<serde_json::Value>, String> { Ok(vec![]) }
    fn update_execution_plan(&self, _id: uuid::Uuid, _update: serde_json::Value) -> Result<serde_json::Value, String> { Err("Not implemented".to_string()) }
    fn delete_execution_plan(&self, _id: uuid::Uuid) -> Result<(), String> { Ok(()) }
}

use crate::planning::{
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

        // Infrastructure
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
            db_ops,
        ))
    }

    /// Create planning system components from infrastructure services
    pub async fn create_planning_components<E>(
        tool_chain_planner: Arc<ToolChainPlanner>,
        research_evidence_collector: Arc<ResearchEvidenceCollector>,
        council_coordinator: Arc<CouncilCoordinator<E>>,
        db_ops: Arc<dyn DatabaseOperations>,
    ) -> Result<PlanningSystemComponents<E>> {
        // Create plan generator with tool chain integration
        let plan_generator = Arc::new(PlanGenerator::new());

        // Create planning storage
        let planning_storage = Arc::new(PlanningStorage::new(db_ops.clone()));

        // Create parallel coordinator
        let parallel_coordinator = Arc::new(ParallelCoordinator::new());

        // Create worker assignment strategy
        let worker_assigner = Arc::new(WorkerAssignmentStrategy::new(db_ops.clone()));

        // Create evidence collector with research integration
        let evidence_collector = Arc::new(EvidenceCollector::new(research_evidence_collector));

        // Create scope guard for file locking
        let scope_guard = Arc::new(ScopeGuard::new());

        // Create council monitor
        let council_monitor = Arc::new(CouncilMonitor::new(council_coordinator, db_ops.clone()));

        // Create TODO integration
        let todo_integration = Arc::new(TodoIntegration::new(
            Arc::new(crate::planning::todo_template::TodoTemplateSystem::new()),
            db_ops.clone(),
        ));

        // Create council plan review
        let council_review = Arc::new(CouncilPlanReview::new(
            council_coordinator.clone(),
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
        })
    }
}

/// Complete set of planning system components
pub struct PlanningSystemComponents<E: agent_agency_contracts::JudgeEngine> {
    pub plan_generator: Arc<PlanGenerator>,
    pub planning_storage: Arc<PlanningStorage>,
    pub parallel_coordinator: Arc<ParallelCoordinator>,
    pub worker_assigner: Arc<WorkerAssignmentStrategy>,
    pub evidence_collector: Arc<EvidenceCollector>,
    pub scope_guard: Arc<ScopeGuard>,
    pub council_monitor: Arc<CouncilMonitor>,
    pub todo_integration: Arc<TodoIntegration>,
    pub council_review: Arc<CouncilPlanReview<E>>,
}

impl<E: agent_agency_contracts::JudgeEngine> PlanningSystemComponents<E> {
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
#[derive(Debug, Clone)]
pub struct PlanningSystemConfig {
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
#[derive(Debug, Clone)]
pub struct PlanningStorageConfig {
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
#[derive(Debug, Clone)]
pub struct EvidenceCollectionConfig {
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
