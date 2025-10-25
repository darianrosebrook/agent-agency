//! Modular self-prompting loop controller
//!
//! This module provides a decomposed version of the monolithic SelfPromptingLoop
//! into focused, maintainable components following SOLID principles.

pub mod types;
pub mod config;
pub mod state;
pub mod monitoring;
pub mod history;
pub mod events;
pub mod execution;

// Re-export the main types and structs for easy access
pub use types::*;
pub use config::*;
pub use state::*;
pub use monitoring::*;
pub use history::*;
pub use events::*;
pub use execution::*;
