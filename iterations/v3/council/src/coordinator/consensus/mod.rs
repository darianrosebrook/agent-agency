//! Modular consensus coordination system
//!
//! This module provides decomposed consensus coordination functionality,
//! organized by domain responsibility for better maintainability and separation of concerns.

pub mod core;
pub mod evaluation;
pub mod debate;
pub mod metrics;

// Re-export all types for convenient access
pub use core::*;
pub use evaluation::*;
pub use debate::*;
pub use metrics::*;
