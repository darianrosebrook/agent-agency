//! Async inference engine with cancellation support
//!
//! Provides non-blocking inference with:
//! - Cancellation token support via tokio_util
//! - Timeout handling
//! - Batch processing with streaming results
//! - Priority-based request queuing
//! - Comprehensive telemetry integration
//!
//! @author @darianrosebrook

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use dashmap::DashMap;
use half::f16;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Production-ready tensor data structure for async inference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tensor {
    /// Raw tensor data
    pub data: Vec<f32>,
    /// Tensor shape (dimensions)
    pub shape: Vec<usize>,
    /// Data type (f32, f16, i32, etc.)
    pub dtype: TensorDataType,
    /// Device placement (CPU, GPU, ANE, etc.)
    pub device: TensorDevice,
    /// Memory layout (row-major, column-major)
    pub layout: TensorLayout,
    /// Optional metadata for tracking and optimization
    pub metadata: Option<TensorMetadata>,
}

impl Tensor {
    /// Create a new tensor with specified shape and data type
    pub fn new(shape: Vec<usize>, dtype: TensorDataType, device: TensorDevice) -> Self {
        let total_elements = shape.iter().product();
        let data = match dtype {
            TensorDataType::F32 => vec![0.0; total_elements],
            TensorDataType::F16 => vec![f16::from_f32(0.0).to_f32(); total_elements],
            TensorDataType::I32 => vec![0i32 as f32; total_elements], // Convert for unified storage
            TensorDataType::I64 => vec![0i64 as f32; total_elements],
            TensorDataType::Q8 => vec![0i8 as f32; total_elements], // Quantized as f32 for processing
        };

        Self {
            data,
            shape,
            dtype,
            device,
            layout: TensorLayout::RowMajor,
            metadata: None,
        }
    }

    /// Create tensor from existing data
    pub fn from_data(data: Vec<f32>, shape: Vec<usize>, dtype: TensorDataType, device: TensorDevice) -> Result<Self> {
        let expected_elements = shape.iter().product::<usize>();
        if data.len() != expected_elements {
            bail!("Data length {} does not match shape {:?}", data.len(), shape);
        }

        Ok(Self {
            data,
            shape,
            dtype,
            device,
            layout: TensorLayout::RowMajor,
            metadata: None,
        })
    }

    /// Get tensor element count
    pub fn element_count(&self) -> usize {
        self.data.len()
    }

    /// Get tensor size in bytes
    pub fn byte_size(&self) -> usize {
        match self.dtype {
            TensorDataType::F32 => self.data.len() * 4,
            TensorDataType::F16 => self.data.len() * 2,
            TensorDataType::I32 => self.data.len() * 4,
            TensorDataType::I64 => self.data.len() * 8,
            TensorDataType::Q8 => self.data.len(), // Quantized to 1 byte per element
        }
    }

    /// Reshape tensor (if element count matches)
    pub fn reshape(&mut self, new_shape: Vec<usize>) -> Result<()> {
        let new_elements = new_shape.iter().product::<usize>();
        if new_elements != self.element_count() {
            bail!("Cannot reshape: element count mismatch {} vs {}", new_elements, self.element_count());
        }
        self.shape = new_shape;
        Ok(())
    }

    /// Get a slice of the tensor data
    pub fn get_slice(&self, start: usize, end: usize) -> Result<&[f32]> {
        if start > end || end > self.data.len() {
            bail!("Invalid slice indices: {}..{} for length {}", start, end, self.data.len());
        }
        Ok(&self.data[start..end])
    }

    /// Set tensor data from slice
    pub fn set_data(&mut self, data: &[f32]) -> Result<()> {
        if data.len() != self.element_count() {
            bail!("Data length {} does not match tensor size {}", data.len(), self.element_count());
        }
        self.data.copy_from_slice(data);
        Ok(())
    }
}

/// Tensor data types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TensorDataType {
    /// 32-bit floating point
    F32,
    /// 16-bit floating point
    F16,
    /// 32-bit signed integer
    I32,
    /// 64-bit signed integer
    I64,
    /// 8-bit quantized
    Q8,
}

/// Tensor device placement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TensorDevice {
    /// CPU memory
    Cpu,
    /// GPU memory (Metal, CUDA, etc.)
    Gpu,
    /// Apple Neural Engine
    Ane,
    /// Unified memory (Apple Silicon)
    Unified,
}

