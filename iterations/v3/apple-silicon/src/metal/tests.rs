//! Tests for Metal GPU acceleration module

#[test]
fn cpu_fallback_kernel_behaves() {
    fn transform(x: f32) -> f32 {
        let v = (x * 0.1).tanh();
        v * v + 0.5
    }

    let input = vec![-3.0, -1.0, 0.0, 1.0, 3.0];
    let out: Vec<f32> = input.iter().map(|x| transform(*x)).collect();

    // All outputs should be in reasonable range [0.5, 1.0]
    for &val in &out {
        assert!(val >= 0.5 && val <= 1.0, "Value {} out of range", val);
    }

    // Test with CpuGPUStub
    let mut stub = CpuGPUStub::new();
    futures::executor::block_on(stub.initialize_blocking()).unwrap();
    assert!(stub.is_ready());
    let result = futures::executor::block_on(stub.run_inference_blocking(&input)).unwrap();
    assert_eq!(result.len(), input.len());

    // Results should be approximately equal
    for (expected, actual) in out.iter().zip(result.iter()) {
        assert!((expected - actual).abs() < 1e-6, "Expected {}, got {}", expected, actual);
    }
}

#[test]
fn gpu_performance_snapshot_structure() {
    let snapshot = GPUPerformanceSnapshot {
        device_name: "Test GPU".to_string(),
        utilization_percent: 75.5,
        memory_used_mb: 1024.0,
        memory_total_mb: 8192.0,
        temperature_celsius: 65.0,
        power_watts: 45.0,
        active_kernels: 3,
        avg_kernel_time_ms: 12.5,
        ts_utc: chrono::Utc::now(),
    };

    // Basic structure validation
    assert_eq!(snapshot.device_name, "Test GPU");
    assert_eq!(snapshot.utilization_percent, 75.5);
    assert_eq!(snapshot.memory_used_mb, 1024.0);
    assert_eq!(snapshot.memory_total_mb, 8192.0);
    assert_eq!(snapshot.temperature_celsius, 65.0);
    assert_eq!(snapshot.power_watts, 45.0);
    assert_eq!(snapshot.active_kernels, 3);
    assert_eq!(snapshot.avg_kernel_time_ms, 12.5);
}
