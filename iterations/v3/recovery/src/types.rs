//! Core types for the V3 Recovery System
//!
//! @author @darianrosebrook

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// Import AuthorInfo from merkle module
use crate::merkle::AuthorInfo;

/// BLAKE3 digest wrapper for content addressing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Digest(pub [u8; 32]);

impl Digest {
    /// Create a new digest from BLAKE3 hash
    pub fn from_blake3(hash: blake3::Hash) -> Self {
        Self(*hash.as_bytes())
    }

    /// Create a new digest from bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get the digest as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Create from hex string
    pub fn from_hex(hex: &str) -> anyhow::Result<Self> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != 32 {
            return Err(anyhow::anyhow!("Invalid digest length"));
        }
        let mut digest_bytes = [0u8; 32];
        digest_bytes.copy_from_slice(&bytes);
        Ok(Self(digest_bytes))
    }
}

impl std::fmt::Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Streaming hasher for large content
pub struct StreamingHasher {
    hasher: blake3::Hasher,
}

impl StreamingHasher {
    /// Create a new streaming hasher
    pub fn new() -> Self {
        Self {
            hasher: blake3::Hasher::new(),
        }
    }

    /// Update the hasher with more data
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalize the hash and return the digest
    pub fn finalize(self) -> Digest {
        Digest::from_blake3(self.hasher.finalize())
    }
}

impl Default for StreamingHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Payload header for content addressing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayloadHeader {
    pub version: u8,         // Start at 1
    pub kind: PayloadKind,   // Full | UnifiedDiff | ChunkMap
    pub codec: Codec,        // zstd | gzip | none
    pub eol: Option<Eol>,    // LF | CRLF | Mixed
    pub content_len: u32,    // Uncompressed size
}

/// Payload type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayloadKind {
    Full,
    UnifiedDiff,
    ChunkMap,
}

/// Compression codec
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Codec {
    None,
    Zstd,
    Gzip,
}

/// End-of-line normalization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Copy)]
pub enum Eol {
    Lf,     // Unix
    Crlf,   // Windows
    Cr,     // Legacy Mac
    Mixed,  // Mixed line endings
}

/// File mode for POSIX compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FileMode {
    #[default]
    Regular,    // 100644
    Executable, // 100755
    Symlink,    // 120000
}


impl FileMode {
    /// Get the POSIX mode bits
    pub fn to_posix(&self) -> u32 {
        match self {
            FileMode::Regular => 0o100644,
            FileMode::Executable => 0o100755,
            FileMode::Symlink => 0o120000,
        }
    }

    /// Get the mode bits for file permissions
    pub fn to_mode_bits(&self) -> Option<u32> {
        match self {
            FileMode::Regular => Some(0o644),
            FileMode::Executable => Some(0o755),
            FileMode::Symlink => None, // Symlinks don't have mode bits
        }
    }

    /// Create from POSIX mode bits
    pub fn from_posix(mode: u32) -> Self {
        match mode & 0o170000 {
            0o100000 => {
                if mode & 0o111 != 0 {
                    FileMode::Executable
                } else {
                    FileMode::Regular
                }
            }
            0o120000 => FileMode::Symlink,
            _ => FileMode::Regular, // Default fallback
        }
    }
}

/// Change payload variants
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangePayload {
    Full(Vec<u8>),              // < 2 KiB files
    UnifiedDiff {
        base_commit: Digest,     // Explicit lineage
        base_digest: Digest,
        after_digest: Digest,
        hunks: Vec<DiffHunk>,
        metadata: DiffMetadata,
    },
    ChunkMap(ChunkList),        // CDC chunks
}

/// Unified diff hunk
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffHunk {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    pub lines: Vec<String>,
}

/// Diff metadata for text normalization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffMetadata {
    pub eol: Eol,
    pub encoding: String,
}

/// CDC chunk list
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkList {
    pub chunks: Vec<ChunkInfo>,
    pub total_size: u64,
}

/// Individual chunk information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub digest: Digest,
    pub offset: u64,
    pub length: u32,
}

/// File change record
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChange {
    pub path: PathBuf,
    pub mode: FileMode,
    pub precondition: Option<Digest>,   // Optimistic concurrency
    pub payload: ChangePayload,
    pub source: ChangeSource,
}

/// Change source attribution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeSource {
    AgentIteration {
        iteration: u32,
        agent_id: String,
    },
    HumanEdit {
        user_id: String,
    },
    SystemRecovery {
        session: String,
    },
    CawsValidation {
        verdict_id: String,
    },
}

