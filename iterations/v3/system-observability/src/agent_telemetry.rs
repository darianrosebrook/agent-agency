//! Agent-specific telemetry and performance tracking
//!
//! Provides comprehensive monitoring of individual agent performance,
//! coordination effectiveness, and system-wide metrics for production
//! observability and optimization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent types in the system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    /// Constitutional judge for CAWS compliance
    ConstitutionalJudge,
    /// Technical auditor for code quality
    TechnicalAuditor,
    /// Quality evaluator for requirements fit
    QualityEvaluator,
    /// Integration validator for system coherence
    IntegrationValidator,
    /// Research agent for context synthesis
    ResearchAgent,
    /// Generalist worker for adaptive tasks
    GeneralistWorker,
    /// Specialist worker for domain expertise
    SpecialistWorker,
}

/// Agent performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceMetrics {
    /// Unique agent identifier
    pub agent_id: String,
    /// Type of agent
    pub agent_type: AgentType,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: u64,
    /// 95th percentile response time in milliseconds
    pub p95_response_time_ms: u64,
    /// 99th percentile response time in milliseconds
    pub p99_response_time_ms: u64,
    /// Error rate (errors per minute)
    pub error_rate: f64,
    /// Total tasks completed
    pub tasks_completed: u64,
    /// Total tasks failed
    pub tasks_failed: u64,
    /// Health score (0.0 to 1.0)
    pub health_score: f64,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Current load (active tasks)
    pub current_load: u32,
    /// Maximum load capacity
    pub max_load: u32,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
}

/// Coordination effectiveness metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationMetrics {
    /// Consensus formation time in milliseconds
    pub consensus_formation_time_ms: u64,
    /// Consensus rate (successful consensus / total attempts)
    pub consensus_rate: f64,
    /// Debate frequency (debates / total evaluations)
    pub debate_frequency: f64,
    /// Constitutional compliance rate
    pub constitutional_compliance_rate: f64,
    /// Cross-agent communication latency in milliseconds
    pub cross_agent_communication_latency_ms: u64,
    /// Coordination overhead percentage
    pub coordination_overhead_percentage: f64,
    /// Number of active coordination sessions
    pub active_coordination_sessions: u32,
    /// Average coordination session duration in milliseconds
    pub avg_coordination_session_duration_ms: u64,
}

/// Business intelligence metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    /// Task completion rate
    pub task_completion_rate: f64,
    /// Overall quality score
    pub quality_score: f64,
    /// False positive rate
    pub false_positive_rate: f64,
    /// False negative rate
    pub false_negative_rate: f64,
    /// Resource utilization percentage
    pub resource_utilization: f64,
    /// Cost per task in dollars
    pub cost_per_task: f64,
    /// Throughput (tasks per hour)
    pub throughput_tasks_per_hour: f64,
    /// Customer satisfaction score
    pub customer_satisfaction_score: f64,
    /// System availability percentage
    pub system_availability: f64,
}

/// System dashboard metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDashboard {
    /// Overall system health status
    pub system_health: SystemHealthStatus,
    /// Status of all active agents
    pub active_agents: Vec<AgentStatus>,
    /// Current system load metrics
    pub current_load: LoadMetrics,
    /// Performance trends over time
    pub performance_trends: PerformanceTrends,
    /// Active system alerts
    pub alerts: Vec<SystemAlert>,
    /// Capacity utilization metrics
    pub capacity_utilization: CapacityMetrics,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemHealthStatus {
    Healthy,
    Degraded,
    Critical,
    Unknown,
}

/// Individual agent status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    /// Agent identifier
    pub agent_id: String,
    /// Agent type
    pub agent_type: AgentType,
    /// Current status
    pub status: AgentStatusType,
    /// Health score
    pub health_score: f64,
    /// Current load
    pub current_load: u32,
    /// Last activity
    pub last_activity: DateTime<Utc>,
    /// Performance metrics
    pub performance: AgentPerformanceMetrics,
}

/// Agent status types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatusType {
    Active,
    Idle,
    Busy,
    Error,
    Offline,
}

/// Load metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadMetrics {
    /// Total active tasks
    pub total_active_tasks: u32,
    /// Tasks in queue
    pub tasks_in_queue: u32,
    /// Average queue wait time in milliseconds
    pub avg_queue_wait_time_ms: u64,
    /// Peak load in last hour
    pub peak_load_last_hour: u32,
    /// Load trend (increasing/decreasing/stable)
    pub load_trend: LoadTrend,
}

