//! WAL (Write-Ahead Log) Storage and Replay System
//!
//! Provides comprehensive WAL log storage, replay, and point-in-time recovery capabilities.
//! Stores logical database changes as records that can be replayed to restore database state.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

/// WAL operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalOperationType {
    Insert,
    Update,
    Delete,
    Ddl,
    Truncate,
}

impl std::fmt::Display for WalOperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalOperationType::Insert => write!(f, "INSERT"),
            WalOperationType::Update => write!(f, "UPDATE"),
            WalOperationType::Delete => write!(f, "DELETE"),
            WalOperationType::Ddl => write!(f, "DDL"),
            WalOperationType::Truncate => write!(f, "TRUNCATE"),
        }
    }
}

impl std::str::FromStr for WalOperationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INSERT" => Ok(WalOperationType::Insert),
            "UPDATE" => Ok(WalOperationType::Update),
            "DELETE" => Ok(WalOperationType::Delete),
            "DDL" => Ok(WalOperationType::Ddl),
            "TRUNCATE" => Ok(WalOperationType::Truncate),
            _ => Err(format!("Invalid WAL operation type: {}", s)),
        }
    }
}

/// WAL log record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalRecord {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub transaction_id: Uuid,
    pub sequence_number: i64,
    pub operation_type: WalOperationType,
    pub schema_name: String,
    pub table_name: String,
    pub record_id: Option<Uuid>,
    pub old_data: Option<JsonValue>,
    pub new_data: Option<JsonValue>,
    pub sql_statement: Option<String>,
    pub checksum: Option<String>,
    pub applied: bool,
    pub replayed_at: Option<DateTime<Utc>>,
}

/// WAL storage manager
pub struct WalStorage {
    pool: Arc<PgPool>,
}

impl WalStorage {
    /// Create new WAL storage manager
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .context("Failed to connect to database for WAL storage")?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Record a WAL entry
    pub async fn record_change(
        &self,
        transaction_id: Uuid,
        operation_type: WalOperationType,
        schema_name: &str,
        table_name: &str,
        record_id: Option<Uuid>,
        old_data: Option<JsonValue>,
        new_data: Option<JsonValue>,
        sql_statement: Option<String>,
    ) -> Result<Uuid> {
        let checksum = self.calculate_checksum(&operation_type, &old_data, &new_data, &sql_statement);
        
        let record_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO wal_log_records (
                transaction_id, operation_type, schema_name, table_name,
                record_id, old_data, new_data, sql_statement, checksum
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
        .bind(transaction_id)
        .bind(operation_type.to_string())
        .bind(schema_name)
        .bind(table_name)
        .bind(record_id)
        .bind(old_data)
        .bind(new_data)
        .bind(sql_statement)
        .bind(checksum)
        .fetch_one(&*self.pool)
        .await
        .context("Failed to record WAL entry")?;

        debug!("Recorded WAL entry: {} for table {}.{}", record_id, schema_name, table_name);
        Ok(record_id)
    }

    /// Get WAL records for replay within a time range
    pub async fn get_records_for_replay(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        table_filter: Option<&str>,
    ) -> Result<Vec<WalRecord>> {
        let records = sqlx::query_as::<_, WalRecordRow>(
            r#"
            SELECT * FROM get_wal_records_for_replay($1, $2, $3)
            ORDER BY sequence_number ASC, timestamp ASC
            "#,
        )
        .bind(start_time)
        .bind(end_time)
        .bind(table_filter)
        .fetch_all(&*self.pool)
        .await
        .context("Failed to fetch WAL records for replay")?;

        let wal_records: Vec<WalRecord> = records.into_iter().map(|r| r.into()).collect();
        
        info!("Fetched {} WAL records for replay from {} to {}", 
              wal_records.len(), start_time, end_time);
        
        Ok(wal_records)
    }

    /// Mark WAL records as applied
    pub async fn mark_records_applied(&self, record_ids: &[Uuid]) -> Result<()> {
        if record_ids.is_empty() {
            return Ok(());
        }

        sqlx::query(
            r#"
            UPDATE wal_log_records
            SET applied = TRUE, replayed_at = NOW()
            WHERE id = ANY($1)
            "#,
        )
        .bind(record_ids)
        .execute(&*self.pool)
        .await
        .context("Failed to mark WAL records as applied")?;

        debug!("Marked {} WAL records as applied", record_ids.len());
        Ok(())
    }

    /// Cleanup old WAL records based on retention policy
    pub async fn cleanup_old_records(&self, retention_days: i32) -> Result<i64> {
        let deleted_count: i64 = sqlx::query_scalar(
            "SELECT cleanup_old_wal_records($1)"
        )
        .bind(retention_days)
        .fetch_one(&*self.pool)
        .await
        .context("Failed to cleanup old WAL records")?;

        info!("Cleaned up {} old WAL records (retention: {} days)", deleted_count, retention_days);
        Ok(deleted_count)
    }

    /// Get WAL statistics
    pub async fn get_statistics(&self) -> Result<WalStatistics> {
        let stats = sqlx::query_as::<_, WalStatisticsRow>(
            r#"
            SELECT 
                COUNT(*) as total_records,
                COUNT(*) FILTER (WHERE applied = FALSE) as pending_records,
                COUNT(*) FILTER (WHERE applied = TRUE) as applied_records,
                MIN(timestamp) as oldest_record,
                MAX(timestamp) as newest_record,
                COUNT(DISTINCT transaction_id) as unique_transactions,
                COUNT(DISTINCT table_name) as affected_tables
            FROM wal_log_records
            "#,
        )
        .fetch_one(&*self.pool)
        .await
        .context("Failed to get WAL statistics")?;

        Ok(stats.into())
    }

    /// Calculate checksum for WAL record
    pub fn calculate_checksum(
        &self,
        operation_type: &WalOperationType,
        old_data: &Option<JsonValue>,
        new_data: &Option<JsonValue>,
        sql_statement: &Option<String>,
    ) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(operation_type.to_string().as_bytes());
        
        if let Some(ref old) = old_data {
            hasher.update(serde_json::to_string(old).unwrap_or_default().as_bytes());
        }
        
        if let Some(ref new) = new_data {
            hasher.update(serde_json::to_string(new).unwrap_or_default().as_bytes());
        }
        
        if let Some(ref sql) = sql_statement {
            hasher.update(sql.as_bytes());
        }
        
        let result = hasher.finalize();
        format!("{:x}", result)
    }

