# ANE Sequence Length Policy

**Date**: 2025-01-XX  
**Status**: Implemented  
**Purpose**: Adaptive sequence length and backend selection based on benchmark findings

## Overview

This policy system implements adaptive sequence length and backend selection based on actual benchmark data from the ANE Performance Investigation. Instead of hardcoding a single "optimal" sequence length, the system adapts to task type and workload characteristics.

## Performance Characteristics

Based on Mistral 7B FP16 benchmarks on Apple Silicon:

| Sequence Length | CPU (ms) | ANE (ms) | Speedup | Recommendation |
|----------------|----------|----------|---------|----------------|
| 64 tokens      | 86.80    | 85.97    | 1.01x   | ✅ ANE slightly faster |
| 128 tokens     | 83.38    | 98.03    | 0.85x   | ❌ **Avoid** - CPU 15% faster |
| 256 tokens     | 99.38    | 87.04    | 1.14x   | ✅ **Optimal** - Best ANE speedup |
| 512 tokens     | 94.64    | 84.36    | 1.12x   | ✅ Good - ANE faster |

## Break-Even Points

- **ANE outperforms CPU**: 
  - seq ≤ 64 tokens (1.01x speedup)
  - seq ≥ 256 tokens (1.14x at 256, 1.12x at 512)
- **Neutral zone**: 
  - ~128 tokens (0.85x - CPU wins, avoid ANE)
- **Critical finding**: 
  - 128 tokens is the worst case for ANE (15% slower than CPU)

## Policy Strategy

### Task Types

1. **Low-Latency Tasks** (`TaskType::LowLatency`)
   - Use cases: Tool calls, classification, routing, quick responses
   - Sequence length: 64 tokens
   - Backend: ANE (1.01x speedup)
   - Rationale: Fast response time, slight ANE advantage

2. **Standard Tasks** (`TaskType::Standard`)
   - Use cases: General text generation, standard inference
   - Sequence length: 256 tokens (optimal)
   - Backend: ANE (1.14x speedup - best performance)
   - Rationale: Best ANE performance, good context window

3. **Long-Context Tasks** (`TaskType::LongContext`)
   - Use cases: Heavy reasoning, document analysis, long-form generation
   - Sequence length: 512 tokens
   - Backend: ANE (1.12x speedup) or CPU if needed
   - Rationale: Larger context window, still good ANE performance

### Backend Selection

The policy automatically selects backend based on sequence length:

- **seq ≤ 64**: ANE (1.01x speedup)
- **seq = 128**: CPU (0.85x - ANE slower, avoid)
- **seq ≥ 256**: ANE (1.14x at 256, 1.12x at 512)

## Usage

### Basic Usage

```rust
use system_acceleration::ane::policy::{PerformancePolicy, TaskType};

// Create policy with defaults
let policy = PerformancePolicy::default();

// Get optimal configuration for a task type
let (seq_len, backend) = policy.optimal_config(TaskType::Standard);
// Returns: (256, BackendPolicy::ANE)

// Get recommended sequence length
let seq_len = policy.recommended_sequence_length(TaskType::LowLatency);
// Returns: 64

// Get recommended backend for a sequence length
let backend = policy.recommended_backend(256);
// Returns: BackendPolicy::ANE
```

### Task Type Detection

```rust
use system_acceleration::ane::policy::TaskType;

// Automatically detect task type from input characteristics
let task_type = TaskType::from_input(input_length, max_tokens);

// Or explicitly set
let task_type = TaskType::LowLatency;
```

### Performance Characteristics

```rust
// Get benchmark data for a sequence length
if let Some((cpu_ms, ane_ms, speedup, recommendation)) = 
    policy.performance_characteristics(256) {
    println!("CPU: {:.2}ms, ANE: {:.2}ms, Speedup: {:.2}x", 
        cpu_ms, ane_ms, speedup);
    println!("Recommendation: {}", recommendation);
}
```

## Configuration

### Default Policy

```rust
PerformancePolicy {
    sequence_length: SequenceLengthPolicy {
        default: 256,      // Optimal ANE performance
        low_latency: 64,  // Fast response
        long_context: 512, // Larger context
        min: 32,           // Safety minimum
        max: 1024,         // Safety maximum
    },
    backend: BackendPolicy::Auto, // Adaptive selection
    adaptive: true,                // Enable adaptive mode
}
```

### Custom Policy

```rust
use system_acceleration::ane::policy::*;

let policy = PerformancePolicy {
    sequence_length: SequenceLengthPolicy {
        default: 256,
        low_latency: 64,
        long_context: 512,
        min: 32,
        max: 1024,
    },
    backend: BackendPolicy::ANE, // Force ANE
    adaptive: false,              // Disable adaptive selection
};
```

## Integration with Inference

The policy should be integrated into inference paths:

```rust
// In inference function
let policy = PerformancePolicy::default();
let task_type = TaskType::from_input(input_tokens.len(), max_tokens);
let (seq_len, backend) = policy.optimal_config(task_type);

// Use seq_len and backend for inference
match backend {
    BackendPolicy::ANE => {
        // Use ANE model with seq_len
    }
    BackendPolicy::CPU => {
        // Use CPU model with seq_len
    }
    BackendPolicy::Auto => {
        // Already determined by policy
    }
}
```

## Constitutional Requirement

The constitutional "local high-performance" requirement is now framed as:

> Must achieve ≤X ms latency and ≥Y tokens/sec locally; choice of CPU/ANE is allowed
> to vary by sequence length and request type.

This replaces the previous requirement of "must always use ANE" with a more flexible,
performance-based approach.

## Future Enhancements

1. **Runtime Adaptation**: Monitor actual performance and adjust policy dynamically
2. **Model-Specific Policies**: Different policies for different models
3. **Quantization Support**: Policies for INT8/FP16 variants
4. **Thermal Awareness**: Adjust policy based on thermal state
5. **Concurrency Awareness**: Adjust policy based on concurrent load

## References

- **Investigation Report**: `iterations/v3/docs/ANE_PERFORMANCE_INVESTIGATION_REPORT.md`
- **Expert Analysis**: `iterations/v3/docs/testing/ANE_INVESTIGATION.md`
- **Implementation**: `iterations/v3/system-acceleration/src/ane/policy.rs`