/// Load trend indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadTrend {
    Increasing,
    Decreasing,
    Stable,
}

/// Performance trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    /// Response time trend over last 24 hours
    pub response_time_trend: Vec<PerformanceDataPoint>,
    /// Success rate trend over last 24 hours
    pub success_rate_trend: Vec<PerformanceDataPoint>,
    /// Error rate trend over last 24 hours
    pub error_rate_trend: Vec<PerformanceDataPoint>,
    /// Throughput trend over last 24 hours
    pub throughput_trend: Vec<PerformanceDataPoint>,
}

/// Performance data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceDataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Value
    pub value: f64,
    /// Sample count
    pub sample_count: u64,
}

/// System alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAlert {
    /// Alert identifier
    pub id: String,
    /// Alert type
    pub alert_type: AlertType,
    /// Severity level
    pub severity: AlertSeverity,
    /// Alert message
    pub message: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Affected agents
    pub affected_agents: Vec<String>,
    /// Alert status
    pub status: AlertStatus,
}

/// Alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    AgentPerformance,
    SystemHealth,
    CoordinationFailure,
    ResourceExhaustion,
    QualityDegradation,
    SecurityViolation,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    Active,
    Acknowledged,
    Resolved,
    Suppressed,
}

/// Capacity utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityMetrics {
    /// CPU utilization percentage
    pub cpu_utilization: f64,
    /// Memory utilization percentage
    pub memory_utilization: f64,
    /// Disk utilization percentage
    pub disk_utilization: f64,
    /// Network utilization percentage
    pub network_utilization: f64,
    /// Agent capacity utilization
    pub agent_capacity_utilization: f64,
    /// Available capacity percentage
    pub available_capacity: f64,
}

/// System resource monitor for tracking system metrics
#[derive(Debug)]
pub struct SystemResourceMonitor {
    /// Current system load
    current_load: u32,
    /// Maximum system load capacity
    max_load: u32,
    /// Memory usage in MB
    memory_usage_mb: f64,
    /// CPU usage percentage
    cpu_usage_percent: f64,
    /// Last update timestamp
    last_update: DateTime<Utc>,
}

impl SystemResourceMonitor {
    /// Create a new system resource monitor
    pub fn new(max_load: u32) -> Self {
        Self {
            current_load: 0,
            max_load,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
            last_update: Utc::now(),
        }
    }

    /// Update system metrics
    pub fn update_metrics(
        &mut self,
        current_load: u32,
        memory_usage_mb: f64,
        cpu_usage_percent: f64,
    ) {
        self.current_load = current_load;
        self.memory_usage_mb = memory_usage_mb;
        self.cpu_usage_percent = cpu_usage_percent;
        self.last_update = Utc::now();
    }

    /// Get current load
    pub fn current_load(&self) -> u32 {
        self.current_load
    }

    /// Get max load
    pub fn max_load(&self) -> u32 {
        self.max_load
    }

    /// Get memory usage
    pub fn memory_usage_mb(&self) -> f64 {
        self.memory_usage_mb
    }

    /// Get CPU usage
    pub fn cpu_usage_percent(&self) -> f64 {
        self.cpu_usage_percent
    }
}

/// Agent telemetry collector
#[derive(Debug)]
pub struct AgentTelemetryCollector {
    /// Agent performance metrics storage
    agent_metrics: Arc<RwLock<HashMap<String, AgentPerformanceMetrics>>>,
    /// Coordination metrics storage
    coordination_metrics: Arc<RwLock<CoordinationMetrics>>,
    /// Business metrics storage
    business_metrics: Arc<RwLock<BusinessMetrics>>,
    /// System dashboard data
    dashboard_data: Arc<RwLock<SystemDashboard>>,
    /// Performance history
    performance_history: Arc<RwLock<Vec<PerformanceDataPoint>>>,
    /// System resource monitor
    system_monitor: Arc<RwLock<SystemResourceMonitor>>,
    /// Alert storage
    alerts: Arc<RwLock<Vec<SystemAlert>>>,
    /// Configuration
    config: TelemetryConfig,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Metrics collection interval in seconds
    pub collection_interval_seconds: u64,
    /// Performance history retention in hours
    pub history_retention_hours: u64,
    /// Alert retention in hours
    pub alert_retention_hours: u64,
    /// Enable real-time streaming
    pub enable_real_time_streaming: bool,
    /// Enable business intelligence metrics
    pub enable_business_metrics: bool,
    /// Enable coordination metrics
    pub enable_coordination_metrics: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            collection_interval_seconds: 30,
            history_retention_hours: 24,
            alert_retention_hours: 168, // 1 week
            enable_real_time_streaming: true,
            enable_business_metrics: true,
            enable_coordination_metrics: true,
        }
    }
}

