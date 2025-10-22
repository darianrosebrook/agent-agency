//! Worker communication message types and handling

use crate::types::*;
use chrono::{DateTime, Utc};

/// Central message broker for coordinating worker communication
pub struct MessageBroker {
    sender: tokio::sync::mpsc::UnboundedSender<WorkerMessage>,
    receiver: tokio::sync::mpsc::UnboundedReceiver<WorkerMessage>,
}

impl MessageBroker {
    /// Create a new message broker with the given buffer size
    pub fn new(buffer_size: usize) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        Self { sender, receiver }
    }

    /// Get a sender handle for workers to send messages
    pub fn sender(&self) -> tokio::sync::mpsc::UnboundedSender<WorkerMessage> {
        self.sender.clone()
    }

    /// Receive the next message (non-blocking)
    pub fn try_recv(&mut self) -> Result<WorkerMessage, tokio::sync::mpsc::error::TryRecvError> {
        self.receiver.try_recv()
    }

    /// Receive the next message asynchronously
    pub async fn recv(&mut self) -> Option<WorkerMessage> {
        self.receiver.recv().await
    }

    /// Check if there are pending messages
    pub fn has_messages(&self) -> bool {
        !self.receiver.is_empty()
    }

    /// Get the number of pending messages
    pub fn pending_count(&self) -> usize {
        self.receiver.len()
    }
}

impl Default for MessageBroker {
    fn default() -> Self {
        Self::new(1000)
    }
}

/// Communication hub that manages multiple message brokers
pub struct CommunicationHub {
    brokers: std::collections::HashMap<TaskId, MessageBroker>,
}

impl CommunicationHub {
    pub fn new() -> Self {
        Self {
            brokers: std::collections::HashMap::new(),
        }
    }

    /// Create a new broker for the given task
    pub fn create_broker(&mut self, task_id: TaskId, buffer_size: usize) -> &MessageBroker {
        self.brokers.insert(task_id.clone(), MessageBroker::new(buffer_size));
        self.brokers.get(&task_id).unwrap()
    }

    /// Get the broker for the given task
    pub fn get_broker(&self, task_id: &TaskId) -> Option<&MessageBroker> {
        self.brokers.get(task_id)
    }

    /// Get a mutable reference to the broker for the given task
    pub fn get_broker_mut(&mut self, task_id: &TaskId) -> Option<&mut MessageBroker> {
        self.brokers.get_mut(task_id)
    }

    /// Remove and return the broker for the given task
    pub fn remove_broker(&mut self, task_id: &TaskId) -> Option<MessageBroker> {
        self.brokers.remove(task_id)
    }

    /// Clean up brokers for completed tasks
    pub fn cleanup_completed_tasks(&mut self, completed_tasks: &[TaskId]) {
        for task_id in completed_tasks {
            self.brokers.remove(task_id);
        }
    }

    /// Get all active task IDs
    pub fn active_tasks(&self) -> Vec<TaskId> {
        self.brokers.keys().cloned().collect()
    }

    /// Check if a task has an active broker
    pub fn has_active_broker(&self, task_id: &TaskId) -> bool {
        self.brokers.contains_key(task_id)
    }
}

impl Default for CommunicationHub {
    fn default() -> Self {
        Self::new()
    }
}

/// Message filter for selective message processing
pub struct MessageFilter {
    pub worker_ids: Option<Vec<WorkerId>>,
    pub subtask_ids: Option<Vec<SubTaskId>>,
    pub message_types: Option<Vec<MessageType>>,
}

/// Types of messages for filtering
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageType {
    Started,
    Progress,
    Blocked,
    Completed,
    Failed,
}

impl From<&WorkerMessage> for MessageType {
    fn from(message: &WorkerMessage) -> Self {
        match message {
            WorkerMessage::Started { .. } => MessageType::Started,
            WorkerMessage::Progress { .. } => MessageType::Progress,
            WorkerMessage::Blocked { .. } => MessageType::Blocked,
            WorkerMessage::Completed { .. } => MessageType::Completed,
            WorkerMessage::Failed { .. } => MessageType::Failed,
        }
    }
}

impl MessageFilter {
    /// Create a new message filter
    pub fn new() -> Self {
        Self {
            worker_ids: None,
            subtask_ids: None,
            message_types: None,
        }
    }

    /// Filter by specific worker IDs
    pub fn workers(mut self, worker_ids: Vec<WorkerId>) -> Self {
        self.worker_ids = Some(worker_ids);
        self
    }

    /// Filter by specific subtask IDs
    pub fn subtasks(mut self, subtask_ids: Vec<SubTaskId>) -> Self {
        self.subtask_ids = Some(subtask_ids);
        self
    }

