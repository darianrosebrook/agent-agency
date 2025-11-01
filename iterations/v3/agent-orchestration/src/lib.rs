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
use crate::autonomous_executor::OrchestrationProvenanceEmitter;

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

mod progress_tracker;
mod consensus_coordinator;

// Re-export types for convenience - use contracts types
pub use agent_agency_contracts::types::prelude::{
    TaskDescriptor, ExecutionMode, BlastRadius, ExecutionContext,
    TaskPriority, RiskTier, AcceptanceCriterion
};
// Re-export contracts WorkingSpec (local WorkingSpec is deprecated - use contracts)
pub use agent_agency_contracts::WorkingSpec;
// Keep orchestration-specific local types that don't exist in contracts
// Note: OrchestratorConfig is also in adapter.rs, so we only export from types.rs here
pub use crate::types::{TaskExecutionResult, ExecutionArtifacts, QualityReport};

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
pub mod error_handling;
pub mod council_errors;
pub mod council_types;
pub mod evidence_enrichment;
pub mod planning;
// pub mod coordinator;

// Test module for restored functionality
#[cfg(test)]
mod restored_tests;

// Examples module for restored functionality
pub mod restored_examples;

// TODO: These modules were moved during refactor - need to locate or recreate
// pub mod models;
// pub mod resilience;
// pub mod claim_extraction_multimodal;
// pub mod learning;
// pub mod model_client;
// pub mod advanced_monitoring;
// pub mod intelligent_testing;
// pub mod predictive_learning;
// pub mod council_errors; // Duplicate - already declared above
// pub mod council_types;
// pub mod debate_types;
// pub mod debate;
// pub mod plan_review;
// pub mod todo_analyzer;
// pub mod semantic;
// pub mod learning_types;
// pub mod learning_storage;
// pub mod contracts;

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
pub mod types;
pub mod frontier;
// pub mod task_api;
// pub mod arbiter;
// pub mod cqrs_router;
// pub mod artifacts;
pub mod autonomous_executor;
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
pub mod multimodal_orchestration;
pub mod coreml;
// pub mod enrichers;
pub mod audit_trail;
// audited_orchestrator module removed - functionality integrated into multimodal_orchestration
// pub mod enhanced_executor;
pub mod multimodal_orchestrator;

// ============================================================================
// RE-EXPORTS - Council (Decision Making)
// ============================================================================

pub use council_errors::{CouncilError, CouncilResult};
pub use judge_backup::{
    Judge, JudgeVerdict, JudgeContribution,
    // Ethical analysis types
    risk::{EthicalAssessment, EthicalConcern, EthicalCategory, EthicalSeverity,
           StakeholderImpact, EthicalTradeoff, ConsequenceAssessment},
    // Ethics judge
    EthicsJudge,
    // Mock judge
    MockJudge,
};
pub use council::{Council, CouncilConfig, CouncilSession};
pub use decision_making::{DecisionEngine, ConsensusStrategy, RiskThresholds};
pub use verdict_aggregation::{VerdictAggregator, AggregationResult, CouncilDecision};
pub use workflow::{CouncilWorkflow, WorkflowState};
// pub use risk_scorer::{RiskScorer, TechnicalRiskWeights, EthicalRiskWeights, OperationalRiskWeights, BusinessRiskWeights, DimensionWeights}; // TEMPORARILY DISABLED
pub use error_handling::{
    AgencyError, ErrorCategory, ErrorSeverity, RecoveryStrategy, RecoveryStrategyType,
    CircuitBreaker, ErrorHandlingCircuitBreakerConfig, CircuitBreakerStats, CircuitBreakerState,
    ErrorHandlingRetryConfig, with_retry, DegradationManager, DegradationState, DegradationPolicy,
    DegradationLevel, RecoveryOrchestrator, SystemHealth, HealthStatus,
    error_factory,
};
// Items from restored modules are available through their module declarations above

// Frontier items are available through the module declaration above

// TODO: These re-exports reference missing modules
// pub use resilience::ResilienceManager;
// pub use claim_extraction_multimodal::{MultimodalEvidenceEnricher, ClaimWithMultimodalEvidence};
// pub use advanced_monitoring::{SLOTracker, SLOStatus, SLOAlert, AlertLevel, SLOComponent, SLODashboardSummary};
// pub use verdict::{VerdictStore, VerdictRecord, VerdictStorage, CacheConfig, StorageStats, CacheStats, VerdictStoreStats};
// pub use coordinator::orchestrator::{ConsensusCoordinator, ProvenanceEmitter};

// ============================================================================
// RE-EXPORTS - Orchestration (Execution)
// ============================================================================

// Multimodal orchestration exports
pub use multimodal_orchestration::{
    MultimodalOrchestrator, ProcessingResult, ProcessingStatus, ProcessingStats,
};

// Autonomous executor exports
pub use autonomous_executor::{
    AutonomousExecutor, AutonomousExecutorConfig, TaskExecutionState,
};

