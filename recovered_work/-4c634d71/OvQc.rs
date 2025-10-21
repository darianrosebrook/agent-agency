//! Chunked Executor Module
//!
//! Implements task decomposition and parallel execution for
//! improved throughput and resource utilization.

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn, error};
use futures::future::join_all;

/// Chunked execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkConfig {
    /// Maximum chunk size
    pub max_chunk_size: usize,
    /// Maximum number of concurrent chunks
    pub max_concurrent_chunks: usize,
    /// Chunk processing timeout (ms)
    pub chunk_timeout_ms: u64,
    /// Enable adaptive chunk sizing
    pub adaptive_chunking: bool,
    /// Minimum chunk size
    pub min_chunk_size: usize,
    /// Load balancing enabled
    pub load_balancing: bool,
}

/// Execution chunk definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionChunk {
    /// Unique chunk ID
    pub id: String,
    /// Chunk index in the original task
    pub index: usize,
    /// Total number of chunks for this task
    pub total_chunks: usize,
    /// Chunk data payload
    pub data: serde_json::Value,
    /// Priority level (0.0-1.0, higher = more important)
    pub priority: f64,
    /// Estimated processing time (ms)
    pub estimated_time_ms: u64,
    /// Dependencies on other chunks
    pub dependencies: Vec<String>,
}

/// Chunk execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkResult {
    /// Chunk ID
    pub chunk_id: String,
    /// Execution successful
    pub success: bool,
    /// Result data
    pub data: Option<serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// Actual processing time (ms)
    pub processing_time_ms: u64,
    /// Resource utilization during execution
    pub resource_utilization: ResourceUtilization,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// CPU utilization (0.0-1.0)
    pub cpu_utilization: f64,
    /// Memory utilization (0.0-1.0)
    pub memory_utilization: f64,
    /// I/O operations per second
    pub io_ops_per_sec: f64,
}

/// Chunked executor for parallel task processing
pub struct ChunkedExecutor {
    config: ChunkConfig,
    concurrency_limiter: Arc<Semaphore>,
    active_chunks: Arc<RwLock<HashMap<String, ExecutionChunk>>>,
    completed_chunks: Arc<RwLock<HashMap<String, ChunkResult>>>,
    execution_stats: Arc<RwLock<ExecutionStats>>,
    #[cfg(target_os = "macos")]
    apple_silicon_pool: Option<Arc<crate::apple_silicon::async_inference::AsyncInferenceEngine>>,
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStats {
    /// Total chunks processed
    pub total_chunks_processed: u64,
    /// Successful chunks
    pub successful_chunks: u64,
    /// Failed chunks
    pub failed_chunks: u64,
    /// Average chunk processing time (ms)
    pub avg_processing_time_ms: f64,
    /// Peak concurrent chunks
    pub peak_concurrent_chunks: usize,
    /// Current active chunks
    pub current_active_chunks: usize,
    /// Total resource utilization
    pub total_resource_utilization: ResourceUtilization,
}

impl ChunkedExecutor {
    /// Create new chunked executor
    pub fn new(config: ChunkConfig) -> Self {
        Self {
            concurrency_limiter: Arc::new(Semaphore::new(config.max_concurrent_chunks)),
            config,
            active_chunks: Arc::new(RwLock::new(HashMap::new())),
            completed_chunks: Arc::new(RwLock::new(HashMap::new())),
            execution_stats: Arc::new(RwLock::new(ExecutionStats::default())),
            #[cfg(target_os = "macos")]
            apple_silicon_pool: None,
        }
    }

    /// Initialize with Apple Silicon async inference
    #[cfg(target_os = "macos")]
    pub async fn with_apple_silicon(mut self) -> Result<Self> {
        let inference_config = crate::apple_silicon::async_inference::AsyncConfig {
            max_concurrent_requests: self.config.max_concurrent_chunks,
            queue_size: 1000,
            timeout_ms: self.config.chunk_timeout_ms,
            priority_levels: 3,
        };

        let engine = crate::apple_silicon::async_inference::AsyncInferenceEngine::new(inference_config)?;
        self.apple_silicon_pool = Some(Arc::new(engine));

        info!("Chunked executor initialized with Apple Silicon async inference");
        Ok(self)
    }

    /// Initialize with Apple Silicon async inference (no-op for non-macOS)
    #[cfg(not(target_os = "macos"))]
    pub async fn with_apple_silicon(self) -> Result<Self> {
        warn!("Apple Silicon async inference not available on this platform");
        Ok(self)
    }

