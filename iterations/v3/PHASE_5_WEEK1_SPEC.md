# Phase 5, Week 1: Async Inference Foundation - Implementation Spec

**Target Date:** Week of October 28, 2025  
**Duration:** 1 week  
**Deliverables:** Async inference API with cancellation support  

---

## Overview

Week 1 establishes the async foundation for Phase 5. This week focuses on:
- Building a non-blocking async inference API
- Implementing cancellation token support
- Integrating with existing telemetry
- Comprehensive unit tests

---

## Module Specification: `async_inference.rs`

### 1. Core Types

```rust
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use anyhow::{Result, bail};
use serde::{Serialize, Deserialize};

/// Priority levels for inference requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Priority {
    Critical,   // P0: must complete
    High,       // P1: user-facing
    Normal,     // P2: normal load
    Low,        // P3: background
}

/// Inference request with async support
#[derive(Debug, Clone)]
pub struct InferenceRequest {
    pub model_id: String,
    pub inputs: TensorMap,
    pub timeout: Duration,
    pub priority: Priority,
    pub metadata: HashMap<String, String>,
    pub request_id: String,  // Unique ID for tracking
}

impl InferenceRequest {
    pub fn new(model_id: String, inputs: TensorMap) -> Self {
        Self {
            model_id,
            inputs,
            timeout: Duration::from_secs(30),
            priority: Priority::Normal,
            metadata: HashMap::new(),
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Result of an async inference operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceResult {
    Success {
        outputs: TensorMap,
        latency_ms: u64,
        device_used: String,
    },
    Cancelled {
        reason: String,
    },
    TimedOut {
        elapsed_ms: u64,
    },
    Failed {
        error: String,
        elapsed_ms: u64,
    },
}

impl InferenceResult {
    pub fn is_success(&self) -> bool {
        matches!(self, InferenceResult::Success { .. })
    }

    pub fn latency_ms(&self) -> Option<u64> {
        match self {
            InferenceResult::Success { latency_ms, .. } => Some(*latency_ms),
            InferenceResult::TimedOut { elapsed_ms } => Some(*elapsed_ms),
            InferenceResult::Failed { elapsed_ms, .. } => Some(*elapsed_ms),
            InferenceResult::Cancelled { .. } => None,
        }
    }
}
```

### 2. Async Inference Engine

