
// ──────────────────────────────────────────────────────────────────────────────
// system_health_monitor/orchestrator.rs
// ──────────────────────────────────────────────────────────────────────────────
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use uuid::Uuid;
use tracing::{info, error};
use chrono::Utc;

use crate::types::*;
use super::metrics::MetricsCollector;
use super::core::{ResponseTimeTracker, ErrorRateTracker, RedisConnectionManager};

#[derive(Debug)]
pub struct SystemHealthMonitor {
    pub config: SystemHealthMonitorConfig,
    pub metrics_collector: Arc<MetricsCollector>,
    pub agent_health_metrics: Arc<DashMap<String, AgentHealthMetrics>>,
    pub response_time_trackers: Arc<DashMap<String, ResponseTimeTracker>>,
    pub error_rate_trackers: Arc<DashMap<String, ErrorRateTracker>>,
    pub metrics_history: Arc<RwLock<Vec<SystemMetrics>>>,
    pub alerts: Arc<RwLock<Vec<HealthAlert>>>,
    pub circuit_breaker_state: Arc<RwLock<CircuitBreakerState>>,
    pub circuit_breaker_failure_count: Arc<RwLock<u32>>,
    pub circuit_breaker_last_failure: Arc<RwLock<i64>>,
    pub metrics_collection_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    pub health_check_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    pub alert_sender: mpsc::UnboundedSender<HealthAlert>,
    pub health_sender: mpsc::UnboundedSender<HealthMetrics>,
    pub stats: Arc<RwLock<HealthMonitorStats>>,
    pub start_time: chrono::DateTime<Utc>,
    pub redis_client: Option<RedisConnectionManager>,
}

