//! Council coordinator orchestrator module
//!
//! Main orchestration for council consensus, evaluation, and decision-making
//! with specialized components for queue management, metrics, and task evaluation.

pub mod coordinator;
pub mod queue;
pub mod metrics;
pub mod evaluation;
pub mod types;

// Re-export main coordinator
pub use coordinator::ConsensusCoordinator;
