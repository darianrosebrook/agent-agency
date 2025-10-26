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
    pub async fn commit_processing_changes(&self, _pre_state_id: StateId) -> DataProcessingResult<()> {
        if !self.config.enable_change_tracking {
            return Ok(());
        }

        // For now, just mark as successful - the workspace manager automatically tracks changes
        // In a full implementation, we might create processing-specific views or tags
        Ok(())
    }

    /// Rollback workspace changes after failed processing
    pub async fn rollback_processing_changes(&self, _pre_state_id: StateId) -> DataProcessingResult<()> {
        if !self.config.enable_rollback {
            return Ok(());
        }

        // Note: The actual rollback implementation would depend on the specific
        // workspace manager API. For now, this is a placeholder.
        // The workspace manager might support restoring to a previous state.
        Ok(())
    }

    /// Create a processing view for debugging or analysis
    pub async fn create_processing_view(&self, _state_id: StateId, _name: &str) -> DataProcessingResult<PathBuf> {
        // Placeholder - would create a view of the workspace at the given state
        Ok(PathBuf::from("processing_view_placeholder"))
    }

    /// Get workspace statistics
    pub async fn get_workspace_stats(&self) -> DataProcessingResult<WorkspaceStats> {
        // Placeholder - would get actual stats from workspace manager
        Ok(WorkspaceStats {
            total_states: 0,
            total_views: 0,
            disk_usage_mb: 0.0,
            avg_state_size_mb: 0.0,
        })
    }

    /// List available processing views
    pub async fn list_processing_views(&self) -> DataProcessingResult<Vec<String>> {
        // Placeholder - would list processing-related views
        Ok(vec![])
    }

    /// Delete old processing views to save disk space
    pub async fn cleanup_old_views(&self, _max_age_days: u32) -> DataProcessingResult<usize> {
        // Placeholder - would clean up old views
        Ok(0)
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
