//! Advanced Monitoring and SLO Tracking for Agent Agency Council
//!
//! This module provides enterprise-grade SLO (Service Level Objective) tracking
//! and monitoring capabilities that integrate with the council's evaluation
//! metrics and system health monitoring infrastructure.

use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use crate::coordinator::metrics::{
    CoordinatorMetricsSnapshot, EvaluationMetrics, SLAMetrics, TimingMetrics,
    JudgePerformanceSnapshot, HealthIndicators
};
use crate::error::{CouncilError, CouncilResult};

/// Service Level Objective definition
#[derive(Debug, Clone)]
pub struct SLO {
    pub name: String,
    pub description: String,
    pub target_percentage: f64, // e.g., 99.9 for 99.9% uptime
    pub measurement_window: Duration, // e.g., 30 days
    pub component: SLOComponent,
}

/// Components that have SLOs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SLOComponent {
    CouncilEvaluation,
    JudgeInference,
    EvidenceEnrichment,
    VerdictAggregation,
    DecisionMaking,
    SystemHealth,
    CoreMLInference,
}

/// SLO measurement data point
#[derive(Debug, Clone)]
pub struct SLOMeasurement {
    pub timestamp: DateTime<Utc>,
    pub component: SLOComponent,
    pub slo_name: String,
    pub numerator: u64,    // successful events
    pub denominator: u64,  // total events
    pub compliance_percentage: f64,
    pub is_compliant: bool,
}

/// SLO compliance status
#[derive(Debug, Clone)]
pub struct SLOStatus {
    pub slo: SLO,
    pub current_compliance: f64,
    pub measurement_window_start: DateTime<Utc>,
    pub measurements: VecDeque<SLOMeasurement>,
    pub burn_rate: f64, // rate at which error budget is being consumed
    pub projected_end_of_period: DateTime<Utc>,
    pub alerts: Vec<SLOAlert>,
}