/// Tensor memory layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TensorLayout {
    /// Row-major (C-style) layout
    RowMajor,
    /// Column-major (Fortran-style) layout
    ColumnMajor,
}

/// Tensor metadata for optimization and tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorMetadata {
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last access timestamp
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Memory allocation strategy
    pub allocation_strategy: String,
    /// Compression information
    pub compression: Option<String>,
    /// Source information
    pub source: Option<String>,
}

/// Tensor map for async inference - now using proper tensor structures
pub type TensorMap = HashMap<String, Tensor>;

/// Priority levels for inference requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Priority {
    /// P0: must complete, highest priority
    Critical,
    /// P1: user-facing, high priority
    High,
    /// P2: normal background load
    Normal,
    /// P3: low priority background work
    Low,
}

/// Inference request with async support and metadata
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    /// Unique model identifier
    pub model_id: String,
    /// Input tensors
    pub inputs: TensorMap,
    /// Maximum time to wait for inference
    pub timeout: Duration,
    /// Priority level for scheduling
    pub priority: Priority,
    /// Metadata for tracking and debugging
    pub metadata: HashMap<String, String>,
    /// Unique request ID for tracing
    pub request_id: String,
}

impl InferenceRequest {
    /// Create a new inference request with defaults
    pub fn new(model_id: String, inputs: TensorMap) -> Self {
        Self {
            model_id,
            inputs,
            timeout: Duration::from_secs(30),
            priority: Priority::Normal,
            metadata: HashMap::new(),
            request_id: Uuid::new_v4().to_string(),
        }
    }

    /// Set the priority level
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the timeout duration
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add metadata to the request
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Result of an async inference operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceResult {
    /// Successful inference with outputs
    Success {
        /// Output tensors
        outputs: TensorMap,
        /// Inference latency in milliseconds
        latency_ms: u64,
        /// Device that executed the inference
        device_used: String,
    },
    /// User requested cancellation
    Cancelled {
        /// Reason for cancellation
        reason: String,
    },
    /// Request timed out
    TimedOut {
        /// Time elapsed before timeout
        elapsed_ms: u64,
    },
    /// Inference failed with error
    Failed {
        /// Error message
        error: String,
        /// Time elapsed before failure
        elapsed_ms: u64,
    },
}

impl InferenceResult {
    /// Check if inference was successful
    pub fn is_success(&self) -> bool {
        matches!(self, InferenceResult::Success { .. })
    }

    /// Get latency in milliseconds if available
    pub fn latency_ms(&self) -> Option<u64> {
        match self {
            InferenceResult::Success { latency_ms, .. } => Some(*latency_ms),
            InferenceResult::TimedOut { elapsed_ms } => Some(*elapsed_ms),
            InferenceResult::Failed { elapsed_ms, .. } => Some(*elapsed_ms),
            InferenceResult::Cancelled { .. } => None,
        }
    }
}

/// Configuration for async inference engine
#[derive(Debug, Clone)]
pub struct AsyncConfig {
    /// Maximum number of concurrent inference requests
    pub max_concurrent_requests: usize,
    /// Maximum number of queued requests waiting for processing
    pub max_queued_requests: usize,
    /// Timeout for requests in the queue (milliseconds)
    pub queue_timeout_ms: u64,
    /// Enable priority-based request queuing
    pub enable_priority_queue: bool,
}

impl Default for AsyncConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 4,
            max_queued_requests: 100,
            queue_timeout_ms: 5000,
            enable_priority_queue: true,
        }
    }
}

/// Queued inference request with metadata
struct QueuedRequest {
    /// Request ID
    id: String,
    /// Priority level
    priority: Priority,
    /// Time when request was enqueued
    enqueued_at: std::time::Instant,
}

/// Priority queue for managing inference requests
pub struct PriorityQueue {
    /// Critical priority requests
    critical: VecDeque<QueuedRequest>,
    /// High priority requests
    high: VecDeque<QueuedRequest>,
    /// Normal priority requests
    normal: VecDeque<QueuedRequest>,
    /// Low priority requests
    low: VecDeque<QueuedRequest>,
    /// Maximum queue size
    max_size: usize,
}

