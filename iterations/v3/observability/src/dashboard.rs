//! Real-time dashboard service for system monitoring
//!
//! Provides a web-based dashboard for real-time monitoring of agent performance,
//! system health, and business metrics.

use crate::agent_telemetry::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Real-time dashboard service
#[derive(Debug)]
pub struct DashboardService {
    /// Telemetry collector
    telemetry_collector: Arc<AgentTelemetryCollector>,
    /// Dashboard configuration
    config: DashboardConfig,
    /// Active dashboard sessions
    sessions: Arc<RwLock<HashMap<String, DashboardSession>>>,
    /// Dashboard data cache
    data_cache: Arc<RwLock<DashboardData>>,
    /// System start time for uptime calculation
    system_start_time: DateTime<Utc>,
}

/// Dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Dashboard refresh interval in seconds
    pub refresh_interval_seconds: u64,
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Enable real-time updates
    pub enable_real_time_updates: bool,
    /// Data retention period in hours
    pub data_retention_hours: u64,
    /// Enable performance metrics
    pub enable_performance_metrics: bool,
    /// Enable business intelligence
    pub enable_business_intelligence: bool,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: 5,
            max_sessions: 100,
            enable_real_time_updates: true,
            data_retention_hours: 24,
            enable_performance_metrics: true,
            enable_business_intelligence: true,
        }
    }
}

/// Dashboard session
#[derive(Debug, Clone)]
pub struct DashboardSession {
    /// Session ID
    pub session_id: String,
    /// User ID (if authenticated)
    pub user_id: Option<String>,
    /// Session start time
    pub start_time: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
    /// Session preferences
    pub preferences: DashboardPreferences,
    /// Active subscriptions
    pub subscriptions: Vec<SubscriptionType>,
}

/// Dashboard preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPreferences {
    /// Preferred refresh interval
    pub refresh_interval: u64,
    /// Show performance metrics
    pub show_performance_metrics: bool,
    /// Show business intelligence
    pub show_business_intelligence: bool,
    /// Show coordination metrics
    pub show_coordination_metrics: bool,
    /// Preferred time range for historical data
    pub time_range_hours: u64,
    /// Alert preferences
    pub alert_preferences: AlertPreferences,
}

/// Alert preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertPreferences {
    /// Enable critical alerts
    pub enable_critical_alerts: bool,
    /// Enable warning alerts
    pub enable_warning_alerts: bool,
    /// Enable info alerts
    pub enable_info_alerts: bool,
    /// Alert sound enabled
    pub alert_sound_enabled: bool,
    /// Alert desktop notifications
    pub desktop_notifications: bool,
}

impl Default for DashboardPreferences {
    fn default() -> Self {
        Self {
            refresh_interval: 5,
            show_performance_metrics: true,
            show_business_intelligence: true,
            show_coordination_metrics: true,
            time_range_hours: 24,
            alert_preferences: AlertPreferences {
                enable_critical_alerts: true,
                enable_warning_alerts: true,
                enable_info_alerts: false,
                alert_sound_enabled: true,
                desktop_notifications: true,
            },
        }
    }
}

/// Subscription types for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionType {
    /// System health updates
    SystemHealth,
    /// Agent performance updates
    AgentPerformance,
    /// Coordination metrics updates
    CoordinationMetrics,
    /// Business metrics updates
    BusinessMetrics,
    /// Alert updates
    Alerts,
    /// All updates
    All,
}

/// Dashboard data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    /// System overview
    pub system_overview: SystemOverview,
    /// Agent performance summary
    pub agent_performance: AgentPerformanceSummary,
    /// Coordination effectiveness
    pub coordination_effectiveness: CoordinationEffectiveness,
    /// Business metrics
    pub business_metrics: BusinessMetrics,
    /// Recent alerts
    pub recent_alerts: Vec<SystemAlert>,
    /// Performance trends
    pub performance_trends: PerformanceTrends,
    /// Capacity utilization
    pub capacity_utilization: CapacityMetrics,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// System overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverview {
    /// Overall system health
    pub health_status: String,
    /// Total active agents
    pub active_agents: usize,
    /// Total tasks in progress
    pub tasks_in_progress: u32,
    /// Tasks in queue
    pub tasks_in_queue: u32,
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// System load average
    pub load_average: f64,
}

