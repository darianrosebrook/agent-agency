//! Telemetry Database Storage
//!
//! Provides database persistence for telemetry data using PostgreSQL.
//! Implements storage for telemetry data points and batches.

use anyhow::{Context, Result};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

use crate::{TelemetryData, TelemetryDataType, TelemetryBatch, TelemetryError};

/// Database storage for telemetry data
#[derive(Clone)]
pub struct TelemetryDatabaseStorage {
    pool: Arc<PgPool>,
}

impl TelemetryDatabaseStorage {
    /// Create a new telemetry database storage instance
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await
            .context("Failed to create telemetry database connection pool")?;

        // Test the connection
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await
            .context("Failed to test telemetry database connection")?;

        debug!("Telemetry database storage initialized successfully");

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    /// Create from existing pool
    pub fn from_pool(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Store a single telemetry data point
    pub async fn store_data(&self, data: &TelemetryData) -> Result<Uuid, TelemetryError> {
        let data_type_str = match data.data_type {
            TelemetryDataType::Metric => "Metric",
            TelemetryDataType::Log => "Log",
            TelemetryDataType::Trace => "Trace",
            TelemetryDataType::Event => "Event",
            TelemetryDataType::Custom => "Custom",
        };

        let tags_json = serde_json::to_value(&data.tags)
            .map_err(|e| TelemetryError::ConfigurationError {
                message: format!("Failed to serialize tags: {}", e),
            })?;

        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO telemetry_data (id, timestamp, source, data_type, payload, tags)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#
        )
        .bind(id)
        .bind(data.timestamp.clone())
        .bind(data.source.clone())
        .bind(data_type_str)
        .bind(data.payload.clone())
        .bind(tags_json)
        .execute(&*self.pool)
        .await
        .map_err(|e| TelemetryError::ConnectionError {
            message: format!("Failed to store telemetry data: {}", e),
        })?;

        debug!("Stored telemetry data point: {}", id);
        Ok(id)
    }

