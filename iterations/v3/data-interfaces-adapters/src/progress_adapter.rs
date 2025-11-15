//! Progress Tracking Service Adapter
//!
//! Adapts progress tracking implementations to `data-interfaces` service traits.
//! Uses database-backed persistent storage for progress tracking with real-time
//! streaming support for active subscribers.

use async_trait::async_trait;
use data_interfaces::service_contracts::{
    ProgressInfo, ProgressStream, ProgressTrackingService, ProgressUpdate, ServiceError,
};
use data_infrastructure::simple_client::DatabaseClient;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Adapter for progress tracking service
pub struct ProgressTrackingServiceAdapter {
    /// Database client for persistent progress storage
    db_client: Option<Arc<DatabaseClient>>,
    /// Active progress streams (keyed by task_id)
    /// Each entry contains a sender that broadcasts progress updates
    /// Note: Streams are in-memory for real-time updates, but progress is persisted to database
    active_streams: Arc<RwLock<HashMap<Uuid, Vec<mpsc::UnboundedSender<ProgressInfo>>>>>,
}

impl ProgressTrackingServiceAdapter {
    /// Create a new progress tracking service adapter without database client
    /// This will attempt to create a database client from environment variables
    /// if DATABASE_URL is set, otherwise it will operate in degraded mode (in-memory only).
    pub fn new() -> Self {
        Self::new_with_db_client(None)
    }

