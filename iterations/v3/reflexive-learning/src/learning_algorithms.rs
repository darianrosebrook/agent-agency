//! Learning algorithms for reflexive learning
//!
//! This module provides a unified interface to various learning algorithms
//! organized in a modular architecture.

pub mod reinforcement;
pub mod supervised; 
pub mod unsupervised;
pub mod ensemble;
pub mod orchestrator;

// Re-export key types and algorithms
pub use reinforcement::*;
pub use supervised::*;
pub use unsupervised::*;
pub use ensemble::*;
pub use orchestrator::*;

// Re-export common types from types module
pub use crate::types::*;
