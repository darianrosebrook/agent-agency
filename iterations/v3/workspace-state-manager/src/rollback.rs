use crate::manager::WorkspaceStateManager;
/**
 * @fileoverview Rollback and view management for workspace state
 * @author @darianrosebrook
 */
use crate::types::*;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

/// Rollback operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    /// State ID that was rolled back to
    pub target_state: StateId,
    /// Number of files restored
    pub files_restored: usize,
    /// Number of files removed
    pub files_removed: usize,
    /// Number of files modified
    pub files_modified: usize,
    /// Total size change in bytes
    pub size_delta: i64,
    /// Duration of rollback operation in milliseconds
    pub duration_ms: u64,
    /// Any warnings generated during rollback
    pub warnings: Vec<String>,
    /// Whether the rollback was successful
    pub success: bool,
}

/// View management for workspace states
pub struct WorkspaceViewManager {
    /// Base workspace state manager
    manager: Arc<WorkspaceStateManager>,
    /// Directory for storing workspace views
    views_dir: PathBuf,
}

impl WorkspaceViewManager {
    /// Create a new workspace view manager
    pub fn new(manager: Arc<WorkspaceStateManager>, views_dir: impl AsRef<Path>) -> Self {
        Self {
            manager,
            views_dir: views_dir.as_ref().to_path_buf(),
        }
    }

    /// Create a view of the workspace at a specific state
    pub async fn create_view(
        &self,
        state_id: StateId,
        view_name: Option<String>,
    ) -> Result<WorkspaceResult<PathBuf>, WorkspaceError> {
        let start_time = std::time::Instant::now();
        let mut warnings = Vec::new();

        // Get the target state
        let state = self.manager.get_state(state_id).await?;

        // Generate view name if not provided
        let view_name = view_name.unwrap_or_else(|| {
            format!(
                "view-{}-{}",
                state_id.0.to_string()[..8].to_string(),
                chrono::Utc::now().format("%Y%m%d-%H%M%S")
            )
        });

        let view_path = self.views_dir.join(&view_name);

        info!(
            "Creating workspace view '{}' at state {:?}",
            view_name, state_id
        );

        // Ensure views directory exists
        std::fs::create_dir_all(&self.views_dir).map_err(|e| WorkspaceError::Io(e))?;

        // Create view directory
        if view_path.exists() {
            std::fs::remove_dir_all(&view_path).map_err(|e| WorkspaceError::Io(e))?;
        }
        std::fs::create_dir_all(&view_path).map_err(|e| WorkspaceError::Io(e))?;

        // Restore files from state
        let mut files_restored = 0;
        for (relative_path, _file_state) in &state.files {
            let target_path = view_path.join(relative_path);

            // Create parent directories
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| WorkspaceError::Io(e))?;
            }

