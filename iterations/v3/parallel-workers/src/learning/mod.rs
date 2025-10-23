//! Learning system for parallel worker optimization
//!
//! Provides adaptive worker selection, configuration optimization, pattern recognition,
//! and integration with the council learning system.

pub mod reward;
pub mod drift;
pub mod feature_store;
pub mod queue_health;
pub mod fairness;
pub mod failure_taxonomy;
pub mod metrics_collector;
pub mod council_bridge;
pub mod pattern_analyzer;
pub mod adaptive_selector;
pub mod config_optimizer;
pub mod persistence;

pub use reward::{RewardWeights, Baseline, compute_reward, LearningMode};
pub use drift::{DriftDetector, DriftDecision, CusumDetector, MetricDriftGuard};
pub use feature_store::{FeatureStore, FeatureVector, InMemoryFeatureStore};
pub use queue_health::{QueueHealth, QueueManager, SchedulingDiscipline};
pub use fairness::{FairnessConstraints, FairnessTracker};
pub use failure_taxonomy::{FailureCategory, Remediation, RCAClassifier, ExecutionContext};
pub use metrics_collector::{ParallelWorkerMetricsCollector, ExecutionRecord, WorkerPerformanceProfile};
pub use council_bridge::{CouncilLearningBridge, ParallelWorkerSignal};
pub use pattern_analyzer::{PatternAnalyzer, PatternCache, SuccessPattern, FailurePattern, OptimalConfig};
pub use adaptive_selector::{AdaptiveWorkerSelector, SelectionStrategy, WorkerRecommendation, PerformanceEstimate};
pub use config_optimizer::{ConfigurationOptimizer, OptimizationEvent, OptimizationType, ConfigurationRecommendations, ConfigRecommendation, ExpectedImpact};
pub use persistence::{LearningPersistence, BatchBuffer};
