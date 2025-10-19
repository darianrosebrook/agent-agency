# Phase 4: Hardening & Production Readiness

**Duration:** 2-3 weeks  
**Status:** Planned (Ready to start)  
**Objective:** Production-grade optimization, device matrix validation, Gate D readiness  

---

## Overview

Phase 4 transforms the proven Phase 3B foundation into a production-hardened system:
- Optimize memory and performance
- Automate testing across device matrix
- Validate 1-hour production soak
- Gate D: Deployment readiness

---

## Phase 4 Roadmap

### Week 1: Buffer Pooling & Optimization (Days 1-5)

#### 4.1 MLMultiArray Buffer Pooling

**Objective:** Reduce allocation churn by reusing tensors

```rust
pub struct BufferPool {
    pools: Arc<RwLock<HashMap<(Shape, DType), VecDeque<MLMultiArray>>>>,
    max_buffers_per_shape: usize,
    max_total_buffers: usize,
    stats: Arc<Mutex<PoolStats>>,
}

impl BufferPool {
    pub fn acquire(&self, shape: Shape, dtype: DType) -> Result<MLMultiArray> {
        // Try to reuse existing buffer
        // Fall back to allocation if pool empty
        // Track metrics
    }
    
    pub fn release(&self, buffer: MLMultiArray) -> Result<()> {
        // Return buffer to pool if not full
        // Otherwise drop
        // Update stats
    }
    
    pub fn stats(&self) -> PoolStats {
        // Hit ratio, allocations avoided, memory saved
    }
}
```

**Metrics to Track:**
- Buffer hit ratio (target >85%)
- Allocations avoided (target >90% after warmup)
- Memory saved (target 5-15MB for typical workload)
- Pool fragmentation

**Tests:**
- [ ] Pool creation and configuration
- [ ] Acquire/release cycle correctness
- [ ] Hit ratio under various shapes
- [ ] Memory deallocation on drop
- [ ] Concurrent access safety
- [ ] Pool saturation handling

#### 4.2 MLModel Instance Pooling

**Objective:** Maintain 2-4 compiled models for concurrent inference

```rust
pub struct ModelPool {
    models: Arc<RwLock<VecDeque<Arc<CoreMLModel>>>>,
    config_hash: String,
    max_instances: usize,
    stats: Arc<Mutex<PoolStats>>,
}

impl ModelPool {
    pub async fn acquire(&self) -> Result<PooledModelGuard> {
        // Wait if all instances busy (with timeout)
        // Reuse existing model
        // Fall back to compile+load if needed
    }
    
    pub fn release(&self, model: PooledModelGuard) {
        // Return to pool (auto on drop)
        // Update stats
    }
}
```

**Configuration:**
- M1/M2: 2-3 instances
- M3/M4: 3-4 instances
- Fallback: 1 instance

**Tests:**
- [ ] Concurrent model access
- [ ] Instance reuse correctness
- [ ] Contention under high load
- [ ] Timeout behavior when all busy
- [ ] Memory efficiency

#### 4.3 Mmap I/O for Large Outputs

**Objective:** Zero-copy for large tensor outputs (images, video frames)

```rust
pub struct LargeOutputHandler {
    mmap_dir: PathBuf,
    max_inline_size: usize,  // Default 10MB
    cleanup_threshold: usize, // Free old files
}

impl LargeOutputHandler {
    pub fn write_output(&self, tensor: &TensorMap) -> Result<OutputRef> {
        // Inline small tensors (<10MB) in result
        // Mmap large tensors to temp file
        // Return reference with lifetime guard
    }
}
```

**Tests:**
- [ ] Small tensors inline (no mmap)
- [ ] Large tensors mmapped
- [ ] Cleanup on drop
- [ ] Cross-process safety
- [ ] Performance validation

### Week 2: Device Matrix Automation (Days 6-10)

#### 4.4 Device Detection & Reporting

**Objective:** Characterize hardware capabilities

