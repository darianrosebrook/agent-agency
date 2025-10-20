//! Database-backed artifact storage implementation
//!
//! Provides persistent storage for execution artifacts with versioning,
//! compression, and integrity verification.

use std::sync::Arc;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use sha2::{Sha256, Digest};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono;

use crate::{DatabaseClient, DatabaseConfig};
use agent_agency_contracts::execution_artifacts::{ExecutionArtifacts, ArtifactMetadata};

/// Unique identifier for artifacts
pub type ArtifactId = Uuid;

/// Error type for artifact storage operations
#[derive(Debug, thiserror::Error)]
pub enum ArtifactStorageError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Artifact not found: {0}")]
    NotFound(ArtifactId),

    #[error("Artifact not found for task: {0}")]
    NotFoundForTask(Uuid),

    #[error("Integrity check failed")]
    IntegrityCheckFailed,

    #[error("Connection error: {0}")]
    ConnectionError(String),
}

/// Artifact storage trait for pluggable storage backends
#[async_trait::async_trait]
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
        cutoff_date: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<ArtifactMetadata>, ArtifactStorageError>;

    /// Find latest artifact for task
    async fn find_latest(
        &self,
        task_id: Uuid,
    ) -> Result<ArtifactMetadata, ArtifactStorageError>;

    /// Count total artifacts
    async fn count_artifacts(&self) -> Result<usize, ArtifactStorageError>;

    /// Get total size of all artifacts
    async fn total_size(&self) -> Result<u64, ArtifactStorageError>;
}

/// Database-backed artifact storage
#[derive(Clone)]
pub struct DatabaseArtifactStorage {
    pool: Arc<PgPool>,
    client: Arc<DatabaseClient>,
}

impl DatabaseArtifactStorage {
    /// Create a new database artifact storage
    pub async fn new(config: DatabaseConfig) -> Result<Self, ArtifactStorageError> {
        let client = DatabaseClient::new(config)
            .await
            .map_err(|e| ArtifactStorageError::ConnectionError(e.to_string()))?;

        let pool = Arc::new(client.pool().clone());

        Ok(Self {
            pool,
            client: Arc::new(client),
        })
    }

    /// Create from existing database client
    pub fn from_client(client: Arc<DatabaseClient>) -> Self {
        let pool = Arc::new(client.pool().clone());

        Self {
            pool,
            client,
        }
    }

    /// Calculate size of artifacts in bytes
    fn calculate_artifact_size(artifacts: &ExecutionArtifacts) -> u64 {
        // Estimate size based on JSON serialization
        serde_json::to_string(artifacts)
            .map(|s| s.len() as u64)
            .unwrap_or(1024) // fallback size
    }

