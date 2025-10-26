//! Model quantization and optimization
//!
//! Dynamic quantization strategies for model size reduction
//! and performance optimization across acceleration backends.

pub mod quantization;

// Re-export main types
pub use quantization::*;
