//! File Operations Interface
//!
//! Defines interfaces for safe file operations with version control,
//! rollback capabilities, and security controls.
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use std::path::Path;
use serde::{Deserialize, Serialize};

/// Result type for file operations
pub type FileResult<T> = Result<T, FileOpsError>;

/// Errors that can occur during file operations
#[derive(thiserror::Error, Debug)]
pub enum FileOpsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path error: {0}")]
    Path(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Permission denied: {0}")]
    Permission(String),

    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("Changeset error: {0}")]
    Changeset(String),
}

/// Unique identifier for a changeset
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChangesetId(pub String);

/// File operation patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    /// Path to the file being modified
    pub path: String,
    /// Hunks of changes to apply
    pub hunks: Vec<Hunk>,
}

/// A hunk represents a contiguous block of changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hunk {
    /// Line number where old content starts (1-indexed)
    pub old_start: usize,
    /// Number of lines of old content
    pub old_lines: usize,
    /// Line number where new content starts (1-indexed)
    pub new_start: usize,
    /// Number of lines of new content
    pub new_lines: usize,
    /// Content lines (prefixed with +/-/space)
    pub lines: String,
}

/// A changeset containing multiple file patches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changeset {
    /// Unique identifier
    pub id: ChangesetId,
    /// Description of the changeset
    pub description: String,
    /// Patches to apply
    pub patches: Vec<Patch>,
    /// Metadata about the changeset
    pub metadata: ChangesetMetadata,
}

/// Metadata for a changeset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangesetMetadata {
    /// Author of the changes
    pub author: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Risk tier (1-3, where 1 is highest risk)
    pub risk_tier: u8,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Allow list for file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowList {
    /// Allowed file patterns (glob patterns)
    pub allowed_patterns: Vec<String>,
    /// Blocked file patterns
    pub blocked_patterns: Vec<String>,
    /// Maximum file size in bytes
    pub max_file_size: Option<u64>,
    /// Maximum total changeset size in bytes
    pub max_changeset_size: Option<u64>,
}

/// Budget limits for file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budgets {
    /// Maximum number of files that can be modified
    pub max_files: Option<usize>,
    /// Maximum number of lines that can be changed
    pub max_lines: Option<usize>,
    /// Maximum time allowed for operation
    pub max_time_seconds: Option<u64>,
}

/// Workspace interface for file operations
#[async_trait]
pub trait Workspace: Send + Sync {
    /// Get the root path of the workspace
    fn root(&self) -> &Path;

    /// Apply a changeset to the workspace
    async fn apply(
        &self,
        changeset: &Changeset,
        allowlist: &AllowList,
        budgets: &Budgets,
    ) -> FileResult<ChangesetId>;

    /// Revert a changeset
    async fn revert(&self, changeset_id: &ChangesetId) -> FileResult<()>;

    /// Promote workspace changes to the main repository
    async fn promote(&self) -> FileResult<()>;
}

/// Workspace factory for creating workspaces
#[async_trait]
pub trait WorkspaceFactory: Send + Sync {
    /// Create a new workspace for the given task
    async fn create_workspace(
        &self,
        task_id: &str,
        repo_path: &Path,
    ) -> FileResult<Box<dyn Workspace>>;
}

/// File operations service interface
#[async_trait]
pub trait FileOperationsService: Send + Sync + std::fmt::Debug {
    /// Validate a changeset against security and budget constraints
    async fn validate_changeset(
        &self,
        changeset: &Changeset,
        allowlist: &AllowList,
        budgets: &Budgets,
    ) -> FileResult<()>;

    /// Create a new workspace for file operations
    async fn create_workspace(
        &self,
        task_id: &str,
        repo_path: &Path,
    ) -> FileResult<Box<dyn Workspace>>;

    /// Get workspace status
    async fn get_workspace_status(&self, task_id: &str) -> FileResult<WorkspaceStatus>;
}

/// Status of a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceStatus {
    /// Task ID
    pub task_id: String,
    /// Current state
    pub state: WorkspaceState,
    /// Active changeset if any
    pub active_changeset: Option<ChangesetId>,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// State of a workspace
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum WorkspaceState {
    /// Workspace is being initialized
    Initializing,
    /// Workspace is ready for operations
    Ready,
    /// Workspace has active operations
    Active,
    /// Workspace is being committed
    Committing,
    /// Workspace encountered an error
    Error,
    /// Workspace is being cleaned up
    CleaningUp,
    /// Workspace is destroyed
    Destroyed,
}
