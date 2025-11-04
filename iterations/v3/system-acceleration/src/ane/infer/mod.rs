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
pub mod yolo;
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

// CRITICAL: DO NOT REMOVE OR DISABLE - YOLO re-exports are production functionality
// These exports enable YOLO object detection capabilities.
// DO NOT comment out or disable these re-exports.
// DO NOT remove YOLO functionality from the public API.
// Last fixed: P0 priority - candle-core dependency alignment (2025-01-XX)
// Re-export YOLO inference
pub use yolo::{
    YOLOInferenceExecutor, create_yolo_executor,
};

// Re-export Mistral inference (stub types only - functions disabled)
pub use mistral::{
    MistralInferenceOptions, ConstitutionalVerdict, ComplianceLevel, RiskTier, Verdict,
    DebateArgument, DebatePosition, ConfidenceLevel,
};
