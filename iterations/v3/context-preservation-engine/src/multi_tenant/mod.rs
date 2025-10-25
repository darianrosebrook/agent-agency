//! Modular multi-tenant context preservation system
//!
//! This module provides a decomposed version of the monolithic MultiTenantManager
//! into focused, maintainable components following SOLID principles.

pub mod types;
pub mod tenant;
pub mod storage;
pub mod limits;
pub mod cache;
pub mod health;
pub mod security;
pub mod manager;

// Re-export the main types and structs for easy access
pub use types::*;
pub use tenant::*;
pub use storage::*;
pub use limits::*;
pub use cache::*;
pub use health::*;
pub use security::*;
pub use manager::*;
