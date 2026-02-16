//! Git Worktree Manager
//!
//! Provides isolated git branches for parallel agent task execution.
//! Each task gets its own worktree, changes are merged back on completion.
//!
//! Ported from v3 `worktree_manager.rs` (722 lines → ~250 lines, simplified).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::sync::RwLock;
use uuid::Uuid;

// ─── Types ───────────────────────────────────────────────────────────

/// Worktree manager configuration.
#[derive(Debug, Clone)]
pub struct WorktreeConfig {
    /// Base directory for worktrees.
    pub worktree_base_path: PathBuf,
    /// Path to the main repository.
    pub main_repo_path: PathBuf,
    /// Base branch to create worktrees from.
    pub base_branch: String,
    /// Maximum number of concurrent worktrees.
    pub max_concurrent: usize,
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            worktree_base_path: PathBuf::from("/tmp/v4-worktrees"),
            main_repo_path: PathBuf::from("."),
            base_branch: "main".to_string(),
            max_concurrent: 10,
        }
    }
}

/// Status of a worktree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorktreeStatus {
    Created,
    InUse,
    Merged,
    Conflict,
    CleanedUp,
}

/// Information about a managed worktree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    /// Unique worktree identifier.
    pub id: Uuid,
    /// Associated task ID.
    pub task_id: String,
    /// Associated worker ID.
    pub worker_id: String,
    /// Filesystem path to the worktree.
    pub path: PathBuf,
    /// Git branch name.
    pub branch: String,
    /// When the worktree was created.
    pub created_at: DateTime<Utc>,
    /// Current status.
    pub status: WorktreeStatus,
}

/// Result of merging a worktree back to the base branch.
#[derive(Debug, Clone)]
pub struct MergeResult {
    /// Whether the merge completed without conflicts.
    pub success: bool,
    /// Conflicting files (empty if success).
    pub conflicts: Vec<String>,
    /// Number of files changed.
    pub files_changed: usize,
}

/// Worktree errors.
#[derive(Debug, thiserror::Error)]
pub enum WorktreeError {
    #[error("Git error: {0}")]
    GitError(String),

    #[error("Max concurrent worktrees ({0}) reached")]
    CapacityExceeded(usize),

    #[error("Worktree not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// ─── WorktreeManager ─────────────────────────────────────────────────

/// Manages git worktrees for parallel agent execution.
pub struct WorktreeManager {
    config: WorktreeConfig,
    active: Arc<RwLock<HashMap<Uuid, WorktreeInfo>>>,
}

impl WorktreeManager {
    /// Create a new worktree manager.
    pub fn new(config: WorktreeConfig) -> Self {
        Self {
            config,
            active: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create an isolated worktree for a task.
    pub async fn create_worktree(
        &self,
        task_id: &str,
        worker_id: &str,
    ) -> Result<WorktreeInfo, WorktreeError> {
        let active = self.active.read().await;
        if active.len() >= self.config.max_concurrent {
            return Err(WorktreeError::CapacityExceeded(self.config.max_concurrent));
        }
        drop(active);

        let id = Uuid::new_v4();
        let id_prefix = &id.to_string()[..8];
        let branch = format!("worktree-{task_id}-{id_prefix}");
        let path = self.config.worktree_base_path.join(id_prefix);

        // Ensure base directory exists
        tokio::fs::create_dir_all(&self.config.worktree_base_path).await?;

        // git worktree add -b <branch> <path> <base_branch>
        let output = Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                &branch,
                path.to_str().unwrap_or(""),
                &self.config.base_branch,
            ])
            .current_dir(&self.config.main_repo_path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorktreeError::GitError(format!(
                "Failed to create worktree: {stderr}"
            )));
        }

        let info = WorktreeInfo {
            id,
            task_id: task_id.to_string(),
            worker_id: worker_id.to_string(),
            path,
            branch,
            created_at: Utc::now(),
            status: WorktreeStatus::Created,
        };

        self.active.write().await.insert(id, info.clone());
        Ok(info)
    }

