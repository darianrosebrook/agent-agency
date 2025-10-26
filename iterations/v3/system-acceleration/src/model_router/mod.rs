//! Model routing and backend selection
//!
//! Intelligent routing of models to optimal acceleration backends
//! based on hardware capabilities and model requirements.

pub mod model_router;

// Re-export main types
pub use model_router::*;
