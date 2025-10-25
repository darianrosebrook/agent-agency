//! Quality Gates for Agent Agency
//!
//! Automated quality enforcement to prevent:
//! - God objects (large files)
//! - Code duplication
//! - Architectural violations
//! - Security issues

pub mod checks;
pub mod config;
pub mod rules;
pub mod runner;

pub use checks::*;
pub use config::*;
pub use rules::*;
pub use runner::*;
