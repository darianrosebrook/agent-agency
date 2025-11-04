//! Judge backup implementations module
//!
//! Backup judge implementations with specialized components
//! for verdict handling, risk assessment, ethics evaluation,
//! and mock testing capabilities.

pub mod types;
pub mod traits;
pub mod verdicts;
pub mod risk;
pub mod ethics;
pub mod mock;
pub mod backup_types;
pub mod quality_judge;
pub mod security_judge;

// Re-export main types and implementations
pub use traits::Judge;
pub use verdicts::{JudgeVerdict, RequiredChange, ChangePriority, EffortEstimate, CriticalIssue, IssueSeverity, ChangeImpact};
pub use risk::{RiskAssessment, RiskLevel, MultiDimensionalRiskAssessment, TechnicalRiskAssessment, EthicalRiskAssessment, OperationalRiskAssessment, BusinessRiskAssessment, EthicalCategory};
pub use ethics::EthicsJudge;
pub use mock::MockJudge;
pub use quality_judge::QualityAssuranceJudge;
pub use security_judge::SecurityJudge;
pub use types::{JudgeConfig, JudgeContribution, JudgeHealthMetrics, VerdictSummary, ReviewContext, PreviousReview};
