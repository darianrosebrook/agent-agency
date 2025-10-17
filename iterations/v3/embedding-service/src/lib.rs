//! Embedding Service for V3 Council System
//! 
//! Provides semantic context understanding through embedding generation and similarity search.
//! Integrates with Ollama embeddinggemma for 768-dimensional vector generation.

pub mod provider;
pub mod service;
pub mod similarity;
pub mod context;
pub mod types;
pub mod cache;

#[cfg(test)]
mod tests;

pub use provider::*;
pub use service::*;
pub use similarity::*;
pub use context::*;
pub use types::*;
pub use cache::*;