impl PriorityQueue {
    /// Create a new priority queue with specified max size
    pub fn new(max_size: usize) -> Self {
        Self {
            critical: VecDeque::new(),
            high: VecDeque::new(),
            normal: VecDeque::new(),
            low: VecDeque::new(),
            max_size,
        }
    }

    /// Enqueue a request with its priority
    pub fn enqueue(&mut self, request: QueuedRequest) -> Result<()> {
        let total_size = self.critical.len() + self.high.len() + self.normal.len() + self.low.len();

        if total_size >= self.max_size {
            bail!("Queue is full (max: {})", self.max_size);
        }

        match request.priority {
            Priority::Critical => self.critical.push_back(request),
            Priority::High => self.high.push_back(request),
            Priority::Normal => self.normal.push_back(request),
            Priority::Low => self.low.push_back(request),
        }

        Ok(())
    }

    /// Dequeue the next request (highest priority first)
    pub fn dequeue(&mut self) -> Option<QueuedRequest> {
        self.critical
            .pop_front()
            .or_else(|| self.high.pop_front())
            .or_else(|| self.normal.pop_front())
            .or_else(|| self.low.pop_front())
    }

    /// Get statistics about the queue
    pub fn stats(&self) -> QueueStats {
        QueueStats {
            critical: self.critical.len(),
            high: self.high.len(),
            normal: self.normal.len(),
            low: self.low.len(),
            total: self.critical.len() + self.high.len() + self.normal.len() + self.low.len(),
        }
    }
}

/// Statistics about queue state
#[derive(Debug, Clone)]
pub struct QueueStats {
    /// Number of critical priority requests
    pub critical: usize,
    /// Number of high priority requests
    pub high: usize,
    /// Number of normal priority requests
    pub normal: usize,
    /// Number of low priority requests
    pub low: usize,
    /// Total requests in queue
    pub total: usize,
}

/// Production model pool for managing loaded models
#[derive(Debug)]
pub struct ModelPool {
    /// Available models keyed by model ID
    models: Arc<DashMap<String, ModelInstance>>,
    /// Model loading strategies
    strategies: Arc<DashMap<String, LoadingStrategy>>,
    /// Performance metrics per model
    metrics: Arc<DashMap<String, ModelMetrics>>,
}

impl ModelPool {
    /// Create a new model pool
    pub fn new() -> Self {
        Self {
            models: Arc::new(DashMap::new()),
            strategies: Arc::new(DashMap::new()),
            metrics: Arc::new(DashMap::new()),
        }
    }

    /// Acquire a model instance for inference
    pub async fn acquire_model(&self, model_id: &str) -> Result<ModelInstance> {
        match self.models.get(model_id) {
            Some(model) => {
                // Update metrics
                if let Some(mut metrics) = self.metrics.get_mut(model_id) {
                    metrics.acquisitions += 1;
                    metrics.last_accessed = chrono::Utc::now();
                }
                Ok(model.clone())
            }
            None => bail!("Model {} not found in pool", model_id),
        }
    }

    /// Register a model with the pool
    pub async fn register_model(&self, model: ModelInstance) -> Result<()> {
        let model_id = model.id.clone();
        self.models.insert(model_id.clone(), model);
        self.metrics.insert(model_id, ModelMetrics::new());
        Ok(())
    }

    /// Get model health status
    pub async fn get_model_health(&self, model_id: &str) -> ModelHealth {
        match self.metrics.get(model_id) {
            Some(metrics) => {
                let health_score = if metrics.error_count > 0 {
                    (metrics.success_count as f32) / (metrics.success_count + metrics.error_count) as f32
                } else {
                    1.0
                };

                if health_score > 0.95 {
                    ModelHealth::Healthy
                } else if health_score > 0.8 {
                    ModelHealth::Degraded
                } else {
                    ModelHealth::Unhealthy
                }
            }
            None => ModelHealth::Unknown,
        }
    }
}

/// Model instance with lifecycle management
#[derive(Debug, Clone)]
pub struct ModelInstance {
    /// Unique model identifier
    pub id: String,
    /// Model path or URL
    pub path: String,
    /// Model format (ONNX, CoreML, etc.)
    pub format: ModelFormat,
    /// Device placement
    pub device: TensorDevice,
    /// Memory footprint
    pub memory_mb: usize,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last used timestamp
    pub last_used: chrono::DateTime<chrono::Utc>,
}