impl AgentTelemetryCollector {
    /// Create a new agent telemetry collector
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            agent_metrics: Arc::new(RwLock::new(HashMap::new())),
            coordination_metrics: Arc::new(RwLock::new(CoordinationMetrics {
                consensus_formation_time_ms: 0,
                consensus_rate: 0.0,
                debate_frequency: 0.0,
                constitutional_compliance_rate: 0.0,
                cross_agent_communication_latency_ms: 0,
                coordination_overhead_percentage: 0.0,
                active_coordination_sessions: 0,
                avg_coordination_session_duration_ms: 0,
            })),
            business_metrics: Arc::new(RwLock::new(BusinessMetrics {
                task_completion_rate: 0.0,
                quality_score: 0.0,
                false_positive_rate: 0.0,
                false_negative_rate: 0.0,
                resource_utilization: 0.0,
                cost_per_task: 0.0,
                throughput_tasks_per_hour: 0.0,
                customer_satisfaction_score: 0.0,
                system_availability: 0.0,
            })),
            dashboard_data: Arc::new(RwLock::new(SystemDashboard {
                system_health: SystemHealthStatus::Unknown,
                active_agents: Vec::new(),
                current_load: LoadMetrics {
                    total_active_tasks: 0,
                    tasks_in_queue: 0,
                    avg_queue_wait_time_ms: 0,
                    peak_load_last_hour: 0,
                    load_trend: LoadTrend::Stable,
                },
                performance_trends: PerformanceTrends {
                    response_time_trend: Vec::new(),
                    success_rate_trend: Vec::new(),
                    error_rate_trend: Vec::new(),
                    throughput_trend: Vec::new(),
                },
                alerts: Vec::new(),
                capacity_utilization: CapacityMetrics {
                    cpu_utilization: 0.0,
                    memory_utilization: 0.0,
                    disk_utilization: 0.0,
                    network_utilization: 0.0,
                    agent_capacity_utilization: 0.0,
                    available_capacity: 100.0,
                },
                last_updated: Utc::now(),
            })),
            performance_history: Arc::new(RwLock::new(Vec::new())),
            system_monitor: Arc::new(RwLock::new(SystemResourceMonitor::new(100))),
            alerts: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Record agent performance metrics
    pub async fn record_agent_metrics(
        &self,
        metrics: AgentPerformanceMetrics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut agent_metrics = self.agent_metrics.write().await;
        agent_metrics.insert(metrics.agent_id.clone(), metrics);
        Ok(())
    }

    /// Get agent performance metrics
    pub async fn get_agent_metrics(&self, agent_id: &str) -> Option<AgentPerformanceMetrics> {
        let agent_metrics = self.agent_metrics.read().await;
        agent_metrics.get(agent_id).cloned()
    }

    /// Get all agent metrics
    pub async fn get_all_agent_metrics(&self) -> HashMap<String, AgentPerformanceMetrics> {
        let agent_metrics = self.agent_metrics.read().await;
        agent_metrics.clone()
    }

    /// Update coordination metrics
    pub async fn update_coordination_metrics(
        &self,
        metrics: CoordinationMetrics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut coordination_metrics = self.coordination_metrics.write().await;
        *coordination_metrics = metrics;
        Ok(())
    }

    /// Get coordination metrics
    pub async fn get_coordination_metrics(&self) -> CoordinationMetrics {
        let coordination_metrics = self.coordination_metrics.read().await;
        coordination_metrics.clone()
    }

    /// Update business metrics
    pub async fn update_business_metrics(
        &self,
        metrics: BusinessMetrics,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut business_metrics = self.business_metrics.write().await;
        *business_metrics = metrics;
        Ok(())
    }

