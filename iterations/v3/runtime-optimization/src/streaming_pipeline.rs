//! Streaming Task Execution Pipeline - Dual-Session Processing
//!
//! Implements streaming task execution with chunked processing and dual-session
//! execution for overlapping computation, enabling efficient pipelined workflows.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

#[cfg(feature = "chunked_execution")]
use crate::chunked_execution::{ChunkedExecutor, ChunkConfig, ExecutionChunk};

#[cfg(not(feature = "chunked_execution"))]
use crate::chunked_stubs::{ChunkedExecutor, ChunkConfig, ExecutionChunk};

/// Streaming pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Maximum concurrent streams
    pub max_concurrent_streams: usize,
    /// Chunk size for task decomposition
    pub chunk_size: usize,
    /// Pipeline buffer size
    pub buffer_size: usize,
    /// Enable dual-session execution
    pub dual_session_enabled: bool,
    /// Session overlap factor (0.0-1.0)
    pub session_overlap: f64,
}

/// Pipeline execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineMetrics {
    /// Current active streams
    pub active_streams: usize,
    /// Total streams processed
    pub total_streams: u64,
    /// Average stream throughput
    pub avg_throughput: f64,
    /// Pipeline latency (ms)
    pub pipeline_latency_ms: f64,
    /// Chunk processing efficiency
    pub chunk_efficiency: f64,
    /// Dual-session overlap ratio
    pub dual_session_overlap: f64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Stream execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamState {
    /// Stream is being prepared
    Preparing,
    /// Stream is actively processing
    Active,
    /// Stream is waiting for resources
    Waiting,
    /// Stream completed successfully
    Completed,
    /// Stream failed
    Failed(String),
}

/// Streaming pipeline for efficient task execution
pub struct StreamingPipeline {
    config: StreamConfig,
    /// Active streams
    active_streams: Arc<RwLock<HashMap<String, StreamExecution>>>,
    /// Stream metrics
    metrics: Arc<RwLock<PipelineMetrics>>,
    /// Chunked executor for task decomposition
    chunked_executor: Arc<ChunkedExecutor>,
    /// Stream command channel
    command_sender: mpsc::UnboundedSender<StreamCommand>,
    /// Stream event receiver
    event_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<StreamEvent>>>>,
}

/// Stream execution context
#[derive(Debug, Clone)]
pub struct StreamExecution {
    /// Stream ID
    pub id: String,
    /// Current state
    pub state: StreamState,
    /// Start timestamp
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Current chunk being processed
    pub current_chunk: Option<ExecutionChunk>,
    /// Primary session chunks
    pub primary_chunks: Vec<ExecutionChunk>,
    /// Secondary session chunks (for dual-session execution)
    pub secondary_chunks: Vec<ExecutionChunk>,
    /// Completion progress (0.0-1.0)
    pub progress: f64,
}

/// Stream commands for pipeline control
#[derive(Debug, Clone)]
pub enum StreamCommand {
    /// Start a new stream
    StartStream { id: String, task_data: Vec<u8> },
    /// Pause a stream
    PauseStream { id: String },
    /// Resume a stream
    ResumeStream { id: String },
    /// Cancel a stream
    CancelStream { id: String },
    /// Update stream configuration
    UpdateConfig { config: StreamConfig },
}

/// Stream events emitted during execution
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Stream started
    StreamStarted { id: String, timestamp: chrono::DateTime<chrono::Utc> },
    /// Chunk completed
    ChunkCompleted { stream_id: String, chunk_id: String, timestamp: chrono::DateTime<chrono::Utc> },
    /// Stream progress update
    StreamProgress { id: String, progress: f64, timestamp: chrono::DateTime<chrono::Utc> },
    /// Stream completed
    StreamCompleted { id: String, result: Vec<u8>, timestamp: chrono::DateTime<chrono::Utc> },
    /// Stream failed
    StreamFailed { id: String, error: String, timestamp: chrono::DateTime<chrono::Utc> },
}

