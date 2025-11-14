//! Optimization module for runtime performance improvements
//!
//! This module provides components for optimizing runtime performance including
//! multi-stage pipelines, auto-tuning, and streaming execution.
//!
//! @author @darianrosebrook

pub mod auto_tuner;
pub mod multi_stage_pipeline;
pub mod streaming_executor;

// Re-export main types
pub use multi_stage_pipeline::{
    DualExecutionConfig, MultiStagePipeline, PipelineStageResult, TaskClassification,
    TaskComplexity, WorkerSelectionResult,
};

pub use auto_tuner::{
    AutoTuner, BayesianOptimizationConfig, OptimizationObjective, OptimizationStatistics,
    ParameterSpace, PerformanceMeasurement,
};

pub use streaming_executor::{
    ExecutionCheckpoint, StreamingConfig, StreamingTaskExecutor, TaskChunk, TaskExecutionState,
};
