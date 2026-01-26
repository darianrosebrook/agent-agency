//! Inference execution for ANE operations
//!
//! This module provides async inference execution with proper timeout handling,
//! performance monitoring, and error recovery for Apple Neural Engine operations.
//!
//! ## Feature Flags
//!
//! - `whisper` - Speech-to-text transcription (PLACEHOLDER - not production-ready)
//! - `yolo` - Object detection (PLACEHOLDER - not production-ready)
//! - `multimodal` - Enables both whisper and yolo
//!
//! WARNING: The whisper and yolo features are currently placeholder implementations.
//! They will compile and run but produce degraded or incorrect results.
//! Do not enable these features in production until full implementations are complete.

pub mod execute;
pub mod mistral;
pub mod policy_integration;

// Whisper module - speech-to-text (placeholder implementation)
// WARNING: This module contains placeholder implementations for STFT, mel filterbank,
// and decoder. Transcription results will be incorrect until full implementation.
#[cfg(feature = "whisper")]
pub mod whisper;

// YOLO module - object detection (placeholder implementation)
// WARNING: This module contains placeholder implementations for object detection.
// Detection results will be incorrect until full implementation.
#[cfg(feature = "yolo")]
pub mod yolo;

// Re-export commonly used types
pub use execute::{
    execute_batch_inference, execute_inference, update_performance_metrics, InferenceMetrics,
    InferenceOptions, InferenceResult,
};

// Re-export Whisper inference (only when feature enabled)
#[cfg(feature = "whisper")]
pub use whisper::{create_whisper_executor, WhisperInferenceExecutor};

// Re-export YOLO inference (only when feature enabled)
#[cfg(feature = "yolo")]
pub use yolo::{create_yolo_executor, YOLOInferenceExecutor};

// Re-export Mistral inference (stub types only - functions disabled)
pub use mistral::{
    ComplianceLevel, ConfidenceLevel, ConstitutionalVerdict, DebateArgument, DebatePosition,
    MistralInferenceOptions, MistralInferenceResult, PrefillDecodeMetrics, RiskTier, Verdict,
};
