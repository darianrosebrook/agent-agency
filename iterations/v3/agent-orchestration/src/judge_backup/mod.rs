//! Judge backup implementations module
//!
//! Backup judge implementations with specialized components
//! for verdict handling, risk assessment, ethics evaluation,
//! and mock testing capabilities.

pub mod backup_types;
pub mod ethics;
pub mod mock;
pub mod quality_judge;
pub mod risk;
pub mod security_judge;
pub mod traits;
pub mod types;
pub mod verdicts;

// Re-export main types and implementations
pub use ethics::EthicsJudge;
pub use mock::MockJudge;
pub use quality_judge::QualityAssuranceJudge;
pub use risk::{
    BusinessRiskAssessment, EthicalCategory, EthicalRiskAssessment, MultiDimensionalRiskAssessment,
    OperationalRiskAssessment, RiskAssessment, RiskLevel, TechnicalRiskAssessment,
};
pub use security_judge::SecurityJudge;
pub use traits::Judge;
pub use types::{
    JudgeConfig, JudgeContribution, JudgeHealthMetrics, PreviousReview, ReviewContext,
    VerdictSummary,
};
pub use verdicts::{
    ChangeImpact, ChangePriority, CriticalIssue, EffortEstimate, IssueSeverity, JudgeVerdict,
    RequiredChange,
};
