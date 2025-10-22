//! Channel-based communication infrastructure

use crate::types::*;
use crate::error::*;

/// Channel configuration for communication
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub buffer_size: usize,
    pub timeout_ms: u64,
    pub retry_attempts: u32,
    pub retry_delay_ms: u64,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            timeout_ms: 5000,
            retry_attempts: 3,
            retry_delay_ms: 100,
        }
    }
}

/// Reliable channel sender with retry logic
#[derive(Clone)]
pub struct ReliableSender {
    sender: tokio::sync::mpsc::UnboundedSender<WorkerMessage>,
    config: ChannelConfig,
}

impl ReliableSender {
    pub fn new(sender: tokio::sync::mpsc::UnboundedSender<WorkerMessage>, config: ChannelConfig) -> Self {
        Self { sender, config }
    }

    /// Send a message with retry logic
    pub async fn send_with_retry(&self, message: WorkerMessage) -> CommunicationResult<()> {
        let mut attempts = 0;
        loop {
            match self.sender.send(message.clone()) {
                Ok(()) => return Ok(()),
                Err(_) if attempts < self.config.retry_attempts => {
                    attempts += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(self.config.retry_delay_ms)).await;
                    continue;
                }
                Err(_) => {
                    return Err(CommunicationError::ChannelSendFailed {
                        message: format!("Failed to send message after {} attempts", attempts + 1),
                    });
                }
            }
        }
    }

    /// Send a message without retry (fire and forget)
    pub fn send(&self, message: WorkerMessage) -> CommunicationResult<()> {
        self.sender.send(message).map_err(|_| CommunicationError::ChannelSendFailed {
            message: "Channel send failed".to_string(),
        })
    }

    /// Check if the channel is closed
    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}

/// Channel receiver with timeout support
pub struct TimeoutReceiver {
    receiver: tokio::sync::mpsc::UnboundedReceiver<WorkerMessage>,
    config: ChannelConfig,
}

impl TimeoutReceiver {
    pub fn new(receiver: tokio::sync::mpsc::UnboundedReceiver<WorkerMessage>, config: ChannelConfig) -> Self {
        Self { receiver, config }
    }

    /// Receive a message with timeout
    pub async fn recv_with_timeout(&mut self) -> CommunicationResult<Option<WorkerMessage>> {
        match tokio::time::timeout(
            std::time::Duration::from_millis(self.config.timeout_ms),
            self.receiver.recv(),
        ).await {
            Ok(message) => Ok(message),
            Err(_) => Err(CommunicationError::CommunicationTimeout {
                timeout_secs: self.config.timeout_ms / 1000,
            }),
        }
    }

    /// Try to receive a message without blocking
    pub fn try_recv(&mut self) -> CommunicationResult<Option<WorkerMessage>> {
        match self.receiver.try_recv() {
            Ok(message) => Ok(Some(message)),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                Err(CommunicationError::ChannelReceiveFailed {
                    message: "Channel disconnected".to_string(),
                })
            }
        }
    }

    /// Check if there are pending messages
    pub fn has_pending(&self) -> bool {
        !self.receiver.is_empty()
    }

    /// Get the number of pending messages
    pub fn pending_count(&self) -> usize {
        self.receiver.len()
    }
}

/// Bidirectional communication channel
pub struct BidirectionalChannel {
    sender: ReliableSender,
    receiver: TimeoutReceiver,
}

impl BidirectionalChannel {
    pub fn new(config: ChannelConfig) -> Self {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let sender = ReliableSender::new(tx, config.clone());
        let receiver = TimeoutReceiver::new(rx, config);

        Self { sender, receiver }
    }

    pub fn split(self) -> (ReliableSender, TimeoutReceiver) {
        (self.sender, self.receiver)
    }

    /// Send a message
    pub async fn send(&self, message: WorkerMessage) -> CommunicationResult<()> {
        self.sender.send_with_retry(message).await
    }

    /// Receive a message with timeout
    pub async fn recv(&mut self) -> CommunicationResult<Option<WorkerMessage>> {
        self.receiver.recv_with_timeout().await
    }

    /// Try to receive without blocking
    pub fn try_recv(&mut self) -> CommunicationResult<Option<WorkerMessage>> {
        self.receiver.try_recv()
    }

    /// Check channel health
    pub fn is_healthy(&self) -> bool {
        !self.sender.is_closed() && self.receiver.has_pending()
    }
}

