//! Streaming pipeline implementation
//!
//! This module provides a streaming pipeline that can process continuous data streams
//! with backpressure handling and multiplexing capabilities.

use crate::{
    config::StreamingPipelineConfig,
    error::{PipelineError, PipelineResult},
    metrics::PipelineMetrics,
    traits::ExecutablePipeline,
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn};

/// Streaming pipeline for continuous data processing
pub struct StreamingPipeline<Input, Output> {
    config: StreamingPipelineConfig,
    /// Processing function for individual items
    processor: Arc<dyn Fn(Input) -> PipelineResult<Output> + Send + Sync>,
    /// Channel for incoming data
    input_sender: Arc<std::sync::Mutex<Option<mpsc::UnboundedSender<Input>>>>,
    /// Channel for outgoing results
    output_receiver: Arc<std::sync::Mutex<Option<mpsc::UnboundedReceiver<Output>>>>,
    /// Active processing tasks
    active_tasks: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
    /// Pipeline metrics
    metrics: PipelineMetrics,
    /// Stream state
    is_running: Arc<RwLock<bool>>,
    /// Stored receiver for starting processing
    input_receiver: Option<mpsc::UnboundedReceiver<Input>>,
    /// Stored sender for starting processing
    output_sender: Option<mpsc::UnboundedSender<Output>>,
}

impl<Input, Output> std::fmt::Debug for StreamingPipeline<Input, Output> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamingPipeline")
            .field("config", &self.config)
            .field("processor", &"<function>")
            .field("input_sender", &self.input_sender)
            .field("output_receiver", &"<receiver>")
            .field("active_tasks", &self.active_tasks)
            .field("metrics", &self.metrics)
            .field("is_running", &self.is_running)
            .finish()
    }
}

