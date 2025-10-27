//! Context Preservation Module - Unified context management
//!
//! This module consolidates context preservation functionality from multiple sources:
//! - Full-featured context management with multi-tenant support
//! - Working memory folding and lifecycle management
//! - Context compression, summarization, and archival

pub mod manager;
pub mod types;

// Re-export main types and functionality
#[cfg(feature = "embeddings")]
pub use manager::ContextManager;
pub use types::*;