```rust
pub struct AsyncInferenceEngine {
    inner: Arc<InnerAsyncEngine>,
}

struct InnerAsyncEngine {
    runtime: Arc<tokio::runtime::Runtime>,
    model_pool: Arc<ModelPool>,
    telemetry: Arc<TelemetryCollector>,
    priority_queue: Arc<tokio::sync::Mutex<PriorityQueue>>,
    config: AsyncConfig,
}

#[derive(Debug, Clone)]
pub struct AsyncConfig {
    pub max_concurrent_requests: usize,
    pub max_queued_requests: usize,
    pub queue_timeout_ms: u64,
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

impl AsyncInferenceEngine {
    /// Create a new async inference engine
    pub fn new(
        model_pool: Arc<ModelPool>,
        telemetry: Arc<TelemetryCollector>,
        config: AsyncConfig,
    ) -> Result<Self> {
        let runtime = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(config.max_concurrent_requests)
                .thread_name("async-inference")
                .build()?
        );

        Ok(Self {
            inner: Arc::new(InnerAsyncEngine {
                runtime,
                model_pool,
                telemetry,
                priority_queue: Arc::new(tokio::sync::Mutex::new(
                    PriorityQueue::new(config.max_queued_requests)
                )),
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

        // Record request in telemetry
        self.inner.telemetry.record_inference_start(&request_id);

        // Create timeout future
        let timeout_duration = request.timeout;
        let inference_future = self.infer_internal(request.clone());

        // Race between inference, cancellation, and timeout
        tokio::select! {
            result = inference_future => {
                let elapsed_ms = start.elapsed().as_millis() as u64;
                let outcome = match &result {
                    Ok(InferenceResult::Success { .. }) => {
                        self.inner.telemetry.record_inference_success(&request_id, elapsed_ms);
                        result
                    },
                    Ok(InferenceResult::Failed { error, .. }) => {
                        self.inner.telemetry.record_inference_failure(&request_id, elapsed_ms, error);
                        result
                    },
                    _ => result,
                };
                outcome
            }
            _ = cancel_token.cancelled() => {
                let elapsed_ms = start.elapsed().as_millis() as u64;
                self.inner.telemetry.record_inference_cancelled(&request_id, elapsed_ms);
                Ok(InferenceResult::Cancelled {
                    reason: "User requested cancellation".to_string(),
                })
            }
            _ = tokio::time::sleep(timeout_duration) => {
                let elapsed_ms = start.elapsed().as_millis() as u64;
                self.inner.telemetry.record_inference_timeout(&request_id, elapsed_ms);
                Ok(InferenceResult::TimedOut {
                    elapsed_ms,
                })
            }
        }
    }

    /// Internal inference execution
    async fn infer_internal(&self, request: InferenceRequest) -> Result<InferenceResult> {
        let start = std::time::Instant::now();

        // Acquire model from pool
        let model_id = self.inner.model_pool.acquire()
            .map_err(|e| anyhow::anyhow!("Failed to acquire model: {}", e))?;

        // Perform synchronous inference on blocking task
        let result = self.inner.runtime.spawn_blocking({
            let model_id = model_id.clone();
            let inputs = request.inputs.clone();
            move || {
                // Synchronous inference (placeholder)
                std::thread::sleep(std::time::Duration::from_millis(100));
                Ok(())
            }
        }).await??;

        // Release model back to pool
        self.inner.model_pool.release(model_id)?;

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(InferenceResult::Success {
            outputs: TensorMap::new(),  // Placeholder
            latency_ms,
            device_used: "CoreML".to_string(),
        })
    }

    /// Batch inference with streaming results
    pub fn infer_batch(
        &self,
        requests: Vec<InferenceRequest>,
        cancel_token: CancellationToken,
    ) -> tokio::sync::mpsc::Receiver<InferenceResult> {
        let (tx, rx) = tokio::sync::mpsc::channel(requests.len());
        let engine = self.clone();

        tokio::spawn(async move {
            for request in requests {
                let cancel = cancel_token.clone();
                match engine.infer(request, cancel).await {
                    Ok(result) => {
                        if let Err(_) = tx.send(result).await {
                            break;  // Receiver dropped
                        }
                    }
                    Err(e) => {
                        eprintln!("Batch inference error: {}", e);
                        break;
                    }
                }
            }
        });

        rx
    }

    /// Get current queue stats
    pub async fn queue_stats(&self) -> QueueStats {
        let queue = self.inner.priority_queue.lock().await;
        queue.stats()
    }

    /// Get engine config
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
```

### 3. Priority Queue

```rust
pub struct PriorityQueue {
    critical: VecDeque<QueuedRequest>,
    high: VecDeque<QueuedRequest>,
    normal: VecDeque<QueuedRequest>,
    low: VecDeque<QueuedRequest>,
    max_size: usize,
}

struct QueuedRequest {
    id: String,
    priority: Priority,
    enqueued_at: std::time::Instant,
}

impl PriorityQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            critical: VecDeque::new(),
            high: VecDeque::new(),
            normal: VecDeque::new(),
            low: VecDeque::new(),
            max_size,
        }
    }

    pub fn enqueue(&mut self, request: QueuedRequest) -> Result<()> {
        let total_size = self.critical.len() + self.high.len() + 
                        self.normal.len() + self.low.len();

        if total_size >= self.max_size {
            bail!("Queue is full");
        }

        match request.priority {
            Priority::Critical => self.critical.push_back(request),
            Priority::High => self.high.push_back(request),
            Priority::Normal => self.normal.push_back(request),
            Priority::Low => self.low.push_back(request),
        }

        Ok(())
    }

    pub fn dequeue(&mut self) -> Option<QueuedRequest> {
        self.critical.pop_front()
            .or_else(|| self.high.pop_front())
            .or_else(|| self.normal.pop_front())
            .or_else(|| self.low.pop_front())
    }

    pub fn stats(&self) -> QueueStats {
        QueueStats {
            critical: self.critical.len(),
            high: self.high.len(),
            normal: self.normal.len(),
            low: self.low.len(),
            total: self.critical.len() + self.high.len() + 
                   self.normal.len() + self.low.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QueueStats {
    pub critical: usize,
    pub high: usize,
    pub normal: usize,
    pub low: usize,
    pub total: usize,
}
```

