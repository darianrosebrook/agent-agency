//! Artifact Versioning Interfaces
//!
//! Defines versioning interfaces and implementations for execution artifacts.

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::manager::{ArtifactMetadata, ArtifactVersion};
use crate::planning::types::ExecutionArtifacts;
use agent_agency_database::DatabaseClient;

/// Version control errors
#[derive(Debug, thiserror::Error)]
pub enum VersionControlError {
    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Git operation failed: {0}")]
    GitError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Version control trait
#[async_trait]
pub trait VersionControl: Send + Sync {
    /// Create a new version of artifacts
    async fn create_version(
        &self,
        metadata: &ArtifactMetadata,
        artifacts: &ExecutionArtifacts,
    ) -> Result<(), VersionControlError>;

    /// Get a specific version
    async fn get_version(
        &self,
        task_id: Uuid,
        version: &str,
    ) -> Result<ArtifactVersion, VersionControlError>;

    /// List all versions for a task
    async fn list_versions(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<ArtifactVersion>, VersionControlError>;

    /// Delete a specific version
    async fn delete_version(
        &self,
        task_id: Uuid,
        version: &str,
    ) -> Result<(), VersionControlError>;

    /// Delete all versions for a task
    async fn delete_versions(
        &self,
        task_id: Uuid,
    ) -> Result<(), VersionControlError>;

    /// Get version statistics
    async fn get_version_statistics(&self) -> Result<HashMap<Uuid, usize>, VersionControlError>;
}

/// Git-based version control for artifacts
pub struct GitVersionControl {
    repo_path: std::path::PathBuf,
    artifacts_subdir: String,
}

impl GitVersionControl {
    pub fn new(repo_path: std::path::PathBuf, artifacts_subdir: String) -> Self {
        Self {
            repo_path,
            artifacts_subdir,
        }
    }

    /// Get artifact file path in git repo
    fn get_artifact_path(&self, metadata: &ArtifactMetadata) -> std::path::PathBuf {
        let task_dir = format!("{}/{}", self.artifacts_subdir, metadata.task_id);
        self.repo_path.join(format!("{}/artifacts-{}.json", task_dir, metadata.version))
    }

    /// Get metadata file path in git repo
    fn get_metadata_path(&self, metadata: &ArtifactMetadata) -> std::path::PathBuf {
        let task_dir = format!("{}/{}", self.artifacts_subdir, metadata.task_id);
        self.repo_path.join(format!("{}/metadata-{}.json", task_dir, metadata.version))
    }
}

#[async_trait]
impl VersionControl for GitVersionControl {
    async fn create_version(
        &self,
        _metadata: &ArtifactMetadata,
        _artifacts: &ExecutionArtifacts,
    ) -> Result<(), VersionControlError> {
        // Git-based versioning is deprecated - use DatabaseVersionControl instead
        Err(VersionControlError::InternalError("Git-based versioning is not implemented".to_string()))
    }

    async fn get_version(
        &self,
        _task_id: Uuid,
        _version: &str,
    ) -> Result<ArtifactVersion, VersionControlError> {
        Err(VersionControlError::InternalError("Git-based versioning is not implemented".to_string()))
    }

    async fn list_versions(
        &self,
        _task_id: Uuid,
    ) -> Result<Vec<ArtifactVersion>, VersionControlError> {
        Err(VersionControlError::InternalError("Git-based versioning is not implemented".to_string()))
    }

    async fn delete_version(
        &self,
        _task_id: Uuid,
        _version: &str,
    ) -> Result<(), VersionControlError> {
        Err(VersionControlError::InternalError("Git-based versioning is not implemented".to_string()))
    }

    async fn delete_versions(
        &self,
        _task_id: Uuid,
    ) -> Result<(), VersionControlError> {
        Err(VersionControlError::InternalError("Git-based versioning is not implemented".to_string()))
    }

    async fn get_version_statistics(&self) -> Result<HashMap<Uuid, usize>, VersionControlError> {
        Err(VersionControlError::InternalError("Git-based versioning is not implemented".to_string()))
    }
}