/// Agent performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceSummary {
    /// Total agents
    pub total_agents: usize,
    /// Healthy agents
    pub healthy_agents: usize,
    /// Degraded agents
    pub degraded_agents: usize,
    /// Failed agents
    pub failed_agents: usize,
    /// Average success rate
    pub avg_success_rate: f64,
    /// Average response time
    pub avg_response_time_ms: u64,
    /// Top performing agents
    pub top_performers: Vec<AgentPerformanceSnapshot>,
    /// Underperforming agents
    pub underperformers: Vec<AgentPerformanceSnapshot>,
}

/// Agent performance snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceSnapshot {
    /// Agent ID
    pub agent_id: String,
    /// Agent type
    pub agent_type: String,
    /// Success rate
    pub success_rate: f64,
    /// Response time
    pub response_time_ms: u64,
    /// Health score
    pub health_score: f64,
    /// Current load
    pub current_load: u32,
}

/// Coordination effectiveness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationEffectiveness {
    /// Consensus rate
    pub consensus_rate: f64,
    /// Average consensus formation time
    pub avg_consensus_time_ms: u64,
    /// Debate frequency
    pub debate_frequency: f64,
    /// Constitutional compliance rate
    pub constitutional_compliance_rate: f64,
    /// Coordination overhead
    pub coordination_overhead_percentage: f64,
    /// Active coordination sessions
    pub active_sessions: u32,
}

