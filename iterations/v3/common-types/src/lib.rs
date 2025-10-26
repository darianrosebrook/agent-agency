//! Common Types and Abstractions for Agent Agency
//!
//! This crate provides shared abstractions and traits to reduce duplication
//! across the codebase while maintaining domain separation.

pub mod result;
pub mod common_errors;
pub mod common_metrics;
pub mod common_config;
pub mod context;
pub mod validation;
pub mod geometry;

pub use result::*;
pub use common_errors::*;
pub use common_metrics::*;
pub use common_config::*;
pub use context::*;
pub use validation::*;
pub use geometry::*;
