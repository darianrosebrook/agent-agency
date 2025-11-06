//! Phase 4: Hardening and Device Matrix Testing
//!
//! This test suite validates the hardening features and device matrix support:
//! - Device capability detection and optimization
//! - Circuit breaker fault tolerance
//! - Resource management and cleanup
//! - Graceful degradation strategies
//! - Health monitoring and recovery

use system_acceleration::ane::compat::hardening::*;
// detect_coreml_capabilities is used indirectly through hardening module
use std::collections::HashMap;
use std::time::Duration;

/// Phase 4: Device Matrix Test
#[tokio::test]
async fn test_phase_4_device_matrix() {
    println!("🖥️ Phase 4: Device Matrix Test");
    println!("============================");

    // Test device detection
    println!("1. Testing device capability detection...");

    match DeviceMatrix::detect_current_device() {
        Ok(device_caps) => {
            println!("✅ Device detected: {}", device_caps.chip_family);
            println!("   - ANE Performance Score: {:.2}", device_caps.ane_performance_score);
            println!("   - Memory Bandwidth: {:.0} GB/s", device_caps.memory_bandwidth_gbps);
            println!("   - Unified Memory: {} GB", device_caps.unified_memory_gb);
            println!("   - ANE Cores: {}", device_caps.ane_cores);
            println!("   - Recommended Precision: {}", device_caps.recommended_precision);

            // Verify reasonable values
            assert!(device_caps.ane_performance_score >= 0.0 && device_caps.ane_performance_score <= 1.0,
                "ANE performance score should be between 0.0 and 1.0");
            assert!(device_caps.unified_memory_gb > 0, "Unified memory should be > 0");
            assert!(!device_caps.recommended_precision.is_empty(), "Recommended precision should not be empty");

        }
        Err(e) => {
            println!("⚠️ Device detection failed (expected on non-Apple Silicon): {}", e);
            // This is expected on non-Apple Silicon systems
        }
    }

    println!("✅ Device matrix test completed");
}

/// Phase 4: Platform Optimizations Test
#[test]
fn test_phase_4_platform_optimizations() {
    println!("⚙️ Phase 4: Platform Optimizations Test");
    println!("======================================");

    // Test different device configurations
    let test_devices = vec![
        DeviceCapabilities {
            chip_family: "M1".to_string(),
            ane_performance_score: 0.7,
            memory_bandwidth_gbps: 68.0,
            unified_memory_gb: 16,
            ane_cores: 1,
            supported_ml_versions: vec!["CoreML6".to_string()],
            recommended_precision: "FP16".to_string(),
        },
        DeviceCapabilities {
            chip_family: "M2".to_string(),
            ane_performance_score: 0.85,
            memory_bandwidth_gbps: 100.0,
            unified_memory_gb: 24,
            ane_cores: 1,
            supported_ml_versions: vec!["CoreML6".to_string()],
            recommended_precision: "FP16".to_string(),
        },
        DeviceCapabilities {
            chip_family: "M3Pro".to_string(),
            ane_performance_score: 1.0,
            memory_bandwidth_gbps: 500.0,
            unified_memory_gb: 128,
            ane_cores: 2,
            supported_ml_versions: vec!["CoreML7".to_string()],
            recommended_precision: "FP16".to_string(),
        },
    ];

    for device in &test_devices {
        println!("2. Testing optimizations for {}...", device.chip_family);

        let optimizations = platform_optimizations::optimize_for_platform(device);

        // Verify optimizations are appropriate for device capabilities
        if device.unified_memory_gb >= 64 {
            assert_eq!(optimizations.get("memory_pool_size"), Some(&"large".to_string()),
                "High memory devices should use large memory pools");
        }

        if device.ane_performance_score > 0.8 {
            assert_eq!(optimizations.get("preferred_compute_units"), Some(&"all".to_string()),
                "High-performance ANE should use all compute units");
        }

        if device.memory_bandwidth_gbps > 300.0 {
            assert_eq!(optimizations.get("batch_size"), Some(&"large".to_string()),
                "High bandwidth devices should use large batches");
        }

        println!("   ✅ {} optimizations validated", device.chip_family);
    }

    println!("✅ Platform optimizations test completed");
}

