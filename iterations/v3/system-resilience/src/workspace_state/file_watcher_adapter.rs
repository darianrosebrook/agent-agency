//! File Watcher Adapter
//!
//! Adapter for integrating agent-data-processing FileWatcher with unified workspace state manager.
//! Converts FileWatcher events into WorkspaceStateEvent for unified event handling.

use super::events::WorkspaceStateEvent;
use super::state_types::StateId;
use std::path::PathBuf;
use tokio::sync::broadcast;

/// File event type from file watcher
#[derive(Debug, Clone)]
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
}

/// Adapter for file watcher events
pub struct FileWatcherAdapter {
    event_sender: broadcast::Sender<WorkspaceStateEvent>,
    workspace_root: PathBuf,
}

impl FileWatcherAdapter {
    /// Create new file watcher adapter
    pub fn new(
        event_sender: broadcast::Sender<WorkspaceStateEvent>,
        workspace_root: impl AsRef<std::path::Path>,
    ) -> Self {
        Self {
            event_sender,
            workspace_root: workspace_root.as_ref().to_path_buf(),
        }
    }
    
    /// Handle file event from watcher
    pub async fn handle_file_event(
        &self,
        path: PathBuf,
        event_type: FileEventType,
        state_id: Option<StateId>,
    ) -> Result<(), String> {
        let relative_path = path.strip_prefix(&self.workspace_root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| path.clone());
        
        let event = match event_type {
            FileEventType::Created => WorkspaceStateEvent::FileCreated {
                path: relative_path,
                state_id,
            },
            FileEventType::Modified => WorkspaceStateEvent::FileModified {
                path: relative_path,
                state_id,
            },
            FileEventType::Deleted => WorkspaceStateEvent::FileDeleted {
                path: relative_path,
                state_id,
            },
        };
        
        self.event_sender.send(event)
            .map_err(|e| format!("Failed to send file event: {}", e))?;
        
        Ok(())
    }
    
    /// Check if file should generate embedding based on extension
    pub fn should_generate_embedding(&self, path: &std::path::Path, embedding_extensions: &[String]) -> bool {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();
        
        embedding_extensions.iter().any(|ext| ext.to_lowercase() == extension)
    }
}









