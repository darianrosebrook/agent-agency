//! Modular Apple Silicon types and data structures
//!
//! This module provides decomposed type definitions for Apple Silicon functionality,
//! organized by domain responsibility for better maintainability and separation of concerns.

pub mod core;
pub mod errors;
pub mod optimization;
pub mod inference;
pub mod resources;
pub mod thermal;
pub mod quality;
pub mod routing;

// Re-export all types for convenient access
pub use core::*;
pub use errors::*;
pub use optimization::*;
pub use inference::*;
pub use resources::*;
pub use thermal::*;
pub use quality::*;
pub use routing::*;
