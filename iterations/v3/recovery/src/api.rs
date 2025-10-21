//! Public API traits for the V3 Recovery System
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::types::*;

/// Main recovery store trait
#[async_trait]
pub trait RecoveryStore {
    /// Begin a new recovery session
    async fn begin_session(&self, meta: SessionMeta) -> Result<SessionRef>;
    
    /// Record a file change in the session
    async fn record_change(
        &self, 
        session: &SessionRef, 
        change: FileChange
    ) -> Result<ChangeId>;
    
    /// Create a checkpoint from the current session
    async fn checkpoint(
        &self, 
        session: &SessionRef, 
        label: Option<String>
    ) -> Result<CommitId>;
    
    /// Plan a restore operation
    async fn plan_restore(
        &self, 
        target: &str,  // Ref or commit
        filters: Option<RestoreFilters>
    ) -> Result<RestorePlan>;
    
    /// Apply a restore plan
    async fn apply_restore(&self, plan: RestorePlan) -> Result<RestoreResult>;
    
    /// Run filesystem check
    async fn fsck(&self, scope: FsckScope) -> Result<FsckReport>;
    
    /// Get recovery metrics
    async fn get_metrics(&self) -> Result<RecoveryMetrics>;
}

/// Restore filters for selective restoration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreFilters {
    pub globs: Vec<String>,
    pub since: Option<CommitId>,
    pub until: Option<CommitId>,
    pub file_types: Option<Vec<String>>,
}

/// Filesystem check scope
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FsckScope {
    Quick,      // Basic integrity check
    Full,       // Complete verification
    Reindex,    // Rebuild SQLite index from Merkle trees
}

/// Filesystem check report
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FsckReport {
    pub status: FsckStatus,
    pub issues: Vec<FsckIssue>,
    pub objects_checked: u64,
    pub objects_corrupted: u64,
    pub refs_checked: u64,
    pub refs_dangling: u64,
}

/// Filesystem check status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FsckStatus {
    Ok,
    Issues,
    Corrupted,
}

/// Filesystem check issue
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FsckIssue {
    pub severity: IssueSeverity,
    pub message: String,
    pub object_id: Option<Digest>,
    pub path: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Garbage collection report
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcReport {
    pub objects_marked: u64,
    pub objects_swept: u64,
    pub bytes_freed: u64,
    pub packs_created: u32,
    pub protected_refs: Vec<String>,
}

/// Content policy for strategy decisions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentPolicy {
    pub small_full_max: usize,      // 2 KiB default
    pub diff_ratio_max: f64,        // 0.45 default
    pub cdc_target: usize,          // 16 KiB default
    pub cdc_min: usize,             // 4 KiB default
    pub cdc_max: usize,             // 64 KiB default
    pub overrides: Vec<ContentOverride>,
}

/// Content strategy override for specific file patterns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentOverride {
    pub glob: String,
    pub strategy: OverrideStrategy,
}

/// Override strategy for content handling
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverrideStrategy {
    Full,       // Always store as full content
    Diff,       // Always use unified diff
    Chunk,      // Always use CDC chunking
}

/// Secret redaction result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckResult {
    Allowed,
    Denied {
        fingerprint: Digest,
        reason: DenialReason,
        matches: Vec<String>,
    },
}

/// Denial reason for blocked content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DenialReason {
    Secret,
    Pii,
    Malware,
    Size,
    Policy,
}
