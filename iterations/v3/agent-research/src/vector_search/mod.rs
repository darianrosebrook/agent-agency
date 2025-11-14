//! Vector Search Engine Modules
//!
//! Decomposed from the monolithic vector_search.rs file (1,259 LOC)
//! into focused, single-responsibility modules.

pub mod embedding;
pub mod qdrant;
pub mod search;
pub mod text_processing;
pub mod vector_core;
pub mod vector_metrics;
pub mod vector_search_cache;

// Re-export main types
pub use embedding::EmbeddingProcessor;
pub use qdrant::QdrantClient;
pub use text_processing::TextProcessor;
pub use vector_core::VectorSearchEngine;
pub use vector_metrics::VectorSearchMetrics;
pub use vector_search_cache::CacheManager;
