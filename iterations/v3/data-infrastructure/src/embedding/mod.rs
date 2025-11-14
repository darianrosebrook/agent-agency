//! Embedding service for semantic understanding
//!
//! Provides semantic context understanding through embedding generation
//! and similarity search using CoreML (with DummyEmbeddingProvider fallback)
//! and vector operations.

pub mod context;
pub mod embedding_cache;
pub mod embedding_service;
pub mod embedding_types;
pub mod indexer;
pub mod model_loading;
pub mod ort_compat;
pub mod provider;
pub mod similarity;
pub mod tokenization;

// Re-export main types
pub use embedding_service::*;
pub use embedding_types::*;
pub use provider::{CoreMLEmbeddingProvider, DummyEmbeddingProvider, EmbeddingProvider};
pub use similarity::*;
pub use tokenization::Tokenizer;
