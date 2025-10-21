//! Artifact Manager
//!
//! Coordinates storage and retrieval of execution artifacts with versioning support.

use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::planning::types::{ExecutionArtifacts, CodeChange, ChangeType};

/// Artifact manager configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArtifactManagerConfig {
    /// Base storage path
    pub base_path: String,
    /// Enable compression for large artifacts
    pub enable_compression: bool,
    /// Maximum artifact size (MB)
    pub max_artifact_size_mb: u64,
    /// Enable versioning
    pub enable_versioning: bool,
    /// Retention policy (days)
    pub retention_days: u32,
    /// Enable integrity checks
    pub enable_integrity_checks: bool,
}

/// Artifact manager for storing and retrieving execution artifacts
pub struct ArtifactManager {
    config: ArtifactManagerConfig,
    storage: Arc<dyn super::storage::ArtifactStorage>,
    version_control: Arc<dyn super::versioning::VersionControl>,
}

impl ArtifactManager {
    pub fn new(
        config: ArtifactManagerConfig,
        storage: Arc<dyn super::storage::ArtifactStorage>,
        version_control: Arc<dyn super::versioning::VersionControl>,
    ) -> Self {
        Self {
            config,
            storage,
            version_control,
        }
    }

    /// Store execution artifacts
    pub async fn store_artifacts(
        &self,
        artifacts: &ExecutionArtifacts,
    ) -> Result<ArtifactMetadata, ArtifactError> {
        tracing::info!("Storing artifacts for task: {}", artifacts.task_id);

        // Validate artifact size
        self.validate_artifact_size(artifacts)?;

        // Generate artifact metadata
        let metadata = ArtifactMetadata {
            id: artifacts.id,
            task_id: artifacts.task_id,
            created_at: artifacts.generated_at,
            size_bytes: self.calculate_size(artifacts),
            checksum: self.calculate_checksum(artifacts).await?,
            version: self.get_next_version(artifacts.task_id).await?,
            compression_used: self.config.enable_compression,
            integrity_verified: self.config.enable_integrity_checks,
        };

        // Store artifacts
        self.storage.store(artifacts, &metadata).await?;

        // Create version control entry if enabled
        if self.config.enable_versioning {
            self.version_control.create_version(&metadata, artifacts).await?;
        }

        tracing::info!("Stored artifacts: {} ({} bytes)", metadata.id, metadata.size_bytes);
        Ok(metadata)
    }

    /// Retrieve execution artifacts
    pub async fn retrieve_artifacts(
        &self,
        task_id: Uuid,
        version: Option<String>,
    ) -> Result<ExecutionArtifacts, ArtifactError> {
        tracing::debug!("Retrieving artifacts for task: {} (version: {:?})", task_id, version);

        let artifacts = if let Some(version) = version {
            // Get specific version
            let metadata = self.version_control.get_version(task_id, &version).await?;
            self.storage.retrieve(&metadata).await?
        } else {
            // Get latest version
            let metadata = self.get_latest_metadata(task_id).await?;
            self.storage.retrieve(&metadata).await?
        };

        Ok(artifacts)
    }

    /// List all artifact versions for a task
    pub async fn list_versions(&self, task_id: Uuid) -> Result<Vec<ArtifactVersion>, ArtifactError> {
        if !self.config.enable_versioning {
            return Ok(Vec::new());
        }

        self.version_control.list_versions(task_id).await
    }

    /// Compare two artifact versions
    pub async fn compare_versions(
        &self,
        task_id: Uuid,
        from_version: &str,
        to_version: &str,
    ) -> Result<ArtifactDiff, ArtifactError> {
        if !self.config.enable_versioning {
            return Err(ArtifactError::VersioningDisabled);
        }

        let from_artifacts = self.retrieve_artifacts(task_id, Some(from_version.to_string())).await?;
        let to_artifacts = self.retrieve_artifacts(task_id, Some(to_version.to_string())).await?;

        let diff = self.compute_diff(&from_artifacts, &to_artifacts);
        Ok(diff)
    }

    /// Delete artifacts (with retention policy)
    pub async fn delete_artifacts(&self, task_id: Uuid) -> Result<(), ArtifactError> {
        tracing::info!("Deleting artifacts for task: {}", task_id);

        // Get all versions
        let versions = self.list_versions(task_id).await?;

        // Delete from storage
        for version in &versions {
            self.storage.delete(&version.metadata).await?;
        }

        // Delete version control entries
        if self.config.enable_versioning {
            self.version_control.delete_versions(task_id).await?;
        }

        Ok(())
    }

