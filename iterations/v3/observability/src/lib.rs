//! Observability crate for comprehensive monitoring and logging
//!
//! This crate provides:
//! - Structured logging with tracing
//! - SLO tracking and alerting
//! - Basic metrics collection
//! - Agent-specific telemetry and performance tracking
//! - Real-time system monitoring and business intelligence

pub mod agent_telemetry;
pub mod alerts;
pub mod analytics;
pub mod analytics_dashboard;
pub mod dashboard;
pub mod logging;
pub mod metrics;
pub mod slo;

pub use agent_telemetry::*;
pub use alerts::*;
pub use analytics::*;
pub use analytics_dashboard::*;
pub use dashboard::*;
pub use logging::*;
pub use metrics::*;
pub use slo::*;
