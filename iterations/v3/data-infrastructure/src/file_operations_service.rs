//! File Operations Service Implementation
//!
//! Implements the shared FileOperationsService interface using the existing
//! file operations infrastructure (GitWorktreeWorkspace, TempMirrorWorkspace).
//!
//! @author @darianrosebrook

use crate::file_operations::{
    AllowList as DataInfraAllowList, Budgets as DataInfraBudgets, ChangeSet, ChangeSetId,
    GitWorktreeWorkspace, Hunk as DataInfraHunk, Patch as DataInfraPatch, TempMirrorWorkspace,
    Workspace as DataInfraWorkspace,
};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use system_common_interfaces::{
    AllowList, Budgets, Changeset, ChangesetId, DirectoryEntry, FileMetadata,
    FileOperationsService, FileOpsError, FileResult, Workspace as SystemWorkspace,
    WorkspaceFactory, WorkspaceState, WorkspaceStatus,
};
use tokio::fs;
use tokio::sync::RwLock;

/// Adapter wrapper that bridges data-infrastructure Workspace to system-common-interfaces Workspace
struct WorkspaceAdapter {
    inner: Arc<dyn DataInfraWorkspace>,
}

impl std::fmt::Debug for WorkspaceAdapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceAdapter")
            .field("inner", &"<Workspace>")
            .finish()
    }
}

impl WorkspaceAdapter {
    fn new(inner: Arc<dyn DataInfraWorkspace>) -> Self {
        Self { inner }
    }

    /// Convert system-common-interfaces Changeset to data-infrastructure ChangeSet
    fn convert_changeset(
        &self,
        changeset: &Changeset,
    ) -> std::result::Result<ChangeSet, FileOpsError> {
        let patches: Vec<DataInfraPatch> = changeset
            .patches
            .iter()
            .map(|p| {
                let hunks: Vec<DataInfraHunk> = p
                    .hunks
                    .iter()
                    .map(|h| DataInfraHunk {
                        old_start: h.old_start as u32,
                        old_lines: h.old_lines as u32,
                        new_start: h.new_start as u32,
                        new_lines: h.new_lines as u32,
                        lines: h.lines.clone(),
                    })
                    .collect();

                Ok(DataInfraPatch {
                    path: p.path.clone(),
                    hunks,
                    expected_prev_sha256: None,
                })
            })
            .collect::<std::result::Result<Vec<_>, FileOpsError>>()?;

        Ok(ChangeSet { patches })
    }

    /// Convert system-common-interfaces AllowList to data-infrastructure AllowList
    fn convert_allowlist(&self, allowlist: &AllowList) -> DataInfraAllowList {
        // Map allowed_patterns to globs
        // Note: system-common-interfaces uses allowed_patterns, data-infrastructure uses globs
        DataInfraAllowList {
            globs: allowlist.allowed_patterns.clone(),
        }
    }

    /// Convert system-common-interfaces Budgets to data-infrastructure Budgets
    fn convert_budgets(&self, budgets: &Budgets) -> DataInfraBudgets {
        DataInfraBudgets {
            max_files: budgets.max_files.unwrap_or(100),
            max_loc: budgets.max_lines.unwrap_or(10000),
        }
    }

    /// Convert data-infrastructure ChangeSetId to system-common-interfaces ChangesetId
    fn convert_changeset_id(&self, id: &ChangeSetId) -> ChangesetId {
        ChangesetId(id.0.clone())
    }
}

#[async_trait]
impl SystemWorkspace for WorkspaceAdapter {
    fn root(&self) -> &Path {
        self.inner.root()
    }

    async fn apply(
        &self,
        changeset: &Changeset,
        allowlist: &AllowList,
        budgets: &Budgets,
    ) -> FileResult<ChangesetId> {
        let data_changeset = self.convert_changeset(changeset)?;
        let data_allowlist = self.convert_allowlist(allowlist);
        let data_budgets = self.convert_budgets(budgets);

        let data_id = self
            .inner
            .apply(&data_changeset, &data_allowlist, &data_budgets)
            .await
            .map_err(|e| FileOpsError::Changeset(format!("Apply failed: {}", e)))?;

        Ok(self.convert_changeset_id(&data_id))
    }