impl SystemHealthMonitor {
    pub fn new(config: SystemHealthMonitorConfig) -> Self {
        let (alert_sender, _) = mpsc::unbounded_channel();
        let (health_sender, _) = mpsc::unbounded_channel();
        Self {
            config,
            metrics_collector: Arc::new(MetricsCollector::new()),
            agent_health_metrics: Arc::new(DashMap::new()),
            response_time_trackers: Arc::new(DashMap::new()),
            error_rate_trackers: Arc::new(DashMap::new()),
            metrics_history: Arc::new(RwLock::new(Vec::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            circuit_breaker_state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            circuit_breaker_failure_count: Arc::new(RwLock::new(0)),
            circuit_breaker_last_failure: Arc::new(RwLock::new(0)),
            metrics_collection_handle: Arc::new(RwLock::new(None)),
            health_check_handle: Arc::new(RwLock::new(None)),
            alert_sender,
            health_sender,
            stats: Arc::new(RwLock::new(HealthMonitorStats::default())),
            start_time: Utc::now(),
            redis_client: None,
        }
    }

    pub async fn initialize(&self) -> anyhow::Result<()> {
        info!("Initializing System Health Monitor");
        self.start_metrics_collection().await?;
        self.start_health_checks().await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        if let Some(handle) = self.metrics_collection_handle.write().take() { handle.abort(); }
        if let Some(handle) = self.health_check_handle.write().take() { handle.abort(); }
        Ok(())
    }

    pub async fn get_health_metrics(&self) -> anyhow::Result<HealthMetrics> {
        let system_metrics = self.get_latest_system_metrics().await?;
        let overall_health = self.calculate_overall_health(&system_metrics);
        let error_rate = self.calculate_system_error_rate().await;
        let queue_depth = self.get_estimated_queue_depth().await;

        let circuit_breaker_state = self.circuit_breaker_state.read().clone();
        let database_health = None; // feature-gated in original

        Ok(HealthMetrics {
            overall_health,
            system: system_metrics,
            agents: self.agent_health_metrics.iter().map(|e| (e.key().clone(), e.value().clone())).collect(),
            alerts: self.alerts.read().clone(),
            error_rate,
            queue_depth,
            circuit_breaker_state,
            database_health,
            embedding_metrics: None,
            timestamp: Utc::now(),
        })
    }

    pub fn get_agent_health(&self, agent_id: &str) -> Option<AgentHealthMetrics> {
        self.agent_health_metrics.get(agent_id).map(|v| v.clone())
    }

    pub async fn update_agent_health(&self, agent_id: &str, mut metrics: AgentHealthMetrics) -> anyhow::Result<()> {
        metrics.health_score = self.calculate_agent_health_score(&metrics);
        self.agent_health_metrics.insert(agent_id.to_string(), metrics.clone());
        self.check_agent_alerts(agent_id, &metrics).await?;
        Ok(())
    }

    pub async fn record_agent_task(&self, agent_id: &str, success: bool, response_time_ms: u64) -> anyhow::Result<()> {
        let mut agent_metrics = self.agent_health_metrics.entry(agent_id.to_string()).or_insert_with(|| AgentHealthMetrics {
            agent_id: agent_id.to_string(),
            health_score: 1.0,
            current_load: 0,
            max_load: 10,
            success_rate: 1.0,
            error_rate: 0.0,
            response_time_p95: 1000,
            response_time_percentiles: None,
            last_activity: Utc::now(),
            tasks_completed_hour: 0,
        });

        if agent_metrics.current_load > 0 { agent_metrics.current_load -= 1; }
        agent_metrics.last_activity = Utc::now();
        agent_metrics.tasks_completed_hour += 1;
        let alpha = 0.1;
        agent_metrics.success_rate = agent_metrics.success_rate * (1.0 - alpha) + (if success {1.0} else {0.0}) * alpha;

        {
            let mut err = self.error_rate_trackers.entry(agent_id.to_string()).or_insert_with(ErrorRateTracker::new);
            err.record_request(success, None);
            agent_metrics.error_rate = err.error_rate_last_hour();
        }
        {
            let mut rt = self.response_time_trackers.entry(agent_id.to_string()).or_insert_with(|| ResponseTimeTracker::new(10_000));
            rt.record_sample(response_time_ms);
            if let Some(p95) = rt.p95_tdigest() { agent_metrics.response_time_p95 = p95 as u64; }
            agent_metrics.response_time_percentiles = rt.percentiles();
        }

        if !success { self.record_agent_error(agent_id).await?; }
        Ok(())
    }

    pub async fn record_agent_error(&self, agent_id: &str) -> anyhow::Result<()> {
        let mut err = self.error_rate_trackers.entry(agent_id.to_string()).or_insert_with(ErrorRateTracker::new);
        err.record_request(false, Some("agent_error".into()));
        if let Some(mut agent_metrics) = self.agent_health_metrics.get_mut(agent_id) {
            agent_metrics.error_rate = err.error_rate_last_hour();
            agent_metrics.last_activity = Utc::now();
        }
        self.update_circuit_breaker().await?;
        Ok(())
    }

    pub async fn get_active_alerts(&self) -> Vec<HealthAlert> {
        self.alerts.read().iter().filter(|a| !a.resolved).cloned().collect()
    }

    pub async fn acknowledge_alert(&self, alert_id: &str) -> anyhow::Result<bool> {
        let mut alerts = self.alerts.write();
        if let Some(a) = alerts.iter_mut().find(|a| a.id == alert_id && !a.acknowledged) { a.acknowledged = true; Ok(true) } else { Ok(false) }
    }

    pub async fn get_monitor_stats(&self) -> HealthMonitorStats {
        let uptime = Utc::now().signed_duration_since(self.start_time).num_seconds() as u64;
        let mut s = self.stats.write();
        s.uptime_seconds = uptime;
        s.active_alerts_count = self.alerts.read().iter().filter(|a| !a.resolved).count() as u32;
        s.clone()
    }

    // ── private helpers ──────────────────────────────────────────────────────
    async fn start_metrics_collection(&self) -> anyhow::Result<()> {
        let metrics_collector = Arc::clone(&self.metrics_collector);
        let metrics_history = Arc::clone(&self.metrics_history);
        let stats = Arc::clone(&self.stats);
        let collection_interval_ms = self.config.collection_interval_ms;

        let handle = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(collection_interval_ms as u64));
            loop {
                ticker.tick().await;
                match metrics_collector.collect_system_metrics().await {
                    Ok(metrics) => {
                        { let mut h = metrics_history.write(); h.push(metrics.clone()); if h.len() > 3600 { h.remove(0); } }
                        { let mut sg = stats.write(); sg.total_metrics_collected += 1; sg.last_collection_timestamp = Utc::now(); }
                    }
                    Err(e) => { error!("Failed to collect system metrics: {}", e); }
                }
            }
        });
        *self.metrics_collection_handle.write() = Some(handle);
        Ok(())
    }

    async fn start_health_checks(&self) -> anyhow::Result<()> {
        let alerts = Arc::clone(&self.alerts);
        let config = self.config.clone();
        let metrics_history = Arc::clone(&self.metrics_history);
        let handle = tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(config.health_check_interval_ms as u64));
            loop {
                ticker.tick().await;
                // Simple CPU/mem/disk threshold checks using the last sample
                if let Some(last) = metrics_history.read().last().cloned() {
                    if last.cpu_usage >= config.thresholds.cpu_critical_threshold {
                        let mut a = alerts.write();
                        a.push(HealthAlert { id: Uuid::new_v4().to_string(), severity: AlertSeverity::Critical, alert_type: AlertType::SystemHealth, message: format!("Critical CPU usage: {:.1}%", last.cpu_usage), target: "cpu".into(), component: "system".into(), timestamp: Utc::now(), acknowledged: false, resolved: false, resolved_at: None, metadata: HashMap::new() });
                    }
                    if last.memory_usage >= config.thresholds.memory_critical_threshold {
                        let mut a = alerts.write();
                        a.push(HealthAlert { id: Uuid::new_v4().to_string(), severity: AlertSeverity::Critical, alert_type: AlertType::SystemHealth, message: format!("Critical memory usage: {:.1}%", last.memory_usage), target: "memory".into(), component: "system".into(), timestamp: Utc::now(), acknowledged: false, resolved: false, resolved_at: None, metadata: HashMap::new() });
                    }
                }
            }
        });
        *self.health_check_handle.write() = Some(handle);
        Ok(())
    }

    async fn get_latest_system_metrics(&self) -> anyhow::Result<SystemMetrics> {
        self.metrics_history.read().last().cloned().ok_or_else(|| anyhow::anyhow!("No system metrics available"))
    }

    fn calculate_overall_health(&self, sm: &SystemMetrics) -> f64 {
        let cpu_h = (100.0 - sm.cpu_usage) / 100.0;
        let mem_h = (100.0 - sm.memory_usage) / 100.0;
        let disk_h = (100.0 - sm.disk_usage) / 100.0;
        let load_h = (4.0 - sm.load_average[0]).max(0.0) / 4.0;
        let score = (cpu_h + mem_h + disk_h + load_h) / 4.0;
        (score * 100.0).round() / 100.0
    }

    fn calculate_agent_health_score(&self, m: &AgentHealthMetrics) -> f64 {
        let success = m.success_rate; // 40%
        let error = (-m.error_rate * 10.0).exp(); // 25%
        let rt = if m.response_time_p95 < 100 { 1.0 } else if m.response_time_p95 < 500 { 0.8 } else if m.response_time_p95 < 2000 { 0.6 } else if m.response_time_p95 < 5000 { 0.3 } else { 0.1 }; // 20%
        let load = if m.max_load > 0 { (m.max_load as f64 - m.current_load as f64) / m.max_load as f64 } else { 1.0 }; // 10%
        let tput = if m.tasks_completed_hour > 10 { 1.0 } else if m.tasks_completed_hour > 5 { 0.8 } else if m.tasks_completed_hour > 1 { 0.6 } else { 0.3 }; // 5%
        let score = success*0.40 + error*0.25 + rt*0.20 + load*0.10 + tput*0.05;
        score.max(0.0).min(1.0)
    }

    async fn calculate_system_error_rate(&self) -> f64 {
        let mut total = 0.0; let mut n = 0;
        for e in self.agent_health_metrics.iter() { total += e.value().error_rate; n += 1; }
        if n == 0 { 0.0 } else { (total / n as f64 * 100.0).round() / 100.0 }
    }

    async fn get_estimated_queue_depth(&self) -> u32 {
        let mut load = 0.0; let mut cap = 0.0;
        for e in self.agent_health_metrics.iter() { let m = e.value(); load += m.current_load as f64; cap += m.max_load as f64; }
        if cap == 0.0 { return 0; }
        let util = load / cap; if util > 0.8 { ((util - 0.8) * 100.0).round() as u32 } else { 0 }
    }

    async fn check_agent_alerts(&self, agent_id: &str, m: &AgentHealthMetrics) -> anyhow::Result<()> {
        let t = &self.config.thresholds;
        if m.error_rate >= t.agent_error_rate_threshold { self.create_alert(AlertSeverity::High, AlertType::AgentHealth, format!("Agent {} error rate too high: {:.2}", agent_id, m.error_rate), agent_id.into(), "agent-health-monitor".into()).await?; }
        if m.response_time_p95 >= t.agent_response_time_threshold { self.create_alert(AlertSeverity::Medium, AlertType::AgentHealth, format!("Agent {} response time too high: {}ms", agent_id, m.response_time_p95), agent_id.into(), "agent-health-monitor".into()).await?; }
        if m.health_score < 0.5 { self.create_alert(AlertSeverity::Critical, AlertType::AgentHealth, format!("Agent {} health score critical: {:.2}", agent_id, m.health_score), agent_id.into(), "agent-health-monitor".into()).await?; }
        Ok(())
    }

    async fn create_alert(&self, severity: AlertSeverity, alert_type: AlertType, message: String, target: String, component: String) -> anyhow::Result<()> {
        let alert = HealthAlert { id: Uuid::new_v4().to_string(), severity, alert_type, message, target, component, timestamp: Utc::now(), acknowledged: false, resolved: false, resolved_at: None, metadata: HashMap::new() };
        { self.alerts.write().push(alert.clone()); }
        { let mut s = self.stats.write(); s.total_alerts_generated += 1; }
        let _ = self.alert_sender.send(alert);
        Ok(())
    }

    async fn update_circuit_breaker(&self) -> anyhow::Result<()> {
        let now = Utc::now().timestamp_millis();
        {
            let mut failure_count = self.circuit_breaker_failure_count.write();
            let mut last_failure = self.circuit_breaker_last_failure.write();
            if now - *last_failure > 60_000 { *failure_count = 0; }
            *failure_count += 1; *last_failure = now;
            let mut state = self.circuit_breaker_state.write();
            if *failure_count >= self.config.circuit_breaker_failure_threshold {
                if *state == CircuitBreakerState::Closed { *state = CircuitBreakerState::Open; let mut s = self.stats.write(); s.circuit_breaker_trips += 1; }
            } else if *state == CircuitBreakerState::Open {
                if now - *last_failure > self.config.circuit_breaker_recovery_timeout_ms as i64 { *state = CircuitBreakerState::HalfOpen; }
            }
        }
        Ok(())
    }
}