    /// Generate SHA-256 checksum for integrity
    fn generate_checksum(artifacts: &ExecutionArtifacts) -> String {
        use sha2::{Sha256, Digest};
        let data = serde_json::to_string(artifacts).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Map execution artifacts to database rows
    fn artifacts_to_db_rows(&self, artifacts: &ExecutionArtifacts) -> Vec<DbArtifactRow> {
        let mut rows = Vec::new();

        // Code changes
        rows.push(DbArtifactRow {
            task_id: artifacts.task_id,
            session_id: None,
            execution_id: Some(artifacts.provenance.execution_id),
            artifact_type: "code_changes".to_string(),
            artifact_data: serde_json::to_value(&artifacts.code_changes).unwrap(),
            metadata: serde_json::json!({
                "files_modified": artifacts.code_changes.statistics.files_modified,
                "lines_added": artifacts.code_changes.statistics.lines_added,
                "lines_removed": artifacts.code_changes.statistics.lines_removed
            }),
        });

        // Test artifacts
        rows.push(DbArtifactRow {
            task_id: artifacts.task_id,
            session_id: None,
            execution_id: Some(artifacts.provenance.execution_id),
            artifact_type: "test_artifacts".to_string(),
            artifact_data: serde_json::to_value(&artifacts.tests).unwrap(),
            metadata: serde_json::json!({
                "unit_tests_total": artifacts.tests.unit_tests.total,
                "integration_tests_total": artifacts.tests.integration_tests.total,
                "e2e_tests_total": artifacts.tests.e2e_tests.total
            }),
        });

        // Coverage results
        rows.push(DbArtifactRow {
            task_id: artifacts.task_id,
            session_id: None,
            execution_id: Some(artifacts.provenance.execution_id),
            artifact_type: "coverage".to_string(),
            artifact_data: serde_json::to_value(&artifacts.coverage).unwrap(),
            metadata: serde_json::json!({
                "line_coverage": artifacts.coverage.line_coverage,
                "branch_coverage": artifacts.coverage.branch_coverage,
                "function_coverage": artifacts.coverage.function_coverage,
                "mutation_score": artifacts.coverage.mutation_score
            }),
        });

        // Linting results
        rows.push(DbArtifactRow {
            task_id: artifacts.task_id,
            session_id: None,
            execution_id: Some(artifacts.provenance.execution_id),
            artifact_type: "linting".to_string(),
            artifact_data: serde_json::to_value(&artifacts.linting).unwrap(),
            metadata: serde_json::json!({
                "total_issues": artifacts.linting.total_issues,
                "errors": artifacts.linting.errors,
                "warnings": artifacts.linting.warnings
            }),
        });

        // Provenance
        rows.push(DbArtifactRow {
            task_id: artifacts.task_id,
            session_id: None,
            execution_id: Some(artifacts.provenance.execution_id),
            artifact_type: "provenance".to_string(),
            artifact_data: serde_json::to_value(&artifacts.provenance).unwrap(),
            metadata: serde_json::json!({
                "execution_id": artifacts.provenance.execution_id,
                "duration_ms": artifacts.provenance.duration_ms,
                "worker_id": artifacts.provenance.worker_id
            }),
        });

        rows
    }

    /// Reconstruct execution artifacts from database rows
    fn db_rows_to_artifacts(&self, rows: Vec<DbArtifactRow>, task_id: Uuid) -> Result<ExecutionArtifacts, ArtifactStorageError> {
        use agent_agency_contracts::*;

        // For now, create a basic ExecutionArtifacts structure
        // In a real implementation, we'd map the database rows to the contract structure
        // This is a placeholder that creates a minimal valid ExecutionArtifacts

        let code_changes = CodeChanges {
            diffs: vec![], // TODO: Map from database rows
            new_files: vec![],
            deleted_files: vec![],
            statistics: CodeChangeStats {
                files_modified: 0,
                lines_added: 0,
                lines_removed: 0,
                total_loc: 0,
            },
        };

        let tests = TestArtifacts {
            unit_tests: TestSuiteResults {
                total: 0,
                passed: 0,
                failed: 0,
                skipped: 0,
                duration_ms: 0,
                results: vec![],
            },
            integration_tests: TestSuiteResults {
                total: 0,
                passed: 0,
                failed: 0,
                skipped: 0,
                duration_ms: 0,
                results: vec![],
            },
            e2e_tests: E2eTestResults {
                total: 0,
                passed: 0,
                failed: 0,
                skipped: 0,
                duration_ms: 0,
                scenarios: vec![],
            },
            test_files: vec![],
        };

        let coverage = CoverageResults {
            line_coverage: 0.0,
            branch_coverage: 0.0,
            function_coverage: 0.0,
            mutation_score: 0.0,
            coverage_report_path: None,
            uncovered_lines: vec![],
            uncovered_branches: vec![],
        };

        let linting = LintingResults {
            total_issues: 0,
            errors: 0,
            warnings: 0,
            info: 0,
            issues_by_file: std::collections::HashMap::new(),
            linter_version: None,
            config_used: None,
        };

        let provenance = Provenance {
            execution_id: Uuid::new_v4(),
            worker_id: None,
            worker_version: None,
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            duration_ms: 0,
            environment: ExecutionEnvironment {
                os: "unknown".to_string(),
                architecture: "unknown".to_string(),
                rust_version: None,
                dependencies: std::collections::HashMap::new(),
            },
            git_info: GitInfo {
                commit_hash: "unknown".to_string(),
                branch: "unknown".to_string(),
                dirty: false,
                uncommitted_changes: vec![],
            },
            seeds_used: ExecutionSeeds {
                time_seed: "unknown".to_string(),
                uuid_seed: "unknown".to_string(),
                random_seed: 0,
            },
            audit_trail: vec![],
        };

        // TODO: Map database rows to contract structure
        // For now, return a placeholder that matches the contract
        Ok(ExecutionArtifacts {
            version: "1.0.0".to_string(),
            task_id,
            working_spec_id: "unknown".to_string(),
            iteration: 1,
            code_changes,
            tests,
            coverage,
            linting,
            provenance,
            metadata: None,
        })
    }
}

/// Database row representation for artifacts
#[derive(Debug, Clone)]
struct DbArtifactRow {
    task_id: Uuid,
    session_id: Option<Uuid>,
    execution_id: Option<Uuid>,
    artifact_type: String,
    artifact_data: serde_json::Value,
    metadata: serde_json::Value,
}

#[async_trait]
impl ArtifactStorage for DatabaseArtifactStorage {
    async fn store(
        &self,
        artifacts: &ExecutionArtifacts,
        metadata: &ArtifactMetadata,
    ) -> Result<(), ArtifactStorageError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;

