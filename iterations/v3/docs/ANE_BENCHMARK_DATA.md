# ANE Performance Benchmark Data

**Date**: 2025-01-XX  
**Model**: Mistral 7B FP16 (StatefulMistral7BInstructFP16.mlpackage.mlmodelc)  
**System**: Apple Silicon (macOS)

## Benchmark Run 1

### Sequence Length Sweep Results

| Sequence Length | CPU Latency (ms) | ANE Latency (ms) | Speedup | ANE Util |
|----------------|------------------|------------------|---------|----------|
| 64 tokens      | 91.82            | 104.93           | 0.88x   | 47.4%    |
| 128 tokens     | 85.41            | 81.97            | **1.04x** | 47.4%    |
| 256 tokens     | 84.48            | 95.89            | 0.88x   | 47.4%    |
| 512 tokens     | 84.87            | 86.31            | 0.98x   | 47.4%    |

**Optimal**: 128 tokens (1.04x speedup)

### Overall Performance

- **CPU**: 98.52ms avg latency, 10.1 IPS throughput
- **ANE**: 92.53ms avg latency, 10.8 IPS throughput
- **ANE Speedup**: 1.1x
- **ANE Dispatch Rate**: 47.4%

## Benchmark Run 2

### Sequence Length Sweep Results

| Sequence Length | CPU Latency (ms) | ANE Latency (ms) | Speedup | ANE Util |
|----------------|------------------|------------------|---------|----------|
| 64 tokens      | 104.08           | 114.97           | 0.91x   | 47.4%    |
| 128 tokens     | 107.49           | 112.02           | 0.96x   | 47.4%    |
| 256 tokens     | 109.94           | 98.71            | **1.11x** | 47.4%    |
| 512 tokens     | 99.38            | 84.74            | **1.17x** | 47.4%    |

**Optimal**: 512 tokens (1.17x speedup)

### Overall Performance

- **CPU**: 87.64ms avg latency, 11.4 IPS throughput
- **ANE**: 83.06ms avg latency, 12.0 IPS throughput
- **ANE Speedup**: 1.1x
- **ANE Dispatch Rate**: 47.4%

## Key Observations

1. **Performance Variability**: Optimal sequence length changes between runs
   - Run 1: 128 tokens optimal (1.04x)
   - Run 2: 512 tokens optimal (1.17x)

2. **ANE Dispatch Rate**: Consistent 47.4% across all tests (below 70% target)

3. **Sequence Length Impact**: 
   - Very short sequences (64 tokens): ANE consistently slower (0.88-0.91x)
   - Medium sequences (128-256 tokens): Variable performance
   - Longer sequences (512 tokens): Best performance in Run 2 (1.17x)

4. **System State Dependency**: Results vary between runs, suggesting:
   - Thermal state differences
   - Model compilation state
   - Background process interference
   - Memory/cache state

## Recommendations

1. **Test multiple sequence lengths** on your system to find optimal
2. **Prefer longer sequences (256-512 tokens)** for better ANE performance
3. **Avoid very short sequences (64 tokens)** where ANE overhead dominates
4. **Monitor system state** (thermal, background processes) during benchmarks
5. **Investigate 47.4% dispatch rate** - why only ~47% of ops use ANE

