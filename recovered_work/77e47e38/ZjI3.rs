//! Integration tests for Agent Agency V3 components

#[cfg(test)]
mod integration_tests {
    use std::sync::Arc;

    /// Test that all four major components can be initialized
    #[tokio::test]
    async fn test_component_initialization() {
        // Test Runtime Optimization
        let kokoro_tuner = runtime_optimization::KokoroTuner::new();
        assert_eq!(kokoro_tuner.get_session_count().await, 0);

        // Test Tool Ecosystem
        let tool_registry = tool_ecosystem::ToolRegistry::new();
        let tool_coordinator = tool_ecosystem::ToolCoordinator::new(Arc::new(tool_registry));
        let stats = tool_coordinator.get_statistics().await;
        assert_eq!(stats.total_tools, 0);

        // Test Federated Learning
        let aggregator = federated_learning::SecureAggregator::new();
        let round_started = aggregator.start_round(1, 3).await;
        assert!(round_started.is_ok());

        // Test Model Hot-Swapping
        let load_balancer = model_hotswap::LoadBalancer::new();
        let stats = load_balancer.get_statistics().await;
        assert_eq!(stats.active_models, 0);
    }

    /// Test runtime optimization workflow
    #[tokio::test]
    async fn test_runtime_optimization_workflow() {
        let tuner = runtime_optimization::KokoroTuner::new();

        let workload = runtime_optimization::WorkloadSpec {
            name: "test_workload".to_string(),
            can_delay: true,
            priority: 5,
            estimated_duration_seconds: 60,
            thermal_impact: 0.3,
        };

        let result = tuner.tune_model(&workload).await;
        assert!(result.is_ok());

        let tuning_result = result.unwrap();
        assert!(!tuning_result.session_id.is_empty());
        assert!(!tuning_result.parameters.is_empty());
        assert!(tuning_result.metrics.throughput_ops_per_sec > 0.0);
    }

    /// Test federated learning with differential privacy
    #[tokio::test]
    async fn test_federated_learning_privacy() {
        let privacy_params = federated_learning::PrivacyParameters {
            epsilon: 1.0,
            delta: 1e-5,
            sensitivity: 1.0,
            mechanism: federated_learning::NoiseMechanism::Gaussian,
            max_norm: 1.0,
        };

        let mut dp_engine = federated_learning::DifferentialPrivacyEngine::new(privacy_params);
        let original = vec![vec![1.0, 2.0, 3.0]];

        let noisy = dp_engine.add_noise(original.clone()).await;
        assert!(noisy.is_ok());

        let noisy_data = noisy.unwrap();
        assert_eq!(noisy_data.len(), 1);
        assert_eq!(noisy_data[0].len(), 3);

        // Noise should be added (values should be different)
        assert_ne!(noisy_data[0][0], original[0][0]);
    }

    /// Test model hot-swapping canary deployment
    #[tokio::test]
    async fn test_model_hotswap_canary() {
        let load_balancer = model_hotswap::LoadBalancer::new();

        // Initial state should be empty
        let initial_stats = load_balancer.get_statistics().await;
        assert_eq!(initial_stats.active_models, 0);

        // Start canary deployment
        let canary_result = load_balancer.start_canary("model_v2", 0.1).await;
        assert!(canary_result.is_ok());

        // Should now have models
        let canary_stats = load_balancer.get_statistics().await;
        assert!(canary_stats.active_models > 0);
    }