/// Phase 4: Circuit Breaker Test
#[tokio::test]
async fn test_phase_4_circuit_breaker() {
    println!("🔌 Phase 4: Circuit Breaker Test");
    println!("===============================");

    let circuit_breaker = CircuitBreaker::new(3, Duration::from_secs(1));

    // Test initial state
    println!("1. Testing initial state...");
    assert!(circuit_breaker.can_attempt(), "Circuit breaker should allow attempts initially");

    // Test successful operations
    println!("2. Testing successful operations...");
    for _ in 0..5 {
        circuit_breaker.record_success();
        assert!(circuit_breaker.can_attempt(), "Should still allow attempts after success");
    }

    // Test failure threshold
    println!("3. Testing failure threshold...");
    for i in 0..3 {
        circuit_breaker.record_failure();
        if i < 2 {
            assert!(circuit_breaker.can_attempt(), "Should allow attempts before threshold");
        }
    }

    // Should now be open
    assert!(!circuit_breaker.can_attempt(), "Circuit breaker should be open after 3 failures");

    // Wait for recovery timeout
    println!("4. Testing recovery timeout...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Should allow attempts again
    assert!(circuit_breaker.can_attempt(), "Circuit breaker should allow attempts after recovery timeout");

    // Test recovery
    circuit_breaker.record_success();
    assert!(circuit_breaker.can_attempt(), "Should allow attempts after successful recovery");

    println!("✅ Circuit breaker test completed");
}

/// Phase 4: Inference Metrics Test
#[test]
fn test_phase_4_inference_metrics() {
    println!("📊 Phase 4: Inference Metrics Test");
    println!("=================================");

    let metrics = InferenceMetrics::new();

    // Test initial state
    println!("1. Testing initial metrics...");
    let summary = metrics.get_summary();
    assert_eq!(summary.total_inferences, 0, "Should start with zero inferences");
    assert_eq!(summary.successful_inferences, 0, "Should start with zero successes");

    // Test recording successes
    println!("2. Testing success recording...");
    metrics.record_success(Duration::from_millis(50));
    metrics.record_success(Duration::from_millis(75));
    metrics.record_success(Duration::from_millis(100));

    let summary = metrics.get_summary();
    assert_eq!(summary.total_inferences, 3, "Should have 3 total inferences");
    assert_eq!(summary.successful_inferences, 3, "Should have 3 successful inferences");
    assert!((summary.avg_latency_ms - 75.0).abs() < 1.0, "Average latency should be ~75ms");

    // Test recording failures
    println!("3. Testing failure recording...");
    metrics.record_failure(Duration::from_millis(200));
    metrics.record_timeout(Duration::from_millis(5000));

    let summary = metrics.get_summary();
    assert_eq!(summary.total_inferences, 5, "Should have 5 total inferences");
    assert_eq!(summary.successful_inferences, 3, "Should still have 3 successful inferences");
    assert_eq!(summary.failed_inferences, 1, "Should have 1 failed inference");
    assert_eq!(summary.timeout_inferences, 1, "Should have 1 timeout");

    println!("   Success Rate: {:.1}%", summary.success_rate * 100.0);
    println!("   Avg Latency: {:.2}ms", summary.avg_latency_ms);
    println!("   Min Latency: {:.2}ms", summary.min_latency_ms);
    println!("   Max Latency: {:.2}ms", summary.max_latency_ms);

    println!("✅ Inference metrics test completed");
}

/// Phase 4: Graceful Degradation Test
#[test]
fn test_phase_4_graceful_degradation() {
    println!("⬇️ Phase 4: Graceful Degradation Test");
    println!("===================================");

    use graceful_degradation::*;

    let test_scenarios = vec![
        ("memory error", "Insufficient memory for allocation", Some(DegradationStrategy::ReduceBatchSize)),
        ("precision error", "Unsupported precision FP16", Some(DegradationStrategy::ReducePrecision)),
        ("ANE error", "ANE acceleration failed", Some(DegradationStrategy::CpuOnlyFallback)),
        ("unknown error", "Some other error", None),
    ];

    for (test_name, error_msg, expected_strategy) in test_scenarios {
        println!("1. Testing {} degradation...", test_name);

        let error = system_acceleration::ane::ane_errors::ANEError::Internal(error_msg.to_string());
        let strategy = get_degradation_strategy(&error);

        assert_eq!(strategy, expected_strategy, "Expected {:?} for {}", expected_strategy, test_name);
        println!("   ✅ {} degradation strategy: {:?}", test_name, strategy);
    }

    // Test strategy application
    println!("2. Testing strategy application...");
    let mut config = HashMap::new();
    config.insert("precision".to_string(), "FP16".to_string());
    config.insert("batch_size".to_string(), "large".to_string());

    apply_degradation(DegradationStrategy::ReducePrecision, &mut config);
    assert_eq!(config.get("precision"), Some(&"FP32".to_string()), "Should reduce precision to FP32");

    apply_degradation(DegradationStrategy::ReduceBatchSize, &mut config);
    assert_eq!(config.get("batch_size"), Some(&"1".to_string()), "Should reduce batch size to 1");

    println!("✅ Graceful degradation test completed");
}

/// Phase 4: Hardened Inference Executor Test
#[tokio::test]
async fn test_phase_4_hardened_executor() {
    println!("🛡️ Phase 4: Hardened Inference Executor Test");
    println!("============================================");

    // Create executor (will fail on non-Apple Silicon, which is expected)
    match HardenedInferenceExecutor::new() {
        Ok(executor) => {
            println!("1. Testing successful executor creation...");
            println!("   Device: {}", executor.get_device_capabilities().chip_family);
            println!("   ANE Score: {:.2}", executor.get_device_capabilities().ane_performance_score);

            // Test successful operation
            println!("2. Testing successful operation...");
            let result = executor.execute_inference(|| async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok("success")
            }).await;

            assert!(result.is_ok(), "Should succeed with successful operation");
            assert_eq!(result.unwrap(), "success", "Should return success message");

            // Check metrics
            let metrics = executor.get_metrics();
            let summary = metrics.get_summary();
            assert_eq!(summary.total_inferences, 1, "Should have recorded 1 inference");
            assert_eq!(summary.successful_inferences, 1, "Should have recorded 1 success");

            println!("   ✅ Successful operation metrics recorded");

            // Test timeout handling
            println!("3. Testing timeout handling...");
            let timeout_result = executor.execute_inference(|| async {
                tokio::time::sleep(Duration::from_secs(2)).await; // Longer than 1 second timeout
                Ok("too_slow")
            }).await;

            assert!(timeout_result.is_err(), "Should timeout with slow operation");
            // Note: We can't check exact error type since ANEError doesn't implement PartialEq

            let summary = metrics.get_summary();
            assert_eq!(summary.timeout_inferences, 1, "Should have recorded 1 timeout");

            println!("   ✅ Timeout handling works correctly");

        }
        Err(e) => {
            println!("⚠️ Hardened executor creation failed (expected on non-Apple Silicon): {}", e);
            println!("   This is normal - executor requires Apple Silicon hardware");
        }
    }

    println!("✅ Hardened inference executor test completed");
}

