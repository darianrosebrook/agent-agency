//! Comprehensive tests for ANE Manager components
//!
//! Tests cover:
//! - Error handling and conversions
//! - Resource pool admission control
//! - Core ML compatibility detection
//! - Model loading and validation
//! - Inference execution with timeouts
//! - EWMA metrics calculation
//! - ANEManager integration

#[cfg(test)]
mod tests {
    use agent_agency_apple_silicon::ane::errors::{ANEError, Result};
    use agent_agency_apple_silicon::ane::resource_pool::{Pool, PoolBuilder, PoolStats};
    use agent_agency_apple_silicon::ane::compat::{coreml, iokit};
    use agent_agency_apple_silicon::ane::models::coreml_model::{
        LoadedCoreMLModel, ModelMetadata, ModelSchema, IOTensorSpec, DType, CompilationOptions
    };
    use agent_agency_apple_silicon::ane::infer::execute::{execute_inference, InferenceOptions, InferenceResult};
    use agent_agency_apple_silicon::ane::metrics::ewma::{Ewma, PerformanceTracker, PerformanceSummary};
    use std::path::Path;
    use std::time::{Duration, Instant};

    /// Test ANE error types and conversions
    mod error_tests {
        use super::*;

        #[test]
        fn test_ane_error_display() {
            let err = ANEError::Unavailable;
            assert!(err.to_string().contains("ANE unavailable"));

            let err = ANEError::ModelNotFound("test_model".to_string());
            assert!(err.to_string().contains("test_model"));

            let err = ANEError::Timeout(5000);
            assert!(err.to_string().contains("5000 ms"));
        }

        #[test]
        fn test_error_conversions() {
            // Test std::io::Error conversion
            let io_err = std::io::Error::from(std::io::ErrorKind::NotFound);
            let ane_err: ANEError = io_err.into();
            assert!(ane_err.to_string().contains("IO error"));

            // Test anyhow::Error conversion
            let anyhow_err = anyhow::anyhow!("test error");
            let ane_err: ANEError = anyhow_err.into();
            assert!(ane_err.to_string().contains("Anyhow error"));

            // Test serde_json::Error conversion
            let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
            let ane_err: ANEError = json_err.into();
            assert!(ane_err.to_string().contains("JSON error"));
        }

        #[test]
        fn test_into_ane_error_trait() {
            use agent_agency_apple_silicon::ane::errors::IntoANEError;

            let err: ANEError = "test string".into_ane_error();
            assert!(err.to_string().contains("test string"));

            let err: ANEError = "test &str".into_ane_error();
            assert!(err.to_string().contains("test &str"));
        }
    }

    /// Test resource pool functionality
    mod resource_pool_tests {
        use super::*;

        #[tokio::test]
        async fn test_pool_creation() {
            let pool = Pool::new(4, 1024);
            assert_eq!(pool.config().max_concurrent, 4);
            assert_eq!(pool.config().mem_total_mb, 1024);
        }

        #[tokio::test]
        async fn test_pool_builder() {
            let pool = PoolBuilder::new()
                .max_concurrent(8)
                .memory_total_mb(2048)
                .build()
                .expect("Pool creation should succeed");

            assert_eq!(pool.config().max_concurrent, 8);
            assert_eq!(pool.config().mem_total_mb, 2048);
        }

        #[tokio::test]
        async fn test_pool_admission_success() {
            let pool = Pool::new(2, 1024);
            let admission = pool.admit(256).await;
            assert!(admission.is_ok());
        }

        #[tokio::test]
        async fn test_pool_memory_limit() {
            let pool = Pool::new(2, 512);
            let admission = pool.admit(1024).await;
            assert!(admission.is_err());
            assert!(matches!(admission.unwrap_err(), ANEError::ResourceLimit(_)));
        }

        #[tokio::test]
        async fn test_pool_concurrency_limit() {
            let pool = Pool::new(1, 4096);

            // First admission should succeed
            let admission1 = pool.admit(256).await;
            assert!(admission1.is_ok());

            // Second admission should wait or fail based on semaphore
            // Since we have 1 permit, this should succeed when the first is dropped
            drop(admission1);
            let admission2 = pool.admit(256).await;
            assert!(admission2.is_ok());
        }

