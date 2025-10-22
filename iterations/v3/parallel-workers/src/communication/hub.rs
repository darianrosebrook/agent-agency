//! Communication hub that orchestrates message passing

use crate::types::*;
use crate::error::*;
use crate::communication::{MessageBroker, ChannelRegistry, ChannelConfig};
use super::channels::BidirectionalChannel;

/// Central communication hub for the parallel worker system
pub struct CommunicationHub {
    message_broker: MessageBroker,
    channel_registry: std::sync::Arc<ChannelRegistry>,
    config: ChannelConfig,
}

impl CommunicationHub {
    /// Create a new communication hub
    pub fn new(config: ChannelConfig) -> Self {
        Self {
            message_broker: MessageBroker::new(config.buffer_size),
            channel_registry: std::sync::Arc::new(ChannelRegistry::new(config.clone())),
            config,
        }
    }

    /// Create a communication channel for a worker
    pub fn create_worker_channel(&self, worker_id: WorkerId) -> BidirectionalChannel {
        let (channel, _receiver) = self.channel_registry.create_channel(worker_id);
        channel
    }

    /// Get the message broker sender for workers to send messages
    pub fn message_sender(&self) -> tokio::sync::mpsc::UnboundedSender<WorkerMessage> {
        self.message_broker.sender()
    }

    /// Process incoming messages from workers
    pub async fn process_messages(&mut self) -> CommunicationResult<Vec<WorkerMessage>> {
        let mut messages = Vec::new();

        // Process all available messages
        while let Ok(message) = self.message_broker.try_recv() {
            messages.push(message);
        }

        Ok(messages)
    }

    /// Send a message to a specific worker
    pub async fn send_to_worker(&self, worker_id: &WorkerId, message: WorkerMessage) -> CommunicationResult<()> {
        self.channel_registry.send_to_worker(worker_id, message).await
    }

    /// Broadcast a message to all active workers
    pub async fn broadcast_to_workers(&self, message: WorkerMessage) -> CommunicationResult<()> {
        self.channel_registry.broadcast(message).await
    }

    /// Check if a worker has an active communication channel
    pub fn has_worker_channel(&self, worker_id: &WorkerId) -> bool {
        self.channel_registry.has_channel(worker_id)
    }

    /// Remove a worker's communication channel (when worker completes/fails)
    pub fn remove_worker_channel(&self, worker_id: &WorkerId) -> Option<BidirectionalChannel> {
        self.channel_registry.remove_channel(worker_id)
    }

    /// Get communication statistics
    pub fn get_stats(&self) -> CommunicationStats {
        let channel_stats = self.channel_registry.get_stats();

        CommunicationStats {
            active_workers: channel_stats.total_channels,
            healthy_channels: channel_stats.healthy_channels,
            pending_messages: self.message_broker.pending_count(),
            buffer_size: self.config.buffer_size,
        }
    }

    /// Clean up channels for completed workers
    pub fn cleanup_completed_workers(&self, completed_workers: &[WorkerId]) {
        self.channel_registry.cleanup_completed_workers(completed_workers);
    }

    /// Get all active worker IDs
    pub fn active_workers(&self) -> Vec<WorkerId> {
        self.channel_registry.active_workers()
    }

    /// Start background monitoring of communication health
    pub fn start_monitoring(&self) -> tokio::task::JoinHandle<()> {
        use super::channels::ChannelMonitor;
        let monitor = ChannelMonitor::new(
            self.channel_registry.clone(),
            std::time::Duration::from_secs(30), // Check every 30 seconds
        );
        monitor.start_monitoring()
    }
}

impl Default for CommunicationHub {
    fn default() -> Self {
        Self::new(ChannelConfig::default())
    }
}

/// Statistics for communication monitoring
#[derive(Debug, Clone)]
pub struct CommunicationStats {
    pub active_workers: usize,
    pub healthy_channels: usize,
    pub pending_messages: usize,
    pub buffer_size: usize,
}

/// Message router for directing messages to appropriate handlers
pub struct MessageRouter {
    routes: std::collections::HashMap<MessageType, Vec<Box<dyn MessageHandler>>>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            routes: std::collections::HashMap::new(),
        }
    }

    /// Register a handler for a specific message type
    pub fn register_handler<H: MessageHandler + 'static>(
        &mut self,
        message_type: MessageType,
        handler: H,
    ) {
        self.routes.entry(message_type).or_insert_with(Vec::new).push(Box::new(handler));
    }

    /// Route a message to all registered handlers for its type
    pub async fn route_message(&self, message: &WorkerMessage) -> CommunicationResult<()> {
        let message_type = MessageType::from(message);

        if let Some(handlers) = self.routes.get(&message_type) {
            for handler in handlers {
                handler.handle_message(message).await?;
            }
        }

        Ok(())
    }
}

/// Message type for routing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// Trait for handling messages
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle_message(&self, message: &WorkerMessage) -> CommunicationResult<()>;
}

/// Example message handler for progress tracking
pub struct ProgressMessageHandler {
    progress_callback: Box<dyn Fn(&WorkerMessage) + Send + Sync>,
}

impl ProgressMessageHandler {
    pub fn new<F>(callback: F) -> Self
    where
        F: Fn(&WorkerMessage) + Send + Sync + 'static,
    {
        Self {
            progress_callback: Box::new(callback),
        }
    }
}

#[async_trait::async_trait]
impl MessageHandler for ProgressMessageHandler {
    async fn handle_message(&self, message: &WorkerMessage) -> CommunicationResult<()> {
        (self.progress_callback)(message);
        Ok(())
    }
}

/// Message filter for selective processing
pub struct MessageFilter {
    pub worker_ids: Option<Vec<WorkerId>>,
    pub subtask_ids: Option<Vec<SubTaskId>>,
    pub message_types: Option<Vec<MessageType>>,
}

impl MessageFilter {
    pub fn new() -> Self {
        Self {
            worker_ids: None,
            subtask_ids: None,
            message_types: None,
        }
    }

    pub fn workers(mut self, worker_ids: Vec<WorkerId>) -> Self {
        self.worker_ids = Some(worker_ids);
        self
    }

    pub fn subtasks(mut self, subtask_ids: Vec<SubTaskId>) -> Self {
        self.subtask_ids = Some(subtask_ids);
        self
    }

    pub fn message_types(mut self, message_types: Vec<MessageType>) -> Self {
        self.message_types = Some(message_types);
        self
    }

    pub fn matches(&self, message: &WorkerMessage) -> bool {
        // Check worker ID filter
        if let Some(ref worker_ids) = self.worker_ids {
            let message_worker_id = super::messages::helpers::get_worker_id(message);
            if !worker_ids.contains(message_worker_id) {
                return false;
            }
        }

        // Check subtask ID filter
        if let Some(ref subtask_ids) = self.subtask_ids {
            let message_subtask_id = super::messages::helpers::get_subtask_id(message);
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
