//! Vector Search Engine Modules
//!
//! Decomposed from the monolithic vector_search.rs file (1,259 LOC)
//! into focused, single-responsibility modules.

pub mod vector_core;
pub mod vector_search_ops;
pub mod vector_embedding;
pub mod vector_search_cache;
pub mod vector_metrics;
pub mod vector_qdrant;
pub mod vector_text_processing;

// Re-export main types
pub use vector_core::VectorSearchEngine;
pub use vector_metrics::VectorSearchMetrics;
pub use vector_embedding::EmbeddingProcessor;
pub use vector_search_cache::CacheManager;
pub use vector_qdrant::QdrantClient;
pub use vector_text_processing::TextProcessor;
