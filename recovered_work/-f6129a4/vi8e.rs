//! Snapshot management for non-Git environments

use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Manages snapshots for rollback functionality
#[derive(Debug)]
pub struct SnapshotManager {
    workspace_root: PathBuf,
    snapshots_dir: PathBuf,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(workspace_root: PathBuf) -> Self {
        let snapshots_dir = workspace_root.join(".snapshots");
        Self {
            workspace_root,
            snapshots_dir,
        }
    }

    /// Create a snapshot of the current workspace
    pub async fn create_snapshot(&self, message: &str) -> Result<String, SnapshotError> {
        // Ensure snapshots directory exists
        fs::create_dir_all(&self.snapshots_dir).await
            .map_err(|e| SnapshotError::IoError(e))?;

        // Generate snapshot ID
        let snapshot_id = Uuid::new_v4().to_string();
        let snapshot_dir = self.snapshots_dir.join(&snapshot_id);

        // Create snapshot directory
        fs::create_dir_all(&snapshot_dir).await
            .map_err(|e| SnapshotError::IoError(e))?;

        // Copy workspace files to snapshot
        self.copy_directory(&self.workspace_root, &snapshot_dir).await?;

        // Create metadata file
        let metadata = SnapshotMetadata {
            id: snapshot_id.clone(),
            message: message.to_string(),
            timestamp: Utc::now(),
            workspace_root: self.workspace_root.clone(),
        };

        let metadata_path = snapshot_dir.join("snapshot.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| SnapshotError::SerializationError(e))?;

        fs::write(&metadata_path, metadata_json).await
            .map_err(|e| SnapshotError::IoError(e))?;

        Ok(snapshot_id)
    }

    /// Rollback to a specific snapshot
    pub async fn rollback_to_snapshot(&self, snapshot_id: &str) -> Result<(), SnapshotError> {
        let snapshot_dir = self.snapshots_dir.join(snapshot_id);

        // Verify snapshot exists
        if !snapshot_dir.exists() {
            return Err(SnapshotError::SnapshotNotFound(snapshot_id.to_string()));
        }

        // Clear current workspace (except snapshots)
        self.clear_workspace().await?;

        // Copy snapshot back to workspace
        self.copy_directory(&snapshot_dir, &self.workspace_root).await?;

        Ok(())
    }

    /// List available snapshots
    pub async fn list_snapshots(&self) -> Result<Vec<SnapshotMetadata>, SnapshotError> {
        let mut snapshots = Vec::new();

        if !self.snapshots_dir.exists() {
            return Ok(snapshots);
        }

        let mut entries = fs::read_dir(&self.snapshots_dir).await
            .map_err(|e| SnapshotError::IoError(e))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| SnapshotError::IoError(e))? {

            if entry.file_type().await
                .map_err(|e| SnapshotError::IoError(e))?
                .is_dir() {

                let metadata_path = entry.path().join("snapshot.json");
                if metadata_path.exists() {
                    let metadata_json = fs::read_to_string(&metadata_path).await
                        .map_err(|e| SnapshotError::IoError(e))?;

                    let metadata: SnapshotMetadata = serde_json::from_str(&metadata_json)
                        .map_err(|e| SnapshotError::SerializationError(e))?;

                    snapshots.push(metadata);
                }
            }
        }

        // Sort by timestamp (newest first)
        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(snapshots)
    }

    /// Delete a snapshot
    pub async fn delete_snapshot(&self, snapshot_id: &str) -> Result<(), SnapshotError> {
        let snapshot_dir = self.snapshots_dir.join(snapshot_id);

        if !snapshot_dir.exists() {
            return Err(SnapshotError::SnapshotNotFound(snapshot_id.to_string()));
        }

        fs::remove_dir_all(&snapshot_dir).await
            .map_err(|e| SnapshotError::IoError(e))?;

        Ok(())
    }

    /// Copy directory contents recursively
    async fn copy_directory(&self, from: &Path, to: &Path) -> Result<(), SnapshotError> {
        let mut entries = fs::read_dir(from).await
            .map_err(|e| SnapshotError::IoError(e))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| SnapshotError::IoError(e))? {

            let entry_path = entry.path();
            let file_name = entry.file_name();
            let dest_path = to.join(file_name);

            // Skip snapshots directory
            if file_name == ".snapshots" {
                continue;
            }

            if entry.file_type().await
                .map_err(|e| SnapshotError::IoError(e))?
                .is_dir() {
                // Recursively copy directory
                fs::create_dir_all(&dest_path).await
                    .map_err(|e| SnapshotError::IoError(e))?;
                self.copy_directory(&entry_path, &dest_path).await?;
            } else {
                // Copy file
                fs::copy(&entry_path, &dest_path).await
                    .map_err(|e| SnapshotError::IoError(e))?;
            }
        }

        Ok(())
    }

    /// Clear workspace contents (except snapshots)
    async fn clear_workspace(&self) -> Result<(), SnapshotError> {
        let mut entries = fs::read_dir(&self.workspace_root).await
            .map_err(|e| SnapshotError::IoError(e))?;

        while let Some(entry) = entries.next_entry().await
            .map_err(|e| SnapshotError::IoError(e))? {

            let entry_path = entry.path();
            let file_name = entry.file_name();

            // Don't delete snapshots directory
            if file_name == ".snapshots" {
                continue;
            }

            if entry.file_type().await
                .map_err(|e| SnapshotError::IoError(e))?
                .is_dir() {
                fs::remove_dir_all(&entry_path).await
                    .map_err(|e| SnapshotError::IoError(e))?;
            } else {
                fs::remove_file(&entry_path).await
                    .map_err(|e| SnapshotError::IoError(e))?;
            }
        }

        Ok(())
    }
}

/// Snapshot metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotMetadata {
    pub id: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub workspace_root: PathBuf,
}

/// Errors from snapshot operations
#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),
}
