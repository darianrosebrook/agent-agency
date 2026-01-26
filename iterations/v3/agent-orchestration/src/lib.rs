//! Agent Orchestration Service - Unified orchestration and governance system
//!
//! This crate consolidates the orchestration and council functionality from Agent Agency V3,
//! providing a comprehensive system for:
//! - Task orchestration and execution (from orchestration crate)
//! - Decision making and arbitration (from council crate)
//! - Multimodal processing coordination
//! - Autonomous execution with governance
//! - Quality assurance and audit trails
//!
//! The consolidation unifies the planning, decision-making, and execution layers
//! into a single, coherent orchestration service.
//!
//! @author @darianrosebrook

// ============================================================================
// EXTERNAL DEPENDENCIES
// ============================================================================

#[macro_use]
extern crate tracing;

use std::sync::Arc;
use uuid::Uuid;

#[cfg(feature = "api-server")]
use schemars::JsonSchema;
#[cfg(feature = "api-server")]
use serde::{Deserialize, Serialize};

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

pub mod consensus_coordinator;
pub mod learning;
pub mod progress_tracker;
mod quality_gates;

#[cfg(any(feature = "data-processing", feature = "memory"))]
pub mod workspace_integration;

// Re-export types for convenience - use contracts types
pub use agent_agency_contracts::types::prelude::{
    AcceptanceCriterion, BlastRadius, ExecutionContext, ExecutionMode, RiskTier, TaskDescriptor,
    TaskPriority,
};
// Re-export contracts WorkingSpec (local WorkingSpec is deprecated - use contracts)
pub use agent_agency_contracts::WorkingSpec;
// Import from contracts - TaskExecutionResult and QualityReport are now in contracts
pub use agent_agency_contracts::quality_report::QualityReport;
pub use agent_agency_contracts::task_executor::TaskExecutionResult;
pub use agent_agency_contracts::ExecutionArtifacts;

// Task executor factory
pub use planning::task_executor_factory::{
    ExecutionStrategy, TaskExecutorConfig, TaskExecutorFactory, TaskExecutorFactoryError,
};

// ============================================================================
// COUNCIL MODULES (Decision Making & Arbitration)
// ============================================================================

pub mod council;
// pub mod judge;
pub mod decision_making;
pub mod verdict_aggregation;
// pub mod verdict;
pub mod workflow;
// pub mod risk_scorer; // TEMPORARILY DISABLED: Missing type definitions
pub mod council_errors;
pub mod council_types;
pub mod error_handling;
pub mod evidence_enrichment;
pub mod planning;
// pub mod coordinator;

// Test module for restored functionality
#[cfg(test)]
mod restored_tests;

// Examples module for restored functionality
pub mod restored_examples;

// Test utilities
#[cfg(test)]
mod test_utils {
    use crate::planning::DatabaseOperations;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    // Mock database operations for testing with in-memory storage
    pub struct MockDatabaseOps {
        // In-memory storage
        sessions: Arc<
            tokio::sync::RwLock<
                std::collections::HashMap<
                    Uuid,
                    crate::planning::data_infrastructure_types::models::PlanningSession,
                >,
            >,
        >,
        plans: Arc<
            tokio::sync::RwLock<
                std::collections::HashMap<
                    Uuid,
                    crate::planning::data_infrastructure_types::models::ExecutionPlan,
                >,
            >,
        >,
        execution_results: Arc<
            tokio::sync::RwLock<
                std::collections::HashMap<
                    Uuid,
                    crate::planning::data_infrastructure_types::models::PlanExecutionResult,
                >,
            >,
        >,
        audit_trails: Arc<
            tokio::sync::RwLock<
                std::collections::HashMap<
                    Uuid,
                    crate::planning::data_infrastructure_types::models::AuditTrailEntry,
                >,
            >,
        >,
    }

