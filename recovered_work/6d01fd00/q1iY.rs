//! Artifact Versioning Interfaces
//!
//! Defines versioning interfaces and implementations for execution artifacts.

use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::manager::{ArtifactMetadata, ExecutionArtifacts, ArtifactVersion};
use agent_agency_database::DatabaseClient;

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
        metadata: &ArtifactMetadata,
        artifacts: &ExecutionArtifacts,
    ) -> Result<(), VersionControlError> {
        use tokio::fs;
        use tokio::process::Command;

        // Create directories
        let artifact_path = self.get_artifact_path(metadata);
        let metadata_path = self.get_metadata_path(metadata);

        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write artifact and metadata files
        let artifacts_json = serde_json::to_string_pretty(artifacts)?;
        let metadata_json = serde_json::to_string_pretty(metadata)?;

        fs::write(&artifact_path, artifacts_json).await?;
        fs::write(&metadata_path, metadata_json).await?;

        // Add to git
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["add", &artifact_path.to_string_lossy(), &metadata_path.to_string_lossy()])
            .output()
            .await?;

        if !output.status.success() {
            return Err(VersionControlError::GitError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        // Commit
        let commit_msg = format!("Add artifacts for task {} version {}", metadata.task_id, metadata.version);
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["commit", "-m", &commit_msg])
            .output()
            .await?;

        if !output.status.success() {
            return Err(VersionControlError::GitError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(())
    }

    async fn get_version(
        &self,
        task_id: Uuid,
        version: &str,
    ) -> Result<ArtifactVersion, VersionControlError> {
        use tokio::fs;

        let metadata_path = self.get_metadata_path(&ArtifactMetadata {
            id: Uuid::nil(), // Not needed for path calculation
            task_id,
            created_at: Utc::now(),
            size_bytes: 0,
            checksum: String::new(),
            version: version.to_string(),
            compression_used: false,
            integrity_verified: false,
        });

        let metadata_json = fs::read_to_string(&metadata_path).await?;
        let metadata: ArtifactMetadata = serde_json::from_str(&metadata_json)?;

        Ok(ArtifactVersion {
            metadata,
            created_at: metadata.created_at,
            description: Some(format!("Version {} of task {}", version, task_id)),
        })
    }

    async fn list_versions(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<ArtifactVersion>, VersionControlError> {
        use tokio::fs;
        use tokio::process::Command;

        let task_dir = format!("{}/{}", self.artifacts_subdir, task_id);

        // Use git to list files in the task directory
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["ls-files", &task_dir])
            .output()
            .await?;

        if !output.status.success() {
            return Err(VersionControlError::GitError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let files = String::from_utf8_lossy(&output.stdout);
        let mut versions = Vec::new();
        let mut version_map = HashMap::new();

        // Parse file names to extract versions
        for line in files.lines() {
            if let Some(filename) = line.split('/').last() {
                if filename.starts_with("metadata-") && filename.ends_with(".json") {
                    let version_part = &filename[9..filename.len() - 5]; // Remove "metadata-" and ".json"
                    version_map.insert(version_part.to_string(), line.to_string());
                }
            }
        }

        // Load metadata for each version
        for (version, _) in version_map {
            match self.get_version(task_id, &version).await {
                Ok(version_info) => versions.push(version_info),
                Err(e) => tracing::warn!("Failed to load version {}: {:?}", version, e),
            }
        }

        // Sort by creation time
        versions.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        Ok(versions)
    }

    async fn delete_version(
        &self,
        task_id: Uuid,
        version: &str,
    ) -> Result<(), VersionControlError> {
        use tokio::process::Command;

        let artifact_path = self.get_artifact_path(&ArtifactMetadata {
            id: Uuid::nil(),
            task_id,
            created_at: Utc::now(),
            size_bytes: 0,
            checksum: String::new(),
            version: version.to_string(),
            compression_used: false,
            integrity_verified: false,
        });

        let metadata_path = self.get_metadata_path(&ArtifactMetadata {
            id: Uuid::nil(),
            task_id,
            created_at: Utc::now(),
            size_bytes: 0,
            checksum: String::new(),
            version: version.to_string(),
            compression_used: false,
            integrity_verified: false,
        });

        // Remove files from git
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["rm", &artifact_path.to_string_lossy(), &metadata_path.to_string_lossy()])
            .output()
            .await?;

        if !output.status.success() {
            return Err(VersionControlError::GitError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        // Commit the removal
        let commit_msg = format!("Remove artifacts for task {} version {}", task_id, version);
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["commit", "-m", &commit_msg])
            .output()
            .await?;

        if !output.status.success() {
            return Err(VersionControlError::GitError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(())
    }

    async fn delete_versions(
        &self,
        task_id: Uuid,
    ) -> Result<(), VersionControlError> {
        let versions = self.list_versions(task_id).await?;

        for version in versions {
            self.delete_version(task_id, &version.metadata.version).await?;
        }

        Ok(())
    }

    async fn get_version_statistics(&self) -> Result<HashMap<Uuid, usize>, VersionControlError> {
        use tokio::process::Command;

        // Use git to find all task directories
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["ls-files", &self.artifacts_subdir])
            .output()
            .await?;

        if !output.status.success() {
            return Err(VersionControlError::GitError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        let files = String::from_utf8_lossy(&output.stdout);
        let mut stats = HashMap::new();

        // Count metadata files per task directory
        for line in files.lines() {
            if let Some(task_dir_part) = line.strip_prefix(&format!("{}/", self.artifacts_subdir)) {
                if let Some(task_id_str) = task_dir_part.split('/').next() {
                    if let Ok(task_id) = Uuid::parse_str(task_id_str) {
                        *stats.entry(task_id).or_insert(0) += 1;
                    }
                }
            }
        }

        // Divide by 2 since each version has both artifact and metadata files
        for count in stats.values_mut() {
            *count /= 2;
        }

        Ok(stats)
    }
}

/// Database-based version control
pub struct DatabaseVersionControl {
    // Database connection would go here
    // For now, this is a placeholder
}

impl DatabaseVersionControl {
    pub fn new(_connection_string: &str) -> Self {
        Self {}
    }
}

#[async_trait]
impl VersionControl for DatabaseVersionControl {
    async fn create_version(
        &self,
        _metadata: &ArtifactMetadata,
        _artifacts: &ExecutionArtifacts,
    ) -> Result<(), VersionControlError> {
        // TODO: Implement database versioning
        Err(VersionControlError::NotImplemented("Database versioning not yet implemented".to_string()))
    }

    async fn get_version(
        &self,
        _task_id: Uuid,
        _version: &str,
    ) -> Result<ArtifactVersion, VersionControlError> {
        // TODO: Implement database versioning
        Err(VersionControlError::NotImplemented("Database versioning not yet implemented".to_string()))
    }

    async fn list_versions(
        &self,
        _task_id: Uuid,
    ) -> Result<Vec<ArtifactVersion>, VersionControlError> {
        // TODO: Implement database versioning
        Err(VersionControlError::NotImplemented("Database versioning not yet implemented".to_string()))
    }

    async fn delete_version(
        &self,
        _task_id: Uuid,
        _version: &str,
    ) -> Result<(), VersionControlError> {
        // TODO: Implement database versioning
        Err(VersionControlError::NotImplemented("Database versioning not yet implemented".to_string()))
    }

    async fn delete_versions(
        &self,
        _task_id: Uuid,
    ) -> Result<(), VersionControlError> {
        // TODO: Implement database versioning
        Err(VersionControlError::NotImplemented("Database versioning not yet implemented".to_string()))
    }

    async fn get_version_statistics(&self) -> Result<HashMap<Uuid, usize>, VersionControlError> {
        // TODO: Implement database versioning
        Err(VersionControlError::NotImplemented("Database versioning not yet implemented".to_string()))
    }
}

pub type Result<T> = std::result::Result<T, VersionControlError>;

#[derive(Debug, thiserror::Error)]
pub enum VersionControlError {
    #[error("Git operation failed: {0}")]
    GitError(String),

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("UUID parsing error: {0}")]
    UuidError(#[from] uuid::Error),

    #[error("Feature not implemented: {0}")]
    NotImplemented(String),
}
