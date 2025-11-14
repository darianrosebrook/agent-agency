//! Modular multimodal retriever components
//!
//! This module provides decomposed components for multimodal retrieval,
//! organized by responsibility for better maintainability and separation of concerns.

pub mod core;
pub mod fusion;
pub mod indexing;
pub mod query_processing;
pub mod search;
pub mod text_search;
pub mod visual_search;

// Re-export the main components for backward compatibility
pub use core::{
    FusionStrategy, MultimodalQuery, MultimodalRetriever, MultimodalRetrieverConfig,
    MultimodalSearchResult,
};
// QueryType is now imported from contracts, not core
pub use agent_agency_contracts::types::research::QueryType;

// Re-export supporting types
pub use fusion::FusionEngine;
pub use query_processing::QueryProcessor;
pub use search::SearchCoordinator;
pub use text_search::{Bm25Index, TextSearchBridge, TextSearchEngine, VectorIndex};
pub use visual_search::VisualSearchBridge;

// Define any missing types for compatibility
pub use core::MultimodalRetriever as Retriever;
