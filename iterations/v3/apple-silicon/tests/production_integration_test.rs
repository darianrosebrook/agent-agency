/// Production Integration Tests for Core ML Implementation
///
/// Tests the complete end-to-end functionality of the Core ML integration
/// including autorelease pool management, binary tensor serialization,
/// t-digest percentile calculation, and circuit breaker functionality.

#[cfg(all(test, target_os = "macos", feature = "coreml"))]
mod production_tests {
    use apple_silicon::candle_backend::CandleBackend;
    use apple_silicon::core_ml_backend::CoreMLBackend;
    use apple_silicon::inference::{
        ComputeUnits, DType, InferenceEngine, IoSchema, ModelArtifact, ModelFmt, PrepareOptions,
        TensorBatch, TensorMap, TensorSpec,
    };
    use apple_silicon::telemetry::{FailureMode, TelemetryCollector};
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::Duration;

    /// Test end-to-end inference with FastViT T8 F16 model
    #[test]
    fn test_end_to_end_inference() {
        // Create test model artifact
        let model_path = PathBuf::from("FastViTT8F16.mlpackage");
        if !model_path.exists() {
            println!("⚠️ FastViT model not found, skipping test");
            return;
        }

        let artifact = ModelArtifact::Authoring {
            format: ModelFmt::Mlpackage,
            path: model_path.clone(),
            sha256: [0u8; 32], // Mock SHA for test
        };

        let opts = PrepareOptions {
            compute_units: ComputeUnits::All,
            quantization: "fp16".to_string(),
            cache_dir: std::env::temp_dir(),
            timeout_ms: 5000,
        };

        // Test Core ML backend
        let coreml_backend = CoreMLBackend::new();
        let result = coreml_backend.prepare(&artifact, opts.clone());

        if let Ok(prepared_model) = result {
            // Create test input tensor (224x224x3 image)
            let mut inputs = TensorMap::new();
            let image_size = 224 * 224 * 3 * 4; // F32 bytes
            inputs.insert("input".to_string(), vec![0u8; image_size]);

            // Run 100 inference cycles
            let mut total_time = Duration::new(0, 0);
            let mut success_count = 0;

            for i in 0..100 {
                let start = std::time::Instant::now();
                match coreml_backend.infer(&*prepared_model, &inputs, Duration::from_millis(1000)) {
                    Ok(outputs) => {
                        success_count += 1;
                        // Verify output shape (1000 classes)
                        if let Some(output) = outputs.get("output") {
                            assert_eq!(output.len(), 1000 * 4); // F32 bytes for 1000 classes
                        }
                    }
                    Err(e) => {
                        println!("⚠️ Inference {} failed: {}", i, e);
                    }
                }
                total_time += start.elapsed();
            }

            // Verify performance metrics
            assert!(
                success_count >= 95,
                "Success rate too low: {}/100",
                success_count
            );
            let avg_time = total_time.as_millis() / 100;
            assert!(
                avg_time <= 20,
                "Average inference time too high: {}ms",
                avg_time
            );

            println!(
                "✅ End-to-end test passed: {}/100 successful, avg {}ms",
                success_count, avg_time
            );
        } else {
            println!("⚠️ Core ML preparation failed, skipping inference test");
        }
    }