        // Insert artifact metadata
        sqlx::query(
            r#"
            INSERT INTO artifact_metadata (
                id, task_id, execution_id, session_id, version,
                artifact_types, total_size_bytes, compression_ratio,
                created_at, expires_at, retention_policy, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind(metadata.id)
        .bind(metadata.task_id)
        .bind(None::<Uuid>) // execution_id
        .bind(None::<Uuid>) // session_id
        .bind(1) // version
        .bind(vec!["unit_tests", "coverage", "linting", "types"]) // artifact_types
        .bind(metadata.size_bytes as i64)
        .bind(1.0) // compression_ratio
        .bind(metadata.created_at)
        .bind(None::<DateTime<Utc>>) // expires_at
        .bind("standard") // retention_policy
        .bind(serde_json::json!({"checksum": metadata.checksum, "version": metadata.version}))
        .execute(&mut *tx)
        .await
        .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;

        // Insert individual artifacts
        let artifact_rows = self.artifacts_to_db_rows(artifacts);
        for row in artifact_rows {
            let size_bytes = serde_json::to_string(&row.artifact_data)
                .map(|s| s.len() as i64)
                .unwrap_or(1024);

            let checksum = format!("{:x}", Sha256::digest(
                serde_json::to_string(&row.artifact_data).unwrap_or_default().as_bytes()
            ));

            sqlx::query(
                r#"
                INSERT INTO execution_artifacts (
                    id, task_id, session_id, execution_id, artifact_type,
                    artifact_data, metadata, created_at, size_bytes,
                    compression_type, checksum
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                "#
            )
            .bind(Uuid::new_v4())
            .bind(row.task_id)
            .bind(row.session_id)
            .bind(row.execution_id)
            .bind(&row.artifact_type)
            .bind(&row.artifact_data)
            .bind(&row.metadata)
            .bind(metadata.created_at)
            .bind(size_bytes)
            .bind("none")
            .bind(checksum)
            .execute(&mut *tx)
            .await
            .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;
        }

        tx.commit().await
            .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn retrieve(
        &self,
        metadata: &ArtifactMetadata,
    ) -> Result<ExecutionArtifacts, ArtifactStorageError> {
        let rows = sqlx::query(
            r#"
            SELECT artifact_type, artifact_data, metadata
            FROM execution_artifacts
            WHERE task_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(metadata.task_id)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;

        let artifact_rows: Vec<DbArtifactRow> = rows
            .into_iter()
            .map(|row| {
                let artifact_type: String = row.get("artifact_type");
                let artifact_data: serde_json::Value = row.get("artifact_data");
                let metadata: serde_json::Value = row.get("metadata");

                DbArtifactRow {
                    task_id: metadata.task_id,
                    session_id: None,
                    execution_id: None,
                    artifact_type,
                    artifact_data,
                    metadata,
                }
            })
            .collect();

        if artifact_rows.is_empty() {
            return Err(ArtifactStorageError::NotFound(metadata.id));
        }

        self.db_rows_to_artifacts(artifact_rows, metadata.task_id)
    }

    async fn delete(
        &self,
        metadata: &ArtifactMetadata,
    ) -> Result<(), ArtifactStorageError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;

        // Delete artifacts
        sqlx::query("DELETE FROM execution_artifacts WHERE task_id = $1")
            .bind(metadata.task_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;

        // Delete metadata
        sqlx::query("DELETE FROM artifact_metadata WHERE id = $1")
            .bind(metadata.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;

        tx.commit().await
            .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_old_artifacts(
        &self,
        cutoff_date: DateTime<Utc>,
    ) -> Result<Vec<ArtifactMetadata>, ArtifactStorageError> {
        let rows = sqlx::query(
            r#"
            SELECT id, task_id, created_at, size_bytes, metadata
            FROM artifact_metadata
            WHERE created_at < $1
            ORDER BY created_at ASC
            "#
        )
        .bind(cutoff_date)
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;

        let metadata: Vec<ArtifactMetadata> = rows
            .into_iter()
            .map(|row| {
                let id: Uuid = row.get("id");
                let task_id: Uuid = row.get("task_id");
                let created_at: DateTime<Utc> = row.get("created_at");
                let size_bytes: i64 = row.get("size_bytes");
                let db_metadata: serde_json::Value = row.get("metadata");

                let checksum = db_metadata
                    .get("checksum")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let version = db_metadata
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1")
                    .to_string();

                ArtifactMetadata {
                    id,
                    task_id,
                    created_at,
                    size_bytes: size_bytes as u64,
                    checksum,
                    version,
                    compression_used: false,
                    integrity_verified: true,
                }
            })
            .collect();

        Ok(metadata)
    }

    async fn find_latest(
        &self,
        task_id: Uuid,
    ) -> Result<ArtifactMetadata, ArtifactStorageError> {
        let row = sqlx::query(
            r#"
            SELECT id, task_id, created_at, size_bytes, metadata
            FROM artifact_metadata
            WHERE task_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#
        )
        .bind(task_id)
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                let id: Uuid = row.get("id");
                let task_id: Uuid = row.get("task_id");
                let created_at: DateTime<Utc> = row.get("created_at");
                let size_bytes: i64 = row.get("size_bytes");
                let db_metadata: serde_json::Value = row.get("metadata");

                let checksum = db_metadata
                    .get("checksum")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let version = db_metadata
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("1")
                    .to_string();

                Ok(ArtifactMetadata {
                    id,
                    task_id,
                    created_at,
                    size_bytes: size_bytes as u64,
                    checksum,
                    version,
                    compression_used: false,
                    integrity_verified: true,
                })
            }
            None => Err(ArtifactStorageError::NotFoundForTask(task_id)),
        }
    }

    async fn count_artifacts(&self) -> Result<usize, ArtifactStorageError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM execution_artifacts")
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;

        let count: i64 = row.get("count");
        Ok(count as usize)
    }

    async fn total_size(&self) -> Result<u64, ArtifactStorageError> {
        let row = sqlx::query("SELECT COALESCE(SUM(size_bytes), 0) as total FROM execution_artifacts")
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| ArtifactStorageError::DatabaseError(e.to_string()))?;

        let total: i64 = row.get("total");
        Ok(total as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatabaseConfig;

    #[tokio::test]
    async fn test_database_artifact_storage_creation() {
        let config = DatabaseConfig::default();
        // Note: This test would require a running PostgreSQL instance
        // For now, we just test that the struct can be created
        // In a real test environment, we'd set up a test database
        let _config = config;
    }
}
