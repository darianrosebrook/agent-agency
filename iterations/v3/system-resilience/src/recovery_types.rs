//! Recovery and resilience types for system-resilience crate

use schemars::JsonSchema;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

// Import from merkle module
use crate::merkle::AuthorInfo;

/// Content-addressable storage digest (BLAKE3 hash)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct Digest ([u8; 32]);

impl Digest {
    /// Create a digest from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Create a digest from hex string
    pub fn from_hex(hex: &str) -> Result<Self> {
        if hex.len() != 64 {
            return Err(anyhow::anyhow!("Invalid hex length: expected 64, got {}", hex.len()));
        }
        
        let mut bytes = [0u8; 32];
        for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
            if i >= 32 {
                break;
            }
            let hex_str = std::str::from_utf8(chunk)?;
            bytes[i] = u8::from_str_radix(hex_str, 16)?;
        }
        
        Ok(Self(bytes))
    }

    /// Get the digest as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Get the digest as hex string
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

impl std::fmt::Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Object reference for content-addressable storage
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ObjectRef {
    /// Content digest
    pub digest: Digest,
    /// Object size in bytes
    pub size: u64,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionMeta {
    /// Task ID associated with this session
    pub task_id: String,
    /// Iteration number
    pub iteration: u32,
    /// Agent ID (optional)
    pub agent_id: Option<String>,
    /// User ID (optional)
    pub user_id: Option<String>,
}

/// Session reference
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionRef {
    /// Session ID
    pub id: String,
    /// Session metadata
    pub meta: SessionMeta,
    /// Creation timestamp
    #[schemars(with = "String")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// File mode for recovery operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum FileMode {
    /// Regular file
    Regular,
    /// Executable file
    Executable,
    /// Symbolic link
    Symlink,
    /// Directory
    Directory,
}

impl Default for FileMode {
    fn default() -> Self {
        Self::Regular
    }
}

impl FileMode {
    /// Convert to POSIX permission bits
    pub fn to_posix(&self) -> u32 {
        match self {
            FileMode::Regular => 0o644,
            FileMode::Executable => 0o755,
            FileMode::Symlink => 0o777, // symlinks don't have restrictive perms
            FileMode::Directory => 0o755,
        }
    }

    /// Convert from POSIX permission bits
    pub fn from_posix(mode: u32) -> Self {
        // Check if it's a directory (bit 14 set)
        if mode & 0o40000 != 0 {
            FileMode::Directory
        }
        // Check if executable
        else if mode & 0o111 != 0 {
            FileMode::Executable
        }
        // Check if symlink (bit 13 set)
        else if mode & 0o120000 != 0 {
            FileMode::Symlink
        }
        else {
            FileMode::Regular
        }
    }

    /// Convert to mode bits for file system operations
    pub fn to_mode_bits(&self) -> Option<u32> {
        Some(self.to_posix())
    }
}

/// Restore action types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RestoreAction {
    /// Write a file
    WriteFile {
        path: PathBuf,
        mode: FileMode,
        expected: Digest,
        source: ObjectRef,
        size: u64,
    },
    /// Write a symbolic link
    WriteSymlink {
        path: PathBuf,
        target: String,
        size: u64,
    },
    /// Create a directory
    CreateDirectory {
        path: PathBuf,
    },
    /// Remove a file or directory
    Remove {
        path: PathBuf,
    },
}

impl RestoreAction {
    /// Get the path for this action
    pub fn path(&self) -> &PathBuf {
        match self {
            RestoreAction::WriteFile { path, .. } => path,
            RestoreAction::WriteSymlink { path, .. } => path,
            RestoreAction::CreateDirectory { path } => path,
            RestoreAction::Remove { path } => path,
        }
    }

    /// Get the size for this action
    pub fn size(&self) -> u64 {
        match self {
            RestoreAction::WriteFile { size, .. } => *size,
            RestoreAction::WriteSymlink { size, .. } => *size,
            RestoreAction::CreateDirectory { .. } => 0,
            RestoreAction::Remove { .. } => 0,
        }
    }

    /// Get the expected digest for this action
    pub fn expected_digest(&self) -> Option<&Digest> {
        match self {
            RestoreAction::WriteFile { expected, .. } => Some(expected),
            _ => None,
        }
    }