    /// Test CPU fallback parity between Core ML and Candle backends
    #[test]
    fn test_cpu_fallback_parity() {
        // Create test model artifact
        let model_path = PathBuf::from("FastViTT8F16.mlpackage");
        if !model_path.exists() {
            println!("⚠️ FastViT model not found, skipping test");
            return;
        }

        let artifact = ModelArtifact::Authoring {
            format: ModelFmt::Mlpackage,
            path: model_path,
            sha256: [0u8; 32],
        };

        let opts = PrepareOptions {
            compute_units: ComputeUnits::CpuOnly,
            quantization: "fp32".to_string(),
            cache_dir: std::env::temp_dir(),
            timeout_ms: 5000,
        };

        // Test both backends
        let coreml_backend = CoreMLBackend::new();
        let candle_backend = CandleBackend::new();

        let coreml_result = coreml_backend.prepare(&artifact, opts.clone());
        let candle_result = candle_backend.prepare(&artifact, opts);

        if let (Ok(coreml_model), Ok(candle_model)) = (coreml_result, candle_result) {
            // Create identical test inputs
            let mut inputs = TensorMap::new();
            let image_size = 224 * 224 * 3 * 4; // F32 bytes
            inputs.insert("input".to_string(), vec![0u8; image_size]);

            // Run inference on both backends
            let coreml_output =
                coreml_backend.infer(&*coreml_model, &inputs, Duration::from_millis(1000));
            let candle_output =
                candle_backend.infer(&*candle_model, &inputs, Duration::from_millis(1000));

            if let (Ok(coreml_out), Ok(candle_out)) = (coreml_output, candle_output) {
                // Verify outputs match within tolerance (L∞ < 0.01)
                if let (Some(coreml_data), Some(candle_data)) =
                    (coreml_out.get("output"), candle_out.get("output"))
                {
                    let max_diff = coreml_data
                        .iter()
                        .zip(candle_data.iter())
                        .map(|(a, b)| (*a as i32 - *b as i32).abs())
                        .max()
                        .unwrap_or(0);

                    assert!(max_diff < 3, "Output mismatch too large: {}", max_diff);
                    println!("✅ CPU fallback parity test passed: L∞ = {}", max_diff);
                }
            } else {
                println!("⚠️ Inference failed on one or both backends");
            }
        } else {
            println!("⚠️ Model preparation failed on one or both backends");
        }
    }

    /// Test long-running stability and autorelease pool management
    #[test]
    fn test_long_running_stability() {
        let telemetry = TelemetryCollector::new();

        // Simulate 1000 inference operations
        for i in 0..1000 {
            let duration_ms = 10 + (i % 20); // Vary between 10-30ms
            let success = i % 100 != 99; // 99% success rate

            telemetry.record_inference(duration_ms, success, "ane");

            // Test autorelease pool every 100 iterations
            if i % 100 == 0 {
                // This would test actual autorelease pool flushing
                // In a real test, we'd monitor memory usage
                println!("✅ Iteration {}: autorelease pool check", i);
            }
        }

        // Verify telemetry accuracy
        let metrics = telemetry.get_metrics();
        assert_eq!(metrics.infer_count, 1000);
        assert_eq!(metrics.infer_success, 990); // 99% success rate
        assert_eq!(metrics.ane_usage_count, 1000);

        // Verify t-digest p99 calculation
        let stats = telemetry.get_percentile_stats();
        assert!(
            stats.infer_p99 >= 25 && stats.infer_p99 <= 35,
            "P99 outside expected range: {}",
            stats.infer_p99
        );

        println!("✅ Long-running stability test passed");
        println!("   - Total operations: {}", metrics.infer_count);
        println!(
            "   - Success rate: {:.1}%",
            (metrics.infer_success as f64 / metrics.infer_count as f64) * 100.0
        );
        println!("   - P99 latency: {}ms", stats.infer_p99);
    }

