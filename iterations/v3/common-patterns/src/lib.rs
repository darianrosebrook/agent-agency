//! Common Patterns and Utilities for Agent Agency V3
//!
//! This crate provides shared patterns, traits, and utilities that are commonly
//! used across the codebase to reduce duplication while maintaining domain separation.

pub mod traits;
pub mod pattern_types;
pub mod validation;

pub use traits::*;
pub use pattern_types::*;
pub use validation::*;
