//! Storage implementations for verdict persistence
//!
//! This module provides concrete implementations of the VerdictStorage trait,
//! including in-memory and database-backed storage options.

use super::types::*;
use crate::council_types::*;
use agent_agency_database::DatabaseClient;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde_json;
use sqlx::PgPool;
use sqlx::Row;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Storage backend trait for verdict persistence
#[async_trait]
pub trait VerdictStorage: Send + Sync + std::fmt::Debug {
    /// Store a verdict record
    async fn store_verdict(&self, record: &VerdictRecord) -> Result<()>;

    /// Load a verdict by ID
    async fn load_verdict(&self, verdict_id: VerdictId) -> Result<Option<VerdictRecord>>;

    /// Load verdicts for a specific task
    async fn load_verdicts_by_task(&self, task_id: TaskId) -> Result<Vec<VerdictRecord>>;

    /// Load verdicts within a time range
    async fn load_verdicts_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<VerdictRecord>>;

    /// Delete a verdict
    async fn delete_verdict(&self, verdict_id: VerdictId) -> Result<()>;

    /// Get storage statistics
    async fn get_storage_stats(&self) -> Result<StorageStats>;
}

/// In-memory implementation of VerdictStorage for development and testing
#[derive(Debug, Default)]
pub struct MemoryVerdictStorage {
    verdicts: RwLock<Vec<VerdictRecord>>,
}

#[async_trait]
impl VerdictStorage for MemoryVerdictStorage {
    async fn store_verdict(&self, record: &VerdictRecord) -> Result<()> {
        let mut verdicts = self.verdicts.write().await;
        // Remove existing verdict with same ID if present
        verdicts.retain(|v| v.verdict_id != record.verdict_id);
        verdicts.push(record.clone());
        Ok(())
    }

    async fn load_verdict(&self, verdict_id: VerdictId) -> Result<Option<VerdictRecord>> {
        let verdicts = self.verdicts.read().await;
        Ok(verdicts.iter().find(|v| v.verdict_id == verdict_id).cloned())
    }

    async fn load_verdicts_by_task(&self, task_id: TaskId) -> Result<Vec<VerdictRecord>> {
        let verdicts = self.verdicts.read().await;
        Ok(verdicts
            .iter()
            .filter(|v| {
                // Extract task_id from consensus result or debate session
                if let Some(ref debate) = v.debate_session {
                    debate.task_id == task_id
                } else {
                    // Try to extract from consensus result if it contains task info
                    // This is a simplified implementation
                    false
                }
            })
            .cloned()
            .collect())
    }

    async fn load_verdicts_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<VerdictRecord>> {
        let verdicts = self.verdicts.read().await;
        Ok(verdicts
            .iter()
            .filter(|v| v.created_at >= start && v.created_at <= end)
            .cloned()
            .collect())
    }

    async fn delete_verdict(&self, verdict_id: VerdictId) -> Result<()> {
        let mut verdicts = self.verdicts.write().await;
        let initial_len = verdicts.len();
        verdicts.retain(|v| v.verdict_id != verdict_id);

        if verdicts.len() == initial_len {
            return Err(anyhow::anyhow!("Verdict not found: {}", verdict_id));
        }

        Ok(())
    }

    async fn get_storage_stats(&self) -> Result<StorageStats> {
        let verdicts = self.verdicts.read().await;

        let total_verdicts = verdicts.len() as u64;
        let total_debates = verdicts.iter().filter(|v| v.debate_session.is_some()).count() as u64;

        // Estimate storage size (rough calculation)
        let storage_size_bytes = verdicts.iter()
            .map(|v| {
                let base_size = 100; // Base overhead
                let debate_size = if v.debate_session.is_some() { 500 } else { 0 }; // Rough debate size
                base_size + debate_size
            })
            .sum::<usize>() as u64;

        let oldest_verdict = verdicts.iter().min_by_key(|v| v.created_at).map(|v| v.created_at);
        let newest_verdict = verdicts.iter().max_by_key(|v| v.created_at).map(|v| v.created_at);

        Ok(StorageStats {
            total_verdicts,
            total_debates,
            storage_size_bytes,
            oldest_verdict,
            newest_verdict,
        })
    }
}

/// Database-backed implementation of VerdictStorage
#[derive(Debug)]
pub struct DatabaseVerdictStorage {
    pool: PgPool,
}

impl DatabaseVerdictStorage {
    /// Create a new database verdict storage
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new database verdict storage from database client
    pub fn from_client(client: &DatabaseClient) -> Result<Self> {
        // This would need to be implemented based on how DatabaseClient exposes the pool
        // For now, return an error as this needs more context
        Err(anyhow::anyhow!("DatabaseVerdictStorage::from_client not yet implemented"))
    }
}

