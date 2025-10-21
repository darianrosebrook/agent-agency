//! Artifact Storage Interfaces
//!
//! Defines storage interfaces and implementations for execution artifacts.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::manager::{ArtifactId, ArtifactMetadata, ExecutionArtifacts, ArtifactStorageError};
use agent_agency_database::client::DatabaseClient;

/// Artifact storage trait
#[async_trait]
pub trait ArtifactStorage: Send + Sync {
    /// Store execution artifacts
    async fn store(
        &self,
        artifacts: &ExecutionArtifacts,
        metadata: &ArtifactMetadata,
    ) -> Result<(), ArtifactStorageError>;

    /// Retrieve execution artifacts
    async fn retrieve(
        &self,
        metadata: &ArtifactMetadata,
    ) -> Result<ExecutionArtifacts, ArtifactStorageError>;

    /// Delete artifacts
    async fn delete(
        &self,
        metadata: &ArtifactMetadata,
    ) -> Result<(), ArtifactStorageError>;

    /// Find artifacts older than cutoff date
    async fn find_old_artifacts(
        &self,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<ArtifactMetadata>, ArtifactStorageError>;

    /// Find latest artifact for task
    async fn find_latest(
        &self,
        task_id: Uuid,
    ) -> Result<ArtifactMetadata, ArtifactStorageError>;

    /// Count total artifacts
    async fn count_artifacts(&self) -> Result<usize, ArtifactStorageError>;

    /// Get total storage size
    async fn total_size(&self) -> Result<u64, ArtifactStorageError>;
}

/// In-memory artifact storage for testing and development
#[derive(Debug, Default)]
pub struct InMemoryStorage {
    artifacts: Arc<RwLock<HashMap<ArtifactId, ExecutionArtifacts>>>,
    metadata: Arc<RwLock<HashMap<ArtifactId, ArtifactMetadata>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ArtifactStorage for InMemoryStorage {
    async fn store(
        &self,
        artifacts: &ExecutionArtifacts,
        metadata: &ArtifactMetadata,
    ) -> Result<(), ArtifactStorageError> {
        let mut artifacts_store = self.artifacts.write().await;
        let mut metadata_store = self.metadata.write().await;

        artifacts_store.insert(metadata.id, artifacts.clone());
        metadata_store.insert(metadata.id, metadata.clone());

        Ok(())
    }

    async fn retrieve(
        &self,
        metadata: &ArtifactMetadata,
    ) -> Result<ExecutionArtifacts, ArtifactStorageError> {
        let artifacts_store = self.artifacts.read().await;
        artifacts_store
            .get(&metadata.id)
            .cloned()
            .ok_or_else(|| ArtifactStorageError::NotFound(metadata.id))
    }

    async fn delete(
        &self,
        metadata: &ArtifactMetadata,
    ) -> Result<(), ArtifactStorageError> {
        let mut artifacts_store = self.artifacts.write().await;
        let mut metadata_store = self.metadata.write().await;

        artifacts_store.remove(&metadata.id);
        metadata_store.remove(&metadata.id);

        Ok(())
    }

    async fn find_old_artifacts(
        &self,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<ArtifactMetadata>, ArtifactStorageError> {
        let metadata_store = self.metadata.read().await;
        let old_artifacts = metadata_store
            .values()
            .filter(|metadata| metadata.created_at < cutoff_date)
            .cloned()
            .collect();

        Ok(old_artifacts)
    }

    async fn find_latest(
        &self,
        task_id: Uuid,
    ) -> Result<ArtifactMetadata, ArtifactStorageError> {
        let metadata_store = self.metadata.read().await;
        let mut candidates: Vec<_> = metadata_store
            .values()
            .filter(|metadata| metadata.task_id == task_id)
            .collect();

        candidates.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        candidates
            .first()
            .cloned()
            .cloned()
            .ok_or_else(|| ArtifactStorageError::NotFoundForTask(task_id))
    }

    async fn count_artifacts(&self) -> Result<usize, ArtifactStorageError> {
        let artifacts_store = self.artifacts.read().await;
        Ok(artifacts_store.len())
    }

    async fn total_size(&self) -> Result<u64, ArtifactStorageError> {
        let artifacts_store = self.artifacts.read().await;
        let total_size: u64 = artifacts_store
            .values()
            .map(|artifacts| serde_json::to_string(artifacts).unwrap_or_default().len() as u64)
            .sum();
        Ok(total_size)
    }
}

/// File system-based artifact storage
pub struct FileSystemStorage {
    base_path: PathBuf,
    enable_compression: bool,
}

impl FileSystemStorage {
    pub fn new(base_path: PathBuf, enable_compression: bool) -> Self {
        Self {
            base_path,
            enable_compression,
        }
    }

    /// Get artifact file path
    fn get_artifact_path(&self, metadata: &ArtifactMetadata) -> PathBuf {
        let task_dir = self.base_path.join(metadata.task_id.to_string());
        let version_dir = task_dir.join(&metadata.version);
        version_dir.join(format!("{}.json", metadata.id))
    }

    /// Get metadata file path
    fn get_metadata_path(&self, metadata: &ArtifactMetadata) -> PathBuf {
        let task_dir = self.base_path.join(metadata.task_id.to_string());
        let version_dir = task_dir.join(&metadata.version);
        version_dir.join("metadata.json")
    }
}

#[async_trait]
impl ArtifactStorage for FileSystemStorage {
    async fn store(
        &self,
        artifacts: &ExecutionArtifacts,
        metadata: &ArtifactMetadata,
    ) -> Result<(), ArtifactStorageError> {
        use tokio::fs;

        // Create directories
        let artifact_path = self.get_artifact_path(metadata);
        let metadata_path = self.get_metadata_path(metadata);

        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Serialize artifacts
        let artifacts_json = serde_json::to_string_pretty(artifacts)?;
        let metadata_json = serde_json::to_string_pretty(metadata)?;

        // Compress if enabled
        let artifacts_data = if self.enable_compression {
            self.compress_data(artifacts_json.as_bytes())?
        } else {
            artifacts_json.into_bytes()
        };

        // Write files
        fs::write(&artifact_path, &artifacts_data).await?;
        fs::write(&metadata_path, metadata_json).await?;

        Ok(())
    }

    async fn retrieve(
        &self,
        metadata: &ArtifactMetadata,
    ) -> Result<ExecutionArtifacts, ArtifactStorageError> {
        use tokio::fs;

        let artifact_path = self.get_artifact_path(metadata);

        let data = fs::read(&artifact_path).await?;
        let json_data = if self.enable_compression {
            self.decompress_data(&data)?
        } else {
            String::from_utf8(data)?
        };

        let artifacts: ExecutionArtifacts = serde_json::from_str(&json_data)?;
        Ok(artifacts)
    }

    async fn delete(
        &self,
        metadata: &ArtifactMetadata,
    ) -> Result<(), ArtifactStorageError> {
        use tokio::fs;

        let version_dir = self.get_artifact_path(metadata).parent().unwrap();

        // Remove entire version directory
        if version_dir.exists() {
            fs::remove_dir_all(version_dir).await?;
        }

        Ok(())
    }

    async fn find_old_artifacts(
        &self,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<ArtifactMetadata>, ArtifactStorageError> {
        use tokio::fs;

        let mut old_artifacts = Vec::new();

        // Walk through all task directories
        let mut entries = fs::read_dir(&self.base_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }

            // Check task directory
            let task_path = entry.path();
            let mut task_entries = fs::read_dir(&task_path).await?;
            while let Some(version_entry) = task_entries.next_entry().await? {
                if !version_entry.file_type().await?.is_dir() {
                    continue;
                }

                // Check metadata file
                let metadata_path = version_entry.path().join("metadata.json");
                if metadata_path.exists() {
                    let metadata_data = fs::read_to_string(&metadata_path).await?;
                    let metadata: ArtifactMetadata = serde_json::from_str(&metadata_data)?;

                    if metadata.created_at < cutoff_date {
                        old_artifacts.push(metadata);
                    }
                }
            }
        }

        Ok(old_artifacts)
    }

    async fn find_latest(
        &self,
        task_id: Uuid,
    ) -> Result<ArtifactMetadata, ArtifactStorageError> {
        use tokio::fs;

        let task_path = self.base_path.join(task_id.to_string());

        if !task_path.exists() {
            return Err(ArtifactStorageError::NotFound(format!("Task {}", task_id)));
        }

        let mut latest_metadata: Option<ArtifactMetadata> = None;
        let mut latest_version = 0u32;

        let mut entries = fs::read_dir(&task_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }

            let metadata_path = entry.path().join("metadata.json");
            if metadata_path.exists() {
                let metadata_data = fs::read_to_string(&metadata_path).await?;
                let metadata: ArtifactMetadata = serde_json::from_str(&metadata_data)?;

                let version_num: u32 = metadata.version.parse().unwrap_or(0);
                if version_num > latest_version {
                    latest_version = version_num;
                    latest_metadata = Some(metadata);
                }
            }
        }

        latest_metadata.ok_or_else(|| ArtifactStorageError::NotFound(format!("No artifacts for task {}", task_id)))
    }

    async fn count_artifacts(&self) -> Result<usize, ArtifactStorageError> {
        use tokio::fs;

        let mut count = 0;

        // Walk through all directories and count metadata files
        let mut stack = vec![self.base_path.clone()];
        while let Some(path) = stack.pop() {
            if let Ok(mut entries) = fs::read_dir(&path).await {
                while let Some(entry) = entries.next_entry().await? {
                    let entry_path = entry.path();
                    if entry.file_type().await?.is_dir() {
                        stack.push(entry_path);
                    } else if entry_path.file_name().unwrap_or_default() == "metadata.json" {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    async fn total_size(&self) -> Result<u64, ArtifactStorageError> {
        use tokio::fs;

        let mut total_size = 0u64;

        // Walk through all files and sum sizes
        let mut stack = vec![self.base_path.clone()];
        while let Some(path) = stack.pop() {
            if let Ok(mut entries) = fs::read_dir(&path).await {
                while let Some(entry) = entries.next_entry().await? {
                    let entry_path = entry.path();
                    if entry.file_type().await?.is_dir() {
                        stack.push(entry_path);
                    } else {
                        if let Ok(metadata) = entry.metadata().await {
                            total_size += metadata.len();
                        }
                    }
                }
            }
        }

        Ok(total_size)
    }
}

impl FileSystemStorage {
    /// Compress data using gzip
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, ArtifactStorageError> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        encoder.finish().map_err(|e| ArtifactStorageError::CompressionError(e.to_string()))
    }

    /// Decompress gzip data
    fn decompress_data(&self, data: &[u8]) -> Result<String, ArtifactStorageError> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(data);
        let mut decompressed = String::new();
        decoder.read_to_string(&mut decompressed)?;
        Ok(decompressed)
    }
}

/// Database-based artifact storage
pub struct DatabaseStorage {
    // Database connection would go here
    // For now, this is a placeholder
}

impl DatabaseStorage {
    pub fn new(_connection_string: &str) -> Self {
        Self {}
    }
}

#[async_trait]
impl ArtifactStorage for DatabaseStorage {
    async fn store(
        &self,
        _artifacts: &ExecutionArtifacts,
        _metadata: &ArtifactMetadata,
    ) -> Result<(), ArtifactStorageError> {
        // TODO: Implement database storage
        Err(ArtifactStorageError::NotImplemented("Database storage not yet implemented".to_string()))
    }

    async fn retrieve(
        &self,
        _metadata: &ArtifactMetadata,
    ) -> Result<ExecutionArtifacts, ArtifactStorageError> {
        // TODO: Implement database retrieval
        Err(ArtifactStorageError::NotImplemented("Database retrieval not yet implemented".to_string()))
    }

    async fn delete(
        &self,
        _metadata: &ArtifactMetadata,
    ) -> Result<(), ArtifactStorageError> {
        // TODO: Implement database deletion
        Err(ArtifactStorageError::NotImplemented("Database deletion not yet implemented".to_string()))
    }

    async fn find_old_artifacts(
        &self,
        _cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<ArtifactMetadata>, ArtifactStorageError> {
        // TODO: Implement database query
        Err(ArtifactStorageError::NotImplemented("Database queries not yet implemented".to_string()))
    }

    async fn find_latest(
        &self,
        _task_id: Uuid,
    ) -> Result<ArtifactMetadata, ArtifactStorageError> {
        // TODO: Implement database query
        Err(ArtifactStorageError::NotImplemented("Database queries not yet implemented".to_string()))
    }

    async fn count_artifacts(&self) -> Result<usize, ArtifactStorageError> {
        // TODO: Implement database count
        Err(ArtifactStorageError::NotImplemented("Database count not yet implemented".to_string()))
    }

    async fn total_size(&self) -> Result<u64, ArtifactStorageError> {
        // TODO: Implement database size calculation
        Err(ArtifactStorageError::NotImplemented("Database size calculation not yet implemented".to_string()))
    }
}

/// Database-backed artifact storage
#[derive(Debug)]
pub struct DatabaseArtifactStorage {
    db_client: std::sync::Arc<DatabaseClient>,
}

impl DatabaseArtifactStorage {
    /// Create a new database artifact storage
    pub fn new(db_client: std::sync::Arc<DatabaseClient>) -> Self {
        Self { db_client }
    }
}

#[async_trait]
impl ArtifactStorage for DatabaseArtifactStorage {
    async fn store(
        &self,
        artifacts: &ExecutionArtifacts,
        metadata: &ArtifactMetadata,
    ) -> Result<(), ArtifactStorageError> {
        // Serialize artifacts to JSON
        let artifact_data = serde_json::to_value(artifacts)
            .map_err(ArtifactStorageError::SerializationError)?;

        // Calculate size
        let size_bytes = serde_json::to_string(&artifact_data)
            .map_err(ArtifactStorageError::SerializationError)?
            .len() as i64;

        // Store metadata first
        let metadata_result = self.db_client.execute_parameterized_query(
            r#"
            INSERT INTO artifact_metadata (
                id, task_id, execution_id, session_id, version,
                artifact_types, total_size_bytes, metadata, expires_at, retention_policy
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                version = EXCLUDED.version,
                artifact_types = EXCLUDED.artifact_types,
                total_size_bytes = EXCLUDED.total_size_bytes,
                metadata = EXCLUDED.metadata,
                expires_at = EXCLUDED.expires_at
            "#,
            &[
                &metadata.id,
                &metadata.task_id,
                &metadata.execution_id,
                &metadata.session_id,
                &metadata.version,
                &metadata.artifact_types,
                &metadata.total_size_bytes,
                &serde_json::to_value(&metadata.metadata).map_err(ArtifactStorageError::SerializationError)?,
                &metadata.expires_at,
                &metadata.retention_policy,
            ],
        ).await;

        if let Err(e) = metadata_result {
            return Err(ArtifactStorageError::DatabaseError(format!("Failed to store metadata: {}", e)));
        }

        // Store individual artifacts
        for (artifact_type, artifact_value) in &artifacts.artifacts {
            let artifact_size = serde_json::to_string(artifact_value)
                .map_err(ArtifactStorageError::SerializationError)?
                .len() as i64;

            let result = self.db_client.execute_parameterized_query(
                r#"
                INSERT INTO execution_artifacts (
                    id, task_id, session_id, execution_id, artifact_type,
                    artifact_data, size_bytes, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                &[
                    &Uuid::new_v4(),
                    &metadata.task_id,
                    &metadata.session_id,
                    &metadata.execution_id,
                    &artifact_type,
                    &artifact_value,
                    &artifact_size,
                    &serde_json::json!({}),
                ],
            ).await;

            if let Err(e) = result {
                return Err(ArtifactStorageError::DatabaseError(format!("Failed to store artifact {}: {}", artifact_type, e)));
            }
        }

        Ok(())
    }

    async fn retrieve(
        &self,
        metadata: &ArtifactMetadata,
    ) -> Result<ExecutionArtifacts, ArtifactStorageError> {
        // Query all artifacts for this metadata
        let rows = self.db_client.execute_parameterized_query(
            r#"
            SELECT artifact_type, artifact_data
            FROM execution_artifacts
            WHERE task_id = $1 AND session_id = $2 AND execution_id = $3
            ORDER BY created_at ASC
            "#,
            &[&metadata.task_id, &metadata.session_id, &metadata.execution_id],
        ).await
        .map_err(|e| ArtifactStorageError::DatabaseError(format!("Failed to retrieve artifacts: {}", e)))?;

        let mut artifacts = std::collections::HashMap::new();

        // Process each artifact
        for row in rows {
            let artifact_type: String = row.get("artifact_type");
            let artifact_data: serde_json::Value = row.get("artifact_data");
            artifacts.insert(artifact_type, artifact_data);
        }

        if artifacts.is_empty() {
            return Err(ArtifactStorageError::NotFound(format!("No artifacts found for task {}", metadata.task_id)));
        }

        Ok(ExecutionArtifacts {
            artifacts,
            metadata: metadata.metadata.clone(),
        })
    }

    async fn delete(
        &self,
        metadata: &ArtifactMetadata,
    ) -> Result<(), ArtifactStorageError> {
        // Delete artifacts
        self.db_client.execute_parameterized_query(
            r#"
            DELETE FROM execution_artifacts
            WHERE task_id = $1 AND session_id = $2 AND execution_id = $3
            "#,
            &[&metadata.task_id, &metadata.session_id, &metadata.execution_id],
        ).await
        .map_err(|e| ArtifactStorageError::DatabaseError(format!("Failed to delete artifacts: {}", e)))?;

        // Delete metadata
        self.db_client.execute_parameterized_query(
            "DELETE FROM artifact_metadata WHERE id = $1",
            &[&metadata.id],
        ).await
        .map_err(|e| ArtifactStorageError::DatabaseError(format!("Failed to delete metadata: {}", e)))?;

        Ok(())
    }

    async fn find_old_artifacts(
        &self,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<ArtifactMetadata>, ArtifactStorageError> {
        let rows = self.db_client.execute_parameterized_query(
            r#"
            SELECT id, task_id, execution_id, session_id, version,
                   artifact_types, total_size_bytes, metadata,
                   created_at, expires_at, retention_policy
            FROM artifact_metadata
            WHERE created_at < $1
            ORDER BY created_at ASC
            "#,
            &[&cutoff_date],
        ).await
        .map_err(|e| ArtifactStorageError::DatabaseError(format!("Failed to find old artifacts: {}", e)))?;

        let mut results = Vec::new();
        for row in rows {
            let metadata: serde_json::Value = row.get("metadata");
            results.push(ArtifactMetadata {
                id: row.get("id"),
                task_id: row.get("task_id"),
                execution_id: row.get("execution_id"),
                session_id: row.get("session_id"),
                version: row.get("version"),
                artifact_types: row.get("artifact_types"),
                total_size_bytes: row.get("total_size_bytes"),
                created_at: row.get("created_at"),
                expires_at: row.get("expires_at"),
                retention_policy: row.get("retention_policy"),
                metadata: serde_json::from_value(metadata)
                    .map_err(ArtifactStorageError::SerializationError)?,
            });
        }

        Ok(results)
    }

    async fn find_latest(
        &self,
        task_id: Uuid,
    ) -> Result<ArtifactMetadata, ArtifactStorageError> {
        let rows = self.db_client.execute_parameterized_query(
            r#"
            SELECT id, task_id, execution_id, session_id, version,
                   artifact_types, total_size_bytes, metadata,
                   created_at, expires_at, retention_policy
            FROM artifact_metadata
            WHERE task_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            &[&task_id],
        ).await
        .map_err(|e| ArtifactStorageError::DatabaseError(format!("Failed to find latest artifact: {}", e)))?;

        if rows.is_empty() {
            return Err(ArtifactStorageError::NotFound(format!("No artifacts found for task {}", task_id)));
        }

        let row = &rows[0];
        let metadata: serde_json::Value = row.get("metadata");

        Ok(ArtifactMetadata {
            id: row.get("id"),
            task_id: row.get("task_id"),
            execution_id: row.get("execution_id"),
            session_id: row.get("session_id"),
            version: row.get("version"),
            artifact_types: row.get("artifact_types"),
            total_size_bytes: row.get("total_size_bytes"),
            created_at: row.get("created_at"),
            expires_at: row.get("expires_at"),
            retention_policy: row.get("retention_policy"),
            metadata: serde_json::from_value(metadata)
                .map_err(ArtifactStorageError::SerializationError)?,
        })
    }

    async fn count_artifacts(&self) -> Result<usize, ArtifactStorageError> {
        let rows = self.db_client.execute_parameterized_query(
            "SELECT COUNT(*) as count FROM execution_artifacts",
            &[],
        ).await
        .map_err(|e| ArtifactStorageError::DatabaseError(format!("Failed to count artifacts: {}", e)))?;

        if rows.is_empty() {
            return Ok(0);
        }

        let count: i64 = rows[0].get("count");
        Ok(count as usize)
    }

    async fn total_size(&self) -> Result<u64, ArtifactStorageError> {
        let rows = self.db_client.execute_parameterized_query(
            "SELECT COALESCE(SUM(size_bytes), 0) as total_size FROM execution_artifacts",
            &[],
        ).await
        .map_err(|e| ArtifactStorageError::DatabaseError(format!("Failed to calculate total size: {}", e)))?;

        if rows.is_empty() {
            return Ok(0);
        }

        let total_size: i64 = rows[0].get("total_size");
        Ok(total_size as u64)
    }
}

pub type Result<T> = std::result::Result<T, ArtifactStorageError>;

#[derive(Debug, thiserror::Error)]
pub enum ArtifactStorageError {
    #[error("Artifact not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Decompression error: {0}")]
    DecompressionError(String),

    #[error("Feature not implemented: {0}")]
    NotImplemented(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}