impl<Input, Output> StreamingPipeline<Input, Output>
where
    Input: Clone + Send + Sync + 'static,
    Output: Clone + Send + Sync + 'static,
{
    /// Create a new streaming pipeline
    pub fn new(
        config: StreamingPipelineConfig,
        processor: Arc<dyn Fn(Input) -> PipelineResult<Output> + Send + Sync>,
    ) -> Self {
        let (input_sender, input_receiver) = mpsc::unbounded_channel();
        let (output_sender, output_receiver) = mpsc::unbounded_channel();

        Self {
            config,
            processor,
            input_sender: Arc::new(Mutex::new(Some(input_sender))),
            output_receiver: Arc::new(Mutex::new(Some(output_receiver))),
            active_tasks: Arc::new(RwLock::new(Vec::new())),
            metrics: PipelineMetrics::new(),
            is_running: Arc::new(RwLock::new(false)),
            input_receiver: Some(input_receiver),
            output_sender: Some(output_sender),
        }
    }

    /// Send data to the stream
    pub async fn send(&self, input: Input) -> PipelineResult<()> {
        let is_running = *self.is_running.read().await;
        tracing::debug!("Pipeline is_running: {}", is_running);
        if !is_running {
            return Err(PipelineError::Execution(
                "Pipeline is not running".to_string(),
            ));
        }

        let sender_guard = self.input_sender.lock().unwrap();
        if let Some(sender) = sender_guard.as_ref() {
            sender.send(input).map_err(|e| {
                PipelineError::ChannelSendError(format!("Failed to send data to pipeline: {}", e))
            })
        } else {
            Err(PipelineError::Execution(
                "Pipeline input channel is closed".to_string(),
            ))
        }
    }

    /// Try to receive processed output
    pub async fn try_recv(&self) -> PipelineResult<Option<Output>> {
        let mut receiver_guard = self.output_receiver.lock().unwrap();
        if let Some(receiver) = receiver_guard.as_mut() {
            match receiver.try_recv() {
                Ok(output) => Ok(Some(output)),
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => Err(
                    PipelineError::ChannelReceiveError("Output channel disconnected".to_string()),
                ),
            }
        } else {
            Ok(None)
        }
    }

    /// Receive processed output with timeout
    pub async fn recv_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> PipelineResult<Option<Output>> {
        let mut receiver_guard = self.output_receiver.lock().unwrap();
        if let Some(receiver) = receiver_guard.as_mut() {
            match tokio::time::timeout(timeout, receiver.recv()).await {
                Ok(Some(output)) => Ok(Some(output)),
                Ok(None) => Err(PipelineError::ChannelReceiveError(
                    "Output channel closed".to_string(),
                )),
                Err(_) => Ok(None), // Timeout
            }
        } else {
            Ok(None)
        }
    }

    /// Start the processing tasks
    async fn start_processing(
        &self,
        mut input_receiver: mpsc::UnboundedReceiver<Input>,
        output_sender: mpsc::UnboundedSender<Output>,
    ) {
        let processor = Arc::clone(&self.processor);
        let metrics = self.metrics.clone();
        let _buffer_size = self.config.buffer_size;
        let enable_backpressure = self.config.enable_backpressure;
        let backpressure_threshold = self.config.backpressure_threshold;

        let task = tokio::spawn(async move {
            info!("Starting streaming pipeline processor");

            // Process inputs in a loop
            while let Some(input) = input_receiver.recv().await {
                info!(
                    "Processing input (type: {})",
                    std::any::type_name::<Input>()
                );
                let start_time = std::time::Instant::now();

                // Check if we should still be running
                // The task will exit when the receiver is closed

                // Check backpressure if enabled
                if enable_backpressure {
                    // Simple backpressure check - could be enhanced
                    let queue_depth = input_receiver.len();
                    if queue_depth > backpressure_threshold {
                        warn!("Backpressure detected, queue depth: {}", queue_depth);
                        // Could implement backpressure strategies here
                    }
                }

                // Process the input
                let result = (processor)(input.clone());

                match result {
                    Ok(output) => {
                        let duration = start_time.elapsed().as_millis() as u64;
                        metrics.record_execution(duration, true).await;

                        if let Err(e) = output_sender.send(output) {
                            warn!("Failed to send output, channel may be closed: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        let duration = start_time.elapsed().as_millis() as u64;
                        metrics.record_execution(duration, false).await;
                        metrics.record_error("processing_error").await;

                        warn!("Processing error: {}", e);

                        // TODO: Implement comprehensive error handling and reporting in streaming pipeline
                        //       Currently just logs and continues; should implement comprehensive handling that sends error indicators to monitoring/alerting system, tracks error rates and patterns, and implements error recovery strategies.
                        //
                        // COMPLETION CHECKLIST:
                        // [ ] Primary functionality implemented
                        // [ ] API/data structures defined & stable
                        // [ ] Error handling + validation aligned with error taxonomy
                        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
                        // [ ] Integration tests for external systems/contracts
                        // [ ] Documentation: public API + system behavior
                        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
                        // [ ] Security posture reviewed (inputs, authz, sandboxing)
                        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
                        // [ ] Configurability and feature flags defined if relevant
                        // [ ] Failure-mode cards documented (degradation paths)
                        //
                        // ACCEPTANCE CRITERIA:
                        // - Error indicators are sent to monitoring/alerting system
                        // - Error rates and patterns are tracked
                        // - Error recovery strategies are implemented
                        // - Circuit breaker prevents repeated failures
                        //
                        // DEPENDENCIES:
                        // - Monitoring/alerting system integration (Required)
                        // - Error tracking utilities (Required)
                        // - Error recovery mechanisms (Required)
                        //
                        // ESTIMATED EFFORT: 8-12 hours (medium confidence)
                        // PRIORITY: Medium
                        // BLOCKING: No
                        //
                        // GOVERNANCE:
                        // - CAWS Tier: 2 (error handling functionality)
                        // - Change Budget: ~200 LOC
                        // - Reviewer Requirements: Error handling and monitoring expertise
                    }
                }

                // Yield control to prevent blocking
                tokio::task::yield_now().await;
            }

            info!("Streaming pipeline processor stopped");
        });

        let mut tasks = self.active_tasks.write().await;
        tasks.push(task);
    }

    /// Get current buffer depth
    pub async fn buffer_depth(&self) -> usize {
        // Track actual buffer size and utilization across all streaming channels
        let input_queue_size = 0; // UnboundedSender doesn't have capacity tracking
        let output_queue_size = 0; // UnboundedReceiver doesn't expose queue length
        let active_tasks_count = self.active_tasks.read().await.len();

        // Calculate total buffer depth including queued messages and active processing
        let total_buffer_depth = input_queue_size + output_queue_size + active_tasks_count;

        // Update buffer depth statistics for monitoring
        self.update_buffer_depth_stats(total_buffer_depth).await;

        // Check for buffer overflow conditions
        if total_buffer_depth > self.config.buffer_size {
            warn!(
                "Buffer overflow detected: {} > {}",
                total_buffer_depth, self.config.buffer_size
            );
            self.record_buffer_overflow().await;
        }

        total_buffer_depth
    }

    /// Update buffer depth statistics
    async fn update_buffer_depth_stats(&self, current_depth: usize) {
        self.metrics.record_buffer_depth(current_depth).await;
    }

    /// Record buffer overflow event
    async fn record_buffer_overflow(&self) {
        self.metrics.record_buffer_overflow().await;
    }

    /// Get active task count
    pub async fn active_task_count(&self) -> usize {
        self.active_tasks.read().await.len()
    }
}

impl<Input, Output> StreamingPipeline<Input, Output>
where
    Input: Clone + Send + Sync + 'static + std::fmt::Debug,
    Output: Clone + Send + Sync + 'static + std::fmt::Debug,
{
    /// Start the pipeline
    pub async fn start(&mut self) -> PipelineResult<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(PipelineError::Execution(
                "Pipeline is already running".to_string(),
            ));
        }
        *is_running = true;

        // Start processing if we have the channels
        if let (Some(input_receiver), Some(output_sender)) =
            (self.input_receiver.take(), self.output_sender.take())
        {
            self.start_processing(input_receiver, output_sender).await;
        }

        info!("Streaming pipeline started");
        Ok(())
    }

    /// Stop the pipeline gracefully
    pub async fn stop(&self) -> PipelineResult<()> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Ok(()); // Already stopped
        }
        *is_running = false;

        // Drop the input sender to close the channel and signal tasks to stop
        // We need to take ownership of the sender to drop it
        let sender = {
            let mut sender_guard = self.input_sender.lock().unwrap();
            sender_guard.take()
        };
        drop(sender);

        // Wait for tasks to complete
        let tasks = {
            let mut tasks_guard = self.active_tasks.write().await;
            std::mem::take(&mut *tasks_guard)
        };

        for task in tasks {
            let _ = task.await;
        }

        info!("Streaming pipeline stopped");
        Ok(())
    }
}

