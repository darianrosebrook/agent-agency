# ANE Policy Integration Guide

**Date**: 2025-01-XX  
**Status**: Integrated  
**Purpose**: Guide for using the performance policy system in inference

## Overview

The performance policy system has been integrated into the Mistral inference path, enabling automatic optimization of sequence length and backend selection based on task characteristics and benchmark findings.

## Integration Points

### 1. MistralInferenceOptions

The `MistralInferenceOptions` struct now includes policy-aware fields:

```rust
pub struct MistralInferenceOptions {
    // ... existing fields ...
    pub sequence_length: Option<usize>,      // Policy-recommended if None
    pub task_type: Option<TaskType>,         // Auto-detected if None
    pub backend_policy: Option<BackendPolicy>, // Auto-selected if None
}
```

### 2. Automatic Policy Application

The `generate_text` function automatically applies policy recommendations:

```rust
// Policy is automatically applied in generate_text
let response = generate_text(model, &prompt, &options).await?;
```

The function will:
1. Auto-detect task type from input length and max_tokens
2. Apply policy-recommended sequence length
3. Select optimal backend (ANE vs CPU)
4. Log policy decisions for observability

### 3. Manual Policy Application

You can also manually apply policy:

```rust
use system_acceleration::ane::infer::mistral::MistralInferenceOptions;
use system_acceleration::ane::policy::PerformancePolicy;

let mut options = MistralInferenceOptions::default();
options.max_tokens = 200;

// Apply policy based on input length
let options = options.with_policy(input_tokens.len(), None);

// Or with custom policy
let custom_policy = PerformancePolicy::default();
let options = options.with_policy(input_tokens.len(), Some(&custom_policy));
```

## Usage Examples

### Example 1: Low-Latency Task (Tool Call)

```rust
let mut options = MistralInferenceOptions {
    max_tokens: 50,
    ..Default::default()
};

// Policy will auto-detect LowLatency task and recommend:
// - Sequence length: 64 tokens
// - Backend: ANE (1.01x speedup)

let response = generate_text(model, &prompt, &options).await?;
```

### Example 2: Standard Task (General Inference)

```rust
let mut options = MistralInferenceOptions {
    max_tokens: 200,
    ..Default::default()
};

// Policy will auto-detect Standard task and recommend:
// - Sequence length: 256 tokens (optimal)
// - Backend: ANE (1.14x speedup - best performance)

let response = generate_text(model, &prompt, &options).await?;
```

### Example 3: Long-Context Task (Document Analysis)

```rust
let mut options = MistralInferenceOptions {
    max_tokens: 600,
    ..Default::default()
};

// Policy will auto-detect LongContext task and recommend:
// - Sequence length: 512 tokens
// - Backend: ANE (1.12x speedup)

let response = generate_text(model, &prompt, &options).await?;
```

### Example 4: Explicit Override

```rust
use system_acceleration::ane::policy::{TaskType, BackendPolicy};

let mut options = MistralInferenceOptions {
    max_tokens: 200,
    task_type: Some(TaskType::LowLatency),  // Force low-latency
    backend_policy: Some(BackendPolicy::CPU), // Force CPU
    ..Default::default()
};

// Explicit settings override policy recommendations
let response = generate_text(model, &prompt, &options).await?;
```

## Policy Decision Logging

The integration includes automatic logging of policy decisions:

```
Policy decision: task_type=Standard, seq_len=256, backend=ANE, input_len=100
```

This helps with:
- Debugging policy behavior
- Understanding performance characteristics
- Optimizing task type detection

## Backend Selection Logic

The policy automatically selects backend based on sequence length:

| Sequence Length | Backend | Speedup | Rationale |
|----------------|---------|---------|-----------|
| ≤ 64 tokens    | ANE     | 1.01x   | ANE slightly faster |
| 128 tokens     | CPU     | 0.85x   | ❌ ANE 15% slower - avoid |
| ≥ 256 tokens   | ANE     | 1.14x   | ✅ Optimal ANE performance |

## Sequence Length Selection

The policy selects sequence length based on task type:

| Task Type      | Sequence Length | Use Case |
|----------------|-----------------|----------|
| LowLatency     | 64 tokens       | Tool calls, classification, routing |
| Standard       | 256 tokens      | General text generation (optimal) |
| LongContext    | 512 tokens      | Heavy reasoning, document analysis |

## Model Loading Integration

The policy can also be used for model loading:

```rust
use system_acceleration::ane::infer::policy_integration::create_compilation_options_from_policy;

let inference_options = MistralInferenceOptions::default();
let compilation_options = create_compilation_options_from_policy(
    &inference_options,
    input_length,
    None // Use default policy
);

// Load model with policy-recommended compute units
let model = load_mistral_model(&model_path, &compilation_options, telemetry).await?;
```

## Testing

Integration tests verify policy behavior:

```bash
cargo test --package system-acceleration --test policy_integration_test
```

Tests cover:
- Policy recommendations for different task types
- Backend selection logic (especially avoiding 128 tokens)
- Explicit overrides
- Compute units conversion

## Performance Impact

The policy integration adds minimal overhead:
- Policy evaluation: < 1μs (simple enum matching)
- Task type detection: < 1μs (simple heuristics)
- Logging: Optional, can be disabled in production

The performance benefits from optimal sequence length and backend selection far outweigh this overhead.

## Next Steps

1. **Run baseline tests** to verify policy recommendations match benchmark data
2. **Monitor production usage** to refine task type detection heuristics
3. **Extend to other models** (Whisper, YOLO) if applicable
4. **Add runtime adaptation** based on actual performance measurements

## References

- **Policy Implementation**: `iterations/v3/system-acceleration/src/ane/policy.rs`
- **Integration Helpers**: `iterations/v3/system-acceleration/src/ane/infer/policy_integration.rs`
- **Policy Documentation**: `iterations/v3/docs/ANE_SEQUENCE_LENGTH_POLICY.md`
- **Benchmark Data**: `iterations/v3/docs/ANE_PERFORMANCE_INVESTIGATION_REPORT.md`