    /// Filter by message types
    pub fn message_types(mut self, message_types: Vec<MessageType>) -> Self {
        self.message_types = Some(message_types);
        self
    }

    /// Check if a message passes the filter
    pub fn matches(&self, message: &WorkerMessage) -> bool {
        // Check worker ID filter
        if let Some(ref worker_ids) = self.worker_ids {
            let message_worker_id = match message {
                WorkerMessage::Started { worker_id, .. } => worker_id,
                WorkerMessage::Progress { worker_id, .. } => worker_id,
                WorkerMessage::Blocked { worker_id, .. } => worker_id,
                WorkerMessage::Completed { worker_id, .. } => worker_id,
                WorkerMessage::Failed { worker_id, .. } => worker_id,
            };
            if !worker_ids.contains(message_worker_id) {
                return false;
            }
        }

        // Check subtask ID filter
        if let Some(ref subtask_ids) = self.subtask_ids {
            let message_subtask_id = match message {
                WorkerMessage::Started { subtask_id, .. } => subtask_id,
                WorkerMessage::Progress { subtask_id, .. } => subtask_id,
                WorkerMessage::Blocked { subtask_id, .. } => subtask_id,
                WorkerMessage::Completed { subtask_id, .. } => subtask_id,
                WorkerMessage::Failed { subtask_id, .. } => subtask_id,
            };
            if !subtask_ids.contains(message_subtask_id) {
                return false;
            }
        }

        // Check message type filter
        if let Some(ref message_types) = self.message_types {
            let message_type = MessageType::from(message);
            if !message_types.contains(&message_type) {
                return false;
            }
        }

        true
    }
}

/// Message processor for handling incoming messages
pub struct MessageProcessor {
    filters: Vec<MessageFilter>,
    handlers: Vec<Box<dyn MessageHandler>>,
}

impl MessageProcessor {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            handlers: Vec::new(),
        }
    }

    /// Add a filter and handler pair
    pub fn add_handler<H: MessageHandler + 'static>(
        mut self,
        filter: MessageFilter,
        handler: H,
    ) -> Self {
        self.filters.push(filter);
        self.handlers.push(Box::new(handler));
        self
    }

    /// Process a message by finding matching handlers
    pub async fn process_message(&self, message: &WorkerMessage) -> crate::ParallelResult<()> {
        for (filter, handler) in self.filters.iter().zip(self.handlers.iter()) {
            if filter.matches(message) {
                handler.handle_message(message).await?;
            }
        }
        Ok(())
    }
}

/// Trait for handling worker messages
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, message: &WorkerMessage) -> crate::ParallelResult<()>;
}

/// Helper functions for message processing
pub mod helpers {
    use super::*;

    /// Extract worker ID from any message
    pub fn get_worker_id(message: &WorkerMessage) -> &WorkerId {
        match message {
            WorkerMessage::Started { worker_id, .. } => worker_id,
            WorkerMessage::Progress { worker_id, .. } => worker_id,
            WorkerMessage::Blocked { worker_id, .. } => worker_id,
            WorkerMessage::Completed { worker_id, .. } => worker_id,
            WorkerMessage::Failed { worker_id, .. } => worker_id,
        }
    }

    /// Extract subtask ID from any message
    pub fn get_subtask_id(message: &WorkerMessage) -> &SubTaskId {
        match message {
            WorkerMessage::Started { subtask_id, .. } => subtask_id,
            WorkerMessage::Progress { subtask_id, .. } => subtask_id,
            WorkerMessage::Blocked { subtask_id, .. } => subtask_id,
            WorkerMessage::Completed { subtask_id, .. } => subtask_id,
            WorkerMessage::Failed { subtask_id, .. } => subtask_id,
        }
    }

    /// Check if message indicates completion (success or failure)
    pub fn is_completion_message(message: &WorkerMessage) -> bool {
        matches!(message,
            WorkerMessage::Completed { .. } | WorkerMessage::Failed { .. }
        )
    }

    /// Check if message indicates an error condition
    pub fn is_error_message(message: &WorkerMessage) -> bool {
        matches!(message,
            WorkerMessage::Failed { .. } | WorkerMessage::Blocked { .. }
        )
    }

    /// Get timestamp from message
    pub fn get_timestamp(message: &WorkerMessage) -> DateTime<Utc> {
        match message {
            WorkerMessage::Started { timestamp, .. } => *timestamp,
            WorkerMessage::Progress { timestamp, .. } => *timestamp,
            WorkerMessage::Blocked { timestamp, .. } => *timestamp,
            WorkerMessage::Completed { timestamp, .. } => *timestamp,
            WorkerMessage::Failed { timestamp, .. } => *timestamp,
        }
    }
}
