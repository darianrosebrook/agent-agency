//! Apple Neural Engine (ANE) module
//!
//! This module has been refactored into submodules for better organization.

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
pub use infer::mistral::{MistralInferenceOptions, ConstitutionalVerdict, ComplianceLevel, RiskTier, Verdict, DebateArgument, DebatePosition, ConfidenceLevel, deliberate_constitution, generate_debate_argument, generate_text};

// Re-export circuit breaker
pub use crate::ane::ane_circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState, CircuitBreakerError};