    async fn revert(&self, changeset_id: &ChangesetId) -> FileResult<()> {
        let data_id = ChangeSetId(changeset_id.0.clone());
        self.inner
            .revert(&data_id)
            .await
            .map_err(|e| FileOpsError::Changeset(format!("Revert failed: {}", e)))
    }

    async fn promote(&self) -> FileResult<()> {
        self.inner
            .promote()
            .await
            .map_err(|e| FileOpsError::Changeset(format!("Promote failed: {}", e)))
    }
}

/// File operations service that implements the shared interface
pub struct DataInfrastructureFileOperationsService {
    /// Workspace registry for tracking active workspaces
    workspace_registry:
        Arc<RwLock<std::collections::HashMap<String, Arc<dyn DataInfraWorkspace + Send + Sync>>>>,
    /// Default repository path for Git operations
    default_repo_path: PathBuf,
}

impl std::fmt::Debug for DataInfrastructureFileOperationsService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataInfrastructureFileOperationsService")
            .field("default_repo_path", &self.default_repo_path)
            .field("workspace_registry", &"<workspace registry>")
            .finish()
    }
}

impl DataInfrastructureFileOperationsService {
    /// Create a new file operations service
    pub fn new(default_repo_path: PathBuf) -> Self {
        Self {
            workspace_registry: Arc::new(RwLock::new(std::collections::HashMap::new())),
            default_repo_path,
        }
    }

    /// Create a new file operations service with custom configuration
    pub fn with_config(default_repo_path: PathBuf) -> Self {
        Self::new(default_repo_path)
    }
}

#[async_trait]
impl FileOperationsService for DataInfrastructureFileOperationsService {
    async fn validate_changeset(
        &self,
        changeset: &Changeset,
        allowlist: &AllowList,
        budgets: &Budgets,
    ) -> FileResult<()> {
        // Convert types and use the existing validate_changeset function
        let adapter = WorkspaceAdapter {
            inner: Arc::new(
                TempMirrorWorkspace::new(&self.default_repo_path, "validation")
                    .await
                    .map_err(|e| {
                        FileOpsError::Validation(format!("Workspace creation failed: {}", e))
                    })?,
            ),
        };

        let data_changeset = adapter.convert_changeset(changeset)?;
        let data_allowlist = adapter.convert_allowlist(allowlist);
        let data_budgets = adapter.convert_budgets(budgets);

        // Use the existing validate_changeset function from data-infrastructure
        crate::file_operations::validate_changeset(&data_changeset, &data_allowlist, &data_budgets)
            .map_err(|e| FileOpsError::Validation(format!("Validation failed: {}", e)))
    }

    async fn create_workspace(
        &self,
        task_id: &str,
        repo_path: &Path,
    ) -> FileResult<Box<dyn SystemWorkspace>> {
        // Check if workspace already exists in registry
        {
            let registry = self.workspace_registry.read().await;
            if let Some(existing) = registry.get(task_id) {
                // Workspace already exists - return adapter wrapping the existing workspace
                return Ok(Box::new(WorkspaceAdapter::new(existing.clone())));
            }
        }

        // Create underlying workspace
        let inner_workspace: Arc<dyn DataInfraWorkspace> = if repo_path.join(".git").exists() {
            Arc::new(
                GitWorktreeWorkspace::new(repo_path, task_id)
                    .await
                    .map_err(|e| {
                        FileOpsError::Path(format!("Git workspace creation failed: {}", e))
                    })?,
            )
        } else {
            Arc::new(
                TempMirrorWorkspace::new(repo_path, task_id)
                    .await
                    .map_err(|e| {
                        FileOpsError::Path(format!("Temp workspace creation failed: {}", e))
                    })?,
            )
        };

        // Register the workspace
        {
            let mut registry = self.workspace_registry.write().await;
            registry.insert(task_id.to_string(), inner_workspace.clone());
        }

        // Wrap in adapter and return
        Ok(Box::new(WorkspaceAdapter::new(inner_workspace)))
    }

    async fn get_workspace_status(&self, task_id: &str) -> FileResult<WorkspaceStatus> {
        let registry = self.workspace_registry.read().await;

        if registry.get(task_id).is_some() {
            Ok(WorkspaceStatus {
                task_id: task_id.to_string(),
                state: WorkspaceState::Ready,
                active_changeset: None,
                created_at: chrono::Utc::now(),
                last_activity: chrono::Utc::now(),
            })
        } else {
            Err(FileOpsError::WorkspaceNotFound(task_id.to_string()))
        }
    }

