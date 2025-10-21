//! Unified Workspace Manager (Tool Boundary)
//!
//! Single choke point for all file operations. Enforces scope safety,
//! budget limits, and atomic changes. Never relax these invariants.
//!
//! @author @darianrosebrook

use crate::sandbox::{GitWorktree, SnapshotManager, WorkspaceBackend};
use crate::types::{ChangeSet, ChangeSetReceipt, FileChange, ChangeOperation};
use crate::caws::BudgetChecker;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("Path not in allow-list: {0}")]
    OutOfScope(PathBuf),

    #[error("Budget exceeded: {current:?} vs {limit:?}")]
    BudgetExceeded {
        current: crate::caws::BudgetState,
        proposed: crate::caws::BudgetState,
        limit: crate::caws::BudgetLimits,
    },

    #[error("File operation failed: {0}")]
    FileOperation(#[from] std::io::Error),

    #[error("Git operation failed: {0}")]
    GitOperation(String),

    #[error("Snapshot operation failed: {0}")]
    SnapshotOperation(String),

    #[error("Atomic apply failed: {operation} - {reason}")]
    AtomicApplyFailed { operation: String, reason: String },
}

/// Unified workspace manager with single tool boundary
///
/// **INVARIANT**: All file modifications go through `apply_changes()`
/// **INVARIANT**: Budget checks happen at apply time, not upstream
/// **INVARIANT**: Scope validation is enforced here
/// **INVARIANT**: Operations are atomic (all-or-nothing)
pub struct WorkspaceManager {
    backend: WorkspaceBackend,
    allow_list: Vec<PathBuf>,
    budget_checker: BudgetChecker,
    workspace_root: PathBuf,
}

impl WorkspaceManager {
    /// Create workspace manager by auto-detecting Git vs non-Git
    pub async fn auto_detect(workspace_root: PathBuf) -> Result<Self, WorkspaceError> {
        // Check if workspace is a Git repository
        let git_dir = workspace_root.join(".git");
        let is_git = git_dir.exists() && git_dir.is_dir();

        let backend = if is_git {
            // Use Git worktree backend
            let worktree = GitWorktree::new(&workspace_root)
                .await
                .map_err(|e| WorkspaceError::GitOperation(e.to_string()))?;
            WorkspaceBackend::Git(worktree)
        } else {
            // Use snapshot backend
            let snapshot_manager = SnapshotManager::new(workspace_root)
                .map_err(|e| WorkspaceError::SnapshotOperation(e.to_string()))?;
            WorkspaceBackend::Snapshot(snapshot_manager)
        };

        // Initialize with empty allow-list (must be set by caller)
        let allow_list = Vec::new();
        let budget_checker = BudgetChecker::new(0, 0); // Default budgets

        Ok(Self {
            backend,
            allow_list,
            budget_checker,
            workspace_root,
        })
    }

    /// Set the allow-list for file operations
    pub fn set_allow_list(&mut self, allow_list: Vec<PathBuf>) {
        self.allow_list = allow_list;
    }

    /// Set budget limits
    pub fn set_budgets(&mut self, max_files: usize, max_loc: usize) {
        self.budget_checker = BudgetChecker::new(max_files, max_loc);
    }

    /// **TOOL BOUNDARY**: Single entry point for all file modifications
    ///
    /// **INVARIANTS ENFORCED**:
    /// - All paths must be in allow-list
    /// - Budget limits cannot be exceeded
    /// - Operations are atomic (temp write + fsync + rename)
    /// - No partial writes on failure
    pub async fn apply_changes(
        &mut self,
        changeset: ChangeSet,
    ) -> Result<ChangeSetReceipt, WorkspaceError> {
        // INVARIANT 1: Validate all paths in allow-list
        for change in &changeset.changes {
            if !self.is_path_allowed(&change.path) {
                return Err(WorkspaceError::OutOfScope(change.path.clone()));
            }
        }

        // INVARIANT 2: Check budgets (fail fast, no writes if exceeded)
        if self.budget_checker.would_exceed(&changeset)? {
            return Err(WorkspaceError::BudgetExceeded {
                current: self.budget_checker.current_state()?,
                proposed: self.budget_checker.projected_state(&changeset)?,
                limit: self.budget_checker.limits(),
            });
        }

        // INVARIANT 3: Atomic apply with transactional budget update
        // Save budget state for potential rollback
        let old_budget_state = self.budget_checker.current_state()?;

        let receipt = match &mut self.backend {
            WorkspaceBackend::Git(worktree) => {
                worktree.apply_changes_atomic(changeset).await
                    .map_err(|e| WorkspaceError::GitOperation(e.to_string()))?
            }
            WorkspaceBackend::Snapshot(snapshot) => {
                snapshot.apply_changes_atomic(changeset).await
                    .map_err(|e| WorkspaceError::SnapshotOperation(e.to_string()))?
            }
        };

        // INVARIANT 4: Update budget tracker transactionally
        // If budget update fails, rollback the applied changes
        if let Err(budget_err) = self.budget_checker.record_changes(&receipt) {
            // Budget update failed - rollback the applied changes
            let rollback_result = match &mut self.backend {
                WorkspaceBackend::Git(worktree) => {
                    worktree.rollback_to_snapshot(&receipt.checkpoint_id).await
                }
                WorkspaceBackend::Snapshot(snapshot) => {
                    snapshot.rollback_to_snapshot(&receipt.checkpoint_id).await
                }
            };

            if let Err(rollback_err) = rollback_result {
                // Both budget update and rollback failed - we're in a bad state
                return Err(WorkspaceError::AtomicApplyFailed {
                    operation: "budget_update_and_rollback".to_string(),
                    reason: format!("Budget update failed: {}, rollback also failed: {}", budget_err, rollback_err),
                });
            }

            // Restore old budget state
            self.budget_checker.restore_state(old_budget_state).await?;

            return Err(WorkspaceError::BudgetUpdateFailed(budget_err.to_string()));
        }

        Ok(receipt)
    }