    /// Get the mode for this action
    pub fn mode(&self) -> Option<&FileMode> {
        match self {
            RestoreAction::WriteFile { mode, .. } => Some(mode),
            _ => None,
        }
    }
}

/// Restore plan
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RestorePlan {
    /// Actions to perform
    pub actions: Vec<RestoreAction>,
    /// Total number of files
    pub total_files: u32,
    /// Total size in bytes
    pub total_bytes: u64,
    /// Target identifier
    pub target: String,
}

/// Restore result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RestoreResult {
    /// Number of files restored
    pub files_restored: u32,
    /// Number of bytes restored
    pub bytes_restored: u64,
    /// Session ID (if applicable)
    pub session_id: Option<String>,
    /// Commit ID (if applicable)
    pub commit_id: Option<String>,
}

/// Restore filters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RestoreFilters {
    /// Include patterns
    pub include_patterns: Vec<String>,
    /// Exclude patterns
    pub exclude_patterns: Vec<String>,
    /// Maximum file size
    pub max_file_size: Option<u64>,
    /// File extensions to include
    pub include_extensions: Vec<String>,
    /// File extensions to exclude
    pub exclude_extensions: Vec<String>,
}

/// Payload kinds for blob storage
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PayloadKind {
    /// Full content
    Full,
    /// Unified diff
    UnifiedDiff,
    /// Chunk map
    ChunkMap,
}

/// Chunk reference
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChunkRef {
    /// Chunk digest
    pub digest: Digest,
    /// Chunk offset
    pub offset: u64,
    /// Chunk length (alias for size)
    pub length: u64,
    /// Chunk size
    pub size: u64,
}

impl ChunkRef {
    /// Get the length (alias for size)
    pub fn length(&self) -> u64 {
        self.size
    }
}

/// Blob storage entry
#[derive(Debug, Clone, JsonSchema)]
pub struct Blob {
    /// Blob header
    pub header: PayloadHeader,
    /// Blob data
    pub data: Vec<u8>,
}

impl Blob {
    /// Create a new blob
    pub fn new(header: PayloadHeader, data: Vec<u8>) -> Self {
        Self { header, data }
    }

    /// Get blob data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get blob content
    pub fn content(&self) -> &[u8] {
        &self.data
    }
}

/// Recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecoveryConfig {
    /// Enable recovery
    pub enabled: bool,
    /// Recovery directory
    pub recovery_dir: PathBuf,
    /// Maximum recovery size
    pub max_recovery_size: Option<u64>,
    /// Enable compression
    pub enable_compression: bool,
    /// Compression level
    pub compression_level: u32,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            recovery_dir: PathBuf::from("./.recovery"),
            max_recovery_size: Some(1024 * 1024 * 1024), // 1GB
            enable_compression: true,
            compression_level: 6,
        }
    }
}

/// Recovery statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct RecoveryStats {
    /// Total recoveries performed
    pub total_recoveries: u64,
    /// Successful recoveries
    pub successful_recoveries: u64,
    /// Failed recoveries
    pub failed_recoveries: u64,
    /// Total bytes recovered
    pub total_bytes_recovered: u64,
    /// Average recovery time (ms)
    pub avg_recovery_time_ms: u64,
    /// Last recovery timestamp
    pub last_recovery: Option<u64>,
}

/// Recovery error types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum RecoveryError {
    /// Object not found
    ObjectNotFound(Digest),
    /// Invalid digest
    InvalidDigest(String),
    /// Corrupted data
    CorruptedData(String),
    /// Insufficient space
    InsufficientSpace(u64),
    /// Permission denied
    PermissionDenied(String),
    /// Network error
    NetworkError(String),
    /// Timeout
    Timeout,
    /// Unknown error
    Unknown(String),
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryError::ObjectNotFound(digest) => {
                write!(f, "Object not found: {}", digest)
            }
            RecoveryError::InvalidDigest(msg) => {
                write!(f, "Invalid digest: {}", msg)
            }
            RecoveryError::CorruptedData(msg) => {
                write!(f, "Corrupted data: {}", msg)
            }
            RecoveryError::InsufficientSpace(size) => {
                write!(f, "Insufficient space: {} bytes needed", size)
            }
            RecoveryError::PermissionDenied(msg) => {
                write!(f, "Permission denied: {}", msg)
            }
            RecoveryError::NetworkError(msg) => {
                write!(f, "Network error: {}", msg)
            }
            RecoveryError::Timeout => {
                write!(f, "Operation timed out")
            }
            RecoveryError::Unknown(msg) => {
                write!(f, "Unknown error: {}", msg)
            }
        }
    }
}