    /// Get database pool for direct access
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// WAL statistics
#[derive(Debug, Clone)]
pub struct WalStatistics {
    pub total_records: i64,
    pub pending_records: i64,
    pub applied_records: i64,
    pub oldest_record: Option<DateTime<Utc>>,
    pub newest_record: Option<DateTime<Utc>>,
    pub unique_transactions: i64,
    pub affected_tables: i64,
}

/// Internal row representation for WAL records
#[derive(Debug)]
struct WalRecordRow {
    id: Uuid,
    timestamp: DateTime<Utc>,
    transaction_id: Uuid,
    sequence_number: i64,
    operation_type: String,
    schema_name: String,
    table_name: String,
    record_id: Option<Uuid>,
    old_data: Option<JsonValue>,
    new_data: Option<JsonValue>,
    sql_statement: Option<String>,
}

impl From<WalRecordRow> for WalRecord {
    fn from(row: WalRecordRow) -> Self {
        WalRecord {
            id: row.id,
            timestamp: row.timestamp,
            transaction_id: row.transaction_id,
            sequence_number: row.sequence_number,
            operation_type: row.operation_type.parse().unwrap_or(WalOperationType::Update),
            schema_name: row.schema_name,
            table_name: row.table_name,
            record_id: row.record_id,
            old_data: row.old_data,
            new_data: row.new_data,
            sql_statement: row.sql_statement,
            checksum: None,
            applied: false,
            replayed_at: None,
        }
    }
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for WalRecordRow {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(WalRecordRow {
            id: row.try_get("id")?,
            timestamp: row.try_get("timestamp")?,
            transaction_id: row.try_get("transaction_id")?,
            sequence_number: row.try_get("sequence_number")?,
            operation_type: row.try_get("operation_type")?,
            schema_name: row.try_get("schema_name")?,
            table_name: row.try_get("table_name")?,
            record_id: row.try_get("record_id")?,
            old_data: row.try_get("old_data")?,
            new_data: row.try_get("new_data")?,
            sql_statement: row.try_get("sql_statement")?,
        })
    }
}

#[derive(Debug)]
struct WalStatisticsRow {
    total_records: i64,
    pending_records: i64,
    applied_records: i64,
    oldest_record: Option<DateTime<Utc>>,
    newest_record: Option<DateTime<Utc>>,
    unique_transactions: i64,
    affected_tables: i64,
}

impl From<WalStatisticsRow> for WalStatistics {
    fn from(row: WalStatisticsRow) -> Self {
        WalStatistics {
            total_records: row.total_records,
            pending_records: row.pending_records,
            applied_records: row.applied_records,
            oldest_record: row.oldest_record,
            newest_record: row.newest_record,
            unique_transactions: row.unique_transactions,
            affected_tables: row.affected_tables,
        }
    }
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for WalStatisticsRow {
    fn from_row(row: &sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        Ok(WalStatisticsRow {
            total_records: row.get("total_records"),
            pending_records: row.get("pending_records"),
            applied_records: row.get("applied_records"),
            oldest_record: row.get("oldest_record"),
            newest_record: row.get("newest_record"),
            unique_transactions: row.get("unique_transactions"),
            affected_tables: row.get("affected_tables"),
        })
    }
}

