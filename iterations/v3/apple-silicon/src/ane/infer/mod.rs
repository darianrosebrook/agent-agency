//! Inference execution for ANE operations
//!
//! This module provides async inference execution with proper timeout handling,
//! performance monitoring, and error recovery for Apple Neural Engine operations.

pub mod execute;
pub mod whisper;

// Re-export commonly used types
pub use execute::{
    execute_inference, execute_batch_inference, InferenceOptions, InferenceResult,
    InferenceMetrics, update_performance_metrics,
};

// Re-export Whisper inference
pub use whisper::{
    WhisperInferenceExecutor, create_whisper_executor,
};
