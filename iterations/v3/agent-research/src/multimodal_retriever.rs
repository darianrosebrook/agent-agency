//! Multimodal Retriever - Modular Cross-Modal Search Engine
//!
//! Main multimodal retriever that orchestrates search across text, visual, and code modalities.
//!
//! This module has been decomposed into focused sub-modules for better maintainability:
//! - core: Main MultimodalRetriever struct and configuration
//! - search: Search coordination across modalities
//! - text_search: BM25 and vector text search
//! - visual_search: Image similarity and description
//! - fusion: Result fusion from multiple modalities
//! - indexing: Document and image indexing
//! - query_processing: Query parsing and validation

pub mod multimodal_retriever;

// Re-export the modular components for backward compatibility
pub use multimodal_retriever::*;
