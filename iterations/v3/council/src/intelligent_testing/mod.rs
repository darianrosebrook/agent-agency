//! Intelligent Edge Case Testing Module
//!
//! This module provides comprehensive edge case testing capabilities
//! with dynamic test generation, analysis, and optimization.

pub mod types;
pub mod generation;
pub mod analysis;
pub mod optimization;
pub mod requirements;
pub mod versioning;
pub mod performance;
pub mod nlp;
pub mod errors;
pub mod orchestrator;

// Re-export main types for convenience
pub use types::*;
pub use orchestrator::*;