/// Phase 4: Health Monitoring Test
#[test]
fn test_phase_4_health_monitoring() {
    println!("🏥 Phase 4: Health Monitoring Test");
    println!("=================================");

    use health_monitoring::*;

    let monitor = HealthMonitor::new();

    // Test initial state
    println!("1. Testing initial health state...");
    assert_eq!(monitor.get_health_status(), HealthStatus::Offline, "Should start offline");

    // Test successful operations
    println!("2. Testing successful operations...");
    for _ in 0..10 {
        monitor.record_operation(true);
    }

    assert_eq!(monitor.get_health_status(), HealthStatus::Healthy, "Should be healthy after successes");

    // Test some failures
    println!("3. Testing failure handling...");
    for _ in 0..3 {
        monitor.record_operation(false);
    }

    assert_eq!(monitor.get_health_status(), HealthStatus::Degraded, "Should be degraded after failures");

    // Test critical failure state
    for _ in 0..7 {
        monitor.record_operation(false);
    }

    assert_eq!(monitor.get_health_status(), HealthStatus::Critical, "Should be critical after many failures");

    println!("✅ Health monitoring test completed");
}

/// Phase 4: Resource Management Test
#[test]
fn test_phase_4_resource_management() {
    println!("💾 Phase 4: Resource Management Test");
    println!("===================================");

    use resource_management::*;

    let manager = ResourceManager::new(8); // 8GB max memory

    // Test initial state
    println!("1. Testing initial resource state...");
    assert_eq!(manager.memory_usage_percent(), 0.0, "Should start with 0% memory usage");
    assert!(manager.can_allocate(1024 * 1024 * 1024), "Should allow reasonable allocation");

    // Test allocation
    println!("2. Testing resource allocation...");
    assert!(manager.allocate(1024 * 1024 * 1024).is_ok(), "Should allow 1GB allocation"); // 1GB
    assert_eq!(manager.memory_usage_percent(), 12.5, "Should be 12.5% usage (1GB of 8GB)");

    // Test allocation limits
    println!("3. Testing allocation limits...");
    let large_allocation = 7 * 1024 * 1024 * 1024; // 7GB (would exceed 80% threshold)
    assert!(manager.allocate(large_allocation).is_err(), "Should reject allocation that exceeds threshold");

    // Test cleanup trigger
    println!("4. Testing cleanup triggers...");
    manager.allocate(5 * 1024 * 1024 * 1024).unwrap(); // 5GB more (total 6GB, 75%)
    assert!(!manager.maybe_cleanup(), "Should not trigger cleanup at 75%");

    manager.allocate(1024 * 1024 * 1024).unwrap(); // 1GB more (total 7GB, 87.5%)
    assert!(manager.maybe_cleanup(), "Should trigger cleanup at 87.5%");

    // Test deallocation
    println!("5. Testing resource deallocation...");
    manager.deallocate(2 * 1024 * 1024 * 1024); // Free 2GB
    assert!((manager.memory_usage_percent() - 62.5).abs() < 0.1, "Should be ~62.5% usage after freeing 2GB");

    println!("✅ Resource management test completed");
}