impl std::error::Error for RecoveryError {}

/// Change identifier for tracking modifications
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct ChangeId (pub String);

impl ChangeId {
    /// Create a new change ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create from string
    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    /// Get as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ChangeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ChangeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Change statistics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ChangeStats {
    /// Number of files added
    pub files_added: u64,
    /// Number of files deleted
    pub files_deleted: u64,
    /// Number of lines added
    pub lines_added: u64,
    /// Number of lines removed
    pub lines_removed: u64,
    /// Total bytes added
    pub bytes_added: u64,
    /// Total bytes changed
    pub bytes_changed: u64,
    /// Deduplication ratio
    /// Number of files changed
    pub files_changed: u64,
    pub dedupe_ratio: f64,
    /// Change timestamp
    pub timestamp: Option<u64>,
}

impl Default for ChangeStats {
    fn default() -> Self {
        Self {
            files_added: 0,
            files_changed: 0,
            files_deleted: 0,
            lines_added: 0,
            lines_removed: 0,
            bytes_added: 0,
            bytes_changed: 0,
            dedupe_ratio: 0.0,
            timestamp: None,
        }
    }
}

/// Compression codec types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Codec {
    /// No compression
    None,
    /// Gzip compression
    Gzip,
    /// Zstandard compression
    Zstd,
}

impl Default for Codec {
    fn default() -> Self {
        Self::Zstd
    }
}

/// End of line types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Eol {
    /// Unix line ending (LF)
    Lf,
    /// Classic Mac line ending (CR)
    Cr,
    /// Windows line ending (CRLF)
    Crlf,
    /// Mixed line endings
    Mixed,
}

impl Default for Eol {
    fn default() -> Self {
        Self::Lf
    }
}

/// Streaming hasher for incremental hashing
#[derive(Debug)]
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

    /// Update with more data
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalize and get digest
    pub fn finalize(self) -> Digest {
        let hash = self.hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(hash.as_bytes());
        Digest(bytes)
    }
}

impl Default for StreamingHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Change payload types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ChangePayload {
    /// Full content
    Full(Vec<u8>),
    /// Unified diff with hunks
    UnifiedDiff {
        hunks: Vec<DiffHunk>,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    },
    /// Chunk-based changes
    ChunkMap(ChunkList),
}

/// Chunk list for chunk maps
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChunkList {
    /// Chunks in this list
    pub chunks: Vec<ChunkRef>,
    /// Total size of all chunks
    pub total_size: u64,
}

/// Diff hunk for unified diffs
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DiffHunk {
    /// Old start line
    pub old_start: u32,
    /// Old line count
    pub old_lines: u32,
    /// New start line
    pub new_start: u32,
    /// New line count
    pub new_lines: u32,
    /// Hunk content
    pub lines: Vec<String>,
    /// Line number mapping
    pub line_numbers: (usize, usize),
}

/// File change information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileChange {
    /// File path
    pub path: PathBuf,
    /// File mode
    pub mode: FileMode,
    /// Change type
    pub change_type: ChangeType,
    /// Old digest (if modified/deleted)
    pub old_digest: Option<Digest>,
    /// New digest (if added/modified)
    pub new_digest: Option<Digest>,
    /// Change payload
    pub payload: ChangePayload,
}

/// Type of change
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ChangeType {
    /// File added
    Added,
    /// File modified
    Modified,
    /// File deleted
    Deleted,
}

/// Change source types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ChangeSource {
    /// User-initiated change
    User,
    /// Automated change
    Automated,
    /// System change
    System,
    /// Agent iteration change
    AgentIteration {
        /// Iteration number
        iteration: u32,
        /// Agent identifier
        agent_id: String,
    },
    /// Human edit change
    HumanEdit {
        /// User identifier
        user_id: String,
    },
    /// System recovery change
    SystemRecovery,
    /// CAWS validation change
    CawsValidation,
}

