//! Modular claim decomposition system
//!
//! This module provides decomposed claim decomposition functionality,
//! organized by domain responsibility for better maintainability and separation of concerns.

pub mod core;
pub mod extractor;
pub mod brackets;
pub mod helpers;

// Re-export all types for convenient access
pub use core::*;
pub use extractor::*;
pub use brackets::*;
pub use helpers::*;
