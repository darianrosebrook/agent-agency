# ANE Performance Comparison: Previous vs Current Run

**Date**: 2025-01-XX  
**Comparison**: Previous benchmark run vs current benchmark run

## Executive Summary

**Verdict: INCONSISTENT - Performance pattern reversed, optimal configuration changed**

The current run shows **opposite performance characteristics** compared to the previous run:
- **Previous**: Smaller sequences (64 tokens) optimal (1.16x speedup)
- **Current**: Larger sequences (256 tokens) optimal (1.14x speedup)
- **Pattern reversal**: Performance characteristics completely inverted

## Detailed Comparison

### Sequence Length Performance Comparison

| Sequence Length | Previous Run | Current Run | Change | Status |
|----------------|-------------|-------------|--------|--------|
| **64 tokens** | CPU 80.45ms, ANE 69.58ms, **1.16x** ✅ | CPU 86.80ms, ANE 85.97ms, **1.01x** ✅ | **-0.15x** | ⚠️ **WORSE** - Speedup decreased |
| **128 tokens** | CPU 75.55ms, ANE 75.29ms, **1.00x** | CPU 83.38ms, ANE 98.03ms, **0.85x** ❌ | **-0.15x** | ❌ **WORSE** - Now slower than CPU |
| **256 tokens** | CPU 79.68ms, ANE 74.76ms, **1.07x** ✅ | CPU 99.38ms, ANE 87.04ms, **1.14x** ✅ | **+0.07x** | ✅ **BETTER** - Speedup increased |
| **512 tokens** | CPU 73.60ms, ANE 80.35ms, **0.92x** ❌ | CPU 94.64ms, ANE 84.36ms, **1.12x** ✅ | **+0.20x** | ✅ **BETTER** - Reversed from slower to faster |

### Optimal Configuration Comparison

| Metric | Previous Run | Current Run | Change |
|--------|-------------|-------------|--------|
| **Optimal Sequence Length** | 64 tokens | 256 tokens | **Changed** |
| **Best Speedup** | 1.16x (64 tokens) | 1.14x (256 tokens) | **-0.02x** (slightly worse) |
| **Worst Speedup** | 0.92x (512 tokens) | 0.85x (128 tokens) | **-0.07x** (worse) |
| **Performance Pattern** | Smaller sequences better | Larger sequences better | **Reversed** |

### Default Configuration (128 tokens) Comparison

| Metric | Previous Run | Current Run | Change |
|--------|-------------|-------------|--------|
| **CPU Latency** | 73.75ms | 81.35ms | **+7.6ms** (10% slower) |
| **ANE Latency** | 75.02ms | 87.78ms | **+12.76ms** (17% slower) |
| **Speedup** | 0.98x | 0.93x | **-0.05x** (worse) |
| **Status** | Slightly slower | Slower | ❌ **WORSE** |

### ANE Utilization Comparison

| Metric | Previous Run | Current Run | Change |
|--------|-------------|-------------|--------|
| **ANE Dispatch Rate** | 47.4% | 47.4% | **Same** ✅ |
| **Consistency** | Consistent across lengths | Consistent across lengths | **Same** ✅ |

## Key Findings

### 1. Performance Pattern Reversal (CRITICAL)

**Previous Run Pattern:**
- Smaller sequences favor ANE: 64 tokens (1.16x) > 256 tokens (1.07x) > 128 tokens (1.00x) > 512 tokens (0.92x)
- ANE performance degrades at larger sequences
- CPU performance improves at larger sequences

**Current Run Pattern:**
- Larger sequences favor ANE: 256 tokens (1.14x) > 512 tokens (1.12x) > 64 tokens (1.01x) > 128 tokens (0.85x)
- ANE performance improves at larger sequences
- CPU performance degrades at larger sequences

**Analysis**: This complete reversal suggests:
- **Non-deterministic behavior** in CoreML/ANE execution
- **System state dependency** (thermal, power, background processes)
- **Model compilation/optimization state** may vary between runs
- **Cache effects** or warmup differences

### 2. Optimal Configuration Changed

**Previous Run:**
- Optimal: 64 tokens (1.16x speedup)
- Recommendation: Use 64 tokens for best performance

**Current Run:**
- Optimal: 256 tokens (1.14x speedup)
- Recommendation: Use 256 tokens for best performance

**Analysis**: The optimal configuration is **not stable** across runs, making it difficult to recommend a single default.

### 3. 128-Token Configuration Degraded

