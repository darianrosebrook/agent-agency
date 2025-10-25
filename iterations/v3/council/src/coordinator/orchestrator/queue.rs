//! Queue management for council orchestrator
//!
//! Task queue management, scheduling, and load balancing
//! for evaluation tasks in the consensus coordinator.

use super::types::{QueueTracker, QueueTask, QueueTaskStatus, QueueProcessingEvent, QueueEventType};
use chrono::{DateTime, Utc};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Priority queue item for task scheduling
#[derive(Debug, Clone)]
struct PriorityQueueItem {
    task_id: Uuid,
    priority: u8,
    created_at: DateTime<Utc>,
}

impl Ord for PriorityQueueItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then earlier creation time
        other.priority.cmp(&self.priority)
            .then_with(|| self.created_at.cmp(&other.created_at))
    }
}

impl PartialOrd for PriorityQueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for PriorityQueueItem {}
impl PartialEq for PriorityQueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.task_id == other.task_id
    }
}

/// Queue manager for handling task scheduling and execution
#[derive(Debug)]
pub struct QueueManager {
    queue_tracker: Arc<RwLock<QueueTracker>>,
    priority_queue: RwLock<BinaryHeap<PriorityQueueItem>>,
    pending_tasks: RwLock<VecDeque<Uuid>>,
    active_tasks: RwLock<HashMap<Uuid, tokio::time::Instant>>,
}

impl QueueManager {
    /// Create a new queue manager
    pub fn new(queue_tracker: Arc<RwLock<QueueTracker>>) -> Self {
        Self {
            queue_tracker,
            priority_queue: RwLock::new(BinaryHeap::new()),
            pending_tasks: RwLock::new(VecDeque::new()),
            active_tasks: RwLock::new(HashMap::new()),
        }
    }

    /// Enqueue a task for processing
    pub async fn enqueue_task(&self, task: QueueTask) -> Result<(), QueueError> {
        let task_id = task.task_id;
        let priority = task.priority;

        // Add to queue tracker
        {
            let mut queue_tracker = self.queue_tracker.write().await;
            queue_tracker.active_tasks.insert(task_id, task);
        }

        // Add to priority queue
        let queue_item = PriorityQueueItem {
            task_id,
            priority,
            created_at: Utc::now(),
        };

        {
            let mut priority_queue = self.priority_queue.write().await;
            priority_queue.push(queue_item);
        }

        {
            let mut pending_tasks = self.pending_tasks.write().await;
            pending_tasks.push_back(task_id);
        }

        // Record queue event
        let event = QueueProcessingEvent {
            task_id,
            event_type: QueueEventType::TaskEnqueued,
            timestamp: Utc::now(),
            duration_ms: None,
            metadata: HashMap::new(),
        };

        {
            let mut queue_tracker = self.queue_tracker.write().await;
            queue_tracker.processing_history.push(event);
        }

        Ok(())
    }

    /// Dequeue the next highest priority task
    pub async fn dequeue_task(&self) -> Option<Uuid> {
        let task_id = {
            let mut pending_tasks = self.pending_tasks.write().await;
            pending_tasks.pop_front()
        };

        if let Some(task_id) = task_id {
            // Mark as active
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.insert(task_id, tokio::time::Instant::now());

            // Update task status
            let mut queue_tracker = self.queue_tracker.write().await;
            if let Some(task) = queue_tracker.active_tasks.get_mut(&task_id) {
                task.status = QueueTaskStatus::Processing;
                task.started_at = Some(Utc::now());
            }

            // Record start event
            let event = QueueProcessingEvent {
                task_id,
                event_type: QueueEventType::TaskStarted,
                timestamp: Utc::now(),
                duration_ms: None,
                metadata: HashMap::new(),
            };
            queue_tracker.processing_history.push(event);

            Some(task_id)
        } else {
            None
        }
    }

    /// Mark a task as completed
    pub async fn complete_task(&self, task_id: Uuid) -> Result<(), QueueError> {
        let duration = {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.remove(&task_id)
        };

        let actual_duration = duration.map(|start| start.elapsed().as_millis() as u64);

        // Update task status
        {
            let mut queue_tracker = self.queue_tracker.write().await;
            if let Some(task) = queue_tracker.active_tasks.get_mut(&task_id) {
                task.status = QueueTaskStatus::Completed;
                task.completed_at = Some(Utc::now());
                task.actual_duration_ms = actual_duration;
            }

            // Record completion event
            let event = QueueProcessingEvent {
                task_id,
                event_type: QueueEventType::TaskCompleted,
                timestamp: Utc::now(),
                duration_ms: actual_duration,
                metadata: HashMap::new(),
            };
            queue_tracker.processing_history.push(event);
        }

        Ok(())
    }

