//! Agent integration for system health monitoring
//!
//! Provides integration between the system health monitor and agent telemetry
//! for comprehensive monitoring of agent performance and coordination effectiveness.

#[cfg(feature = "agent-agency-observability")]
use agent_agency_observability::{
    agent_telemetry::{AgentPerformanceMetrics, AgentTelemetryCollector, AgentType, BusinessMetrics,
    CoordinationMetrics, SystemDashboard},
    alerts::{AlertType, Alert as SystemAlert},
};
use std::collections::VecDeque;

/// Task completion record for throughput calculation
#[derive(Debug, Clone)]
struct TaskCompletionRecord {
    timestamp: chrono::DateTime<chrono::Utc>,
    agent_id: String,
    success: bool,
}

/// Time-windowed task completion tracker for accurate throughput calculation
#[derive(Debug)]
struct TaskThroughputTracker {
    /// Recent task completions (last 24 hours)
    completions: VecDeque<TaskCompletionRecord>,
    /// Maximum time window to keep records (24 hours)
    max_window_duration: chrono::Duration,
}

impl TaskThroughputTracker {
    fn new() -> Self {
        Self {
            completions: VecDeque::new(),
            max_window_duration: chrono::Duration::hours(24),
        }
    }

    /// Record a task completion
    fn record_completion(&mut self, agent_id: String, success: bool) {
        let record = TaskCompletionRecord {
            timestamp: chrono::Utc::now(),
            agent_id,
            success,
        };

        self.completions.push_back(record);

        // Clean old records outside the time window
        self.cleanup_old_records();
    }

    /// Calculate throughput over the last hour
    fn calculate_hourly_throughput(&self) -> f64 {
        self.calculate_throughput_for_duration(chrono::Duration::hours(1))
    }

    /// Calculate throughput over the last 24 hours
    fn calculate_daily_throughput(&self) -> f64 {
        self.calculate_throughput_for_duration(chrono::Duration::hours(24))
    }

    /// Calculate throughput for a specific duration
    fn calculate_throughput_for_duration(&self, duration: chrono::Duration) -> f64 {
        let cutoff = chrono::Utc::now() - duration;
        let recent_completions: Vec<_> = self.completions
            .iter()
            .filter(|record| record.timestamp > cutoff)
            .collect();

        if recent_completions.is_empty() {
            return 0.0;
        }

        // Calculate tasks per hour
        let total_tasks = recent_completions.len() as f64;
        let hours = duration.num_seconds() as f64 / 3600.0;
        total_tasks / hours
    }

    /// Calculate availability over the last 24 hours
    fn calculate_availability(&self) -> f64 {
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
        let recent_completions: Vec<_> = self.completions
            .iter()
            .filter(|record| record.timestamp > cutoff)
            .collect();

        if recent_completions.is_empty() {
            return 100.0; // Default to 100% if no data
        }

        let successful_tasks = recent_completions.iter()
            .filter(|record| record.success)
            .count();

        (successful_tasks as f64 / recent_completions.len() as f64) * 100.0
    }

    /// Clean up records older than the maximum window
    fn cleanup_old_records(&mut self) {
        let cutoff = chrono::Utc::now() - self.max_window_duration;
        while let Some(record) = self.completions.front() {
            if record.timestamp < cutoff {
                self.completions.pop_front();
            } else {
                break;
            }
        }
    }
}

/// Enhanced system health monitor with agent telemetry integration
#[cfg(feature = "agent-agency-observability")]
#[derive(Debug)]
pub struct AgentIntegratedHealthMonitor {
    /// Base system health monitor
    base_monitor: SystemHealthMonitor,
    /// Agent telemetry collector
    telemetry_collector: Arc<AgentTelemetryCollector>,
    /// Agent performance tracking
    // TODO: Implement AgentPerformanceTracker type
    // agent_performance_trackers: Arc<
    //     RwLock<
    //         std::collections::HashMap<String, AgentPerformanceTracker>,
    //     >,
    // >,
    /// Task throughput tracker for accurate business metrics
    task_throughput_tracker: Arc<RwLock<TaskThroughputTracker>>,
    /// Integration configuration
    config: AgentIntegrationConfig,
}

