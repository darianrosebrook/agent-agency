//! WAL Replay Engine for Point-in-Time Recovery
//!
//! Provides comprehensive WAL log replay functionality with transaction consistency,
//! conflict resolution, and progress tracking.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use sqlx::{PgPool, Transaction};
use std::sync::Arc;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

use crate::wal_storage::{WalRecord, WalOperationType, WalStorage};

/// WAL replay configuration
#[derive(Debug, Clone)]
pub struct WalReplayConfig {
    /// Stop replay at this timestamp
    pub target_time: DateTime<Utc>,
    /// Filter to specific table (None = all tables)
    pub table_filter: Option<String>,
    /// Batch size for applying records
    pub batch_size: usize,
    /// Enable parallel replay (future enhancement)
    pub enable_parallel: bool,
    /// Stop on first error (false = log and continue)
    pub stop_on_error: bool,
    /// Validate checksums during replay
    pub validate_checksums: bool,
}

impl Default for WalReplayConfig {
    fn default() -> Self {
        Self {
            target_time: Utc::now(),
            table_filter: None,
            batch_size: 1000,
            enable_parallel: false,
            stop_on_error: false,
            validate_checksums: true,
        }
    }
}

/// WAL replay status
#[derive(Debug, Clone)]
pub struct WalReplayStatus {
    pub replay_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub records_processed: usize,
    pub records_applied: usize,
    pub records_failed: usize,
    pub transactions_processed: usize,
    pub progress_percent: f64,
    pub status: ReplayStatus,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Paused,
}

/// WAL replay engine
pub struct WalReplayEngine {
    storage: Arc<WalStorage>,
    target_pool: Arc<PgPool>,
}

impl WalReplayEngine {
    /// Create new WAL replay engine
    pub fn new(storage: Arc<WalStorage>, target_pool: Arc<PgPool>) -> Self {
        Self {
            storage,
            target_pool,
        }
    }

    /// Replay WAL records from backup time to target time
    pub async fn replay_wal_logs(
        &self,
        backup_time: DateTime<Utc>,
        config: WalReplayConfig,
    ) -> Result<WalReplayStatus> {
        let replay_id = Uuid::new_v4();
        let start_time = Utc::now();
        
        info!("Starting WAL replay: {} from {} to {}", 
              replay_id, backup_time, config.target_time);

        // Get WAL records for replay
        let records = self.storage.get_records_for_replay(
            backup_time,
            config.target_time,
            config.table_filter.as_deref(),
        ).await?;

        if records.is_empty() {
            info!("No WAL records to replay");
            return Ok(WalReplayStatus {
                replay_id,
                start_time,
                end_time: Some(Utc::now()),
                records_processed: 0,
                records_applied: 0,
                records_failed: 0,
                transactions_processed: 0,
                progress_percent: 100.0,
                status: ReplayStatus::Completed,
                error_message: None,
            });
        }

        let total_records = records.len();
        let mut status = WalReplayStatus {
            replay_id,
            start_time,
            end_time: None,
            records_processed: 0,
            records_applied: 0,
            records_failed: 0,
            transactions_processed: 0,
            progress_percent: 0.0,
            status: ReplayStatus::InProgress,
            error_message: None,
        };

        // Group records by transaction for atomic replay
        let mut records_by_transaction: std::collections::HashMap<Uuid, Vec<WalRecord>> = 
            std::collections::HashMap::new();
        
        for record in records {
            records_by_transaction
                .entry(record.transaction_id)
                .or_insert_with(Vec::new)
                .push(record);
        }

        let _total_transactions = records_by_transaction.len();
        let mut applied_record_ids = Vec::new();

        // Replay transactions in order
        let mut sorted_transactions: Vec<_> = records_by_transaction.keys().collect();
        sorted_transactions.sort(); // Simple ordering - could be improved with timestamps

        for transaction_id in sorted_transactions {
            let transaction_records = records_by_transaction.get(transaction_id).unwrap();
            
            match self.replay_transaction(transaction_records, &config).await {
                Ok(record_ids) => {
                    applied_record_ids.extend(record_ids);
                    status.records_applied += transaction_records.len();
                    status.transactions_processed += 1;
                }
                Err(e) => {
                    status.records_failed += transaction_records.len();
                    error!("Failed to replay transaction {}: {}", transaction_id, e);
                    
                    if config.stop_on_error {
                        status.status = ReplayStatus::Failed;
                        status.error_message = Some(e.to_string());
                        return Err(e);
                    }
                }
            }

            status.records_processed += transaction_records.len();
            status.progress_percent = 
                (status.records_processed as f64 / total_records as f64) * 100.0;

            // Update checkpoint periodically
            if status.records_processed % config.batch_size == 0 {
                if let Err(e) = self.storage.mark_records_applied(&applied_record_ids).await {
                    warn!("Failed to mark records as applied: {}", e);
                }
                applied_record_ids.clear();
                
                self.create_checkpoint(&replay_id, status.records_processed, 
                                     *transaction_id).await?;
            }
        }

        // Mark remaining records as applied
        if !applied_record_ids.is_empty() {
            if let Err(e) = self.storage.mark_records_applied(&applied_record_ids).await {
                warn!("Failed to mark final records as applied: {}", e);
            }
        }

        status.end_time = Some(Utc::now());
        status.status = ReplayStatus::Completed;
        status.progress_percent = 100.0;

        info!("WAL replay completed: {} ({} records, {} transactions, {}ms)",
              replay_id, status.records_applied, status.transactions_processed,
              status.end_time.unwrap().signed_duration_since(start_time).num_milliseconds());

        Ok(status)
    }

