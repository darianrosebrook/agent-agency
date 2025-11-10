//! Integration hooks for workspace-state-manager
//!
//! Provides hooks to track workspace changes during processing and enable
//! rollback capabilities for failed or incorrect processing operations.

use schemars::JsonSchema;
use system_resilience::workspace_state::{WorkspaceStateManager, WorkspaceViewManager, RollbackManager, StateId};
use crate::{DataProcessingResult, DataProcessingError};
use std::sync::Arc;
use std::path::PathBuf;
use tracing::{info, warn, error};

/// Configuration for workspace integration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct WorkspaceConfig {
    pub enable_change_tracking: bool,
    pub enable_rollback: bool,
    pub create_processing_views: bool,
    pub workspace_root: PathBuf,
    pub views_directory: Option<PathBuf>,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            enable_change_tracking: true,
            enable_rollback: true,
            create_processing_views: true,
            workspace_root: PathBuf::from("."),
            views_directory: Some(PathBuf::from("processing-views")),
        }
    }
}

impl From<&WorkspaceConfig> for system_resilience::workspace_state::WorkspaceConfig {
    fn from(config: &WorkspaceConfig) -> Self {
        system_resilience::workspace_state::WorkspaceConfig {
            track_git: config.enable_change_tracking,
            compute_hashes: true,
            max_file_size: 1024 * 1024, // 1MB
            ignore_patterns: vec![
                "**/.git/**".to_string(),
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
            ],
            compress_states: true,
            max_states: 100,
            track_directories: true,
            default_capture_method: system_resilience::workspace_state::CaptureMethod::FullScan,
        }
    }
}

/// Hooks for integrating with workspace state manager
pub struct WorkspaceIntegrationHooks {
    workspace_manager: Arc<WorkspaceStateManager>,
    view_manager: Option<Arc<WorkspaceViewManager>>,
    rollback_manager: Option<Arc<RollbackManager>>,
    config: WorkspaceConfig,
    _pre_processing_state: Option<StateId>,
}

impl WorkspaceIntegrationHooks {
    /// Create new workspace integration hooks
    pub async fn new(config: &WorkspaceConfig) -> DataProcessingResult<Self> {
        let workspace_config = config.into();
        let storage = Box::new(system_resilience::workspace_state::FileStorage::new(&config.workspace_root, false));

        let workspace_manager = Arc::new(
            WorkspaceStateManager::new(&config.workspace_root, workspace_config, storage)
        );

        let view_manager = if config.create_processing_views {
            let views_dir = if let Some(ref dir) = config.views_directory {
                dir.clone()
            } else {
                config.workspace_root.join("processing-views")
            };
            Some(Arc::new(WorkspaceViewManager::new(
                Arc::clone(&workspace_manager),
                &views_dir
            )))
        } else {
            None
        };

        // Create rollback manager if rollback is enabled
        let rollback_manager = if config.enable_rollback {
            let backup_dir = config.workspace_root.join(".workspace-backups");
            // Ensure backup directory exists
            if let Err(e) = std::fs::create_dir_all(&backup_dir) {
                warn!("Failed to create backup directory {:?}: {}", backup_dir, e);
            }
            Some(Arc::new(RollbackManager::new(
                Arc::clone(&workspace_manager),
                &backup_dir
            )))
        } else {
            None
        };

        Ok(Self {
            workspace_manager,
            view_manager,
            rollback_manager,
            config: config.clone(),
            _pre_processing_state: None,
        })
    }

    /// Capture workspace state before processing begins
    pub async fn capture_pre_processing_state(&self) -> DataProcessingResult<StateId> {
        if !self.config.enable_change_tracking {
            return Ok(StateId::new()); // Return dummy ID
        }

        let result = self.workspace_manager.capture_state().await
            .map_err(|e| DataProcessingError::Other(format!("Failed to capture workspace state: {:?}", e)))?;

        Ok(result.data)
    }

