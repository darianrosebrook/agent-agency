# Core ML Acceleration API Reference

## Overview

The Core ML acceleration API provides a high-level Rust interface for AI inference on Apple Silicon hardware. The API is designed for production use with comprehensive error handling, performance monitoring, and fault tolerance.

## Core Types

### CoreMLAccelerationSystem

The main entry point for the Core ML acceleration system.

#### Methods

**`initialize() -> Result<Self>`**
- Initializes the complete Core ML acceleration system
- Performs component validation and performance target assessment
- Returns a configured system ready for inference operations

**`load_model(&self, model_path: &str, model_name: &str) -> Result<ModelRef>`**
- Loads a compiled Core ML model (.mlmodelc) into the system
- Registers the model in thread-local storage for safe access
- Tracks memory usage for resource management

**`execute_inference<F, Fut, T>(&self, operation: F) -> Result<T>`**
- Executes inference operations with full system protection
- Applies circuit breaker, timeout, and resource monitoring
- Updates performance metrics and health status

**`get_metrics(&self) -> Arc<SystemMetrics>`**
- Returns current system performance and health metrics
- Includes inference counts, latency statistics, and resource usage

**`get_performance_validation(&self) -> PerformanceValidation`**
- Returns validation results against performance targets
- Includes ANE speedup ratio and dispatch rate achievements

**`get_status(&self) -> IntegrationStatus`**
- Returns current system integration status
- Tracks progression from initialization to production readiness

**`run_system_diagnostic(&self) -> Result<SystemDiagnostic>`**
- Performs comprehensive system health check
- Returns detailed diagnostic information for monitoring

### SystemMetrics

Real-time system performance and health metrics.

#### Fields

- **`total_inferences: u64`** - Total number of inferences processed
- **`successful_inferences: u64`** - Number of successful inferences
- **`failed_inferences: u64`** - Number of failed inferences
- **`avg_latency_ms: f64`** - Average inference latency in milliseconds
- **`ane_utilization_percent: f64`** - Percentage of inferences using ANE
- **`memory_usage_percent: f64`** - Current memory usage percentage
- **`health_status: HealthStatus`** - Current system health status
- **`integration_status: IntegrationStatus`** - Current integration status

### PerformanceValidation

Results of performance target validation.

#### Fields

- **`ane_speedup_ratio: Option<f64>`** - Measured ANE speedup vs CPU baseline
- **`ane_dispatch_rate: Option<f64>`** - Percentage of inferences using ANE
- **`speedup_target_met: bool`** - Whether 2.8x speedup target is achieved
- **`dispatch_target_met: bool`** - Whether 70% dispatch rate target is achieved
- **`latency_target_met: bool`** - Whether <250ms P95 latency target is met
- **`success_rate_target_met: bool`** - Whether >95% success rate target is met
- **`overall_status: bool`** - Whether all performance targets are met

### DeviceCapabilities

Apple Silicon device capability information.

#### Fields

- **`chip_family: String`** - Device family ("M1", "M2", "M3")
- **`ane_performance_score: f64`** - ANE performance rating (0.0-1.0)
- **`unified_memory_gb: usize`** - Available unified memory in GB
- **`ane_cores: usize`** - Number of ANE cores
- **`supported_ml_versions: Vec<String>`** - Compatible Core ML versions
- **`recommended_precision: String`** - Recommended precision ("FP16", "FP32")

## Agent Integration API

### AgentJudgeIntegration

Integration point for agent judge system inference.

#### Methods

**`new(acceleration_system: Arc<CoreMLAccelerationSystem>) -> Self`**
- Creates a new agent judge integration instance
- Configures performance monitoring for judge operations

**`execute_judge_inference(&self, input: Vec<f32>) -> Result<Vec<f32>>`**
- Executes judge inference with acceleration
- Returns processed output vector
- Tracks judge-specific performance metrics

**`get_judge_metrics(&self) -> JudgeMetrics`**
- Returns judge system performance metrics
- Includes judgment counts and acceleration statistics

### JudgeMetrics

Performance metrics specific to judge system operations.

#### Fields

- **`total_judgments: u64`** - Total number of judgments processed
- **`accelerated_judgments: u64`** - Number of ANE-accelerated judgments
- **`avg_judgment_time_ms: f64`** - Average judgment processing time
- **`acceleration_speedup: f64`** - Acceleration factor vs CPU-only execution

## Production Readiness API

### production_readiness::assess_readiness()

Assesses overall production readiness of the system.

**Parameters:**
- `system: &CoreMLAccelerationSystem` - System instance to assess

**Returns:**
- `Result<ReadinessChecklist>` - Detailed readiness assessment

### ReadinessChecklist

Production readiness assessment results.

#### Fields

- **`components_integrated: bool`** - Whether all system components are integrated
- **`performance_targets_met: bool`** - Whether performance targets are achieved
- **`health_monitoring_active: bool`** - Whether health monitoring is operational
- **`resource_management_configured: bool`** - Whether resource management is configured
- **`error_handling_robust: bool`** - Whether error handling is comprehensive
- **`monitoring_integrated: bool`** - Whether monitoring systems are integrated
- **`agent_integration_ready: bool`** - Whether agent integration is functional

## Low-Level Core ML Types

### MLModel

Opaque handle to a compiled Core ML model.

#### Methods

**`from_path(path: &str) -> Result<Self>`**
- Loads a compiled model from file system
- Validates model compatibility with runtime

