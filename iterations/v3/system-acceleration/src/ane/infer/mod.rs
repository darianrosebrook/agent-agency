//! Inference execution for ANE operations
//!
//! This module provides async inference execution with proper timeout handling,
//! performance monitoring, and error recovery for Apple Neural Engine operations.

pub mod execute;
pub mod whisper;
// TEMPORARILY DISABLED: yolo module due to candle-core dependency conflicts
// pub mod yolo;
pub mod mistral;

// Re-export commonly used types
pub use execute::{
    execute_inference, execute_batch_inference, InferenceOptions, InferenceResult,
    InferenceMetrics, update_performance_metrics,
};

// Re-export Whisper inference
pub use whisper::{
    WhisperInferenceExecutor, create_whisper_executor,
};

// TEMPORARILY DISABLED: YOLO re-exports due to candle-core dependency conflicts
// Re-export YOLO inference
// pub use yolo::{
//     YOLOInferenceExecutor, create_yolo_executor,
// };

// Re-export Mistral inference (stub types only - functions disabled)
pub use mistral::{
    MistralInferenceOptions, ConstitutionalVerdict, ComplianceLevel, RiskTier, Verdict,
    DebateArgument, DebatePosition, ConfidenceLevel,
};