    /// Mark a task as failed
    pub async fn fail_task(&self, task_id: Uuid) -> Result<(), QueueError> {
        let duration = {
            let mut active_tasks = self.active_tasks.write().await;
            active_tasks.remove(&task_id)
        };

        let actual_duration = duration.map(|start| start.elapsed().as_millis() as u64);

        // Update task status
        {
            let mut queue_tracker = self.queue_tracker.write().await;
            if let Some(task) = queue_tracker.active_tasks.get_mut(&task_id) {
                task.status = QueueTaskStatus::Failed;
                task.completed_at = Some(Utc::now());
                task.actual_duration_ms = actual_duration;
            }

            // Record failure event
            let event = QueueProcessingEvent {
                task_id,
                event_type: QueueEventType::TaskFailed,
                timestamp: Utc::now(),
                duration_ms: actual_duration,
                metadata: HashMap::new(),
            };
            queue_tracker.processing_history.push(event);
        }

        Ok(())
    }

    /// Cancel a task
    pub async fn cancel_task(&self, task_id: Uuid) -> Result<(), QueueError> {
        // Remove from active tasks
        let mut active_tasks = self.active_tasks.write().await;
        active_tasks.remove(&task_id);

        // Update task status
        {
            let mut queue_tracker = self.queue_tracker.write().await;
            if let Some(task) = queue_tracker.active_tasks.get_mut(&task_id) {
                task.status = QueueTaskStatus::Cancelled;
                task.completed_at = Some(Utc::now());
            }

            // Record cancellation event
            let event = QueueProcessingEvent {
                task_id,
                event_type: QueueEventType::TaskCancelled,
                timestamp: Utc::now(),
                duration_ms: None,
                metadata: HashMap::new(),
            };
            queue_tracker.processing_history.push(event);
        }

        // Remove from pending tasks
        let mut pending_tasks = self.pending_tasks.write().await;
        pending_tasks.retain(|&id| id != task_id);

        Ok(())
    }

    /// Get queue statistics
    pub async fn get_queue_stats(&self) -> QueueStats {
        let queue_tracker = self.queue_tracker.read().await;
        let pending_count = self.pending_tasks.read().await.len();
        let active_count = self.active_tasks.read().await.len();

        QueueStats {
            pending_tasks: pending_count,
            active_tasks: active_count,
            total_tasks: queue_tracker.active_tasks.len(),
            max_depth: queue_tracker.performance_metrics.peak_depth,
            avg_processing_time_ms: queue_tracker.performance_metrics.avg_processing_time_ms,
        }
    }

    /// Check for timeouts and cancel expired tasks
    pub async fn check_timeouts(&self) -> Result<Vec<Uuid>, QueueError> {
        let timeout_duration = std::time::Duration::from_secs(300); // 5 minutes
        let mut expired_tasks = Vec::new();

        let active_tasks = self.active_tasks.read().await.clone();

        for (task_id, start_time) in active_tasks {
            if start_time.elapsed() > timeout_duration {
                expired_tasks.push(task_id);
            }
        }

        // Cancel expired tasks
        for task_id in &expired_tasks {
            let _ = self.cancel_task(*task_id).await;
        }

        Ok(expired_tasks)
    }

    /// Optimize queue based on current conditions
    pub async fn optimize_queue(&self) -> Result<(), QueueError> {
        // Record optimization event
        let event = QueueProcessingEvent {
            task_id: Uuid::new_v4(), // Not task-specific
            event_type: QueueEventType::QueueOptimized,
            timestamp: Utc::now(),
            duration_ms: None,
            metadata: HashMap::new(),
        };

        let mut queue_tracker = self.queue_tracker.write().await;
        queue_tracker.processing_history.push(event);

        Ok(())
    }
}

/// Queue statistics
#[derive(Debug, Clone)]
pub struct QueueStats {
    pub pending_tasks: usize,
    pub active_tasks: usize,
    pub total_tasks: usize,
    pub max_depth: u64,
    pub avg_processing_time_ms: u64,
}

/// Queue operation errors
#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),

    #[error("Queue is full")]
    QueueFull,

    #[error("Invalid task state")]
    InvalidTaskState,
}


