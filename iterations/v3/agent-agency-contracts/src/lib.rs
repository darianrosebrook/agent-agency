//! Agent Agency V3 shared interoperability contracts.
//!
//! Provides strongly typed data contracts and JSON Schema backed validators
//! so workers, council, orchestration, and provenance components exchange
//! data safely with deterministic error handling.

pub mod error;
mod schema;
pub mod worker_output;
pub mod judge_verdict;
pub mod final_verdict;
pub mod router_decision;

pub use error::{ContractError, ContractKind, ValidationIssue};
pub use final_verdict::{
    validate_final_verdict_value, FinalDecision, FinalVerdictContract, VerificationSummary,
    VoteEntry, VoteVerdict,
};
pub use judge_verdict::{
    validate_judge_verdict_value, EvidenceItem, EvidenceType, JudgeDecision, JudgeVerdictContract,
};
pub use router_decision::{
    validate_router_decision_value, Assignment, RouterDecisionContract, WorkerType,
};
pub use worker_output::{
    validate_worker_output_value, CawsChecklist, ClaimContract, CommandArtifact, EvidenceReference,
    PatchArtifact, WaiverContract, WorkerArtifacts, WorkerMetadata, WorkerOutputContract,
    WorkerSeeds, WorkerSelfAssessment,
};
pub use schema::{
    final_verdict_schema_source, judge_verdict_schema_source, router_decision_schema_source,
    worker_output_schema_source,
};