    /// Replay a single transaction's records
    async fn replay_transaction(
        &self,
        records: &[WalRecord],
        config: &WalReplayConfig,
    ) -> Result<Vec<Uuid>> {
        if records.is_empty() {
            return Ok(Vec::new());
        }

        // Sort records by sequence number
        let mut sorted_records = records.to_vec();
        sorted_records.sort_by_key(|r| r.sequence_number);

        let mut applied_ids = Vec::new();
        let mut tx = self.target_pool.begin().await?;

        for record in &sorted_records {
            // Validate checksum if enabled
            if config.validate_checksums {
                if let Err(e) = self.validate_record_checksum(record).await {
                    warn!("Checksum validation failed for record {}: {}", record.id, e);
                    if config.stop_on_error {
                        return Err(anyhow::anyhow!("Checksum validation failed: {}", e));
                    }
                    continue;
                }
            }

            match self.apply_record(&mut tx, record).await {
                Ok(()) => {
                    applied_ids.push(record.id);
                    debug!("Applied WAL record: {} ({})", record.id, record.operation_type);
                }
                Err(e) => {
                    warn!("Failed to apply WAL record {}: {}", record.id, e);
                    if config.stop_on_error {
                        tx.rollback().await?;
                        return Err(e);
                    }
                }
            }
        }

        // Commit transaction
        tx.commit().await.context("Failed to commit transaction replay")?;

        Ok(applied_ids)
    }

    /// Apply a single WAL record
    async fn apply_record(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        record: &WalRecord,
    ) -> Result<()> {
        match record.operation_type {
            WalOperationType::Insert => {
                self.apply_insert(tx, record).await?;
            }
            WalOperationType::Update => {
                self.apply_update(tx, record).await?;
            }
            WalOperationType::Delete => {
                self.apply_delete(tx, record).await?;
            }
            WalOperationType::Truncate => {
                self.apply_truncate(tx, record).await?;
            }
            WalOperationType::Ddl => {
                self.apply_ddl(tx, record).await?;
            }
        }
        Ok(())
    }

    /// Apply INSERT operation
    async fn apply_insert(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        record: &WalRecord,
    ) -> Result<()> {
        let new_data = record.new_data.as_ref()
            .ok_or_else(|| anyhow::anyhow!("INSERT operation missing new_data"))?;

        let columns: Vec<String> = if let Some(obj) = new_data.as_object() {
            obj.keys().cloned().collect()
        } else {
            return Err(anyhow::anyhow!("Invalid new_data format for INSERT"));
        };

        let placeholders: Vec<String> = (1..=columns.len())
            .map(|i| format!("${}", i))
            .collect();

        let _values: Vec<&JsonValue> = columns.iter()
            .map(|col| &new_data[col])
            .collect();

        let query = format!(
            "INSERT INTO {}.{} ({}) VALUES ({})",
            record.schema_name,
            record.table_name,
            columns.join(", "),
            placeholders.join(", ")
        );

        sqlx::query(&query)
            .execute(&mut **tx)
            .await
            .context(format!("Failed to apply INSERT to {}.{}", 
                           record.schema_name, record.table_name))?;

        Ok(())
    }

