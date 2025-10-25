//! Common Types and Abstractions for Agent Agency
//!
//! This crate provides shared abstractions and traits to reduce duplication
//! across the codebase while maintaining domain separation.

pub mod result;
pub mod errors;
pub mod metrics;
pub mod config;
pub mod context;
pub mod validation;

pub use result::*;
pub use errors::*;
pub use metrics::*;
pub use config::*;
pub use context::*;
pub use validation::*;
