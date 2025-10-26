//! Vector Search Engine
//!
//! Provides semantic search capabilities using vector embeddings and Qdrant database.
//!
//! This file has been decomposed into focused modules:
//! - core.rs: Main engine structure and initialization
//! - search.rs: Search operations and result processing
//! - embedding.rs: Embedding generation and processing
//! - cache.rs: Caching functionality
//! - metrics.rs: Performance metrics collection
//! - qdrant.rs: Qdrant database integration
//! - text_processing.rs: Text preprocessing and normalization

pub use vector_search::*;