**`prediction_from_features(&self, features: &MLFeatureProvider, input_name: &str) -> Result<MLFeatureProvider>`**
- Executes inference with input features
- Returns prediction results as feature provider

**`model_info(&self) -> Result<ModelMetadata>`**
- Retrieves model metadata and capabilities
- Includes input/output specifications

### MLFeatureProvider

Abstraction for model input and output features.

#### Methods

**`from_dictionary(features: HashMap<String, MLFeatureValue>) -> Result<Self>`**
- Creates feature provider from key-value feature map
- Validates feature compatibility

### MLMultiArray

Multi-dimensional tensor representation.

#### Methods

**`from_slice(data: &[f32], shape: &[i32]) -> Result<Self>`**
- Creates array from slice with specified shape
- Validates data size against shape

**`data_pointer(&self) -> *const f32`**
- Returns pointer to underlying data
- Used for FFI operations

**`shape(&self) -> &[i32]`**
- Returns array shape dimensions
- Used for validation and compatibility checks

## Error Types

### ANEError

Core ML acceleration specific errors.

#### Variants

- **`Internal(String)`** - Internal system errors with descriptive message
- **`ModelLoadFailed(String)`** - Model loading failures
- **`InferenceFailed(String)`** - Inference execution failures
- **`ValidationError(String)`** - Input validation failures
- **`ResourceExhausted(String)`** - Resource limitation errors
- **`Timeout(String)`** - Operation timeout errors
- **`CircuitBreakerOpen`** - Circuit breaker preventing operations
- **`UnsupportedDevice(String)`** - Device compatibility issues

## Configuration

### Environment Variables

**Required:**
- None - system auto-detects capabilities

**Optional:**
- `COREML_MODEL_PATH` - Custom model storage directory (default: system temp)
- `COREML_TIMEOUT_MS` - Inference timeout in milliseconds (default: 5000)
- `COREML_MEMORY_LIMIT_MB` - Memory usage limit in MB (default: system-based)

### Performance Targets

Default targets for system validation:

```rust
PerformanceTargets {
    target_ane_speedup: 2.8,      // 2.8x speedup target
    target_dispatch_rate: 0.7,    // 70% ANE utilization target
    max_latency_ms: 250.0,        // P95 < 250ms
    min_success_rate: 0.95,       // >95% success rate
}
```

## Usage Examples

### Basic System Initialization

```rust
use system_acceleration::ane::compat::integration::*;

// Initialize the system
let system = CoreMLAccelerationSystem::initialize()?;

// Load a model
let model_ref = system.load_model("/path/to/model.mlmodelc", "my_model")?;

// Execute inference
let result: Vec<f32> = system.execute_inference(|| async {
    // Your inference logic here
    Ok(vec![1.0, 2.0, 3.0])
}).await?;

// Check performance
let validation = system.get_performance_validation();
println!("ANE Speedup: {:.2}x", validation.ane_speedup_ratio.unwrap_or(0.0));
println!("Dispatch Rate: {:.1}%", validation.ane_dispatch_rate.unwrap_or(0.0) * 100.0);
```

### Agent Judge Integration

```rust
use system_acceleration::ane::compat::integration::agent_integration::*;

// Create integration
let judge_integration = AgentJudgeIntegration::new(system);

// Execute judge inference
let input = vec![0.1, 0.2, 0.3, 0.4, 0.5];
let output = judge_integration.execute_judge_inference(input).await?;

// Get metrics
let metrics = judge_integration.get_judge_metrics();
println!("Total judgments: {}", metrics.total_judgments);
println!("Acceleration speedup: {:.2}x", metrics.acceleration_speedup);
```

### Production Readiness Assessment

```rust
use system_acceleration::ane::compat::integration::production_readiness::*;

// Assess readiness
let checklist = assess_readiness(&system)?;

println!("Production Readiness: {}/7 items complete",
    [checklist.components_integrated,
     checklist.performance_targets_met,
     checklist.health_monitoring_active,
     checklist.resource_management_configured,
     checklist.error_handling_robust,
     checklist.monitoring_integrated,
     checklist.agent_integration_ready]
    .iter().filter(|&&x| x).count());
```

## Thread Safety

- **CoreMLAccelerationSystem**: Thread-safe, can be shared across threads
- **Model handles**: Thread-local storage prevents Send/Sync issues
- **FFI operations**: Safe wrappers prevent memory corruption
- **Resource management**: Atomic operations for concurrent access

## Performance Characteristics

### Latency Expectations

- **ANE-accelerated inference**: 20-100ms depending on model complexity
- **CPU fallback**: 60-300ms depending on model complexity
- **Model loading**: 100-500ms for compiled models
- **System initialization**: 50-200ms for component validation

### Resource Usage

- **Memory per model**: 50-200MB depending on model size
- **Memory per inference**: 1-10MB depending on batch size
- **CPU usage**: Minimal when using ANE acceleration
- **Thread usage**: Single-threaded per inference operation

## Error Handling

### Common Error Scenarios

**Model Loading Failures:**
- Invalid model file format
- Incompatible model version
- Insufficient system resources

**Inference Failures:**
- Invalid input dimensions
- Timeout during execution
- Resource exhaustion

**System Errors:**
- Circuit breaker activation
- Health monitoring failures
- Device compatibility issues

### Recovery Strategies

- Automatic retry with exponential backoff
- Circuit breaker prevents cascading failures
- Graceful degradation to CPU execution
- Resource cleanup on failure

---

*This API reference documents the implemented Core ML acceleration system interfaces. All examples are based on working code and validated through automated testing.*
