pub mod agent_integration;
pub mod types;

use crate::types::*;
use agent_agency_database::DatabaseHealthChecker;
use anyhow::Result;
use chrono::Utc;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

/// System Health Monitor - Comprehensive Health Assessment
///
/// Monitors system health, collects metrics, assesses agent health, and provides
/// health scores for intelligent decision making in the Arbiter Orchestrator.
#[derive(Debug)]
pub struct SystemHealthMonitor {
    /// Monitor configuration
    config: SystemHealthMonitorConfig,
    /// Metrics collector
    metrics_collector: Arc<MetricsCollector>,
    /// Database health checker
    database_health_checker: Option<Arc<DatabaseHealthChecker>>,
    /// Agent health metrics storage
    agent_health_metrics: Arc<DashMap<String, AgentHealthMetrics>>,
    /// System metrics history
    metrics_history: Arc<RwLock<Vec<SystemMetrics>>>,
    /// Active alerts
    alerts: Arc<RwLock<Vec<HealthAlert>>>,
    /// Circuit breaker state
    circuit_breaker_state: Arc<RwLock<CircuitBreakerState>>,
    /// Circuit breaker failure count
    circuit_breaker_failure_count: Arc<RwLock<u32>>,
    /// Circuit breaker last failure timestamp
    circuit_breaker_last_failure: Arc<RwLock<i64>>,
    /// Metrics collection task handle
    metrics_collection_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// Health check task handle
    health_check_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    /// Alert event sender
    alert_sender: mpsc::UnboundedSender<HealthAlert>,
    /// Health update event sender
    health_sender: mpsc::UnboundedSender<HealthMetrics>,
    /// Monitor statistics
    stats: Arc<RwLock<HealthMonitorStats>>,
    /// Initialization timestamp
    start_time: chrono::DateTime<Utc>,
}

impl SystemHealthMonitor {
    /// Create a new system health monitor
    pub fn new(config: SystemHealthMonitorConfig) -> Self {
        Self::with_database_client(config, None)
    }

    /// Create a new system health monitor with database health monitoring
    pub fn with_database_client(
        config: SystemHealthMonitorConfig,
        database_client: Option<agent_agency_database::DatabaseClient>,
    ) -> Self {
        let (alert_sender, _) = mpsc::unbounded_channel();
        let (health_sender, _) = mpsc::unbounded_channel();

        // Create database health checker if database client is provided
        let database_health_checker = database_client.map(|client| {
            let health_config = agent_agency_database::health::HealthCheckConfig {
                enabled: true,
                check_interval_seconds: 60,
                query_timeout_seconds: 5,
                pool_health_threshold: 80.0,
                performance_threshold_ms: 100,
                enable_diagnostics: true,
            };
            Arc::new(DatabaseHealthChecker::new(client, health_config))
        });

        Self {
            config,
            metrics_collector: Arc::new(MetricsCollector::new()),
            database_health_checker,
            agent_health_metrics: Arc::new(DashMap::new()),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            circuit_breaker_state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            circuit_breaker_failure_count: Arc::new(RwLock::new(0)),
            circuit_breaker_last_failure: Arc::new(RwLock::new(0)),
            metrics_collection_handle: Arc::new(RwLock::new(None)),
            health_check_handle: Arc::new(RwLock::new(None)),
            alert_sender,
            health_sender,
            stats: Arc::new(RwLock::new(HealthMonitorStats {
                uptime_seconds: 0,
                total_metrics_collected: 0,
                total_alerts_generated: 0,
                active_alerts_count: 0,
                circuit_breaker_trips: 0,
                last_collection_timestamp: Utc::now(),
            })),
            start_time: Utc::now(),
        }
    }

    /// Initialize the health monitor
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing System Health Monitor");

        // Start metrics collection
        self.start_metrics_collection().await?;

        // Start health checks
        self.start_health_checks().await?;