/// Conflict classification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ConflictClass {
    /// User vs User conflict
    UserVsUser,
    /// User vs System conflict
    UserVsSystem,
    /// System vs System conflict
    SystemVsSystem,
    /// Validation vs System conflict
    ValidationVsSystem,
    /// Agent vs Agent conflict
    AgentVsAgent,
    /// Agent vs System conflict
    AgentVsSystem,
    /// Human vs Agent conflict
    HumanVsAgent,
    /// Human vs System conflict
    HumanVsSystem,
}

/// Payload header for blob storage
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PayloadHeader {
    /// Version of the payload format
    pub version: u32,
    /// Content length (alias for content_length)
    pub content_len: u32,
    /// Payload kind
    pub kind: PayloadKind,
    /// Content length
    pub content_length: u64,
    /// Creation timestamp
    pub created_at: u64,
    /// Compression algorithm (if any)
    pub compression: Option<String>,
    /// Codec used
    pub codec: Codec,
    /// End of line type
    pub eol: Eol,
}

impl Default for PayloadHeader {
    fn default() -> Self {
        Self {
            version: 1,
            content_len: 0,
            kind: PayloadKind::Full,
            content_length: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            compression: None,
            codec: Codec::Zstd,
            eol: Eol::Lf,
        }
    }
}

/// File restore action types
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum FileRestoreAction {
    /// Write a file
    WriteFile {
        path: PathBuf,
        mode: FileMode,
        expected: Digest,
        source: ObjectRef,
        size: u64,
    },
    /// Write a symbolic link
    WriteSymlink {
        path: PathBuf,
        target: String,
        size: u64,
    },
    /// Create a directory
    CreateDirectory {
        path: PathBuf,
    },
    /// Remove a file or directory
    Remove {
        path: PathBuf,
    },
    /// Delete a file
    DeleteFile {
        path: PathBuf,
        size: u64,
    },
    /// Change file mode
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
            FileRestoreAction::CreateDirectory { path } => path,
            FileRestoreAction::Remove { path } => path,
            FileRestoreAction::DeleteFile { path, .. } => path,
            FileRestoreAction::Chmod { path, .. } => path,
        }
    }

    /// Get the size for this action
    pub fn size(&self) -> u64 {
        match self {
            FileRestoreAction::WriteFile { size, .. } => *size,
            FileRestoreAction::WriteSymlink { size, .. } => *size,
            FileRestoreAction::CreateDirectory { .. } => 0,
            FileRestoreAction::Remove { .. } => 0,
            FileRestoreAction::DeleteFile { size, .. } => *size,
            FileRestoreAction::Chmod { size, .. } => *size,
        }
    }

    /// Get the expected digest for this action
    pub fn expected_digest(&self) -> Option<&Digest> {
        match self {
            FileRestoreAction::WriteFile { expected, .. } => Some(expected),
            _ => None,
        }
    }

    /// Get the mode for this action
    pub fn mode(&self) -> Option<&FileMode> {
        match self {
            FileRestoreAction::WriteFile { mode, .. } => Some(mode),
            FileRestoreAction::Chmod { mode, .. } => Some(mode),
            _ => None,
        }
    }
}

/// Commit identifier for versioning
pub type CommitId = Digest;