    async fn read_file(&self, file_path: &Path, max_size: Option<u64>) -> FileResult<Vec<u8>> {
        // Resolve path relative to default repo path
        let full_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            self.default_repo_path.join(file_path)
        };

        // Check file size before reading
        let metadata = fs::metadata(&full_path)
            .await
            .map_err(|e| FileOpsError::Io(e))?;

        if let Some(max) = max_size {
            if metadata.len() > max {
                return Err(FileOpsError::Validation(format!(
                    "File size {} exceeds maximum allowed size {}",
                    metadata.len(),
                    max
                )));
            }
        }

        // Read file content
        fs::read(&full_path).await.map_err(|e| FileOpsError::Io(e))
    }

    async fn file_exists(&self, file_path: &Path) -> FileResult<bool> {
        let full_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            self.default_repo_path.join(file_path)
        };

        match fs::metadata(&full_path).await {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(FileOpsError::Io(e)),
        }
    }

    async fn get_file_metadata(&self, file_path: &Path) -> FileResult<FileMetadata> {
        let full_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            self.default_repo_path.join(file_path)
        };

        let metadata = fs::metadata(&full_path)
            .await
            .map_err(|e| FileOpsError::Io(e))?;

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
            .flatten();

        #[cfg(unix)]
        let permissions = {
            use std::os::unix::fs::PermissionsExt;
            Some(metadata.permissions().mode())
        };
        #[cfg(not(unix))]
        let permissions = None;

        Ok(FileMetadata {
            path: file_path.to_string_lossy().to_string(),
            size: metadata.len(),
            is_directory: metadata.is_dir(),
            modified,
            permissions,
        })
    }

    async fn list_directory(&self, dir_path: &Path) -> FileResult<Vec<DirectoryEntry>> {
        let full_path = if dir_path.is_absolute() {
            dir_path.to_path_buf()
        } else {
            self.default_repo_path.join(dir_path)
        };

        let mut entries = Vec::new();
        let mut dir = fs::read_dir(&full_path)
            .await
            .map_err(|e| FileOpsError::Io(e))?;

        while let Some(entry) = dir.next_entry().await.map_err(|e| FileOpsError::Io(e))? {
            let metadata = entry.metadata().await.map_err(|e| FileOpsError::Io(e))?;

            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
                .flatten();

            entries.push(DirectoryEntry {
                name: entry.file_name().to_string_lossy().to_string(),
                path: entry.path().to_string_lossy().to_string(),
                is_directory: metadata.is_dir(),
                size: if metadata.is_dir() { 0 } else { metadata.len() },
                modified,
            });
        }

        Ok(entries)
    }

    async fn create_directory(&self, dir_path: &Path) -> FileResult<()> {
        let full_path = if dir_path.is_absolute() {
            dir_path.to_path_buf()
        } else {
            self.default_repo_path.join(dir_path)
        };

        fs::create_dir_all(&full_path)
            .await
            .map_err(|e| FileOpsError::Io(e))
    }

    async fn delete_file(&self, file_path: &Path) -> FileResult<()> {
        let full_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            self.default_repo_path.join(file_path)
        };

        let metadata = fs::metadata(&full_path)
            .await
            .map_err(|e| FileOpsError::Io(e))?;

        if metadata.is_dir() {
            fs::remove_dir_all(&full_path).await
        } else {
            fs::remove_file(&full_path).await
        }
        .map_err(|e| FileOpsError::Io(e))
    }

    async fn move_file(&self, from: &Path, to: &Path) -> FileResult<()> {
        let from_path = if from.is_absolute() {
            from.to_path_buf()
        } else {
            self.default_repo_path.join(from)
        };

        let to_path = if to.is_absolute() {
            to.to_path_buf()
        } else {
            self.default_repo_path.join(to)
        };

        // Ensure parent directory exists
        if let Some(parent) = to_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| FileOpsError::Io(e))?;
        }

        fs::rename(&from_path, &to_path)
            .await
            .map_err(|e| FileOpsError::Io(e))
    }

    async fn copy_file(&self, from: &Path, to: &Path) -> FileResult<()> {
        let from_path = if from.is_absolute() {
            from.to_path_buf()
        } else {
            self.default_repo_path.join(from)
        };

        let to_path = if to.is_absolute() {
            to.to_path_buf()
        } else {
            self.default_repo_path.join(to)
        };

        // Ensure parent directory exists
        if let Some(parent) = to_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| FileOpsError::Io(e))?;
        }

        let metadata = fs::metadata(&from_path)
            .await
            .map_err(|e| FileOpsError::Io(e))?;

        if metadata.is_dir() {
            // Implement recursive directory copy using tokio::fs
            self.copy_directory_recursive(&from_path, &to_path).await?;
        } else {
            // Copy single file
            fs::copy(&from_path, &to_path)
                .await
                .map_err(|e| FileOpsError::Io(e))?;
        }

        Ok(())
    }
}