    /// Commit workspace changes after successful processing
    pub async fn commit_processing_changes(&self, pre_state_id: StateId) -> DataProcessingResult<()> {
        if !self.config.enable_change_tracking {
            return Ok(());
        }

        // Capture current state to compare with pre-processing state
        let current_state_result = self.workspace_manager.capture_state().await
            .map_err(|e| DataProcessingError::Other(format!("Failed to capture current state: {:?}", e)))?;
        
        let current_state_id = current_state_result.data;

        // Create a processing view if view manager is available
        if let Some(ref view_manager) = self.view_manager {
            let view_name = format!("processing_commit_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
            let view_result = view_manager.create_view(current_state_id, Some(view_name.clone())).await
                .map_err(|e| DataProcessingError::Other(format!("Failed to create processing view: {:?}", e)))?;
            
            tracing::info!("Created processing view: {} at {:?}", view_name, view_result.data);
        }

        // Log the processing completion
        tracing::info!("Processing changes committed successfully. Pre-state: {}, Post-state: {}", 
                      pre_state_id, current_state_id);

        Ok(())
    }

    /// Rollback workspace changes after failed processing
    /// Uses RollbackManager to restore workspace state from pre_state_id snapshot
    pub async fn rollback_processing_changes(&self, pre_state_id: StateId) -> DataProcessingResult<()> {
        if !self.config.enable_rollback {
            return Ok(());
        }

        // Implemented: Real workspace rollback functionality using RollbackManager
        // Restores workspace state from pre_state_id snapshot with automatic state restoration
        
        info!("Starting rollback to state: {:?}", pre_state_id);
        
        // Validate that the target state exists
        match self.workspace_manager.get_state(pre_state_id).await {
            Ok(_) => {
                info!("Target state {:?} exists, proceeding with rollback", pre_state_id);
            }
            Err(e) => {
                error!("Target state {:?} not found: {:?}", pre_state_id, e);
                return Err(DataProcessingError::Other(format!(
                    "Cannot rollback: target state {:?} not found: {:?}",
                    pre_state_id, e
                )));
            }
        }
        
        // Perform rollback using RollbackManager
        if let Some(ref rollback_manager) = self.rollback_manager {
            // Create backup of current state before rollback (for safety)
            let create_backup = true;
            
            match rollback_manager.rollback_to_state(pre_state_id, create_backup).await {
                Ok(rollback_result) => {
                    info!(
                        "Rollback completed successfully: {} files restored, {} removed, {} modified (duration: {}ms)",
                        rollback_result.files_restored,
                        rollback_result.files_removed,
                        rollback_result.files_modified,
                        rollback_result.duration_ms
                    );
                    
                    // Log any warnings from rollback operation
                    if !rollback_result.warnings.is_empty() {
                        for warning in &rollback_result.warnings {
                            warn!("Rollback warning: {}", warning);
                        }
                    }
                    
                    // Verify rollback was successful
                    if !rollback_result.success {
                        error!("Rollback operation reported failure despite returning Ok");
                        return Err(DataProcessingError::Other(
                            "Rollback operation failed".to_string()
                        ));
                    }
                    
                    // Create a rollback view for debugging/verification
                    if let Some(ref view_manager) = self.view_manager {
                        let view_name = format!("rollback_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                        match view_manager.create_view(pre_state_id, Some(view_name.clone())).await {
                            Ok(view_result) => {
                                info!("Created rollback view: {} at {:?}", view_name, view_result.data);
                            }
                            Err(e) => {
                                warn!("Failed to create rollback view: {:?}", e);
                                // Don't fail the rollback if view creation fails
                            }
                        }
                    }
                    
                    Ok(())
                }
                Err(e) => {
                    error!("Rollback failed: {:?}", e);
                    
                    // Attempt partial recovery: create a view of the target state for manual recovery
                    if let Some(ref view_manager) = self.view_manager {
                        let view_name = format!("rollback_failed_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                        if let Ok(view_result) = view_manager.create_view(pre_state_id, Some(view_name.clone())).await {
                            warn!("Rollback failed, but created recovery view: {} at {:?}", view_name, view_result.data);
                        }
                    }
                    
                    Err(DataProcessingError::Other(format!(
                        "Rollback to state {:?} failed: {:?}",
                        pre_state_id, e
                    )))
                }
            }
        } else {
            // Rollback manager not available - fallback to view creation only
            warn!("Rollback manager not available, creating rollback view only");
            
            if let Some(ref view_manager) = self.view_manager {
                let view_name = format!("rollback_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
                let view_result = view_manager.create_view(pre_state_id, Some(view_name.clone())).await
                    .map_err(|e| DataProcessingError::Other(format!("Failed to create rollback view: {:?}", e)))?;
                
                info!("Created rollback view: {} at {:?}", view_name, view_result.data);
            }
            
            Err(DataProcessingError::Other(
                "Rollback manager not initialized - rollback not performed".to_string()
            ))
        }
    }

    /// Create a processing view for debugging or analysis
    pub async fn create_processing_view(&self, state_id: StateId, name: &str) -> DataProcessingResult<PathBuf> {
        if let Some(ref view_manager) = self.view_manager {
            let view_result = view_manager.create_view(state_id, Some(name.to_string())).await
                .map_err(|e| DataProcessingError::Other(format!("Failed to create processing view: {:?}", e)))?;
            
            tracing::info!("Created processing view '{}' at: {:?}", name, view_result.data);
            Ok(view_result.data)
        } else {
            Err(DataProcessingError::Other("View manager not available".to_string()))
        }
    }

    /// Get workspace statistics
    pub async fn get_workspace_stats(&self) -> DataProcessingResult<WorkspaceStats> {
        // Calculate disk usage
        let disk_usage_mb = self.calculate_disk_usage().await?;

        // Get view statistics if view manager is available
        let (total_views, avg_state_size_mb) = if let Some(ref view_manager) = self.view_manager {
            let views = view_manager.list_views().await
                .map_err(|e| DataProcessingError::Other(format!("Failed to list views: {:?}", e)))?;
            
            let total_views = views.len();
            let avg_state_size_mb = if total_views > 0 {
                disk_usage_mb / total_views as f64
            } else {
                0.0
            };
            
            (total_views, avg_state_size_mb)
        } else {
            (0, 0.0)
        };

        // Implemented: Accurate state counting from workspace state manager
        let total_states = match self.workspace_manager.list_states().await {
            Ok(state_ids) => {
                // Get actual count of states from workspace manager
                let state_count = state_ids.len();
                tracing::debug!("Retrieved {} states from workspace manager", state_count);
                state_count
            }
            Err(e) => {
                // Fallback to estimation if state listing fails
                tracing::warn!("Failed to list states from workspace manager: {:?}, using estimation", e);
                // Use views + 1 as fallback estimation (current state not yet captured)
                total_views + 1
            }
        };

        Ok(WorkspaceStats {
            total_states,
            total_views,
            disk_usage_mb,
            avg_state_size_mb,
        })
    }

    /// List available processing views
    pub async fn list_processing_views(&self) -> DataProcessingResult<Vec<String>> {
        if let Some(ref view_manager) = self.view_manager {
            let views = view_manager.list_views().await
                .map_err(|e| DataProcessingError::Other(format!("Failed to list views: {:?}", e)))?;
            
            // Filter for processing-related views and extract names
            let processing_views: Vec<String> = views.into_iter()
                .filter(|view| view.name.contains("processing") || view.name.contains("rollback"))
                .map(|view| view.name)
                .collect();
            
            Ok(processing_views)
        } else {
            Ok(vec![])
        }
    }

    /// Delete old processing views to save disk space
    pub async fn cleanup_old_views(&self, max_age_days: u32) -> DataProcessingResult<usize> {
        if let Some(ref view_manager) = self.view_manager {
            let views = view_manager.list_views().await
                .map_err(|e| DataProcessingError::Other(format!("Failed to list views: {:?}", e)))?;
            
            let cutoff_time = chrono::Utc::now() - chrono::Duration::days(max_age_days as i64);
            let mut deleted_count = 0;
            
            for view in views {
                // Try to parse creation time from view name (assuming format with timestamp)
                if let Some(timestamp_str) = view.name.split('_').last() {
                    if let Ok(view_time) = chrono::DateTime::parse_from_str(timestamp_str, "%Y%m%d_%H%M%S") {
                        if view_time.with_timezone(&chrono::Utc) < cutoff_time {
                            if let Err(e) = view_manager.delete_view(&view.name).await {
                                tracing::warn!("Failed to delete old view '{}': {:?}", view.name, e);
                            } else {
                                deleted_count += 1;
                                tracing::info!("Deleted old view: {}", view.name);
                            }
                        }
                    }
                }
            }
            
            Ok(deleted_count)
        } else {
            Ok(0)
        }
    }

    /// Calculate disk usage of workspace states and views
    async fn calculate_disk_usage(&self) -> DataProcessingResult<f64> {
        use std::fs;
        
        let mut total_size = 0u64;
        
        // Calculate size of workspace root
        if let Ok(entries) = fs::read_dir(&self.config.workspace_root) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len();
                    }
                }
            }
        }
        
        // Calculate size of views directory if it exists
        if let Some(ref views_dir) = self.config.views_directory {
            if let Ok(entries) = fs::read_dir(views_dir) {
                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            total_size += metadata.len();
                        }
                    }
                }
            }
        }
        
        // Convert bytes to MB
        Ok(total_size as f64 / (1024.0 * 1024.0))
    }
}

/// Workspace statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct WorkspaceStats {
    pub total_states: usize,
    pub total_views: usize,
    pub disk_usage_mb: f64,
    pub avg_state_size_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_workspace_hooks_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = WorkspaceConfig {
            workspace_root: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let hooks = WorkspaceIntegrationHooks::new(&config).await;
        assert!(hooks.is_ok());
    }

    #[test]
    fn test_workspace_config_defaults() {
        let config = WorkspaceConfig::default();
        assert!(config.enable_change_tracking);
        assert!(config.enable_rollback);
        assert!(config.create_processing_views);
        assert_eq!(config.workspace_root, PathBuf::from("."));
    }
}
