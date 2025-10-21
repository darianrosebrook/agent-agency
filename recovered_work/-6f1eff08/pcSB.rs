//! Temp Mirror Workspace Implementation
//!
//! Uses temporary directory mirroring for safe file editing in non-Git environments
//! with rsync-based copying and snapshot capabilities.

use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use uuid::Uuid;
use crate::{Workspace, ChangeSet, AllowList, Budgets, ChangeSetId, FileOpsError, Result, validate_changeset};

/// Temp directory mirror workspace for safe file operations
pub struct TempMirrorWorkspace {
    /// Original project root
    source_root: PathBuf,
    /// Temporary workspace directory
    workspace_path: PathBuf,
    /// Task ID for this workspace
    task_id: String,
    /// Applied changesets for rollback tracking
    applied_changesets: Vec<(ChangeSetId, ChangeSet)>,
}

impl TempMirrorWorkspace {
    /// Create a new temp mirror workspace
    pub async fn new(project_path: &Path, task_id: &str) -> Result<Self> {
        let source_root = project_path.canonicalize()
            .map_err(|e| FileOpsError::Path(format!("Cannot canonicalize project path: {}", e)))?;

        // Create temp workspace directory
        let workspace_path = std::env::temp_dir()
            .join(format!("caws-workspace-{}", task_id));

        // Clean up any existing workspace
        let _ = fs::remove_dir_all(&workspace_path).await;

        // Create workspace directory
        fs::create_dir_all(&workspace_path).await
            .map_err(FileOpsError::Io)?;

        // Mirror source to workspace using rsync if available, otherwise manual copy
        Self::mirror_directory(&source_root, &workspace_path).await?;

        Ok(Self {
            source_root,
            workspace_path,
            task_id: task_id.to_string(),
            applied_changesets: Vec::new(),
        })
    }

    /// Mirror directory contents (rsync if available, manual copy otherwise)
    async fn mirror_directory(source: &Path, dest: &Path) -> Result<()> {
        // Try rsync first
        let rsync_result = Command::new("rsync")
            .args(["-a", "--exclude=.git", &format!("{}/", source.display()), &dest.display().to_string()])
            .output();

        if rsync_result.is_ok() && rsync_result.as_ref().unwrap().status.success() {
            return Ok(());
        }

        // Fallback to manual copy
        Self::copy_directory_recursive(source, dest).await
    }

    /// Recursively copy directory contents
    async fn copy_directory_recursive(source: &Path, dest: &Path) -> Result<()> {
        let mut stack = vec![(source.to_path_buf(), dest.to_path_buf())];

        while let Some((src, dst)) = stack.pop() {
            // Create destination directory
            fs::create_dir_all(&dst).await.map_err(FileOpsError::Io)?;

            // Read source directory
            let mut entries = fs::read_dir(&src).await.map_err(FileOpsError::Io)?;

            while let Some(entry) = entries.next_entry().await.map_err(FileOpsError::Io)? {
                let entry_type = entry.file_type().await.map_err(FileOpsError::Io)?;
                let src_path = entry.path();
                let dst_path = dst.join(entry.file_name());

                if entry_type.is_dir() {
                    // Skip .git directories
                    if entry.file_name() != ".git" {
                        stack.push((src_path, dst_path));
                    }
                } else if entry_type.is_file() {
                    fs::copy(&src_path, &dst_path).await.map_err(FileOpsError::Io)?;
                }
            }
        }

        Ok(())
    }

    /// Apply patches to files in the workspace
    async fn apply_patches(&self, changeset: &ChangeSet) -> Result<()> {
        for patch in &changeset.patches {
            self.apply_single_patch(patch).await?;
        }
        Ok(())
    }

    /// Apply a single patch
    async fn apply_single_patch(&self, patch: &crate::Patch) -> Result<()> {
        let file_path = self.workspace_path.join(&patch.path);

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await.map_err(FileOpsError::Io)?;
        }

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

    /// Create a snapshot of current workspace state
    async fn create_snapshot(&self) -> Result<PathBuf> {
        let snapshot_dir = std::env::temp_dir()
            .join(format!("caws-snapshot-{}-{}", self.task_id, Uuid::new_v4()));

        fs::create_dir_all(&snapshot_dir).await.map_err(FileOpsError::Io)?;
        Self::copy_directory_recursive(&self.workspace_path, &snapshot_dir).await?;

        Ok(snapshot_dir)
    }
}

#[async_trait::async_trait]
impl Workspace for TempMirrorWorkspace {
    fn root(&self) -> &Path {
        &self.workspace_path
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

        // Create snapshot before applying changes
        let _snapshot = self.create_snapshot().await?;

        // Apply patches
        self.apply_patches(changeset).await?;

        // Track applied changeset for rollback
        // Note: In real implementation, we'd store this persistently
        // self.applied_changesets.push((changeset_id.clone(), changeset.clone()));

        Ok(changeset_id)
    }

    async fn revert(&self, _changeset_id: &ChangeSetId) -> Result<()> {
        // Find the changeset to revert
          // TODO: Implement persistent changeset storage
          // - Create changeset database schema and models
          // - Implement changeset serialization and storage
          // - Add changeset retrieval and replay capabilities
          // - Support changeset versioning and conflict resolution
          // - Implement changeset cleanup and retention policies
          // - Add changeset integrity validation and checksums
          // PLACEHOLDER: Using in-memory approach for now

        // Since we don't have persistent changeset tracking yet,
        // we'll restore from the most recent snapshot
        // TODO: Implement comprehensive changeset application system
        // - Add changeset validation and conflict detection
        // - Implement atomic changeset application with rollback
        // - Support partial changeset application and recovery
        // - Add changeset dependency resolution and ordering
        // - Implement changeset progress tracking and status
        // - Add changeset application performance monitoring
        // PLACEHOLDER: Using simplified in-memory application

        Err(FileOpsError::Path("Revert not yet implemented for temp workspace".to_string()))
    }

    async fn promote(&self) -> Result<()> {
        // Copy workspace changes back to source
        Self::mirror_directory(&self.workspace_path, &self.source_root).await
    }
}

impl Drop for TempMirrorWorkspace {
    fn drop(&mut self) {
        // Clean up workspace on drop
        let _ = std::fs::remove_dir_all(&self.workspace_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    async fn setup_temp_project() -> Result<(TempDir, PathBuf)> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().to_path_buf();

        // Create some test files
        fs::write(project_path.join("README.md"), "# Test Project").await?;
        fs::create_dir_all(project_path.join("src")).await?;
        fs::write(project_path.join("src/main.rs"), "fn main() {}").await?;

        Ok((temp_dir, project_path))
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
    fn test_temp_workspace_types() {
        // Basic type checking test
        let changeset_id = ChangeSetId("test-456".to_string());
        assert_eq!(changeset_id.0, "test-456");
    }
}