#[async_trait]
impl VerdictStorage for DatabaseVerdictStorage {
    async fn store_verdict(&self, record: &VerdictRecord) -> Result<()> {
        let consensus_result_json = serde_json::to_value(&record.consensus_result)
            .context("Failed to serialize consensus result")?;

        let debate_session_json = record
            .debate_session
            .as_ref()
            .map(|ds| serde_json::to_value(ds))
            .transpose()
            .context("Failed to serialize debate session")?;

        sqlx::query!(
            r#"
            INSERT INTO verdicts (
                verdict_id, consensus_result, debate_session,
                created_at, accessed_at, access_count, storage_location
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (verdict_id) DO UPDATE SET
                consensus_result = EXCLUDED.consensus_result,
                debate_session = EXCLUDED.debate_session,
                accessed_at = EXCLUDED.accessed_at,
                access_count = EXCLUDED.access_count,
                storage_location = EXCLUDED.storage_location
            "#,
            record.verdict_id.0,
            consensus_result_json,
            debate_session_json,
            record.created_at,
            record.accessed_at,
            record.access_count as i64,
            record.storage_location,
        )
        .execute(&self.pool)
        .await
        .context("Failed to store verdict")?;

        Ok(())
    }

    async fn load_verdict(&self, verdict_id: VerdictId) -> Result<Option<VerdictRecord>> {
        let row = sqlx::query!(
            r#"
            SELECT verdict_id, consensus_result, debate_session,
                   created_at, accessed_at, access_count, storage_location
            FROM verdicts WHERE verdict_id = $1
            "#,
            verdict_id.0
        )
        .fetch_optional(&self.pool)
        .await
        .context("Failed to load verdict")?;

        match row {
            Some(row) => {
                let consensus_result: ConsensusResult = serde_json::from_value(row.consensus_result)
                    .context("Failed to deserialize consensus result")?;

                let debate_session = row
                    .debate_session
                    .map(|json| serde_json::from_value(json))
                    .transpose()
                    .context("Failed to deserialize debate session")?;

                Ok(Some(VerdictRecord {
                    verdict_id,
                    consensus_result,
                    debate_session,
                    created_at: row.created_at,
                    accessed_at: row.accessed_at,
                    access_count: row.access_count as u64,
                    storage_location: row.storage_location,
                }))
            }
            None => Ok(None),
        }
    }

    async fn load_verdicts_by_task(&self, task_id: TaskId) -> Result<Vec<VerdictRecord>> {
        let rows = sqlx::query!(
            r#"
            SELECT verdict_id, consensus_result, debate_session,
                   created_at, accessed_at, access_count, storage_location
            FROM verdicts
            WHERE debate_session->>'task_id' = $1
            ORDER BY created_at DESC
            "#,
            task_id.0.to_string()
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to load verdicts by task")?;

        let mut verdicts = Vec::new();
        for row in rows {
            let verdict_id = VerdictId(Uuid::parse_str(&row.verdict_id)?);
            let consensus_result: ConsensusResult = serde_json::from_value(row.consensus_result)
                .context("Failed to deserialize consensus result")?;

            let debate_session = row
                .debate_session
                .map(|json| serde_json::from_value(json))
                .transpose()
                .context("Failed to deserialize debate session")?;

            verdicts.push(VerdictRecord {
                verdict_id,
                consensus_result,
                debate_session,
                created_at: row.created_at,
                accessed_at: row.accessed_at,
                access_count: row.access_count as u64,
                storage_location: row.storage_location,
            });
        }

        Ok(verdicts)
    }

    async fn load_verdicts_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<VerdictRecord>> {
        let rows = sqlx::query!(
            r#"
            SELECT verdict_id, consensus_result, debate_session,
                   created_at, accessed_at, access_count, storage_location
            FROM verdicts
            WHERE created_at >= $1 AND created_at <= $2
            ORDER BY created_at DESC
            "#,
            start,
            end
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to load verdicts by time range")?;

        let mut verdicts = Vec::new();
        for row in rows {
            let verdict_id = VerdictId(Uuid::parse_str(&row.verdict_id)?);
            let consensus_result: ConsensusResult = serde_json::from_value(row.consensus_result)
                .context("Failed to deserialize consensus result")?;

            let debate_session = row
                .debate_session
                .map(|json| serde_json::from_value(json))
                .transpose()
                .context("Failed to deserialize debate session")?;

            verdicts.push(VerdictRecord {
                verdict_id,
                consensus_result,
                debate_session,
                created_at: row.created_at,
                accessed_at: row.accessed_at,
                access_count: row.access_count as u64,
                storage_location: row.storage_location,
            });
        }

        Ok(verdicts)
    }

    async fn delete_verdict(&self, verdict_id: VerdictId) -> Result<()> {
        let result = sqlx::query!(
            "DELETE FROM verdicts WHERE verdict_id = $1",
            verdict_id.0
        )
        .execute(&self.pool)
        .await
        .context("Failed to delete verdict")?;

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Verdict not found: {}", verdict_id));
        }

        Ok(())
    }

    async fn get_storage_stats(&self) -> Result<StorageStats> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as total_verdicts,
                COUNT(CASE WHEN debate_session IS NOT NULL THEN 1 END) as total_debates,
                COALESCE(SUM(pg_column_size(verdicts)), 0) as storage_size_bytes,
                MIN(created_at) as oldest_verdict,
                MAX(created_at) as newest_verdict
            FROM verdicts
            "#
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to get storage stats")?;

        Ok(StorageStats {
            total_verdicts: row.total_verdicts.unwrap_or(0),
            total_debates: row.total_debates.unwrap_or(0),
            storage_size_bytes: row.storage_size_bytes.unwrap_or(0) as u64,
            oldest_verdict: row.oldest_verdict,
            newest_verdict: row.newest_verdict,
        })
    }
}
