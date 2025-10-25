//! Intelligent Edge Case Testing Module
//!
//! This module provides comprehensive intelligent testing capabilities
//! broken down into focused, SOLID-compliant components.

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

// Re-export main types and orchestrator
pub use orchestrator::IntelligentEdgeCaseTesting;
pub use types::*;
pub use errors::*;
