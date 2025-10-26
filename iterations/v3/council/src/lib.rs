#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

//! Council Review & Decision Making for Agent Agency V3
//!
//! The Council system coordinates multiple AI judges to review working specifications,
//! aggregate verdicts, and make final decisions on task execution. It implements
//! sophisticated consensus algorithms and handles dissenting opinions.

pub mod council_errors;
pub mod judge;
pub mod mistral_tokenizer;
pub mod mistral_judge_integration_test;
pub mod mistral_integration_demo;
pub mod council;
pub mod decision_making;
pub mod verdict_aggregation;
pub mod verdict;
pub mod workflow;
pub mod risk_scorer;
pub mod error_handling;
pub mod coordinator;
pub mod models;
pub mod prompting_types;
pub mod evidence_enrichment;
pub mod resilience;
pub mod claim_extraction_multimodal;
pub mod learning;
pub mod model_client;
pub mod advanced_monitoring;
pub mod intelligent_testing;
pub mod predictive_learning;

pub use council_errors::{CouncilError, CouncilResult};
pub use judge_backup::{
    Judge, JudgeConfig, JudgeVerdict, JudgeType, JudgeContribution,
    // Ethical analysis types
    risk::{EthicalAssessment, EthicalConcern, EthicalCategory, EthicalSeverity,
           StakeholderImpact, EthicalTradeoff, ConsequenceAssessment},
    // Ethics judge
    EthicsJudge,
    // Mock judge
    MockJudge,
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
pub use evidence_enrichment::EvidenceEnrichmentCoordinator;
pub use resilience::ResilienceManager;
pub use claim_extraction_multimodal::{MultimodalEvidenceEnricher, ClaimWithMultimodalEvidence};
pub use advanced_monitoring::{SLOTracker, SLOStatus, SLOAlert, AlertLevel, SLOComponent, SLODashboardSummary};
pub use verdict::{VerdictStore, VerdictRecord, VerdictStorage, CacheConfig, StorageStats, CacheStats, VerdictStoreStats};
pub use prompting_council_types::ResourceUsageMetrics;
pub use coordinator::orchestrator::{ConsensusCoordinator, ProvenanceEmitter};