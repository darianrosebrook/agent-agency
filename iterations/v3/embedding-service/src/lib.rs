#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

//! Embedding Service for V3 Council System
//!
//! Provides semantic context understanding through embedding generation and similarity search.
//! Integrates with Ollama embeddinggemma for 768-dimensional vector generation.

pub mod cache;
pub mod context;
pub mod model_loading;
pub mod multimodal_indexer;
pub mod provider;
pub mod service;
pub mod similarity;
pub mod tokenization;
pub mod types;

#[cfg(test)]
mod tests;

pub use cache::{EmbeddingCache, ModelCache, ModelCacheStats, ModelCacheInfo, ModelCacheError};
pub use context::*;
pub use provider::*;
pub use service::*;
pub use similarity::*;
pub use types::*;

// Explicitly re-export from multimodal_indexer to avoid conflicts
pub use multimodal_indexer::MultimodalIndexer;
