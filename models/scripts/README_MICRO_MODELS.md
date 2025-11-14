# Micro-Model Baselines for ANE Performance Testing

## Purpose

Micro-models are small, ANE-friendly models used to establish baseline performance. They help separate:

- **Platform performance** (CoreML+ANE as a platform)
- **Model-specific performance** (Mistral 7B converted to CoreML)

## Models Created

### 1. Micro Dense Layer
- **Architecture**: Single linear layer (matmul) with GELU activation
- **Size**: Hidden dimension 4096 (matches Mistral 7B)
- **Input**: `[batch=1, seq_len=128, hidden=4096]`
- **Purpose**: Tests pure matrix multiplication on ANE

### 2. Micro Attention Block
- **Architecture**: Single self-attention block with layer norm
- **Size**: Hidden dimension 4096, 32 attention heads
- **Input**: `[batch=1, seq_len=128, hidden=4096]`
- **Purpose**: Tests attention operations (QK^T, softmax, V projection) on ANE

## Creating Micro-Models

### Prerequisites

```bash
pip install coremltools torch
```

### Generate Models

```bash
cd models/scripts
python create_micro_models.py
```

This will create:
- `models/coreml/micro/micro_dense_layer.mlpackage`
- `models/coreml/micro/micro_attention_block.mlpackage`

## Running Benchmarks

### Run All Tests (Including Micro-Models)

```bash
cd iterations/v3
cargo test --test ane_performance_benchmarks -- --nocapture
```

The benchmark will:
1. **Test micro-models first** (ANE baseline sanity check)
2. **Interpret results**:
   - ✅ 2-3x speedup → Runtime path is fine, limitation is Mistral 7B architecture
   - ⚠️ ~1.1x speedup → This is what ANE vs CPU looks like for FP16 workloads
   - ❌ <1x speedup → Something is wrong with conversion or CoreML mapping
3. **Test full models** (production workloads like Mistral 7B)

### Expected Results

Based on expert analysis:

- **If micro-models show 2-3x speedup**:
  - Runtime path is fine
  - Limitation is Mistral 7B architecture / CoreML op support
  - Focus optimization on model conversion and graph partitioning

- **If micro-models show ~1.1x speedup**:
  - This is what ANE vs CPU looks like for FP16 workloads on this chip
  - Platform limit, not a bug
  - Consider quantization (INT8) or smaller models

- **If micro-models show <1x speedup**:
  - Something is wrong with conversion or CoreML mapping
  - Check model conversion process
  - Verify ANE compute units are being used

## Integration with Investigation

These micro-models are part of **Step 4: Micro-Model Baselines** from the ANE Performance Investigation:

- See: `iterations/v3/docs/testing/ANE_INVESTIGATION.md`
- Report: `iterations/v3/docs/ANE_PERFORMANCE_INVESTIGATION_REPORT.md`

## Troubleshooting

### Models Not Found

If benchmarks don't find micro-models:

1. Check models exist:
   ```bash
   ls -la models/coreml/micro/*.mlpackage.mlmodelc
   ```

2. Verify path in benchmark:
   - Benchmark looks in: `models/coreml/micro/`
   - Models should be: `micro_dense_layer.mlpackage.mlmodelc` and `micro_attention_block.mlpackage.mlmodelc`

### Conversion Errors

If `create_micro_models.py` fails:

1. Check CoreML Tools version:
   ```bash
   pip show coremltools
   ```
   - Requires: macOS 13+ deployment target support

2. Check PyTorch version:
   ```bash
   pip show torch
   ```

3. Verify Apple Silicon:
   - Micro-models require Apple Silicon Mac
   - ANE acceleration only available on M1/M2/M3 chips

## Next Steps

After running micro-model baselines:

1. **Compare results** with Mistral 7B performance
2. **Update investigation report** with findings
3. **Decide optimization strategy**:
   - If 2-3x → Focus on model conversion
   - If ~1.1x → Accept platform limits or try quantization
   - If <1x → Debug conversion/mapping issues