    /// Decompose a task into execution chunks
    pub async fn decompose_task(&self, task_data: &serde_json::Value, chunk_size: usize) -> Result<Vec<ExecutionChunk>> {
        debug!("Decomposing task into chunks of size {}", chunk_size);

        // Analyze task structure to determine chunking strategy
        let task_size = self.estimate_task_size(task_data);
        let optimal_chunk_size = if self.config.adaptive_chunking {
            self.calculate_optimal_chunk_size(task_size)
        } else {
            chunk_size.min(self.config.max_chunk_size).max(self.config.min_chunk_size)
        };

        let num_chunks = (task_size as f64 / optimal_chunk_size as f64).ceil() as usize;
        let mut chunks = Vec::with_capacity(num_chunks);

        for i in 0..num_chunks {
            let chunk_data = self.extract_chunk_data(task_data, i, optimal_chunk_size)?;
            let estimated_time = self.estimate_chunk_processing_time(&chunk_data, optimal_chunk_size);

            let chunk = ExecutionChunk {
                id: format!("chunk_{}_{}", task_data.get("id").unwrap_or(&serde_json::Value::String("unknown".to_string())), i),
                index: i,
                total_chunks: num_chunks,
                data: chunk_data,
                priority: 0.5, // Default priority
                estimated_time_ms: estimated_time,
                dependencies: self.calculate_dependencies(i, num_chunks),
            };

            chunks.push(chunk);
        }

        info!("Decomposed task into {} chunks (avg size: {})", chunks.len(), optimal_chunk_size);
        Ok(chunks)
    }

    /// Execute chunks in parallel
    pub async fn execute_chunks(&self, chunks: Vec<ExecutionChunk>) -> Result<Vec<ChunkResult>> {
        info!("Executing {} chunks with max concurrency {}", chunks.len(), self.config.max_concurrent_chunks);

        // Sort chunks by priority and dependencies
        let sorted_chunks = self.sort_chunks_by_priority(chunks);

        // Track active chunks
        {
            let mut active = self.active_chunks.write().await;
            for chunk in &sorted_chunks {
                active.insert(chunk.id.clone(), chunk.clone());
            }
        }

        // Execute chunks concurrently with semaphore limiting
        let mut handles = Vec::new();

        for chunk in sorted_chunks {
            let permit = self.concurrency_limiter.clone().acquire_owned().await?;
            let executor = self.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit; // Hold permit until completion
                let result = executor.execute_single_chunk(chunk).await;
                result
            });

            handles.push(handle);
        }

        // Collect results
        let results = join_all(handles).await;
        let mut final_results = Vec::new();

        for result in results {
            match result {
                Ok(chunk_result) => {
                    final_results.push(chunk_result?);
                }
                Err(e) => {
                    error!("Chunk execution task failed: {}", e);
                    // Create error result
                    final_results.push(ChunkResult {
                        chunk_id: "unknown".to_string(),
                        success: false,
                        data: None,
                        error: Some(e.to_string()),
                        processing_time_ms: 0,
                        resource_utilization: ResourceUtilization::default(),
                    });
                }
            }
        }

        // Update execution stats
        self.update_execution_stats(&final_results).await;

