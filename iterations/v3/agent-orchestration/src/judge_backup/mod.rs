//! Judge backup implementations module
//!
//! Backup judge implementations with specialized components
//! for verdict handling, risk assessment, ethics evaluation,
//! and mock testing capabilities.

pub mod verdicts;
pub mod risk;
pub mod ethics;
pub mod mock;
pub mod types;
pub mod traits;

// Re-export main types and implementations
pub use verdicts::{JudgeVerdict, RequiredChange, ChangePriority, EffortEstimate};
pub use risk::{RiskAssessment, RiskLevel, MultiDimensionalRiskAssessment};
pub use ethics::EthicsJudge;
pub use mock::MockJudge;
pub use types::{JudgeConfig, JudgeContribution, JudgeHealthMetrics};
pub use traits::Judge;