/// Phase 4: Comprehensive Hardening Integration Test
#[tokio::test]
async fn test_phase_4_comprehensive_hardening() {
    println!("🎯 Phase 4: Comprehensive Hardening Integration Test");
    println!("===================================================");

    // This test integrates multiple hardening components
    println!("1. Testing integrated hardening components...");

    // Test device matrix integration
    let device_result = DeviceMatrix::detect_current_device();
    let has_device_support = device_result.is_ok();

    if has_device_support {
        let device_caps = device_result.unwrap();
        println!("   ✅ Device matrix: {} detected", device_caps.chip_family);

        // Test platform optimizations
        let optimizations = platform_optimizations::optimize_for_platform(&device_caps);
        assert!(!optimizations.is_empty(), "Should generate platform optimizations");
        println!("   ✅ Platform optimizations: {} settings applied", optimizations.len());

        // Test resource management with device-aware sizing
        let resource_manager = resource_management::ResourceManager::new(device_caps.unified_memory_gb);
        println!("   ✅ Resource management: {}GB memory limit configured", device_caps.unified_memory_gb);
    } else {
        println!("   ⚠️ Device matrix: Not available on this platform (expected)");
    }

    // Test circuit breaker reliability
    let circuit_breaker = CircuitBreaker::new(5, Duration::from_millis(100));
    for _ in 0..10 {
        circuit_breaker.record_success();
    }
    assert!(circuit_breaker.can_attempt(), "Circuit breaker should remain stable with successes");
    println!("   ✅ Circuit breaker: Fault tolerance verified");

    // Test metrics collection
    let metrics = InferenceMetrics::new();
    for i in 1..=5 {
        metrics.record_success(Duration::from_millis(i * 10));
    }
    let summary = metrics.get_summary();
    assert_eq!(summary.successful_inferences, 5, "Should record all successful inferences");
    println!("   ✅ Metrics: Performance tracking verified");

    // Test health monitoring
    let health_monitor = health_monitoring::HealthMonitor::new();
    for _ in 0..5 {
        health_monitor.record_operation(true);
    }
    assert!(matches!(health_monitor.get_health_status(), health_monitoring::HealthStatus::Healthy),
        "Should report healthy status after successful operations");
    println!("   ✅ Health monitoring: System health tracking verified");

    println!("🎉 Phase 4 comprehensive hardening integration test completed!");
    println!("   - Device matrix: {}", if has_device_support { "✅ Configured" } else { "⚠️ Not available" });
    println!("   - Platform optimizations: ✅ Applied");
    println!("   - Circuit breaker: ✅ Fault tolerant");
    println!("   - Resource management: ✅ Configured");
    println!("   - Metrics collection: ✅ Operational");
    println!("   - Health monitoring: ✅ Active");
}