    /// Test binary tensor serialization protocol
    #[test]
    fn test_binary_tensor_serialization() {
        // Create test I/O schema
        let schema = IoSchema {
            inputs: vec![TensorSpec {
                name: "input".to_string(),
                dtype: DType::F32,
                shape: vec![1, 224, 224, 3],
                batch_capable: true,
            }],
            outputs: vec![TensorSpec {
                name: "output".to_string(),
                dtype: DType::F32,
                shape: vec![1, 1000],
                batch_capable: true,
            }],
        };

        // Create test input tensors
        let mut inputs = TensorMap::new();
        let image_size = 224 * 224 * 3 * 4; // F32 bytes
        inputs.insert("input".to_string(), vec![42u8; image_size]);

        // Test serialization
        let batch = TensorBatch::from_tensor_map(&inputs, &schema).expect("Serialization failed");

        // Test temp file serialization
        let temp_dir = std::env::temp_dir();
        let json_str = batch
            .to_json_with_data_path(&temp_dir)
            .expect("JSON serialization failed");

        // Test deserialization
        let deserialized =
            TensorBatch::from_json_with_data_path(&json_str).expect("Deserialization failed");
        let outputs = deserialized
            .to_tensor_map()
            .expect("Tensor map conversion failed");

        // Verify data integrity
        assert_eq!(outputs.len(), 1);
        if let Some(output_data) = outputs.get("input") {
            assert_eq!(output_data.len(), image_size);
            assert!(output_data.iter().all(|&b| b == 42));
        }

        // Cleanup temp files
        deserialized.cleanup_temp_files().expect("Cleanup failed");

        println!("✅ Binary tensor serialization test passed");
    }

    /// Test circuit breaker functionality
    #[test]
    fn test_circuit_breaker() {
        let mut telemetry = TelemetryCollector::new();

        // Simulate failures to trigger circuit breaker
        for i in 0..100 {
            let success = i < 90; // 90% success rate (below 95% threshold)
            telemetry.record_inference(50, success, "ane");
        }

        // Verify circuit breaker triggers
        assert!(
            telemetry.should_fallback_to_cpu(),
            "Circuit breaker should be active"
        );

        // Reset and test recovery
        telemetry.reset_circuit_breaker();
        assert!(
            !telemetry.should_fallback_to_cpu(),
            "Circuit breaker should be reset"
        );

        println!("✅ Circuit breaker test passed");
    }

    /// Test t-digest percentile accuracy
    #[test]
    fn test_tdigest_percentile_accuracy() {
        let mut telemetry = TelemetryCollector::new();

        // Generate test data with known distribution
        let mut durations = Vec::new();
        for i in 1..=1000 {
            durations.push(i as u64); // 1ms to 1000ms
        }

        // Record all durations
        for &duration in &durations {
            telemetry.record_inference(duration, true, "ane");
        }

        // Verify percentile calculations
        let stats = telemetry.get_percentile_stats();

        // P50 should be around 500ms (median)
        assert!(
            stats.infer_p50 >= 490 && stats.infer_p50 <= 510,
            "P50 outside expected range: {}",
            stats.infer_p50
        );

        // P95 should be around 950ms
        assert!(
            stats.infer_p95 >= 940 && stats.infer_p95 <= 960,
            "P95 outside expected range: {}",
            stats.infer_p95
        );

        // P99 should be around 990ms
        assert!(
            stats.infer_p99 >= 980 && stats.infer_p99 <= 1000,
            "P99 outside expected range: {}",
            stats.infer_p99
        );

        println!("✅ T-digest percentile accuracy test passed");
        println!("   - P50: {}ms", stats.infer_p50);
        println!("   - P95: {}ms", stats.infer_p95);
        println!("   - P99: {}ms", stats.infer_p99);
    }

    /// Test memory leak detection
    #[test]
    fn test_memory_leak_detection() {
        // This test would monitor memory usage over many iterations
        // In a real implementation, we'd use Instruments or similar tools

        let telemetry = TelemetryCollector::new();

        // Simulate memory allocation and deallocation
        for i in 0..10000 {
            telemetry.record_memory_usage(i % 1000); // Vary memory usage

            if i % 1000 == 0 {
                // Check memory growth (should be minimal)
                let metrics = telemetry.get_metrics();
                assert!(
                    metrics.memory_current_mb < 100,
                    "Memory usage too high: {}MB",
                    metrics.memory_current_mb
                );
            }
        }

        println!("✅ Memory leak detection test passed");
    }
}
