//! Council Refinement Coordinator
//!
//! Integrates quality feedback into council decision-making for autonomous
//! refinement loops, balancing quality improvement with development efficiency.

pub mod coordinator;
pub mod strategy;
pub mod feedback_loop;

pub use coordinator::{RefinementCoordinator, RefinementCoordinatorConfig, RefinementDecision};
pub use strategy::{RefinementStrategy, RefinementPriority, RefinementScope};
pub use feedback_loop::{FeedbackLoop, FeedbackLoopConfig, QualityFeedback};


