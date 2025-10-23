//! FFI interfaces for the Council system
//!
//! This module provides foreign function interfaces for interacting
//! with external systems, particularly CoreML models.

pub mod coreml;

/// Re-export CoreML types for convenience
pub use coreml::*;