/// Commit object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Commit {
    pub id: Digest,
    pub parent: Option<Digest>,
    pub tree: Digest,                   // Merkle root (trees are authoritative)
    pub session_id: String,
    pub caws_verdict_id: Option<String>,
    pub message: Option<String>,
    pub stats: ChangeStats,             // Observability
    pub timestamp: DateTime<Utc>,
    pub author: AuthorInfo,             // Author information
}

/// Change statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeStats {
    pub files_added: u32,
    pub files_changed: u32,
    pub files_deleted: u32,
    pub bytes_added: u64,
    pub bytes_changed: u64,
    pub dedupe_ratio: f64,
}

/// Session metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionMeta {
    pub task_id: String,
    pub iteration: u32,
    pub agent_id: Option<String>,
    pub user_id: Option<String>,
}

/// Session reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRef {
    pub id: String,
    pub meta: SessionMeta,
    pub created_at: DateTime<Utc>,
}

/// Change ID for tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChangeId(pub String);

impl Default for ChangeId {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl std::fmt::Display for ChangeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Commit ID
pub type CommitId = Digest;

/// Conflict classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictClass {
    NonOverlapping,
    Adjacent,
    Overlapping,
    Binary,
    AgentVsAgent,
    AgentVsSystem,
    HumanVsAgent,
    HumanVsSystem,
    SystemVsSystem,
    ValidationVsSystem,
}

/// Conflict object
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
    pub path: PathBuf,
    pub base: Digest,
    pub theirs: Digest,
    pub yours: ChangePayload,
    pub classification: ConflictClass,
}

/// File restore action
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileRestoreAction {
    WriteFile {
        path: PathBuf,
        mode: FileMode,
        expected: Digest,
        source: ObjectRef,
        size: u64,
    },
    WriteSymlink {
        path: PathBuf,
        target: String,
        size: u64,
    },
    DeleteFile {
        path: PathBuf,
        size: u64,
    },
    Chmod {
        path: PathBuf,
        mode: FileMode,
        size: u64,
    },
}

impl FileRestoreAction {
    /// Get the path for this action
    pub fn path(&self) -> &PathBuf {
        match self {
            FileRestoreAction::WriteFile { path, .. } => path,
            FileRestoreAction::WriteSymlink { path, .. } => path,
            FileRestoreAction::DeleteFile { path, .. } => path,
            FileRestoreAction::Chmod { path, .. } => path,
        }
    }

    /// Get the size for this action
    pub fn size(&self) -> u64 {
        match self {
            FileRestoreAction::WriteFile { size, .. } => *size,
            FileRestoreAction::WriteSymlink { size, .. } => *size,
            FileRestoreAction::DeleteFile { size, .. } => *size,
            FileRestoreAction::Chmod { size, .. } => *size,
        }
    }

    /// Get the expected digest for this action (if applicable)
    pub fn expected_digest(&self) -> Option<&Digest> {
        match self {
            FileRestoreAction::WriteFile { expected, .. } => Some(expected),
            _ => None,
        }
    }

    /// Get the mode for this action (if applicable)
    pub fn mode(&self) -> Option<&FileMode> {
        match self {
            FileRestoreAction::WriteFile { mode, .. } => Some(mode),
            FileRestoreAction::Chmod { mode, .. } => Some(mode),
            _ => None,
        }
    }
}

/// Object reference
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectRef {
    pub digest: Digest,
    pub size: u64,
}

/// Chunk reference for CDC
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkRef {
    pub digest: Digest,
    pub offset: u64,
    pub length: u32,
}

/// Restore filters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreFilters {
    pub globs: Vec<String>,
    pub since: Option<String>,
    pub until: Option<String>,
    pub include_deleted: bool,
}

/// Restore action (alias for FileRestoreAction)
pub type RestoreAction = FileRestoreAction;

/// Restore plan
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestorePlan {
    pub target: String,  // Ref or commit
    pub actions: Vec<FileRestoreAction>,
    pub total_files: u32,
    pub total_bytes: u64,
}

/// Restore result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreResult {
    pub files_restored: u32,
    pub bytes_restored: u64,
    pub session_id: Option<String>,
    pub commit_id: Option<CommitId>,
}

/// Recovery metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryMetrics {
    pub dedupe_ratio: f64,           // Unique / total bytes
    pub diff_ratio: f64,             // Avg diff size / full size
    pub restore_latency_p50_ms: u64,
    pub restore_latency_p95_ms: u64,
    pub conflict_rate: f64,          // Conflicts / writes
    pub redaction_hits: u64,
    pub gc_freed_mb: u64,
    pub pack_efficiency: f64,        // Packed / original
    pub budget_usage_pct: f64,       // Current / budget
}