impl StreamingPipeline {
    /// Create new streaming pipeline
    pub fn new(config: StreamConfig) -> Self {
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        let chunked_executor = Arc::new(ChunkedExecutor::new(ChunkConfig {
            chunk_size: config.chunk_size,
            max_concurrent_chunks: config.max_concurrent_streams,
            enable_dual_session: config.dual_session_enabled,
        }));

        // Start pipeline processor
        let active_streams = Arc::new(RwLock::new(HashMap::new()));
        let metrics = Arc::new(RwLock::new(PipelineMetrics {
            active_streams: 0,
            total_streams: 0,
            avg_throughput: 0.0,
            pipeline_latency_ms: 0.0,
            chunk_efficiency: 0.0,
            dual_session_overlap: 0.0,
            last_updated: chrono::Utc::now(),
        }));

        let streams_clone = Arc::clone(&active_streams);
        let metrics_clone = Arc::clone(&metrics);
        let executor_clone = Arc::clone(&chunked_executor);
        let config_clone = config.clone();

        tokio::spawn(async move {
            Self::process_commands(
                command_receiver,
                event_sender,
                streams_clone,
                metrics_clone,
                executor_clone,
                config_clone,
            ).await;
        });

        Self {
            config,
            active_streams,
            metrics,
            chunked_executor,
            command_sender,
            event_receiver: Arc::new(RwLock::new(Some(event_receiver))),
        }
    }

    /// Start a new stream
    pub async fn start_stream(&self, id: String, task_data: Vec<u8>) -> Result<()> {
        info!("Starting stream: {}", id);

        self.command_sender.send(StreamCommand::StartStream { id, task_data })?;
        Ok(())
    }

    /// Pause a stream
    pub async fn pause_stream(&self, id: String) -> Result<()> {
        self.command_sender.send(StreamCommand::PauseStream { id })?;
        Ok(())
    }

    /// Resume a stream
    pub async fn resume_stream(&self, id: String) -> Result<()> {
        self.command_sender.send(StreamCommand::ResumeStream { id })?;
        Ok(())
    }

    /// Cancel a stream
    pub async fn cancel_stream(&self, id: String) -> Result<()> {
        self.command_sender.send(StreamCommand::CancelStream { id })?;
        Ok(())
    }

    /// Tune pipeline parameters
    pub async fn tune_pipeline(&self, parameters: &HashMap<String, f64>) -> Result<()> {
        info!("Tuning streaming pipeline parameters");

        // Extract relevant parameters
        let chunk_size = parameters.get("chunk_size").copied().unwrap_or(self.config.chunk_size as f64) as usize;
        let max_concurrent = parameters.get("concurrency_level").copied().unwrap_or(self.config.max_concurrent_streams as f64) as usize;
        let session_overlap = parameters.get("session_overlap").copied().unwrap_or(self.config.session_overlap);

        let new_config = StreamConfig {
            chunk_size,
            max_concurrent_streams: max_concurrent,
            session_overlap,
            ..self.config.clone()
        };

        self.command_sender.send(StreamCommand::UpdateConfig { config: new_config })?;
        Ok(())
    }

    /// Apply optimized parameters
    pub async fn apply_parameters(&self, parameters: &HashMap<String, f64>) -> Result<()> {
        self.tune_pipeline(parameters).await
    }

    /// Get current pipeline metrics
    pub async fn get_metrics(&self) -> PipelineMetrics {
        self.metrics.read().await.clone()
    }