    /// Test quality guardrails validation
    #[tokio::test]
    async fn test_quality_guardrails() {
        let guardrails = runtime_optimization::QualityGuardrails::new();

        let context = runtime_optimization::CheckContext {
            current_metrics: runtime_optimization::performance_monitor::SLAMetrics {
                response_time_p95_ms: 50.0,
                throughput_ops_per_sec: 100.0,
                memory_usage_mb: 256,
                cpu_utilization_percent: 60.0,
                thermal_throttling_events: 0,
                accuracy_score: 0.95,
                total_requests: 1000,
                error_count: 5,
            },
            thresholds: runtime_optimization::quality_guardrails::PerformanceThreshold {
                min_throughput: 80.0,
                max_latency_ms: 100.0,
                max_memory_mb: 512,
                min_accuracy: 0.9,
                max_error_rate: 0.1,
            },
            optimization_config: serde_json::json!({"test": true}),
        };

        let checks = guardrails.run_all_checks(&context).await;
        assert!(checks.is_ok());

        let check_results = checks.unwrap();
        assert!(!check_results.is_empty());

        // Should pass quality checks with these parameters
        let passed_checks = check_results.iter().filter(|c| c.is_valid).count();
        assert!(passed_checks > 0);
    }

    /// Test tool registry functionality
    #[tokio::test]
    async fn test_tool_registry() {
        let mut registry = tool_ecosystem::ToolRegistry::new();

        let tool_info = tool_ecosystem::ToolInfo {
            id: "test_tool".to_string(),
            name: "Test Tool".to_string(),
            description: "A tool for testing".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["test".to_string()],
            security_level: tool_ecosystem::SecurityLevel::Standard,
            resource_requirements: tool_ecosystem::ResourceRequirements {
                cpu_cores: 1,
                memory_mb: 128,
                timeout_seconds: 60,
            },
        };

        let register_result = registry.register_tool(tool_info).await;
        assert!(register_result.is_ok());

        let tools = registry.discover_tools("test").await;
        assert!(tools.is_ok());

        let discovered_tools = tools.unwrap();
        assert_eq!(discovered_tools.len(), 1);
        assert_eq!(discovered_tools[0].id, "test_tool");
    }

    /// Test end-to-end workflow simulation
    #[tokio::test]
    async fn test_end_to_end_workflow() {
        // Initialize all components
        let runtime_optimizer = Arc::new(runtime_optimization::KokoroTuner::new());
        let tool_registry = Arc::new(tool_ecosystem::ToolRegistry::new());
        let tool_coordinator = Arc::new(tool_ecosystem::ToolCoordinator::new(Arc::clone(&tool_registry)));
        let aggregator = Arc::new(federated_learning::SecureAggregator::new());
        let protocol = Arc::new(federated_learning::FederationProtocol::new());
        let federation_coordinator = Arc::new(federated_learning::FederationCoordinator::new(
            federated_learning::FederationConfig {
                min_participants: 2,
                max_participants: 10,
                round_timeout_seconds: 60,
                aggregation_timeout_seconds: 30,
                privacy_parameters: federated_learning::PrivacyParameters {
                    epsilon: 1.0,
                    delta: 1e-5,
                    sensitivity: 1.0,
                    mechanism: federated_learning::NoiseMechanism::Gaussian,
                    max_norm: 1.0,
                },
                security_requirements: federated_learning::SecurityRequirements {
                    require_zkp: false, // Simplified for testing
                    min_encryption_bits: 128,
                    require_differential_privacy: true,
                    max_information_leakage: 0.1,
                },
                quality_thresholds: federated_learning::QualityThresholds {
                    min_accuracy: 0.8,
                    max_staleness: 10,
                    min_contribution_size: 100,
                    max_contribution_size: 10000,
                },
            },
            Arc::clone(&aggregator),
            Arc::clone(&protocol),
        ));

        let model_load_balancer = Arc::new(model_hotswap::LoadBalancer::new());

        // Test that all components can operate together
        let start_federation = federation_coordinator.start().await;
        assert!(start_federation.is_ok());

        let start_round = federation_coordinator.start_round().await;
        assert!(start_round.is_ok());

        let canary_deployment = model_load_balancer.start_canary("test_model", 0.05).await;
        assert!(canary_deployment.is_ok());

        // All components working together successfully
        println!("✅ End-to-end workflow test passed");
    }
}
