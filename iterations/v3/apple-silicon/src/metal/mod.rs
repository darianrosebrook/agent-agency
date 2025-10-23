//! Metal GPU acceleration module for Apple Silicon
//!
//! This module provides GPU-accelerated inference capabilities using Metal on Apple Silicon.
//! It includes embeddings, caching, quantization, and performance monitoring.

pub mod core;
pub mod embeddings;
pub mod models;
pub mod cache;
pub mod quantization;

#[cfg(test)]
mod tests;

// Re-export main types
pub use core::*;
pub use embeddings::*;
pub use models::*;
pub use cache::*;
pub use quantization::*;
