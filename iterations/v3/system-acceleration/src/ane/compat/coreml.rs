//! Core ML compatibility layer for ANE operations
//!
//! This module provides a safe interface to Core ML framework functionality
//! for Apple Neural Engine operations, avoiding direct private framework usage.

// Re-export types from the types module
pub use crate::ane::compat::types::*;

// Re-export model functionality
pub use crate::ane::compat::model::*;

// Re-export tokenizer functionality
pub use crate::ane::compat::tokenizer::*;

// Re-export KV cache functionality
pub use crate::ane::compat::kv_cache::*;

// Re-export testing infrastructure
pub use crate::ane::compat::testing::*;

// Re-export safety utilities
pub use crate::ane::compat::safety::*;

// Re-export hardening utilities
pub use crate::ane::compat::hardening::*;

// Re-export integration system
pub use crate::ane::compat::integration::*;

// Re-export registry types and functions
pub use crate::ane::compat::registry::*;

// Re-export coreml_module as coreml for backward compatibility
pub use crate::ane::compat::coreml_module as coreml;

// Explicitly re-export key functions from coreml_module
pub use crate::ane::compat::coreml_module::{run_inference, load_model, compile_model, is_ane_available, detect_coreml_capabilities};

// All implementations moved to appropriate modules
// This file now serves as a clean re-export facade
