//! Artifact Management System
//!
//! Manages storage, versioning, and retrieval of execution artifacts
//! including code changes, test results, coverage reports, and provenance.

pub mod manager;
pub mod storage;
pub mod versioning;

pub use manager::{ArtifactManager, ArtifactManagerConfig};
pub use storage::{ArtifactStorage, FileSystemStorage, DatabaseStorage};
pub use versioning::{VersionControl, GitVersionControl, DatabaseVersionControl};