    /// Create a checkpoint (Git commit or snapshot)
    pub async fn create_checkpoint(&self, label: &str) -> Result<String, WorkspaceError> {
        match &self.backend {
            WorkspaceBackend::Git(worktree) => {
                worktree.create_checkpoint(label).await
                    .map_err(|e| WorkspaceError::GitOperation(e.to_string()))
            }
            WorkspaceBackend::Snapshot(snapshot) => {
                snapshot.create_checkpoint(label).await
                    .map_err(|e| WorkspaceError::SnapshotOperation(e.to_string()))
            }
        }
    }

    /// Rollback to a checkpoint
    pub async fn rollback_to_checkpoint(&self, checkpoint_id: &str) -> Result<(), WorkspaceError> {
        match &self.backend {
            WorkspaceBackend::Git(worktree) => {
                worktree.rollback_to_checkpoint(checkpoint_id).await
                    .map_err(|e| WorkspaceError::GitOperation(e.to_string()))
            }
            WorkspaceBackend::Snapshot(snapshot) => {
                snapshot.rollback_to_checkpoint(checkpoint_id).await
                    .map_err(|e| WorkspaceError::SnapshotOperation(e.to_string()))
            }
        }
    }

    /// Get current budget state
    pub async fn budget_state(&self) -> Result<crate::caws::BudgetState, WorkspaceError> {
        self.budget_checker.current_state().await
            .map_err(|e| WorkspaceError::AtomicApplyFailed {
                operation: "budget_check".to_string(),
                reason: e.to_string(),
            })
    }

    /// Check if a path is allowed (helper for validation)
    fn is_path_allowed(&self, path: &Path) -> bool {
        // Normalize path to be relative to workspace root
        let normalized = match path.strip_prefix(&self.workspace_root) {
            Ok(relative) => relative,
            Err(_) => path, // Path is already relative
        };

        self.allow_list.iter().any(|allowed| {
            normalized.starts_with(allowed) || normalized == allowed.as_path()
        })
    }

    /// Get workspace root path
    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    /// Get allow-list (for inspection/debugging)
    pub fn allow_list(&self) -> &[PathBuf] {
        &self.allow_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use tokio::fs;

    #[tokio::test]
    async fn test_workspace_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let workspace_root = temp_dir.path().to_path_buf();

        // Create non-Git workspace
        let manager = WorkspaceManager::auto_detect(workspace_root).await;
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        match manager.backend {
            WorkspaceBackend::Snapshot(_) => {} // Expected for non-Git
            WorkspaceBackend::Git(_) => panic!("Should not auto-detect Git in empty dir"),
        }
    }

    #[tokio::test]
    async fn test_allow_list_enforcement() {
        let temp_dir = tempdir().unwrap();
        let workspace_root = temp_dir.path().to_path_buf();

        let mut manager = WorkspaceManager::auto_detect(workspace_root).await.unwrap();
        manager.set_allow_list(vec![PathBuf::from("src/")]);

        // Create test file outside allow-list
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "content").await.unwrap();

        let changeset = ChangeSet {
            changes: vec![FileChange {
                path: test_file.clone(),
                operation: ChangeOperation::Modify {
                    expected_content: "content".to_string(),
                    new_content: "modified".to_string(),
                },
            }],
        };

        // Should fail due to allow-list violation
        let result = manager.apply_changes(changeset).await;
        assert!(matches!(result, Err(WorkspaceError::OutOfScope(_))));

        // Now add to allow-list
        manager.set_allow_list(vec![PathBuf::from("test.txt")]);

        let changeset = ChangeSet {
            changes: vec![FileChange {
                path: test_file,
                operation: ChangeOperation::Modify {
                    expected_content: "content".to_string(),
                    new_content: "modified".to_string(),
                },
            }],
        };

        // Should work now
        let result = manager.apply_changes(changeset).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_budget_enforcement() {
        let temp_dir = tempdir().unwrap();
        let workspace_root = temp_dir.path().to_path_buf();

        let mut manager = WorkspaceManager::auto_detect(workspace_root).await.unwrap();
        manager.set_allow_list(vec![PathBuf::from("src/")]);
        manager.set_budgets(1, 10); // Max 1 file, 10 LOC

        // Create test file
        let test_file = temp_dir.path().join("src/test.txt");
        fs::create_dir_all(test_file.parent().unwrap()).await.unwrap();
        fs::write(&test_file, "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11").await.unwrap();

        let changeset = ChangeSet {
            changes: vec![FileChange {
                path: test_file,
                operation: ChangeOperation::Create {
                    content: "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8\nline9\nline10\nline11".to_string(),
                },
            }],
        };

        // Should fail due to LOC budget (11 > 10)
        let result = manager.apply_changes(changeset).await;
        assert!(matches!(result, Err(WorkspaceError::BudgetExceeded { .. })));
    }
}
