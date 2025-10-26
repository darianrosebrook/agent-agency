//! Buffer pool management for acceleration backends
//!
//! Provides efficient memory management for GPU buffers and tensors
//! across different acceleration backends.

pub mod buffer_pool;

// Re-export main types
pub use buffer_pool::*;
