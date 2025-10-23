
// ──────────────────────────────────────────────────────────────────────────────
// system_health_monitor/alerts.rs
// ──────────────────────────────────────────────────────────────────────────────
use std::collections::HashMap;
use crate::types::*;

#[derive(Debug, Clone)]
pub struct AlertStatistics {
    pub total_alerts: usize,
    pub critical_alerts: u32,
    pub high_alerts: u32,
    pub recent_alerts: usize,
    pub severity_distribution: HashMap<String, u32>,
    pub alert_trend: AlertTrend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertTrend { Increasing, Decreasing, Stable }

#[derive(Debug, Clone)]
pub struct AlertSummaryItem {
    pub id: String,
    pub message: String,
    pub timestamp: std::time::SystemTime,
    pub component: String,
}

#[derive(Debug, Clone)]
pub struct AlertSummary {
    pub statistics: AlertStatistics,
    pub critical_alerts: Vec<AlertSummaryItem>,
    pub high_alerts: Vec<AlertSummaryItem>,
    pub last_updated: std::time::SystemTime,
}

pub fn aggregate_alerts_by_severity(alerts: &[HealthAlert]) -> HashMap<String, u32> {
    let mut counts = HashMap::new();
    for a in alerts {
        let sev = match a.severity {
            AlertSeverity::Critical => "critical",
            AlertSeverity::High => "high",
            AlertSeverity::Medium => "medium",
            AlertSeverity::Low => "low",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Info => "info",
        };
        *counts.entry(sev.to_string()).or_insert(0) += 1;
    }
    for s in ["critical","high","medium","low","info"] { counts.entry(s.to_string()).or_insert(0); }
    counts
}

pub fn summarize_alerts(all: &[HealthAlert]) -> AlertSummary {
    let stats_counts = aggregate_alerts_by_severity(all);
    let total = all.len();
    let recent = all.iter().filter(|a| {
        let now = std::time::SystemTime::now();
        now.duration_since(a.timestamp.into()).map(|d| d.as_secs() < 3600).unwrap_or(false)
    }).count();

    let critical_alerts: Vec<_> = all.iter().filter(|a| a.severity == AlertSeverity::Critical)
        .take(5)
        .map(|a| AlertSummaryItem { id: a.id.clone(), message: a.message.clone(), timestamp: a.timestamp.into(), component: a.component.clone() })
        .collect();

    let high_alerts: Vec<_> = all.iter().filter(|a| a.severity == AlertSeverity::High)
        .take(5)
        .map(|a| AlertSummaryItem { id: a.id.clone(), message: a.message.clone(), timestamp: a.timestamp.into(), component: a.component.clone() })
        .collect();

    let stats = AlertStatistics {
        total_alerts: total,
        critical_alerts: *stats_counts.get("critical").unwrap_or(&0),
        high_alerts: *stats_counts.get("high").unwrap_or(&0),
        recent_alerts: recent,
        severity_distribution: stats_counts,
        alert_trend: if recent > total / 2 { AlertTrend::Increasing } else if recent < total / 4 { AlertTrend::Decreasing } else { AlertTrend::Stable },
    };

    AlertSummary { statistics: stats, critical_alerts, high_alerts, last_updated: std::time::SystemTime::now() }
}
