//! File Watcher Trait
//!
//! Trait for file watching functionality to avoid circular dependencies.
//! Implementations can be provided by agent-data-processing or other crates.

use super::events::WorkspaceStateEvent;
use std::path::PathBuf;
use tokio::sync::broadcast;

/// Trait for file watching functionality
#[async_trait::async_trait]
pub trait FileWatcherTrait: Send + Sync {
    /// Start watching for file changes
    async fn start_watching(&self) -> Result<(), String>;
    
    /// Stop watching for file changes
    async fn stop_watching(&self) -> Result<(), String>;
    
    /// Get the paths being watched
    fn watch_paths(&self) -> &[PathBuf];
}

/// File watcher event handler
pub struct FileWatcherEventHandler {
    pub(crate) event_sender: broadcast::Sender<WorkspaceStateEvent>,
    workspace_root: PathBuf,
}

impl FileWatcherEventHandler {
    /// Create new event handler
    pub fn new(
        event_sender: broadcast::Sender<WorkspaceStateEvent>,
        workspace_root: impl AsRef<std::path::Path>,
    ) -> Self {
        Self {
            event_sender,
            workspace_root: workspace_root.as_ref().to_path_buf(),
        }
    }
    
    /// Handle file created event
    pub fn handle_file_created(&self, path: PathBuf, state_id: Option<super::state_types::StateId>) {
        let relative_path = self.normalize_path(path);
        let _ = self.event_sender.send(WorkspaceStateEvent::FileCreated {
            path: relative_path,
            state_id,
        });
    }
    
    /// Handle file modified event
    pub fn handle_file_modified(&self, path: PathBuf, state_id: Option<super::state_types::StateId>) {
        let relative_path = self.normalize_path(path);
        let _ = self.event_sender.send(WorkspaceStateEvent::FileModified {
            path: relative_path,
            state_id,
        });
    }
    
    /// Handle file deleted event
    pub fn handle_file_deleted(&self, path: PathBuf, state_id: Option<super::state_types::StateId>) {
        let relative_path = self.normalize_path(path);
        let _ = self.event_sender.send(WorkspaceStateEvent::FileDeleted {
            path: relative_path,
            state_id,
        });
    }
    
    /// Normalize path to be relative to workspace root
    fn normalize_path(&self, path: PathBuf) -> PathBuf {
        path.strip_prefix(&self.workspace_root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| path)
    }
}