    impl Default for MockDatabaseOps {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockDatabaseOps {
        /// Create a new mock database operations instance
        pub fn new() -> Self {
            Self {
                sessions: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
                plans: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
                execution_results: Arc::new(tokio::sync::RwLock::new(
                    std::collections::HashMap::new(),
                )),
                audit_trails: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl DatabaseOperations for MockDatabaseOps {
        async fn create_execution_plan(
            &self,
            plan: crate::planning::data_infrastructure_types::CreateExecutionPlan,
        ) -> Result<crate::planning::data_infrastructure_types::models::ExecutionPlan, anyhow::Error>
        {
            use chrono::Utc;
            let now = Utc::now();
            let db_plan = crate::planning::data_infrastructure_types::models::ExecutionPlan {
                id: plan.id,
                session_id: Uuid::new_v4(),
                workspace_id: None,
                working_spec_id: plan
                    .working_spec_id
                    .unwrap_or_else(|| format!("PLAN-{}", plan.id)),
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
            };

            self.plans.write().await.insert(plan.id, db_plan.clone());
            Ok(db_plan)
        }
        async fn get_execution_plan(
            &self,
            id: Uuid,
        ) -> Result<
            Option<crate::planning::data_infrastructure_types::models::ExecutionPlan>,
            anyhow::Error,
        > {
            Ok(self.plans.read().await.get(&id).cloned())
        }
        async fn get_execution_plans(
            &self,
        ) -> Result<
            Vec<crate::planning::data_infrastructure_types::models::ExecutionPlan>,
            anyhow::Error,
        > {
            Ok(self.plans.read().await.values().cloned().collect())
        }
        async fn update_execution_plan(
            &self,
            id: Uuid,
            update: crate::planning::data_infrastructure_types::UpdateExecutionPlan,
        ) -> Result<crate::planning::data_infrastructure_types::models::ExecutionPlan, anyhow::Error>
        {
            use chrono::Utc;
            let mut plans = self.plans.write().await;
            let plan = plans
                .get_mut(&id)
                .ok_or_else(|| anyhow::anyhow!("Execution plan not found: {}", id))?;

            if let Some(title) = update.title {
                plan.title = title;
            }
            if let Some(overview) = update.overview {
                plan.overview = Some(overview);
            }
            if let Some(status) = update.status {
                plan.state = status;
            }
            plan.updated_at = Utc::now();

            Ok(plan.clone())
        }
        async fn create_audit_trail_entry(
            &self,
            entry: crate::planning::data_infrastructure_types::CreateAuditTrailEntry,
        ) -> Result<
            crate::planning::data_infrastructure_types::models::AuditTrailEntry,
            anyhow::Error,
        > {
            use chrono::Utc;
            let id = Uuid::new_v4();
            let db_entry = crate::planning::data_infrastructure_types::models::AuditTrailEntry {
                id,
                event_type: entry.event_type,
                description: entry.description,
                timestamp: Utc::now(),
                metadata: entry.metadata,
            };

            self.audit_trails.write().await.insert(id, db_entry.clone());
            Ok(db_entry)
        }
        async fn get_audit_trail_entries(
            &self,
            _task_id: Uuid,
        ) -> Result<
            Vec<crate::planning::data_infrastructure_types::models::AuditTrailEntry>,
            anyhow::Error,
        > {
            Ok(vec![])
        }
        async fn get_audit_trail_entry(
            &self,
            _id: Uuid,
        ) -> Result<
            Option<crate::planning::data_infrastructure_types::models::AuditTrailEntry>,
            anyhow::Error,
        > {
            Ok(None)
        }
        async fn create_planning_session(
            &self,
            session: crate::planning::data_infrastructure_types::CreatePlanningSession,
        ) -> Result<
            crate::planning::data_infrastructure_types::models::PlanningSession,
            anyhow::Error,
        > {
            use chrono::Utc;
            // Extract session_id from metadata if present (PlanningStorage generates it and stores it there)
            // Otherwise generate a new one
            let id = session
                .metadata
                .get("session_id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or_else(Uuid::new_v4);
            let now = Utc::now();

            // Extract status from metadata if present, otherwise default to "active"
            let status = session
                .metadata
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("active")
                .to_string();

            let db_session = crate::planning::data_infrastructure_types::models::PlanningSession {
                id,
                plan_id: session.plan_id,
                status,
                created_at: now,
                updated_at: now,
                metadata: session.metadata,
            };

            self.sessions.write().await.insert(id, db_session.clone());
            Ok(db_session)
        }
        async fn get_planning_session(
            &self,
            id: Uuid,
        ) -> Result<
            Option<crate::planning::data_infrastructure_types::models::PlanningSession>,
            anyhow::Error,
        > {
            Ok(self.sessions.read().await.get(&id).cloned())
        }
        async fn update_planning_session(
            &self,
            id: Uuid,
            session: crate::planning::data_infrastructure_types::UpdatePlanningSession,
        ) -> Result<(), anyhow::Error> {
            use chrono::Utc;
            let mut sessions = self.sessions.write().await;
            let db_session = sessions
                .get_mut(&id)
                .ok_or_else(|| anyhow::anyhow!("Planning session not found: {}", id))?;

            if let Some(status) = session.status {
                db_session.status = status;
            }
            if let Some(metadata) = session.metadata {
                db_session.metadata = metadata;
            }
            db_session.updated_at = Utc::now();

            Ok(())
        }
        async fn create_planning_telemetry(
            &self,
            _telemetry: crate::planning::data_infrastructure_types::CreatePlanningTelemetry,
        ) -> Result<
            crate::planning::data_infrastructure_types::models::PlanningTelemetry,
            anyhow::Error,
        > {
            Err(anyhow::anyhow!("Not implemented"))
        }
        async fn get_planning_telemetry(
            &self,
            _plan_id: Uuid,
            _metric_type: Option<String>,
        ) -> Result<
            Vec<crate::planning::data_infrastructure_types::models::PlanningTelemetry>,
            anyhow::Error,
        > {
            Ok(vec![])
        }
        async fn create_planning_audit_event(
            &self,
            _event: crate::planning::data_infrastructure_types::CreatePlanningAuditEvent,
        ) -> Result<(), anyhow::Error> {
            Ok(())
        }
        async fn get_planning_audit_events(
            &self,
            _plan_id: Uuid,
        ) -> Result<
            Vec<crate::planning::data_infrastructure_types::models::PlanningAuditEvent>,
            anyhow::Error,
        > {
            Ok(vec![])
        }
        async fn delete_execution_plan(&self, _id: Uuid) -> Result<(), anyhow::Error> {
            Ok(())
        }
        async fn create_judge(
            &self,
            _judge: crate::planning::data_infrastructure_types::CreateJudge,
        ) -> Result<crate::planning::data_infrastructure_types::models::Judge, anyhow::Error>
        {
            Err(anyhow::anyhow!("Not implemented"))
        }
        async fn get_judge(
            &self,
            _id: Uuid,
        ) -> Result<Option<crate::planning::data_infrastructure_types::models::Judge>, anyhow::Error>
        {
            Ok(None)
        }
        async fn get_judges(
            &self,
        ) -> Result<Vec<crate::planning::data_infrastructure_types::models::Judge>, anyhow::Error>
        {
            Ok(vec![])
        }
        async fn create_judge_evaluation(
            &self,
            _evaluation: crate::planning::data_infrastructure_types::CreateJudgeEvaluation,
        ) -> Result<
            crate::planning::data_infrastructure_types::models::JudgeEvaluation,
            anyhow::Error,
        > {
            Err(anyhow::anyhow!("Not implemented"))
        }
        async fn get_judge_evaluations(
            &self,
            _task_id: Uuid,
        ) -> Result<
            Vec<crate::planning::data_infrastructure_types::models::JudgeEvaluation>,
            anyhow::Error,
        > {
            Ok(vec![])
        }
        async fn get_workers(
            &self,
        ) -> Result<Vec<crate::planning::data_infrastructure_types::models::Worker>, anyhow::Error>
        {
            Ok(vec![])
        }
        async fn get_worker(
            &self,
            _id: Uuid,
        ) -> Result<Option<crate::planning::data_infrastructure_types::models::Worker>, anyhow::Error>
        {
            Ok(None)
        }
        async fn create_worker(
            &self,
            _worker: crate::planning::data_infrastructure_types::CreateWorker,
        ) -> Result<crate::planning::data_infrastructure_types::models::Worker, anyhow::Error>
        {
            Err(anyhow::anyhow!("Not implemented"))
        }
        async fn update_worker(
            &self,
            _id: Uuid,
            _update: crate::planning::data_infrastructure_types::UpdateWorker,
        ) -> Result<crate::planning::data_infrastructure_types::models::Worker, anyhow::Error>
        {
            Err(anyhow::anyhow!("Not implemented"))
        }
        async fn create_execution_result(
            &self,
            result: crate::planning::data_infrastructure_types::CreateExecutionResult,
        ) -> Result<
            crate::planning::data_infrastructure_types::models::PlanExecutionResult,
            anyhow::Error,
        > {
            use chrono::Utc;
            let now = Utc::now();
            let db_result =
                crate::planning::data_infrastructure_types::models::PlanExecutionResult {
                    plan_id: result.plan_id,
                    success: result.success,
                    milestones_completed: result.milestones_completed as i32,
                    total_duration_ms: result.total_duration_ms as i64,
                    evidence: result.evidence,
                    metrics: result.metrics,
                    final_state: result.final_state,
                    timeline: result.timeline,
                    created_at: now,
                    updated_at: now,
                };

            self.execution_results
                .write()
                .await
                .insert(result.plan_id, db_result.clone());
            Ok(db_result)
        }
        async fn get_execution_result(
            &self,
            plan_id: Uuid,
        ) -> Result<
            Option<crate::planning::data_infrastructure_types::models::PlanExecutionResult>,
            anyhow::Error,
        > {
            Ok(self.execution_results.read().await.get(&plan_id).cloned())
        }
        async fn get_waivers(
            &self,
            _status: Option<String>,
        ) -> Result<Vec<crate::planning::data_infrastructure_types::models::Waiver>, anyhow::Error>
        {
            Ok(vec![])
        }
        async fn create_waiver(
            &self,
            waiver: crate::planning::data_infrastructure_types::CreateWaiver,
        ) -> Result<crate::planning::data_infrastructure_types::models::Waiver, anyhow::Error>
        {
            use chrono::Utc;
            use std::collections::HashMap;
            let id = Uuid::new_v4();
            let now = Utc::now();
            let db_waiver = crate::planning::data_infrastructure_types::models::Waiver {
                id,
                plan_id: waiver.plan_id,
                waiver_type: "test".to_string(),
                reason: waiver.reason,
                approved_by: "test".to_string(),
                status: "pending".to_string(),
                gates: waiver.waived_gates,
                impact_level: "low".to_string(),
                mitigation_plan: None,
                created_at: now,
                expires_at: None,
                metadata: HashMap::new(),
            };

            Ok(db_waiver)
        }
        async fn update_waiver(
            &self,
            _id: Uuid,
            _update: crate::planning::data_infrastructure_types::UpdateWaiver,
        ) -> Result<crate::planning::data_infrastructure_types::models::Waiver, anyhow::Error>
        {
            Err(anyhow::anyhow!("Not implemented"))
        }
        async fn create_council_session(
            &self,
            _session: crate::planning::data_infrastructure_types::CreateCouncilSession,
        ) -> Result<crate::planning::data_infrastructure_types::models::CouncilSession, anyhow::Error>
        {
            use chrono::Utc;
            let id = Uuid::new_v4();
            let now = Utc::now();
            Ok(
                crate::planning::data_infrastructure_types::models::CouncilSession {
                    id,
                    session_id: _session.session_id,
                    task_id: _session.task_id,
                    working_spec_id: _session.working_spec_id,
                    review_context: _session.review_context,
                    status: _session.status.unwrap_or_else(|| "initialized".to_string()),
                    selected_judges: _session
                        .selected_judges
                        .unwrap_or_else(|| serde_json::json!([])),
                    contributions: _session
                        .contributions
                        .unwrap_or_else(|| serde_json::json!([])),
                    aggregation_result: None,
                    final_decision: None,
                    progress: _session.progress.unwrap_or(0.0),
                    started_at: now,
                    completed_at: None,
                    created_at: now,
                    updated_at: now,
                    metadata: _session.metadata.unwrap_or_else(|| serde_json::json!({})),
                },
            )
        }
        async fn get_council_session(
            &self,
            _session_id: Uuid,
        ) -> Result<
            Option<crate::planning::data_infrastructure_types::models::CouncilSession>,
            anyhow::Error,
        > {
            Ok(None)
        }
        async fn get_council_session_by_task(
            &self,
            _task_id: Uuid,
        ) -> Result<
            Option<crate::planning::data_infrastructure_types::models::CouncilSession>,
            anyhow::Error,
        > {
            Ok(None)
        }
        async fn update_council_session(
            &self,
            _session_id: Uuid,
            _update: crate::planning::data_infrastructure_types::UpdateCouncilSession,
        ) -> Result<crate::planning::data_infrastructure_types::models::CouncilSession, anyhow::Error>
        {
            Err(anyhow::anyhow!("Not implemented"))
        }
    }
}

// Modules moved to other crates during V3 architecture refactor:
// - models -> system-federated-ml
// - resilience -> system-quality-security
// - claim_extraction_multimodal -> system-federated-ml
// - learning -> agent-research (reflexive learning)
// - model_client -> agent-model-management
// - advanced_monitoring -> system-quality-security
// - intelligent_testing -> testing-validation
// - predictive_learning -> system-federated-ml
// - council_types, debate_types, debate, plan_review -> agent-orchestration/planning
// - todo_analyzer -> development-tools
// - semantic -> system-federated-ml
// - learning_types, learning_storage -> agent-research

// Judge submodules
pub mod judge_backup;
// pub mod judge_types;
// pub mod judge_cache;
// pub mod mistral_tokenizer;
// pub mod mistral_judge_integration_test;
// pub mod mistral_integration_demo;

// Advanced arbitration and testing
// pub mod advanced_arbitration;
// pub mod advanced_arbitration_tests;
// pub mod intelligent_edge_case_testing_tests;

// Predictive learning
// pub mod predictive_learning_system_tests;
// pub mod predictive_quality_assessor;

// ============================================================================
// ORCHESTRATION MODULES (Execution & Coordination)
// ============================================================================

// Restored orchestration modules
pub mod adapter;
pub mod frontier;
pub mod types;
// pub mod task_api;
// pub mod arbiter;
// pub mod cqrs_router;
// pub mod artifacts;
pub mod autonomous_file_editor;
pub mod autonomous_integration;
// pub mod caws_runtime;
// pub mod cqrs;
// pub mod db;
// pub mod orchestrate;
// pub mod orchestration_core;
// pub mod persistence;
// pub mod persistence_postgres;
// pub mod planning;
// pub mod provenance;
// pub mod quality;
// pub mod worker_registry;
// pub mod refinement;
// pub mod tracking;
pub mod coreml;
pub mod multimodal_orchestration;
// pub mod enrichers;
pub mod audit_trail;
pub mod chain_of_thought;
pub mod optimization;
pub mod orchestration;
pub mod workers;

// Evaluation framework (feature-gated)
#[cfg(feature = "evaluation")]
pub mod evaluation;

// Playground module always available for tests (independent of evaluation feature)
#[cfg(test)]
mod playground {
    // Re-export playground module if evaluation feature is enabled
    #[cfg(feature = "evaluation")]
    #[allow(ambiguous_glob_reexports)]
    pub use crate::evaluation::playground::*;
}

#[cfg(feature = "evaluation")]
pub use evaluation::{
    run_scenario, AgentEvaluation, EvaluationDimensions, EvaluationEngine, EvaluationReport,
    EvaluationScenario,
};

// ============================================================================
// RE-EXPORTS - Council (Decision Making)
// ============================================================================

pub use council::{Council, CouncilConfig, CouncilSession};
pub use council_errors::{CouncilError, CouncilResult};
pub use decision_making::{ConsensusStrategy, DecisionEngine, RiskThresholds};
pub use judge_backup::{
    // Ethical analysis types
    risk::{
        ConsequenceAssessment, EthicalAssessment, EthicalCategory, EthicalConcern, EthicalSeverity,
        EthicalTradeoff, StakeholderImpact,
    },
    // Ethics judge
    EthicsJudge,
    Judge,
    JudgeContribution,
    JudgeVerdict,
    // Mock judge
    MockJudge,
};
pub use verdict_aggregation::{AggregationResult, CouncilDecision, VerdictAggregator};
pub use workflow::{CouncilWorkflow, WorkflowState};
// pub use risk_scorer::{RiskScorer, TechnicalRiskWeights, EthicalRiskWeights, OperationalRiskWeights, BusinessRiskWeights, DimensionWeights}; // TEMPORARILY DISABLED
pub use error_handling::{
    error_factory, with_retry, AgencyError, CircuitBreaker, CircuitBreakerState,
    CircuitBreakerStats, DegradationLevel, DegradationManager, DegradationPolicy, DegradationState,
    ErrorCategory, ErrorHandlingCircuitBreakerConfig, ErrorHandlingRetryConfig, ErrorSeverity,
    HealthStatus, RecoveryOrchestrator, RecoveryStrategy, RecoveryStrategyType, SystemHealth,
};
// Council types including ConsensusResult
pub use council_types::{ChangeBudget, ConsensusResult, FinalVerdict, Task};
// Items from restored modules are available through their module declarations above

// Frontier items are available through the module declaration above

// NOTE: These modules were planned but not yet implemented. They can be added when needed:
// - resilience: ResilienceManager for fault tolerance
// - claim_extraction_multimodal: MultimodalEvidenceEnricher for evidence processing
// - advanced_monitoring: SLO tracking and alerting
// - verdict: VerdictStore for verdict persistence
// - coordinator::orchestrator: ProvenanceEmitter for audit trails
// These are non-blocking as the core orchestration functionality works without them.

// ============================================================================
// RE-EXPORTS - Orchestration (Execution)
// ============================================================================

// Multimodal orchestration exports
pub use multimodal_orchestration::{
    MultimodalOrchestrator, ProcessingResult, ProcessingStats, ProcessingStatus,
};

// Autonomous file editor exports
pub use autonomous_file_editor::{
    AutonomousFileEditError, AutonomousFileEditor, ChangeType, ChangesetPreview, FileChange,
    RiskAssessment, RiskLevel,
};

// Autonomous integration exports
pub use autonomous_integration::{
    AutonomousAgentIntegration, AutonomousExecutionResult, AutonomousHealthStatus,
    AutonomousIntegrationError,
};

// Audit trail exports
pub use audit_trail::{
    AgentThinkingAuditor, AuditCategory, AuditConfig, AuditError, AuditEvent, AuditLogLevel,
    AuditOutputFormat, AuditPerformance, AuditQuery, AuditResult, AuditSeverity, AuditTrailManager,
    CouncilAuditor, ErrorRecoveryAuditor, FileOperationsAuditor, LearningAuditor,
    PerformanceAuditor, TerminalAuditor,
};

// Restored frontier exports (now available)
pub use frontier::{Frontier, FrontierConfig, FrontierStats, TaskEntry, TaskStatus};

// NOTE: Arbiter module is planned for multi-agent debate resolution.
// Currently, council-based consensus is used instead. Arbiter can be added when needed.

pub use types::{DiffStats, MultimodalProcessingResult, MultimodalTask, OrchestratorConfig};

// ============================================================================
// CONDITIONAL EXPORTS - API Server
// ============================================================================

// NOTE: Task API and CQRS router are handled by data-infrastructure crate.
// Re-exports are not needed here as API handlers are in data-infrastructure/src/api/handlers/.

// ============================================================================
// MAIN ORCHESTRATION SERVICE
// ============================================================================

/// Main Agent Orchestration Service
///
/// Unified service that combines orchestration execution capabilities
/// with council decision-making and arbitration systems.
#[cfg(feature = "api-server")]
#[derive(Debug)]
struct AgentOrchestrationService {
    /// Council for task review and approval
    #[cfg(feature = "api-server")]
    council: std::sync::Arc<council::Council>,
    /// Orchestrator for task execution
    #[cfg(feature = "api-server")]
    orchestrator: std::sync::Arc<multimodal_orchestration::MultimodalOrchestrator>,
    /// Audit trail manager for tracking all operations
    pub audit_trail: audit_trail::AuditTrailManager,
}

#[cfg(feature = "api-server")]
impl AgentOrchestrationService {
    /// Create a new Agent Orchestration Service
    pub async fn new(config: OrchestrationConfig) -> Result<Self, OrchestrationError> {
        // Initialize council with default components
        // Custom judges can be registered after creation via council's judge registration API
        let available_judges: Vec<Arc<dyn crate::judge_backup::Judge>> = vec![];
        let verdict_aggregator = Arc::new(crate::verdict_aggregation::create_verdict_aggregator());
        let decision_engine = crate::decision_making::create_decision_engine();

        let council = std::sync::Arc::new(council::Council::new(
            config.council_config,
            available_judges,
            verdict_aggregator,
            decision_engine,
        ));

        let orchestrator = std::sync::Arc::new(multimodal_orchestration::MultimodalOrchestrator::new()
            .await
            .map_err(|e| OrchestrationError::AnyhowError(e))?);

        let audit_trail = audit_trail::AuditTrailManager::new(config.audit_config);

        Ok(Self {
            council,
            orchestrator,
            audit_trail,
        })
    }

    /// Execute a task with full orchestration and governance
    ///
    /// This method coordinates the complete lifecycle:
    /// 1. Council review and approval
    /// 2. Task orchestration and execution
    /// 3. Audit trail recording
    /// 4. Quality assurance and monitoring
    #[cfg(feature = "api-server")]
    pub async fn execute_orchestrated_task(
        &self,
        task: OrchestratedTask,
    ) -> Result<OrchestrationResult, OrchestrationError> {
        // Convert OrchestratedTask to TaskDescriptor for council review
        let task_descriptor = self.to_task_descriptor(&task);

        // 1. Council review and approval
        // Convert TaskDescriptor to WorkingSpec for council review
        // (Duplicating logic from Council::convert_task_to_working_spec since it's private)
        use agent_agency_contracts::task_request::Environment;
        use agent_agency_contracts::{RollbackPlan, TestPlan, WorkingSpec, WorkingSpecConstraints, WorkingSpecContext};
        
        let working_spec = WorkingSpec {
            id: task_descriptor.task_id.to_string(),
            title: format!("Task: {}", task_descriptor.task_id),
            description: task_descriptor.description.clone(),
            version: "1.0.0".to_string(),
            goals: vec![task_descriptor.description.clone()],
            acceptance_criteria: vec![],
            test_plan: TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: None,
            },
            rollback_plan: RollbackPlan {
                strategy: agent_agency_contracts::RollbackStrategy::GitRevert,
                automated_steps: vec!["git revert".to_string()],
                manual_steps: vec![],
                data_impact: agent_agency_contracts::DataImpact::None,
                downtime_required: Some(false),
                rollback_window_minutes: Some(30),
            },
            risk_tier: match task_descriptor.priority {
                agent_agency_contracts::types::planning::TaskPriority::Critical
                | agent_agency_contracts::types::planning::TaskPriority::Urgent => 1,
                agent_agency_contracts::types::planning::TaskPriority::High => 1,
                agent_agency_contracts::types::planning::TaskPriority::Normal
                | agent_agency_contracts::types::planning::TaskPriority::Medium => 2,
                agent_agency_contracts::types::planning::TaskPriority::Low => 3,
            },
            constraints: WorkingSpecConstraints {
                max_duration_minutes: None,
                max_iterations: None,
                budget_limits: None,
                scope_restrictions: None,
            },
            context: WorkingSpecContext {
                workspace_root: ".".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: Environment::Development,
            },
            change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                max_files: 100,
                max_loc: 2000,
                max_migrations: 10,
                allow_breaking_changes: false,
                allow_new_dependencies: true,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
            },
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            coverage_targets: None,
            file_changes: vec![],
            milestones: vec![],
            quality_gates: None,
            scope: vec![],
            overview: String::new(),
            metadata: None,
            non_functional_requirements: None,
            validation_results: None,
        };

        // Create review context for full review
        let review_context = crate::judge_backup::types::ReviewContext {
            session_id: format!("council_session_{}", uuid::Uuid::new_v4()),
            working_spec: serde_json::to_string(&working_spec)
                .map_err(|e| OrchestrationError::CouncilError(
                    crate::council_errors::CouncilError::InvalidInput {
                        message: format!("Failed to serialize working spec: {}", e),
                    }
                ))?,
            risk_tier: match task_descriptor.priority {
                agent_agency_contracts::types::planning::TaskPriority::Critical
                | agent_agency_contracts::types::planning::TaskPriority::High => 1,
                agent_agency_contracts::types::planning::TaskPriority::Normal
                | agent_agency_contracts::types::planning::TaskPriority::Medium => 2,
                agent_agency_contracts::types::planning::TaskPriority::Low => 3,
                agent_agency_contracts::types::planning::TaskPriority::Urgent => 1,
            },
            previous_reviews: Vec::new(),
            constraints: std::collections::HashMap::new(),
            review_type: crate::judge_backup::types::ReviewType::PlanReview,
        };

        // Perform full council review using conduct_review
        let reviewed_session = self
            .council
            .conduct_review(working_spec, review_context)
            .await
            .map_err(|e| OrchestrationError::CouncilError(e))?;

        // Extract consensus result from final decision
        let consensus_result = if let Some(ref decision) = reviewed_session.final_decision {
            match decision {
                crate::decision_making::FinalDecision::Proceed { confidence, .. } => {
                    crate::council_types::ConsensusResult {
                        approved: true,
                        confidence: *confidence,
                        reason: format!(
                            "Task approved by council with {:.1}% confidence",
                            confidence * 100.0
                        ),
                    }
                }
                crate::decision_making::FinalDecision::Refine {
                    refinement_directive,
                    ..
                } => {
                    crate::council_types::ConsensusResult {
                        approved: false,
                        confidence: 0.5,
                        reason: format!(
                            "Task requires refinement: {} changes required",
                            refinement_directive.required_changes.len()
                        ),
                    }
                }
                crate::decision_making::FinalDecision::Reject { reason, .. } => {
                    crate::council_types::ConsensusResult {
                        approved: false,
                        confidence: 0.2,
                        reason: reason.clone(),
                    }
                }
                crate::decision_making::FinalDecision::Escalate { reason, .. } => {
                    crate::council_types::ConsensusResult {
                        approved: false,
                        confidence: 0.3,
                        reason: reason.clone(),
                    }
                }
            }
        } else {
            return Err(OrchestrationError::CouncilError(
                crate::council_errors::CouncilError::InvalidInput {
                    message: "Council session completed without final decision".to_string(),
                },
            ));
        };

        if !consensus_result.approved {
            return Err(OrchestrationError::CouncilRejection(
                consensus_result.reason,
            ));
        }

        // Convert ConsensusResult to CouncilDecision for result
        let council_decision = self.convert_consensus_to_decision(&consensus_result);

        // 2. Execute task using planning orchestrator
        let execution_result = self
            .orchestrator
            .execute_planning_with_audit(
                &task.description,
                None, // No additional context
            )
            .await
            .map_err(|e| OrchestrationError::ExecutionError(Box::new(e)))?;

        // 3. Create TaskExecutionResult for audit trail (contract type - artifacts stored separately)
        use chrono::Utc;
        use uuid::Uuid;
        let execution_id = Uuid::new_v4();
        let task_execution_result = TaskExecutionResult {
            execution_id,
            task_id: Uuid::parse_str(&task.id).unwrap_or_else(|_| Uuid::new_v4()),
            success: matches!(execution_result.status, multimodal_orchestration::ProcessingStatus::Completed),
            output: format!(
                "Multimodal processing: {} blocks processed",
                execution_result.blocks_processed
            ),
            errors: match execution_result.status {
                multimodal_orchestration::ProcessingStatus::Completed => vec![],
                _ => vec!["Processing failed".to_string()],
            },
            metadata: std::collections::HashMap::new(),
            started_at: Utc::now(),
            completed_at: Utc::now(),
            duration_ms: 0,
            worker_id: None,
        };

        // 4. Record audit trail
        self.audit_trail
            .record_execution(&task_execution_result)
            .await
            .map_err(|e| OrchestrationError::AuditError(e.to_string()))?;

        // 5. Return comprehensive result
        Ok(OrchestrationResult {
            council_decision,
            execution_result,
            audit_id: Some(task.id.clone()), // Use task ID as audit correlation ID
        })
    }

    /// Convert OrchestratedTask to TaskDescriptor
    #[cfg(feature = "api-server")]
    fn to_task_descriptor(
        &self,
        task: &OrchestratedTask,
    ) -> agent_agency_contracts::TaskDescriptor {
        agent_agency_contracts::TaskDescriptor {
            task_id: Uuid::parse_str(&task.id).unwrap_or_else(|_| Uuid::new_v4()),
            description: task.description.clone(),
            scope_in: agent_agency_contracts::ScopeRestrictions {
                allowed_paths: vec![],
                blocked_paths: vec![],
            },
            scope_out: Some(agent_agency_contracts::ScopeRestrictions {
                allowed_paths: vec![],
                blocked_paths: vec![],
            }),
            change_budget: agent_agency_contracts::ChangeBudget {
                max_files: 25,
                max_loc: 1000,
                max_migrations: 0,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
            },
            blast_radius: agent_agency_contracts::BlastRadius {
                modules: vec![],
                data_migration: false,
                external_deps: vec![],
            },
            priority: task.priority.clone(),
            execution_mode: agent_agency_contracts::ExecutionMode::Auto,
            risk_tier: Some(match task.priority {
                TaskPriority::Critical | TaskPriority::High | TaskPriority::Urgent => {
                    agent_agency_contracts::task_request::RiskTier::Tier1
                }
                TaskPriority::Normal | TaskPriority::Medium => {
                    agent_agency_contracts::RiskTier::Tier2
                }
                TaskPriority::Low => agent_agency_contracts::RiskTier::Tier3,
            }),
            acceptance: Some("Orchestrated task".to_string()),
        }
    }

    /// Convert ConsensusResult to CouncilDecision
    fn convert_consensus_to_decision(
        &self,
        consensus: &crate::council_types::ConsensusResult,
    ) -> verdict_aggregation::CouncilDecision {
        if consensus.approved {
            verdict_aggregation::CouncilDecision::Approve {
                confidence: consensus.confidence as f64,
                quality_score: consensus.confidence as f64,
                risk_assessment: verdict_aggregation::AggregatedRiskAssessment {
                    overall_risk: crate::judge_backup::risk::RiskLevel::Low,
                    risk_factors: vec![],
                    mitigation_suggestions: vec![],
                    confidence: consensus.confidence as f64,
                },
            }
        } else {
            verdict_aggregation::CouncilDecision::Reject {
                confidence: consensus.confidence as f64,
                critical_issues: vec![crate::judge_backup::CriticalIssue {
                    severity: crate::judge_backup::IssueSeverity::Critical,
                    category: "Council Rejection".to_string(),
                    description: consensus.reason.clone(),
                    evidence: vec![],
                }],
                alternative_approaches: vec![],
            }
        }
    }
}

/// Configuration for the Agent Orchestration Service
#[cfg(feature = "api-server")]
#[derive(Debug, Clone)]
struct OrchestrationConfig {
    pub council_config: council::CouncilConfig,
    pub orchestrator_config: crate::types::OrchestratorConfig,
    pub audit_config: audit_trail::AuditConfig,
}

/// Orchestrated task input
#[cfg(feature = "api-server")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct OrchestratedTask {
    pub id: String,
    pub description: String,
    pub requirements: Vec<String>,
    pub priority: TaskPriority,
}

/// Orchestration execution result
#[cfg(feature = "api-server")]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct OrchestrationResult {
    pub council_decision: verdict_aggregation::CouncilDecision,
    pub execution_result: multimodal_orchestration::ProcessingResult,
    pub audit_id: Option<String>,
}

/// Unified orchestration error type

#[derive(Debug, thiserror::Error)]
enum OrchestrationError {
    #[error("Council rejected task: {0}")]
    CouncilRejection(String),

    #[error("Orchestration execution failed: {0}")]
    ExecutionError(Box<dyn std::error::Error + Send + Sync>),

    #[error("Council error: {0}")]
    CouncilError(#[from] council_errors::CouncilError),

    #[error("Circuit breaker error: {0}")]
    CircuitBreakerError(String),

    #[error("Audit error: {0}")]
    AuditError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}