    /// Create a new progress tracking service adapter with optional database client
    pub fn new_with_db_client(db_client: Option<Arc<DatabaseClient>>) -> Self {
        // If no database client provided, try to create one from environment
        let client = if let Some(db) = db_client {
            Some(db)
        } else {
            // Try to create a database client from environment
            match std::env::var("DATABASE_URL") {
                Ok(database_url) => {
                    let config = data_infrastructure::database_config::DatabaseConfig {
                        database_url,
                        ..Default::default()
                    };
                    match tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(DatabaseClient::new(config))
                    }) {
                        Ok(db) => {
                            info!("Created database client for progress tracking from DATABASE_URL");
                            Some(Arc::new(db))
                        }
                        Err(e) => {
                            warn!(
                                "Failed to create database client from DATABASE_URL: {}. Progress tracking will operate in degraded mode (in-memory only).",
                                e
                            );
                            None
                        }
                    }
                }
                Err(_) => {
                    warn!("DATABASE_URL not set. Progress tracking will operate in degraded mode (in-memory only).");
                    None
                }
            }
        };

        Self {
            db_client: client,
            active_streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new progress tracking service adapter with explicit database client
    pub fn with_db_client(db_client: Arc<DatabaseClient>) -> Self {
        Self {
            db_client: Some(db_client),
            active_streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store progress in database
    async fn store_progress_in_db(
        &self,
        task_id: &Uuid,
        progress_info: &ProgressInfo,
    ) -> Result<(), ServiceError> {
        let Some(ref db) = self.db_client else {
            // If no database client, just log a warning but don't fail
            // This allows the system to work in degraded mode
            warn!("No database client available for progress storage");
            return Ok(());
        };

        let query = r#"
            INSERT INTO task_progress (
                task_id, progress_percent, current_stage, status_message, metadata
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (task_id) DO UPDATE SET
                progress_percent = EXCLUDED.progress_percent,
                current_stage = EXCLUDED.current_stage,
                status_message = EXCLUDED.status_message,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
        "#;

        let metadata = serde_json::json!({});

        match sqlx::query(query)
            .bind(task_id)
            .bind(progress_info.progress_percent as i16)
            .bind(&progress_info.current_stage)
            .bind(progress_info.status_message.as_ref())
            .bind(&metadata)
            .execute(db.pool())
            .await
        {
            Ok(_) => {
                info!(
                    "Stored progress for task {}: {}% - {}",
                    task_id, progress_info.progress_percent, progress_info.current_stage
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to store progress in database: {}", e);
                // Don't fail the entire operation if database write fails
                // This allows the system to continue operating
                Err(ServiceError::Internal(format!(
                    "Failed to store progress: {}",
                    e
                )))
            }
        }
    }

    /// Retrieve progress from database
    async fn get_progress_from_db(
        &self,
        task_id: &Uuid,
    ) -> Result<Option<ProgressInfo>, ServiceError> {
        let Some(ref db) = self.db_client else {
            return Ok(None);
        };

        let query = r#"
            SELECT task_id, progress_percent, current_stage, status_message
            FROM task_progress
            WHERE task_id = $1
        "#;

        match sqlx::query(query)
            .bind(task_id)
            .fetch_optional(db.pool())
            .await
        {
            Ok(Some(row)) => {
                let progress_percent: i16 = row.try_get("progress_percent")?;
                Ok(Some(ProgressInfo {
                    task_id: row.try_get("task_id")?,
                    progress_percent: progress_percent as u8,
                    current_stage: row.try_get("current_stage")?,
                    status_message: row.try_get("status_message")?,
                }))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                error!("Failed to retrieve progress from database: {}", e);
                Err(ServiceError::Internal(format!(
                    "Failed to retrieve progress: {}",
                    e
                )))
            }
        }
    }
}

#[async_trait]
impl ProgressTrackingService for ProgressTrackingServiceAdapter {
    async fn track_progress(
        &self,
        task_id: &Uuid,
        progress: ProgressUpdate,
    ) -> Result<(), ServiceError> {
        // Create ProgressInfo from ProgressUpdate
        let progress_info = ProgressInfo {
            task_id: *task_id,
            progress_percent: progress.progress_percent,
            current_stage: progress
                .status_message
                .clone()
                .unwrap_or_else(|| "In progress".to_string()),
            status_message: progress.status_message,
        };

        // Store progress in database (persistent storage)
        if let Err(e) = self.store_progress_in_db(task_id, &progress_info).await {
            // Log error but continue - we can still broadcast to streams
            error!("Failed to store progress in database: {}", e);
        }

        // Broadcast to all active streams for this task (real-time updates)
        {
            let streams = self.active_streams.read().await;
            if let Some(senders) = streams.get(task_id) {
                let mut dead_senders = Vec::new();
                for (idx, sender) in senders.iter().enumerate() {
                    if sender.send(progress_info.clone()).is_err() {
                        // Receiver dropped, mark for removal
                        dead_senders.push(idx);
                    }
                }
                // Note: We'll clean up dead senders on next update when we have mutable access
            }
        }

        info!(
            "Tracking progress for task {}: {}% - {}",
            task_id, progress_info.progress_percent, progress_info.current_stage
        );
        Ok(())
    }

    async fn get_progress(&self, task_id: &Uuid) -> Result<ProgressInfo, ServiceError> {
        // Try to retrieve progress from database first
        match self.get_progress_from_db(task_id).await {
            Ok(Some(progress)) => {
                info!("Retrieved progress for task {} from database", task_id);
                return Ok(progress);
            }
            Ok(None) => {
                // No progress found in database, return default
                info!("No progress found in database for task {}", task_id);
            }
            Err(e) => {
                // Database query failed, log and return default
                warn!("Failed to retrieve progress from database: {}", e);
            }
        }

        // Return default progress info if not found in database
        Ok(ProgressInfo {
            task_id: *task_id,
            progress_percent: 0,
            current_stage: "Unknown".to_string(),
            status_message: Some("No progress information available".to_string()),
        })
    }

    async fn subscribe_progress(&self, task_id: &Uuid) -> Result<ProgressStream, ServiceError> {
        // Create channel for progress updates
        let (tx, rx) = mpsc::unbounded_channel();

        // Register sender for this task
        {
            let mut streams = self.active_streams.write().await;
            streams
                .entry(*task_id)
                .or_insert_with(Vec::new)
                .push(tx.clone());
        }

        // Send current progress immediately if available in database
        if let Ok(Some(progress)) = self.get_progress_from_db(task_id).await {
            let _ = tx.send(progress);
        }

        info!("Subscribed to progress updates for task {}", task_id);
        Ok(rx)
    }
}
