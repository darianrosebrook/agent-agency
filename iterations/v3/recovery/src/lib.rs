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

pub mod api;
pub mod cas;
pub mod merkle;
pub mod journal;
pub mod refs;
pub mod policy;
pub mod gc;
pub mod fsck;
pub mod index;
pub mod types;
pub mod integration;
pub mod metrics;

// Re-export key types for convenience
pub use api::*;
pub use types::*;
pub use integration::*;
pub use metrics::*;
// pub use source_integrity::{Digest, StreamingHasher, MerkleTree};  // Temporarily disabled