/// Journal record types for write-ahead logging
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum JournalRecord {
    /// Begin change operation
    Begin {
        change_id: ChangeId,
        path: PathBuf,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Commit change operation
    Commit {
        change_id: ChangeId,
        digest: Digest,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Deny change operation
    Denied {
        change_id: ChangeId,
        reason: DenialReason,
        fingerprint: Digest,
        #[schemars(with = "String")]
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Reason for denying a change
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum DenialReason {
    /// File too large
    FileTooLarge,
    /// Permission denied
    PermissionDenied,
    /// Invalid content
    InvalidContent,
    /// Policy violation
    PolicyViolation,
    /// System busy
    SystemBusy,
}

/// Recovery metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecoveryMetrics {
    /// Total recoveries performed
    pub total_recoveries: u64,
    /// Successful recoveries
    pub successful_recoveries: u64,
    /// Failed recoveries
    pub failed_recoveries: u64,
    /// Total bytes recovered
    pub total_bytes_recovered: u64,
    /// Average recovery time in milliseconds
    pub avg_recovery_time_ms: u64,
    /// Last recovery timestamp
    pub last_recovery: Option<u64>,
    /// Deduplication ratio
    pub dedupe_ratio: f64,
    /// Diff ratio
    pub diff_ratio: f64,
    /// P50 restore latency in milliseconds
    pub restore_latency_p50_ms: u64,
    /// P95 restore latency in milliseconds
    pub restore_latency_p95_ms: u64,
    /// Conflict rate
    pub conflict_rate: f64,
    /// Redaction hits
    pub redaction_hits: u64,
    /// Garbage collected MB
    pub gc_freed_mb: u64,
    /// Pack efficiency
    pub pack_efficiency: f64,
    /// Budget usage percentage
    pub budget_usage_pct: f64,
}

impl Default for RecoveryMetrics {
    fn default() -> Self {
        Self {
            total_recoveries: 0,
            successful_recoveries: 0,
            failed_recoveries: 0,
            total_bytes_recovered: 0,
            avg_recovery_time_ms: 0,
            last_recovery: None,
            dedupe_ratio: 0.0,
            diff_ratio: 0.0,
            restore_latency_p50_ms: 0,
            restore_latency_p95_ms: 0,
            conflict_rate: 0.0,
            redaction_hits: 0,
            gc_freed_mb: 0,
            pack_efficiency: 0.0,
            budget_usage_pct: 0.0,
        }
    }
}

/// Recovery result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RecoveryResult {
    /// Whether recovery was successful
    pub success: bool,
    /// Number of objects recovered
    pub objects_recovered: u64,
    /// Total bytes recovered
    pub bytes_recovered: u64,
    /// Recovery duration (ms)
    pub duration_ms: u64,
    /// Errors encountered
    pub errors: Vec<RecoveryError>,
    /// Warnings
    pub warnings: Vec<String>,
}

impl Default for RecoveryResult {
    fn default() -> Self {
        Self {
            success: true,
            objects_recovered: 0,
            bytes_recovered: 0,
            duration_ms: 0,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_creation() {
        let bytes = [1u8; 32];
        let digest = Digest::from_bytes(bytes);
        assert_eq!(digest.as_bytes(), &bytes);
    }

    #[test]
    fn test_digest_hex() {
        let bytes = [0u8; 32];
        let digest = Digest::from_bytes(bytes);
        assert_eq!(digest.to_hex(), "0".repeat(64));
    }

    #[test]
    fn test_object_ref() {
        let digest = Digest::from_bytes([2u8; 32]);
        let obj_ref = ObjectRef {
            digest,
            size: 1024,
        };
        assert_eq!(obj_ref.size, 1024);
    }

    #[test]
    fn test_session_meta() {
        let meta = SessionMeta {
            task_id: "task1".to_string(),
            iteration: 1,
            agent_id: Some("agent1".to_string()),
            user_id: Some("user1".to_string()),
        };
        assert_eq!(meta.task_id, "task1");
        assert_eq!(meta.iteration, 1);
    }

    #[test]
    fn test_restore_action() {
        let digest = Digest::from_bytes([3u8; 32]);
        let obj_ref = ObjectRef {
            digest: digest.clone(),
            size: 512,
        };
        
        let action = RestoreAction::WriteFile {
            path: PathBuf::from("test.txt"),
            mode: FileMode::Regular,
            expected: digest,
            source: obj_ref,
            size: 512,
        };
        
        assert_eq!(action.size(), 512);
        assert_eq!(action.path(), &PathBuf::from("test.txt"));
    }
}

/// Represents a commit in the recovery system
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Commit {
    pub id: Digest,
    pub parent: Option<Digest>,
    pub tree: Digest,
    pub session_id: String,
    pub caws_verdict_id: Option<String>,
    pub message: Option<String>,
    pub stats: ChangeStats,
    #[schemars(with = "String")]

    pub timestamp: DateTime<Utc>,
    pub author: AuthorInfo,
}

impl Default for Commit {
    fn default() -> Self {
        Self {
            id: Digest::from_bytes([0u8; 32]),
            parent: None,
            tree: Digest::from_bytes([0u8; 32]),
            session_id: String::new(),
            caws_verdict_id: None,
            message: None,
            stats: ChangeStats::default(),
            timestamp: Utc::now(),
            author: AuthorInfo::default(),
        }
    }
}
