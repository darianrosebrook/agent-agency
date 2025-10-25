//! Multimodal indexer module
//!
//! Comprehensive multimodal indexing system supporting text, visual,
//! and graph modalities with unified search and retrieval capabilities.

pub mod text;
pub mod visual;
pub mod graph;
pub mod search;
pub mod storage;
pub mod orchestrator;

// Re-export main indexer
pub use orchestrator::MultimodalIndexer;


