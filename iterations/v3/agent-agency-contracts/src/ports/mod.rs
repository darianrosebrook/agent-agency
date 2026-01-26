//! Ports for hexagonal architecture
//!
//! Ports define service boundaries and enable dependency injection.
//! Implementations live in consuming crates to break circular dependencies.

pub mod council_coordinator;
pub mod data_processing;
pub mod database_operations;
pub mod file_operations;
pub mod memory_system;
pub mod planning_engine;
pub mod research_evidence;
pub mod task_executor;
pub mod tool_chain;

// Re-export commonly used port traits and types
pub use database_operations::{
    // Core trait
    DatabaseOperationsPort,
    DatabaseError,
    // Execution Plan types
    CreateExecutionPlanRequest,
    UpdateExecutionPlanRequest,
    ExecutionPlanRecord,
    // Audit Entry types
    CreateAuditEntryRequest,
    AuditEntryRecord,
    // Planning Session types
    CreatePlanningSessionRequest,
    UpdatePlanningSessionRequest,
    PlanningSessionRecord,
    // Planning Telemetry types
    CreatePlanningTelemetryRequest,
    PlanningTelemetryRecord,
    // Planning Audit Event types
    CreatePlanningAuditEventRequest,
    PlanningAuditEventRecord,
    // Judge types
    CreateJudgeRequest,
    JudgeRecord,
    // Judge Evaluation types
    CreateJudgeEvaluationRequest,
    JudgeEvaluationRecord,
    // Worker types
    CreateWorkerRequest,
    UpdateWorkerRequest,
    WorkerRecord,
    // Waiver types
    CreateWaiverRequest,
    UpdateWaiverRequest,
    WaiverRecord,
    // Execution Result types
    CreateExecutionResultRequest,
    ExecutionResultRecord,
    // Council Session types
    CreateCouncilSessionRequest,
    UpdateCouncilSessionRequest,
    CouncilSessionRecord,
};

// Re-export task executor port types
pub use task_executor::{
    // Core trait
    TaskExecutorPort,
    TaskExecutionError,
    // Result types
    ExecutionResultWithObservability,
    TaskObservabilityData,
    // Observability types
    ChainOfThoughtEntry,
    CouncilDecisionData,
    JudgeContributionData,
    WorkerActionData,
    DecisionPointData,
    CoordinationEventData,
};

// Note: FileOperationsService is defined in system-common-interfaces
// to avoid circular dependencies. See file_operations.rs for documentation.
