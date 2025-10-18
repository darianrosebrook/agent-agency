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
    /// Timestamp
    pub timestamp: DateTime<Utc>,
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
