//! V3 Recovery System: Content-Addressable Storage
//!
//! This crate provides a Git-like content-addressable storage system with:
//! - BLAKE3-based content addressing
//! - Crash-safe journaled writes with directory fsyncs
//! - File metadata preservation (modes, symlinks)
//! - Secret pre-admission scanning
//! - CAWS governance integration
//!
//! @author @darianrosebrook

#![deny(unused_imports, unused_must_use)]
#![warn(unused_variables, dead_code)]
#![allow(ambiguous_glob_reexports, unused_variables, dead_code, unused_assignments)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Scope for filesystem check operations
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum FsckScope {
    /// Check entire repository
    Full,
    /// Check specific paths
    Paths(Vec<String>),
    /// Check recent commits
    Recent { days: u32 },
}

/// Status of filesystem check operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum FsckStatus {
    /// Check completed successfully
    Ok,
    /// Check found issues
    IssuesFound,
    /// Check failed due to errors
    Failed,
}

/// Report from filesystem check operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FsckReport {
    /// Overall status
    pub status: FsckStatus,
    /// Issues found during check
    pub issues: Vec<String>,
    /// Number of objects checked
    pub objects_checked: u64,
    /// Number of corrupted objects found
    pub objects_corrupted: u64,
    /// Number of refs checked
    pub refs_checked: u64,
    /// Number of dangling refs found
    pub refs_dangling: u64,
}

pub mod recovery_api;
pub mod cas;
pub mod merkle;
pub mod journal;
pub mod refs;
pub mod policy;
pub mod gc;
pub mod fsck;
pub mod index;
pub mod recovery_types;
pub mod integration;
pub mod recovery_metrics;
pub mod resilience_circuit_breaker;
pub mod retry;

// Workspace state management (consolidated from workspace-state-manager crate)
pub mod workspace_state;

// Memory management (consolidated from memory crate)
pub mod memory;

// Re-export key types for convenience
pub use recovery_api::*;
pub use recovery_types::*;
pub use integration::*;
pub use recovery_metrics::{*, MetricsBackend};
pub use resilience_circuit_breaker::*;
pub use retry::*;
// pub use source_integrity::{Digest, StreamingHasher, MerkleTree};  // Temporarily disabled
