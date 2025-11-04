//! Modular multimodal retriever components
//!
//! This module provides decomposed components for multimodal retrieval,
//! organized by responsibility for better maintainability and separation of concerns.

pub mod core;
pub mod search;
pub mod text_search;
pub mod visual_search;
pub mod fusion;
pub mod indexing;
pub mod query_processing;

// Re-export the main components for backward compatibility
pub use core::{MultimodalRetriever, MultimodalRetrieverConfig, MultimodalQuery, QueryType, FusionStrategy, MultimodalSearchResult};

// Re-export supporting types
pub use text_search::{Bm25Index, VectorIndex, TextSearchBridge, TextSearchEngine};
pub use visual_search::VisualSearchBridge;
pub use search::SearchCoordinator;
pub use fusion::FusionEngine;
pub use query_processing::QueryProcessor;

// Define any missing types for compatibility
pub use core::MultimodalRetriever as Retriever;