    /// Merge a worktree branch back to the base branch.
    pub async fn merge_worktree(&self, worktree_id: Uuid) -> Result<MergeResult, WorktreeError> {
        let mut active = self.active.write().await;
        let info = active
            .get_mut(&worktree_id)
            .ok_or_else(|| WorktreeError::NotFound(worktree_id.to_string()))?;

        // Checkout base branch
        let output = Command::new("git")
            .args(["checkout", &self.config.base_branch])
            .current_dir(&self.config.main_repo_path)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WorktreeError::GitError(format!(
                "Failed to checkout base: {stderr}"
            )));
        }

        // Merge
        let merge_msg = format!("Merge worktree {} (task: {})", worktree_id, info.task_id);
        let output = Command::new("git")
            .args(["merge", "--no-ff", "-m", &merge_msg, &info.branch])
            .current_dir(&self.config.main_repo_path)
            .output()
            .await?;

        if output.status.success() {
            info.status = WorktreeStatus::Merged;

            // Count changed files
            let diff_output = Command::new("git")
                .args(["diff", "--name-only", "HEAD~1..HEAD"])
                .current_dir(&self.config.main_repo_path)
                .output()
                .await?;

            let files_changed = String::from_utf8_lossy(&diff_output.stdout)
                .lines()
                .count();

            return Ok(MergeResult {
                success: true,
                conflicts: vec![],
                files_changed,
            });
        }

        // Check for conflicts
        let conflict_output = Command::new("git")
            .args(["diff", "--name-only", "--diff-filter=U"])
            .current_dir(&self.config.main_repo_path)
            .output()
            .await?;

        let conflicts: Vec<String> = String::from_utf8_lossy(&conflict_output.stdout)
            .lines()
            .map(|l| l.to_string())
            .collect();

        info.status = WorktreeStatus::Conflict;

        // Abort the failed merge
        let _ = Command::new("git")
            .args(["merge", "--abort"])
            .current_dir(&self.config.main_repo_path)
            .output()
            .await;

        Ok(MergeResult {
            success: false,
            conflicts,
            files_changed: 0,
        })
    }

    /// Remove a worktree and its branch.
    pub async fn cleanup_worktree(&self, worktree_id: Uuid) -> Result<(), WorktreeError> {
        let mut active = self.active.write().await;
        let info = active
            .remove(&worktree_id)
            .ok_or_else(|| WorktreeError::NotFound(worktree_id.to_string()))?;

        // git worktree remove <path>
        let output = Command::new("git")
            .args(["worktree", "remove", info.path.to_str().unwrap_or("")])
            .current_dir(&self.config.main_repo_path)
            .output()
            .await;

        // Force remove if normal removal fails
        if output.is_err() || !output.unwrap().status.success() {
            let _ = Command::new("git")
                .args([
                    "worktree",
                    "remove",
                    "--force",
                    info.path.to_str().unwrap_or(""),
                ])
                .current_dir(&self.config.main_repo_path)
                .output()
                .await;

            // Last resort: rm the directory
            let _ = tokio::fs::remove_dir_all(&info.path).await;
        }

        // Delete the branch
        let _ = Command::new("git")
            .args(["branch", "-d", &info.branch])
            .current_dir(&self.config.main_repo_path)
            .output()
            .await;

        // Force delete if normal fails
        let _ = Command::new("git")
            .args(["branch", "-D", &info.branch])
            .current_dir(&self.config.main_repo_path)
            .output()
            .await;

        Ok(())
    }

    /// Clean up all active worktrees.
    pub async fn cleanup_all(&self) -> Result<(), WorktreeError> {
        let ids: Vec<Uuid> = self.active.read().await.keys().copied().collect();
        for id in ids {
            if let Err(e) = self.cleanup_worktree(id).await {
                tracing::warn!(worktree_id = %id, error = %e, "Failed to cleanup worktree");
            }
        }
        Ok(())
    }

    /// List all active worktrees.
    pub async fn list_worktrees(&self) -> Vec<WorktreeInfo> {
        self.active.read().await.values().cloned().collect()
    }

    /// Get the worktree path for a given worktree ID.
    pub async fn get_worktree_path(&self, worktree_id: Uuid) -> Result<PathBuf, WorktreeError> {
        self.active
            .read()
            .await
            .get(&worktree_id)
            .map(|info| info.path.clone())
            .ok_or_else(|| WorktreeError::NotFound(worktree_id.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> WorktreeConfig {
        WorktreeConfig {
            max_concurrent: 2,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_capacity_exceeded() {
        let mgr = WorktreeManager {
            config: test_config(),
            active: Arc::new(RwLock::new(HashMap::new())),
        };

        // Fill to capacity directly (skip git ops)
        {
            let mut active = mgr.active.write().await;
            for i in 0..2 {
                let id = Uuid::new_v4();
                active.insert(
                    id,
                    WorktreeInfo {
                        id,
                        task_id: format!("task-{i}"),
                        worker_id: "w1".to_string(),
                        path: PathBuf::from(format!("/tmp/wt-{i}")),
                        branch: format!("branch-{i}"),
                        created_at: Utc::now(),
                        status: WorktreeStatus::Created,
                    },
                );
            }
        }

        // Next create should fail
        let result = mgr.create_worktree("task-3", "w1").await;
        assert!(matches!(result, Err(WorktreeError::CapacityExceeded(2))));
    }

    #[tokio::test]
    async fn test_cleanup_removes_from_map() {
        let mgr = WorktreeManager {
            config: test_config(),
            active: Arc::new(RwLock::new(HashMap::new())),
        };

        let id = Uuid::new_v4();
        {
            let mut active = mgr.active.write().await;
            active.insert(
                id,
                WorktreeInfo {
                    id,
                    task_id: "task-1".to_string(),
                    worker_id: "w1".to_string(),
                    path: PathBuf::from("/tmp/nonexistent-worktree"),
                    branch: "test-branch".to_string(),
                    created_at: Utc::now(),
                    status: WorktreeStatus::Created,
                },
            );
        }

        assert_eq!(mgr.list_worktrees().await.len(), 1);

        // Cleanup will fail git ops (no real repo) but should still remove from map
        let _ = mgr.cleanup_worktree(id).await;
        assert_eq!(mgr.list_worktrees().await.len(), 0);
    }

    #[tokio::test]
    async fn test_worktree_not_found() {
        let mgr = WorktreeManager::new(test_config());
        let result = mgr.get_worktree_path(Uuid::new_v4()).await;
        assert!(matches!(result, Err(WorktreeError::NotFound(_))));
    }

    /// Helper: create a temporary git repo for worktree tests.
    async fn init_temp_repo() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path().to_path_buf();

        // git init + initial commit (worktree requires at least one commit)
        Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()
            .await
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(&repo_path)
            .output()
            .await
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(&repo_path)
            .output()
            .await
            .unwrap();

        // Create initial file and commit
        tokio::fs::write(repo_path.join("README.md"), "# Test\n")
            .await
            .unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(&repo_path)
            .output()
            .await
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&repo_path)
            .output()
            .await
            .unwrap();

        (dir, repo_path)
    }

    #[tokio::test]
    async fn test_create_worktree_real_git() {
        let (_dir, repo_path) = init_temp_repo().await;
        let wt_base = tempfile::tempdir().unwrap();

        let config = WorktreeConfig {
            worktree_base_path: wt_base.path().to_path_buf(),
            main_repo_path: repo_path.clone(),
            base_branch: "main".to_string(),
            max_concurrent: 5,
        };

        let mgr = WorktreeManager::new(config);
        let info = mgr.create_worktree("task-1", "worker-1").await.unwrap();

        // Verify the worktree was created
        assert_eq!(info.task_id, "task-1");
        assert_eq!(info.worker_id, "worker-1");
        assert!(info.path.exists(), "Worktree path should exist on disk");
        assert!(info.branch.starts_with("worktree-task-1-"));
        assert_eq!(info.status, WorktreeStatus::Created);

        // Verify the branch exists via git
        let output = Command::new("git")
            .args(["branch", "--list", &info.branch])
            .current_dir(&repo_path)
            .output()
            .await
            .unwrap();
        let branches = String::from_utf8_lossy(&output.stdout);
        assert!(
            branches.contains(&info.branch),
            "Branch '{}' should exist, got: {branches}",
            info.branch
        );

        // Verify it appears in list_worktrees
        let list = mgr.list_worktrees().await;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, info.id);

        // Cleanup
        mgr.cleanup_worktree(info.id).await.unwrap();
        assert_eq!(mgr.list_worktrees().await.len(), 0);
    }
}