/// Channel registry for managing multiple channels
pub struct ChannelRegistry {
    channels: dashmap::DashMap<WorkerId, BidirectionalChannel>,
    config: ChannelConfig,
}

impl ChannelRegistry {
    pub fn new(config: ChannelConfig) -> Self {
        Self {
            channels: dashmap::DashMap::new(),
            config,
        }
    }

    /// Create a new channel for a worker
    pub fn create_channel(&self, worker_id: WorkerId) -> BidirectionalChannel {
        let channel = BidirectionalChannel::new(self.config.clone());
        self.channels.insert(worker_id.clone(), channel);
        self.channels.get(&worker_id).unwrap().clone()
    }

    /// Get a channel for sending messages to a worker
    pub fn get_sender(&self, worker_id: &WorkerId) -> Option<ReliableSender> {
        self.channels.get(worker_id).map(|channel| channel.sender.clone())
    }

    /// Send a message to a specific worker
    pub async fn send_to_worker(&self, worker_id: &WorkerId, message: WorkerMessage) -> CommunicationResult<()> {
        if let Some(channel) = self.channels.get(worker_id) {
            channel.sender.send_with_retry(message).await
        } else {
            Err(CommunicationError::ChannelSendFailed {
                message: format!("No channel found for worker {}", worker_id.0),
            })
        }
    }

    /// Broadcast a message to all workers
    pub async fn broadcast(&self, message: WorkerMessage) -> CommunicationResult<()> {
        let mut errors = Vec::new();

        for entry in self.channels.iter() {
            if let Err(e) = entry.value().sender.send(message.clone()) {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(CommunicationError::ChannelSendFailed {
                message: format!("Broadcast failed for {} workers", errors.len()),
            })
        }
    }

    /// Remove a worker's channel
    pub fn remove_channel(&self, worker_id: &WorkerId) -> Option<BidirectionalChannel> {
        self.channels.remove(worker_id).map(|(_, channel)| channel)
    }

    /// Get all active worker IDs
    pub fn active_workers(&self) -> Vec<WorkerId> {
        self.channels.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Check if a worker has an active channel
    pub fn has_channel(&self, worker_id: &WorkerId) -> bool {
        self.channels.contains_key(worker_id)
    }

    /// Clean up channels for completed workers
    pub fn cleanup_completed_workers(&self, completed_workers: &[WorkerId]) {
        for worker_id in completed_workers {
            self.channels.remove(worker_id);
        }
    }

    /// Get channel statistics
    pub fn get_stats(&self) -> ChannelStats {
        let mut total_pending = 0;
        let mut healthy_channels = 0;

        for channel in self.channels.iter() {
            if channel.is_healthy() {
                healthy_channels += 1;
            }
            // Note: We can't easily get pending count from the current design
            // This would require storing receivers in the registry as well
        }

        ChannelStats {
            total_channels: self.channels.len(),
            healthy_channels,
            pending_messages: total_pending,
        }
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new(ChannelConfig::default())
    }
}

/// Statistics for channel health monitoring
#[derive(Debug, Clone)]
pub struct ChannelStats {
    pub total_channels: usize,
    pub healthy_channels: usize,
    pub pending_messages: usize,
}

/// Channel monitoring for health checks
pub struct ChannelMonitor {
    registry: std::sync::Arc<ChannelRegistry>,
    health_check_interval: std::time::Duration,
}

impl ChannelMonitor {
    pub fn new(registry: std::sync::Arc<ChannelRegistry>, health_check_interval: std::time::Duration) -> Self {
        Self {
            registry,
            health_check_interval,
        }
    }

    /// Start monitoring channels in the background
    pub fn start_monitoring(self) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(self.health_check_interval);
            loop {
                interval.tick().await;
                self.check_channel_health().await;
            }
        })
    }

    /// Perform a health check on all channels
    async fn check_channel_health(&self) {
        let stats = self.registry.get_stats();

        tracing::debug!(
            "Channel health check: {}/{} healthy channels, {} pending messages",
            stats.healthy_channels,
            stats.total_channels,
            stats.pending_messages
        );

        // Log warnings for unhealthy channels
        if stats.healthy_channels < stats.total_channels {
            tracing::warn!(
                "Unhealthy channels detected: {}/{} channels are unhealthy",
                stats.total_channels - stats.healthy_channels,
                stats.total_channels
            );
        }
    }
}
