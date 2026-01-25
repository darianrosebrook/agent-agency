//! V4 Executable Invariants
//!
//! This crate implements Sterling-style invariants that are **executable code**,
//! not just documentation. Each invariant can be checked at runtime, and
//! violations are hard failures that block execution.
//!
//! ## Design Principles
//!
//! 1. **Invariants are code** - Not comments, not markdown
//! 2. **Fail-closed** - Unknown state = invariant violated
//! 3. **Auditable** - Every check is logged with result
//! 4. **Composable** - Invariants can be combined

pub mod caws;
pub mod checker;
pub mod core;

#[cfg(test)]
mod proptest_checks;

pub use caws::{CAWSInvariant, run_caws_checks};
pub use core::{CoreInvariant, InvariantId};
pub use checker::{InvariantChecker, InvariantResult, InvariantViolation};

/// Re-export commonly used types
pub use v4_types::errors::ValidationSeverity as Severity;