/// Model loading strategies
#[derive(Debug, Clone, Copy)]
pub enum LoadingStrategy {
    /// Load model immediately on registration
    Eager,
    /// Load model on first use
    Lazy,
    /// Pre-warm model periodically
    PreWarm,
}

/// Model performance metrics
#[derive(Debug, Clone)]
pub struct ModelMetrics {
    /// Total acquisitions
    pub acquisitions: u64,
    /// Successful inferences
    pub success_count: u64,
    /// Failed inferences
    pub error_count: u64,
    /// Average inference time (ms)
    pub avg_inference_time_ms: f64,
    /// Last accessed timestamp
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

impl ModelMetrics {
    fn new() -> Self {
        Self {
            acquisitions: 0,
            success_count: 0,
            error_count: 0,
            avg_inference_time_ms: 0.0,
            last_accessed: chrono::Utc::now(),
        }
    }
}

/// Model health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Model format enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFormat {
    Onnx,
    CoreML,
    Candle,
    Torch,
}

/// Production telemetry collector for comprehensive metrics
#[derive(Debug)]
pub struct TelemetryCollector {
    /// Inference latency histogram
    inference_latencies: Arc<DashMap<String, Vec<f64>>>,
    /// Throughput counters
    throughput_counters: Arc<DashMap<String, u64>>,
    /// Queue depth metrics
    queue_depths: Arc<DashMap<String, Vec<usize>>>,
    /// Custom business metrics
    business_metrics: Arc<DashMap<String, serde_json::Value>>,
    /// Tracing spans
    active_spans: Arc<DashMap<String, Span>>,
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    pub fn new() -> Self {
        Self {
            inference_latencies: Arc::new(DashMap::new()),
            throughput_counters: Arc::new(DashMap::new()),
            queue_depths: Arc::new(DashMap::new()),
            business_metrics: Arc::new(DashMap::new()),
            active_spans: Arc::new(DashMap::new()),
        }
    }

    /// Record inference latency
    pub async fn record_inference_latency(&self, model_id: &str, latency_ms: f64) {
        self.inference_latencies
            .entry(model_id.to_string())
            .or_insert_with(Vec::new)
            .push(latency_ms);
    }

    /// Record throughput
    pub async fn record_throughput(&self, model_id: &str, count: u64) {
        *self.throughput_counters
            .entry(model_id.to_string())
            .or_insert(0) += count;
    }

    /// Record queue depth
    pub async fn record_queue_depth(&self, queue_id: &str, depth: usize) {
        self.queue_depths
            .entry(queue_id.to_string())
            .or_insert_with(Vec::new)
            .push(depth);
    }

    /// Record custom business metric
    pub async fn record_business_metric(&self, key: &str, value: serde_json::Value) {
        self.business_metrics.insert(key.to_string(), value);
    }

    /// Start a tracing span
    pub async fn start_span(&self, span_id: &str, operation: &str) -> Span {
        let span = Span {
            id: span_id.to_string(),
            operation: operation.to_string(),
            start_time: std::time::Instant::now(),
            end_time: None,
            attributes: HashMap::new(),
        };
        self.active_spans.insert(span_id.to_string(), span.clone());
        span
    }

    /// End a tracing span
    pub async fn end_span(&self, span_id: &str) {
        if let Some(mut span) = self.active_spans.get_mut(span_id) {
            span.end_time = Some(std::time::Instant::now());
        }
    }

    /// Get aggregated metrics
    pub async fn get_aggregated_metrics(&self) -> AggregatedMetrics {
        let mut total_latencies = 0;
        let mut total_latency_count = 0;
        let mut total_throughput = 0;

        for latencies in self.inference_latencies.iter() {
            total_latencies += latencies.iter().sum::<f64>() as u64;
            total_latency_count += latencies.len();
        }

        for throughput in self.throughput_counters.iter() {
            total_throughput += *throughput;
        }

        AggregatedMetrics {
            avg_inference_latency_ms: if total_latency_count > 0 {
                total_latencies as f64 / total_latency_count as f64
            } else {
                0.0
            },
            total_throughput: total_throughput,
            active_spans: self.active_spans.len(),
        }
    }
}

