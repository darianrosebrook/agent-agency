#![allow(warnings)] // Disables all warnings for the crate
#![allow(dead_code)] // Disables dead_code warnings for the crate

// ──────────────────────────────────────────────────────────────────────────────
// system-health-monitor/src/lib.rs
// @darianrosebrook
//
// System health monitoring and alerting infrastructure for Agent Agency V3
// ──────────────────────────────────────────────────────────────────────────────

pub mod core;
pub mod metrics;
pub mod alerts;
pub mod orchestrator;
pub mod agent_integration;
pub mod types;

// Re-exports for external users of the crate
pub use core::{
    ErrorRateTracker, ErrorStats, RedisConnectionManager, ResponseTimePercentiles, ResponseTimeTracker
};
pub use metrics::MetricsCollector;
pub use alerts::{AlertStatistics, AlertSummary, AlertSummaryItem, AlertTrend};
pub use orchestrator::SystemHealthMonitor;

#[cfg(feature = "agent-agency-observability")]
pub use agent_integration::{AgentIntegratedHealthMonitor, AgentIntegrationConfig, HealthSummary};

pub use types::*;
