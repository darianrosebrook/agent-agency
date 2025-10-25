//! Coordinator module
//!
//! Multi-turn learning coordination with specialized components
//! for quality analysis, resource monitoring, failure detection,
//! and learning algorithm orchestration.

pub mod quality;
pub mod resources;
pub mod failures;
pub mod algorithms;
pub mod state;
pub mod orchestrator;

// Re-export main coordinator
pub use orchestrator::MultiTurnLearningCoordinator;