            // Copy file from workspace root
            let source_path = state.workspace_root.join(relative_path);
            if source_path.exists() {
                std::fs::copy(&source_path, &target_path).map_err(|e| WorkspaceError::Io(e))?;
                files_restored += 1;
            } else {
                warnings.push(format!("Source file not found: {:?}", source_path));
            }
        }

        // Create a metadata file for the view
        let view_metadata = ViewMetadata {
            name: view_name.clone(),
            state_id,
            created_at: chrono::Utc::now(),
            source_workspace: state.workspace_root.clone(),
            files_count: files_restored,
            total_size: state.total_size,
        };

        let metadata_path = view_path.join(".workspace-view.json");
        let metadata_json = serde_json::to_string_pretty(&view_metadata)
            .map_err(|e| WorkspaceError::Serialization(e))?;
        std::fs::write(&metadata_path, metadata_json).map_err(|e| WorkspaceError::Io(e))?;

        let duration = start_time.elapsed();
        info!(
            "Created workspace view '{}' with {} files",
            view_name, files_restored
        );

        Ok(WorkspaceResult::with_warnings(
            view_path,
            warnings,
            duration.as_millis() as u64,
        ))
    }

    /// List all available views
    pub async fn list_views(&self) -> Result<Vec<ViewMetadata>, WorkspaceError> {
        let mut views = Vec::new();

        if !self.views_dir.exists() {
            return Ok(views);
        }

        for entry in std::fs::read_dir(&self.views_dir).map_err(|e| WorkspaceError::Io(e))? {
            let entry = entry.map_err(|e| WorkspaceError::Io(e))?;
            let path = entry.path();

            if path.is_dir() {
                let metadata_path = path.join(".workspace-view.json");
                if metadata_path.exists() {
                    let metadata_json = std::fs::read_to_string(&metadata_path)
                        .map_err(|e| WorkspaceError::Io(e))?;

                    if let Ok(metadata) = serde_json::from_str::<ViewMetadata>(&metadata_json) {
                        views.push(metadata);
                    }
                }
            }
        }

        // Sort by creation time (newest first)
        views.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(views)
    }

    /// Delete a workspace view
    pub async fn delete_view(&self, view_name: &str) -> Result<(), WorkspaceError> {
        let view_path = self.views_dir.join(view_name);

        if view_path.exists() {
            std::fs::remove_dir_all(&view_path).map_err(|e| WorkspaceError::Io(e))?;
            info!("Deleted workspace view '{}'", view_name);
        }

        Ok(())
    }

    /// Get metadata for a specific view
    pub async fn get_view_metadata(&self, view_name: &str) -> Result<ViewMetadata, WorkspaceError> {
        let view_path = self.views_dir.join(view_name);
        let metadata_path = view_path.join(".workspace-view.json");

        if !metadata_path.exists() {
            return Err(WorkspaceError::Storage(format!(
                "View '{}' not found",
                view_name
            )));
        }

        let metadata_json =
            std::fs::read_to_string(&metadata_path).map_err(|e| WorkspaceError::Io(e))?;

        let metadata: ViewMetadata =
            serde_json::from_str(&metadata_json).map_err(|e| WorkspaceError::Serialization(e))?;

        Ok(metadata)
    }
}

/// Metadata for a workspace view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewMetadata {
    /// Name of the view
    pub name: String,
    /// State ID this view represents
    pub state_id: StateId,
    /// When the view was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Source workspace path
    pub source_workspace: PathBuf,
    /// Number of files in the view
    pub files_count: usize,
    /// Total size of the view in bytes
    pub total_size: u64,
}

/// Rollback manager for workspace states
pub struct RollbackManager {
    /// Base workspace state manager
    manager: Arc<WorkspaceStateManager>,
    /// Backup directory for rollback operations
    backup_dir: PathBuf,
}

impl RollbackManager {
    /// Create a new rollback manager
    pub fn new(manager: Arc<WorkspaceStateManager>, backup_dir: impl AsRef<Path>) -> Self {
        Self {
            manager,
            backup_dir: backup_dir.as_ref().to_path_buf(),
        }
    }

    /// Perform a rollback to a specific state
    pub async fn rollback_to_state(
        &self,
        target_state: StateId,
        create_backup: bool,
    ) -> Result<RollbackResult, WorkspaceError> {
        let start_time = std::time::Instant::now();
        let mut warnings = Vec::new();

        info!("Starting rollback to state {:?}", target_state);

        // Get the target state
        let target_state_data = self.manager.get_state(target_state).await?;

        // Create backup of current state if requested
        if create_backup {
            match self.create_backup().await {
                Ok(backup_id) => {
                    info!("Created backup state {:?} before rollback", backup_id);
                }
                Err(e) => {
                    warnings.push(format!("Failed to create backup: {}", e));
                }
            }
        }

        // Get current state for comparison
        let current_state_result = self.manager.capture_state().await;
        let current_state = match current_state_result {
            Ok(result) => {
                if let Ok(state) = self.manager.get_state(result.data).await {
                    Some(state)
                } else {
                    None
                }
            }
            Err(_) => None,
        };

        // Perform the rollback
        let rollback_result = self
            .perform_rollback(&target_state_data, &current_state)
            .await?;

        let duration = start_time.elapsed();
        let result = RollbackResult {
            target_state,
            files_restored: rollback_result.files_restored,
            files_removed: rollback_result.files_removed,
            files_modified: rollback_result.files_modified,
            size_delta: rollback_result.size_delta,
            duration_ms: duration.as_millis() as u64,
            warnings,
            success: true,
        };

        info!(
            "Rollback completed: {} files restored, {} removed, {} modified",
            result.files_restored, result.files_removed, result.files_modified
        );

        Ok(result)
    }

