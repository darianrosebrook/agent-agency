//! Builder for unified workspace state manager
//!
//! Provides a fluent API for constructing UnifiedWorkspaceStateManager instances
//! with optional components (file watching, embeddings, context generation).

use super::unified::{
    UnifiedWorkspaceStateManager, UnifiedWorkspaceConfig,
    FileWatchConfig, ContextGenerationConfig, MetricsConfig,
};
use super::state_types::{WorkspaceConfig, WorkspaceError};
use super::StateStorage;
use std::path::{Path, PathBuf};

/// Builder for unified workspace state manager
pub struct UnifiedWorkspaceStateManagerBuilder {
    workspace_root: PathBuf,
    config: UnifiedWorkspaceConfig,
    state_storage: Option<Box<dyn StateStorage>>,
}

impl UnifiedWorkspaceStateManagerBuilder {
    /// Create new builder
    pub fn new(workspace_root: impl AsRef<Path>) -> Self {
        Self {
            workspace_root: workspace_root.as_ref().to_path_buf(),
            config: UnifiedWorkspaceConfig::default(),
            state_storage: None,
        }
    }
    
    /// Set state management configuration
    pub fn with_state_config(mut self, config: WorkspaceConfig) -> Self {
        self.config.state_config = config;
        self
    }
    
    /// Enable and configure file watching
    pub fn with_file_watching(mut self, config: FileWatchConfig) -> Self {
        self.config.watch_config = Some(config);
        self
    }
    
    /// Enable and configure context generation
    pub fn with_context_generation(mut self, config: ContextGenerationConfig) -> Self {
        self.config.context_config = Some(config);
        self
    }
    
    /// Set metrics configuration
    pub fn with_metrics_config(mut self, config: MetricsConfig) -> Self {
        self.config.metrics_config = config;
        self
    }
    
    /// Set storage backend
    pub fn with_storage(mut self, storage: Box<dyn StateStorage>) -> Self {
        self.state_storage = Some(storage);
        self
    }
    
    /// Build the unified workspace state manager
    pub fn build(self) -> Result<UnifiedWorkspaceStateManager, WorkspaceError> {
        // Use provided storage or create default file storage
        let storage = if let Some(storage) = self.state_storage {
            storage
        } else {
            use super::storage::FileStorage;
            let storage_path = self.workspace_root.join(".workspace-state");
            Box::new(FileStorage::new(&storage_path, self.config.state_config.compress_states))
        };
        
        Ok(UnifiedWorkspaceStateManager::new(
            &self.workspace_root,
            self.config,
            storage,
        ))
    }
}

