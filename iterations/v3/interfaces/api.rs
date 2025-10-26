//! Backward-compatible re-export of the decomposed REST API interface.
//!
//! This file maintains the original public API while the actual implementation
//! has been decomposed into focused modules in the `api/` subdirectory for
//! better maintainability and separation of concerns.

pub use api::*;
