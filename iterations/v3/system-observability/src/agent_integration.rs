//! Agent integration for system health monitoring
//!
//! Provides integration between the system health monitor and agent telemetry
//! for comprehensive monitoring of agent performance and coordination effectiveness.

// Note: agent_agency_observability integration is placeholder
// For now, we implement local agent tracking types
use std::collections::VecDeque;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::orchestrator::SystemHealthMonitor;

/// Agent types for performance tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentType {
    /// Reasoning and planning agent
    Reasoner,
    /// Tool execution agent
    Executor,
    /// Coordination and orchestration agent
    Coordinator,
    /// Memory and context management agent
    MemoryManager,
    /// Learning and adaptation agent
    Learner,
}

/// Performance metrics for an individual agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceMetrics {
    /// Agent identifier
    pub agent_id: String,
    /// Agent type
    pub agent_type: AgentType,
    /// Total tasks completed
    pub total_tasks_completed: u64,
    /// Total tasks failed
    pub total_tasks_failed: u64,
    /// Average response time in milliseconds
    pub average_response_time_ms: f64,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Current active tasks
    pub active_tasks: u32,
}

/// Business metrics for the overall system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    /// Total tasks processed per hour
    pub throughput_tasks_per_hour: f64,
    /// System availability percentage (0.0 to 100.0)
    pub system_availability: f64,
    /// Average task completion time in milliseconds
    pub average_task_completion_time_ms: f64,
    /// Error rate (0.0 to 1.0)
    pub error_rate: f64,
}

/// Coordination metrics for multi-agent interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationMetrics {
    /// Average time to form consensus in milliseconds
    pub average_consensus_formation_time_ms: f64,
    /// Consensus success rate (0.0 to 1.0)
    pub consensus_success_rate: f64,
    /// Percentage of tasks requiring debate (0.0 to 1.0)
    pub debate_required_percentage: f64,
}

/// Availability SLA tracking and breach detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilitySLA {
    /// Current overall availability percentage (0.0 to 100.0)
    pub overall_availability: f64,
    /// Target availability percentage
    pub target_availability: f64,
    /// Whether current availability is breaching SLA
    pub is_breaching_sla: bool,
    /// Estimated downtime in minutes over last 24 hours
    pub downtime_minutes_last_24h: u32,
}

/// System dashboard with comprehensive metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemDashboard {
    /// System health metrics
    pub system_health: SystemHealth,
    /// Agent performance metrics
    pub agent_metrics: std::collections::HashMap<String, AgentPerformanceMetrics>,
    /// Coordination metrics
    pub coordination_metrics: CoordinationMetrics,
    /// Business metrics
    pub business_metrics: BusinessMetrics,
}

/// Simplified system health enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemHealth {
    Healthy,
    Degraded,
    Critical,
    Unknown,
}

/// Placeholder alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    System,
    Agent,
    Coordination,
}

/// Placeholder alert struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemAlert {
    pub alert_type: AlertType,
    pub message: String,
    pub severity: String,
}

/// Placeholder agent telemetry collector
#[derive(Debug)]
pub struct AgentTelemetryCollector {
    agent_metrics: Arc<RwLock<std::collections::HashMap<String, AgentPerformanceMetrics>>>,
    coordination_metrics: Arc<RwLock<CoordinationMetrics>>,
    business_metrics: Arc<RwLock<BusinessMetrics>>,
}

impl AgentTelemetryCollector {
    pub fn new() -> Self {
        Self {
            agent_metrics: Arc::new(RwLock::new(std::collections::HashMap::new())),
            coordination_metrics: Arc::new(RwLock::new(CoordinationMetrics {
                average_consensus_formation_time_ms: 0.0,
                consensus_success_rate: 0.0,
                debate_required_percentage: 0.0,
            })),
            business_metrics: Arc::new(RwLock::new(BusinessMetrics {
                throughput_tasks_per_hour: 0.0,
                system_availability: 0.0,
                average_task_completion_time_ms: 0.0,
                error_rate: 0.0,
            })),
        }
    }

    pub async fn get_agent_metrics(&self, agent_id: &str) -> Option<AgentPerformanceMetrics> {
        let metrics = self.agent_metrics.read().await;
        metrics.get(agent_id).cloned()
    }