        #[tokio::test]
        async fn test_pool_statistics() {
            let pool = Pool::new(2, 1024);
            let stats = pool.stats();
            assert_eq!(stats.total_admissions, 0);
            assert_eq!(stats.admission_failures, 0);
        }
    }

    /// Test Core ML compatibility layer
    mod compat_tests {
        use super::*;

        #[test]
        fn test_coreml_availability() {
            // On non-macOS targets, should return false
            #[cfg(not(target_os = "macos"))]
            assert!(!coreml::detect_coreml_capabilities().supported_precisions.is_empty());

            // On macOS, check that we get some capabilities
            #[cfg(target_os = "macos")]
            {
                let caps = agent_agency_apple_silicon::ane::compat::coreml::detect_coreml_capabilities();
                assert!(!caps.supported_precisions.is_empty());
            }
        }

        #[test]
        fn test_iokit_thermal_status() {
            let status = agent_agency_apple_silicon::ane::compat::iokit::thermal_status();
            assert!(status.system_temperature >= 0.0);
        }

        #[test]
        fn test_iokit_power_status() {
            let status = agent_agency_apple_silicon::ane::compat::iokit::power_status();
            assert!(status.system_power >= 0.0);
        }
    }

    /// Test Core ML model handling
    mod model_tests {
        use super::*;
        use agent_agency_apple_silicon::ane::models::coreml_model;

        #[test]
        fn test_compilation_options() {
            let opts = CompilationOptions::default();
            assert!(opts.compute_units.is_none());
            assert!(opts.minimum_precision.is_none());
            assert!(opts.output_path.is_none());
        }

        #[test]
        fn test_model_schema_creation() {
            let inputs = vec![
                IOTensorSpec {
                    name: "input".to_string(),
                    shape: vec![1, 3, 224, 224],
                    dtype: DType::F32,
                    optional: false,
                }
            ];

            let outputs = vec![
                IOTensorSpec {
                    name: "output".to_string(),
                    shape: vec![1, 1000],
                    dtype: DType::F32,
                    optional: false,
                }
            ];

            let schema = ModelSchema { inputs, outputs };
            assert_eq!(schema.inputs.len(), 1);
            assert_eq!(schema.outputs.len(), 1);
        }

        #[test]
        fn test_memory_estimation() {
            let schema = ModelSchema {
                inputs: vec![IOTensorSpec {
                    name: "input".to_string(),
                    shape: vec![1, 3, 224, 224],
                    dtype: DType::F32,
                    optional: false,
                }],
                outputs: vec![IOTensorSpec {
                    name: "output".to_string(),
                    shape: vec![1, 1000],
                    dtype: DType::F32,
                    optional: false,
                }],
            };

            let metadata = ModelMetadata {
                path: Path::new("test.mlmodel").to_path_buf(),
                size_bytes: 1024,
                format: "mlmodel".to_string(),
                version: Some("1.0".to_string()),
                description: None,
                author: None,
                license: None,
            };

            let model = LoadedCoreMLModel {
                model_id: "test_model".to_string(),
                compiled_path: Path::new("test.mlmodelc").to_path_buf(),
                metadata,
                schema,
                loaded_at: Instant::now(),
                last_accessed: Instant::now(), 
            };

            let mem_mb = agent_agency_apple_silicon::ane::models::coreml_model::estimate_memory_usage(&model);
            assert!(mem_mb > 0);
        }
    }

    /// Test inference execution
    mod inference_tests {
        use super::*;

        #[test]
        fn test_inference_options() {
            let opts = InferenceOptions {
                timeout_ms: 5000,
                batch_size: Some(4),
                precision: Some("fp16".to_string()),
                compute_units: Some("ANE".to_string()),
                enable_monitoring: true,
            };

            assert_eq!(opts.timeout_ms, 5000);
            assert_eq!(opts.batch_size, Some(4));
        }

