//! Council Review & Decision Making for Agent Agency V3
//!
//! The Council system coordinates multiple AI judges to review working specifications,
//! aggregate verdicts, and make final decisions on task execution. It implements
//! sophisticated consensus algorithms and handles dissenting opinions.

pub mod error;
pub mod judge;
pub mod council;
pub mod decision_making;
pub mod verdict_aggregation;
pub mod workflow;
pub mod risk_scorer;
pub mod error_handling;

pub use error::{CouncilError, CouncilResult};
pub use judge::{
    Judge, JudgeConfig, JudgeVerdict, JudgeType, JudgeContribution,
    // Ethical analysis types
    EthicalAssessment, EthicalConcern, EthicalCategory, EthicalSeverity,
    StakeholderImpact, ImpactType, ImpactDuration, EthicalTradeoff,
    ConsequenceAssessment, TimeHorizon, ConsequenceSeverity,
    CulturalConsideration, CulturalSensitivity,
    // Ethics judge
    EthicsJudge,
};
pub use council::{Council, CouncilConfig, CouncilSession};
pub use judge::ReviewContext;
pub use decision_making::{DecisionEngine, ConsensusStrategy};
pub use verdict_aggregation::{VerdictAggregator, AggregationResult};
pub use workflow::{CouncilWorkflow, WorkflowState};
pub use risk_scorer::{RiskScorer, TechnicalRiskWeights, EthicalRiskWeights, OperationalRiskWeights, BusinessRiskWeights, DimensionWeights};
pub use error_handling::{
    AgencyError, ErrorCategory, ErrorSeverity, RecoveryStrategy, RecoveryStrategyType,
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerStats, CircuitBreakerState,
    RetryConfig, with_retry, DegradationManager, DegradationState, DegradationPolicy,
    DegradationLevel, RecoveryOrchestrator, SystemHealth, HealthStatus,
    error_factory,
};