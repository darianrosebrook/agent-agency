//! Learning algorithms for reflexive learning
//!
//! This module provides a unified interface to various learning algorithms
//! organized in a modular architecture.

pub mod ensemble;
pub mod supervised;
pub mod unsupervised;
// Note: orchestrator has been moved to crate::orchestrator
// Use crate::orchestrator::LearningOrchestrator directly
// Note: reinforcement is at crate root, not in learning_algorithms

// Re-export key types and algorithms
pub use ensemble::*;
pub use supervised::*;
pub use unsupervised::*;
// Orchestrator re-export removed - use crate::orchestrator::LearningOrchestrator directly

// Re-export common types from types module
pub use crate::reflexive_types::*;