impl DashboardService {
    /// Create a new dashboard service
    pub fn new(
        telemetry_collector: Arc<AgentTelemetryCollector>,
        config: DashboardConfig,
    ) -> Self {
        Self {
            telemetry_collector,
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            data_cache: Arc::new(RwLock::new(DashboardData {
                system_overview: SystemOverview {
                    health_status: "Unknown".to_string(),
                    active_agents: 0,
                    tasks_in_progress: 0,
                    tasks_in_queue: 0,
                    uptime_seconds: 0,
                    load_average: 0.0,
                },
                agent_performance: AgentPerformanceSummary {
                    total_agents: 0,
                    healthy_agents: 0,
                    degraded_agents: 0,
                    failed_agents: 0,
                    avg_success_rate: 0.0,
                    avg_response_time_ms: 0,
                    top_performers: Vec::new(),
                    underperformers: Vec::new(),
                },
                coordination_effectiveness: CoordinationEffectiveness {
                    consensus_rate: 0.0,
                    avg_consensus_time_ms: 0,
                    debate_frequency: 0.0,
                    constitutional_compliance_rate: 0.0,
                    coordination_overhead_percentage: 0.0,
                    active_sessions: 0,
                },
                business_metrics: BusinessMetrics {
                    task_completion_rate: 0.0,
                    quality_score: 0.0,
                    false_positive_rate: 0.0,
                    false_negative_rate: 0.0,
                    resource_utilization: 0.0,
                    cost_per_task: 0.0,
                    throughput_tasks_per_hour: 0.0,
                    customer_satisfaction_score: 0.0,
                    system_availability: 0.0,
                },
                recent_alerts: Vec::new(),
                performance_trends: PerformanceTrends {
                    response_time_trend: Vec::new(),
                    success_rate_trend: Vec::new(),
                    error_rate_trend: Vec::new(),
                    throughput_trend: Vec::new(),
                },
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
            system_start_time: Utc::now(),
        }
    }

    /// Start the dashboard service
    pub async fn start(&self) -> Result<()> {
        // Start data refresh task
        let service = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(service.config.refresh_interval_seconds)
            );

            loop {
                interval.tick().await;
                
                if let Err(e) = service.refresh_dashboard_data().await {
                    eprintln!("Failed to refresh dashboard data: {}", e);
                }
            }
        });

        // Start session cleanup task
        let service = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));

            loop {
                interval.tick().await;
                
                if let Err(e) = service.cleanup_expired_sessions().await {
                    eprintln!("Failed to cleanup expired sessions: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Create a new dashboard session
    pub async fn create_session(
        &self,
        user_id: Option<String>,
        preferences: Option<DashboardPreferences>,
    ) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();
        
        // Check session limit
        let sessions = self.sessions.read().await;
        if sessions.len() >= self.config.max_sessions {
            return Err(anyhow::anyhow!("Maximum number of sessions reached"));
        }
        drop(sessions);

        let session = DashboardSession {
            session_id: session_id.clone(),
            user_id,
            start_time: Utc::now(),
            last_activity: Utc::now(),
            preferences: preferences.unwrap_or_default(),
            subscriptions: vec![SubscriptionType::All],
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);

        Ok(session_id)
    }

    /// Get dashboard data for a session
    pub async fn get_dashboard_data(&self, session_id: &str) -> Result<DashboardData> {
        // Update session activity
        self.update_session_activity(session_id).await?;

        // Get cached data
        let data = self.data_cache.read().await;
        Ok(data.clone())
    }

    /// Get real-time updates for a session
    pub async fn get_real_time_updates(
        &self,
        session_id: &str,
        subscription_types: Vec<SubscriptionType>,
    ) -> Result<RealTimeUpdate> {
        // Update session activity
        self.update_session_activity(session_id).await?;

        // Get current data
        let dashboard_data = self.data_cache.read().await;
        let system_dashboard = self.telemetry_collector.get_dashboard().await;

        // Filter updates based on subscription types
        let mut update = RealTimeUpdate {
            timestamp: Utc::now(),
            system_health: None,
            agent_performance: None,
            coordination_metrics: None,
            business_metrics: None,
            alerts: None,
        };

        for subscription_type in subscription_types {
            match subscription_type {
                SubscriptionType::SystemHealth => {
                    update.system_health = Some(SystemHealthUpdate {
                        health_status: format!("{:?}", system_dashboard.system_health),
                        active_agents: system_dashboard.active_agents.len(),
                        total_tasks: system_dashboard.current_load.total_active_tasks,
                        tasks_in_queue: system_dashboard.current_load.tasks_in_queue,
                    });
                }
                SubscriptionType::AgentPerformance => {
                    update.agent_performance = Some(AgentPerformanceUpdate {
                        total_agents: dashboard_data.agent_performance.total_agents,
                        healthy_agents: dashboard_data.agent_performance.healthy_agents,
                        avg_success_rate: dashboard_data.agent_performance.avg_success_rate,
                        avg_response_time_ms: dashboard_data.agent_performance.avg_response_time_ms,
                    });
                }
                SubscriptionType::CoordinationMetrics => {
                    update.coordination_metrics = Some(CoordinationMetricsUpdate {
                        consensus_rate: dashboard_data.coordination_effectiveness.consensus_rate,
                        avg_consensus_time_ms: dashboard_data.coordination_effectiveness.avg_consensus_time_ms,
                        debate_frequency: dashboard_data.coordination_effectiveness.debate_frequency,
                        constitutional_compliance_rate: dashboard_data.coordination_effectiveness.constitutional_compliance_rate,
                    });
                }
                SubscriptionType::BusinessMetrics => {
                    update.business_metrics = Some(BusinessMetricsUpdate {
                        task_completion_rate: dashboard_data.business_metrics.task_completion_rate,
                        quality_score: dashboard_data.business_metrics.quality_score,
                        throughput_tasks_per_hour: dashboard_data.business_metrics.throughput_tasks_per_hour,
                        system_availability: dashboard_data.business_metrics.system_availability,
                    });
                }
                SubscriptionType::Alerts => {
                    update.alerts = Some(AlertsUpdate {
                        active_alerts: dashboard_data.recent_alerts.len(),
                        critical_alerts: dashboard_data.recent_alerts.iter()
                            .filter(|alert| matches!(alert.severity, AlertSeverity::Critical))
                            .count(),
                        warning_alerts: dashboard_data.recent_alerts.iter()
                            .filter(|alert| matches!(alert.severity, AlertSeverity::Warning))
                            .count(),
                    });
                }
                SubscriptionType::All => {
                    // Include all updates
                    update.system_health = Some(SystemHealthUpdate {
                        health_status: format!("{:?}", system_dashboard.system_health),
                        active_agents: system_dashboard.active_agents.len(),
                        total_tasks: system_dashboard.current_load.total_active_tasks,
                        tasks_in_queue: system_dashboard.current_load.tasks_in_queue,
                    });
                    update.agent_performance = Some(AgentPerformanceUpdate {
                        total_agents: dashboard_data.agent_performance.total_agents,
                        healthy_agents: dashboard_data.agent_performance.healthy_agents,
                        avg_success_rate: dashboard_data.agent_performance.avg_success_rate,
                        avg_response_time_ms: dashboard_data.agent_performance.avg_response_time_ms,
                    });
                    update.coordination_metrics = Some(CoordinationMetricsUpdate {
                        consensus_rate: dashboard_data.coordination_effectiveness.consensus_rate,
                        avg_consensus_time_ms: dashboard_data.coordination_effectiveness.avg_consensus_time_ms,
                        debate_frequency: dashboard_data.coordination_effectiveness.debate_frequency,
                        constitutional_compliance_rate: dashboard_data.coordination_effectiveness.constitutional_compliance_rate,
                    });
                    update.business_metrics = Some(BusinessMetricsUpdate {
                        task_completion_rate: dashboard_data.business_metrics.task_completion_rate,
                        quality_score: dashboard_data.business_metrics.quality_score,
                        throughput_tasks_per_hour: dashboard_data.business_metrics.throughput_tasks_per_hour,
                        system_availability: dashboard_data.business_metrics.system_availability,
                    });
                    update.alerts = Some(AlertsUpdate {
                        active_alerts: dashboard_data.recent_alerts.len(),
                        critical_alerts: dashboard_data.recent_alerts.iter()
                            .filter(|alert| matches!(alert.severity, AlertSeverity::Critical))
                            .count(),
                        warning_alerts: dashboard_data.recent_alerts.iter()
                            .filter(|alert| matches!(alert.severity, AlertSeverity::Warning))
                            .count(),
                    });
                }
            }
        }

        Ok(update)
    }

    /// Refresh dashboard data
    async fn refresh_dashboard_data(&self) -> Result<()> {
        let system_dashboard = self.telemetry_collector.get_dashboard().await;
        let coordination_metrics = self.telemetry_collector.get_coordination_metrics().await;
        let business_metrics = self.telemetry_collector.get_business_metrics().await;
        let agent_metrics = self.telemetry_collector.get_all_agent_metrics().await;
        let active_alerts = self.telemetry_collector.get_active_alerts().await;

        // Calculate system overview
        let system_overview = SystemOverview {
            health_status: format!("{:?}", system_dashboard.system_health),
            active_agents: system_dashboard.active_agents.len(),
            tasks_in_progress: system_dashboard.current_load.total_active_tasks,
            tasks_in_queue: system_dashboard.current_load.tasks_in_queue,
            uptime_seconds: self.calculate_system_uptime(),
            load_average: system_dashboard.capacity_utilization.cpu_utilization,
        };

        // Calculate agent performance summary
        let total_agents = agent_metrics.len();
        let healthy_agents = agent_metrics.values()
            .filter(|m| m.health_score > 0.8)
            .count();
        let degraded_agents = agent_metrics.values()
            .filter(|m| m.health_score > 0.5 && m.health_score <= 0.8)
            .count();
        let failed_agents = agent_metrics.values()
            .filter(|m| m.health_score <= 0.5)
            .count();

        let avg_success_rate = if total_agents > 0 {
            agent_metrics.values()
                .map(|m| m.success_rate)
                .sum::<f64>() / total_agents as f64
        } else {
            0.0
        };

        let avg_response_time_ms = if total_agents > 0 {
            agent_metrics.values()
                .map(|m| m.avg_response_time_ms)
                .sum::<u64>() / total_agents as u64
        } else {
            0
        };

        // Get top performers and underperformers
        let mut agent_snapshots: Vec<AgentPerformanceSnapshot> = agent_metrics.values()
            .map(|m| AgentPerformanceSnapshot {
                agent_id: m.agent_id.clone(),
                agent_type: format!("{:?}", m.agent_type),
                success_rate: m.success_rate,
                response_time_ms: m.avg_response_time_ms,
                health_score: m.health_score,
                current_load: m.current_load,
            })
            .collect();

        agent_snapshots.sort_by(|a, b| b.health_score.partial_cmp(&a.health_score).unwrap());
        let top_performers = agent_snapshots.iter().take(5).cloned().collect();
        let underperformers = agent_snapshots.iter().rev().take(5).cloned().collect();

        let agent_performance = AgentPerformanceSummary {
            total_agents,
            healthy_agents,
            degraded_agents,
            failed_agents,
            avg_success_rate,
            avg_response_time_ms,
            top_performers,
            underperformers,
        };

        // Calculate coordination effectiveness
        let coordination_effectiveness = CoordinationEffectiveness {
            consensus_rate: coordination_metrics.consensus_rate,
            avg_consensus_time_ms: coordination_metrics.consensus_formation_time_ms,
            debate_frequency: coordination_metrics.debate_frequency,
            constitutional_compliance_rate: coordination_metrics.constitutional_compliance_rate,
            coordination_overhead_percentage: coordination_metrics.coordination_overhead_percentage,
            active_sessions: coordination_metrics.active_coordination_sessions,
        };

        // Update dashboard data
        let mut data = self.data_cache.write().await;
        data.system_overview = system_overview;
        data.agent_performance = agent_performance;
        data.coordination_effectiveness = coordination_effectiveness;
        data.business_metrics = business_metrics;
        data.recent_alerts = active_alerts;
        data.performance_trends = system_dashboard.performance_trends;
        data.capacity_utilization = system_dashboard.capacity_utilization;
        data.last_updated = Utc::now();

        Ok(())
    }

    /// Update session activity
    async fn update_session_activity(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity = Utc::now();
        }
        Ok(())
    }

    /// Cleanup expired sessions
    async fn cleanup_expired_sessions(&self) -> Result<()> {
        let cutoff_time = Utc::now() - chrono::Duration::hours(self.config.data_retention_hours as i64);
        
        let mut sessions = self.sessions.write().await;
        sessions.retain(|_, session| session.last_activity > cutoff_time);
        
        Ok(())
    }

    /// Calculate system uptime in seconds
    fn calculate_system_uptime(&self) -> u64 {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.system_start_time);
        duration.num_seconds().max(0) as u64
    }
}

