//! Learning System Components
//!
//! This module provides adaptive learning capabilities for the worker orchestration system.
//! It includes components for tracking performance, analyzing patterns, optimizing configurations,
//! and persisting learning data.

pub mod adaptive_selector;
pub mod config_optimizer;
pub mod fairness_monitor;
pub mod learning_persistence;
pub mod metrics_collector;
pub mod pattern_analyzer;
pub mod queue_health_monitor;
pub mod failure_taxonomy;
pub mod types;

// Re-export main types and traits
pub use adaptive_selector::{AdaptiveWorkerSelector, WorkerSelectionStrategy};
pub use config_optimizer::ConfigurationOptimizer;
pub use fairness_monitor::FairnessMonitor;
pub use learning_persistence::LearningPersistence;
pub use metrics_collector::ParallelWorkerMetricsCollector;
pub use pattern_analyzer::PatternAnalyzer;
pub use queue_health_monitor::QueueHealthMonitor;
pub use failure_taxonomy::FailureTaxonomy;
pub use types::{
    OptimizationEvent, ConfigurationRecommendations, FairnessMetrics, ExecutionRecord,
    WorkerPerformanceProfile, SuccessPattern, FailurePattern, OptimalConfig,
    RewardWeights, Baseline, TaskPattern, PatternMatch, QueueHealthMetrics,
    FailureCategory, FailureAnalysis
};
