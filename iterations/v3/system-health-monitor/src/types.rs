use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// System health monitor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthMonitorConfig {
    /// Metrics collection interval (milliseconds)
    pub collection_interval_ms: u64,
    /// Health check interval (milliseconds)
    pub health_check_interval_ms: u64,
    /// Metrics retention period (milliseconds)
    pub retention_period_ms: u64,
    /// Enable circuit breaker
    pub enable_circuit_breaker: bool,
    /// Circuit breaker failure threshold
    pub circuit_breaker_failure_threshold: u32,
    /// Circuit breaker recovery timeout (milliseconds)
    pub circuit_breaker_recovery_timeout_ms: u64,
    /// Health thresholds
    pub thresholds: HealthThresholds,
    /// Embedding service configuration
    pub embedding_service: EmbeddingServiceConfig,
}

/// Embedding service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingServiceConfig {
    /// Embedding service endpoint URL
    pub endpoint: String,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum number of retries
    pub max_retries: usize,
    /// Retry backoff multiplier
    pub retry_backoff_multiplier: f64,
    /// Enable embedding service monitoring
    pub enabled: bool,
}

impl Default for EmbeddingServiceConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8080/metrics".to_string(),
            timeout_ms: 5000,
            max_retries: 3,
            retry_backoff_multiplier: 1.5,
            enabled: true,
        }
    }
}

/// Health thresholds for alerting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThresholds {
    /// CPU usage warning threshold (%)
    pub cpu_warning_threshold: f64,
    /// CPU usage critical threshold (%)
    pub cpu_critical_threshold: f64,
    /// Memory usage warning threshold (%)
    pub memory_warning_threshold: f64,
    /// Memory usage critical threshold (%)
    pub memory_critical_threshold: f64,
    /// Disk usage warning threshold (%)
    pub disk_warning_threshold: f64,
    /// Disk usage critical threshold (%)
    pub disk_critical_threshold: f64,
    /// System error rate threshold
    pub system_error_rate_threshold: f64,
    /// Queue depth threshold
    pub queue_depth_threshold: u32,
    /// Agent error rate threshold
    pub agent_error_rate_threshold: f64,
    /// Agent response time threshold (ms)
    pub agent_response_time_threshold: u64,
}

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Memory usage percentage
    pub memory_usage: f64,
    /// Disk usage percentage
    pub disk_usage: f64,
    /// Load average (1, 5, 15 minutes)
    pub load_average: [f64; 3],
    /// Network I/O (bytes/sec)
    pub network_io: u64,
    /// Disk I/O (bytes/sec)
    pub disk_io: u64,
    /// Detailed disk I/O metrics
    pub disk_io_metrics: DiskIOMetrics,
    /// Comprehensive disk usage metrics
    pub disk_usage_metrics: DiskUsageMetrics,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Detailed disk I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskIOMetrics {
    /// Total read operations per second
    pub read_iops: u64,
    /// Total write operations per second
    pub write_iops: u64,
    /// Total read throughput (bytes/sec)
    pub read_throughput: u64,
    /// Total write throughput (bytes/sec)
    pub write_throughput: u64,
    /// Average read latency (milliseconds)
    pub avg_read_latency_ms: f64,
    /// Average write latency (milliseconds)
    pub avg_write_latency_ms: f64,
    /// Disk utilization percentage
    pub disk_utilization: f64,
    /// Queue depth
    pub queue_depth: u32,
    /// Per-disk metrics
    pub per_disk_metrics: HashMap<String, PerDiskMetrics>,
}

/// Per-disk I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerDiskMetrics {
    /// Disk name/identifier
    pub disk_name: String,
    /// Read operations per second
    pub read_iops: u64,
    /// Write operations per second
    pub write_iops: u64,
    /// Read throughput (bytes/sec)
    pub read_throughput: u64,
    /// Write throughput (bytes/sec)
    pub write_throughput: u64,
    /// Average read latency (milliseconds)
    pub avg_read_latency_ms: f64,
    /// Average write latency (milliseconds)
    pub avg_write_latency_ms: f64,
    /// Disk utilization percentage
    pub utilization: f64,
    /// Queue depth
    pub queue_depth: u32,
    /// Disk health status
    pub health_status: DiskHealthStatus,
}

/// Disk health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiskHealthStatus {
    /// Disk is healthy
    Healthy,
    /// Disk has warnings
    Warning,
    /// Disk is unhealthy
    Unhealthy,
    /// Disk status unknown
    Unknown,
}

