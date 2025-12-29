//! Task Queue Implementation
//!
//! Provides a robust task queue service for processing tasks asynchronously
//! with priority support, persistence, and monitoring capabilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::BinaryHeap;
use std::sync::Arc;
use tokio::sync::{Notify, RwLock};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::DatabaseClient;

/// Task queue service for managing asynchronous task processing
pub struct TaskQueueService {
    /// Database client for persistence
    db_client: Arc<DatabaseClient>,
    /// In-memory priority queue for active tasks
    priority_queue: Arc<RwLock<BinaryHeap<PrioritizedTask>>>,
    /// Notification for waiting workers
    notify: Arc<Notify>,
    /// Queue metrics
    metrics: Arc<RwLock<QueueMetrics>>,
}

/// Task with priority for queue ordering
#[derive(Debug, Clone, PartialEq, Eq)]
struct PrioritizedTask {
    /// Task priority (higher = more important)
    priority: i32,
    /// Task ID for tie-breaking
    id: Uuid,
    /// Task data
    task: QueuedTask,
}

impl Ord for PrioritizedTask {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then earlier ID for stability
        other
            .priority
            .cmp(&self.priority)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for PrioritizedTask {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Task stored in the queue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueuedTask {
    /// Unique task identifier
    pub id: Uuid,
    /// Task type/name
    pub task_type: String,
    /// Task priority (higher = more important)
    pub priority: i32,
    /// Task payload data
    pub payload: serde_json::Value,
    /// When the task was created
    pub created_at: DateTime<Utc>,
    /// When the task was last updated
    pub updated_at: DateTime<Utc>,
    /// Current task status
    pub status: TaskStatus,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// When to process the task (for delayed tasks)
    pub process_at: Option<DateTime<Utc>>,
    /// Task timeout in seconds
    pub timeout_seconds: Option<u64>,
}

/// Task status in the queue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    /// Task is waiting to be processed
    Pending,
    /// Task is currently being processed
    Processing,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was cancelled
    Cancelled,
    /// Task timed out
    Timeout,
}

/// Queue metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMetrics {
    /// Total tasks processed
    pub tasks_processed: u64,
    /// Total tasks failed
    pub tasks_failed: u64,
    /// Current queue depth
    pub queue_depth: usize,
    /// Average processing time in milliseconds
    pub avg_processing_time_ms: f64,
    /// Tasks processed per second
    pub throughput: f64,
    /// Current error rate (0.0-1.0)
    pub error_rate: f64,
}

/// Result of dequeuing a task
#[derive(Debug)]
pub struct DequeueResult {
    /// The dequeued task
    pub task: QueuedTask,
    /// Acknowledgement token for completion
    pub ack_token: String,
}

