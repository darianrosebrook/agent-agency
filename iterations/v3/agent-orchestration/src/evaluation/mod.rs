//! Evaluation Framework Module
//!
//! This module provides comprehensive evaluation capabilities for agent orchestration,
//! including scenario-based testing, multi-dimensional metrics, and operator-quality reporting.
//!
//! @author @darianrosebrook

pub mod framework;
pub mod trace;
pub mod determinism;
pub mod query;
pub mod metrics;
pub mod contracts;
pub mod scenario_runner;
pub mod playground;
pub mod sinks;
pub mod reporters;

#[cfg(test)]
mod integration_test;

#[cfg(test)]
mod property_tests;

#[cfg(test)]
mod success_criteria;

pub use framework::{EvaluationEngine, AgentEvaluation, EvaluationDimensions, EvaluationScenario};
pub use scenario_runner::run_scenario;
pub use framework::EvaluationReport;