    /// Get business metrics
    pub async fn get_business_metrics(&self) -> BusinessMetrics {
        let business_metrics = self.business_metrics.read().await;
        business_metrics.clone()
    }

    /// Add system alert
    pub async fn add_alert(
        &self,
        alert: SystemAlert,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut alerts = self.alerts.write().await;
        alerts.push(alert);
        Ok(())
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<SystemAlert> {
        let alerts = self.alerts.read().await;
        alerts
            .iter()
            .filter(|alert| matches!(alert.status, AlertStatus::Active))
            .cloned()
            .collect()
    }

    /// Update system dashboard
    pub async fn update_dashboard(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut dashboard = self.dashboard_data.write().await;

        // Collect current agent statuses
        let agent_metrics = self.agent_metrics.read().await;
        let mut active_agents = Vec::new();

        for (agent_id, metrics) in agent_metrics.iter() {
            let status = if metrics.current_load > 0 {
                AgentStatusType::Busy
            } else if metrics.health_score > 0.8 {
                AgentStatusType::Active
            } else if metrics.health_score > 0.5 {
                AgentStatusType::Idle
            } else {
                AgentStatusType::Error
            };

            active_agents.push(AgentStatus {
                agent_id: agent_id.clone(),
                agent_type: metrics.agent_type.clone(),
                status,
                health_score: metrics.health_score,
                current_load: metrics.current_load,
                last_activity: metrics.last_activity,
                performance: metrics.clone(),
            });
        }

        // Update dashboard data
        dashboard.active_agents = active_agents;
        dashboard.last_updated = Utc::now();

        // Calculate system health
        let avg_health_score: f64 = agent_metrics.values().map(|m| m.health_score).sum::<f64>()
            / agent_metrics.len() as f64;

        dashboard.system_health = if avg_health_score > 0.9 {
            SystemHealthStatus::Healthy
        } else if avg_health_score > 0.7 {
            SystemHealthStatus::Degraded
        } else {
            SystemHealthStatus::Critical
        };

        Ok(())
    }

    /// Get system dashboard
    pub async fn get_dashboard(&self) -> SystemDashboard {
        let dashboard = self.dashboard_data.read().await;
        dashboard.clone()
    }

    /// Start metrics collection
    pub async fn start_collection(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let collector = self.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(
                config.collection_interval_seconds,
            ));

            loop {
                interval.tick().await;

                // Update dashboard
                if let Err(e) = collector.update_dashboard().await {
                    eprintln!("Failed to update dashboard: {}", e);
                }

                // Clean up old data
                if let Err(e) = collector.cleanup_old_data().await {
                    eprintln!("Failed to cleanup old data: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Clean up old performance data and alerts
    async fn cleanup_old_data(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let cutoff_time =
            Utc::now() - chrono::Duration::hours(self.config.history_retention_hours as i64);

        // Clean up performance history
        {
            let mut history = self.performance_history.write().await;
            history.retain(|point| point.timestamp > cutoff_time);
        }

        // Clean up old alerts
        {
            let mut alerts = self.alerts.write().await;
            alerts.retain(|alert| alert.timestamp > cutoff_time);
        }

        Ok(())
    }
}

impl Clone for AgentTelemetryCollector {
    fn clone(&self) -> Self {
        Self {
            agent_metrics: Arc::clone(&self.agent_metrics),
            coordination_metrics: Arc::clone(&self.coordination_metrics),
            business_metrics: Arc::clone(&self.business_metrics),
            dashboard_data: Arc::clone(&self.dashboard_data),
            performance_history: Arc::clone(&self.performance_history),
            system_monitor: Arc::clone(&self.system_monitor),
            alerts: Arc::clone(&self.alerts),
            config: self.config.clone(),
        }
    }
}

/// Agent performance tracker for individual agents
#[derive(Debug)]
pub struct AgentPerformanceTracker {
    /// Agent identifier
    agent_id: String,
    /// Agent type
    agent_type: AgentType,
    /// Telemetry collector
    collector: Arc<AgentTelemetryCollector>,
    /// Performance history
    response_times: Vec<u64>,
    /// Task counters
    tasks_completed: u64,
    tasks_failed: u64,
    /// Error tracking
    errors: Vec<DateTime<Utc>>,
}

impl AgentPerformanceTracker {
    /// Create a new agent performance tracker
    pub fn new(
        agent_id: String,
        agent_type: AgentType,
        collector: Arc<AgentTelemetryCollector>,
    ) -> Self {
        Self {
            agent_id,
            agent_type,
            collector,
            response_times: Vec::new(),
            tasks_completed: 0,
            tasks_failed: 0,
            errors: Vec::new(),
        }
    }

    /// Record task completion
    pub async fn record_task_completion(
        &mut self,
        response_time_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tasks_completed += 1;
        self.response_times.push(response_time_ms);

        // Keep only last 1000 response times for memory efficiency
        if self.response_times.len() > 1000 {
            self.response_times.remove(0);
        }

        self.update_metrics().await?;
        Ok(())
    }

    /// Record task failure
    pub async fn record_task_failure(
        &mut self,
        _error: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.tasks_failed += 1;
        self.errors.push(Utc::now());

        // Keep only last 100 errors for memory efficiency
        if self.errors.len() > 100 {
            self.errors.remove(0);
        }

        self.update_metrics().await?;
        Ok(())
    }

    /// Update agent metrics
    async fn update_metrics(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let total_tasks = self.tasks_completed + self.tasks_failed;
        let success_rate = if total_tasks > 0 {
            self.tasks_completed as f64 / total_tasks as f64
        } else {
            0.0
        };

        let avg_response_time = if !self.response_times.is_empty() {
            self.response_times.iter().sum::<u64>() as f64 / self.response_times.len() as f64
        } else {
            0.0
        };

        let p95_response_time = if self.response_times.len() >= 20 {
            let mut sorted_times = self.response_times.clone();
            sorted_times.sort();
            let p95_index = (sorted_times.len() as f64 * 0.95) as usize;
            sorted_times[p95_index]
        } else {
            avg_response_time as u64
        };

        let p99_response_time = if self.response_times.len() >= 100 {
            let mut sorted_times = self.response_times.clone();
            sorted_times.sort();
            let p99_index = (sorted_times.len() as f64 * 0.99) as usize;
            sorted_times[p99_index]
        } else {
            p95_response_time
        };

        // Calculate error rate (errors per minute)
        let now = Utc::now();
        let one_minute_ago = now - chrono::Duration::minutes(1);
        let recent_errors = self
            .errors
            .iter()
            .filter(|&&timestamp| timestamp > one_minute_ago)
            .count();
        let error_rate = recent_errors as f64;

        // Calculate health score based on success rate and response time
        let health_score = if success_rate > 0.95 && avg_response_time < 1000.0 {
            1.0
        } else if success_rate > 0.9 && avg_response_time < 2000.0 {
            0.8
        } else if success_rate > 0.8 && avg_response_time < 5000.0 {
            0.6
        } else if success_rate > 0.7 {
            0.4
        } else {
            0.2
        };

        let metrics = AgentPerformanceMetrics {
            agent_id: self.agent_id.clone(),
            agent_type: self.agent_type.clone(),
            success_rate,
            avg_response_time_ms: avg_response_time as u64,
            p95_response_time_ms: p95_response_time,
            p99_response_time_ms: p99_response_time,
            error_rate,
            tasks_completed: self.tasks_completed,
            tasks_failed: self.tasks_failed,
            health_score,
            last_activity: now,
            current_load: self.get_current_system_load().await,
            max_load: self.get_max_system_load().await,
            memory_usage_mb: self.get_memory_usage().await,
            cpu_usage_percent: self.get_cpu_usage().await,
        };

        self.collector.record_agent_metrics(metrics).await?;
        Ok(())
    }

    /// Get current system load
    async fn get_current_system_load(&self) -> u32 {
        let monitor = self.collector.system_monitor.read().await;
        monitor.current_load()
    }

    /// Get maximum system load
    async fn get_max_system_load(&self) -> u32 {
        let monitor = self.collector.system_monitor.read().await;
        monitor.max_load()
    }

    /// Get memory usage in MB
    async fn get_memory_usage(&self) -> f64 {
        let monitor = self.collector.system_monitor.read().await;
        monitor.memory_usage_mb()
    }

    /// Get CPU usage percentage
    async fn get_cpu_usage(&self) -> f64 {
        let monitor = self.collector.system_monitor.read().await;
        monitor.cpu_usage_percent()
    }
}