**Previous Run:**
- 128 tokens: 1.00x speedup (tied with CPU)

**Current Run:**
- 128 tokens: 0.85x speedup (ANE 15% slower)

**Analysis**: The default configuration (128 tokens) is **unstable** and can be either tied or significantly slower than CPU.

### 4. ANE Utilization Consistent

**Both Runs:**
- ANE dispatch rate: 47.4% (consistent)
- Same across all sequence lengths

**Analysis**: Dispatch rate is **stable** but below target (70%), indicating consistent hybrid execution pattern.

## Performance Stability Analysis

### Metrics That Are Consistent

1. ✅ **ANE Dispatch Rate**: 47.4% in both runs
2. ✅ **Relative performance ranking**: Some consistency in which lengths perform better/worse
3. ✅ **ANE utilization consistency**: Same across sequence lengths in both runs

### Metrics That Are Inconsistent

1. ❌ **Optimal sequence length**: 64 tokens → 256 tokens
2. ❌ **Performance pattern**: Smaller sequences better → Larger sequences better
3. ❌ **Absolute latency values**: Significant variation (5-20ms differences)
4. ❌ **Speedup values**: 0.15x-0.20x variation at same sequence lengths
5. ❌ **Default configuration performance**: 0.98x → 0.93x

## Root Cause Hypotheses

### Hypothesis 1: System State Dependency

**Evidence:**
- Different thermal states between runs
- Different power management states
- Background processes affecting ANE availability
- System load variations

**Test**: Run benchmarks multiple times in controlled conditions (same thermal state, minimal background processes)

### Hypothesis 2: Model Compilation State

**Evidence:**
- CoreML may optimize graph differently between runs
- First-run compilation effects
- Cache state differences

**Test**: Run benchmarks with explicit model reloading and compilation state control

### Hypothesis 3: Non-Deterministic Graph Partitioning

**Evidence:**
- CoreML may partition graph differently between runs
- ANE vs CPU op assignment may vary
- Dynamic optimization decisions

**Test**: Profile with Instruments to see actual op placement in both runs

### Hypothesis 4: Measurement Variability

**Evidence:**
- Benchmark timing may have variability
- Warmup effects
- System jitter

**Test**: Run multiple iterations and calculate confidence intervals

## Recommendations

### Immediate Actions

1. **Run Multiple Benchmark Iterations**:
   - Execute 5-10 benchmark runs
   - Calculate mean, std dev, and confidence intervals
   - Identify if variation is within expected range or indicates instability

2. **Control System State**:
   - Run benchmarks with minimal background processes
   - Monitor and log thermal state, power consumption
   - Ensure consistent system conditions

3. **Profile with Instruments**:
   - Use Xcode Instruments Core ML template
   - Compare op placement between runs
   - Identify if graph partitioning varies

### Short-Term Improvements

1. **Dynamic Sequence Length Selection**:
   - Instead of fixed default, measure performance at runtime
   - Select optimal sequence length based on current system state
   - Cache optimal configuration per system state

2. **Performance Monitoring**:
   - Track performance over time
   - Alert on significant degradation
   - Maintain performance history

3. **Hybrid Strategy with Fallback**:
   - Start with ANE at optimal sequence length
   - Monitor actual performance
   - Fall back to CPU if ANE underperforms

### Long-Term Solutions

1. **Stabilize ANE Performance**:
   - Investigate root cause of non-deterministic behavior
   - Work with CoreML/Apple to understand variability
   - Consider model recompilation with fixed optimizations

2. **Performance Requirements Reframing**:
   - Accept that ANE performance may vary
   - Set performance targets with confidence intervals
   - Allow CPU fallback when ANE underperforms

## Conclusion

**Verdict: INCONSISTENT - Performance characteristics reversed between runs**

The comparison reveals **significant inconsistency** in ANE performance:
- Optimal sequence length changed (64 → 256 tokens)
- Performance pattern completely reversed
- Default configuration degraded (0.98x → 0.93x)
- Absolute latency values vary significantly

**Key Takeaway**: ANE performance is **not stable** across runs, making it difficult to recommend a single optimal configuration. The system requires:
1. Multiple benchmark iterations to establish confidence
2. Runtime performance monitoring
3. Dynamic configuration selection
4. CPU fallback strategy when ANE underperforms

**ANE Dispatch Rate**: Consistent at 47.4% (below 70% target), indicating stable hybrid execution pattern but suboptimal ANE utilization.