    pub async fn get_all_agent_metrics(&self) -> std::collections::HashMap<String, AgentPerformanceMetrics> {
        let metrics = self.agent_metrics.read().await;
        metrics.clone()
    }

    pub async fn get_coordination_metrics(&self) -> CoordinationMetrics {
        let metrics = self.coordination_metrics.read().await;
        metrics.clone()
    }

    pub async fn get_business_metrics(&self) -> BusinessMetrics {
        let metrics = self.business_metrics.read().await;
        metrics.clone()
    }

    pub async fn get_dashboard(&self) -> SystemDashboard {
        let agent_metrics = self.get_all_agent_metrics().await;
        let coordination_metrics = self.get_coordination_metrics().await;
        let business_metrics = self.get_business_metrics().await;

        SystemDashboard {
            system_health: SystemHealth::Healthy, // Placeholder
            agent_metrics,
            coordination_metrics,
            business_metrics,
        }
    }

    pub async fn update_business_metrics(&self, metrics: BusinessMetrics) -> anyhow::Result<()> {
        let mut business_metrics = self.business_metrics.write().await;
        *business_metrics = metrics;
        Ok(())
    }

    pub async fn update_agent_metrics(&self, agent_id: String, metrics: AgentPerformanceMetrics) -> anyhow::Result<()> {
        let mut agent_metrics = self.agent_metrics.write().await;
        agent_metrics.insert(agent_id, metrics);
        Ok(())
    }
}

/// Task completion record for throughput calculation
#[derive(Debug, Clone)]
struct TaskCompletionRecord {
    _timestamp: chrono::DateTime<chrono::Utc>,
    _agent_id: String,
    _success: bool,
}

/// Time-windowed task completion tracker for accurate throughput calculation
#[derive(Debug)]
struct TaskThroughputTracker {
    /// Recent task completions (last 24 hours)
    _completions: VecDeque<TaskCompletionRecord>,
    /// Maximum time window to keep records (24 hours)
    _max_window_duration: chrono::Duration,
}

impl TaskThroughputTracker {
    fn new() -> Self {
        Self {
            _completions: VecDeque::new(),
            _max_window_duration: chrono::Duration::hours(24),
        }
    }

    /// Record a task completion
    fn record_completion(&mut self, agent_id: String, success: bool) {
        let record = TaskCompletionRecord {
            _timestamp: chrono::Utc::now(),
            _agent_id: agent_id,
            _success: success,
        };

        self._completions.push_back(record);

        // Clean old records outside the time window
        self.cleanup_old_records();
    }





    /// Clean up records older than the maximum window
    fn cleanup_old_records(&mut self) {
        let cutoff = chrono::Utc::now() - self._max_window_duration;
        while let Some(record) = self._completions.front() {
            if record._timestamp < cutoff {
                self._completions.pop_front();
            } else {
                break;
            }
        }
    }
}

/// Tracks performance metrics for individual agents
#[derive(Debug)]
pub struct AgentPerformanceTracker {
    /// Agent identifier
    agent_id: String,
    /// Agent type
    agent_type: AgentType,
    /// Total tasks completed successfully
    total_tasks_completed: u64,
    /// Total tasks failed
    total_tasks_failed: u64,
    /// Response times for calculating averages
    response_times: VecDeque<u64>,
    /// Last activity timestamp
    last_activity: DateTime<Utc>,
    /// Current active tasks
    active_tasks: u32,
    /// Maximum response times to keep for averaging
    max_response_samples: usize,
}

impl AgentPerformanceTracker {
    /// Create a new agent performance tracker
    pub fn new(agent_id: String, agent_type: AgentType) -> Self {
        Self {
            agent_id,
            agent_type,
            total_tasks_completed: 0,
            total_tasks_failed: 0,
            response_times: VecDeque::with_capacity(100), // Keep last 100 samples
            last_activity: Utc::now(),
            active_tasks: 0,
            max_response_samples: 100,
        }
    }

