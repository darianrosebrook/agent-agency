pub mod types;

use crate::types::*;
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};
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
        let (alert_sender, _) = mpsc::unbounded_channel();
        let (health_sender, _) = mpsc::unbounded_channel();

        Self {
            config,
            metrics_collector: Arc::new(MetricsCollector::new()),
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
            embedding_metrics: None, // TODO: Integrate with embedding monitor
            timestamp: Utc::now(),
        };

        Ok(health_metrics)
    }

    /// Get agent health metrics
    pub fn get_agent_health(&self, agent_id: &str) -> Option<AgentHealthMetrics> {
        self.agent_health_metrics.get(agent_id).map(|v| v.clone())
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

        // Alerts by severity (simplified)
        let alerts = self.alerts.read();
        let alerts_by_severity = HashMap::new(); // TODO: Implement proper alert aggregation

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

    async fn start_health_checks(&self) -> Result<()> {
        info!("Starting health checks");

        let alerts = Arc::clone(&self.alerts);
        let config = self.config.clone();
        let agent_health_metrics = Arc::clone(&self.agent_health_metrics);
        let circuit_breaker_state = Arc::clone(&self.circuit_breaker_state);

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(60000)); // 1 minute

            loop {
                interval.tick().await;

                // TODO: Implement comprehensive health checks with the following requirements:
                // 1. System component health monitoring: Monitor health of all system components
                //    - Check database connectivity and query performance
                //    - Monitor API endpoints and response times
                //    - Track memory usage and garbage collection metrics
                //    - Monitor CPU utilization and thread health
                //    - Check disk space and I/O performance
                //    - Validate network connectivity and latency
                // 2. Service dependency checking: Verify all service dependencies are healthy
                //    - Check external service availability and responsiveness
                //    - Monitor message queue health and backlog
                //    - Validate authentication and authorization services
                //    - Check cache services and data consistency
                //    - Monitor background job processing and queues
                // 3. Performance metrics collection: Collect comprehensive performance metrics
                //    - Track request latency and throughput metrics
                //    - Monitor error rates and exception frequencies
                //    - Collect resource utilization statistics
                //    - Track business logic performance indicators
                //    - Monitor user experience metrics and SLIs
                // 4. Health check alerting and reporting: Implement health check alerting system
                //    - Define health check thresholds and alert conditions
                //    - Implement multi-level alerting (warning, critical, emergency)
                //    - Create health check dashboards and reporting
                //    - Support health check notification and escalation
                //    - Implement health check trend analysis and prediction
                // For now, just check circuit breaker state changes
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

        Ok(SystemMetrics {
            cpu_usage,
            memory_usage,
            disk_usage,
            load_average,
            network_io: 0, // TODO: Implement network I/O monitoring
            disk_io: 0,    // TODO: Implement disk I/O monitoring
            timestamp: Utc::now(),
        })
    }
}