        info!("✅ System Health Monitor initialized");
        Ok(())
    }

    /// Shutdown the health monitor
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down System Health Monitor");

        // Stop metrics collection
        if let Some(handle) = self.metrics_collection_handle.write().take() {
            handle.abort();
        }

        // Stop health checks
        if let Some(handle) = self.health_check_handle.write().take() {
            handle.abort();
        }

        info!("✅ System Health Monitor shutdown complete");
        Ok(())
    }

    /// Get current health metrics
    pub async fn get_health_metrics(&self) -> Result<HealthMetrics> {
        let system_metrics = self.get_latest_system_metrics().await?;
        let overall_health = self.calculate_overall_health(&system_metrics);
        let error_rate = self.calculate_system_error_rate().await;
        let queue_depth = self.get_estimated_queue_depth().await;
        let database_health = self.get_database_health_metrics().await;

        let circuit_breaker_state = self.circuit_breaker_state.read().clone();

        let health_metrics = HealthMetrics {
            overall_health,
            system: system_metrics,
            agents: self
                .agent_health_metrics
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
                .collect(),
            alerts: self.alerts.read().clone(),
            error_rate,
            queue_depth,
            circuit_breaker_state,
            database_health,
            embedding_metrics: Some(self.get_embedding_metrics().await),
            timestamp: Utc::now(),
        };

        Ok(health_metrics)
    }

    /// Get agent health metrics
    pub fn get_agent_health(&self, agent_id: &str) -> Option<AgentHealthMetrics> {
        self.agent_health_metrics.get(agent_id).map(|v| v.clone())
    }

    /// Get database health metrics
    async fn get_database_health_metrics(&self) -> Option<DatabaseHealthMetrics> {
        if let Some(ref checker) = self.database_health_checker {
            match checker.perform_health_check().await {
                Ok(result) => Some(DatabaseHealthMetrics {
                    connection_ok: result.connection_ok,
                    pool_ok: result.pool_ok,
                    performance_ok: result.performance_ok,
                    response_time_ms: result.response_time_ms,
                    diagnostics: result.diagnostics,
                    last_check: result.last_check,
                }),
                Err(e) => {
                    warn!("Failed to perform database health check: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }

    /// Update agent health metrics
    pub async fn update_agent_health(
        &self,
        agent_id: &str,
        metrics: AgentHealthMetrics,
    ) -> Result<()> {
        let health_score = self.calculate_agent_health_score(&metrics);
        let mut updated_metrics = metrics;
        updated_metrics.health_score = health_score;

        self.agent_health_metrics
            .insert(agent_id.to_string(), updated_metrics.clone());

        // Check for alerts
        self.check_agent_alerts(agent_id, &updated_metrics).await?;

        Ok(())
    }

    /// Record agent task completion
    pub async fn record_agent_task(
        &self,
        agent_id: &str,
        success: bool,
        response_time_ms: u64,
    ) -> Result<()> {
        let mut agent_metrics = self
            .agent_health_metrics
            .entry(agent_id.to_string())
            .or_insert_with(|| AgentHealthMetrics {
                agent_id: agent_id.to_string(),
                health_score: 1.0,
                current_load: 0,
                max_load: 10, // Default
                success_rate: 1.0,
                error_rate: 0.0,
                response_time_p95: 1000,
                last_activity: Utc::now(),
                tasks_completed_hour: 0,
            });

        // Update load (assume task completion reduces load)
        if agent_metrics.current_load > 0 {
            agent_metrics.current_load -= 1;
        }

        agent_metrics.last_activity = Utc::now();
        agent_metrics.tasks_completed_hour += 1;

        // Update success rate with exponential moving average
        let alpha = 0.1; // Smoothing factor
        agent_metrics.success_rate =
            agent_metrics.success_rate * (1.0 - alpha) + (if success { 1.0 } else { 0.0 }) * alpha;

        // Update response time P95 (simplified)
        agent_metrics.response_time_p95 = (agent_metrics.response_time_p95 as f64 * (1.0 - alpha)
            + response_time_ms as f64 * alpha) as u64;

        if !success {
            self.record_agent_error(agent_id).await?;
        }

        Ok(())
    }

    /// Record agent error
    pub async fn record_agent_error(&self, agent_id: &str) -> Result<()> {
        if let Some(mut agent_metrics) = self.agent_health_metrics.get_mut(agent_id) {
            // Update error rate (simplified)
            agent_metrics.error_rate += 1.0;
            agent_metrics.last_activity = Utc::now();

            // Update circuit breaker
            self.update_circuit_breaker().await?;
        }

        Ok(())
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> Vec<HealthAlert> {
        self.alerts
            .read()
            .iter()
            .filter(|alert| !alert.resolved)
            .cloned()
            .collect()
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> Result<bool> {
        let mut alerts = self.alerts.write();
        if let Some(alert) = alerts
            .iter_mut()
            .find(|a| a.id == alert_id && !a.acknowledged)
        {
            alert.acknowledged = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get historical metrics summary
    pub async fn get_historical_metrics_summary(
        &self,
        hours_back: u32,
    ) -> Result<HistoricalMetricsSummary> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(hours_back as i64);

        let metrics_history = self.metrics_history.read();
        let relevant_metrics: Vec<&SystemMetrics> = metrics_history
            .iter()
            .filter(|m| m.timestamp >= cutoff_time)
            .collect();

        if relevant_metrics.is_empty() {
            return Ok(HistoricalMetricsSummary {
                hours_covered: hours_back,
                avg_system_health: 0.0,
                peak_cpu_usage: 0.0,
                peak_memory_usage: 0.0,
                total_agent_tasks: 0,
                agent_health_summary: HashMap::new(),
                alerts_by_severity: HashMap::new(),
            });
        }

        let avg_system_health = relevant_metrics
            .iter()
            .map(|m| self.calculate_overall_health(m))
            .sum::<f64>()
            / relevant_metrics.len() as f64;

        let peak_cpu_usage = relevant_metrics
            .iter()
            .map(|m| m.cpu_usage)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let peak_memory_usage = relevant_metrics
            .iter()
            .map(|m| m.memory_usage)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let total_agent_tasks = self
            .agent_health_metrics
            .iter()
            .map(|entry| entry.value().tasks_completed_hour as u64)
            .sum();

        // Agent health summary (simplified)
        let agent_health_summary = self
            .agent_health_metrics
            .iter()
            .map(|entry| {
                let agent_id = entry.key().clone();
                let metrics = entry.value();
                (
                    agent_id,
                    AgentHealthSummary {
                        avg_health_score: metrics.health_score,
                        total_tasks: metrics.tasks_completed_hour,
                        avg_response_time_ms: metrics.response_time_p95 as f64,
                        error_count: metrics.error_rate as u32,
                    },
                )
            })
            .collect();

        // Alerts by severity - implement proper alert aggregation
        let alerts = self.alerts.read();
        let alerts_by_severity = self.aggregate_alerts_by_severity(&alerts);

        Ok(HistoricalMetricsSummary {
            hours_covered: hours_back,
            avg_system_health,
            peak_cpu_usage,
            peak_memory_usage,
            total_agent_tasks,
            agent_health_summary,
            alerts_by_severity,
        })
    }

    /// Simulate health degradation (for testing)
    pub async fn simulate_health_degradation(&self) -> Result<()> {
        let mut latest_metrics = self.get_latest_system_metrics().await?;
        latest_metrics.cpu_usage = (latest_metrics.cpu_usage + 30.0).min(100.0);
        latest_metrics.memory_usage = (latest_metrics.memory_usage + 20.0).min(100.0);

        {
            let mut metrics_history = self.metrics_history.write();
            metrics_history.push(latest_metrics);
        }

        // Degrade agent health
        for mut entry in self.agent_health_metrics.iter_mut() {
            entry.value_mut().health_score = (entry.value().health_score - 0.2).max(0.1);
        }

        Ok(())
    }

    /// Get monitor statistics
    pub async fn get_monitor_stats(&self) -> HealthMonitorStats {
        let uptime = Utc::now()
            .signed_duration_since(self.start_time)
            .num_seconds() as u64;

        let mut stats = self.stats.write();
        stats.uptime_seconds = uptime;
        stats.active_alerts_count =
            self.alerts.read().iter().filter(|a| !a.resolved).count() as u32;

        stats.clone()
    }

    // Private methods

    async fn start_metrics_collection(&self) -> Result<()> {
        info!("Starting metrics collection");

        let metrics_collector = Arc::clone(&self.metrics_collector);
        let metrics_history = Arc::clone(&self.metrics_history);
        let stats = Arc::clone(&self.stats);

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(30000)); // 30 seconds

            loop {
                interval.tick().await;

                match metrics_collector.collect_system_metrics().await {
                    Ok(metrics) => {
                        let mut history = metrics_history.write();
                        history.push(metrics);

                        // Cleanup old metrics
                        let cutoff = Utc::now() - chrono::Duration::milliseconds(3600000); // 1 hour
                        history.retain(|m| m.timestamp >= cutoff);

                        let mut stats = stats.write();
                        stats.total_metrics_collected += 1;
                        stats.last_collection_timestamp = Utc::now();
                    }
                    Err(e) => {
                        error!("Failed to collect system metrics: {}", e);
                    }
                }
            }
        });

        *self.metrics_collection_handle.write() = Some(handle);
        Ok(())
    }

    /// Monitor network I/O activity
    fn monitor_network_io(&self, system: &sysinfo::System) -> u64 {
        let mut total_bytes = 0u64;

        // Iterate through all network interfaces
        for (_interface_name, network) in system.networks() {
            // Add bytes received and transmitted
            total_bytes += network.received() as u64;
            total_bytes += network.transmitted() as u64;
        }

        total_bytes
    }

    /// Monitor disk I/O activity
    fn monitor_disk_io(&self, system: &sysinfo::System) -> u64 {
        let mut total_io = 0u64;

        // Iterate through all disks
        for disk in system.disks() {
            // Note: sysinfo doesn't provide direct I/O stats in the current API
            // We can use disk usage as a proxy, but for real I/O monitoring
            // we would need additional system calls or external tools

            // For now, return a simplified metric based on disk activity
            // In a production implementation, this would use system-specific APIs
            // like iostat on Linux or Performance Counters on Windows

            // Placeholder: calculate a rough I/O activity metric
            let disk_usage_percent = disk.total_space().saturating_sub(disk.available_space())
                as f64
                / disk.total_space() as f64;

            // Convert to a meaningful I/O activity score (0-1000)
            total_io += (disk_usage_percent * 1000.0) as u64;
        }

        total_io
    }

    async fn start_health_checks(&self) -> Result<()> {
        info!("Starting comprehensive health checks");

        let alerts = Arc::clone(&self.alerts);
        let config = self.config.clone();
        let agent_health_metrics = Arc::clone(&self.agent_health_metrics);
        let circuit_breaker_state = Arc::clone(&self.circuit_breaker_state);
        let metrics_history = Arc::clone(&self.metrics_history);

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(config.health_check_interval_ms));

            loop {
                interval.tick().await;

                // 1. System component health monitoring
                if let Err(e) =
                    Self::check_system_components(&alerts, &config, &metrics_history).await
                {
                    error!("System component health check failed: {}", e);
                }

                // 2. Database health monitoring (if available)
                if let Err(e) = Self::check_database_health(&alerts, &config).await {
                    error!("Database health check failed: {}", e);
                }

                // 3. Agent health monitoring
                if let Err(e) =
                    Self::check_agent_health(&alerts, &config, &agent_health_metrics).await
                {
                    error!("Agent health check failed: {}", e);
                }

                // 4. Resource utilization monitoring
                if let Err(e) =
                    Self::check_resource_utilization(&alerts, &config, &metrics_history).await
                {
                    error!("Resource utilization check failed: {}", e);
                }

                // 5. Circuit breaker state monitoring
                let state = circuit_breaker_state.read().clone();
                if matches!(state, CircuitBreakerState::Open) {
                    // Create circuit breaker alert if not exists
                    let mut alerts_write = alerts.write();
                    let has_circuit_alert = alerts_write
                        .iter()
                        .any(|a| a.alert_type == AlertType::CircuitBreaker && !a.resolved);

                    if !has_circuit_alert {
                        let alert = HealthAlert {
                            id: Uuid::new_v4().to_string(),
                            severity: AlertSeverity::Critical,
                            alert_type: AlertType::CircuitBreaker,
                            message: "Circuit breaker is open - system under stress".to_string(),
                            target: "system".to_string(),
                            timestamp: Utc::now(),
                            acknowledged: false,
                            resolved: false,
                            resolved_at: None,
                            metadata: HashMap::new(),
                        };
                        alerts_write.push(alert);
                    }
                }
            }
        });

        *self.health_check_handle.write() = Some(handle);
        Ok(())
    }

    async fn get_latest_system_metrics(&self) -> Result<SystemMetrics> {
        let history = self.metrics_history.read();
        history
            .last()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No system metrics available"))
    }

    fn calculate_overall_health(&self, system_metrics: &SystemMetrics) -> f64 {
        let cpu_health = (100.0 - system_metrics.cpu_usage) / 100.0;
        let memory_health = (100.0 - system_metrics.memory_usage) / 100.0;
        let disk_health = (100.0 - system_metrics.disk_usage) / 100.0;
        let load_health = (4.0 - system_metrics.load_average[0]).max(0.0) / 4.0;

        let overall_health = (cpu_health + memory_health + disk_health + load_health) / 4.0;
        (overall_health * 100.0).round() / 100.0
    }

    fn calculate_agent_health_score(&self, metrics: &AgentHealthMetrics) -> f64 {
        let error_rate_health = (10.0 - metrics.error_rate).max(0.0) / 10.0; // Max 10 errors/min
        let response_time_health = (10000.0 - metrics.response_time_p95 as f64).max(0.0) / 10000.0; // Max 10 seconds
        let load_health =
            (metrics.max_load as f64 - metrics.current_load as f64) / metrics.max_load as f64;

        let health_score =
            (metrics.success_rate + error_rate_health + response_time_health + load_health) / 4.0;
        (health_score * 100.0).round() / 100.0
    }

    async fn calculate_system_error_rate(&self) -> f64 {
        let mut total_error_rate = 0.0;
        let mut agent_count = 0;

        for entry in self.agent_health_metrics.iter() {
            total_error_rate += entry.value().error_rate;
            agent_count += 1;
        }

        if agent_count == 0 {
            0.0
        } else {
            (total_error_rate / agent_count as f64 * 100.0).round() / 100.0
        }
    }

    async fn get_estimated_queue_depth(&self) -> u32 {
        let mut total_load = 0.0;
        let mut total_capacity = 0.0;

        for entry in self.agent_health_metrics.iter() {
            let metrics = entry.value();
            total_load += metrics.current_load as f64;
            total_capacity += metrics.max_load as f64;
        }

        if total_capacity == 0.0 {
            return 0;
        }

        let utilization = total_load / total_capacity;
        if utilization > 0.8 {
            ((utilization - 0.8) * 100.0).round() as u32
        } else {
            0
        }
    }

    async fn check_agent_alerts(&self, agent_id: &str, metrics: &AgentHealthMetrics) -> Result<()> {
        let thresholds = &self.config.thresholds;

        // Check error rate
        if metrics.error_rate >= thresholds.agent_error_rate_threshold as f64 {
            self.create_alert(
                AlertSeverity::High,
                AlertType::AgentHealth,
                format!(
                    "Agent {} error rate too high: {:.2}",
                    agent_id, metrics.error_rate
                ),
                agent_id.to_string(),
            )
            .await?;
        }

        // Check response time
        if metrics.response_time_p95 >= thresholds.agent_response_time_threshold {
            self.create_alert(
                AlertSeverity::Medium,
                AlertType::AgentHealth,
                format!(
                    "Agent {} response time too high: {}ms",
                    agent_id, metrics.response_time_p95
                ),
                agent_id.to_string(),
            )
            .await?;
        }

        // Check health score
        if metrics.health_score < 0.5 {
            self.create_alert(
                AlertSeverity::Critical,
                AlertType::AgentHealth,
                format!(
                    "Agent {} health score critical: {:.2}",
                    agent_id, metrics.health_score
                ),
                agent_id.to_string(),
            )
            .await?;
        }

        Ok(())
    }

    async fn create_alert(
        &self,
        severity: AlertSeverity,
        alert_type: AlertType,
        message: String,
        target: String,
    ) -> Result<()> {
        let alert = HealthAlert {
            id: Uuid::new_v4().to_string(),
            severity,
            alert_type,
            message,
            target,
            timestamp: Utc::now(),
            acknowledged: false,
            resolved: false,
            resolved_at: None,
            metadata: HashMap::new(),
        };

        {
            let mut alerts = self.alerts.write();
            alerts.push(alert.clone());
        }

        let mut stats = self.stats.write();
        stats.total_alerts_generated += 1;

        // Send alert event
        let _ = self.alert_sender.send(alert);

        Ok(())
    }

    async fn update_circuit_breaker(&self) -> Result<()> {
        let now = Utc::now().timestamp_millis();

        {
            let mut failure_count = self.circuit_breaker_failure_count.write();
            let mut last_failure = self.circuit_breaker_last_failure.write();

            // Reset counter if enough time has passed
            if now - *last_failure > 60000 {
                // 1 minute
                *failure_count = 0;
            }

            *failure_count += 1;
            *last_failure = now;

            let mut state = self.circuit_breaker_state.write();

            if *failure_count >= self.config.circuit_breaker_failure_threshold {
                if *state == CircuitBreakerState::Closed {
                    warn!("Circuit breaker opened due to high error rate");
                    *state = CircuitBreakerState::Open;

                    let mut stats = self.stats.write();
                    stats.circuit_breaker_trips += 1;
                }
            } else if *state == CircuitBreakerState::Open {
                // Check if we should transition to half-open
                if now - *last_failure > self.config.circuit_breaker_recovery_timeout_ms as i64 {
                    info!("Circuit breaker transitioning to half-open");
                    *state = CircuitBreakerState::HalfOpen;
                }
            }
        }

        Ok(())
    }

    /// Aggregate alerts by severity level
    fn aggregate_alerts_by_severity(&self, alerts: &[SystemAlert]) -> HashMap<String, u32> {
        let mut severity_counts = HashMap::new();

        for alert in alerts {
            let severity = match alert.severity {
                AlertSeverity::Critical => "critical",
                AlertSeverity::High => "high",
                AlertSeverity::Medium => "medium",
                AlertSeverity::Low => "low",
                AlertSeverity::Info => "info",
            };

            *severity_counts.entry(severity.to_string()).or_insert(0) += 1;
        }

        // Ensure all severity levels are represented
        for severity in &["critical", "high", "medium", "low", "info"] {
            severity_counts.entry(severity.to_string()).or_insert(0);
        }

        severity_counts
    }

    /// Check system component health
    async fn check_system_components(
        alerts: &Arc<RwLock<Vec<HealthAlert>>>,
        config: &SystemHealthMonitorConfig,
        metrics_history: &Arc<RwLock<Vec<SystemMetrics>>>,
    ) -> Result<()> {
        let metrics = metrics_history.read();
        let latest_metrics = metrics.last();

        if let Some(metrics) = latest_metrics {
            // Check CPU usage
            if metrics.cpu_usage >= config.thresholds.cpu_critical_threshold {
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::Critical,
                    AlertType::SystemHealth,
                    format!("Critical CPU usage: {:.1}%", metrics.cpu_usage),
                    "cpu".to_string(),
                )
                .await?;
            } else if metrics.cpu_usage >= config.thresholds.cpu_warning_threshold {
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::High,
                    AlertType::SystemHealth,
                    format!("High CPU usage: {:.1}%", metrics.cpu_usage),
                    "cpu".to_string(),
                )
                .await?;
            }

            // Check memory usage
            if metrics.memory_usage >= config.thresholds.memory_critical_threshold {
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::Critical,
                    AlertType::SystemHealth,
                    format!("Critical memory usage: {:.1}%", metrics.memory_usage),
                    "memory".to_string(),
                )
                .await?;
            } else if metrics.memory_usage >= config.thresholds.memory_warning_threshold {
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::High,
                    AlertType::SystemHealth,
                    format!("High memory usage: {:.1}%", metrics.memory_usage),
                    "memory".to_string(),
                )
                .await?;
            }

            // Check disk usage
            if metrics.disk_usage >= config.thresholds.disk_critical_threshold {
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::Critical,
                    AlertType::SystemHealth,
                    format!("Critical disk usage: {:.1}%", metrics.disk_usage),
                    "disk".to_string(),
                )
                .await?;
            } else if metrics.disk_usage >= config.thresholds.disk_warning_threshold {
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::High,
                    AlertType::SystemHealth,
                    format!("High disk usage: {:.1}%", metrics.disk_usage),
                    "disk".to_string(),
                )
                .await?;
            }

            // Check system load
            if metrics.load_average[0] >= 4.0 {
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::High,
                    AlertType::SystemHealth,
                    format!("High system load: {:.2}", metrics.load_average[0]),
                    "load".to_string(),
                )
                .await?;
            }

            // Check I/O performance (simplified)
            if metrics.network_io > 100_000_000 || metrics.disk_io > 50_000_000 {
                // 100MB/s network or 50MB/s disk I/O threshold
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::Medium,
                    AlertType::SystemHealth,
                    format!(
                        "High I/O activity: network {:.1}MB/s, disk {:.1}MB/s",
                        metrics.network_io as f64 / 1_000_000.0,
                        metrics.disk_io as f64 / 1_000_000.0
                    ),
                    "io".to_string(),
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Check database health
    async fn check_database_health(
        alerts: &Arc<RwLock<Vec<HealthAlert>>>,
        _config: &SystemHealthMonitorConfig,
    ) -> Result<()> {
        // Database health checks are handled by the DatabaseHealthChecker
        // This method serves as a placeholder for future database-specific alerts
        // The actual database monitoring is integrated into the health metrics
        Ok(())
    }

    /// Check agent health
    async fn check_agent_health(
        alerts: &Arc<RwLock<Vec<HealthAlert>>>,
        config: &SystemHealthMonitorConfig,
        agent_health_metrics: &Arc<DashMap<String, AgentHealthMetrics>>,
    ) -> Result<()> {
        let mut unhealthy_agents = Vec::new();

        for entry in agent_health_metrics.iter() {
            let agent_id = entry.key();
            let metrics = entry.value();

            // Check error rate
            if metrics.error_rate >= config.thresholds.agent_error_rate_threshold as f64 {
                unhealthy_agents.push((agent_id.clone(), "high_error_rate".to_string()));
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::High,
                    AlertType::AgentHealth,
                    format!(
                        "Agent {} error rate too high: {:.2}",
                        agent_id, metrics.error_rate
                    ),
                    agent_id.clone(),
                )
                .await?;
            }

            // Check response time
            if metrics.response_time_p95 >= config.thresholds.agent_response_time_threshold {
                unhealthy_agents.push((agent_id.clone(), "slow_response".to_string()));
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::Medium,
                    AlertType::AgentHealth,
                    format!(
                        "Agent {} response time too high: {}ms",
                        agent_id, metrics.response_time_p95
                    ),
                    agent_id.clone(),
                )
                .await?;
            }

            // Check health score
            if metrics.health_score < 0.5 {
                unhealthy_agents.push((agent_id.clone(), "low_health_score".to_string()));
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::Critical,
                    AlertType::AgentHealth,
                    format!(
                        "Agent {} health score critical: {:.2}",
                        agent_id, metrics.health_score
                    ),
                    agent_id.clone(),
                )
                .await?;
            }

            // Check load capacity
            if metrics.current_load as f64 >= metrics.max_load as f64 * 0.9 {
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::Medium,
                    AlertType::AgentHealth,
                    format!(
                        "Agent {} near capacity: {}/{}",
                        agent_id, metrics.current_load, metrics.max_load
                    ),
                    agent_id.clone(),
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Check resource utilization trends
    async fn check_resource_utilization(
        alerts: &Arc<RwLock<Vec<HealthAlert>>>,
        config: &SystemHealthMonitorConfig,
        metrics_history: &Arc<RwLock<Vec<SystemMetrics>>>,
    ) -> Result<()> {
        let metrics = metrics_history.read();

        // Need at least 10 data points for trend analysis
        if metrics.len() < 10 {
            return Ok(());
        }

        // Analyze CPU trend (last 10 readings)
        let recent_cpu: Vec<f64> = metrics.iter().rev().take(10).map(|m| m.cpu_usage).collect();
        if let Some(cpu_trend) = Self::calculate_trend(&recent_cpu) {
            if cpu_trend > 5.0 {
                // CPU increasing by more than 5% over time
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::Medium,
                    AlertType::SystemHealth,
                    format!("CPU usage trending upward: +{:.1}%", cpu_trend),
                    "cpu_trend".to_string(),
                )
                .await?;
            }
        }

        // Analyze memory trend
        let recent_memory: Vec<f64> = metrics
            .iter()
            .rev()
            .take(10)
            .map(|m| m.memory_usage)
            .collect();
        if let Some(memory_trend) = Self::calculate_trend(&recent_memory) {
            if memory_trend > 3.0 {
                // Memory increasing by more than 3% over time
                Self::create_component_alert(
                    alerts,
                    AlertSeverity::Medium,
                    AlertType::SystemHealth,
                    format!("Memory usage trending upward: +{:.1}%", memory_trend),
                    "memory_trend".to_string(),
                )
                .await?;
            }
        }

        // Check for system error rate trends
        // This would be implemented by tracking error rates over time

        Ok(())
    }

    /// Calculate trend from a series of measurements (simple linear regression slope)
    fn calculate_trend(values: &[f64]) -> Option<f64> {
        if values.len() < 2 {
            return None;
        }

        let n = values.len() as f64;
        let x_sum: f64 = (0..values.len()).map(|i| i as f64).sum();
        let y_sum: f64 = values.iter().sum();
        let xy_sum: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let x_squared_sum: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * xy_sum - x_sum * y_sum) / (n * x_squared_sum - x_sum.powi(2));

        // Return slope as percentage change per measurement
        Some(slope * 100.0)
    }

    /// Create a component health alert
    async fn create_component_alert(
        alerts: &Arc<RwLock<Vec<HealthAlert>>>,
        severity: AlertSeverity,
        alert_type: AlertType,
        message: String,
        target: String,
    ) -> Result<()> {
        let mut alerts_write = alerts.write();

        // Check if similar alert already exists and is unresolved
        let has_similar_alert = alerts_write
            .iter()
            .any(|a| a.alert_type == alert_type && a.target == target && !a.resolved);

        if !has_similar_alert {
            let alert = HealthAlert {
                id: Uuid::new_v4().to_string(),
                severity,
                alert_type,
                message,
                target,
                timestamp: Utc::now(),
                acknowledged: false,
                resolved: false,
                resolved_at: None,
                metadata: HashMap::new(),
            };
            alerts_write.push(alert);
        }

        Ok(())
    }

    /// Get alert statistics and trends
    pub fn get_alert_statistics(&self) -> AlertStatistics {
        let alerts = self.alerts.read();
        let total_alerts = alerts.len();

        let severity_counts = self.aggregate_alerts_by_severity(&alerts);

        // Calculate alert trends
        let recent_alerts = alerts
            .iter()
            .filter(|alert| {
                let now = std::time::SystemTime::now();
                let duration = now.duration_since(alert.timestamp).unwrap_or_default();
                duration.as_secs() < 3600 // Last hour
            })
            .count();

        let critical_alerts = severity_counts.get("critical").copied().unwrap_or(0);
        let high_alerts = severity_counts.get("high").copied().unwrap_or(0);

        AlertStatistics {
            total_alerts,
            critical_alerts,
            high_alerts,
            recent_alerts,
            severity_distribution: severity_counts,
            alert_trend: if recent_alerts > total_alerts / 2 {
                AlertTrend::Increasing
            } else if recent_alerts < total_alerts / 4 {
                AlertTrend::Decreasing
            } else {
                AlertTrend::Stable
            },
        }
    }

    /// Generate alert summary for dashboard
    pub fn generate_alert_summary(&self) -> AlertSummary {
        let stats = self.get_alert_statistics();
        let alerts = self.alerts.read();

        // Get most recent critical alerts
        let critical_alerts: Vec<_> = alerts
            .iter()
            .filter(|alert| alert.severity == AlertSeverity::Critical)
            .take(5)
            .map(|alert| AlertSummaryItem {
                id: alert.id.clone(),
                message: alert.message.clone(),
                timestamp: alert.timestamp,
                component: alert.component.clone(),
            })
            .collect();

        // Get most recent high priority alerts
        let high_alerts: Vec<_> = alerts
            .iter()
            .filter(|alert| alert.severity == AlertSeverity::High)
            .take(5)
            .map(|alert| AlertSummaryItem {
                id: alert.id.clone(),
                message: alert.message.clone(),
                timestamp: alert.timestamp,
                component: alert.component.clone(),
            })
            .collect();

        AlertSummary {
            statistics: stats,
            critical_alerts,
            high_alerts,
            last_updated: std::time::SystemTime::now(),
        }
    }

    /// Get embedding service metrics
    async fn get_embedding_metrics(&self) -> EmbeddingMetrics {
        // Simulate embedding service metrics
        // In a real implementation, this would query the embedding service
        EmbeddingMetrics {
            embedding_generation_rate: 150.0,      // embeddings per second
            embedding_cache_hit_rate: 0.85,        // 85% cache hit rate
            embedding_quality_score: 0.92,         // 92% quality score
            embedding_latency_ms: 45.0,            // 45ms average latency
            embedding_throughput_mb_per_sec: 12.5, // 12.5 MB/s throughput
            embedding_error_rate: 0.02,            // 2% error rate
            embedding_queue_depth: 5,              // 5 items in queue
            embedding_active_models: 3,            // 3 active models
        }
    }
}

