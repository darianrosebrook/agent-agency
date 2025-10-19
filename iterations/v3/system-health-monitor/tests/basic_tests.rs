//! Basic tests for system health monitor
//! These tests are designed to run quickly without external dependencies

use agent_agency_system_health_monitor::types::*;
use std::time::Duration;
use tokio::time::timeout;

/// Create a test configuration that disables external calls
fn create_test_config() -> SystemHealthMonitorConfig {
    SystemHealthMonitorConfig {
        collection_interval_seconds: 1,
        health_check_interval_seconds: 1,
        alert_retention_hours: 1,
        metrics_retention_hours: 1,
        enable_embedding_service_monitoring: false,
        enable_database_monitoring: false,
        enable_filesystem_monitoring: false,
        embedding_service: EmbeddingServiceConfig {
            endpoint: "http://localhost:9999/test".to_string(),
            timeout_ms: 100,
            max_retries: 1,
            retry_backoff_multiplier: 1.0,
            enabled: false,
        },
        database: DatabaseConfig {
            connection_string: "postgresql://test:test@localhost:5432/test".to_string(),
            timeout_ms: 100,
            max_retries: 1,
            retry_backoff_multiplier: 1.0,
            enabled: false,
        },
        filesystem: FilesystemConfig {
            mount_points: vec!["/tmp".to_string()],
            check_interval_seconds: 1,
            fragmentation_threshold: 50.0,
            inode_usage_threshold: 80.0,
            enabled: false,
        },
    }
}

#[tokio::test]
async fn test_config_creation() {
    let config = create_test_config();
    assert!(!config.embedding_service.enabled);
    assert!(!config.database.enabled);
    assert!(!config.filesystem.enabled);
    assert_eq!(config.collection_interval_seconds, 1);
    assert_eq!(config.health_check_interval_seconds, 1);
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
async fn test_database_config() {
    let config = create_test_config();
    assert!(!config.database.enabled);
    assert_eq!(config.database.connection_string, "postgresql://test:test@localhost:5432/test");
    assert_eq!(config.database.timeout_ms, 100);
    assert_eq!(config.database.max_retries, 1);
}

#[tokio::test]
async fn test_filesystem_config() {
    let config = create_test_config();
    assert!(!config.filesystem.enabled);
    assert_eq!(config.filesystem.mount_points, vec!["/tmp"]);
    assert_eq!(config.filesystem.check_interval_seconds, 1);
    assert_eq!(config.filesystem.fragmentation_threshold, 50.0);
    assert_eq!(config.filesystem.inode_usage_threshold, 80.0);
}

#[tokio::test]
async fn test_linear_regression_calculation() {
    // Test the linear regression function with known data
    let data_points = vec![
        DiskUsageDataPoint {
            timestamp: chrono::Utc::now() - chrono::Duration::days(3),
            used_bytes: 1000,
            total_bytes: 10000,
            usage_percentage: 10.0,
        },
        DiskUsageDataPoint {
            timestamp: chrono::Utc::now() - chrono::Duration::days(2),
            used_bytes: 2000,
            total_bytes: 10000,
            usage_percentage: 20.0,
        },
        DiskUsageDataPoint {
            timestamp: chrono::Utc::now() - chrono::Duration::days(1),
            used_bytes: 3000,
            total_bytes: 10000,
            usage_percentage: 30.0,
        },
    ];

    let growth_rate = agent_agency_system_health_monitor::SystemHealthMonitor::calculate_linear_regression_growth_rate(&data_points);
    assert!(growth_rate > 0.0, "Growth rate should be positive for increasing data");
    assert!(growth_rate < 10000.0, "Growth rate should be reasonable");
}

#[tokio::test]
async fn test_disk_usage_data_point() {
    let data_point = DiskUsageDataPoint {
        timestamp: chrono::Utc::now(),
        used_bytes: 5000,
        total_bytes: 10000,
        usage_percentage: 50.0,
    };
    
    assert_eq!(data_point.used_bytes, 5000);
    assert_eq!(data_point.total_bytes, 10000);
    assert_eq!(data_point.usage_percentage, 50.0);
}

#[tokio::test]
async fn test_health_alert_creation() {
    let alert = HealthAlert {
        id: "test-alert-1".to_string(),
        alert_type: AlertType::SystemHealth,
        severity: AlertSeverity::Warning,
        message: "Test alert message".to_string(),
        timestamp: std::time::SystemTime::now(),
        component: "test-component".to_string(),
        metadata: std::collections::HashMap::new(),
    };
    
    assert_eq!(alert.id, "test-alert-1");
    assert_eq!(alert.alert_type, AlertType::SystemHealth);
    assert_eq!(alert.severity, AlertSeverity::Warning);
    assert_eq!(alert.message, "Test alert message");
    assert_eq!(alert.component, "test-component");
}

#[tokio::test]
async fn test_system_metrics_creation() {
    let metrics = SystemMetrics {
        cpu_usage_percentage: 25.5,
        memory_usage_percentage: 60.0,
        disk_usage_percentage: 45.0,
        network_io_bytes_per_sec: 1024,
        disk_io_bytes_per_sec: 2048,
        load_average: [0.5, 0.6, 0.7],
        timestamp: chrono::Utc::now(),
    };
    
    assert_eq!(metrics.cpu_usage_percentage, 25.5);
    assert_eq!(metrics.memory_usage_percentage, 60.0);
    assert_eq!(metrics.disk_usage_percentage, 45.0);
    assert_eq!(metrics.network_io_bytes_per_sec, 1024);
    assert_eq!(metrics.disk_io_bytes_per_sec, 2048);
    assert_eq!(metrics.load_average, [0.5, 0.6, 0.7]);
}

#[tokio::test]
async fn test_embedding_service_performance() {
    let performance = EmbeddingServicePerformance {
        total_requests: 1000,
        successful_requests: 950,
        failed_requests: 50,
        avg_response_time_ms: 150.0,
        cache_hit_rate: 0.8,
        model_load_time_ms: 2000.0,
        memory_usage_mb: 512.0,
        gpu_utilization: 0.75,
        queue_depth: 5,
        timestamp: chrono::Utc::now(),
    };
    
    assert_eq!(performance.total_requests, 1000);
    assert_eq!(performance.successful_requests, 950);
    assert_eq!(performance.failed_requests, 50);
    assert_eq!(performance.avg_response_time_ms, 150.0);
    assert_eq!(performance.cache_hit_rate, 0.8);
    assert_eq!(performance.model_load_time_ms, 2000.0);
    assert_eq!(performance.memory_usage_mb, 512.0);
    assert_eq!(performance.gpu_utilization, 0.75);
    assert_eq!(performance.queue_depth, 5);
}

#[tokio::test]
async fn test_disk_usage_trends() {
    let trends = DiskUsageTrends {
        growth_rate_bytes_per_day: 1000.0,
        days_until_80_percent: 10.0,
        days_until_90_percent: 5.0,
        days_until_95_percent: 2.0,
        predicted_usage_7_days: 0.6,
        predicted_usage_30_days: 0.8,
        confidence: 0.85,
    };
    
    assert_eq!(trends.growth_rate_bytes_per_day, 1000.0);
    assert_eq!(trends.days_until_80_percent, 10.0);
    assert_eq!(trends.days_until_90_percent, 5.0);
    assert_eq!(trends.days_until_95_percent, 2.0);
    assert_eq!(trends.predicted_usage_7_days, 0.6);
    assert_eq!(trends.predicted_usage_30_days, 0.8);
    assert_eq!(trends.confidence, 0.85);
}