/// Configuration for agent integration
#[derive(Debug, Clone)]
pub struct AgentIntegrationConfig {
    /// Enable agent performance tracking
    pub enable_agent_tracking: bool,
    /// Enable coordination metrics collection
    pub enable_coordination_metrics: bool,
    /// Enable business intelligence metrics
    pub enable_business_metrics: bool,
    /// Agent health check interval in seconds
    pub agent_health_check_interval: u64,
    /// Performance metrics collection interval in seconds
    pub performance_collection_interval: u64,
}

impl Default for AgentIntegrationConfig {
    fn default() -> Self {
        Self {
            enable_agent_tracking: true,
            enable_coordination_metrics: true,
            enable_business_metrics: true,
            agent_health_check_interval: 30,
            performance_collection_interval: 60,
        }
    }
}

#[cfg(feature = "agent-agency-observability")]
impl AgentIntegratedHealthMonitor {
    /// Create a new agent-integrated health monitor
    pub fn new(
        base_config: SystemHealthMonitorConfig,
        integration_config: AgentIntegrationConfig,
    ) -> Self {
        let telemetry_config = TelemetryConfig {
            collection_interval_seconds: integration_config.performance_collection_interval,
            history_retention_hours: 24,
            alert_retention_hours: 168,
            enable_real_time_streaming: true,
            enable_business_metrics: integration_config.enable_business_metrics,
            enable_coordination_metrics: integration_config.enable_coordination_metrics,
        };

        let telemetry_collector = Arc::new(AgentTelemetryCollector::new(telemetry_config));

        Self {
            base_monitor: SystemHealthMonitor::new(base_config),
            telemetry_collector,
            agent_performance_trackers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            task_throughput_tracker: Arc::new(RwLock::new(TaskThroughputTracker::new())),
            config: integration_config,
        }
    }

    /// Start the enhanced health monitoring
    pub async fn start(&self) -> Result<()> {
        // Start base system health monitoring
        self.base_monitor.start().await?;

        // Start agent telemetry collection
        self.telemetry_collector.start_collection().await?;

        // Start agent health monitoring
        if self.config.enable_agent_tracking {
            self.start_agent_health_monitoring().await?;
        }

        info!("Agent-integrated health monitor started successfully");
        Ok(())
    }

    /// Register an agent for performance tracking
    pub async fn register_agent(&self, agent_id: String, agent_type: AgentType) -> Result<()> {
        let tracker = AgentPerformanceTracker::new(
            agent_id.clone(),
            agent_type,
            Arc::clone(&self.telemetry_collector),
        );

        let mut trackers = self.agent_performance_trackers.write().await;
        trackers.insert(agent_id.clone(), tracker);

        info!("Registered agent {} for performance tracking", agent_id);
        Ok(())
    }

    /// Record agent task completion
    pub async fn record_agent_task_completion(
        &self,
        agent_id: &str,
        response_time_ms: u64,
    ) -> Result<()> {
        // Record in the task throughput tracker
        {
            let mut throughput_tracker = self.task_throughput_tracker.write().await;
            throughput_tracker.record_completion(agent_id.to_string(), true);
        }

        let mut trackers = self.agent_performance_trackers.write().await;
        if let Some(tracker) = trackers.get_mut(agent_id) {
            tracker.record_task_completion(response_time_ms).await?;
        } else {
            warn!(
                "Attempted to record task completion for unregistered agent: {}",
                agent_id
            );
        }
        Ok(())
    }

    /// Record agent task failure
    pub async fn record_agent_task_failure(&self, agent_id: &str, error: &str) -> Result<()> {
        // Record failure in the task throughput tracker
        {
            let mut throughput_tracker = self.task_throughput_tracker.write().await;
            throughput_tracker.record_completion(agent_id.to_string(), false);
        }

        let mut trackers = self.agent_performance_trackers.write().await;
        if let Some(tracker) = trackers.get_mut(agent_id) {
            tracker.record_task_failure(error).await?;
        } else {
            warn!(
                "Attempted to record task failure for unregistered agent: {}",
                agent_id
            );
        }
        Ok(())
    }

