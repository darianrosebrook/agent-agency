#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

//! Embedding Service for V3 Council System
//!
//! Provides semantic context understanding through embedding generation and similarity search.
//! Integrates with Ollama embeddinggemma for 768-dimensional vector generation.

pub mod cache;
pub mod context;
pub mod indexer;
pub mod model_loading;
pub mod provider;
pub mod embedding_service;
pub mod similarity;
pub mod tokenization;
pub mod prompting_types;

#[cfg(test)]
mod tests;

pub use cache::{EmbeddingCache, ModelCache, ModelCacheStats, ModelCacheInfo, ModelCacheError};
pub use context::*;
pub use indexer::orchestrator::MultimodalIndexer;
pub use provider::*;
pub use embedding_service::*;
pub use similarity::*;
pub use prompting_prompting_embedding_types::*;
