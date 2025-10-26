/**
 * @fileoverview Core types for workspace state management
 * @author @darianrosebrook
 */
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for a workspace state snapshot
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateId(pub Uuid);

impl StateId {
    /// Generate a new unique state ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for StateId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for StateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a file in the workspace with its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    /// Path relative to workspace root
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// SHA-256 hash of file contents
    pub content_hash: String,
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
    /// File permissions (Unix-style)
    pub permissions: u32,
    /// Whether this file is tracked by git
    pub git_tracked: bool,
    /// Git commit hash if tracked
    pub git_commit: Option<String>,
    /// File content (optional, for small files)
    pub content: Option<Vec<u8>>,
    /// Whether content is compressed
    pub compressed: bool,
}

/// Represents a directory in the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryState {
    /// Path relative to workspace root
    pub path: PathBuf,
    /// Number of files in this directory
    pub file_count: usize,
    /// Number of subdirectories
    pub subdirectory_count: usize,
    /// Total size of all files in this directory (recursive)
    pub total_size: u64,
    /// Last modified timestamp of most recent file
    pub last_modified: DateTime<Utc>,
}

/// Represents the complete state of a workspace at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    /// Unique identifier for this state
    pub id: StateId,
    /// Timestamp when this state was captured
    pub captured_at: DateTime<Utc>,
    /// Workspace root path
    pub workspace_root: PathBuf,
    /// Git commit hash at time of capture
    pub git_commit: Option<String>,
    /// Git branch name at time of capture
    pub git_branch: Option<String>,
    /// All files in the workspace
    pub files: HashMap<PathBuf, FileState>,
    /// All directories in the workspace
    pub directories: HashMap<PathBuf, DirectoryState>,
    /// Total number of files
    pub total_files: usize,
    /// Total workspace size in bytes
    pub total_size: u64,
    /// Metadata about the capture process
    pub metadata: CaptureMetadata,
    /// Legacy timestamp field for compatibility
    pub timestamp: DateTime<Utc>,
}

impl WorkspaceState {
    /// Estimate the size of this state in bytes for storage metrics
    pub fn estimated_size_bytes(&self) -> u64 {
        // Use the total_size field which already contains the workspace size
        // Add some overhead for metadata and structure
        self.total_size + 1024 // 1KB overhead for metadata
    }
}

/// Metadata about how a workspace state was captured
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureMetadata {
    /// Duration of capture process in milliseconds
    pub capture_duration_ms: u64,
    /// Number of files processed
    pub files_processed: usize,
    /// Number of directories processed
    pub directories_processed: usize,
    /// Whether git information was available
    pub git_available: bool,
    /// Any warnings or errors during capture
    pub warnings: Vec<String>,
    /// Capture method used
    pub method: CaptureMethod,
}

/// Method used to capture workspace state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaptureMethod {
    /// Full recursive scan of all files
    FullScan,
    /// Git-based capture using git ls-files
    GitBased,
    /// Incremental capture based on git diff
    Incremental,
    /// Hybrid approach combining git and filesystem
    Hybrid,
}

/// Represents changes between two workspace states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceDiff {
    /// Source state ID
    pub from_state: StateId,
    /// Target state ID
    pub to_state: StateId,
    /// Files that were added
    pub added_files: Vec<PathBuf>,
    /// Files that were removed
    pub removed_files: Vec<PathBuf>,
    /// Files that were modified
    pub modified_files: Vec<PathBuf>,
    /// Directories that were added
    pub added_directories: Vec<PathBuf>,
    /// Directories that were removed
    pub removed_directories: Vec<PathBuf>,
    /// Size change in bytes (positive = increase, negative = decrease)
    pub size_delta: i64,
    /// Number of files added
    pub files_added: usize,
    /// Number of files removed
    pub files_removed: usize,
    /// Number of files modified
    pub files_modified: usize,
    /// Timestamp when diff was computed
    pub computed_at: DateTime<Utc>,
    /// Legacy timestamp field for compatibility
    pub timestamp: DateTime<Utc>,
    /// Detailed changes for advanced diff operations
    pub changes: Vec<DiffChange>,
}