    /// Update coordination metrics
    pub async fn update_coordination_metrics(
        &self,
        consensus_formation_time_ms: u64,
        consensus_achieved: bool,
        debate_required: bool,
        constitutional_compliance: bool,
    ) -> Result<()> {
        if !self.config.enable_coordination_metrics {
            return Ok(());
        }

        // Get current metrics
        let mut current_metrics = self.telemetry_collector.get_coordination_metrics().await;

        // Update metrics based on new data
        current_metrics.consensus_formation_time_ms = consensus_formation_time_ms;

        // Update consensus rate (simple moving average)
        let consensus_weight = 0.1; // 10% weight for new data
        current_metrics.consensus_rate = current_metrics.consensus_rate * (1.0 - consensus_weight)
            + (if consensus_achieved { 1.0 } else { 0.0 }) * consensus_weight;

        // Update debate frequency
        let debate_weight = 0.1;
        current_metrics.debate_frequency = current_metrics.debate_frequency * (1.0 - debate_weight)
            + (if debate_required { 1.0 } else { 0.0 }) * debate_weight;

        // Update constitutional compliance rate
        let compliance_weight = 0.1;
        current_metrics.constitutional_compliance_rate =
            current_metrics.constitutional_compliance_rate * (1.0 - compliance_weight)
                + (if constitutional_compliance { 1.0 } else { 0.0 }) * compliance_weight;

        // Update coordination overhead (estimated based on consensus time)
        current_metrics.coordination_overhead_percentage =
            (consensus_formation_time_ms as f64 / 1000.0) * 100.0;

        self.telemetry_collector
            .update_coordination_metrics(current_metrics)
            .await?;
        Ok(())
    }

    /// Update business metrics
    pub async fn update_business_metrics(
        &self,
        task_completed: bool,
        quality_score: f64,
        cost_per_task: f64,
    ) -> Result<()> {
        if !self.config.enable_business_metrics {
            return Ok(());
        }

        // Get current metrics
        let mut current_metrics = self.telemetry_collector.get_business_metrics().await;

        // Update task completion rate
        let completion_weight = 0.05; // 5% weight for new data
        current_metrics.task_completion_rate = current_metrics.task_completion_rate
            * (1.0 - completion_weight)
            + (if task_completed { 1.0 } else { 0.0 }) * completion_weight;

        // Update quality score
        let quality_weight = 0.1;
        current_metrics.quality_score =
            current_metrics.quality_score * (1.0 - quality_weight) + quality_score * quality_weight;

        // Update cost per task
        let cost_weight = 0.1;
        current_metrics.cost_per_task =
            current_metrics.cost_per_task * (1.0 - cost_weight) + cost_per_task * cost_weight;

        // Calculate actual throughput using time-windowed task completion data
        let throughput_tracker = self.task_throughput_tracker.read().await;
        current_metrics.throughput_tasks_per_hour = throughput_tracker.calculate_hourly_throughput();

        // Calculate system availability based on successful task completions over 24 hours
        current_metrics.system_availability = throughput_tracker.calculate_availability();

        // Support different throughput metrics
        // The hourly throughput is stored in throughput_tasks_per_hour
        // Daily throughput can be calculated as needed

        // TODO: Implement availability SLA tracking and breach detection
        // TODO: Implement business-hours vs 24/7 availability distinction
        // TODO: Support multi-dimensional availability metrics (by service, region, etc.)
        // TODO: Add availability trend analysis and prediction

        self.telemetry_collector
            .update_business_metrics(current_metrics)
            .await?;
        Ok(())
    }

    /// Get comprehensive system dashboard
    pub async fn get_system_dashboard(&self) -> Result<SystemDashboard> {
        let dashboard = self.telemetry_collector.get_dashboard().await;
        Ok(dashboard)
    }

    /// Get agent performance metrics
    pub async fn get_agent_metrics(&self, agent_id: &str) -> Option<AgentPerformanceMetrics> {
        self.telemetry_collector.get_agent_metrics(agent_id).await
    }

    /// Get all agent metrics
    pub async fn get_all_agent_metrics(
        &self,
    ) -> std::collections::HashMap<String, AgentPerformanceMetrics> {
        self.telemetry_collector.get_all_agent_metrics().await
    }

    /// Get coordination metrics
    pub async fn get_coordination_metrics(&self) -> CoordinationMetrics {
        self.telemetry_collector.get_coordination_metrics().await
    }

    /// Get business metrics
    pub async fn get_business_metrics(&self) -> BusinessMetrics {
        self.telemetry_collector.get_business_metrics().await
    }