impl DataInfrastructureFileOperationsService {
    /// Recursively copy a directory and all its contents (helper method)
    /// Uses iterative approach to avoid recursive async function limitations
    async fn copy_directory_recursive(&self, from: &Path, to: &Path) -> FileResult<()> {
        use std::collections::VecDeque;

        // Stack of (source_path, dest_path) pairs to process
        let mut stack = VecDeque::new();
        stack.push_back((from.to_path_buf(), to.to_path_buf()));

        while let Some((src_path, dst_path)) = stack.pop_front() {
            // Create destination directory if it doesn't exist
            fs::create_dir_all(&dst_path)
                .await
                .map_err(|e| FileOpsError::Io(e))?;

            // Read source directory entries
            let mut entries = fs::read_dir(&src_path)
                .await
                .map_err(|e| FileOpsError::Io(e))?;

            // Process each entry in the directory
            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| FileOpsError::Io(e))?
            {
                let entry_path = entry.path();
                let entry_name = entry.file_name();
                let dest_path = dst_path.join(&entry_name);

                // Get entry metadata to determine if it's a directory or file
                let entry_metadata = entry.metadata().await.map_err(|e| FileOpsError::Io(e))?;

                if entry_metadata.is_dir() {
                    // Add subdirectory to stack for processing
                    stack.push_back((entry_path, dest_path));
                } else {
                    // Copy file
                    fs::copy(&entry_path, &dest_path)
                        .await
                        .map_err(|e| FileOpsError::Io(e))?;
                }
            }
        }

        Ok(())
    }
}

/// Workspace factory implementation
#[derive(Debug)]
pub struct DataInfrastructureWorkspaceFactory {
    _default_repo_path: PathBuf,
}

impl DataInfrastructureWorkspaceFactory {
    pub fn new(default_repo_path: PathBuf) -> Self {
        Self {
            _default_repo_path: default_repo_path,
        }
    }
}

#[async_trait]
impl WorkspaceFactory for DataInfrastructureWorkspaceFactory {
    async fn create_workspace(
        &self,
        task_id: &str,
        repo_path: &Path,
    ) -> FileResult<Box<dyn SystemWorkspace>> {
        let inner: Arc<dyn DataInfraWorkspace> = if repo_path.join(".git").exists() {
            Arc::new(
                GitWorktreeWorkspace::new(repo_path, task_id)
                    .await
                    .map_err(|e| {
                        FileOpsError::Path(format!("Git workspace creation failed: {}", e))
                    })?,
            )
        } else {
            Arc::new(
                TempMirrorWorkspace::new(repo_path, task_id)
                    .await
                    .map_err(|e| {
                        FileOpsError::Path(format!("Temp workspace creation failed: {}", e))
                    })?,
            )
        };

        Ok(Box::new(WorkspaceAdapter::new(inner)))
    }
}

/// Helper function to create a file operations service instance
pub fn create_file_operations_service(
    default_repo_path: PathBuf,
) -> Arc<dyn FileOperationsService> {
    Arc::new(DataInfrastructureFileOperationsService::new(
        default_repo_path,
    ))
}

/// Helper function to create a workspace factory instance
pub fn create_workspace_factory(default_repo_path: PathBuf) -> Arc<dyn WorkspaceFactory> {
    Arc::new(DataInfrastructureWorkspaceFactory::new(default_repo_path))
}