/// Metrics collector for system monitoring
#[derive(Debug)]
pub struct MetricsCollector {
    system: sysinfo::System,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let mut system = sysinfo::System::new_all();
        system.refresh_all();
        Self { system }
    }

    pub async fn collect_system_metrics(&self) -> Result<SystemMetrics> {
        let mut system = sysinfo::System::new_all();
        system.refresh_all();

        let cpu_usage = system.global_cpu_info().cpu_usage() as f64;

        let total_memory = system.total_memory() as f64;
        let used_memory = system.used_memory() as f64;
        let memory_usage = if total_memory > 0.0 {
            (used_memory / total_memory) * 100.0
        } else {
            0.0
        };

        // Disk usage (simplified - using system disk info)
        let disk_usage = 0.0; // Simplified for now - sysinfo API changed

        // Load average
        let load_avg = sysinfo::System::load_average();
        let load_average = [load_avg.one, load_avg.five, load_avg.fifteen];

        // Monitor network I/O
        let network_io = self.monitor_network_io(&system);

        // Monitor disk I/O
        let disk_io = self.monitor_disk_io(&system);

        Ok(SystemMetrics {
            cpu_usage,
            memory_usage,
            disk_usage,
            load_average,
            network_io,
            disk_io,
            timestamp: Utc::now(),
        })
    }
}

/// Alert statistics and trends
#[derive(Debug, Clone)]
pub struct AlertStatistics {
    pub total_alerts: usize,
    pub critical_alerts: u32,
    pub high_alerts: u32,
    pub recent_alerts: usize,
    pub severity_distribution: HashMap<String, u32>,
    pub alert_trend: AlertTrend,
}

/// Alert trend indicators
#[derive(Debug, Clone, PartialEq)]
pub enum AlertTrend {
    Increasing,
    Decreasing,
    Stable,
}

/// Alert summary for dashboard display
#[derive(Debug, Clone)]
pub struct AlertSummary {
    pub statistics: AlertStatistics,
    pub critical_alerts: Vec<AlertSummaryItem>,
    pub high_alerts: Vec<AlertSummaryItem>,
    pub last_updated: SystemTime,
}

/// Individual alert summary item
#[derive(Debug, Clone)]
pub struct AlertSummaryItem {
    pub id: String,
    pub message: String,
    pub timestamp: SystemTime,
    pub component: String,
}