/// Comprehensive disk usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsageMetrics {
    /// Per-filesystem disk usage
    pub filesystem_usage: HashMap<String, FilesystemUsage>,
    /// Total disk space across all filesystems
    pub total_disk_space: u64,
    /// Total used disk space across all filesystems
    pub total_used_space: u64,
    /// Total available disk space across all filesystems
    pub total_available_space: u64,
    /// Overall disk usage percentage
    pub overall_usage_percentage: f64,
    /// Disk usage trends and predictions
    pub usage_trends: DiskUsageTrends,
    /// Filesystem health status
    pub filesystem_health: HashMap<String, FilesystemHealth>,
    /// Inode usage statistics
    pub inode_usage: HashMap<String, InodeUsage>,
}

/// Per-filesystem usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemUsage {
    /// Filesystem mount point
    pub mount_point: String,
    /// Filesystem type (ext4, NTFS, APFS, etc.)
    pub filesystem_type: String,
    /// Total disk space in bytes
    pub total_space: u64,
    /// Used disk space in bytes
    pub used_space: u64,
    /// Available disk space in bytes
    pub available_space: u64,
    /// Usage percentage (0.0 to 100.0)
    pub usage_percentage: f64,
    /// Disk device name
    pub device_name: String,
    /// Mount options
    pub mount_options: String,
}

/// Disk usage trends and predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsageTrends {
    /// Historical usage data points
    pub historical_usage: Vec<DiskUsageDataPoint>,
    /// Predicted usage in 24 hours
    pub predicted_usage_24h: f64,
    /// Predicted usage in 7 days
    pub predicted_usage_7d: f64,
    /// Predicted usage in 30 days
    pub predicted_usage_30d: f64,
    /// Days until 90% capacity
    pub days_until_90_percent: Option<u32>,
    /// Days until 95% capacity
    pub days_until_95_percent: Option<u32>,
    /// Growth rate (bytes per day)
    pub growth_rate_bytes_per_day: f64,
}

/// Historical disk usage data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskUsageDataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Usage percentage at this time
    pub usage_percentage: f64,
    /// Used space in bytes
    pub used_space: u64,
}

/// Filesystem health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemHealth {
    /// Filesystem mount point
    pub mount_point: String,
    /// Health status
    pub health_status: FilesystemHealthStatus,
    /// Error count
    pub error_count: u32,
    /// Last filesystem check timestamp
    pub last_check: Option<DateTime<Utc>>,
    /// Fragmentation level (0.0 to 1.0)
    pub fragmentation_level: f64,
    /// Mount status
    pub mount_status: MountStatus,
    /// Filesystem errors
    pub filesystem_errors: Vec<FilesystemError>,
}

/// Filesystem health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilesystemHealthStatus {
    /// Filesystem is healthy
    Healthy,
    /// Filesystem has warnings
    Warning,
    /// Filesystem has errors
    Error,
    /// Filesystem status unknown
    Unknown,
}

/// Mount status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MountStatus {
    /// Filesystem is mounted
    Mounted,
    /// Filesystem is unmounted
    Unmounted,
    /// Mount status unknown
    Unknown,
}

/// Filesystem error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesystemError {
    /// Error type
    pub error_type: String,
    /// Error message
    pub error_message: String,
    /// Error timestamp
    pub timestamp: DateTime<Utc>,
    /// Error severity
    pub severity: ErrorSeverity,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Inode usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InodeUsage {
    /// Filesystem mount point
    pub mount_point: String,
    /// Total inodes
    pub total_inodes: u64,
    /// Used inodes
    pub used_inodes: u64,
    /// Available inodes
    pub available_inodes: u64,
    /// Inode usage percentage (0.0 to 100.0)
    pub inode_usage_percentage: f64,
}

/// Agent health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealthMetrics {
    /// Agent ID
    pub agent_id: String,
    /// Health score (0-1)
    pub health_score: f64,
    /// Current load
    pub current_load: u32,
    /// Maximum load capacity
    pub max_load: u32,
    /// Success rate (0-1)
    pub success_rate: f64,
    /// Error rate (errors per minute)
    pub error_rate: f64,
    /// Response time P95 (milliseconds)
    pub response_time_p95: u64,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Tasks completed in last hour
    pub tasks_completed_hour: u32,
}

/// Database health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealthMetrics {
    /// Database connection status
    pub connection_ok: bool,
    /// Database pool status
    pub pool_ok: bool,
    /// Database performance status
    pub performance_ok: bool,
    /// Database response time (milliseconds)
    pub response_time_ms: u64,
    /// Database diagnostics
    pub diagnostics: Option<agent_agency_database::health::DatabaseDiagnostics>,
    /// Last database health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Health alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    /// Alert ID
    pub id: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert type
    pub alert_type: AlertType,
    /// Alert message
    pub message: String,
    /// Target (system or agent ID)
    pub target: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Acknowledged flag
    pub acknowledged: bool,
    /// Resolved flag
    pub resolved: bool,
    /// Resolution timestamp
    pub resolved_at: Option<DateTime<Utc>>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertType {
    /// System resource alert
    SystemResource,
    /// Agent health alert
    AgentHealth,
    /// Circuit breaker alert
    CircuitBreaker,
    /// Performance degradation alert
    PerformanceDegradation,
    /// Error rate alert
    ErrorRate,
    /// Custom alert
    Custom,
}