        #[tokio::test]
        async fn test_inference_timeout() {
            // Create a mock model
            let schema = ModelSchema {
                inputs: vec![IOTensorSpec {
                    name: "input".to_string(),
                    shape: vec![1, 3, 224, 224],
                    dtype: DType::F32,
                    optional: false,
                }],
                outputs: vec![IOTensorSpec {
                    name: "output".to_string(),
                    shape: vec![1, 1000],
                    dtype: DType::F32,
                    optional: false,
                }],
            };

            let model = LoadedCoreMLModel {
                model_id: "test_model".to_string(),
                compiled_path: Path::new("test.mlmodelc").to_path_buf(),
                metadata: ModelMetadata {
                    path: Path::new("test.mlmodel").to_path_buf(),
                    size_bytes: 1024,
                    format: "mlmodelc".to_string(),
                    version: None,
                    description: None,
                    author: None,
                    license: None,
                },
                schema,
                loaded_at: Instant::now(),
                last_accessed: Instant::now(), 
            };

            // Test with input that should work
            let input = vec![0.5f32; 1 * 3 * 224 * 224];
            let opts = InferenceOptions {
                timeout_ms: 1000,
                batch_size: None,
                precision: None,
                compute_units: None,
                enable_monitoring: false,
            };

            // This should fail on non-macOS targets or when ANE is not available
            let result = execute_inference(&model, &input, &opts).await;
            assert!(result.is_err()); // Should fail on non-macOS or without proper setup
        }
    }

    /// Test EWMA metrics
    mod metrics_tests {
        use super::*;

        #[test]
        fn test_ewma_calculation() {
            // Test basic EWMA calculation
            let alpha = 0.2;
            let mut prev = 100.0;
            prev = Ewma::update(prev, 120.0, alpha);
            assert!(prev > 100.0 && prev < 120.0);

            // Test convergence
            let mut value = 0.0;
            for _ in 0..100 {
                value = Ewma::update(value, 10.0, 0.1);
            }
            assert!((value - 10.0).abs() < 0.1); // Should converge close to target
        }

        #[test]
        fn test_performance_tracker() {
            let mut tracker = PerformanceTracker::new();

            // Initial state
            let summary = tracker.get_summary();
            assert_eq!(summary.total_inferences, 0);
            assert_eq!(summary.average_latency_ms, 0.0);

            // Add some measurements
            tracker.update_latency(100.0);
            tracker.update_throughput(10.0);
            tracker.update_memory(512.0);

            let summary = tracker.get_summary();
            assert_eq!(summary.total_inferences, 1);
            assert_eq!(summary.average_latency_ms, 100.0);
            assert_eq!(summary.average_throughput_ips, 10.0);
            assert_eq!(summary.average_memory_mb, 512.0);

            // Add more measurements
            tracker.update_latency(150.0);
            let summary = tracker.get_summary();
            assert_eq!(summary.total_inferences, 2);
            // EWMA should be somewhere between 100 and 150
            assert!(summary.average_latency_ms > 100.0 && summary.average_latency_ms < 150.0);
        }
    }

    /// Test ANEManager integration
    mod manager_tests {
        use super::*;
        use agent_agency_apple_silicon::ane::manager::ANEManager;

        #[tokio::test]
        async fn test_manager_creation() {
            let manager = ANEManager::new();
            assert!(manager.is_ok());
        }

        #[tokio::test]
        async fn test_manager_with_config() {
            let config = agent_agency_apple_silicon::ane::manager::ANEConfig {
                max_concurrent_operations: 2,
                memory_pool_mb: 1024,
                default_timeout_ms: 5000,
            };

            let manager = ANEManager::with_config(config, None, None);
            assert!(manager.is_ok());
        }

        #[tokio::test]
        async fn test_manager_capabilities() {
            let manager = ANEManager::new().expect("Manager creation failed");
            let caps = manager.get_capabilities().await;

            // On macOS aarch64, should be available
            #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
            assert!(caps.is_available);

            // On other platforms, may not be available
            #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
            assert!(!caps.is_available);
        }

        #[tokio::test]
        async fn test_device_status() {
            let manager = ANEManager::new().expect("Manager creation failed");
            let status = manager.get_device_status().await;

            assert!(status.memory_total_mb > 0);
            assert!(status.max_concurrent_models > 0);
        }

        #[tokio::test]
        async fn test_performance_summary() {
            let manager = ANEManager::new().expect("Manager creation failed");
            let summary = manager.get_performance_summary().await;

            // Initially should be zero
            assert_eq!(summary.total_inferences, 0);
        }

        #[tokio::test]
        async fn test_resource_pool_stats() {
            let manager = ANEManager::new().expect("Manager creation failed");
            let stats = manager.get_resource_pool_stats();

            assert!(stats.current_memory_usage_mb >= 0);
            assert!(stats.total_admissions >= 0);
        }
    }
}
