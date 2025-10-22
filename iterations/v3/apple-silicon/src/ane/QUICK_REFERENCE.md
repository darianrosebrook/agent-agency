# ANE Manager Quick Reference

## 🚀 Quick Start

```rust
use apple_silicon::ane::manager::{ANEManager, ANEConfig};

// Basic usage
let manager = ANEManager::new()?;
let model_id = manager.load_model("model.mlmodel", &Default::default()).await?;
let result = manager.execute_inference(model_id, &input, &Default::default()).await?;
```

## 📋 Common Patterns

### 1. Production Configuration

```rust
let config = ANEConfig {
    max_concurrent_operations: 8,
    memory_pool_mb: 2048,
    default_timeout_ms: 5000,
};
let manager = ANEManager::with_config(config)?;
```

### 2. Inference with Custom Options

```rust
let options = InferenceOptions {
    timeout_ms: 10000,
    batch_size: Some(4),
    precision: Some("fp16".to_string()),
    compute_units: Some("ANE".to_string()),
    enable_monitoring: true,
};
let result = manager.execute_inference(model_id, &input, &options).await?;
```

### 3. Resource-Aware Inference

```rust
// Check available resources
let admission = manager.request_admission(model_id, memory_mb).await?;
if let Ok(permit) = admission {
    let result = manager.execute_inference_admitted(permit, &input, &options).await?;
}
```

### 4. Performance Monitoring

```rust
let metrics = manager.get_performance_summary().await?;
println!("Latency: {:.2}ms, Throughput: {:.1} IPS",
    metrics.average_latency_ms,
    metrics.average_throughput_ips);
```

## 🔧 Configuration Reference

### ANEConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_concurrent_operations` | `usize` | 4 | Maximum concurrent inferences |
| `memory_pool_mb` | `usize` | 1024 | Total memory allocation (MB) |
| `default_timeout_ms` | `u64` | 5000 | Default inference timeout |

### InferenceOptions

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `timeout_ms` | `u64` | 5000 | Operation timeout |
| `batch_size` | `Option<usize>` | None | Batch processing size |
| `precision` | `Option<String>` | None | "fp16", "fp32", "int8" |
| `compute_units` | `Option<String>` | None | "ANE", "CPU", "GPU" |
| `enable_monitoring` | `bool` | false | Collect performance metrics |

## ⚠️ Error Handling

```rust
use apple_silicon::ane::errors::ANEError;

match operation().await {
    Ok(result) => println!("Success: {:?}", result),
    Err(ANEError::Timeout(ms)) => println!("Timeout after {}ms", ms),
    Err(ANEError::ResourceLimit(msg)) => println!("Resource limit: {}", msg),
    Err(ANEError::ModelNotFound(path)) => println!("Model missing: {}", path),
    Err(e) => println!("Other error: {}", e),
}
```

## 📊 Key Metrics

| Metric | Access | Description |
|--------|--------|-------------|
| `total_inferences` | `get_performance_summary()` | Total operations |
| `average_latency_ms` | `get_performance_summary()` | EWMA latency |
| `average_throughput_ips` | `get_performance_summary()` | Inferences/second |
| `memory_usage_mb` | `get_performance_summary()` | Current memory |
| `current_memory_usage_mb` | `get_resource_pool_stats()` | Pool usage |
| `total_admissions` | `get_resource_pool_stats()` | Successful admissions |

## 🔍 Debugging

### Enable Debug Logging

```rust
use tracing_subscriber;
tracing_subscriber::fmt().init();

// Operations are automatically instrumented
let result = manager.execute_inference(model_id, &input, &options).await?;
```

### Health Checks

```rust
let capabilities = manager.detect_capabilities().await?;
let device_status = manager.get_device_status().await?;
let pool_stats = manager.get_resource_pool_stats();

println!("ANE Available: {}", capabilities.is_available);
println!("Memory Used: {}MB", pool_stats.current_memory_usage_mb);
```

### Performance Profiling

```rust
// Time a specific operation
let start = std::time::Instant::now();
let result = manager.execute_inference(model_id, &input, &options).await?;
let duration = start.elapsed();

println!("Total time: {:.2}ms", duration.as_millis());
println!("ANE time: {:.2}ms", result.execution_time_ms);
```

## 🧪 Testing

### Unit Test Pattern

```rust
#[tokio::test]
async fn test_inference_basic() {
    let manager = ANEManager::new().unwrap();
    let model_id = manager.load_model("test.mlmodel", &Default::default()).await.unwrap();

    let input = vec![0.0f32; 224 * 224 * 3];
    let result = manager.execute_inference(model_id, &input, &Default::default()).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.output.is_empty());
}
```

### Benchmark Pattern

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_inference(c: &mut Criterion) {
    let manager = ANEManager::new().unwrap();
    let model_id = manager.load_model("bench.mlmodel", &Default::default()).await.unwrap();
    let input = vec![0.5f32; 224 * 224 * 3];

    c.bench_function("ane_inference", |b| {
        b.iter(|| {
            let result = black_box(
                manager.execute_inference(model_id, &input, &Default::default())
            );
            drop(result);
        });
    });
}
```

## 🎯 Best Practices

### Memory Management
- Pre-allocate buffers for repeated inferences
- Monitor memory usage via `get_resource_pool_stats()`
- Use appropriate batch sizes to balance throughput/latency

### Performance Optimization
- Use FP16 precision when possible for 2x speedup
- Enable batching for throughput-critical workloads
- Monitor and tune `max_concurrent_operations` for your hardware

### Error Recovery
- Always handle `ANEError::Timeout` with retries or fallbacks
- Check `ANEError::ResourceLimit` and scale down batch sizes
- Use `detect_capabilities()` to verify ANE availability

### Monitoring
- Enable `enable_monitoring: true` in production
- Log performance metrics periodically
- Alert on increasing latency or error rates

## 🔗 Integration Points

### With Core ML Backend

```rust
use apple_silicon::core_ml_backend::CoreMLBackend;

// ANE Manager provides low-level control
let ane_manager = ANEManager::new()?;

// Core ML Backend provides high-level API
let coreml_backend = CoreMLBackend::new();

// Use together for optimal performance
```

### With Model Router

```rust
use apple_silicon::model_router::ModelRouter;

// Intelligent backend selection
let router = ModelRouter::new(config);
let backend = router.select_backend(&model, &hardware_caps)?;

match backend {
    BackendType::ANE => {
        // Use ANE Manager
        let manager = ANEManager::new()?;
        // ... ANE-specific logic
    }
    _ => {
        // Fallback to other backends
    }
}
```

## 📈 Performance Tuning

### Latency Optimization
```rust
// Minimize latency
let options = InferenceOptions {
    timeout_ms: 1000,
    batch_size: None,  // Single inference
    precision: "fp16", // Faster precision
    compute_units: "ANE",
    enable_monitoring: false, // Skip metrics
};
```

### Throughput Optimization
```rust
// Maximize throughput
let options = InferenceOptions {
    timeout_ms: 10000,
    batch_size: Some(8), // Batch processing
    precision: "fp16",
    compute_units: "ANE",
    enable_monitoring: true,
};
```

### Memory Optimization
```rust
// Minimize memory usage
let config = ANEConfig {
    max_concurrent_operations: 2,  // Reduce concurrency
    memory_pool_mb: 512,           // Smaller pool
    default_timeout_ms: 5000,
};
```

---

**ANE Manager: Production-grade ML inference on Apple Silicon** 🚀
