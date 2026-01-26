//! File Operations Port Documentation
//!
//! The `FileOperationsService` trait is defined in `system-common-interfaces`
//! to avoid circular dependencies. This module documents the canonical location
//! and provides guidance for consumers.
//!
//! ## Canonical Location
//!
//! The `FileOperationsService` trait and related types are defined in:
//! `system_common_interfaces::file_operations`
//!
//! ## Usage
//!
//! ```rust,ignore
//! use system_common_interfaces::{
//!     FileOperationsService,
//!     FileOpsError,
//!     FileResult,
//!     Changeset,
//!     Workspace,
//! };
//! ```
//!
//! ## Why Not in Contracts?
//!
//! The `FileOperationsService` trait cannot be in `agent-agency-contracts` because:
//! 1. `system-common-interfaces` already depends on `agent-agency-contracts`
//! 2. Adding a reverse dependency would create a cycle
//! 3. The trait is already in a shared location accessible to all crates
//!
//! ## Implementing the Trait
//!
//! Real implementations of `FileOperationsService` are provided by:
//! - `data-infrastructure::file_operations_service` - Full implementation with git workspace support
//!
//! Placeholder implementations are used when the real service is not available:
//! - `agent-mcp::tool_registry::PlaceholderFileOperationsService` - Returns errors for all operations
//!
//! @author @darianrosebrook

// This module intentionally does not re-export types to avoid circular dependencies.
// See the module documentation above for the canonical location of FileOperationsService.
