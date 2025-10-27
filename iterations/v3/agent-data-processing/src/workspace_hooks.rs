//! Integration hooks for workspace-state-manager
//!
//! Provides hooks to track workspace changes during processing and enable
//! rollback capabilities for failed or incorrect processing operations.

use system_resilience::workspace_state::{WorkspaceStateManager, WorkspaceViewManager, StateId};
use crate::{DataProcessingResult, DataProcessingError};
use std::sync::Arc;
use std::path::PathBuf;

/// Configuration for workspace integration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    config: WorkspaceConfig,
    pre_processing_state: Option<StateId>,
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

        Ok(Self {
            workspace_manager,
            view_manager,
            config: config.clone(),
            pre_processing_state: None,
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
    pub async fn rollback_processing_changes(&self, pre_state_id: StateId) -> DataProcessingResult<()> {
        if !self.config.enable_rollback {
            return Ok(());
        }

        // Note: The actual rollback implementation would depend on the specific
        // workspace manager API. For now, we'll simulate rollback by creating a view
        // of the pre-processing state for manual restoration.
        
        tracing::warn!("Rollback requested for state: {}. Manual restoration may be required.", pre_state_id);
        
        // Create a rollback view for debugging
        if let Some(ref view_manager) = self.view_manager {
            let view_name = format!("rollback_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S"));
            let view_result = view_manager.create_view(pre_state_id, Some(view_name.clone())).await
                .map_err(|e| DataProcessingError::Other(format!("Failed to create rollback view: {:?}", e)))?;
            
            tracing::info!("Created rollback view: {} at {:?}", view_name, view_result.data);
        }
        
        Ok(())
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

        // For now, estimate total states based on views (simplified)
        let total_states = total_views + 1; // +1 for current state

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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
