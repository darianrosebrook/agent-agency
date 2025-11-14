//! Modular claim decomposition system
//!
//! This module provides decomposed claim decomposition functionality,
//! organized by domain responsibility for better maintainability and separation of concerns.

pub mod brackets;
pub mod core;
pub mod extractor;
pub mod helpers;

// Re-export all types for convenient access
pub use brackets::*;
pub use core::*;
pub use extractor::*;
pub use helpers::*;