// Autonomous file editor exports
pub use autonomous_file_editor::{
    AutonomousFileEditor, FileChange, ChangeType, RiskAssessment, RiskLevel,
    ChangesetPreview, AutonomousFileEditError,
};

// Autonomous integration exports
pub use autonomous_integration::{
    AutonomousAgentIntegration, AutonomousExecutionResult, AutonomousHealthStatus,
    AutonomousIntegrationError,
};

// Audit trail exports
pub use audit_trail::{
    AuditTrailManager, AuditConfig, AuditLogLevel, AuditOutputFormat,
    FileOperationsAuditor, TerminalAuditor, CouncilAuditor, AgentThinkingAuditor,
    PerformanceAuditor, ErrorRecoveryAuditor, LearningAuditor,
    AuditEvent, AuditCategory, AuditSeverity, AuditResult, AuditPerformance,
    AuditQuery, AuditError,
};

// Audited orchestrator functionality integrated into MultimodalOrchestrator

// Restored frontier exports (now available)
pub use frontier::{
    Frontier, FrontierConfig, FrontierStats, TaskEntry, TaskStatus,
};

// TODO: These re-exports reference missing modules
// Arbiter exports
// pub use arbiter::{
//     ArbiterOrchestrator, ArbiterConfig, ArbiterVerdict, VerdictStatus,
//     WorkerOutput, EvidenceManifest, DebateResult, ArbiterError,
// };

// Multimodal Orchestrator
pub use multimodal_orchestrator::{
    KimiK2MultimodalOrchestrator, OrchestratorPerformanceStats, OrchestratorError,
};
pub use types::{MultimodalTask, MultimodalProcessingResult};
pub use multimodal_orchestration::OrchestratorConfig;

// Council types
pub use council_types::{FinalVerdict, Task, ChangeBudget};
// BlastRadius is now from agent_agency_contracts::types::planning (exported above)
// ExecutionMode is now exported from agent_agency_contracts::types::prelude above
pub use types::DiffStats;

// ============================================================================
// CONDITIONAL EXPORTS - API Server
// ============================================================================

#[cfg(feature = "api-server")]
// TODO: These re-exports reference missing modules
// Re-export API functions
// pub use task_api::{
//     get_tasks, get_task_detail, get_task_events, cancel_task,
//     TaskResponse, TaskDetail, TaskEvent, TaskApiError,
// };

#[cfg(feature = "api-server")]
// TODO: These re-exports reference missing modules
// Re-export CQRS router functions
// pub use cqrs_router::{
//     create_cqrs_router, create_legacy_router, create_combined_router,
// };

// ============================================================================
// MAIN ORCHESTRATION SERVICE
// ============================================================================

/// Main Agent Orchestration Service
///
/// Unified service that combines orchestration execution capabilities
/// with council decision-making and arbitration systems.
// #[derive(Debug)]
pub struct AgentOrchestrationService {
    /// Council for decision making and arbitration
    // pub council: council::Council,
    /// Multimodal orchestrator for task execution
    // pub orchestrator: multimodal_orchestration::MultimodalOrchestrator,
    /// Autonomous executor for self-directed task execution
    // pub autonomous_executor: autonomous_executor::AutonomousExecutor,
    /// Audit trail manager for tracking all operations
    pub audit_trail: audit_trail::AuditTrailManager,
}