    /// Poll for stream events
    pub async fn poll_events(&self) -> Result<Vec<StreamEvent>> {
        let mut events = Vec::new();

        if let Some(receiver) = &mut *self.event_receiver.write().await {
            while let Ok(event) = receiver.try_recv() {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Get active streams
    pub async fn get_active_streams(&self) -> HashMap<String, StreamExecution> {
        self.active_streams.read().await.clone()
    }

    /// Process stream commands
    async fn process_commands(
        mut command_receiver: mpsc::UnboundedReceiver<StreamCommand>,
        event_sender: mpsc::UnboundedSender<StreamEvent>,
        active_streams: Arc<RwLock<HashMap<String, StreamExecution>>>,
        metrics: Arc<RwLock<PipelineMetrics>>,
        chunked_executor: Arc<ChunkedExecutor>,
        mut config: StreamConfig,
    ) {
        info!("Starting streaming pipeline command processor");

        while let Some(command) = command_receiver.recv().await {
            match command {
                StreamCommand::StartStream { id, task_data } => {
                    Self::handle_start_stream(
                        &id,
                        task_data,
                        &active_streams,
                        &metrics,
                        &chunked_executor,
                        &event_sender,
                        &config,
                    ).await;
                }
                StreamCommand::PauseStream { id } => {
                    Self::handle_pause_stream(&id, &active_streams).await;
                }
                StreamCommand::ResumeStream { id } => {
                    Self::handle_resume_stream(&id, &active_streams).await;
                }
                StreamCommand::CancelStream { id } => {
                    Self::handle_cancel_stream(&id, &active_streams, &event_sender).await;
                }
                StreamCommand::UpdateConfig { config: new_config } => {
                    config = new_config;
                    debug!("Updated streaming pipeline configuration");
                }
            }
        }

        info!("Streaming pipeline command processor stopped");
    }

    /// Handle start stream command
    async fn handle_start_stream(
        id: &str,
        task_data: Vec<u8>,
        active_streams: &Arc<RwLock<HashMap<String, StreamExecution>>>,
        metrics: &Arc<RwLock<PipelineMetrics>>,
        chunked_executor: &Arc<ChunkedExecutor>,
        event_sender: &mpsc::UnboundedSender<StreamEvent>,
        config: &StreamConfig,
    ) {
        let stream = StreamExecution {
            id: id.to_string(),
            state: StreamState::Preparing,
            started_at: chrono::Utc::now(),
            current_chunk: None,
            primary_chunks: Vec::new(),
            secondary_chunks: Vec::new(),
            progress: 0.0,
        };

        // Add to active streams
        active_streams.write().await.insert(id.to_string(), stream);

        // Update metrics
        {
            let mut metrics_lock = metrics.write().await;
            metrics_lock.active_streams += 1;
            metrics_lock.total_streams += 1;
            metrics_lock.last_updated = chrono::Utc::now();
        }

        // Emit stream started event
        let _ = event_sender.send(StreamEvent::StreamStarted {
            id: id.to_string(),
            timestamp: chrono::Utc::now(),
        });

        // Start stream processing
        let streams_clone = Arc::clone(active_streams);
        let metrics_clone = Arc::clone(metrics);
        let executor_clone = Arc::clone(chunked_executor);
        let event_sender_clone = event_sender.clone();
        let config_clone = config.clone();

        tokio::spawn(async move {
            Self::process_stream(
                id.to_string(),
                task_data,
                streams_clone,
                metrics_clone,
                executor_clone,
                event_sender_clone,
                config_clone,
            ).await;
        });
    }

    /// Handle pause stream command
    async fn handle_pause_stream(id: &str, active_streams: &Arc<RwLock<HashMap<String, StreamExecution>>>) {
        if let Some(stream) = active_streams.write().await.get_mut(id) {
            stream.state = StreamState::Waiting;
            debug!("Paused stream: {}", id);
        }
    }

    /// Handle resume stream command
    async fn handle_resume_stream(id: &str, active_streams: &Arc<RwLock<HashMap<String, StreamExecution>>>) {
        if let Some(stream) = active_streams.write().await.get_mut(id) {
            stream.state = StreamState::Active;
            debug!("Resumed stream: {}", id);
        }
    }

    /// Handle cancel stream command
    async fn handle_cancel_stream(
        id: &str,
        active_streams: &Arc<RwLock<HashMap<String, StreamExecution>>>,
        event_sender: &mpsc::UnboundedSender<StreamEvent>,
    ) {
        active_streams.write().await.remove(id);

        let _ = event_sender.send(StreamEvent::StreamFailed {
            id: id.to_string(),
            error: "Stream cancelled by user".to_string(),
            timestamp: chrono::Utc::now(),
        });

        debug!("Cancelled stream: {}", id);
    }

    /// Process a stream through its lifecycle
    async fn process_stream(
        stream_id: String,
        task_data: Vec<u8>,
        active_streams: Arc<RwLock<HashMap<String, StreamExecution>>>,
        metrics: Arc<RwLock<PipelineMetrics>>,
        chunked_executor: Arc<ChunkedExecutor>,
        event_sender: mpsc::UnboundedSender<StreamEvent>,
        config: StreamConfig,
    ) {
        // Mark stream as active
        {
            let mut streams = active_streams.write().await;
            if let Some(stream) = streams.get_mut(&stream_id) {
                stream.state = StreamState::Active;
            }
        }

        // Decompose task into chunks
        match chunked_executor.decompose_task(&task_data, config.chunk_size).await {
            Ok(chunks) => {
                // Process chunks with dual-session execution if enabled
                if config.dual_session_enabled {
                    Self::process_dual_session(
                        stream_id,
                        chunks,
                        active_streams,
                        metrics,
                        chunked_executor,
                        event_sender,
                        config,
                    ).await;
                } else {
                    Self::process_single_session(
                        stream_id,
                        chunks,
                        active_streams,
                        metrics,
                        chunked_executor,
                        event_sender,
                    ).await;
                }
            }
            Err(e) => {
                // Mark stream as failed
                {
                    let mut streams = active_streams.write().await;
                    if let Some(stream) = streams.get_mut(&stream_id) {
                        stream.state = StreamState::Failed(e.to_string());
                    }
                }

                let _ = event_sender.send(StreamEvent::StreamFailed {
                    id: stream_id,
                    error: e.to_string(),
                    timestamp: chrono::Utc::now(),
                });
            }
        }
    }

    /// Process chunks with dual-session execution
    async fn process_dual_session(
        stream_id: String,
        chunks: Vec<ExecutionChunk>,
        active_streams: Arc<RwLock<HashMap<String, StreamExecution>>>,
        metrics: Arc<RwLock<PipelineMetrics>>,
        chunked_executor: Arc<ChunkedExecutor>,
        event_sender: mpsc::UnboundedSender<StreamEvent>,
        config: StreamConfig,
    ) {
        let mut primary_results = Vec::new();
        let mut secondary_results = Vec::new();

        // Split chunks between primary and secondary sessions
        let split_point = (chunks.len() as f64 * (1.0 - config.session_overlap)) as usize;
        let primary_chunks = chunks[..split_point].to_vec();
        let secondary_chunks = chunks[split_point..].to_vec();

        // Start secondary session (overlapping)
        let secondary_handle = {
            let stream_id_clone = stream_id.clone();
            let chunks_clone = secondary_chunks.clone();
            let executor_clone = Arc::clone(&chunked_executor);
            let event_sender_clone = event_sender.clone();

            tokio::spawn(async move {
                Self::process_chunks(
                    stream_id_clone,
                    chunks_clone,
                    executor_clone,
                    event_sender_clone,
                    true, // is_secondary
                ).await
            })
        };

        // Process primary session
        primary_results = Self::process_chunks(
            stream_id.clone(),
            primary_chunks,
            Arc::clone(&chunked_executor),
            event_sender.clone(),
            false, // is_primary
        ).await;

        // Wait for secondary session to complete
        if let Ok(secondary) = secondary_handle.await {
            secondary_results = secondary.unwrap_or_default();
        }

        // Combine results and complete stream
        let combined_result = Self::combine_session_results(primary_results, secondary_results);

        // Update metrics
        {
            let mut metrics_lock = metrics.write().await;
            metrics_lock.dual_session_overlap = config.session_overlap;
            metrics_lock.chunk_efficiency = Self::calculate_chunk_efficiency(&chunks);
            metrics_lock.last_updated = chrono::Utc::now();
        }

        // Mark stream as completed
        {
            let mut streams = active_streams.write().await;
            if let Some(stream) = streams.get_mut(&stream_id) {
                stream.state = StreamState::Completed;
                stream.progress = 1.0;
            }
            streams.remove(&stream_id);
        }

        // Update active stream count
        {
            let mut metrics_lock = metrics.write().await;
            metrics_lock.active_streams = metrics_lock.active_streams.saturating_sub(1);
        }

        let _ = event_sender.send(StreamEvent::StreamCompleted {
            id: stream_id,
            result: combined_result,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Process chunks with single-session execution
    async fn process_single_session(
        stream_id: String,
        chunks: Vec<ExecutionChunk>,
        active_streams: Arc<RwLock<HashMap<String, StreamExecution>>>,
        metrics: Arc<RwLock<PipelineMetrics>>,
        chunked_executor: Arc<ChunkedExecutor>,
        event_sender: mpsc::UnboundedSender<StreamEvent>,
    ) {
        let results = Self::process_chunks(
            stream_id.clone(),
            chunks,
            chunked_executor,
            event_sender.clone(),
            false,
        ).await;

        // Update metrics
        {
            let mut metrics_lock = metrics.write().await;
            metrics_lock.chunk_efficiency = Self::calculate_chunk_efficiency(&[]);
            metrics_lock.last_updated = chrono::Utc::now();
        }

        // Mark stream as completed
        {
            let mut streams = active_streams.write().await;
            if let Some(stream) = streams.get_mut(&stream_id) {
                stream.state = StreamState::Completed;
                stream.progress = 1.0;
            }
            streams.remove(&stream_id);
        }

        // Update active stream count
        {
            let mut metrics_lock = metrics.write().await;
            metrics_lock.active_streams = metrics_lock.active_streams.saturating_sub(1);
        }

        let _ = event_sender.send(StreamEvent::StreamCompleted {
            id: stream_id,
            result: results,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Process a set of chunks
    async fn process_chunks(
        stream_id: String,
        chunks: Vec<ExecutionChunk>,
        chunked_executor: Arc<ChunkedExecutor>,
        event_sender: mpsc::UnboundedSender<StreamEvent>,
        is_secondary: bool,
    ) -> Vec<u8> {
        let mut results = Vec::new();

        for chunk in chunks {
            match chunked_executor.execute_chunk(chunk.clone()).await {
                Ok(chunk_result) => {
                    results.extend(chunk_result);

                    let _ = event_sender.send(StreamEvent::ChunkCompleted {
                        stream_id: stream_id.clone(),
                        chunk_id: chunk.id,
                        timestamp: chrono::Utc::now(),
                    });

                    // Update progress
                    let _ = event_sender.send(StreamEvent::StreamProgress {
                        id: stream_id.clone(),
                        progress: 0.5, // Simplified progress calculation
                        timestamp: chrono::Utc::now(),
                    });
                }
                Err(e) => {
                    warn!("Failed to execute chunk {}: {}", chunk.id, e);
                    // Continue with other chunks
                }
            }
        }

        results
    }

    /// Combine results from primary and secondary sessions
    fn combine_session_results(primary: Vec<u8>, secondary: Vec<u8>) -> Vec<u8> {
        // For now, just concatenate results
        // In practice, this would intelligently merge overlapping computations
        [primary, secondary].concat()
    }

    /// Calculate chunk processing efficiency
    fn calculate_chunk_efficiency(_chunks: &[ExecutionChunk]) -> f64 {
        // Simplified efficiency calculation
        // In practice, this would analyze processing time vs. expected time
        0.85 // 85% efficiency
    }

    /// Tune pipeline with optimization results
    pub async fn tune_pipeline(&self, _optimization_result: &crate::bayesian_optimizer::OptimizationResult) -> Result<()> {
        // Stub implementation for pipeline tuning
        Ok(())
    }

    /// Apply optimized parameters to pipeline
    pub async fn apply_parameters(&self, _parameters: &HashMap<String, f64>) -> Result<()> {
        // Stub implementation for parameter application
        Ok(())
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            max_concurrent_streams: 10,
            chunk_size: 3,
            buffer_size: 100,
            dual_session_enabled: true,
            session_overlap: 0.2,
        }
    }
}