```rust
pub struct DeviceProfile {
    mac_model: String,        // "M1", "M2", "M3", "M4"
    macos_version: String,    // "14.6", "15.0", etc
    cpu_count: usize,
    gpu_count: usize,
    neural_engine_capable: bool,
    thermal_sensors: bool,
}

impl DeviceProfile {
    pub async fn detect() -> Result<Self> {
        // Use sysctl, system_profiler, etc
        // Cache result
    }
    
    pub fn ane_available(&self) -> bool {
        // M1+ have ANE
    }
}
```

**Implementation:**
- [ ] Detect Mac model from `sysctl hw.product`
- [ ] Get macOS version from `sw_vers`
- [ ] Query ANE availability
- [ ] Report in telemetry

#### 4.5 Device Matrix Test Harness

**Objective:** Automate testing on M1, M2, M3

```rust
#[cfg(test)]
mod device_matrix {
    #[test]
    #[ignore = "requires specific hardware"]
    fn test_fastvit_m1() {
        let profile = DeviceProfile::detect().unwrap();
        assert_eq!(profile.mac_model, "M1");
        // Run inference, collect metrics
        // Assert against M1-specific baselines
    }
    
    #[test]
    #[ignore = "requires specific hardware"]
    fn test_fastvit_m2() {
        // Similar for M2
    }
    
    #[test]
    #[ignore = "requires specific hardware"]
    fn test_fastvit_m3() {
        // Similar for M3
    }
}
```

**Matrix:**
- M1, M2, M3 (one CI runner per device OR manual)
- macOS 14.x, 15.x
- FastViT T8, ResNet-50 (stretch)

**Metrics per device:**
- Speedup (baseline: CPU)
- ANE dispatch %
- P99 latency
- Memory growth
- Compilation time

#### 4.6 Soak Test Infrastructure

**Objective:** Run 1+ hour continuous inference

```rust
#[tokio::test]
#[ignore = "long-running soak test"]
async fn test_core_ml_1hour_soak() {
    let backend = CoreMLBackend::new();
    let model = backend.prepare(
        &ModelArtifact::from_path(FASTVIT_PATH),
        PrepareOptions::default(),
    ).await?;
    
    let start = Instant::now();
    let mut metrics = SoakMetrics::new();
    
    while start.elapsed() < Duration::from_secs(3600) {
        let inference_start = Instant::now();
        
        let result = backend.infer(&model, &inputs, Duration::from_secs(5)).await;
        
        metrics.record(result, inference_start.elapsed());
        
        // Every 5 minutes: report metrics
        if metrics.inference_count % 1000 == 0 {
            println!("{}", metrics.summary());
        }
    }
    
    // Validate soak results
    assert!(metrics.success_rate > 0.99);
    assert!(metrics.memory_growth_mb < 50);
    assert!(metrics.p99_latency_ms < 25);
    assert!(metrics.circuit_breaker_trips == 0);
}
```

**Success Criteria (Gate D):**
- ✅ 1+ hour continuous inference
- ✅ >99% success rate
- ✅ <50MB memory growth
- ✅ P99 latency stable (<25ms)
- ✅ Zero circuit breaker trips
- ✅ Zero panics/crashes

### Week 3: Integration & Validation (Days 11-15)

#### 4.7 Performance Profiling with Instruments

**Objective:** Verify optimization effectiveness

```bash
# Record 5-minute profile
sudo xctrace record \
  -d /tmp/coreml_profile.trace \
  --time-limit 5m \
  --attach "target_binary" \
  System Trace

# Analyze:
# - ANE utilization
# - Memory allocations
# - Thread scheduling
# - Power consumption
```

**Key Metrics:**
- ANE busy time %
- GPU busy time %
- Memory pressure
- Thermal state
- Battery impact (if applicable)

#### 4.8 Regression Testing

**Objective:** Ensure Phase 3 properties still hold

```rust
#[test]
fn test_phase3_properties_still_valid() {
    // Re-run Phase 3 tests with Phase 4 code
    assert_eq!(speedup, 2.84, "Speedup regression");
    assert_eq!(ane_dispatch, 78.5, "ANE dispatch regression");
    assert_eq!(p99_latency, 18, "P99 regression");
    assert_eq!(memory_growth, 6, "Memory regression");
}
```

#### 4.9 Gate D Checklist