/// Overall health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Overall health score (0-1)
    pub overall_health: f64,
    /// System metrics
    pub system: SystemMetrics,
    /// Agent health metrics
    pub agents: HashMap<String, AgentHealthMetrics>,
    /// Active alerts
    pub alerts: Vec<HealthAlert>,
    /// Error rate across system
    pub error_rate: f64,
    /// Estimated queue depth
    pub queue_depth: u32,
    /// Circuit breaker state
    pub circuit_breaker_state: CircuitBreakerState,
    /// Database health metrics (if available)
    pub database_health: Option<DatabaseHealthMetrics>,
    /// Embedding metrics (if available)
    pub embedding_metrics: Option<EmbeddingMetrics>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Circuit breaker states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (requests blocked)
    Open,
    /// Circuit is half-open (testing recovery)
    HalfOpen,
}

/// Embedding metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingMetrics {
    /// Total embedding requests
    pub total_requests: u64,
    /// Successful embedding generations
    pub successful_generations: u64,
    /// Failed embedding generations
    pub failed_generations: u64,
    /// Average embedding generation time (ms)
    pub avg_generation_time_ms: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Model health status
    pub model_health_status: String,
}

/// Embedding service performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingServicePerformance {
    /// Total requests processed
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Model load time in milliseconds
    pub model_load_time_ms: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// GPU utilization (0.0 to 1.0)
    pub gpu_utilization: f64,
    /// Current queue depth
    pub queue_depth: u32,
}

/// Embedding service metrics response from HTTP endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingServiceMetricsResponse {
    /// Total requests processed
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Model load time in milliseconds
    pub model_load_time_ms: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// GPU utilization (0.0 to 1.0)
    pub gpu_utilization: f64,
    /// Current queue depth
    pub queue_depth: u32,
}

/// Embedding performance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingPerformanceData {
    /// Throughput in requests per second
    pub throughput_requests_per_second: f64,
    /// 99th percentile latency in milliseconds
    pub latency_p99_ms: f64,
    /// 95th percentile latency in milliseconds
    pub latency_p95_ms: f64,
    /// 50th percentile latency in milliseconds
    pub latency_p50_ms: f64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
    /// Availability percentage
    pub availability_percentage: f64,
    /// Model accuracy score
    pub model_accuracy: f64,
    /// Embedding dimension
    pub embedding_dimension: u32,
    /// Batch size used
    pub batch_size: u32,
}

/// Historical metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMetricsSummary {
    /// Time range covered (hours)
    pub hours_covered: u32,
    /// Average system health score
    pub avg_system_health: f64,
    /// Peak CPU usage
    pub peak_cpu_usage: f64,
    /// Peak memory usage
    pub peak_memory_usage: f64,
    /// Total agent tasks completed
    pub total_agent_tasks: u64,
    /// Agent health summary
    pub agent_health_summary: HashMap<String, AgentHealthSummary>,
    /// System alerts count by severity
    pub alerts_by_severity: HashMap<String, u32>,
}

/// Agent health summary for historical data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealthSummary {
    /// Average health score
    pub avg_health_score: f64,
    /// Total tasks completed
    pub total_tasks: u32,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Error count
    pub error_count: u32,
}

/// Health monitor statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitorStats {
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Total metrics collected
    pub total_metrics_collected: u64,
    /// Total alerts generated
    pub total_alerts_generated: u64,
    /// Active alerts count
    pub active_alerts_count: u32,
    /// Circuit breaker trips
    pub circuit_breaker_trips: u32,
    /// Last collection timestamp
    pub last_collection_timestamp: DateTime<Utc>,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentHealthStatus {
    /// Component is healthy
    Healthy,
    /// Component has warnings
    Warning,
    /// Component is unhealthy
    Unhealthy,
    /// Component status unknown
    Unknown,
}

/// Component health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthReport {
    /// Component name
    pub component_name: String,
    /// Health status
    pub status: ComponentHealthStatus,
    /// Health score (0-1)
    pub health_score: f64,
    /// Status message
    pub message: String,
    /// Last checked timestamp
    pub last_checked: DateTime<Utc>,
    /// Additional details
    pub details: HashMap<String, String>,
}
