//! Optimization module for runtime performance improvements
//!
//! This module provides components for optimizing runtime performance including
//! multi-stage pipelines, auto-tuning, and streaming execution.
//!
//! @author @darianrosebrook

pub mod multi_stage_pipeline;
pub mod auto_tuner;
pub mod streaming_executor;

// Re-export main types
pub use multi_stage_pipeline::{
    MultiStagePipeline, TaskClassification, TaskComplexity, WorkerSelectionResult,
    PipelineStageResult, DualExecutionConfig,
};

pub use auto_tuner::{
    AutoTuner, ParameterSpace, OptimizationObjective, PerformanceMeasurement,
    BayesianOptimizationConfig, OptimizationStatistics,
};

pub use streaming_executor::{
    StreamingTaskExecutor, StreamingConfig, TaskExecutionState, TaskChunk,
    ExecutionCheckpoint,
};


