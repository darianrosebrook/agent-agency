//! Agent Evaluation Framework
//!
//! Provides iteration limits, quality ceiling detection, and delta thresholds
//! for autonomous task execution evaluation.

pub mod evaluation;

pub use evaluation::{
    EvaluationConfig, EvaluationHook, EvaluationOrchestrator, IterationEvaluation,
    NoOpEvaluationHook, StopReason,
};