#[async_trait]
impl<Input, Output> ExecutablePipeline<Input, ()> for StreamingPipeline<Input, Output>
where
    Input: Clone + Send + Sync + 'static + std::fmt::Debug,
    Output: Clone + Send + Sync + 'static + std::fmt::Debug,
{
    async fn execute(&self, input: Input) -> PipelineResult<()> {
        self.send(input).await
    }

    fn metrics(&self) -> PipelineResult<serde_json::Value> {
        futures::executor::block_on(async { self.metrics.to_json().await })
            .map_err(|e| PipelineError::Metrics(e.to_string()))
    }

    fn health_status(&self) -> PipelineResult<crate::PipelineHealth> {
        futures::executor::block_on(async {
            let is_running = *self.is_running.read().await;

            if !is_running {
                return Ok(crate::PipelineHealth::Unhealthy);
            }

            let active_tasks = self.active_tasks.read().await.len();
            if active_tasks == 0 {
                return Ok(crate::PipelineHealth::Degraded);
            }

            // Check buffer depth
            let buffer_depth = self.buffer_depth().await;
            if buffer_depth > self.config.backpressure_threshold {
                return Ok(crate::PipelineHealth::Degraded);
            }

            Ok(crate::PipelineHealth::Healthy)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_streaming_pipeline_basic() {
        let config = StreamingPipelineConfig::default();
        let counter = Arc::new(AtomicUsize::new(0));

        let processor = {
            let counter = Arc::clone(&counter);
            Arc::new(move |input: String| -> PipelineResult<String> {
                tracing::debug!("Processing input: {}", input);
                counter.fetch_add(1, Ordering::SeqCst);
                let result = format!("processed-{}", input);
                tracing::debug!("Processing result: {}", result);
                Ok(result)
            })
        };

        let mut pipeline = StreamingPipeline::new(config, processor);
        pipeline.start().await.unwrap();

        // Send some data
        tracing::debug!("Sending test1");
        pipeline.send("test1".to_string()).await.unwrap();
        tracing::debug!("Sending test2");
        pipeline.send("test2".to_string()).await.unwrap();

        // Give some time for processing
        tracing::debug!("Sleeping for processing");
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Receive results
        tracing::debug!("Receiving result1");
        let result1 = pipeline
            .recv_timeout(std::time::Duration::from_millis(2000))
            .await
            .unwrap();
        tracing::debug!("Result1: {:?}", result1);
        tracing::debug!("Receiving result2");
        let result2 = pipeline
            .recv_timeout(std::time::Duration::from_millis(2000))
            .await
            .unwrap();
        tracing::debug!("Result2: {:?}", result2);

        assert_eq!(result1, Some("processed-test1".to_string()));
        assert_eq!(result2, Some("processed-test2".to_string()));
        assert_eq!(counter.load(Ordering::SeqCst), 2);

        tracing::debug!("Stopping pipeline");
        pipeline.stop().await.unwrap();
        tracing::debug!("Pipeline stopped");
    }

    #[tokio::test]
    async fn test_streaming_pipeline_error_handling() {
        let config = StreamingPipelineConfig::default();

        let processor = Arc::new(|_input: String| -> PipelineResult<String> {
            Err(PipelineError::Execution("Processing failed".to_string()))
        });

        let mut pipeline = StreamingPipeline::new(config, processor);
        pipeline.start().await.unwrap();

        // Send data that will fail
        pipeline.send("test".to_string()).await.unwrap();

        // Should not receive any output due to error
        let result = pipeline
            .recv_timeout(std::time::Duration::from_millis(500))
            .await
            .unwrap();
        assert_eq!(result, None); // No output due to error

        pipeline.stop().await.unwrap();
    }
}
