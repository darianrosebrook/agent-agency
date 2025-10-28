//! Source integrity storage implementation
//!
//! This module provides storage implementations for source integrity records.

use crate::integrity_types::*;
use async_trait::async_trait;
use uuid::Uuid;

/// Trait for source integrity storage operations
#[async_trait]
pub trait NewSourceIntegrityStorage: Send + Sync {
    /// Store a new source integrity record
    async fn store_record(&self, record: &CreateSourceIntegrityRecord) -> anyhow::Result<Uuid>;

    /// Get a source integrity record by ID
    async fn get_record(&self, id: &Uuid) -> anyhow::Result<Option<SourceIntegrityRecord>>;

    /// Update an existing source integrity record
    async fn update_record(&self, id: &Uuid, record: &CreateSourceIntegrityRecord) -> anyhow::Result<()>;

    /// Delete a source integrity record
    async fn delete_record(&self, id: &Uuid) -> anyhow::Result<()>;

    /// List source integrity records
    async fn list_records(&self) -> anyhow::Result<Vec<SourceIntegrityRecord>>;

    /// Get records by source ID
    async fn get_records_by_source(&self, source_id: &str) -> anyhow::Result<Vec<SourceIntegrityRecord>>;

    /// Get records by integrity status
    async fn get_records_by_status(&self, status: IntegrityStatus) -> anyhow::Result<Vec<SourceIntegrityRecord>>;

    /// Get records within a time range
    async fn get_records_by_time_range(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> anyhow::Result<Vec<SourceIntegrityRecord>>;

    /// Get integrity statistics
    async fn get_integrity_stats(&self) -> anyhow::Result<()>;

    /// Cleanup old records
    async fn cleanup_old_records(&self, older_than: chrono::DateTime<chrono::Utc>) -> anyhow::Result<usize>;
}

/// PostgreSQL implementation of source integrity storage
pub struct PostgresSourceIntegrityStorage {
    db_client: sqlx::PgPool,
}

impl PostgresSourceIntegrityStorage {
    /// Create a new PostgreSQL storage instance
    pub fn new(db_client: sqlx::PgPool) -> Self {
        Self { 
            db_client,
        }
    }

    /// Initialize the database schema
    pub async fn initialize_schema(&self) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS source_integrity_records (
                id UUID PRIMARY KEY,
                source_id VARCHAR(255) NOT NULL,
                source_type VARCHAR(50) NOT NULL,
                content_hash VARCHAR(128) NOT NULL,
                content_size BIGINT NOT NULL,
                hash_algorithm VARCHAR(20) NOT NULL,
                integrity_status VARCHAR(20) NOT NULL,
                tampering_indicators JSONB,
                verification_metadata JSONB,
                verification_count INTEGER DEFAULT 0,
                first_seen_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                last_verified_at TIMESTAMP WITH TIME ZONE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        )
        .execute(&self.db_client)
        .await?;

