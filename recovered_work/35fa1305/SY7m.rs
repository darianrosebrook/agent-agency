//! Agent Agency V3 shared interoperability contracts.
//!
//! Provides strongly typed data contracts and JSON Schema backed validators
//! so workers, council, orchestration, and provenance components exchange
//! data safely with deterministic error handling.

pub mod error;
pub mod execution_artifacts;
pub mod final_verdict;
pub mod judge_verdict;
pub mod quality_report;
pub mod refinement_decision;
pub mod router_decision;
mod schema;
pub mod task_request;
pub mod task_response;
pub mod working_spec;
pub mod worker_output;

pub use error::{ContractError, ContractKind, ValidationIssue};
pub use execution_artifacts::{
    validate_execution_artifacts_value, AuditEvent, CodeChanges, CoverageResults, DiffArtifact,
    DiffHunk, E2eScenarioResult, E2eTestResults, ExecutionArtifacts, ExecutionEnvironment,
    ExecutionSeeds, GitInfo, LintingIssue, LintingResults, NewFileArtifact,
    Provenance, TestArtifacts, TestFileInfo, TestFileStatus, TestFileType, TestResult, TestStatus, TestSuiteResults,
    UncoveredBranch, UncoveredLines,
};
pub use final_verdict::{
    validate_final_verdict_value, FinalDecision, FinalVerdictContract, VerificationSummary,
    VoteEntry, VoteVerdict,
};
pub use judge_verdict::{
    validate_judge_verdict_value, EvidenceItem as JudgeEvidenceItem, EvidenceType as JudgeEvidenceType, JudgeDecision, JudgeVerdictContract,
};
pub use quality_report::{
    validate_quality_report_value, GateIssue, GatePerformanceMetrics, GateResult, GateStatus, GateType,
    IssueSeverity, OverallStatus, QualityDeltas, QualityReport, QualityThresholds, Recommendation,
    RecommendationCategory, RecommendationPriority, ReportMetadata, ResourceUsage,
};
pub use refinement_decision::{
    validate_refinement_decision_value, ActionPriority, ActionType, ConstraintAdjustments,
    CouncilDecision, CouncilVerdict, DecisionMetadata, DissentingOpinion, EffortLevel,
    EvidenceItem as RefinementEvidenceItem, EvidenceSeverity, EvidenceType as RefinementEvidenceType, FocusArea, JudgeContribution, JudgeType, RefinementDecision,
    RefinementDirective, RiskAssessment, RiskFactor, RiskLevel, ScopeAdjustment,
    SpecificAction, BudgetIncrease, QualityTargets,
};
pub use router_decision::{
    validate_router_decision_value, Assignment, RouterDecisionContract, WorkerType,
};
pub use schema::{
    execution_artifacts_schema_source, final_verdict_schema_source, judge_verdict_schema_source,
    quality_report_schema_source, refinement_decision_schema_source, router_decision_schema_source,
    task_request_schema_source, task_response_schema_source, working_spec_schema_source,
    worker_output_schema_source,
};
pub use task_request::{
    validate_task_request_value, BudgetLimits, ChangeType, Environment, FileChange,
    RiskTier, ScopeRestrictions, TaskConstraints, TaskMetadata, TaskPriority,
    TaskRequest, TaskContext,
};
pub use task_response::{
    validate_task_response_value, TaskError, TaskProgress, TaskResponse, TaskStatus,
    TaskExecutionMetadata, WorkingSpecSummary,
};
pub use working_spec::{
    validate_working_spec_value, AcceptanceCriterion, ChangeType as WorkingSpecChangeType,
    CoverageTargets, DataImpact, task_request::Environment as WorkingSpecEnvironment,
    FileChange as WorkingSpecFileChange, MoSCoWPriority, NonFunctionalRequirements, PerformanceRequirements,
    RollbackPlan, RollbackStrategy, ScalabilityRequirements,
    TestPlan, UnitTestSpec, IntegrationTestSpec, E2eScenario, ValidationResults,
    WorkingSpec, WorkingSpecConstraints, WorkingSpecContext, WorkingSpecMetadata,
    WeightStats,
};
pub use worker_output::{
    validate_worker_output_value, CawsChecklist, ClaimContract, CommandArtifact, EvidenceReference,
    PatchArtifact, WaiverContract, WorkerArtifacts, WorkerMetadata, WorkerOutputContract,
    WorkerSeeds, WorkerSelfAssessment,
};
