//! Git Worktree Workspace Implementation
//!
//! Uses Git worktrees for safe, versioned file editing with automatic rollback
//! capabilities and integration with Git's change tracking.

use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use uuid::Uuid;
use crate::{Workspace, ChangeSet, AllowList, Budgets, ChangeSetId, FileOpsError, Result, validate_changeset};

/// Git worktree-based workspace for safe file operations
pub struct GitWorktreeWorkspace {
    /// Root path of the Git repository
    repo_root: PathBuf,
    /// Path to the worktree directory
    worktree_path: PathBuf,
    /// Task ID for this workspace
    _task_id: String,
    /// Original branch we branched from
    _original_branch: String,
    /// Worktree branch name
    worktree_branch: String,
}

impl GitWorktreeWorkspace {
    /// Create a new Git worktree workspace
    pub async fn new(repo_path: &Path, task_id: &str) -> Result<Self> {
        let repo_root = repo_path.canonicalize()
            .map_err(|e| FileOpsError::Path(format!("Cannot canonicalize repo path: {}", e)))?;

        // Verify this is a Git repository
        if !repo_root.join(".git").exists() {
            return Err(FileOpsError::Path("Not a Git repository".to_string()));
        }

        // Get current branch
        let original_branch = Self::get_current_branch(&repo_root)?;

        // Create worktree branch name
        let worktree_branch = format!("caws/{}", task_id);

        // Create worktree directory
        let worktree_path = repo_root.join("..").join(format!("caws-worktree-{}", task_id));

        // Clean up any existing worktree
        let _ = fs::remove_dir_all(&worktree_path).await;

        // Create Git worktree
        Self::create_git_worktree(&repo_root, &worktree_branch, &worktree_path)?;

        Ok(Self {
            repo_root,
            worktree_path,
            _task_id: task_id.to_string(),
            _original_branch: original_branch,
            worktree_branch,
        })
    }