/// SLO alert conditions
#[derive(Debug, Clone)]
pub struct SLOAlert {
    pub id: String,
    pub level: AlertLevel,
    pub message: String,
    pub triggered_at: DateTime<Utc>,
    pub slo_name: String,
    pub current_compliance: f64,
    pub threshold_breached: f64,
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// SLO tracking configuration
#[derive(Debug, Clone)]
pub struct SLOTrackerConfig {
    pub measurement_window_days: i64,
    pub alert_thresholds: SLOAlertThresholds,
    pub max_measurements_per_slo: usize,
}

#[derive(Debug, Clone)]
pub struct SLOAlertThresholds {
    pub warning_threshold: f64,   // e.g., 99.0%
    pub critical_threshold: f64,  // e.g., 95.0%
    pub burn_rate_warning: f64,   // e.g., 2.0x normal burn rate
    pub burn_rate_critical: f64,  // e.g., 10.0x normal burn rate
}

/// Advanced SLO Tracker
#[derive(Debug)]
pub struct SLOTracker {
    config: SLOTrackerConfig,
    slos: HashMap<String, SLO>,
    statuses: Arc<RwLock<HashMap<String, SLOStatus>>>,
    alerts: Arc<RwLock<Vec<SLOAlert>>>,
}

impl Default for SLOTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl SLOTracker {
    /// Create a new SLO tracker with default configuration
    pub fn new() -> Self {
        let config = SLOTrackerConfig {
            measurement_window_days: 30,
            alert_thresholds: SLOAlertThresholds {
                warning_threshold: 99.0,
                critical_threshold: 95.0,
                burn_rate_warning: 2.0,
                burn_rate_critical: 10.0,
            },
            max_measurements_per_slo: 1000,
        };

        Self {
            config,
            slos: Self::default_slos(),
            statuses: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create default SLO definitions for Agent Agency
    fn default_slos() -> HashMap<String, SLO> {
        let mut slos = HashMap::new();

        // Council Evaluation SLOs
        slos.insert("council_evaluation_success".to_string(), SLO {
            name: "council_evaluation_success".to_string(),
            description: "Council evaluations complete successfully".to_string(),
            target_percentage: 99.5,
            measurement_window: Duration::days(30),
            component: SLOComponent::CouncilEvaluation,
        });

        slos.insert("council_evaluation_latency".to_string(), SLO {
            name: "council_evaluation_latency".to_string(),
            description: "Council evaluations complete within 30 seconds".to_string(),
            target_percentage: 99.0,
            measurement_window: Duration::days(30),
            component: SLOComponent::CouncilEvaluation,
        });

        // Judge Inference SLOs
        slos.insert("judge_inference_success".to_string(), SLO {
            name: "judge_inference_success".to_string(),
            description: "Judge inferences complete successfully".to_string(),
            target_percentage: 99.9,
            measurement_window: Duration::days(30),
            component: SLOComponent::JudgeInference,
        });

        slos.insert("judge_inference_latency".to_string(), SLO {
            name: "judge_inference_latency".to_string(),
            description: "Judge inferences complete within 10 seconds".to_string(),
            target_percentage: 99.5,
            measurement_window: Duration::days(30),
            component: SLOComponent::JudgeInference,
        });

        // CoreML Inference SLOs
        slos.insert("coreml_inference_success".to_string(), SLO {
            name: "coreml_inference_success".to_string(),
            description: "CoreML inferences complete successfully".to_string(),
            target_percentage: 99.9,
            measurement_window: Duration::days(30),
            component: SLOComponent::CoreMLInference,
        });

        slos.insert("coreml_inference_latency".to_string(), SLO {
            name: "coreml_inference_latency".to_string(),
            description: "CoreML inferences complete within 500ms".to_string(),
            target_percentage: 99.0,
            measurement_window: Duration::days(30),
            component: SLOComponent::CoreMLInference,
        });

        slos
    }

    /// Record metrics from a coordinator metrics snapshot
    pub async fn record_metrics(&self, metrics: &CoordinatorMetricsSnapshot) -> CouncilResult<()> {
        let mut statuses = self.statuses.write().await;

        // Record evaluation success SLO
        self.record_evaluation_success_slo(&mut statuses, metrics).await?;

        // Record evaluation latency SLO
        self.record_evaluation_latency_slo(&mut statuses, metrics).await?;

        // Record judge inference SLOs
        self.record_judge_inference_slos(&mut statuses, metrics).await?;

        // Check for alerts
        self.check_alerts(&statuses).await?;

        Ok(())
    }

    /// Record evaluation success SLO
    async fn record_evaluation_success_slo(
        &self,
        statuses: &mut HashMap<String, SLOStatus>,
        metrics: &CoordinatorMetricsSnapshot,
    ) -> CouncilResult<()> {
        let slo_name = "council_evaluation_success";
        let successful = metrics.evaluations.successful;
        let total = metrics.evaluations.total;
        let compliance = if total > 0 {
            (successful as f64 / total as f64) * 100.0
        } else {
            100.0
        };

        self.record_measurement(
            statuses,
            slo_name,
            successful,
            total,
            compliance,
            SLOComponent::CouncilEvaluation,
        ).await
    }

    /// Record evaluation latency SLO
    async fn record_evaluation_latency_slo(
        &self,
        statuses: &mut HashMap<String, SLOStatus>,
        metrics: &CoordinatorMetricsSnapshot,
    ) -> CouncilResult<()> {
        let slo_name = "council_evaluation_latency";
        let violations = metrics.sla.violations;
        let total = metrics.timing.total_evaluations;
        let compliant = total.saturating_sub(violations);
        let compliance = if total > 0 {
            (compliant as f64 / total as f64) * 100.0
        } else {
            100.0
        };

        self.record_measurement(
            statuses,
            slo_name,
            compliant,
            total,
            compliance,
            SLOComponent::CouncilEvaluation,
        ).await
    }

    /// Record judge inference SLOs
    async fn record_judge_inference_slos(
        &self,
        statuses: &mut HashMap<String, SLOStatus>,
        metrics: &CoordinatorMetricsSnapshot,
    ) -> CouncilResult<()> {
        // Judge inference success SLO
        let successful_judges = metrics.judge_performance.judge_stats
            .values()
            .map(|stats| stats.successful_evaluations)
            .sum::<u64>();
        let total_judges = metrics.judge_performance.judge_stats
            .values()
            .map(|stats| stats.total_evaluations)
            .sum::<u64>();

        if total_judges > 0 {
            let compliance = (successful_judges as f64 / total_judges as f64) * 100.0;
            self.record_measurement(
                statuses,
                "judge_inference_success",
                successful_judges,
                total_judges,
                compliance,
                SLOComponent::JudgeInference,
            ).await?;
        }

        // Judge inference latency SLO (simplified - assume SLA violations indicate latency issues)
        let total_judge_time = metrics.judge_performance.judge_stats
            .values()
            .map(|stats| stats.total_time_ms)
            .sum::<u64>();
        let avg_judge_time = if metrics.judge_performance.total_judges > 0 {
            total_judge_time / metrics.judge_performance.total_judges
        } else {
            0
        };

        // Consider it compliant if average judge time is under 10 seconds
        let compliant = if avg_judge_time < 10000 { total_judges } else { 0 };
        let compliance = if total_judges > 0 {
            (compliant as f64 / total_judges as f64) * 100.0
        } else {
            100.0
        };

        self.record_measurement(
            statuses,
            "judge_inference_latency",
            compliant,
            total_judges,
            compliance,
            SLOComponent::JudgeInference,
        ).await
    }

    /// Record a single SLO measurement
    async fn record_measurement(
        &self,
        statuses: &mut HashMap<String, SLOStatus>,
        slo_name: &str,
        numerator: u64,
        denominator: u64,
        compliance_percentage: f64,
        component: SLOComponent,
    ) -> CouncilResult<()> {
        let slo = self.slos.get(slo_name).cloned()
            .ok_or_else(|| CouncilError::InvalidInput { message: format!("Unknown SLO: {}", slo_name) })?;

        let measurement = SLOMeasurement {
            timestamp: Utc::now(),
            component,
            slo_name: slo_name.to_string(),
            numerator,
            denominator,
            compliance_percentage,
            is_compliant: compliance_percentage >= slo.target_percentage,
        };

        let status = statuses.entry(slo_name.to_string())
            .or_insert_with(|| SLOStatus {
                slo,
                current_compliance: compliance_percentage,
                measurement_window_start: Utc::now() - Duration::days(self.config.measurement_window_days),
                measurements: VecDeque::with_capacity(self.config.max_measurements_per_slo),
                burn_rate: 1.0,
                projected_end_of_period: Utc::now() + Duration::days(self.config.measurement_window_days),
                alerts: Vec::new(),
            });

        status.current_compliance = compliance_percentage;
        status.measurements.push_back(measurement);

        // Maintain measurement window
        while status.measurements.len() > self.config.max_measurements_per_slo {
            status.measurements.pop_front();
        }

        // Clean old measurements
        let cutoff = Utc::now() - Duration::days(self.config.measurement_window_days);
        while let Some(front) = status.measurements.front() {
            if front.timestamp < cutoff {
                status.measurements.pop_front();
            } else {
                break;
            }
        }

        // Calculate burn rate
        status.burn_rate = self.calculate_burn_rate(status);

        Ok(())
    }

    /// Calculate burn rate for an SLO (how fast error budget is being consumed)
    fn calculate_burn_rate(&self, status: &SLOStatus) -> f64 {
        if status.measurements.len() < 2 {
            return 1.0;
        }

        let recent_measurements: Vec<_> = status.measurements.iter()
            .rev()
            .take(10)
            .collect();

        let avg_compliance = recent_measurements.iter()
            .map(|m| m.compliance_percentage)
            .sum::<f64>() / recent_measurements.len() as f64;

        let target = status.slo.target_percentage;

        if avg_compliance >= target {
            0.0 // No burn rate if meeting target
        } else {
            (target - avg_compliance) / target
        }
    }

    /// Check for SLO alerts and generate them
    async fn check_alerts(&self, statuses: &HashMap<String, SLOStatus>) -> CouncilResult<()> {
        let mut alerts = self.alerts.write().await;

        for (slo_name, status) in statuses {
            // Compliance threshold alerts
            if status.current_compliance < self.config.alert_thresholds.critical_threshold {
                self.generate_alert(
                    &mut alerts,
                    slo_name,
                    AlertLevel::Critical,
                    format!("SLO '{}' compliance critically low: {:.2}%", slo_name, status.current_compliance),
                    status.current_compliance,
                    self.config.alert_thresholds.critical_threshold,
                ).await;
            } else if status.current_compliance < self.config.alert_thresholds.warning_threshold {
                self.generate_alert(
                    &mut alerts,
                    slo_name,
                    AlertLevel::Warning,
                    format!("SLO '{}' compliance below warning threshold: {:.2}%", slo_name, status.current_compliance),
                    status.current_compliance,
                    self.config.alert_thresholds.warning_threshold,
                ).await;
            }

            // Burn rate alerts
            if status.burn_rate > self.config.alert_thresholds.burn_rate_critical {
                self.generate_alert(
                    &mut alerts,
                    slo_name,
                    AlertLevel::Critical,
                    format!("SLO '{}' burn rate critically high: {:.2}x", slo_name, status.burn_rate),
                    status.burn_rate,
                    self.config.alert_thresholds.burn_rate_critical,
                ).await;
            } else if status.burn_rate > self.config.alert_thresholds.burn_rate_warning {
                self.generate_alert(
                    &mut alerts,
                    slo_name,
                    AlertLevel::Warning,
                    format!("SLO '{}' burn rate elevated: {:.2}x", slo_name, status.burn_rate),
                    status.burn_rate,
                    self.config.alert_thresholds.burn_rate_warning,
                ).await;
            }
        }

        Ok(())
    }

    /// Generate a new alert if one doesn't already exist
    async fn generate_alert(
        &self,
        alerts: &mut Vec<SLOAlert>,
        slo_name: &str,
        level: AlertLevel,
        message: String,
        current_value: f64,
        threshold: f64,
    ) {
        // Check if similar alert already exists and is recent
        let recent_alert_cutoff = Utc::now() - Duration::hours(1);
        let existing_alert = alerts.iter().any(|alert|
            alert.slo_name == slo_name &&
            alert.level == level &&
            alert.triggered_at > recent_alert_cutoff
        );

        if !existing_alert {
            let alert = SLOAlert {
                id: format!("alert_{}_{}", slo_name, Utc::now().timestamp()),
                level,
                message,
                triggered_at: Utc::now(),
                slo_name: slo_name.to_string(),
                current_compliance: current_value,
                threshold_breached: threshold,
            };

            info!("SLO Alert: {}", alert.message);
            alerts.push(alert);
        }
    }

    /// Get current SLO statuses
    pub async fn get_slo_statuses(&self) -> CouncilResult<HashMap<String, SLOStatus>> {
        Ok(self.statuses.read().await.clone())
    }

    /// Get active alerts
    pub async fn get_alerts(&self) -> CouncilResult<Vec<SLOAlert>> {
        Ok(self.alerts.read().await.clone())
    }

    /// Clear resolved alerts
    pub async fn clear_resolved_alerts(&self) -> CouncilResult<()> {
        let statuses = self.statuses.read().await;
        let mut alerts = self.alerts.write().await;

        alerts.retain(|alert| {
            if let Some(status) = statuses.get(&alert.slo_name) {
                match alert.level {
                    AlertLevel::Critical => {
                        status.current_compliance < self.config.alert_thresholds.critical_threshold ||
                        status.burn_rate > self.config.alert_thresholds.burn_rate_critical
                    }
                    AlertLevel::Warning => {
                        status.current_compliance < self.config.alert_thresholds.warning_threshold ||
                        status.burn_rate > self.config.alert_thresholds.burn_rate_warning
                    }
                    AlertLevel::Info => true, // Info alerts don't auto-resolve
                }
            } else {
                false // Remove alerts for non-existent SLOs
            }
        });

        Ok(())
    }

    /// Get SLO dashboard summary
    pub async fn get_dashboard_summary(&self) -> CouncilResult<SLODashboardSummary> {
        let statuses = self.statuses.read().await;
        let alerts = self.alerts.read().await;

        let mut component_summaries = HashMap::new();
        let mut critical_alerts = 0;
        let mut warning_alerts = 0;

        for alert in &*alerts {
            match alert.level {
                AlertLevel::Critical => critical_alerts += 1,
                AlertLevel::Warning => warning_alerts += 1,
                AlertLevel::Info => {}
            }
        }

        for (slo_name, status) in &*statuses {
            let component = &status.slo.component;
            let summary = component_summaries.entry(component.clone())
                .or_insert_with(|| SLOComponentSummary {
                    component: component.clone(),
                    total_slos: 0,
                    compliant_slos: 0,
                    average_compliance: 0.0,
                    burn_rate_sum: 0.0,
                });

            summary.total_slos += 1;
            if status.current_compliance >= status.slo.target_percentage {
                summary.compliant_slos += 1;
            }
            summary.average_compliance += status.current_compliance;
            summary.burn_rate_sum += status.burn_rate;
        }

        for summary in component_summaries.values_mut() {
            if summary.total_slos > 0 {
                summary.average_compliance /= summary.total_slos as f64;
                summary.burn_rate_sum /= summary.total_slos as f64;
            }
        }

        Ok(SLODashboardSummary {
            overall_health_score: self.calculate_overall_health_score(&statuses),
            total_slos: statuses.len(),
            compliant_slos: statuses.values()
                .filter(|s| s.current_compliance >= s.slo.target_percentage)
                .count(),
            critical_alerts,
            warning_alerts,
            component_summaries,
            last_updated: Utc::now(),
        })
    }

    /// Calculate overall health score (0-100)
    fn calculate_overall_health_score(&self, statuses: &HashMap<String, SLOStatus>) -> f64 {
        if statuses.is_empty() {
            return 100.0;
        }

        let mut total_weighted_score = 0.0;
        let mut total_weight = 0.0;

        for status in statuses.values() {
            let weight = match status.slo.component {
                SLOComponent::CouncilEvaluation => 3.0,
                SLOComponent::JudgeInference => 3.0,
                SLOComponent::CoreMLInference => 2.0,
                _ => 1.0,
            };

            let compliance_ratio = (status.current_compliance / 100.0).min(1.0);
            let burn_rate_penalty = (status.burn_rate / 10.0).min(1.0);

            let score = compliance_ratio * (1.0 - burn_rate_penalty) * 100.0;
            total_weighted_score += score * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            total_weighted_score / total_weight
        } else {
            100.0
        }
    }
}

/// SLO Dashboard Summary
#[derive(Debug, Clone)]
pub struct SLODashboardSummary {
    pub overall_health_score: f64,
    pub total_slos: usize,
    pub compliant_slos: usize,
    pub critical_alerts: usize,
    pub warning_alerts: usize,
    pub component_summaries: HashMap<SLOComponent, SLOComponentSummary>,
    pub last_updated: DateTime<Utc>,
}

/// Component-specific SLO summary
#[derive(Debug, Clone)]
pub struct SLOComponentSummary {
    pub component: SLOComponent,
    pub total_slos: usize,
    pub compliant_slos: usize,
    pub average_compliance: f64,
    pub burn_rate_sum: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_slo_tracker_basic_functionality() {
        let tracker = SLOTracker::new();

        // Test default SLOs are loaded
        assert!(tracker.slos.contains_key("council_evaluation_success"));
        assert!(tracker.slos.contains_key("judge_inference_success"));
        assert!(tracker.slos.contains_key("coreml_inference_success"));

        // Test initial status
        let statuses = tracker.get_slo_statuses().await.unwrap();
        assert!(statuses.is_empty()); // No measurements yet
    }

    #[tokio::test]
    async fn test_slo_measurement_recording() {
        let tracker = SLOTracker::new();

        // Create mock metrics
        let metrics = CoordinatorMetricsSnapshot {
            timestamp: Utc::now(),
            uptime_seconds: 3600,
            evaluations: EvaluationMetrics {
                total: 100,
                successful: 98,
                failed: 2,
                success_rate: 98.0,
            },
            timing: TimingMetrics {
                total_evaluations: 100,
                successful_evaluations: 98,
                failed_evaluations: 2,
                total_evaluation_time_ms: 300000,
                total_enrichment_time_ms: 50000,
                total_judge_inference_time_ms: 200000,
                total_debate_time_ms: 50000,
                sla_violations: 5,
                average_evaluation_time_ms: 3000,
                average_enrichment_time_ms: 500,
                average_judge_inference_time_ms: 2000,
                average_debate_time_ms: 500,
            },
            sla: SLAMetrics {
                violations: 5,
                violation_rate: 5.0,
                threshold_ms: 30000,
            },
            judge_performance: JudgePerformanceSnapshot {
                judge_stats: {
                    let mut stats = HashMap::new();
                    stats.insert("ethics".to_string(), crate::coordinator::metrics::JudgePerformanceStats {
                        total_evaluations: 50,
                        successful_evaluations: 49,
                        average_confidence: 0.9,
                        total_time_ms: 100000,
                    });
                    stats.insert("quality".to_string(), crate::coordinator::metrics::JudgePerformanceStats {
                        total_evaluations: 50,
                        successful_evaluations: 49,
                        average_confidence: 0.85,
                        total_time_ms: 100000,
                    });
                    stats
                },
                total_judges: 2,
                average_confidence: 0.875,
            },
            health: HealthIndicators {
                active_evaluations: 5,
                queue_depth: 10,
                error_rate: 2.0,
            },
        };

        // Record metrics
        tracker.record_metrics(&metrics).await.unwrap();

        // Check SLO statuses
        let statuses = tracker.get_slo_statuses().await.unwrap();
        assert!(!statuses.is_empty());

        // Check evaluation success SLO
        let eval_success = statuses.get("council_evaluation_success").unwrap();
        assert_eq!(eval_success.current_compliance, 98.0);

        // Check evaluation latency SLO
        let eval_latency = statuses.get("council_evaluation_latency").unwrap();
        assert_eq!(eval_latency.current_compliance, 95.0); // 95% compliant (95/100)

        // Check judge inference SLO
        let judge_success = statuses.get("judge_inference_success").unwrap();
        assert_eq!(judge_success.current_compliance, 98.0); // 98% successful
    }
}