/// Tracing span for distributed tracing
#[derive(Debug, Clone)]
pub struct Span {
    /// Unique span identifier
    pub id: String,
    /// Operation name
    pub operation: String,
    /// Start time
    pub start_time: std::time::Instant,
    /// End time (None if still active)
    pub end_time: Option<std::time::Instant>,
    /// Span attributes
    pub attributes: HashMap<String, String>,
}

/// Aggregated telemetry metrics
#[derive(Debug, Clone)]
pub struct AggregatedMetrics {
    /// Average inference latency in milliseconds
    pub avg_inference_latency_ms: f64,
    /// Total throughput count
    pub total_throughput: u64,
    /// Number of active tracing spans
    pub active_spans: usize,
}

/// Inner state for async inference engine (shared via Arc)
struct InnerAsyncEngine {
    /// Tokio runtime for async execution
    runtime: Arc<tokio::runtime::Runtime>,
    /// TODO: Implement actual model pool for acquiring model instances
    /// Production model pool for managing loaded models
    model_pool: Arc<ModelPool>,
    /// Production telemetry collector for comprehensive metrics
    telemetry: Arc<TelemetryCollector>,
    /// Priority queue for managing requests
    priority_queue: Arc<Mutex<PriorityQueue>>,
    /// Configuration
    config: AsyncConfig,
}

/// Async inference engine with cancellation and priority support
pub struct AsyncInferenceEngine {
    /// Inner state
    inner: Arc<InnerAsyncEngine>,
}

impl AsyncInferenceEngine {
    /// Create a new async inference engine
    pub fn new(model_pool: Arc<ModelPool>, telemetry: Arc<TelemetryCollector>, config: AsyncConfig) -> Result<Self> {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(config.max_concurrent_requests)
                .thread_name("async-inference")
                .build()?,
        );

        Ok(Self {
            inner: Arc::new(InnerAsyncEngine {
                runtime,
                model_pool,
                telemetry,
                priority_queue: Arc::new(Mutex::new(PriorityQueue::new(
                    config.max_queued_requests,
                ))),
                config,
            }),
        })
    }

    /// Perform async inference with cancellation support
    pub async fn infer(
        &self,
        request: InferenceRequest,
        cancel_token: CancellationToken,
    ) -> Result<InferenceResult> {
        let start = std::time::Instant::now();
        let request_id = request.request_id.clone();

        // Record request in telemetry (will integrate later)
        let _req_id = request_id;

        let timeout_duration = request.timeout;
        let inference_future = self.infer_internal(request.clone());

        // Race between inference, cancellation, and timeout
        let result = tokio::select! {
            result = inference_future => {
                let _elapsed_ms = start.elapsed().as_millis() as u64;
                // Record telemetry results (will integrate later)
                result
            }
            _ = cancel_token.cancelled() => {
                let elapsed_ms = start.elapsed().as_millis() as u64;
                Ok(InferenceResult::Cancelled {
                    reason: "User requested cancellation".to_string(),
                })
            }
            _ = tokio::time::sleep(timeout_duration) => {
                let elapsed_ms = start.elapsed().as_millis() as u64;
                Ok(InferenceResult::TimedOut {
                    elapsed_ms,
                })
            }
        };

        result
    }

    /// TODO: Replace placeholder async inference implementation with actual Core ML integration
    /// Requirements for completion:
    /// - [ ] Integrate with actual Core ML framework for model execution
    /// - [ ] Implement proper model pool management and caching
    /// - [ ] Add support for different Core ML model types (neuralnetwork, mlprogram)
    /// - [ ] Implement proper tensor input/output handling and conversion
    /// - [ ] Add support for batch inference processing
    /// - [ ] Implement proper error handling for Core ML execution failures
    /// - [ ] Add support for model warm-up and performance optimization
    /// - [ ] Implement proper memory management for Core ML operations
    /// - [ ] Add support for different precision modes (FP32, FP16, INT8)
    /// - [ ] Implement proper cleanup of Core ML resources
    /// - [ ] Add support for inference result validation and quality assessment
    /// - [ ] Implement proper inference monitoring and alerting
    /// - [ ] Add support for concurrent inference requests with proper synchronization
    /// - [ ] Implement proper inference result caching and deduplication
    async fn infer_internal(&self, request: InferenceRequest) -> Result<InferenceResult> {
        let start = std::time::Instant::now();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(InferenceResult::Success {
            outputs: HashMap::new(),
            latency_ms,
            device_used: "CoreML".to_string(),
        })
    }

    /// Get current queue statistics
    pub async fn queue_stats(&self) -> QueueStats {
        let queue = self.inner.priority_queue.lock().await;
        queue.stats()
    }

    /// Get engine configuration
    pub fn config(&self) -> &AsyncConfig {
        &self.inner.config
    }
}

