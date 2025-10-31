//! File Operations Service Implementation
//!
//! Implements the shared FileOperationsService interface using the existing
//! file operations infrastructure (GitWorktreeWorkspace, TempMirrorWorkspace).
//!
//! @author @darianrosebrook

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use system_common_interfaces::{
    FileOperationsService, Workspace as SystemWorkspace, WorkspaceFactory, WorkspaceStatus, WorkspaceState,
    FileResult, FileOpsError, Changeset, AllowList, Budgets, ChangesetId,
};
use crate::file_operations::{
    Workspace as DataInfraWorkspace, GitWorktreeWorkspace, TempMirrorWorkspace,
    ChangeSet, ChangeSetId, Patch as DataInfraPatch, Hunk as DataInfraHunk,
    AllowList as DataInfraAllowList, Budgets as DataInfraBudgets,
};

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
    fn convert_changeset(&self, changeset: &Changeset) -> std::result::Result<ChangeSet, FileOpsError> {
        let patches: Vec<DataInfraPatch> = changeset.patches.iter()
            .map(|p| {
                let hunks: Vec<DataInfraHunk> = p.hunks.iter()
                    .map(|h| {
                        DataInfraHunk {
                            old_start: h.old_start as u32,
                            old_lines: h.old_lines as u32,
                            new_start: h.new_start as u32,
                            new_lines: h.new_lines as u32,
                            lines: h.lines.clone(),
                        }
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
        DataInfraAllowList { globs: allowlist.allowed_patterns.clone() }
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

        let data_id = self.inner.apply(&data_changeset, &data_allowlist, &data_budgets).await
            .map_err(|e| FileOpsError::Changeset(format!("Apply failed: {}", e)))?;

        Ok(self.convert_changeset_id(&data_id))
    }

    async fn revert(&self, changeset_id: &ChangesetId) -> FileResult<()> {
        let data_id = ChangeSetId(changeset_id.0.clone());
        self.inner.revert(&data_id).await
            .map_err(|e| FileOpsError::Changeset(format!("Revert failed: {}", e)))
    }

    async fn promote(&self) -> FileResult<()> {
        self.inner.promote().await
            .map_err(|e| FileOpsError::Changeset(format!("Promote failed: {}", e)))
    }
}

/// File operations service that implements the shared interface
pub struct DataInfrastructureFileOperationsService {
    /// Workspace registry for tracking active workspaces
    workspace_registry: Arc<RwLock<std::collections::HashMap<String, Arc<dyn DataInfraWorkspace + Send + Sync>>>>,
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
                TempMirrorWorkspace::new(&self.default_repo_path, "validation").await
                    .map_err(|e| FileOpsError::Validation(format!("Workspace creation failed: {}", e)))?
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
            Arc::new(GitWorktreeWorkspace::new(repo_path, task_id).await
                .map_err(|e| FileOpsError::Path(format!("Git workspace creation failed: {}", e)))?)
        } else {
            Arc::new(TempMirrorWorkspace::new(repo_path, task_id).await
                .map_err(|e| FileOpsError::Path(format!("Temp workspace creation failed: {}", e)))?)
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
}

/// Workspace factory implementation
#[derive(Debug)]
pub struct DataInfrastructureWorkspaceFactory {
    default_repo_path: PathBuf,
}

impl DataInfrastructureWorkspaceFactory {
    pub fn new(default_repo_path: PathBuf) -> Self {
        Self { default_repo_path }
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
            Arc::new(GitWorktreeWorkspace::new(repo_path, task_id).await
                .map_err(|e| FileOpsError::Path(format!("Git workspace creation failed: {}", e)))?)
        } else {
            Arc::new(TempMirrorWorkspace::new(repo_path, task_id).await
                .map_err(|e| FileOpsError::Path(format!("Temp workspace creation failed: {}", e)))?)
        };

        Ok(Box::new(WorkspaceAdapter::new(inner)))
    }
}

/// Helper function to create a file operations service instance
pub fn create_file_operations_service(default_repo_path: PathBuf) -> Arc<dyn FileOperationsService> {
    Arc::new(DataInfrastructureFileOperationsService::new(default_repo_path))
}

/// Helper function to create a workspace factory instance
pub fn create_workspace_factory(default_repo_path: PathBuf) -> Arc<dyn WorkspaceFactory> {
    Arc::new(DataInfrastructureWorkspaceFactory::new(default_repo_path))
}