/// Represents a specific change in a diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffChange {
    /// File was added with content
    Add { path: PathBuf, content: Vec<u8> },
    /// File was removed
    Remove { path: PathBuf },
    /// File was modified
    Modify {
        path: PathBuf,
        old_content: Option<Vec<u8>>,
        new_content: Vec<u8>,
    },
    /// Directory was added
    AddDirectory { path: PathBuf },
    /// Directory was removed
    RemoveDirectory { path: PathBuf },
}

/// Configuration for workspace state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Whether to track git information
    pub track_git: bool,
    /// Whether to compute file hashes
    pub compute_hashes: bool,
    /// Maximum file size to track (in bytes)
    pub max_file_size: u64,
    /// File patterns to ignore (glob patterns)
    pub ignore_patterns: Vec<String>,
    /// Whether to compress stored states
    pub compress_states: bool,
    /// Maximum number of states to keep
    pub max_states: usize,
    /// Whether to track directory metadata
    pub track_directories: bool,
    /// Capture method to use by default
    pub default_capture_method: CaptureMethod,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            track_git: true,
            compute_hashes: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
            ignore_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/dist/**".to_string(),
                "**/.git/**".to_string(),
                "**/.*".to_string(), // Hidden files
            ],
            compress_states: true,
            max_states: 100,
            track_directories: true,
            default_capture_method: CaptureMethod::Hybrid,
        }
    }
}

/// Result of a workspace state operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceResult<T> {
    /// The result data
    pub data: T,
    /// Any warnings generated during the operation
    pub warnings: Vec<String>,
    /// Duration of the operation in milliseconds
    pub duration_ms: u64,
    /// Whether the operation was successful
    pub success: bool,
}

impl<T> WorkspaceResult<T> {
    /// Create a successful result
    pub fn success(data: T, duration_ms: u64) -> Self {
        Self {
            data,
            warnings: Vec::new(),
            duration_ms,
            success: true,
        }
    }

    /// Create a result with warnings
    pub fn with_warnings(data: T, warnings: Vec<String>, duration_ms: u64) -> Self {
        Self {
            data,
            warnings,
            duration_ms,
            success: true,
        }
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Error types for workspace state operations
#[derive(Debug, thiserror::Error)]
pub enum WorkspaceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("State not found: {0}")]
    StateNotFound(StateId),

    #[error("Invalid workspace path: {0}")]
    InvalidWorkspacePath(PathBuf),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Capture error: {0}")]
    Capture(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Diff computation error: {0}")]
    DiffComputation(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Storage timeout for state: {0}")]
    StorageTimeout(StateId),
}

/// Trait for workspace state storage backends
#[async_trait::async_trait]
pub trait StateStorage: Send + Sync {
    /// Store a workspace state
    async fn store_state(&self, state: &WorkspaceState) -> Result<(), WorkspaceError>;

    /// Retrieve a workspace state by ID
    async fn get_state(&self, id: StateId) -> Result<WorkspaceState, WorkspaceError>;

    /// List all stored state IDs
    async fn list_states(&self) -> Result<Vec<StateId>, WorkspaceError>;

    /// Delete a workspace state
    async fn delete_state(&self, id: StateId) -> Result<(), WorkspaceError>;

    /// Store a workspace diff
    async fn store_diff(&self, diff: &WorkspaceDiff) -> Result<(), WorkspaceError>;

    /// Retrieve a workspace diff
    async fn get_diff(&self, from: StateId, to: StateId) -> Result<WorkspaceDiff, WorkspaceError>;

    /// Clean up old states based on retention policy
    async fn cleanup(&self, max_states: usize) -> Result<usize, WorkspaceError>;

    /// Validate diff format and data integrity
    async fn validate_diff(&self, diff: &WorkspaceDiff) -> Result<(), WorkspaceError>;

    /// Validate a specific diff change
    async fn validate_diff_change(&self, change: &DiffChange) -> Result<(), WorkspaceError>;

    /// Update diff-related metrics
    async fn update_diff_metrics(&self) -> Result<(), WorkspaceError>;

    /// Clean up old diffs based on retention policy
    async fn cleanup_old_diffs(&self) -> Result<(), WorkspaceError>;
}