    /// Store multiple telemetry data points in a batch
    pub async fn store_batch(&self, batch: &TelemetryBatch) -> Result<(), TelemetryError> {
        let mut tx = self.pool.begin().await.map_err(|e| TelemetryError::ConnectionError {
            message: format!("Failed to begin transaction: {}", e),
        })?;

        // Store batch metadata
        sqlx::query(
            r#"
            INSERT INTO telemetry_batches (id, timestamp, data_count, processing_status)
            VALUES ($1, $2, $3, 'pending')
            ON CONFLICT (id) DO UPDATE
            SET data_count = EXCLUDED.data_count,
                timestamp = EXCLUDED.timestamp
            "#
        )
        .bind(batch.id.clone())
        .bind(batch.timestamp.clone())
        .bind(batch.data_points.len() as i32)
        .execute(&mut *tx)
        .await
        .map_err(|e| TelemetryError::ConnectionError {
            message: format!("Failed to store batch metadata: {}", e),
        })?;

        // Store all data points
        for data in &batch.data_points {
            let data_type_str = match data.data_type {
                TelemetryDataType::Metric => "Metric",
                TelemetryDataType::Log => "Log",
                TelemetryDataType::Trace => "Trace",
                TelemetryDataType::Event => "Event",
                TelemetryDataType::Custom => "Custom",
            };

            let tags_json = serde_json::to_value(&data.tags)
                .map_err(|e| TelemetryError::ConfigurationError {
                    message: format!("Failed to serialize tags: {}", e),
                })?;

            let id = Uuid::new_v4();

            sqlx::query(
                r#"
                INSERT INTO telemetry_data (id, timestamp, source, data_type, payload, tags)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#
            )
            .bind(id)
            .bind(data.timestamp.clone())
            .bind(data.source.clone())
            .bind(data_type_str)
            .bind(data.payload.clone())
            .bind(tags_json)
            .execute(&mut *tx)
            .await
            .map_err(|e| TelemetryError::ConnectionError {
                message: format!("Failed to store telemetry data point: {}", e),
            })?;
        }

        // Mark batch as completed
        sqlx::query(
            r#"
            UPDATE telemetry_batches
            SET processing_status = 'completed', processed_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(batch.id.clone())
        .execute(&mut *tx)
        .await
        .map_err(|e| TelemetryError::ConnectionError {
            message: format!("Failed to update batch status: {}", e),
        })?;

        tx.commit().await.map_err(|e| TelemetryError::ConnectionError {
            message: format!("Failed to commit transaction: {}", e),
        })?;

        debug!("Stored telemetry batch: {} with {} data points", batch.id, batch.data_points.len());
        Ok(())
    }

    /// Query telemetry data by source and time range
    pub async fn query_data(
        &self,
        source: Option<&str>,
        data_type: Option<TelemetryDataType>,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        limit: Option<i64>,
    ) -> Result<Vec<TelemetryData>, TelemetryError> {
        use sqlx::Row;

        let mut query_builder = sqlx::QueryBuilder::new("SELECT id, timestamp, source, data_type, payload, tags FROM telemetry_data WHERE 1=1");
        let mut separated = query_builder.separated(" AND ");

        if let Some(src) = source {
            separated.push("source = ");
            separated.push_bind(src);
        }

        if let Some(dt) = data_type {
            let dt_str = match dt {
                TelemetryDataType::Metric => "Metric",
                TelemetryDataType::Log => "Log",
                TelemetryDataType::Trace => "Trace",
                TelemetryDataType::Event => "Event",
                TelemetryDataType::Custom => "Custom",
            };
            separated.push("data_type = ");
            separated.push_bind(dt_str);
        }

        if let Some(start) = start_time {
            separated.push("timestamp >= ");
            separated.push_bind(start);
        }

        if let Some(end) = end_time {
            separated.push("timestamp <= ");
            separated.push_bind(end);
        }

        query_builder.push(" ORDER BY timestamp DESC");

        if let Some(lim) = limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(lim);
        }

        let rows = query_builder
            .build()
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| TelemetryError::ConnectionError {
                message: format!("Failed to query telemetry data: {}", e),
            })?;

        // Parse rows into TelemetryData
        let mut results = Vec::new();
        for row in rows {
            let data_type_str: String = row.get("data_type");
            let data_type = match data_type_str.as_str() {
                "Metric" => TelemetryDataType::Metric,
                "Log" => TelemetryDataType::Log,
                "Trace" => TelemetryDataType::Trace,
                "Event" => TelemetryDataType::Event,
                "Custom" => TelemetryDataType::Custom,
                _ => return Err(TelemetryError::ConfigurationError {
                    message: format!("Unknown data type: {}", data_type_str),
                }),
            };

            let tags_value: serde_json::Value = row.get("tags");
            let tags: std::collections::HashMap<String, String> = serde_json::from_value(tags_value)
                .map_err(|e| TelemetryError::ConfigurationError {
                    message: format!("Failed to deserialize tags: {}", e),
                })?;

            results.push(TelemetryData {
                timestamp: row.get("timestamp"),
                source: row.get("source"),
                data_type,
                payload: row.get("payload"),
                tags,
            });
        }

        Ok(results)
    }

    /// Clean up old telemetry data
    pub async fn cleanup_old_data(&self, retention_days: i32) -> Result<i64, TelemetryError> {
        let deleted_count: Option<i64> = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT cleanup_old_telemetry_data($1)"
        )
        .bind(retention_days)
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| TelemetryError::ConnectionError {
            message: format!("Failed to cleanup old telemetry data: {}", e),
        })?;

        debug!("Cleaned up {} old telemetry data points", deleted_count.unwrap_or(0));
        Ok(deleted_count.unwrap_or(0))
    }

    /// Get database connection pool (for advanced usage)
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