    /// Create a backup of the current workspace state
    pub async fn create_backup(&self) -> Result<StateId, WorkspaceError> {
        let result = self.manager.capture_state().await?;
        info!("Created backup state {:?}", result.data);
        Ok(result.data)
    }

    /// List available backups
    pub async fn list_backups(&self) -> Result<Vec<StateId>, WorkspaceError> {
        self.manager.list_states().await
    }

    /// Restore from a backup
    pub async fn restore_from_backup(
        &self,
        backup_id: StateId,
    ) -> Result<RollbackResult, WorkspaceError> {
        self.rollback_to_state(backup_id, false).await
    }

    /// Perform the actual rollback operation
    async fn perform_rollback(
        &self,
        target_state: &WorkspaceState,
        current_state: &Option<WorkspaceState>,
    ) -> Result<RollbackOperation, WorkspaceError> {
        let mut files_restored = 0;
        let mut files_removed = 0;
        let mut files_modified = 0;
        let mut size_delta = 0i64;

        // Ensure workspace root exists
        std::fs::create_dir_all(&target_state.workspace_root).map_err(|e| WorkspaceError::Io(e))?;

        // Restore files from target state
        for (relative_path, file_state) in &target_state.files {
            let target_path = target_state.workspace_root.join(relative_path);
            let source_path = target_state.workspace_root.join(relative_path);

            // Create parent directories
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| WorkspaceError::Io(e))?;
            }

            // Check if file exists and is different
            let needs_restore = if target_path.exists() {
                // File exists, check if it's different
                if let Ok(metadata) = std::fs::metadata(&target_path) {
                    let current_size = metadata.len();
                    let current_modified = metadata.modified()?;

                    current_size != file_state.size
                        || current_modified
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                            != file_state.modified_at.timestamp() as u64
                } else {
                    true
                }
            } else {
                // File doesn't exist, needs to be created
                true
            };

            if needs_restore {
                if source_path.exists() {
                    std::fs::copy(&source_path, &target_path).map_err(|e| WorkspaceError::Io(e))?;

                    if target_path.exists() {
                        files_modified += 1;
                    } else {
                        files_restored += 1;
                    }
                }
            }
        }

        // Remove files that shouldn't exist in target state
        if let Some(current) = current_state {
            for relative_path in current.files.keys() {
                if !target_state.files.contains_key(relative_path) {
                    let file_path = target_state.workspace_root.join(relative_path);
                    if file_path.exists() {
                        std::fs::remove_file(&file_path).map_err(|e| WorkspaceError::Io(e))?;
                        files_removed += 1;
                    }
                }
            }
        }

        // Calculate size delta
        size_delta = target_state.total_size as i64;
        if let Some(current) = current_state {
            size_delta -= current.total_size as i64;
        }

        Ok(RollbackOperation {
            files_restored,
            files_removed,
            files_modified,
            size_delta,
        })
    }
}

/// Internal structure for rollback operation details
#[derive(Debug)]
struct RollbackOperation {
    files_restored: usize,
    files_removed: usize,
    files_modified: usize,
    size_delta: i64,
}
