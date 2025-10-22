//! Apple Neural Engine (ANE) module
//!
//! This module has been refactored into submodules for better organization.

// Re-export public types from submodules
pub use self::ffi::*;
pub use self::filesystem::*;
pub use self::manager::*;

// Submodules
pub mod ffi;
pub mod filesystem;
pub mod manager;

// New ANE implementation modules
pub mod errors;
pub mod compat;
pub mod resource_pool;
pub mod models;
pub mod infer;
pub mod metrics;