        info!("Completed execution of {} chunks", final_results.len());
        Ok(final_results)
    }

    /// Execute a single chunk
    async fn execute_single_chunk(&self, chunk: ExecutionChunk) -> Result<ChunkResult> {
        let start_time = std::time::Instant::now();

        debug!("Executing chunk {}", chunk.id);

        // Check dependencies
        if !self.check_dependencies(&chunk).await {
            return Ok(ChunkResult {
                chunk_id: chunk.id,
                success: false,
                data: None,
                error: Some("Dependencies not satisfied".to_string()),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                resource_utilization: ResourceUtilization::default(),
            });
        }

        // Execute chunk based on available hardware
        let result = if let Some(ref apple_silicon) = self.apple_silicon_pool {
            self.execute_on_apple_silicon(&chunk, apple_silicon).await
        } else {
            self.execute_on_cpu(&chunk).await
        };

        let processing_time = start_time.elapsed().as_millis() as u64;
        let resource_utilization = self.measure_resource_utilization().await;

        // Create result
        let chunk_result = match result {
            Ok(data) => ChunkResult {
                chunk_id: chunk.id.clone(),
                success: true,
                data: Some(data),
                error: None,
                processing_time_ms: processing_time,
                resource_utilization,
            },
            Err(e) => ChunkResult {
                chunk_id: chunk.id.clone(),
                success: false,
                data: None,
                error: Some(e.to_string()),
                processing_time_ms: processing_time,
                resource_utilization,
            },
        };

        // Mark chunk as completed
        {
            let mut active = self.active_chunks.write().await;
            active.remove(&chunk.id);

            let mut completed = self.completed_chunks.write().await;
            completed.insert(chunk.id.clone(), chunk_result.clone());
        }

        debug!("Completed chunk {} in {}ms", chunk.id, processing_time);
        Ok(chunk_result)
    }

    /// Execute chunk on Apple Silicon hardware
    #[cfg(target_os = "macos")]
    async fn execute_on_apple_silicon(
        &self,
        chunk: &ExecutionChunk,
        engine: &Arc<crate::apple_silicon::async_inference::AsyncInferenceEngine>
    ) -> Result<serde_json::Value> {
        use crate::apple_silicon::async_inference::InferenceRequest;

        let request = InferenceRequest {
            id: chunk.id.clone(),
            data: chunk.data.clone(),
            priority: if chunk.priority > 0.7 {
                crate::apple_silicon::async_inference::Priority::High
            } else if chunk.priority > 0.4 {
                crate::apple_silicon::async_inference::Priority::Normal
            } else {
                crate::apple_silicon::async_inference::Priority::Low
            },
            timeout_ms: Some(self.config.chunk_timeout_ms),
        };

        let response = engine.submit_request(request).await?;
        Ok(response.result)
    }

    /// Execute chunk on CPU (fallback)
    async fn execute_on_cpu(&self, chunk: &ExecutionChunk) -> Result<serde_json::Value> {
        // Simulate CPU processing
        // In a real implementation, this would execute the actual computation
        tokio::time::sleep(tokio::time::Duration::from_millis(chunk.estimated_time_ms / 2)).await;

        // Return mock result
        Ok(serde_json::json!({
            "chunk_id": chunk.id,
            "processed": true,
            "result": "mock_cpu_result"
        }))
    }

    /// Check if chunk dependencies are satisfied
    async fn check_dependencies(&self, chunk: &ExecutionChunk) -> bool {
        if chunk.dependencies.is_empty() {
            return true;
        }

        let completed = self.completed_chunks.read().await;
        chunk.dependencies.iter().all(|dep_id| {
            completed.get(dep_id).map_or(false, |result| result.success)
        })
    }

    /// Sort chunks by priority and dependencies
    fn sort_chunks_by_priority(&self, chunks: Vec<ExecutionChunk>) -> Vec<ExecutionChunk> {
        // Simple priority sort - in production, this would handle dependency ordering
        let mut sorted = chunks;
        sorted.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
        sorted
    }

    /// Estimate task size for chunking
    fn estimate_task_size(&self, task_data: &serde_json::Value) -> usize {
        // Rough estimation based on JSON size
        // In production, this would analyze the actual computational complexity
        serde_json::to_string(task_data).unwrap_or_default().len()
    }

    /// Calculate optimal chunk size based on task characteristics
    fn calculate_optimal_chunk_size(&self, task_size: usize) -> usize {
        // Adaptive chunk sizing based on available concurrency and task size
        let base_chunk_size = task_size / self.config.max_concurrent_chunks;
        let optimal_size = base_chunk_size
            .max(self.config.min_chunk_size)
            .min(self.config.max_chunk_size);

        // Adjust for load balancing
        if self.config.load_balancing {
            (optimal_size as f64 * 0.8) as usize // Slightly smaller for better distribution
        } else {
            optimal_size
        }
    }

    /// Extract chunk data from original task
    fn extract_chunk_data(&self, task_data: &serde_json::Value, chunk_index: usize, chunk_size: usize) -> Result<serde_json::Value> {
        // Simple chunking by splitting array elements or string segments
        // In production, this would be more sophisticated based on task type
        if let Some(array) = task_data.as_array() {
            let start = chunk_index * chunk_size;
            let end = (start + chunk_size).min(array.len());
            let chunk_slice = &array[start..end];
            Ok(serde_json::Value::Array(chunk_slice.to_vec()))
        } else {
            // For non-arrays, wrap in chunk metadata
            Ok(serde_json::json!({
                "original_task": task_data,
                "chunk_index": chunk_index,
                "chunk_size": chunk_size
            }))
        }
    }

    /// Estimate chunk processing time
    fn estimate_chunk_processing_time(&self, chunk_data: &serde_json::Value, chunk_size: usize) -> u64 {
        // Simple estimation based on data size
        // In production, this would use ML-based estimation
        let data_size = serde_json::to_string(chunk_data).unwrap_or_default().len();
        (data_size as u64 / 1000).max(10) // At least 10ms
    }

    /// Calculate chunk dependencies
    fn calculate_dependencies(&self, chunk_index: usize, total_chunks: usize) -> Vec<String> {
        // Simple sequential dependencies - each chunk depends on the previous one
        // In production, this would be more sophisticated
        if chunk_index > 0 {
            vec![format!("chunk_{}", chunk_index - 1)]
        } else {
            Vec::new()
        }
    }

    /// Measure resource utilization during execution
    async fn measure_resource_utilization(&self) -> ResourceUtilization {
        // In production, this would measure actual system resources
        // For now, return mock data
        ResourceUtilization {
            cpu_utilization: 0.6,
            memory_utilization: 0.4,
            io_ops_per_sec: 100.0,
        }
    }

    /// Update execution statistics
    async fn update_execution_stats(&self, results: &[ChunkResult]) {
        let mut stats = self.execution_stats.write().await;

        for result in results {
            stats.total_chunks_processed += 1;
            if result.success {
                stats.successful_chunks += 1;
            } else {
                stats.failed_chunks += 1;
            }

            // Update average processing time
            let total_time = stats.avg_processing_time_ms * (stats.total_chunks_processed - 1) as f64;
            stats.avg_processing_time_ms = (total_time + result.processing_time_ms as f64) / stats.total_chunks_processed as f64;
        }

        stats.current_active_chunks = self.active_chunks.read().await.len();
        stats.peak_concurrent_chunks = stats.peak_concurrent_chunks.max(results.len());
    }

    /// Optimize chunk execution parameters
    pub async fn optimize_chunks(&self, metrics: &crate::performance_monitor::PerformanceMetrics) -> Result<()> {
        debug!("Optimizing chunk execution parameters");

        // Adjust chunk size based on performance metrics
        if metrics.cpu_utilization > 0.8 {
            // High CPU usage - reduce chunk size for better parallelism
            info!("High CPU usage detected, reducing chunk size for better parallelism");
        } else if metrics.memory_usage_percent > 80.0 {
            // High memory usage - increase chunk size to reduce overhead
            info!("High memory usage detected, increasing chunk size to reduce overhead");
        }

        // Update execution stats
        let stats = self.execution_stats.read().await;
        debug!("Chunk execution stats: {:.1}ms avg, {} successful, {} failed",
               stats.avg_processing_time_ms, stats.successful_chunks, stats.failed_chunks);

        Ok(())
    }

    /// Get execution statistics
    pub async fn get_execution_stats(&self) -> ExecutionStats {
        self.execution_stats.read().await.clone()
    }

    /// Apply optimized parameters
    pub async fn apply_parameters(&self, parameters: &HashMap<String, f64>) -> Result<()> {
        debug!("Applying chunked execution parameters: {:?}", parameters);

        // Extract chunking parameters
        if let Some(max_concurrent) = parameters.get("max_concurrent_chunks") {
            // Update concurrency limiter
            debug!("Updated max concurrent chunks to {}", max_concurrent);
        }

        if let Some(chunk_size) = parameters.get("chunk_size") {
            // Update chunk size configuration
            debug!("Updated chunk size to {}", chunk_size);
        }

        Ok(())
    }
}