**Verification:**
- [ ] Buffer pool reduces allocations >90% (after warmup)
- [ ] Model pool handles 4 concurrent inferences
- [ ] Mmap works for >10MB outputs
- [ ] Device matrix tests pass on M1/M2/M3
- [ ] Soak test: 1 hour, 0 failures
- [ ] Memory stable (<50MB growth)
- [ ] P99 consistent (<25ms)
- [ ] Circuit breaker never trips
- [ ] Numeric parity maintained
- [ ] No regressions from Phase 3

---

## Implementation Schedule

| Week | Task | Effort | Owner | Status |
|------|------|--------|-------|--------|
| 1 | Buffer pooling | 3-4 days | TBD | Planned |
| 1 | Model pooling | 1-2 days | TBD | Planned |
| 1 | Mmap I/O | 1 day | TBD | Planned |
| 2 | Device detection | 1 day | TBD | Planned |
| 2 | Test harness | 2 days | TBD | Planned |
| 2 | Soak test infra | 2 days | TBD | Planned |
| 3 | Profiling & optimization | 3 days | TBD | Planned |
| 3 | Integration & validation | 2 days | TBD | Planned |

---

## Success Criteria (Gate D)

### Performance
- ✅ ANE speedup maintained: ≥2.8x
- ✅ ANE dispatch stable: ≥70%
- ✅ P99 latency: <25ms (5ms margin)
- ✅ No regressions from Phase 3

### Reliability
- ✅ 1-hour soak: 0 failures
- ✅ Memory growth: <50MB
- ✅ Circuit breaker: Never trips
- ✅ Numeric parity: Maintained

### Code Quality
- ✅ >95% test coverage (Phase 4 code)
- ✅ Zero panics
- ✅ Zero memory leaks
- ✅ Thread-safe all components

### Device Validation
- ✅ M1: Baseline metrics captured
- ✅ M2: Metrics captured
- ✅ M3: Metrics captured
- ✅ Consistency validation passed

---

## Deliverables

### Code
- [ ] `apple-silicon/src/buffer_pool.rs` (300+ lines)
- [ ] `apple-silicon/src/model_pool.rs` (200+ lines)
- [ ] `apple-silicon/src/large_output.rs` (150+ lines)
- [ ] Device detection module (100+ lines)
- [ ] Updated telemetry with pool metrics

### Tests
- [ ] Buffer pool tests (15+ tests)
- [ ] Model pool tests (10+ tests)
- [ ] Mmap I/O tests (8+ tests)
- [ ] Device matrix tests (12+ tests)
- [ ] 1-hour soak test
- [ ] Regression tests

### Documentation
- [ ] Phase 4 Implementation Guide
- [ ] Device Matrix Results Report
- [ ] Soak Test Analysis
- [ ] Performance Optimization Report
- [ ] Gate D Verdict

### Metrics
- [ ] Device-specific baselines (M1/M2/M3)
- [ ] Buffer pool efficiency report
- [ ] Memory optimization analysis
- [ ] Thermal profile under load
- [ ] Power consumption estimate

---

## Notes

### Known Risks
- Device matrix testing requires physical hardware (M1, M2, M3 macs)
- Soak test is time-consuming (1 hour minimum)
- Thermal behavior varies by ambient conditions
- ANE availability may vary across macOS versions

### Mitigation
- Use local machines for device matrix (solo dev)
- Run soak tests during off-hours
- Document thermal baseline per device
- Test on stable macOS versions (current & current-1)

### Future Optimization Opportunities
- Precompiled model cache (avoid runtime compilation)
- Multi-model batching
- Dynamic batch sizing
- Thermal-aware throttling
- Alternative backends (Metal, MPSGraph)

---

## Resources

### Tools Needed
- Instruments.app (Xcode)
- System Profiler
- sysctl / sw_vers
- Custom telemetry system

### Reference Models
- FastViT T8 F16 (primary validation)
- ResNet-50 (optional stretch)
- DETR (optional stretch)

### Documentation
- See `PHASE_3B_GATE_C_REPORT.md` for Phase 3 baseline
- See `coreml-impl.plan.md` for overall architecture
- See `CORE_ML_IMPLEMENTATION_PATH.md` for design decisions

---

**Phase 4 Ready: Start when Phase 3B complete ✅**

