pub mod orchestrator;
pub mod resolution;
pub mod debate;
pub mod extraction;
pub mod authority;
pub mod metrics;

// Re-exports for ergonomic access
pub use orchestrator::{ConsensusCoordinator, ProvenanceEmitter};
pub use resolution::{CawsResolutionResult, ResolutionType};
pub use debate::{DebateContribution, SignedTranscript, ContributionAnalysis, CompiledContributions};
pub use extraction::{
    AdvancedPositionExtractor, ExtractionConfig, ExtractionResult, ExtractionMetadata,
    ExtractionStats, DecisionType, PositionConfidence, DecisionReasoning,
    RiskAssessment, PositionConsistency, PositionExtractionError, SentenceEmbeddingsModelType,
};
pub use authority::{
    ExpertAuthorityManager, ExpertAuthorityLevel, ExpertQualification, OverrideRequest,
    OverrideRiskAssessment, OverrideStatus, OverrideReason, ImpactLevel, OverrideAuditEntry,
    OverrideAction,
};
pub use metrics::{
    CoordinatorMetricsSnapshot, EvaluationMetrics, TimingMetrics, SLAMetrics,
    JudgePerformanceSnapshot, HealthIndicators,
};