impl AgentOrchestrationService {
    /// Create a new Agent Orchestration Service
    pub async fn new(config: OrchestrationConfig) -> Result<Self, OrchestrationError> {
        // Create basic council components - TODO: make configurable
        let available_judges: Vec<Arc<dyn crate::judge_backup::Judge>> = vec![]; // Empty for now
        let verdict_aggregator = Arc::new(crate::verdict_aggregation::create_verdict_aggregator());
        let decision_engine = crate::decision_making::create_decision_engine();

        let council = council::Council::new(
            config.council_config,
            available_judges,
            verdict_aggregator,
            decision_engine
        );
        
        let orchestrator = multimodal_orchestration::MultimodalOrchestrator::new().await
            .map_err(|e| OrchestrationError::ExecutionError(Box::new(e)))?;
        
        let executor_config = config.executor_config;
        let autonomous_executor = autonomous_executor::AutonomousExecutor::new(
            executor_config,
            None, // progress_tracker
            Arc::new(crate::progress_tracker::RealTimeProgressTracker::new(None)), // runtime_validator - TODO: proper implementation
            None, // consensus_coordinator
            None, // verdict_writer - TODO: proper implementation
            Arc::new(OrchestrationProvenanceEmitter::new()), // provenance_emitter - TODO: proper implementation
            None, // cache
            None, // metrics
            {
                // Create a simple factory function for TaskExecutor
                let factory = || -> Arc<dyn agent_agency_contracts::TaskExecutor> {
                    // PLACEHOLDER: Real TaskExecutor implementation needed
                    panic!("TaskExecutor factory not implemented - requires agent-workers integration")
                };
                agent_agency_contracts::task_executor_provider::TaskExecutorProvider::new(factory)
            }, // task_executor_provider - TODO: proper implementation
            #[cfg(feature = "memory")]
            None, // memory_system
            None, // planning_integration
        );
        
        let audit_trail = audit_trail::AuditTrailManager::new(config.audit_config);

        Ok(Self {
            // council,
            // orchestrator,
            // autonomous_executor,
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
    pub async fn execute_orchestrated_task(
        &self,
        task: OrchestratedTask,
    ) -> Result<OrchestrationResult, OrchestrationError> {
        // TEMPORARILY DISABLED - struct fields commented out
        todo!("Re-enable when struct fields are restored");
        // Convert OrchestratedTask to TaskDescriptor for council review
        let task_descriptor = self.to_task_descriptor(&task);
        
        // 1. Council review and approval
        let council_session = self.council.start_session(&task_descriptor).await
            .map_err(|e| OrchestrationError::CouncilError(e))?;
        
        let consensus_result = council_session.review_task(&task).await
            .map_err(|e| OrchestrationError::CouncilError(e))?;

        if !consensus_result.approved {
            return Err(OrchestrationError::CouncilRejection(consensus_result.reason));
        }

        // Convert ConsensusResult to CouncilDecision for result
        let council_decision = self.convert_consensus_to_decision(&consensus_result);

        // 2. Execute task using planning orchestrator
        let execution_result = self.orchestrator.execute_planning_with_audit(
            &task.description,
            None, // No additional context
        ).await
            .map_err(|e| OrchestrationError::ExecutionError(Box::new(e)))?;

        // 3. Create TaskExecutionResult for audit trail
        let task_execution_result = crate::types::TaskExecutionResult {
            artifacts: crate::types::ExecutionArtifacts {
                execution_id: task.id.clone(),
                worker_id: "orchestrator".to_string(),
                output_files: vec![],
                diff_stats: crate::types::DiffStats::default(),
            },
            status: match execution_result.status {
                multimodal_orchestration::ProcessingStatus::Completed => crate::types::ExecutionStatus::Completed,
                multimodal_orchestration::ProcessingStatus::Failed => crate::types::ExecutionStatus::Failed,
                multimodal_orchestration::ProcessingStatus::InProgress => crate::types::ExecutionStatus::Running,
                _ => crate::types::ExecutionStatus::Pending,
            },
            quality_report: None, // Would be populated from execution analysis
        };

        // 4. Record audit trail
        self.audit_trail.record_execution(&task_execution_result).await
            .map_err(|e| OrchestrationError::AuditError(e.to_string()))?;

        // 5. Return comprehensive result
        Ok(OrchestrationResult {
            council_decision,
            execution_result,
            audit_id: Some(task.id.clone()), // Use task ID as audit correlation ID
        })
    }

    /// Convert OrchestratedTask to TaskDescriptor
    fn to_task_descriptor(&self, task: &OrchestratedTask) -> agent_agency_contracts::TaskDescriptor {
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
                TaskPriority::Critical | TaskPriority::High => agent_agency_contracts::RiskTier::Tier1,
                TaskPriority::Medium => agent_agency_contracts::RiskTier::Tier2,
                TaskPriority::Low => agent_agency_contracts::RiskTier::Tier3,
            }),
            acceptance: Some("Orchestrated task".to_string()),
        }
    }

    /// Convert ConsensusResult to CouncilDecision
    fn convert_consensus_to_decision(&self, consensus: &crate::autonomous_executor::ConsensusResult) -> verdict_aggregation::CouncilDecision {
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
#[derive(Debug, Clone)]
pub struct OrchestrationConfig {
    pub council_config: council::CouncilConfig,
    pub orchestrator_config: multimodal_orchestration::OrchestratorConfig,
    pub executor_config: autonomous_executor::AutonomousExecutorConfig,
    pub audit_config: audit_trail::AuditConfig,
}

/// Orchestrated task input
#[derive(Debug, Clone)]
pub struct OrchestratedTask {
    pub id: String,
    pub description: String,
    pub requirements: Vec<String>,
    pub priority: TaskPriority,
}


/// Orchestration execution result
#[derive(Debug, Clone)]
pub struct OrchestrationResult {
    pub council_decision: verdict_aggregation::CouncilDecision,
    pub execution_result: multimodal_orchestration::ProcessingResult,
    pub audit_id: Option<String>,
}

/// Unified orchestration error type
#[derive(Debug, thiserror::Error)]
pub enum OrchestrationError {
    #[error("Council rejected task: {0}")]
    CouncilRejection(String),

    #[error("Orchestration execution failed: {0}")]
    ExecutionError(#[from] Box<dyn std::error::Error + Send + Sync>),

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
