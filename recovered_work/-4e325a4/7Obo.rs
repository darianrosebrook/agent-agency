//! Sandbox environment for safe file editing with Git snapshots

pub mod diff_applier;
pub mod diff_generator;
pub mod file_guard;
pub mod git_worktree;
pub mod snapshot;
pub mod workspace_manager;

pub use diff_applier::DiffApplier;
pub use diff_generator::DiffGenerator;
pub use file_guard::FileGuard;
pub use git_worktree::GitWorktree;
pub use snapshot::SnapshotManager;
pub use workspace_manager::WorkspaceManager;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{UnifiedDiff, Artifact, ArtifactType};

/// Safety modes for sandbox operations
#[derive(Debug, Clone, PartialEq)]
pub enum SafetyMode {
    Strict,      // No file operations allowed
    Sandbox,     // Limited operations within workspace
    Autonomous,  // Full autonomous operations
}

/// Backend types for workspace management
#[derive(Debug)]
pub enum WorkspaceBackend {
    Git(GitWorktree),
    Snapshot(SnapshotManager),
}

/// Operations that can be performed in the sandbox
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxOperation {
    ReadFile,
    WriteFile,
    CreateFile,
    DeleteFile,
    ApplyDiff,
    CreateSnapshot,
    Rollback,
}


/// Result of applying a diff
#[derive(Debug, Clone)]
pub struct ApplyResult {
    pub iteration: usize,
    pub files_modified: Vec<String>,
    pub snapshot_id: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Sandbox environment for controlled file editing
pub struct SandboxEnvironment {
    workspace_root: PathBuf,
    allowed_paths: Vec<PathBuf>,
    safety_mode: SafetyMode,
    git_worktree: Option<GitWorktree>,
    file_guard: FileGuard,
    diff_applier: DiffApplier,
    snapshot_manager: SnapshotManager,
}

impl SandboxEnvironment {
    /// Create a new sandbox environment
    pub async fn new(
        workspace_root: PathBuf,
        allowed_paths: Vec<PathBuf>,
        safety_mode: SafetyMode,
        use_git: bool,
    ) -> Result<Self, SandboxError> {
        // Ensure workspace exists
        tokio::fs::create_dir_all(&workspace_root).await
            .map_err(|e| SandboxError::IoError(e))?;

        // Initialize Git worktree if requested
        let git_worktree = if use_git {
            Some(GitWorktree::new(workspace_root.clone()).await?)
        } else {
            None
        };

        let file_guard = FileGuard::new(allowed_paths.clone());
        let diff_applier = DiffApplier::new(workspace_root.clone());
        let snapshot_manager = SnapshotManager::new(workspace_root.clone());

        Ok(Self {
            workspace_root,
            allowed_paths,
            safety_mode,
            git_worktree,
            file_guard,
            diff_applier,
            snapshot_manager,
        })
    }

    /// Apply a unified diff to the workspace
    pub async fn apply_diff(&mut self, diff: &UnifiedDiff, iteration: usize) -> Result<ApplyResult, SandboxError> {
        // Validate diff against allowed paths
        self.validate_diff_scope(diff).await?;

        // Check safety mode
        match self.safety_mode {
            SafetyMode::Strict => {
                return Err(SandboxError::ApprovalRequired(diff.file_path.clone()));
            }
            SafetyMode::Sandbox | SafetyMode::Autonomous => {
                // Proceed with application
            }
        }

        // Apply the diff
        let modified_files = self.diff_applier.apply_diff(diff, false).await?;

        // Create git snapshot if enabled
        let snapshot_id = if let Some(ref git) = self.git_worktree {
            Some(git.commit_snapshot(&format!("Iteration {}", iteration)).await?)
        } else {
            None
        };

        let result = ApplyResult {
            iteration,
            files_modified: modified_files,
            snapshot_id,
            timestamp: Utc::now(),
        };

        Ok(result)
    }

    /// Rollback to a specific snapshot
    pub async fn rollback_to_snapshot(&mut self, snapshot_id: &str) -> Result<(), SandboxError> {
        if let Some(ref git) = self.git_worktree {
            git.rollback_to_snapshot(snapshot_id).await?;
        } else {
            self.snapshot_manager.rollback_to_snapshot(snapshot_id).await?;
        }
        Ok(())
    }

    /// Validate that a diff only touches allowed paths
    pub async fn validate_diff_scope(&self, diff: &UnifiedDiff) -> Result<(), SandboxError> {
        self.file_guard.validate_path(&diff.file_path)?;
        Ok(())
    }

    /// Get the workspace root path
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Check if a path is allowed
    pub fn is_path_allowed(&self, path: &Path) -> bool {
        self.file_guard.is_allowed(path)
    }
}

/// Errors that can occur in the sandbox
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Git error: {0}")]
    GitError(String),

    #[error("Diff application error: {0}")]
    DiffError(String),

    #[error("Path not allowed: {0}")]
    PathNotAllowed(String),

    #[error("Approval required for file: {0}")]
    ApprovalRequired(String),

    #[error("Snapshot error: {0}")]
    SnapshotError(String),

    #[error("Budget update failed: {0}")]
    BudgetUpdateFailed(String),
}
