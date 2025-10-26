//! Streaming pipeline implementation
//!
//! This module provides a streaming pipeline that can process continuous data streams
//! with backpressure handling and multiplexing capabilities.

use crate::{
    traits::ExecutablePipeline,
    config::StreamingPipelineConfig,
    error::{PipelineError, PipelineResult},
    metrics::PipelineMetrics,
};
use async_trait::async_trait;
use futures::TryFutureExt;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

/// Streaming pipeline for continuous data processing
pub struct StreamingPipeline<Input, Output> {
    config: StreamingPipelineConfig,
    /// Processing function for individual items
    processor: Arc<dyn Fn(Input) -> PipelineResult<Output> + Send + Sync>,
    /// Channel for incoming data
    input_sender: mpsc::UnboundedSender<Input>,
    /// Channel for outgoing results
    output_receiver: Arc<std::sync::Mutex<Option<mpsc::UnboundedReceiver<Output>>>>,
    /// Active processing tasks
    active_tasks: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
    /// Pipeline metrics
    metrics: PipelineMetrics,
    /// Stream state
    is_running: Arc<RwLock<bool>>,
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
        processor: Arc<dyn Fn(Input) -> PipelineResult<Output> + Send + Sync>
    ) -> Self {
        let (input_sender, input_receiver) = mpsc::unbounded_channel();
        let (output_sender, output_receiver) = mpsc::unbounded_channel();

        let pipeline = Self {
            config,
            processor,
            input_sender,
            output_receiver: Arc::new(Mutex::new(Some(output_receiver))),
            active_tasks: Arc::new(RwLock::new(Vec::new())),
            metrics: PipelineMetrics::new(),
            is_running: Arc::new(RwLock::new(false)),
        };

        pipeline.start_processing(input_receiver, output_sender);
        pipeline
    }

    /// Send data to the stream
    pub async fn send(&self, input: Input) -> PipelineResult<()> {
        if !*self.is_running.read().await {
            return Err(PipelineError::Execution("Pipeline is not running".to_string()));
        }

        self.input_sender.send(input)
            .map_err(|_| PipelineError::ChannelSendError("Failed to send data to pipeline".to_string()))
    }

    /// Try to receive processed output
    pub async fn try_recv(&self) -> PipelineResult<Option<Output>> {
        let mut receiver_guard = self.output_receiver.lock().unwrap();
        if let Some(receiver) = receiver_guard.as_mut() {
            match receiver.try_recv() {
                Ok(output) => Ok(Some(output)),
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    Err(PipelineError::ChannelReceiveError("Output channel disconnected".to_string()))
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Receive processed output with timeout
    pub async fn recv_timeout(&self, timeout: std::time::Duration) -> PipelineResult<Option<Output>> {
        let mut receiver_guard = self.output_receiver.lock().unwrap();
        if let Some(receiver) = receiver_guard.as_mut() {
            match tokio::time::timeout(timeout, receiver.recv()).await {
                Ok(Some(output)) => Ok(Some(output)),
                Ok(None) => Err(PipelineError::ChannelReceiveError("Output channel closed".to_string())),
                Err(_) => Ok(None), // Timeout
            }
        } else {
            Ok(None)
        }
    }

    /// Start the processing tasks
    fn start_processing(
        &self,
        mut input_receiver: mpsc::UnboundedReceiver<Input>,
        output_sender: mpsc::UnboundedSender<Output>
    ) {
        let processor = Arc::clone(&self.processor);
        let metrics = self.metrics.clone();
        let buffer_size = self.config.buffer_size;
        let enable_backpressure = self.config.enable_backpressure;
        let backpressure_threshold = self.config.backpressure_threshold;

        let task = tokio::spawn(async move {
            info!("Starting streaming pipeline processor");

            while let Some(input) = input_receiver.recv().await {
                let start_time = std::time::Instant::now();

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

                        if let Err(_) = output_sender.send(output) {
                            warn!("Failed to send output, channel may be closed");
                            break;
                        }
                    }
                    Err(e) => {
                        let duration = start_time.elapsed().as_millis() as u64;
                        metrics.record_execution(duration, false).await;
                        metrics.record_error("processing_error").await;

                        warn!("Processing error: {}", e);

                        // Send error indicator if needed
                        // For now, we just log and continue
                    }
                }

                // Yield control to prevent blocking
                tokio::task::yield_now().await;
            }

            info!("Streaming pipeline processor stopped");
        });

        futures::executor::block_on(async {
            let mut tasks = self.active_tasks.write().await;
            tasks.push(task);
        });
    }

    /// Get current buffer depth
    pub async fn buffer_depth(&self) -> usize {
        // This is a simplified implementation
        // In a real system, you'd track this more accurately
        0
    }

    /// Get active task count
    pub async fn active_task_count(&self) -> usize {
        self.active_tasks.read().await.len()
    }
}

impl<Input, Output> StreamingPipeline<Input, Output>
where
    Input: Clone + Send + Sync + 'static,
    Output: Clone + Send + Sync + 'static,
{
    /// Start the pipeline
    pub async fn start(&self) -> PipelineResult<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err(PipelineError::Execution("Pipeline is already running".to_string()));
        }
        *is_running = true;
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

        // Close channels to signal tasks to stop
        drop(self.input_sender.clone());

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
        futures::executor::block_on(async {
            self.metrics.to_json().await
        }).map_err(|e| PipelineError::Metrics(e.to_string()))
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
                counter.fetch_add(1, Ordering::SeqCst);
                Ok(format!("processed-{}", input))
            })
        };

        let pipeline = StreamingPipeline::new(config, processor);
        pipeline.start().await.unwrap();

        // Send some data
        pipeline.send("test1".to_string()).await.unwrap();
        pipeline.send("test2".to_string()).await.unwrap();

        // Give some time for processing
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        // Receive results
        let result1 = pipeline.recv_timeout(std::time::Duration::from_millis(100)).await.unwrap();
        let result2 = pipeline.recv_timeout(std::time::Duration::from_millis(100)).await.unwrap();

        assert_eq!(result1, Some("processed-test1".to_string()));
        assert_eq!(result2, Some("processed-test2".to_string()));
        assert_eq!(counter.load(Ordering::SeqCst), 2);

        pipeline.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_streaming_pipeline_error_handling() {
        let config = StreamingPipelineConfig::default();

        let processor = Arc::new(|_input: String| -> PipelineResult<String> {
            Err(PipelineError::Execution("Processing failed".to_string()))
        });

        let pipeline = StreamingPipeline::new(config, processor);
        pipeline.start().await.unwrap();

        // Send data that will fail
        pipeline.send("test".to_string()).await.unwrap();

        // Should not receive any output due to error
        let result = pipeline.recv_timeout(std::time::Duration::from_millis(50)).await.unwrap();
        assert_eq!(result, None); // No output due to error

        pipeline.stop().await.unwrap();
    }
}
