//! Inference execution for ANE operations
//!
//! This module provides async inference execution with proper timeout handling,
//! performance monitoring, and error recovery for Apple Neural Engine operations.

pub mod execute;
pub mod whisper;
// CRITICAL: DO NOT REMOVE OR DISABLE - YOLO module is production functionality
// This module provides object detection capabilities and was restored after fixing candle-core conflicts.
// DO NOT comment out this module declaration.
// DO NOT disable YOLO functionality.
// If you encounter issues, fix them rather than disabling this module.
// Last fixed: P0 priority - candle-core dependency alignment (2025-01-XX)
pub mod mistral;
pub mod yolo;
pub mod policy_integration;

// Re-export commonly used types
pub use execute::{
    execute_batch_inference, execute_inference, update_performance_metrics, InferenceMetrics,
    InferenceOptions, InferenceResult,
};

// Re-export Whisper inference
pub use whisper::{create_whisper_executor, WhisperInferenceExecutor};

// CRITICAL: DO NOT REMOVE OR DISABLE - YOLO re-exports are production functionality
// These exports enable YOLO object detection capabilities.
// DO NOT comment out or disable these re-exports.
// DO NOT remove YOLO functionality from the public API.
// Last fixed: P0 priority - candle-core dependency alignment (2025-01-XX)
// Re-export YOLO inference
pub use yolo::{create_yolo_executor, YOLOInferenceExecutor};

// Re-export Mistral inference (stub types only - functions disabled)
pub use mistral::{
    ComplianceLevel, ConfidenceLevel, ConstitutionalVerdict, DebateArgument, DebatePosition,
    MistralInferenceOptions, MistralInferenceResult, PrefillDecodeMetrics, RiskTier, Verdict,
};