    /// Get the current Git branch
    fn get_current_branch(repo_path: &Path) -> Result<String> {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| FileOpsError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get current branch: {}", e)
            )))?;

        if !output.status.success() {
            return Err(FileOpsError::Path("Failed to get current Git branch".to_string()));
        }

        Ok(String::from_utf8(output.stdout)
            .map_err(|e| FileOpsError::Path(format!("Invalid branch name: {}", e)))?
            .trim()
            .to_string())
    }

    /// Create a Git worktree
    fn create_git_worktree(repo_path: &Path, branch_name: &str, worktree_path: &Path) -> Result<()> {
        // Create the worktree
        let output = Command::new("git")
            .args(["worktree", "add", "-b", branch_name, &worktree_path.to_string_lossy()])
            .current_dir(repo_path)
            .output()
            .map_err(|e| FileOpsError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create worktree: {}", e)
            )))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FileOpsError::Path(format!("Git worktree creation failed: {}", stderr)));
        }

        Ok(())
    }

    /// Apply patches to files in the worktree
    async fn apply_patches(&self, changeset: &ChangeSet) -> Result<()> {
        for patch in &changeset.patches {
            self.apply_single_patch(patch).await?;
        }
        Ok(())
    }

    /// Apply a single patch
    async fn apply_single_patch(&self, patch: &crate::Patch) -> Result<()> {
        let file_path = self.worktree_path.join(&patch.path);

        // Read current file content
        let current_content = match fs::read_to_string(&file_path).await {
            Ok(content) => content,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(FileOpsError::Io(e)),
        };

        // Apply patch hunks
        let new_content = self.apply_hunks_to_content(&current_content, &patch.hunks)?;

        // Write new content
        fs::write(&file_path, new_content).await
            .map_err(FileOpsError::Io)?;

        Ok(())
    }

    /// Apply hunks to file content
    fn apply_hunks_to_content(&self, content: &str, hunks: &[crate::Hunk]) -> Result<String> {
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        let mut offset: i32 = 0;

        for hunk in hunks {
            let base_start = (hunk.old_start as usize).saturating_sub(1);
            let start_line = if offset >= 0 {
                base_start + offset as usize
            } else {
                base_start.saturating_sub(offset.abs() as usize)
            };
            let _end_line = start_line + (hunk.old_lines as usize);

            // Remove old lines
            if start_line < lines.len() {
                let remove_count = std::cmp::min(hunk.old_lines as usize, lines.len() - start_line);
                lines.drain(start_line..start_line + remove_count);
                offset -= remove_count as i32;
            }

            // Add new lines
            if hunk.new_lines > 0 {
                let insert_pos = std::cmp::min(start_line, lines.len());
                let new_lines: Vec<String> = hunk.lines
                    .lines()
                    .filter(|line| line.starts_with('+') || line.starts_with(' '))
                    .map(|line| line[1..].to_string())
                    .collect();

                for (i, new_line) in new_lines.into_iter().enumerate() {
                    lines.insert(insert_pos + i, new_line);
                }
                offset += hunk.new_lines as i32;
            }
        }

        Ok(lines.join("\n"))
    }

    /// Commit changes in the worktree
    async fn commit_changes(&self, changeset_id: &ChangeSetId) -> Result<()> {
        let output = Command::new("git")
            .args(["add", "."])
            .current_dir(&self.worktree_path)
            .output()
            .map_err(|e| FileOpsError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to add files: {}", e)
            )))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FileOpsError::Path(format!("Git add failed: {}", stderr)));
        }

        // Commit with changeset ID
        let commit_msg = format!("CAWS changeset: {}", changeset_id.0);
        let output = Command::new("git")
            .args(["commit", "-m", &commit_msg])
            .current_dir(&self.worktree_path)
            .output()
            .map_err(|e| FileOpsError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to commit: {}", e)
            )))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Don't fail if there are no changes to commit
            if !stderr.contains("nothing to commit") {
                return Err(FileOpsError::Path(format!("Git commit failed: {}", stderr)));
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Workspace for GitWorktreeWorkspace {
    fn root(&self) -> &Path {
        &self.worktree_path
    }

    async fn apply(
        &self,
        changeset: &ChangeSet,
        allowlist: &AllowList,
        budgets: &Budgets,
    ) -> Result<ChangeSetId> {
        // Validate changeset first
        validate_changeset(changeset, allowlist, budgets)?;

        // Generate changeset ID
        let changeset_id = ChangeSetId(Uuid::new_v4().to_string());

        // Apply patches
        self.apply_patches(changeset).await?;

        // Commit changes
        self.commit_changes(&changeset_id).await?;

        Ok(changeset_id)
    }

    async fn revert(&self, _changeset_id: &ChangeSetId) -> Result<()> {
        // Reset to the commit before this changeset
        let output = Command::new("git")
            .args(["reset", "--hard", "HEAD~1"])
            .current_dir(&self.worktree_path)
            .output()
            .map_err(|e| FileOpsError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to reset: {}", e)
            )))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FileOpsError::Path(format!("Git reset failed: {}", stderr)));
        }

        Ok(())
    }

    async fn promote(&self) -> Result<()> {
        // Merge worktree branch back to original branch
        let output = Command::new("git")
            .args(["merge", &self.worktree_branch])
            .current_dir(&self.repo_root)
            .output()
            .map_err(|e| FileOpsError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to merge: {}", e)
            )))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FileOpsError::Path(format!("Git merge failed: {}", stderr)));
        }

        Ok(())
    }
}

impl Drop for GitWorktreeWorkspace {
    fn drop(&mut self) {
        // Clean up worktree on drop
        let _ = Command::new("git")
            .args(["worktree", "remove", &self.worktree_path.to_string_lossy()])
            .current_dir(&self.repo_root)
            .status();

        let _ = std::fs::remove_dir_all(&self.worktree_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    async fn setup_git_repo() -> Result<(TempDir, PathBuf)> {
        let temp_dir = TempDir::new()?;
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize git repo
        let output = Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()?;

        assert!(output.status.success());

        // Configure git
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&repo_path)
            .output()?;

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&repo_path)
            .output()?;

        // Create initial file and commit
        fs::write(repo_path.join("README.md"), "# Test Repo").await?;
        Command::new("git")
            .args(["add", "README.md"])
            .current_dir(&repo_path)
            .output()?;

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&repo_path)
            .output()?;

        Ok((temp_dir, repo_path))
    }

      // TODO: Implement comprehensive async testing infrastructure
      // - Add tokio-test dependency and configuration
      // - Create async test utilities and fixtures
      // - Implement proper async test cleanup and teardown
      // - Add async test timeouts and cancellation handling
      // - Support concurrent test execution
      // - Add async test debugging and profiling tools
      // PLACEHOLDER: Relying on integration tests for now

    #[test]
    fn test_git_workspace_types() {
        // Basic type checking test
        let changeset_id = ChangeSetId("test-123".to_string());
        assert_eq!(changeset_id.0, "test-123");
    }
}