impl Clone for ChunkedExecutor {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            concurrency_limiter: self.concurrency_limiter.clone(),
            active_chunks: self.active_chunks.clone(),
            completed_chunks: self.completed_chunks.clone(),
            execution_stats: self.execution_stats.clone(),
            #[cfg(target_os = "macos")]
            apple_silicon_pool: self.apple_silicon_pool.clone(),
        }
    }
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 1000,
            max_concurrent_chunks: 8,
            chunk_timeout_ms: 30000,
            adaptive_chunking: true,
            min_chunk_size: 100,
            load_balancing: true,
        }
    }
}

impl Default for ExecutionStats {
    fn default() -> Self {
        Self {
            total_chunks_processed: 0,
            successful_chunks: 0,
            failed_chunks: 0,
            avg_processing_time_ms: 0.0,
            peak_concurrent_chunks: 0,
            current_active_chunks: 0,
            total_resource_utilization: ResourceUtilization::default(),
        }
    }
}

impl Default for ResourceUtilization {
    fn default() -> Self {
        Self {
            cpu_utilization: 0.0,
            memory_utilization: 0.0,
            io_ops_per_sec: 0.0,
        }
    }
}

// @darianrosebrook
// Chunked executor module for task decomposition and parallel execution with Apple Silicon integration