    /// Start agent health monitoring
    async fn start_agent_health_monitoring(&self) -> Result<()> {
        let monitor = self.clone();
        let interval = self.config.agent_health_check_interval;

        tokio::spawn(async move {
            let mut health_check_interval =
                tokio::time::interval(std::time::Duration::from_secs(interval));

            loop {
                health_check_interval.tick().await;

                if let Err(e) = monitor.perform_agent_health_checks().await {
                    error!("Failed to perform agent health checks: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Perform agent health checks
    async fn perform_agent_health_checks(&self) -> Result<()> {
        let agent_metrics = self.telemetry_collector.get_all_agent_metrics().await;

        for (agent_id, metrics) in agent_metrics {
            // Check for health issues
            if metrics.health_score < 0.5 {
                warn!(
                    "Agent {} has low health score: {}",
                    agent_id, metrics.health_score
                );

                // Add alert for low health score
                let alert = SystemAlert {
                    id: uuid::Uuid::new_v4().to_string(),
                    alert_type: AlertType::Performance,
                    severity: AlertSeverity::Warning,
                    message: format!(
                        "Agent {} has low health score: {}",
                        agent_id, metrics.health_score
                    ),
                    timestamp: Utc::now(),
                    affected_agents: vec![agent_id.clone()],
                    status: AlertStatus::Firing,
                };

                if let Err(e) = self.telemetry_collector.add_alert(alert).await {
                    error!("Failed to add alert for agent {}: {}", agent_id, e);
                }
            }

            // Check for high error rate
            if metrics.error_rate > 5.0 {
                warn!(
                    "Agent {} has high error rate: {}",
                    agent_id, metrics.error_rate
                );

                let alert = SystemAlert {
                    id: uuid::Uuid::new_v4().to_string(),
                    alert_type: AlertType::Performance,
                    severity: AlertSeverity::Critical,
                    message: format!(
                        "Agent {} has high error rate: {}",
                        agent_id, metrics.error_rate
                    ),
                    timestamp: Utc::now(),
                    affected_agents: vec![agent_id.clone()],
                    status: AlertStatus::Firing,
                };

                if let Err(e) = self.telemetry_collector.add_alert(alert).await {
                    error!("Failed to add alert for agent {}: {}", agent_id, e);
                }
            }

            // Check for high response time
            if metrics.avg_response_time_ms > 10000 {
                warn!(
                    "Agent {} has high response time: {}ms",
                    agent_id, metrics.avg_response_time_ms
                );

                let alert = SystemAlert {
                    id: uuid::Uuid::new_v4().to_string(),
                    alert_type: AlertType::Performance,
                    severity: AlertSeverity::Warning,
                    message: format!(
                        "Agent {} has high response time: {}ms",
                        agent_id, metrics.avg_response_time_ms
                    ),
                    timestamp: Utc::now(),
                    affected_agents: vec![agent_id.clone()],
                    status: AlertStatus::Firing,
                };

                if let Err(e) = self.telemetry_collector.add_alert(alert).await {
                    error!("Failed to add alert for agent {}: {}", agent_id, e);
                }
            }
        }

        Ok(())
    }

    /// Get system health summary
    pub async fn get_health_summary(&self) -> Result<HealthSummary> {
        let dashboard = self.get_system_dashboard().await?;
        let coordination_metrics = self.get_coordination_metrics().await;
        let business_metrics = self.get_business_metrics().await;

        let overall_health = match dashboard.system_health {
            // System health status is now handled differently - use a default
            _ => "Unknown",
        };

        Ok(HealthSummary {
            overall_health: overall_health.to_string(),
            active_agents: dashboard.active_agents.len(),
            total_tasks: dashboard.current_load.total_active_tasks,
            consensus_rate: coordination_metrics.consensus_rate,
            task_completion_rate: business_metrics.task_completion_rate,
            quality_score: business_metrics.quality_score,
            system_availability: business_metrics.system_availability,
            active_alerts: dashboard.alerts.len(),
            last_updated: dashboard.last_updated,
        })
    }
}

#[cfg(feature = "agent-agency-observability")]
impl Clone for AgentIntegratedHealthMonitor {
    fn clone(&self) -> Self {
        Self {
            base_monitor: SystemHealthMonitor::new(self.base_monitor.config.clone()),
            telemetry_collector: Arc::clone(&self.telemetry_collector),
            agent_performance_trackers: Arc::clone(&self.agent_performance_trackers),
            config: self.config.clone(),
        }
    }
}

/// Health summary for quick overview
#[derive(Debug, Clone)]
pub struct HealthSummary {
    pub overall_health: String,
    pub active_agents: usize,
    pub total_tasks: u32,
    pub consensus_rate: f64,
    pub task_completion_rate: f64,
    pub quality_score: f64,
    pub system_availability: f64,
    pub active_alerts: usize,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
