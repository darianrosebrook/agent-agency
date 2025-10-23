//! Basic tests for system health monitor
//! These tests are designed to run quickly without external dependencies

use agent_agency_system_health_monitor::types::*;
use std::time::Duration;
use tokio::time::timeout;

/// Create a test configuration that disables external calls
fn create_test_config() -> SystemHealthMonitorConfig {
    SystemHealthMonitorConfig {
        collection_interval_ms: 1000,
        health_check_interval_ms: 1000,
        retention_period_ms: 3600000,
        enable_circuit_breaker: false,
        circuit_breaker_failure_threshold: 5,
        circuit_breaker_recovery_timeout_ms: 60000,
        thresholds: HealthThresholds::default(),
        embedding_service: EmbeddingServiceConfig {
            endpoint: "http://localhost:9999/test".to_string(),
            timeout_ms: 100,
            max_retries: 1,
            retry_backoff_multiplier: 1.0,
            enabled: false,
        },
        redis: None,
        filesystem: FilesystemConfig {
            enabled: false,
            critical_threshold_percent: 90.0,
            warning_threshold_percent: 75.0,
            disk_io_enabled: false,
        },
    }
}

#[tokio::test]
async fn test_config_creation() {
    let config = create_test_config();
    assert!(!config.embedding_service.enabled);
    assert!(!config.filesystem.enabled);
    assert_eq!(config.collection_interval_ms, 1000);
    assert_eq!(config.health_check_interval_ms, 1000);
}

#[tokio::test]
async fn test_embedding_service_config() {
    let config = create_test_config();
    assert!(!config.embedding_service.enabled);
    assert_eq!(config.embedding_service.endpoint, "http://localhost:9999/test");
    assert_eq!(config.embedding_service.timeout_ms, 100);
    assert_eq!(config.embedding_service.max_retries, 1);
}

#[tokio::test]
async fn test_filesystem_config() {
    let config = create_test_config();
    assert!(!config.filesystem.enabled);
    assert_eq!(config.filesystem.critical_threshold_percent, 90.0);
    assert_eq!(config.filesystem.warning_threshold_percent, 75.0);
    assert!(!config.filesystem.disk_io_enabled);
}

// Linear regression test removed - functionality not implemented

#[tokio::test]
async fn test_disk_usage_data_point() {
    let data_point = DiskUsageDataPoint {
        timestamp: chrono::Utc::now(),
        usage_percentage: 50.0,
        used_space: 5000,
    };

    assert_eq!(data_point.used_space, 5000);
    assert_eq!(data_point.usage_percentage, 50.0);
}

#[tokio::test]
async fn test_health_alert_creation() {
    let alert = HealthAlert {
        id: "test-alert-1".to_string(),
        severity: AlertSeverity::Warning,
        alert_type: AlertType::SystemHealth,
        message: "Test alert message".to_string(),
        target: "test-system".to_string(),
        component: "test-component".to_string(),
        timestamp: chrono::Utc::now(),
        acknowledged: false,
        resolved: false,
        resolved_at: None,
        metadata: std::collections::HashMap::new(),
    };

    assert_eq!(alert.id, "test-alert-1");
    assert_eq!(alert.alert_type, AlertType::SystemHealth);
    assert_eq!(alert.severity, AlertSeverity::Warning);
    assert_eq!(alert.message, "Test alert message");
    assert_eq!(alert.component, "test-component");
    assert_eq!(alert.target, "test-system");
    assert!(!alert.acknowledged);
    assert!(!alert.resolved);
    assert!(alert.resolved_at.is_none());
}

#[tokio::test]
async fn test_system_metrics_creation() {
    let metrics = SystemMetrics {
        cpu_usage: 25.5,
        memory_usage: 60.0,
        disk_usage: 45.0,
        load_average: [0.5, 0.6, 0.7],
        network_io: 1024,
        disk_io: 2048,
        disk_io_metrics: Default::default(),
        disk_usage_metrics: Default::default(),
        timestamp: chrono::Utc::now(),
    };

    assert_eq!(metrics.cpu_usage, 25.5);
    assert_eq!(metrics.memory_usage, 60.0);
    assert_eq!(metrics.disk_usage, 45.0);
    assert_eq!(metrics.network_io, 1024);
    assert_eq!(metrics.disk_io, 2048);
    assert_eq!(metrics.load_average, [0.5, 0.6, 0.7]);
}

#[tokio::test]
async fn test_embedding_service_performance() {
    let performance = EmbeddingServicePerformance {
        total_requests: 1000,
        successful_requests: 950,
        failed_requests: 50,
        avg_response_time_ms: 150.0,
        cache_hits: 800,
        cache_misses: 200,
        model_load_time_ms: 2000.0,
        memory_usage_mb: 512.0,
        gpu_utilization: 0.75,
        queue_depth: 5,
    };

    assert_eq!(performance.total_requests, 1000);
    assert_eq!(performance.successful_requests, 950);
    assert_eq!(performance.failed_requests, 50);
    assert_eq!(performance.avg_response_time_ms, 150.0);
    assert_eq!(performance.cache_hits, 800);
    assert_eq!(performance.cache_misses, 200);
    assert_eq!(performance.model_load_time_ms, 2000.0);
    assert_eq!(performance.memory_usage_mb, 512.0);
    assert_eq!(performance.gpu_utilization, 0.75);
    assert_eq!(performance.queue_depth, 5);
}

#[tokio::test]
async fn test_disk_usage_trends() {
    let trends = DiskUsageTrends {
        current_usage_percentage: 50.0,
        growth_rate_bytes_per_day: 1000.0,
        predicted_usage_7_days: 0.6,
        predicted_usage_30_days: 0.8,
        days_until_90_percent: Some(10),
        days_until_95_percent: Some(5),
        days_until_100_percent: Some(2),
        confidence: 0.85,
        historical_data_points: 30,
    };

    assert_eq!(trends.growth_rate_bytes_per_day, 1000.0);
    assert_eq!(trends.days_until_90_percent, Some(10));
    assert_eq!(trends.days_until_95_percent, Some(5));
    assert_eq!(trends.days_until_100_percent, Some(2));
    assert_eq!(trends.predicted_usage_7_days, 0.6);
    assert_eq!(trends.predicted_usage_30_days, 0.8);
    assert_eq!(trends.confidence, 0.85);
    assert_eq!(trends.current_usage_percentage, 50.0);
    assert_eq!(trends.historical_data_points, 30);
}