    /// Record a successful task completion
    pub async fn record_task_completion(&mut self, response_time_ms: u64) -> anyhow::Result<()> {
        self.total_tasks_completed += 1;
        self.last_activity = Utc::now();
        self.active_tasks = self.active_tasks.saturating_sub(1);

        // Add response time to rolling average
        self.response_times.push_back(response_time_ms);
        if self.response_times.len() > self.max_response_samples {
            self.response_times.pop_front();
        }

        Ok(())
    }

    /// Record a task failure
    pub async fn record_task_failure(&mut self, _error: &str) -> anyhow::Result<()> {
        self.total_tasks_failed += 1;
        self.last_activity = Utc::now();
        self.active_tasks = self.active_tasks.saturating_sub(1);
        Ok(())
    }

    /// Record the start of a new task
    pub fn record_task_start(&mut self) {
        self.active_tasks += 1;
        self.last_activity = Utc::now();
    }

    /// Get current performance metrics
    pub fn get_metrics(&self) -> AgentPerformanceMetrics {
        let total_tasks = self.total_tasks_completed + self.total_tasks_failed;
        let success_rate = if total_tasks > 0 {
            self.total_tasks_completed as f64 / total_tasks as f64
        } else {
            0.0
        };

        let average_response_time_ms = if !self.response_times.is_empty() {
            self.response_times.iter().sum::<u64>() as f64 / self.response_times.len() as f64
        } else {
            0.0
        };

        AgentPerformanceMetrics {
            agent_id: self.agent_id.clone(),
            agent_type: self.agent_type.clone(),
            total_tasks_completed: self.total_tasks_completed,
            total_tasks_failed: self.total_tasks_failed,
            average_response_time_ms,
            success_rate,
            last_activity: self.last_activity,
            active_tasks: self.active_tasks,
        }
    }
}

/// Enhanced system health monitor with agent telemetry integration
#[derive(Debug)]
pub struct AgentIntegratedHealthMonitor {
    /// Base system health monitor
    _base_monitor: SystemHealthMonitor,
    /// Agent telemetry collector (placeholder)
    _telemetry_collector: Arc<AgentTelemetryCollector>,
    /// Agent performance tracking
    _agent_performance_trackers: Arc<RwLock<std::collections::HashMap<String, AgentPerformanceTracker>>>,
    /// Task throughput tracker for accurate business metrics
    _task_throughput_tracker: Arc<RwLock<TaskThroughputTracker>>,
    /// Integration configuration
    _config: AgentIntegrationConfig,
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
        let telemetry_collector = Arc::new(AgentTelemetryCollector::new());

        Self {
            _base_monitor: SystemHealthMonitor::new(base_config),
            _telemetry_collector: telemetry_collector,
            _agent_performance_trackers: Arc::new(RwLock::new(std::collections::HashMap::new())),
            _task_throughput_tracker: Arc::new(RwLock::new(TaskThroughputTracker::new())),
            _config: integration_config,
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

        // Calculate availability SLA metrics
        // For now, use a simple heuristic based on error rate
        // In production, this would track actual uptime windows
        let availability_sla = self.calculate_availability_sla().await;

        // Update metrics with SLA information
        current_metrics.system_availability = availability_sla.overall_availability;

        // TODO: Implement business-hours vs 24/7 availability distinction
        // TODO: Support multi-dimensional availability metrics (by service, region, etc.)
        // TODO: Add availability trend analysis and prediction

        self.telemetry_collector
            .update_business_metrics(current_metrics)
            .await?;
        Ok(())
    }

    /// Calculate availability SLA metrics
    async fn calculate_availability_sla(&self) -> AvailabilitySLA {
        // Get current error rate from business metrics
        let business_metrics = self.get_business_metrics().await;

        // Simple availability calculation based on error rate
        // In production, this would track actual uptime/downtime windows
        let error_rate = business_metrics.error_rate;
        let overall_availability = (1.0 - error_rate) * 100.0;

        // Target 99.9% availability (8.77 hours downtime per year)
        let target_availability = 99.9;
        let is_breaching_sla = overall_availability < target_availability;

        AvailabilitySLA {
            overall_availability,
            target_availability,
            is_breaching_sla,
            downtime_minutes_last_24h: if is_breaching_sla {
                ((100.0 - overall_availability) / 100.0 * 1440.0) as u32 // 24h in minutes
            } else {
                0
            },
        }
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