    /// Cleanup old artifacts based on retention policy
    pub async fn cleanup_old_artifacts(&self) -> Result<CleanupResult, ArtifactError> {
        let cutoff_date = Utc::now() - chrono::Duration::days(self.config.retention_days as i64);

        tracing::info!("Cleaning up artifacts older than: {}", cutoff_date);

        let old_artifacts = self.storage.find_old_artifacts(cutoff_date).await?;
        let mut deleted_count = 0;
        let mut bytes_freed = 0;

        for metadata in old_artifacts {
            self.storage.delete(&metadata).await?;
            if self.config.enable_versioning {
                self.version_control.delete_version(metadata.task_id, &metadata.version).await?;
            }
            deleted_count += 1;
            bytes_freed += metadata.size_bytes;
        }

        let result = CleanupResult {
            artifacts_deleted: deleted_count,
            bytes_freed,
            cutoff_date,
        };

        tracing::info!("Cleanup completed: {:?}", result);
        Ok(result)
    }

    /// Get artifact statistics
    pub async fn get_statistics(&self) -> Result<ArtifactStatistics, ArtifactError> {
        let total_artifacts = self.storage.count_artifacts().await?;
        let total_size = self.storage.total_size().await?;
        let versions_per_task = if self.config.enable_versioning {
            self.version_control.get_version_statistics().await?
        } else {
            HashMap::new()
        };

        Ok(ArtifactStatistics {
            total_artifacts,
            total_size_bytes: total_size,
            average_size_bytes: if total_artifacts > 0 { total_size / total_artifacts as u64 } else { 0 },
            versions_per_task,
            retention_days: self.config.retention_days,
            versioning_enabled: self.config.enable_versioning,
        })
    }

    /// Validate artifact size constraints
    fn validate_artifact_size(&self, artifacts: &ExecutionArtifacts) -> Result<(), ArtifactError> {
        let size_bytes = self.calculate_size(artifacts);
        let size_mb = size_bytes as f64 / (1024.0 * 1024.0);

        if size_mb > self.config.max_artifact_size_mb as f64 {
            return Err(ArtifactError::ArtifactTooLarge {
                size_mb,
                max_size_mb: self.config.max_artifact_size_mb,
            });
        }

        Ok(())
    }

    /// Calculate total artifact size
    fn calculate_size(&self, artifacts: &ExecutionArtifacts) -> u64 {
        // Rough estimation - in practice this would be more sophisticated
        let code_size: u64 = artifacts.code_changes.iter()
            .map(|c| c.diff.len() as u64)
            .sum();

        let test_size = artifacts.test_results.total as u64 * 100; // Estimate
        let coverage_size = artifacts.coverage.lines_total as u64 * 10; // Estimate
        let mutation_size = artifacts.mutation.mutants_generated as u64 * 50; // Estimate
        let lint_size = artifacts.lint.issues.len() as u64 * 200; // Estimate
        let types_size = artifacts.types.issues.len() as u64 * 150; // Estimate

        code_size + test_size + coverage_size + mutation_size + lint_size + types_size
    }

    /// Calculate artifact checksum for integrity
    async fn calculate_checksum(&self, artifacts: &ExecutionArtifacts) -> Result<String, ArtifactError> {
        use sha2::{Sha256, Digest};

        let mut hasher = Sha256::new();

        // Hash key artifact components
        hasher.update(format!("task_id:{}", artifacts.task_id));
        hasher.update(format!("code_changes:{}", artifacts.code_changes.len()));
        hasher.update(format!("test_passed:{}", artifacts.test_results.passed));
        hasher.update(format!("coverage:{:.2}", artifacts.coverage.coverage_percentage));

        let hash = hasher.finalize();
        Ok(format!("{:x}", hash))
    }

    /// Get next version number for task
    async fn get_next_version(&self, task_id: Uuid) -> Result<String, ArtifactError> {
        if !self.config.enable_versioning {
            return Ok("1".to_string());
        }

        let versions = self.version_control.list_versions(task_id).await?;
        let next_version = versions.len() + 1;
        Ok(next_version.to_string())
    }

    /// Get latest artifact metadata for task
    async fn get_latest_metadata(&self, task_id: Uuid) -> Result<ArtifactMetadata, ArtifactError> {
        if self.config.enable_versioning {
            let versions = self.version_control.list_versions(task_id).await?;
            if let Some(latest) = versions.last() {
                return Ok(latest.metadata.clone());
            }
        }

        // Fallback to storage lookup
        self.storage.find_latest(task_id).await?
    }

