//! Core orchestration functionality
//!
//! This module contains the decomposed orchestration logic,
//! split into focused sub-modules for better maintainability.

pub mod types;
pub mod validation;
pub mod execution;
pub mod coordination;

// Re-export the main orchestration function for easy access
pub use coordination::orchestrate_task;