impl Clone for DashboardService {
    fn clone(&self) -> Self {
        Self {
            telemetry_collector: Arc::clone(&self.telemetry_collector),
            config: self.config.clone(),
            sessions: Arc::clone(&self.sessions),
            data_cache: Arc::clone(&self.data_cache),
            system_start_time: self.system_start_time,
        }
    }
}

/// Real-time update structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealTimeUpdate {
    pub timestamp: DateTime<Utc>,
    pub system_health: Option<SystemHealthUpdate>,
    pub agent_performance: Option<AgentPerformanceUpdate>,
    pub coordination_metrics: Option<CoordinationMetricsUpdate>,
    pub business_metrics: Option<BusinessMetricsUpdate>,
    pub alerts: Option<AlertsUpdate>,
}

/// System health update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthUpdate {
    pub health_status: String,
    pub active_agents: usize,
    pub total_tasks: u32,
    pub tasks_in_queue: u32,
}

/// Agent performance update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPerformanceUpdate {
    pub total_agents: usize,
    pub healthy_agents: usize,
    pub avg_success_rate: f64,
    pub avg_response_time_ms: u64,
}

/// Coordination metrics update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationMetricsUpdate {
    pub consensus_rate: f64,
    pub avg_consensus_time_ms: u64,
    pub debate_frequency: f64,
    pub constitutional_compliance_rate: f64,
}

/// Business metrics update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetricsUpdate {
    pub task_completion_rate: f64,
    pub quality_score: f64,
    pub throughput_tasks_per_hour: f64,
    pub system_availability: f64,
}

/// Alerts update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsUpdate {
    pub active_alerts: usize,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
}
