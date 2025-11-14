//! Multimodal indexer module
//!
//! Comprehensive multimodal indexing system supporting text, visual,
//! and graph modalities with unified search and retrieval capabilities.

pub mod graph;
pub mod orchestrator;
pub mod search;
pub mod storage;
pub mod text;
pub mod visual;

// Re-export main indexer
pub use orchestrator::MultimodalIndexer;
