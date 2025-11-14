//! Agent Agency V3 shared interoperability contracts.
//!
//! Provides strongly typed data contracts and JSON Schema backed validators
//! so workers, council, orchestration, and provenance components exchange
//! data safely with deterministic error handling.

/// API version constants for compatibility checking
pub const API_MAJOR: u32 = 1;
pub const API_MINOR: u32 = 3;

/// Returns the current API version as a string
pub const fn api_version() -> &'static str {
    const VERSION: &str = concat!(
        env!("CARGO_PKG_VERSION_MAJOR"),
        ".",
        env!("CARGO_PKG_VERSION_MINOR")
    );
    VERSION
}

pub mod contract_errors;
pub mod engine;
pub mod errors;
pub mod execution_artifacts;
pub mod execution_events;
pub mod final_verdict;
pub mod invariants;
pub mod judge_io;
pub mod judge_verdict;
pub mod milestone_contract;
pub mod planning;
pub mod planning_io;
pub mod ports;
pub mod quality_report;
pub mod refinement_decision;
pub mod router_decision;
mod schema;
pub mod task_executor;
pub mod task_executor_provider;
pub mod task_request;
pub mod task_response;
pub mod types;
pub mod worker_output;
pub mod worker_types;
pub mod working_spec;

pub use contract_errors::{ContractError, ContractKind, ValidationIssue};
pub use execution_artifacts::{
    validate_execution_artifacts_value, AuditEvent, CodeChanges, CoverageResults, DiffArtifact,
    DiffHunk, E2eScenarioResult, E2eTestResults, ExecutionArtifacts, ExecutionEnvironment,
    ExecutionSeeds, GitInfo, LintingIssue, LintingResults, NewFileArtifact, Provenance,
    TestArtifacts, TestFileInfo, TestFileStatus, TestFileType, TestResult, TestStatus,
    TestSuiteResults, UncoveredBranch, UncoveredLines,
};
pub use execution_events::ExecutionEvent;
pub use final_verdict::{
    validate_final_verdict_value, FinalDecision, FinalVerdictContract, VerificationSummary,
    VoteEntry, VoteVerdict,
};
pub use invariants::{
    run_caws_invariants, CAWSInvariant, InvariantCheck, InvariantResults, Severity,
    ViolationLocation,
};
pub use judge_verdict::{
    validate_judge_verdict_value, EvidenceItem as JudgeEvidenceItem,
    EvidenceType as JudgeEvidenceType, JudgeDecision, JudgeVerdictContract,
};
pub use quality_report::{
    validate_quality_report_value, GateIssue, GatePerformanceMetrics, GateResult, GateStatus,
    GateType, IssueSeverity, OverallStatus, QualityDeltas, QualityReport, QualityThresholds,
    Recommendation, RecommendationCategory, RecommendationPriority, ReportMetadata, ResourceUsage,
};
pub use refinement_decision::{
    validate_refinement_decision_value, ActionPriority, ActionType, BudgetIncrease,
    ConstraintAdjustments, CouncilDecision, CouncilVerdict, DecisionMetadata, DissentingOpinion,
    EffortLevel, EvidenceItem as RefinementEvidenceItem, EvidenceSeverity,
    EvidenceType as RefinementEvidenceType, FocusArea, JudgeContribution, JudgeType,
    QualityTargets, RefinementDecision, RefinementDirective, RiskAssessment, RiskFactor, RiskLevel,
    ScopeAdjustment, SpecificAction,
};
pub use router_decision::{
    validate_router_decision_value, Assignment, RouterDecisionContract, WorkerType,
};
pub use schema::{
    execution_artifacts_schema_source, final_verdict_schema_source, judge_verdict_schema_source,
    quality_report_schema_source, refinement_decision_schema_source, router_decision_schema_source,
    task_request_schema_source, task_response_schema_source, worker_output_schema_source,
    working_spec_schema_source,
};
pub use task_executor::{
    ExecutionStatus, Progress, TaskContext, TaskExecutionResult, TaskExecutor, TaskRequirements,
    TaskScope, TaskSpec,
};
pub use task_executor_provider::{TaskExecutorFactory, TaskExecutorProvider};
pub use task_request::{
    validate_task_request_value, BudgetLimits, ChangeType, Environment, FileChange, RiskTier,
    ScopeRestrictions, TaskConstraints, TaskContext as RequestTaskContext, TaskMetadata,
    TaskRequest,
};
pub use task_response::{
    validate_task_response_value, TaskError, TaskExecutionMetadata, TaskProgress, TaskResponse,
    TaskStatus, WorkingSpecSummary,
};
pub use worker_output::{
    validate_worker_output_value, CawsChecklist, ClaimContract, CommandArtifact, EvidenceReference,
    PatchArtifact, WaiverContract, WorkerArtifacts, WorkerMetadata, WorkerOutputContract,
    WorkerSeeds, WorkerSelfAssessment,
};
pub use worker_types::{
    SpecializedWorker, WorkerAssignment, WorkerContext, WorkerEventType, WorkerHealthMetrics,
    WorkerHealthStatus, WorkerPoolEvent, WorkerPoolStats, WorkerRegistration, WorkerResult,
    WorkerSpecialty, WorkerUpdate,
};
pub use working_spec::{
    validate_working_spec_value, AcceptanceCriterion, ChangeType as WorkingSpecChangeType,
    CoverageTargets, DataImpact, E2eScenario, FileChange as WorkingSpecFileChange,
    IntegrationTestSpec, MoSCoWPriority, NonFunctionalRequirements, PerformanceRequirements,
    RollbackPlan, RollbackStrategy, ScalabilityRequirements, TestPlan, UnitTestSpec,
    ValidationResults, WorkingSpec, WorkingSpecConstraints, WorkingSpecContext,
    WorkingSpecMetadata,
};

// Test utilities
pub use engine::{EngineCaps, EngineError, EngineRequest, EngineResponse, JudgeEngine, TokenUsage};
pub use errors::{
    ConfigError, ConfigResult, CouncilError, CouncilResult, DataProcessingError,
    DataProcessingResult, DatabaseError, MemoryError, OperationalError, OperationalResult,
    PlanningError, SecurityError, SecurityResult, ServiceError, ServiceResult, ValidationError,
};
pub use judge_io::{
    JudgePrompt, JudgeVerdict, RubricItem, Severity as JudgeSeverity, VerdictLabel, Violation,
    WorkingSpecEvidence,
};
pub use planning::{
    CodeQualityMetrics, CoverageMetrics, DetailedQualityMetrics, DocumentationQualityMetrics,
    HardwareResourceRequirements, HumanResourceRequirements, NetworkRequirements,
    PerformanceMetrics, QualityMetrics, TestQualityMetrics,
};
pub use planning_io::*;
pub use ports::council_coordinator::CouncilCoordinator;
pub use ports::data_processing::DataProcessingService;
pub use ports::memory_system::MemorySystem;
pub use ports::planning_engine::PlanningEngine;
pub use ports::research_evidence::ResearchEvidenceCollector;
pub use ports::tool_chain::ToolChainPlanner;
#[cfg(test)]
pub use task_executor_provider::tests::{MockTaskExecutor, MockTaskExecutorProvider};
pub use types::prelude::*;
