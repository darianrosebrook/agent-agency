//! Apple Neural Engine (ANE) module
//!
//! This module has been refactored into submodules for better organization.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// ANE capabilities information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ANECapabilities {
    /// Whether ANE is available on this system
    pub is_available: bool,
    /// Number of ANE compute units available
    pub compute_units: u32,
    /// Maximum memory available for ANE operations (in MB)
    pub max_memory_mb: Option<u64>,
    /// Supported precision formats
    pub supported_precisions: Vec<String>,
    /// Performance characteristics
    pub performance_score: Option<f64>,
}

/// Tensor specification for model I/O
#[derive(Debug, Clone, JsonSchema)]
pub struct TensorSpec {
    /// Tensor name
    pub name: String,
    /// Data type
    pub dtype: String,
    /// Shape dimensions
    pub shape: Vec<usize>,
    /// Whether this tensor is required for input
    pub required: bool,
    /// Whether this tensor supports batching
    pub batch_capable: bool,
}

// Re-export public types from submodules
pub use self::ffi::*;
pub use self::filesystem::*;
pub use self::manager::*;

// Submodules
pub mod ffi;
pub mod filesystem;
pub mod manager;

// New ANE implementation modules
pub mod ane_errors;
pub mod compat;
pub mod resource_pool;
pub mod models;
pub mod infer;
pub mod metrics;
pub mod ane_circuit_breaker;
pub mod monitoring;
pub mod optimization;

// Re-export Mistral functionality
pub use models::mistral_model::{MistralModel, MistralCompilationOptions, load_mistral_model, estimate_memory_usage, validate_mistral_compatibility};
// Re-export Mistral types (functions disabled due to candle-core conflicts)
pub use infer::mistral::{MistralInferenceOptions, ConstitutionalVerdict, ComplianceLevel, RiskTier, Verdict, DebateArgument, DebatePosition, ConfidenceLevel};

// Re-export circuit breaker
pub use crate::ane::ane_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState, CircuitBreakerError};