impl TaskQueueService {
    /// Create a new task queue service
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self {
            db_client,
            priority_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            notify: Arc::new(Notify::new()),
            metrics: Arc::new(RwLock::new(QueueMetrics {
                tasks_processed: 0,
                tasks_failed: 0,
                queue_depth: 0,
                avg_processing_time_ms: 0.0,
                throughput: 0.0,
                error_rate: 0.0,
            })),
        }
    }

    /// Enqueue a new task
    pub async fn enqueue_task(&self, task: QueuedTask) -> Result<(), TaskQueueError> {
        debug!(
            "Enqueuing task: {} (type: {}, priority: {})",
            task.id, task.task_type, task.priority
        );

        // Insert into database for persistence
        self.persist_task(&task).await?;

        // Add to in-memory queue
        let prioritized_task = PrioritizedTask {
            priority: task.priority,
            id: task.id,
            task: task.clone(),
        };

        {
            let mut queue = self.priority_queue.write().await;
            queue.push(prioritized_task);

            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.queue_depth = queue.len();
        }

        // Notify waiting workers
        self.notify.notify_one();

        info!(
            "Task enqueued successfully: {} (queue depth: {})",
            task.id,
            self.metrics.read().await.queue_depth
        );

        Ok(())
    }

    /// Dequeue the highest priority task
    pub async fn dequeue_task(&self) -> Result<Option<DequeueResult>, TaskQueueError> {
        // Get the highest priority task
        let prioritized_task = {
            let mut queue = self.priority_queue.write().await;
            queue.pop()
        };

        if let Some(prioritized_task) = prioritized_task {
            // Update task status to processing
            self.update_task_status(prioritized_task.task.id, TaskStatus::Processing)
                .await?;

            // Generate acknowledgement token
            let ack_token = format!(
                "ack_{}_{}",
                prioritized_task.task.id,
                Utc::now().timestamp()
            );

            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.queue_depth = self.priority_queue.read().await.len();
            }

            debug!(
                "Task dequeued: {} (remaining queue depth: {})",
                prioritized_task.task.id,
                self.metrics.read().await.queue_depth
            );

            Ok(Some(DequeueResult {
                task: prioritized_task.task,
                ack_token,
            }))
        } else {
            Ok(None)
        }
    }

    /// Acknowledge task completion
    pub async fn acknowledge_task(
        &self,
        task_id: Uuid,
        _ack_token: &str,
        success: bool,
    ) -> Result<(), TaskQueueError> {
        debug!("Acknowledging task: {} (success: {})", task_id, success);

        let new_status = if success {
            TaskStatus::Completed
        } else {
            TaskStatus::Failed
        };

        // Update task status in database
        self.update_task_status(task_id, new_status.clone()).await?;

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            if success {
                metrics.tasks_processed += 1;
            } else {
                metrics.tasks_failed += 1;
                // Increment retry count and re-queue if retries remaining
                if let Some(task) = self.get_task(task_id).await? {
                    if task.retry_count < task.max_retries {
                        let retry_count = task.retry_count + 1;
                        debug!(
                            "Task {} re-queued for retry (attempt {}/{})",
                            task_id, retry_count, task.max_retries
                        );

                        let mut retry_task = task;
                        retry_task.retry_count = retry_count;
                        retry_task.status = TaskStatus::Pending;
                        retry_task.updated_at = Utc::now();

                        // Re-queue the task
                        let _ = self.enqueue_task(retry_task).await;
                    }
                }
            }

            // Update error rate
            let total_tasks = metrics.tasks_processed + metrics.tasks_failed;
            if total_tasks > 0 {
                metrics.error_rate = metrics.tasks_failed as f64 / total_tasks as f64;
            }
        }

        info!("Task acknowledged: {} (status: {:?})", task_id, new_status);
        Ok(())
    }

    /// Get current queue metrics
    pub async fn get_metrics(&self) -> QueueMetrics {
        self.metrics.read().await.clone()
    }

    /// Wait for a task to become available
    pub async fn wait_for_task(&self) {
        self.notify.notified().await;
    }

    /// Get a specific task by ID
    async fn get_task(&self, task_id: Uuid) -> Result<Option<QueuedTask>, TaskQueueError> {
        // Query from database
        let query = r#"
            SELECT id, task_type, priority, payload, created_at, updated_at,
                   status, retry_count, max_retries, process_at, timeout_seconds
            FROM tasks WHERE id = $1
        "#;

        let row_result = self.db_client.query_one(query, &[&task_id]).await;

        match row_result {
            Ok(Some(row)) => {
                let task = QueuedTask {
                    id: row
                        .try_get("id")
                        .map_err(|e| TaskQueueError::Generic(format!("Failed to get id: {}", e)))?,
                    task_type: row.try_get("task_type").map_err(|e| {
                        TaskQueueError::Generic(format!("Failed to get task_type: {}", e))
                    })?,
                    priority: row.try_get("priority").map_err(|e| {
                        TaskQueueError::Generic(format!("Failed to get priority: {}", e))
                    })?,
                    payload: row.try_get("payload").map_err(|e| {
                        TaskQueueError::Generic(format!("Failed to get payload: {}", e))
                    })?,
                    created_at: row.try_get("created_at").map_err(|e| {
                        TaskQueueError::Generic(format!("Failed to get created_at: {}", e))
                    })?,
                    updated_at: row.try_get("updated_at").map_err(|e| {
                        TaskQueueError::Generic(format!("Failed to get updated_at: {}", e))
                    })?,
                    status: {
                        let status_val: serde_json::Value = row.try_get("status").map_err(|e| {
                            TaskQueueError::Generic(format!("Failed to get status: {}", e))
                        })?;
                        serde_json::from_value(status_val).map_err(|e| {
                            TaskQueueError::Generic(format!("Failed to parse status: {}", e))
                        })?
                    },
                    retry_count: row.try_get::<i32, _>("retry_count").map_err(|e| {
                        TaskQueueError::Generic(format!("Failed to get retry_count: {}", e))
                    })? as u32,
                    max_retries: row.try_get::<i32, _>("max_retries").map_err(|e| {
                        TaskQueueError::Generic(format!("Failed to get max_retries: {}", e))
                    })? as u32,
                    process_at: row.try_get("process_at").map_err(|e| {
                        TaskQueueError::Generic(format!("Failed to get process_at: {}", e))
                    })?,
                    timeout_seconds: {
                        let timeout_opt: Option<i32> =
                            row.try_get("timeout_seconds").map_err(|e| {
                                TaskQueueError::Generic(format!(
                                    "Failed to get timeout_seconds: {}",
                                    e
                                ))
                            })?;
                        timeout_opt.map(|t| t as u64)
                    },
                };
                Ok(Some(task))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(TaskQueueError::from_anyhow(e)),
        }
    }

    /// Persist task to database
    async fn persist_task(&self, task: &QueuedTask) -> Result<(), TaskQueueError> {
        let query = r#"
            INSERT INTO tasks (
                id, task_type, priority, payload, created_at, updated_at,
                status, retry_count, max_retries, process_at, timeout_seconds
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (id) DO UPDATE SET
                priority = EXCLUDED.priority,
                payload = EXCLUDED.payload,
                updated_at = EXCLUDED.updated_at,
                status = EXCLUDED.status,
                retry_count = EXCLUDED.retry_count
        "#;

        let status_value = serde_json::to_value(&task.status)?;

        self.db_client
            .execute(
                query,
                &[
                    &task.id,
                    &task.task_type,
                    &task.priority,
                    &task.payload,
                    &task.created_at,
                    &task.updated_at,
                    &status_value,
                    &(task.retry_count as i32),
                    &(task.max_retries as i32),
                    &task.process_at,
                    &(task.timeout_seconds.map(|t| t as i32)),
                ],
            )
            .await
            .map_err(TaskQueueError::from_anyhow)?;

        Ok(())
    }

    /// Update task status in database
    async fn update_task_status(
        &self,
        task_id: Uuid,
        status: TaskStatus,
    ) -> Result<(), TaskQueueError> {
        let query = r#"
            UPDATE tasks
            SET status = $1, updated_at = NOW()
            WHERE id = $2
        "#;

        let status_value = serde_json::to_value(&status)?;
        self.db_client
            .execute(query, &[&status_value, &task_id])
            .await
            .map_err(TaskQueueError::from_anyhow)?;

        Ok(())
    }
}

/// Errors that can occur in the task queue
#[derive(Debug, thiserror::Error)]
pub enum TaskQueueError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),

    #[error("Invalid task status transition")]
    InvalidStatusTransition,

    #[error("Queue is full")]
    QueueFull,

    #[error("Generic error: {0}")]
    Generic(String),
}

impl TaskQueueError {
    /// Convert an anyhow::Error to TaskQueueError
    fn from_anyhow(error: anyhow::Error) -> Self {
        // Check if the error chain contains sqlx::Error
        // If so, we can try to extract it, otherwise use Generic
        let error_string = error.to_string();
        if error_string.contains("database") || error_string.contains("sqlx") {
            // Try to find sqlx::Error in the chain
            TaskQueueError::Generic(format!("Database error: {}", error_string))
        } else {
            TaskQueueError::Generic(error_string)
        }
    }
}

/// Task queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskQueueConfig {
    /// Maximum queue depth
    pub max_queue_depth: usize,
    /// Default task timeout in seconds
    pub default_timeout_seconds: u64,
    /// Default maximum retry attempts
    pub default_max_retries: u32,
    /// Database table name
    pub table_name: String,
}

impl Default for TaskQueueConfig {
    fn default() -> Self {
        Self {
            max_queue_depth: 10000,
            default_timeout_seconds: 300, // 5 minutes
            default_max_retries: 3,
            table_name: "tasks".to_string(),
        }
    }
}
