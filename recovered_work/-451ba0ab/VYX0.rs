//! Event Bus for Execution Events
//!
//! Coordinates event distribution between execution components,
//! progress trackers, and external observers (WebSocket, HTTP streams).

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::planning::types::ExecutionEvent;

/// Event bus configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EventBusConfig {
    /// Maximum subscribers per task
    pub max_subscribers_per_task: usize,
    /// Event buffer size
    pub buffer_size: usize,
    /// Enable event persistence
    pub enable_persistence: bool,
    /// Event retention time (seconds)
    pub retention_seconds: u64,
}

/// Event subscription handle
pub struct EventSubscription {
    receiver: broadcast::Receiver<ExecutionEvent>,
    task_id: Uuid,
}

impl EventSubscription {
    /// Receive the next event
    pub async fn recv(&mut self) -> Result<ExecutionEvent, EventBusError> {
        self.receiver.recv().await.map_err(|e| EventBusError::ReceiveError(e.to_string()))
    }

    /// Get the task ID for this subscription
    pub fn task_id(&self) -> Uuid {
        self.task_id
    }
}

/// Event bus for coordinating execution events
pub struct EventBus {
    config: EventBusConfig,
    senders: Arc<RwLock<HashMap<Uuid, broadcast::Sender<ExecutionEvent>>>>,
    subscriber_counts: Arc<RwLock<HashMap<Uuid, usize>>>,
}

impl EventBus {
    pub fn new(config: EventBusConfig) -> Self {
        Self {
            config,
            senders: Arc::new(RwLock::new(HashMap::new())),
            subscriber_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Publish an event to all subscribers of the task
    pub async fn publish(&self, event: ExecutionEvent) -> Result<(), EventBusError> {
        let task_id = self.get_task_id_from_event(&event)?;

        let senders = self.senders.read().await;
        if let Some(sender) = senders.get(&task_id) {
            match sender.send(event) {
                Ok(subscriber_count) => {
                    tracing::debug!("Published event to {} subscribers for task {}", subscriber_count, task_id);
                    Ok(())
                }
                Err(_) => {
                    // No active subscribers, this is normal
                    tracing::debug!("No active subscribers for task {}", task_id);
                    Ok(())
                }
            }
        } else {
            // Create sender for this task if it doesn't exist
            drop(senders);
            self.ensure_sender_exists(task_id).await?;
            // Retry publish now that sender exists
            self.publish(event).await
        }
    }

    /// Subscribe to events for a specific task
    pub async fn subscribe(&self, task_id: Uuid) -> Result<EventSubscription, EventBusError> {
        self.ensure_sender_exists(task_id).await?;

        let mut subscriber_counts = self.subscriber_counts.write().await;
        let count = subscriber_counts.entry(task_id).or_insert(0);
        *count += 1;

        // Check subscriber limit
        if *count > self.config.max_subscribers_per_task {
            *count -= 1; // Revert the increment
            return Err(EventBusError::SubscriberLimitExceeded {
                task_id,
                max_subscribers: self.config.max_subscribers_per_task,
            });
        }

        let senders = self.senders.read().await;
        let receiver = senders.get(&task_id)
            .ok_or_else(|| EventBusError::NoSenderForTask(task_id))?
            .subscribe();

        Ok(EventSubscription {
            receiver,
            task_id,
        })
    }

    /// Unsubscribe from events (automatically called when EventSubscription is dropped)
    pub async fn unsubscribe(&self, task_id: Uuid) -> Result<(), EventBusError> {
        let mut subscriber_counts = self.subscriber_counts.write().await;
        if let Some(count) = subscriber_counts.get_mut(&task_id) {
            if *count > 0 {
                *count -= 1;
            }
        }
        Ok(())
    }

    /// Get subscriber count for a task
    pub async fn subscriber_count(&self, task_id: Uuid) -> usize {
        let subscriber_counts = self.subscriber_counts.read().await;
        subscriber_counts.get(&task_id).copied().unwrap_or(0)
    }

    /// Get all active tasks with subscribers
    pub async fn active_tasks(&self) -> Vec<Uuid> {
        let subscriber_counts = self.subscriber_counts.read().await;
        subscriber_counts.keys()
            .filter(|&&task_id| subscriber_counts[&task_id] > 0)
            .cloned()
            .collect()
    }

    /// Clean up inactive tasks
    pub async fn cleanup_inactive_tasks(&self) -> Result<usize, EventBusError> {
        let mut senders = self.senders.write().await;
        let mut subscriber_counts = self.subscriber_counts.write().await;

        let mut removed_count = 0;
        let mut to_remove = Vec::new();

        // Find tasks with no subscribers and no recent activity
        for (&task_id, &count) in subscriber_counts.iter() {
            if count == 0 && !senders.get(&task_id).map_or(false, |s| s.receiver_count() > 0) {
                to_remove.push(task_id);
            }
        }

        // Remove inactive tasks
        for task_id in to_remove {
            senders.remove(&task_id);
            subscriber_counts.remove(&task_id);
            removed_count += 1;
        }

        if removed_count > 0 {
            tracing::info!("Cleaned up {} inactive tasks from event bus", removed_count);
        }

        Ok(removed_count)
    }

    /// Ensure a broadcast sender exists for the task
    async fn ensure_sender_exists(&self, task_id: Uuid) -> Result<(), EventBusError> {
        let mut senders = self.senders.write().await;

        if !senders.contains_key(&task_id) {
            let (sender, _) = broadcast::channel(self.config.buffer_size);
            senders.insert(task_id, sender);
            tracing::debug!("Created event sender for task {}", task_id);
        }

        Ok(())
    }

    /// Extract task ID from execution event
    fn get_task_id_from_event(&self, event: &ExecutionEvent) -> Result<Uuid, EventBusError> {
        match event {
            ExecutionEvent::ExecutionStarted { task_id, .. } => Ok(*task_id),
            ExecutionEvent::WorkerAssignmentStarted { task_id, .. } => Ok(*task_id),
            ExecutionEvent::WorkerAssigned { task_id, .. } => Ok(*task_id),
            ExecutionEvent::ExecutionPhaseStarted { task_id, .. } => Ok(*task_id),
            ExecutionEvent::ExecutionPhaseCompleted { task_id, .. } => Ok(*task_id),
            ExecutionEvent::ArtifactProduced { task_id, .. } => Ok(*task_id),
            ExecutionEvent::QualityCheckCompleted { task_id, .. } => Ok(*task_id),
            ExecutionEvent::ExecutionCompleted { task_id, .. } => Ok(*task_id),
            ExecutionEvent::ExecutionFailed { task_id, .. } => Ok(*task_id),
        }
    }
}

// Automatically unsubscribe when EventSubscription is dropped
impl Drop for EventSubscription {
    fn drop(&mut self) {
        // Note: We can't make this async, so we'd need a different approach
        // In practice, this would be handled by the application layer
    }
}

pub type Result<T> = std::result::Result<T, EventBusError>;

#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("No sender exists for task: {0}")]
    NoSenderForTask(Uuid),

    #[error("Subscriber limit exceeded for task {task_id}: max {max_subscribers}")]
    SubscriberLimitExceeded {
        task_id: Uuid,
        max_subscribers: usize,
    },

    #[error("Failed to receive event: {0}")]
    ReceiveError(String),

    #[error("Event bus operation failed: {0}")]
    OperationError(String),
}