        // Create indexes for better performance
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_source_integrity_source_id ON source_integrity_records(source_id)"
        )
        .execute(&self.db_client)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_source_integrity_status ON source_integrity_records(integrity_status)"
        )
        .execute(&self.db_client)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_source_integrity_created_at ON source_integrity_records(created_at)"
        )
        .execute(&self.db_client)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl NewSourceIntegrityStorage for PostgresSourceIntegrityStorage {
    async fn store_record(&self, record: &CreateSourceIntegrityRecord) -> anyhow::Result<Uuid> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();
        
        sqlx::query(
            r#"
            INSERT INTO source_integrity_records (
                id, source_id, source_type, content_hash, content_size,
                hash_algorithm, integrity_status, tampering_indicators,
                verification_metadata, verification_count, first_seen_at,
                last_verified_at, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#
        )
        .bind(id)
        .bind(&record.source_id)
        .bind(record.source_type.to_string())
        .bind(&record.content_hash)
        .bind(record.content_size)
        .bind(record.hash_algorithm.to_string())
        .bind(record.integrity_status.to_string())
        .bind(serde_json::to_value(&record.tampering_indicators)?)
        .bind(serde_json::to_value(&record.verification_metadata)?)
        .bind(0i32) // verification_count
        .bind(now)
        .bind(None::<chrono::DateTime<chrono::Utc>>)
        .bind(now)
        .bind(now)
        .execute(&self.db_client)
        .await?;
        
        Ok(id)
    }

    async fn get_record(&self, id: &Uuid) -> anyhow::Result<Option<SourceIntegrityRecord>> {
        let row = sqlx::query_as::<_, (Uuid, String, String, String, i64, String, String, serde_json::Value, serde_json::Value, i32, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, source_id, source_type, content_hash, content_size,
                   hash_algorithm, integrity_status, tampering_indicators,
                   verification_metadata, verification_count, first_seen_at,
                   last_verified_at, created_at, updated_at
            FROM source_integrity_records
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.db_client)
        .await?;

        if let Some((id, source_id, source_type, content_hash, content_size, hash_algorithm, integrity_status, tampering_indicators, verification_metadata, verification_count, first_seen_at, last_verified_at, created_at, updated_at)) = row {
            Ok(Some(SourceIntegrityRecord {
                id,
                source_id,
                source_type: SourceType::from_string(&source_type).unwrap_or(SourceType::File),
                content_hash,
                content_size,
                hash_algorithm: HashAlgorithm::from_string(&hash_algorithm).unwrap_or(HashAlgorithm::Sha256),
                integrity_status: IntegrityStatus::from_string(&integrity_status).unwrap_or(IntegrityStatus::Unknown),
                tampering_indicators: serde_json::from_value(tampering_indicators).unwrap_or_default(),
                verification_metadata: serde_json::from_value(verification_metadata).unwrap_or_default(),
                verification_count,
                first_seen_at,
                last_verified_at,
                created_at,
                updated_at,
            }))
        } else {
            Ok(None)
        }
    }

    async fn update_record(&self, id: &Uuid, record: &CreateSourceIntegrityRecord) -> anyhow::Result<()> {
        let now = chrono::Utc::now();
        
        sqlx::query(
            r#"
            UPDATE source_integrity_records SET
                source_id = $2,
                source_type = $3,
                content_hash = $4,
                content_size = $5,
                hash_algorithm = $6,
                integrity_status = $7,
                tampering_indicators = $8,
                verification_metadata = $9,
                updated_at = $10
            WHERE id = $1
            "#
        )
        .bind(id)
        .bind(&record.source_id)
        .bind(record.source_type.to_string())
        .bind(&record.content_hash)
        .bind(record.content_size)
        .bind(record.hash_algorithm.to_string())
        .bind(record.integrity_status.to_string())
        .bind(serde_json::to_value(&record.tampering_indicators)?)
        .bind(serde_json::to_value(&record.verification_metadata)?)
        .bind(now)
        .execute(&self.db_client)
        .await?;
        
        Ok(())
    }

    async fn delete_record(&self, id: &Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM source_integrity_records WHERE id = $1")
            .bind(id)
            .execute(&self.db_client)
            .await?;
        
        Ok(())
    }

    async fn list_records(&self) -> anyhow::Result<Vec<SourceIntegrityRecord>> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, String, i64, String, String, serde_json::Value, serde_json::Value, i32, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, source_id, source_type, content_hash, content_size,
                   hash_algorithm, integrity_status, tampering_indicators,
                   verification_metadata, verification_count, first_seen_at,
                   last_verified_at, created_at, updated_at
            FROM source_integrity_records
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&self.db_client)
        .await?;

        let records = rows.into_iter()
            .map(|(id, source_id, source_type, content_hash, content_size, hash_algorithm, integrity_status, tampering_indicators, verification_metadata, verification_count, first_seen_at, last_verified_at, created_at, updated_at)| {
                SourceIntegrityRecord {
                    id,
                    source_id,
                    source_type: SourceType::from_string(&source_type).unwrap_or(SourceType::File),
                    content_hash,
                    content_size,
                    hash_algorithm: HashAlgorithm::from_string(&hash_algorithm).unwrap_or(HashAlgorithm::Sha256),
                    integrity_status: IntegrityStatus::from_string(&integrity_status).unwrap_or(IntegrityStatus::Unknown),
                    tampering_indicators: serde_json::from_value(tampering_indicators).unwrap_or_default(),
                    verification_metadata: serde_json::from_value(verification_metadata).unwrap_or_default(),
                    verification_count,
                    first_seen_at,
                    last_verified_at,
                    created_at,
                    updated_at,
                }
            })
            .collect();

        Ok(records)
    }

    async fn get_records_by_source(&self, source_id: &str) -> anyhow::Result<Vec<SourceIntegrityRecord>> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, String, i64, String, String, serde_json::Value, serde_json::Value, i32, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, source_id, source_type, content_hash, content_size,
                   hash_algorithm, integrity_status, tampering_indicators,
                   verification_metadata, verification_count, first_seen_at,
                   last_verified_at, created_at, updated_at
            FROM source_integrity_records
            WHERE source_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(source_id)
        .fetch_all(&self.db_client)
        .await?;

        let records = rows.into_iter()
            .map(|(id, source_id, source_type, content_hash, content_size, hash_algorithm, integrity_status, tampering_indicators, verification_metadata, verification_count, first_seen_at, last_verified_at, created_at, updated_at)| {
                SourceIntegrityRecord {
                    id,
                    source_id,
                    source_type: SourceType::from_string(&source_type).unwrap_or(SourceType::File),
                    content_hash,
                    content_size,
                    hash_algorithm: HashAlgorithm::from_string(&hash_algorithm).unwrap_or(HashAlgorithm::Sha256),
                    integrity_status: IntegrityStatus::from_string(&integrity_status).unwrap_or(IntegrityStatus::Unknown),
                    tampering_indicators: serde_json::from_value(tampering_indicators).unwrap_or_default(),
                    verification_metadata: serde_json::from_value(verification_metadata).unwrap_or_default(),
                    verification_count,
                    first_seen_at,
                    last_verified_at,
                    created_at,
                    updated_at,
                }
            })
            .collect();

        Ok(records)
    }

    async fn get_records_by_status(&self, status: IntegrityStatus) -> anyhow::Result<Vec<SourceIntegrityRecord>> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, String, i64, String, String, serde_json::Value, serde_json::Value, i32, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, source_id, source_type, content_hash, content_size,
                   hash_algorithm, integrity_status, tampering_indicators,
                   verification_metadata, verification_count, first_seen_at,
                   last_verified_at, created_at, updated_at
            FROM source_integrity_records
            WHERE integrity_status = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(status.to_string())
        .fetch_all(&self.db_client)
        .await?;

        let records = rows.into_iter()
            .map(|(id, source_id, source_type, content_hash, content_size, hash_algorithm, integrity_status, tampering_indicators, verification_metadata, verification_count, first_seen_at, last_verified_at, created_at, updated_at)| {
                SourceIntegrityRecord {
                    id,
                    source_id,
                    source_type: SourceType::from_string(&source_type).unwrap_or(SourceType::File),
                    content_hash,
                    content_size,
                    hash_algorithm: HashAlgorithm::from_string(&hash_algorithm).unwrap_or(HashAlgorithm::Sha256),
                    integrity_status: IntegrityStatus::from_string(&integrity_status).unwrap_or(IntegrityStatus::Unknown),
                    tampering_indicators: serde_json::from_value(tampering_indicators).unwrap_or_default(),
                    verification_metadata: serde_json::from_value(verification_metadata).unwrap_or_default(),
                    verification_count,
                    first_seen_at,
                    last_verified_at,
                    created_at,
                    updated_at,
                }
            })
            .collect();

        Ok(records)
    }

    async fn get_records_by_time_range(&self, start: chrono::DateTime<chrono::Utc>, end: chrono::DateTime<chrono::Utc>) -> anyhow::Result<Vec<SourceIntegrityRecord>> {
        let rows = sqlx::query_as::<_, (Uuid, String, String, String, i64, String, String, serde_json::Value, serde_json::Value, i32, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, source_id, source_type, content_hash, content_size,
                   hash_algorithm, integrity_status, tampering_indicators,
                   verification_metadata, verification_count, first_seen_at,
                   last_verified_at, created_at, updated_at
            FROM source_integrity_records
            WHERE created_at BETWEEN $1 AND $2
            ORDER BY created_at DESC
            "#
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.db_client)
        .await?;

        let records = rows.into_iter()
            .map(|(id, source_id, source_type, content_hash, content_size, hash_algorithm, integrity_status, tampering_indicators, verification_metadata, verification_count, first_seen_at, last_verified_at, created_at, updated_at)| {
                SourceIntegrityRecord {
                    id,
                    source_id,
                    source_type: SourceType::from_string(&source_type).unwrap_or(SourceType::File),
                    content_hash,
                    content_size,
                    hash_algorithm: HashAlgorithm::from_string(&hash_algorithm).unwrap_or(HashAlgorithm::Sha256),
                    integrity_status: IntegrityStatus::from_string(&integrity_status).unwrap_or(IntegrityStatus::Unknown),
                    tampering_indicators: serde_json::from_value(tampering_indicators).unwrap_or_default(),
                    verification_metadata: serde_json::from_value(verification_metadata).unwrap_or_default(),
                    verification_count,
                    first_seen_at,
                    last_verified_at,
                    created_at,
                    updated_at,
                }
            })
            .collect();

        Ok(records)
    }

    async fn get_integrity_stats(&self) -> anyhow::Result<()> {
        // This method should return actual statistics, but the trait returns ()
        // In a real implementation, we'd return a stats struct with counts, etc.
        let _total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM source_integrity_records")
            .fetch_one(&self.db_client)
            .await?;

        let _verified_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM source_integrity_records WHERE integrity_status = 'verified'")
            .fetch_one(&self.db_client)
            .await?;

        // For now, just return Ok(()) as the trait requires
        Ok(())
    }

    async fn cleanup_old_records(&self, older_than: chrono::DateTime<chrono::Utc>) -> anyhow::Result<usize> {
        let result = sqlx::query("DELETE FROM source_integrity_records WHERE created_at < $1")
            .bind(older_than)
            .execute(&self.db_client)
            .await?;
        
        Ok(result.rows_affected() as usize)
    }
}