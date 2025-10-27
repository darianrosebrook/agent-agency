//! Vector Search Engine Modules
//!
//! Decomposed from the monolithic vector_search.rs file (1,259 LOC)
//! into focused, single-responsibility modules.

pub mod vector_core;
pub mod search;
pub mod embedding;
pub mod vector_search_cache;
pub mod vector_metrics;
pub mod qdrant;
pub mod text_processing;

// Re-export main types
pub use vector_core::VectorSearchEngine;
pub use vector_metrics::VectorSearchMetrics;
pub use embedding::EmbeddingProcessor;
pub use vector_search_cache::CacheManager;
pub use qdrant::QdrantClient;
pub use text_processing::TextProcessor;
