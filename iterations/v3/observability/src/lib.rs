//! Observability crate for comprehensive monitoring and logging
//!
//! This crate provides:
//! - Structured logging with tracing
//! - SLO tracking and alerting
//! - Basic metrics collection

pub mod alerts;
pub mod logging;
pub mod metrics;
pub mod slo;

pub use alerts::*;
pub use logging::*;
pub use metrics::*;
pub use slo::*;
