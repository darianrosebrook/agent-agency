//! Git worktree management for snapshots and rollbacks

use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Manages Git worktree for snapshot functionality
#[derive(Debug)]
pub struct GitWorktree {
    worktree_root: PathBuf,
    main_repo: PathBuf,
}

impl GitWorktree {
    /// Create a new Git worktree manager
    pub async fn new(worktree_root: PathBuf) -> Result<Self, GitWorktreeError> {
        // Find the main repository (parent of .git or current dir if it's a repo)
        let main_repo = Self::find_git_repo(&worktree_root).await?;

        // Create worktree directory if it doesn't exist
        tokio::fs::create_dir_all(&worktree_root).await
            .map_err(|e| GitWorktreeError::IoError(e))?;

        // Initialize worktree if not already a git repo
        if !Self::is_git_repo(&worktree_root).await? {
            Self::init_worktree(&main_repo, &worktree_root).await?;
        }

        Ok(Self {
            worktree_root,
            main_repo,
        })
    }

    /// Commit current changes as a snapshot
    pub async fn commit_snapshot(&self, message: &str) -> Result<String, GitWorktreeError> {
        // Add all changes
        self.run_git_command(&["add", "."]).await?;

        // Check if there are changes to commit
        let status = self.run_git_command(&["status", "--porcelain"]).await?;
        if status.trim().is_empty() {
            // No changes, return current commit hash
            let hash = self.run_git_command(&["rev-parse", "HEAD"]).await?;
            return Ok(hash.trim().to_string());
        }

        // Commit with timestamp
        let commit_msg = format!("{} - {}", message, chrono::Utc::now().to_rfc3339());
        self.run_git_command(&["commit", "-m", &commit_msg]).await?;

        // Get the commit hash
        let hash = self.run_git_command(&["rev-parse", "HEAD"]).await?;
        Ok(hash.trim().to_string())
    }

    /// Rollback to a specific snapshot
    pub async fn rollback_to_snapshot(&self, snapshot_id: &str) -> Result<(), GitWorktreeError> {
        // Reset hard to the snapshot
        self.run_git_command(&["reset", "--hard", snapshot_id]).await?;
        Ok(())
    }

    /// Get current branch name
    pub async fn current_branch(&self) -> Result<String, GitWorktreeError> {
        let output = self.run_git_command(&["rev-parse", "--abbrev-ref", "HEAD"]).await?;
        Ok(output.trim().to_string())
    }

    /// Check if a directory is a git repository
    async fn is_git_repo(path: &Path) -> Result<bool, GitWorktreeError> {
        let git_dir = path.join(".git");
        Ok(tokio::fs::metadata(&git_dir).await.is_ok())
    }

    /// Find the git repository root
    async fn find_git_repo(start_path: &Path) -> Result<PathBuf, GitWorktreeError> {
        let mut current = start_path.to_path_buf();

        loop {
            if Self::is_git_repo(&current).await? {
                return Ok(current);
            }

            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }

        Err(GitWorktreeError::NoGitRepo)
    }

    /// Initialize a worktree from main repo
    async fn init_worktree(main_repo: &Path, worktree_path: &Path) -> Result<(), GitWorktreeError> {
        // Use git worktree add to create a new worktree
        let output = Command::new("git")
            .current_dir(main_repo)
            .args(&["worktree", "add", &worktree_path.to_string_lossy(), "--detach"])
            .output()
            .await
            .map_err(|e| GitWorktreeError::CommandError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitWorktreeError::GitCommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Run a git command in the worktree
    async fn run_git_command(&self, args: &[&str]) -> Result<String, GitWorktreeError> {
        let output = Command::new("git")
            .current_dir(&self.worktree_root)
            .args(args)
            .output()
            .await
            .map_err(|e| GitWorktreeError::CommandError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitWorktreeError::GitCommandFailed(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }
}

/// Errors from Git worktree operations
#[derive(Debug, thiserror::Error)]
pub enum GitWorktreeError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Git command failed: {0}")]
    GitCommandFailed(String),

    #[error("Command execution error: {0}")]
    CommandError(String),

    #[error("No git repository found")]
    NoGitRepo,
}