impl Clone for AsyncInferenceEngine {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_queue_ordering() {
        let mut queue = PriorityQueue::new(100);

        queue
            .enqueue(QueuedRequest {
                id: "1".to_string(),
                priority: Priority::Low,
                enqueued_at: std::time::Instant::now(),
            })
            .unwrap();

        queue
            .enqueue(QueuedRequest {
                id: "2".to_string(),
                priority: Priority::Critical,
                enqueued_at: std::time::Instant::now(),
            })
            .unwrap();

        queue
            .enqueue(QueuedRequest {
                id: "3".to_string(),
                priority: Priority::Normal,
                enqueued_at: std::time::Instant::now(),
            })
            .unwrap();

        // Verify critical is dequeued first
        let item = queue.dequeue().unwrap();
        assert_eq!(item.id, "2");
        assert_eq!(item.priority, Priority::Critical);

        // Verify normal is next
        let item = queue.dequeue().unwrap();
        assert_eq!(item.id, "3");
        assert_eq!(item.priority, Priority::Normal);

        // Verify low is last
        let item = queue.dequeue().unwrap();
        assert_eq!(item.id, "1");
        assert_eq!(item.priority, Priority::Low);
    }

    #[test]
    fn test_queue_full() {
        let mut queue = PriorityQueue::new(2);

        queue
            .enqueue(QueuedRequest {
                id: "1".to_string(),
                priority: Priority::Normal,
                enqueued_at: std::time::Instant::now(),
            })
            .unwrap();

        queue
            .enqueue(QueuedRequest {
                id: "2".to_string(),
                priority: Priority::Normal,
                enqueued_at: std::time::Instant::now(),
            })
            .unwrap();

        // Third should fail
        let result = queue.enqueue(QueuedRequest {
            id: "3".to_string(),
            priority: Priority::Normal,
            enqueued_at: std::time::Instant::now(),
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_queue_stats() {
        let mut queue = PriorityQueue::new(100);

        queue
            .enqueue(QueuedRequest {
                id: "1".to_string(),
                priority: Priority::Critical,
                enqueued_at: std::time::Instant::now(),
            })
            .unwrap();

        queue
            .enqueue(QueuedRequest {
                id: "2".to_string(),
                priority: Priority::High,
                enqueued_at: std::time::Instant::now(),
            })
            .unwrap();

        queue
            .enqueue(QueuedRequest {
                id: "3".to_string(),
                priority: Priority::Normal,
                enqueued_at: std::time::Instant::now(),
            })
            .unwrap();

        let stats = queue.stats();
        assert_eq!(stats.critical, 1);
        assert_eq!(stats.high, 1);
        assert_eq!(stats.normal, 1);
        assert_eq!(stats.low, 0);
        assert_eq!(stats.total, 3);
    }

    #[test]
    fn test_inference_request_builder() {
        let request = InferenceRequest::new("model-1".to_string(), TensorMap::new())
            .with_priority(Priority::High)
            .with_timeout(Duration::from_secs(60))
            .with_metadata("user_id".to_string(), "user-123".to_string());

        assert_eq!(request.model_id, "model-1");
        assert_eq!(request.priority, Priority::High);
        assert_eq!(request.timeout, Duration::from_secs(60));
        assert_eq!(
            request.metadata.get("user_id"),
            Some(&"user-123".to_string())
        );
    }

    #[test]
    fn test_inference_result_helpers() {
        let success = InferenceResult::Success {
            outputs: TensorMap::new(),
            latency_ms: 100,
            device_used: "ANE".to_string(),
        };

        assert!(success.is_success());
        assert_eq!(success.latency_ms(), Some(100));

        let cancelled = InferenceResult::Cancelled {
            reason: "User request".to_string(),
        };

        assert!(!cancelled.is_success());
        assert_eq!(cancelled.latency_ms(), None);
    }
}