---

## Test Specification

### Test 1: Basic Async Inference
```rust
#[tokio::test]
async fn test_basic_async_inference() {
    let engine = create_test_engine();
    let request = InferenceRequest::new(
        "test-model".to_string(),
        TensorMap::new(),
    );
    let cancel_token = CancellationToken::new();

    let result = engine.infer(request, cancel_token).await.unwrap();
    assert!(result.is_success());
    assert!(result.latency_ms().is_some());
}
```

### Test 2: Cancellation Support
```rust
#[tokio::test]
async fn test_inference_cancellation() {
    let engine = create_test_engine();
    let cancel_token = CancellationToken::new();

    let request = InferenceRequest::new(
        "test-model".to_string(),
        TensorMap::new(),
    ).with_timeout(Duration::from_secs(10));

    let handle = tokio::spawn({
        let engine = engine.clone();
        let cancel = cancel_token.clone();
        let request = request.clone();
        async move {
            engine.infer(request, cancel).await
        }
    });

    // Cancel after 50ms
    tokio::time::sleep(Duration::from_millis(50)).await;
    cancel_token.cancel();

    let result = handle.await.unwrap().unwrap();
    assert!(matches!(result, InferenceResult::Cancelled { .. }));
}
```

### Test 3: Timeout Handling
```rust
#[tokio::test]
async fn test_inference_timeout() {
    let engine = create_test_engine();
    let cancel_token = CancellationToken::new();

    let request = InferenceRequest::new(
        "test-model".to_string(),
        TensorMap::new(),
    ).with_timeout(Duration::from_millis(50));

    let result = engine.infer(request, cancel_token).await.unwrap();
    assert!(matches!(result, InferenceResult::TimedOut { .. }));
}
```

### Test 4: Batch Inference
```rust
#[tokio::test]
async fn test_batch_inference() {
    let engine = create_test_engine();
    let cancel_token = CancellationToken::new();

    let requests = vec![
        InferenceRequest::new("model1".to_string(), TensorMap::new()),
        InferenceRequest::new("model2".to_string(), TensorMap::new()),
        InferenceRequest::new("model3".to_string(), TensorMap::new()),
    ];

    let mut rx = engine.infer_batch(requests, cancel_token);
    let mut count = 0;

    while let Some(result) = rx.recv().await {
        assert!(result.is_success());
        count += 1;
    }

    assert_eq!(count, 3);
}
```

### Test 5: Priority Queue
```rust
#[tokio::test]
async fn test_priority_queue() {
    let mut queue = PriorityQueue::new(100);

    queue.enqueue(QueuedRequest {
        id: "1".to_string(),
        priority: Priority::Low,
        enqueued_at: std::time::Instant::now(),
    }).unwrap();

    queue.enqueue(QueuedRequest {
        id: "2".to_string(),
        priority: Priority::Critical,
        enqueued_at: std::time::Instant::now(),
    }).unwrap();

    // Critical should be dequeued first
    let item = queue.dequeue().unwrap();
    assert_eq!(item.priority, Priority::Critical);
}
```

---

## Integration Points

### With Existing Components

1. **ModelPool Integration**
   - Acquire model before inference
   - Release model after completion
   - Handle acquire timeout

2. **TelemetryCollector Integration**
   - Record async inference start/end
   - Track cancellations and timeouts
   - Measure async overhead

3. **InferenceEngine Trait**
   - Keep sync API as backup
   - Expose async API as primary
   - Share telemetry backend

---

## Success Criteria

- ✅ Basic async inference working
- ✅ Cancellation support functional
- ✅ Timeout handling correct
- ✅ Batch inference streaming
- ✅ Priority queue working
- ✅ 5/5 unit tests passing
- ✅ < 5ms async overhead
- ✅ Zero panics on cancellation

---

## Next Steps

1. Implement `async_inference.rs`
2. Write 5 unit tests
3. Integrate with existing telemetry
4. Performance benchmarking
5. Documentation and examples

---

**Week 1 Target:** October 28-November 3, 2025