    /// Compute diff between two artifact sets
    fn compute_diff(&self, from: &ExecutionArtifacts, to: &ExecutionArtifacts) -> ArtifactDiff {
        let code_changes = self.diff_code_changes(&from.code_changes, &to.code_changes);

        let test_diff = TestDiff {
            passed_delta: to.test_results.passed as i32 - from.test_results.passed as i32,
            failed_delta: to.test_results.failed as i32 - from.test_results.failed as i32,
            skipped_delta: to.test_results.skipped as i32 - from.test_results.skipped as i32,
            coverage_delta: to.coverage.coverage_percentage - from.coverage.coverage_percentage,
        };

        let quality_improved = to.coverage.coverage_percentage >= from.coverage.coverage_percentage
            && to.test_results.failed <= from.test_results.failed
            && to.lint.errors <= from.lint.errors;

        ArtifactDiff {
            code_changes,
            test_diff,
            quality_improved,
            significant_changes: self.has_significant_changes(from, to),
        }
    }

    /// Diff code changes between versions
    fn diff_code_changes(&self, from: &[CodeChange], to: &[CodeChange]) -> Vec<CodeChangeDiff> {
        // Simple diff - in practice this would be more sophisticated
        let mut diffs = Vec::new();

        for to_change in to {
            let from_change = from.iter().find(|c| c.file_path == to_change.file_path);

            match from_change {
                Some(from_change) if from_change.diff != to_change.diff => {
                    diffs.push(CodeChangeDiff {
                        file_path: to_change.file_path.clone(),
                        change_type: ChangeType::Modified,
                        lines_added_delta: to_change.lines_added as i32 - from_change.lines_added as i32,
                        lines_removed_delta: to_change.lines_removed as i32 - from_change.lines_removed as i32,
                    });
                }
                None => {
                    diffs.push(CodeChangeDiff {
                        file_path: to_change.file_path.clone(),
                        change_type: ChangeType::Added,
                        lines_added_delta: to_change.lines_added as i32,
                        lines_removed_delta: to_change.lines_removed as i32,
                    });
                }
                _ => {} // No change
            }
        }

        diffs
    }

    /// Check if changes are significant
    fn has_significant_changes(&self, from: &ExecutionArtifacts, to: &ExecutionArtifacts) -> bool {
        let code_changed = from.code_changes.len() != to.code_changes.len();
        let tests_failed_worse = to.test_results.failed > from.test_results.failed;
        let coverage_dropped = to.coverage.coverage_percentage < from.coverage.coverage_percentage - 5.0;
        let more_lint_errors = to.lint.errors > from.lint.errors;

        code_changed || tests_failed_worse || coverage_dropped || more_lint_errors
    }
}

/// Artifact metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArtifactMetadata {
    pub id: Uuid,
    pub task_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub size_bytes: u64,
    pub checksum: String,
    pub version: String,
    pub compression_used: bool,
    pub integrity_verified: bool,
}

/// Artifact version information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArtifactVersion {
    pub metadata: ArtifactMetadata,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
}

/// Difference between code changes
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeChangeDiff {
    pub file_path: String,
    pub change_type: ChangeType,
    pub lines_added_delta: i32,
    pub lines_removed_delta: i32,
}

/// Test differences between versions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestDiff {
    pub passed_delta: i32,
    pub failed_delta: i32,
    pub skipped_delta: i32,
    pub coverage_delta: f64,
}

/// Artifact diff between versions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArtifactDiff {
    pub code_changes: Vec<CodeChangeDiff>,
    pub test_diff: TestDiff,
    pub quality_improved: bool,
    pub significant_changes: bool,
}

/// Cleanup operation result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CleanupResult {
    pub artifacts_deleted: usize,
    pub bytes_freed: u64,
    pub cutoff_date: DateTime<Utc>,
}

/// Artifact storage statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArtifactStatistics {
    pub total_artifacts: usize,
    pub total_size_bytes: u64,
    pub average_size_bytes: u64,
    pub versions_per_task: std::collections::HashMap<Uuid, usize>,
    pub retention_days: u32,
    pub versioning_enabled: bool,
}

pub type Result<T> = std::result::Result<T, ArtifactError>;

#[derive(Debug, thiserror::Error)]
pub enum ArtifactError {
    #[error("Artifact too large: {size_mb:.1}MB (max: {max_size_mb}MB)")]
    ArtifactTooLarge { size_mb: f64, max_size_mb: u64 },

    #[error("Artifact not found: {0}")]
    ArtifactNotFound(Uuid),

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("Versioning is disabled")]
    VersioningDisabled,

    #[error("Storage operation failed: {0}")]
    StorageError(String),

    #[error("Version control operation failed: {0}")]
    VersionControlError(String),

    #[error("Integrity check failed: {0}")]
    IntegrityError(String),

    #[error("IO operation failed: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization failed: {0}")]
    SerializationError(String),
}