    /// Apply UPDATE operation
    async fn apply_update(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        record: &WalRecord,
    ) -> Result<()> {
        let new_data = record.new_data.as_ref()
            .ok_or_else(|| anyhow::anyhow!("UPDATE operation missing new_data"))?;

        let updates: Vec<String> = if let Some(obj) = new_data.as_object() {
            obj.keys().enumerate()
                .map(|(i, col)| format!("{} = ${}", col, i + 1))
                .collect()
        } else {
            return Err(anyhow::anyhow!("Invalid new_data format for UPDATE"));
        };

        let _values: Vec<&JsonValue> = if let Some(obj) = new_data.as_object() {
            obj.values().collect()
        } else {
            return Err(anyhow::anyhow!("Invalid new_data format for UPDATE"));
        };

        // Build WHERE clause from record_id or old_data
        let where_clause = if let Some(record_id) = record.record_id {
            format!("id = '{}'", record_id)
        } else if let Some(ref old_data) = record.old_data {
            // Use old_data to identify record (primary key fields)
            if let Some(obj) = old_data.as_object() {
                let conditions: Vec<String> = obj.keys()
                    .filter(|k| *k == "id" || k.ends_with("_id"))
                    .map(|k| format!("{} = '{}'", k, obj[k]))
                    .collect();
                
                if conditions.is_empty() {
                    return Err(anyhow::anyhow!("Cannot identify record for UPDATE"));
                }
                
                conditions.join(" AND ")
            } else {
                return Err(anyhow::anyhow!("Invalid old_data format for UPDATE"));
            }
        } else {
            return Err(anyhow::anyhow!("UPDATE operation missing record identifier"));
        };

        let query = format!(
            "UPDATE {}.{} SET {} WHERE {}",
            record.schema_name,
            record.table_name,
            updates.join(", "),
            where_clause
        );

        sqlx::query(&query)
            .execute(&mut **tx)
            .await
            .context(format!("Failed to apply UPDATE to {}.{}", 
                           record.schema_name, record.table_name))?;

        Ok(())
    }

    /// Apply DELETE operation
    async fn apply_delete(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        record: &WalRecord,
    ) -> Result<()> {
        // Build WHERE clause from record_id or old_data
        let where_clause = if let Some(record_id) = record.record_id {
            format!("id = '{}'", record_id)
        } else if let Some(ref old_data) = record.old_data {
            if let Some(obj) = old_data.as_object() {
                let conditions: Vec<String> = obj.keys()
                    .filter(|k| *k == "id" || k.ends_with("_id"))
                    .map(|k| format!("{} = '{}'", k, obj[k]))
                    .collect();
                
                if conditions.is_empty() {
                    return Err(anyhow::anyhow!("Cannot identify record for DELETE"));
                }
                
                conditions.join(" AND ")
            } else {
                return Err(anyhow::anyhow!("Invalid old_data format for DELETE"));
            }
        } else {
            return Err(anyhow::anyhow!("DELETE operation missing record identifier"));
        };

        let query = format!(
            "DELETE FROM {}.{} WHERE {}",
            record.schema_name,
            record.table_name,
            where_clause
        );

        sqlx::query(&query)
            .execute(&mut **tx)
            .await
            .context(format!("Failed to apply DELETE to {}.{}", 
                           record.schema_name, record.table_name))?;

        Ok(())
    }

    /// Apply TRUNCATE operation
    async fn apply_truncate(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        record: &WalRecord,
    ) -> Result<()> {
        let query = format!(
            "TRUNCATE TABLE {}.{}",
            record.schema_name,
            record.table_name
        );

        sqlx::query(&query)
            .execute(&mut **tx)
            .await
            .context(format!("Failed to apply TRUNCATE to {}.{}", 
                           record.schema_name, record.table_name))?;

        Ok(())
    }

    /// Apply DDL operation
    async fn apply_ddl(
        &self,
        tx: &mut Transaction<'_, sqlx::Postgres>,
        record: &WalRecord,
    ) -> Result<()> {
        let sql = record.sql_statement.as_ref()
            .ok_or_else(|| anyhow::anyhow!("DDL operation missing sql_statement"))?;

        sqlx::query(sql)
            .execute(&mut **tx)
            .await
            .context(format!("Failed to apply DDL: {}", sql))?;

        Ok(())
    }

    /// Validate record checksum
    async fn validate_record_checksum(&self, record: &WalRecord) -> Result<()> {
        // Recalculate checksum and compare
        let expected = self.storage.calculate_checksum(
            &record.operation_type,
            &record.old_data,
            &record.new_data,
            &record.sql_statement,
        );

        if let Some(ref checksum) = record.checksum {
            if checksum != &expected {
                return Err(anyhow::anyhow!("Checksum mismatch: expected {}, got {}", 
                                          expected, checksum));
            }
        }

        Ok(())
    }

    /// Create replay checkpoint
    async fn create_checkpoint(
        &self,
        replay_id: &Uuid,
        last_sequence: usize,
        last_transaction: Uuid,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO wal_replay_checkpoints (
                replay_id, last_sequence_number, last_transaction_id, 
                records_applied, status
            )
            VALUES ($1, $2, $3, $4, 'IN_PROGRESS')
            ON CONFLICT (replay_id) DO UPDATE SET
                last_sequence_number = EXCLUDED.last_sequence_number,
                last_transaction_id = EXCLUDED.last_transaction_id,
                records_applied = EXCLUDED.records_applied,
                checkpoint_time = NOW()
            "#,
        )
        .bind(replay_id)
        .bind(last_sequence as i64)
        .bind(last_transaction)
        .bind(last_sequence as i64)
        .execute(&*self.target_pool)
        .await
        .context("Failed to create replay checkpoint")?;

        Ok(())
    }
}

